use ddnsupd_core::{decrypt, public_ip};
use serde_json::{Value, json};
use sqlx::{PgConnection, PgPool, Row};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Clone)]
pub struct Worker {
    pub pool: PgPool,
    pub key: [u8; 32],
    pub engine_url: String,
    pub engine_token: String,
    pub client: reqwest::Client,
}

impl Worker {
    pub async fn run(self) {
        loop {
            if let Err(_error) = self.tick().await {
                tracing::warn!("scheduler iteration failed");
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    async fn tick(&self) -> anyhow::Result<()> {
        sqlx::query("UPDATE update_jobs SET state='queued',lease_until=NULL WHERE state='running' AND lease_until<now()") .execute(&self.pool).await?;
        sqlx::query("INSERT INTO update_jobs(id,record_id,trigger) SELECT gen_random_uuid(),r.id,'automatic' FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id JOIN providers p ON p.id=z.provider_id WHERE r.active AND p.enabled AND NOT p.suspended AND (p.blocked_until IS NULL OR p.blocked_until<=now()) AND r.next_check<=now() ON CONFLICT DO NOTHING").execute(&self.pool).await?;
        let job=sqlx::query("UPDATE update_jobs SET state='running',lease_until=now()+interval '2 minutes',attempts=attempts+1 WHERE id=(SELECT id FROM update_jobs WHERE state='queued' AND due_at<=now() ORDER BY due_at FOR UPDATE SKIP LOCKED LIMIT 1) RETURNING id,record_id,trigger,attempts").fetch_optional(&self.pool).await?;
        if let Some(job) = job {
            let job_id: Uuid = job.get("id");
            if self
                .process(
                    job_id,
                    job.get("record_id"),
                    job.get("trigger"),
                    job.get("attempts"),
                )
                .await
                .is_err()
            {
                tracing::warn!(%job_id,"job interrupted; lease recovery will reconcile provider state");
            }
        }
        Ok(())
    }
    async fn process(
        &self,
        job_id: Uuid,
        record_id: Uuid,
        trigger: String,
        attempts: i32,
    ) -> anyhow::Result<()> {
        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock_shared(784312)")
            .execute(&mut *tx)
            .await?;
        let claimed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM update_jobs WHERE id=$1 AND state='running' AND attempts=$2)",
        )
        .bind(job_id)
        .bind(attempts)
        .fetch_one(&mut *tx)
        .await?;
        if !claimed {
            return Ok(());
        }
        // A provider-wide transaction lock serializes account and RRSet changes.
        // Network operations are bounded; heartbeat cancels the request on DB loss.
        let binding=sqlx::query("SELECT z.provider_id,p.kind,z.name FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id JOIN providers p ON p.id=z.provider_id WHERE r.id=$1").bind(record_id).fetch_optional(&mut *tx).await?;
        let Some(binding) = binding else {
            return Ok(());
        };
        let provider_id: Uuid = binding.get("provider_id");
        let zone_lock = format!(
            "{}:{}",
            binding.get::<String, _>("kind"),
            binding.get::<String, _>("name")
        );
        let locked: bool =
            sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtextextended($1,0))")
                .bind(zone_lock)
                .fetch_one(&mut *tx)
                .await?;
        if !locked {
            sqlx::query("UPDATE update_jobs SET state='queued',due_at=now()+interval '5 seconds' WHERE id=$1").bind(job_id).execute(&mut *tx).await?;
            tx.commit().await?;
            return Ok(());
        }
        let record=sqlx::query("SELECT r.*,z.provider_id,z.remote_id AS zone_remote_id,z.name AS zone_name,p.kind,p.name AS provider_name,p.enabled,p.suspended,coalesce(p.blocked_until>now(),false) AS blocked,c.ciphertext FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id JOIN providers p ON p.id=z.provider_id JOIN provider_credentials c ON c.provider_id=p.id WHERE r.id=$1 FOR UPDATE OF r,p").bind(record_id).fetch_one(&mut *tx).await?;
        let name: String = record.get("name");
        let zone: String = record.get("zone_name");
        let fqdn = if name == "@" {
            zone.clone()
        } else {
            format!("{name}.{zone}")
        };
        let start = Instant::now();
        let kind: String = record.get("record_type");
        let interval: i32 = record.get("interval_seconds");
        let managed: String = record.get("managed_ip");
        let ready = record.get::<bool, _>("active")
            && record.get::<bool, _>("enabled")
            && !record.get::<bool, _>("suspended")
            && !record.get::<bool, _>("blocked");
        let due: bool = sqlx::query_scalar("SELECT next_check<=now() FROM dns_records WHERE id=$1")
            .bind(record_id)
            .fetch_one(&mut *tx)
            .await?;
        let mut desired = record.get::<String, _>("manual_ip");
        let mut code = None;
        let mut outcome = "skipped";
        let mut result = json!({});
        let create_missing = trigger == "create_missing";
        if ready && due {
            if desired.is_empty() {
                match self.discover(&kind).await {
                    Ok(ip) => desired = ip,
                    Err(_) => code = Some("ip_unavailable".to_string()),
                }
            }
            if code.is_none() && !public_ip(&desired, &kind) {
                code = Some("invalid_ip".to_string())
            }
            if code.is_none() {
                let secret = decrypt(
                    &self.key,
                    provider_id,
                    &record.get::<Vec<u8>, _>("ciphertext"),
                )?;
                let mut payload = json!({"provider":record.get::<String,_>("kind"),"token":secret,"zone_id":record.get::<String,_>("zone_remote_id"),"zone_name":zone,"name":name,"record_id":record.get::<String,_>("remote_record_id"),"record_type":kind,"desired":desired,"managed":managed,"dry_run":false});
                if create_missing {
                    payload["create_missing"] = json!(true);
                }
                let request = self
                    .client
                    .post(format!("{}/v1/reconcile", self.engine_url))
                    .bearer_auth(&self.engine_token)
                    .json(&payload)
                    .send();
                let response = tokio::select! {response=request=>response, _=heartbeat(&mut tx)=>return Err(anyhow::anyhow!("lock connection lost"))};
                match response {
                    Ok(response) => {
                        let successful = response.status().is_success();
                        result = response
                            .json::<Value>()
                            .await
                            .unwrap_or(json!({"code":"uncertain"}));
                        if successful && result["verified"] == true {
                            outcome = if result["changed"] == true {
                                "updated"
                            } else {
                                "no_change"
                            };
                        } else {
                            code = Some(
                                result["code"]
                                    .as_str()
                                    .filter(|v| v.len() < 40)
                                    .unwrap_or("uncertain")
                                    .to_string(),
                            );
                        }
                    }
                    Err(_) => code = Some("uncertain".to_string()),
                }
            }
            if code.is_some() {
                outcome = "failed";
            }
        }
        let failures = record.get::<i32, _>("consecutive_failures");
        let delay = if code.is_some() {
            (30_i32.saturating_mul(2_i32.saturating_pow((failures + attempts).min(7) as u32)))
                .clamp(60, 3600)
                .max(interval)
        } else {
            interval
        };
        let retry_after = result["retry_after"].as_i64().unwrap_or(0).clamp(0, 86400) as i32;
        let jitter = (Uuid::new_v4().as_u128() & 15) as i32;
        let delay = if code.is_some() {
            delay.saturating_add(jitter).min(86400)
        } else {
            delay
        }
        .max(retry_after);
        if matches!(code.as_deref(), Some("auth" | "permission")) {
            sqlx::query("UPDATE providers SET suspended=true WHERE id=$1")
                .bind(provider_id)
                .execute(&mut *tx)
                .await?;
        }
        if code.as_deref() == Some("rate_limit") {
            sqlx::query(
                "UPDATE providers SET blocked_until=now()+make_interval(secs=>$2) WHERE id=$1",
            )
            .bind(provider_id)
            .bind(f64::from(delay))
            .execute(&mut *tx)
            .await?;
        }
        if ready && due {
            sqlx::query("UPDATE dns_records SET consecutive_failures=CASE WHEN $2 THEN consecutive_failures+1 ELSE 0 END WHERE id=$1").bind(record_id).bind(code.is_some()).execute(&mut *tx).await?;
            sqlx::query("UPDATE dns_records SET status=$2,last_check=now(),next_check=now()+make_interval(secs=>$3),managed_ip=CASE WHEN $4 THEN $5 ELSE managed_ip END,last_update=CASE WHEN $6 THEN now() ELSE last_update END WHERE id=$1").bind(record_id).bind(outcome).bind(f64::from(delay)).bind(outcome=="updated"||outcome=="no_change").bind(&desired).bind(outcome=="updated").execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO update_history(id,record_id,fqdn,provider_name,previous_ip,new_ip,trigger,outcome,duration_ms,error_code) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)").bind(Uuid::new_v4()).bind(record_id).bind(fqdn).bind(record.get::<String,_>("provider_name")).bind(result["before"].as_str().unwrap_or(&managed)).bind(if code.is_none(){desired.as_str()}else{""}).bind(trigger).bind(outcome).bind(start.elapsed().as_millis() as i64).bind(code).execute(&mut *tx).await?;
        sqlx::query("UPDATE update_jobs SET state='done',lease_until=NULL WHERE id=$1")
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
        if outcome == "updated" {
            sqlx::query("INSERT INTO audit_logs(id,action,object_id) VALUES($1,$2,$3)")
                .bind(Uuid::new_v4())
                .bind(if create_missing {
                    "dns.created"
                } else {
                    "dns.updated"
                })
                .bind(record_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
    async fn discover(&self, kind: &str) -> anyhow::Result<String> {
        let urls = if kind == "A" {
            ["https://api4.ipify.org", "https://ipv4.icanhazip.com"]
        } else {
            ["https://api6.ipify.org", "https://ipv6.icanhazip.com"]
        };
        for url in urls {
            let Ok(mut response) = self
                .client
                .get(url)
                .timeout(Duration::from_secs(5))
                .send()
                .await
            else {
                continue;
            };
            if !response.status().is_success() {
                continue;
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await? {
                if bytes.len() + chunk.len() > 128 {
                    bytes.clear();
                    break;
                }
                bytes.extend_from_slice(&chunk);
            }
            if let Ok(ip) = std::str::from_utf8(&bytes) {
                let ip = ip.trim();
                if public_ip(ip, kind) {
                    return Ok(ip.to_string());
                }
            }
        }
        anyhow::bail!("no valid public IP")
    }
}

async fn heartbeat(connection: &mut PgConnection) {
    loop {
        if sqlx::query("SELECT 1")
            .execute(&mut *connection)
            .await
            .is_err()
        {
            return;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

use sqlx::Connection;

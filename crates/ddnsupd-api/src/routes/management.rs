use super::{ApiError, Result, User, auth, bad, forbidden, internal, log, remote_valid};
use crate::State;
use axum::{
    Json, Router,
    extract::{Path, State as AppState},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use ddnsupd_core::{
    config_import::{self, ImportOptions, ImportPlan, RemoteRecord},
    decrypt, digest,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use std::{collections::HashSet, sync::Arc, time::Duration};
use uuid::Uuid;

pub(super) fn router() -> Router<Arc<State>> {
    Router::new()
        .route("/providers/{id}/discover", post(discovery))
        .route("/imports/preview", post(preview))
        .route("/imports/{id}/confirm", post(confirm))
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct DiscoveryInput {
    operation: String,
    #[serde(default)]
    zone_id: String,
    #[serde(default)]
    zone_name: String,
    #[serde(default = "first_page")]
    page: i32,
}
fn first_page() -> i32 {
    1
}
#[derive(Deserialize, Serialize)]
struct Zone {
    id: String,
    name: String,
    mode: String,
}
#[derive(Deserialize, Serialize)]
struct DiscoveryData {
    zones: Vec<Zone>,
    records: Vec<RemoteRecord>,
    page: i32,
    next_page: Option<i32>,
    permission_verification: String,
}
struct ProviderAccess {
    id: Uuid,
    kind: String,
    version: i64,
    secret: String,
}

async fn provider(s: &State, u: &User, id: Uuid) -> Result<ProviderAccess> {
    let row=sqlx::query("SELECT p.kind,p.credential_version,c.ciphertext,coalesce(p.blocked_until>now(),false) AS blocked FROM providers p JOIN provider_credentials c ON c.provider_id=p.id WHERE p.id=$1 AND ($3 OR EXISTS(SELECT 1 FROM provider_permissions pp WHERE pp.provider_id=p.id AND pp.user_id=$2))").bind(id).bind(u.id).bind(u.admin).fetch_optional(&s.pool).await?.ok_or(ApiError(StatusCode::NOT_FOUND,"not_found"))?;
    if row.get::<bool, _>("blocked") {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "provider_rate_limit",
        ));
    }
    let secret =
        decrypt(&s.key, id, &row.get::<Vec<u8>, _>("ciphertext")).map_err(|_| internal())?;
    Ok(ProviderAccess {
        id,
        kind: row.get("kind"),
        version: row.get("credential_version"),
        secret,
    })
}
async fn engine(s: &State, p: &ProviderAccess, input: &DiscoveryInput) -> Result<DiscoveryData> {
    if !matches!(input.operation.as_str(), "zones" | "records")
        || !(1..=10000).contains(&input.page)
    {
        return Err(bad());
    }
    if input.operation == "records"
        && (!remote_valid(&input.zone_id) || !ddnsupd_core::valid_name(&input.zone_name, false))
    {
        return Err(bad());
    }
    let requests:i32=sqlx::query_scalar("INSERT INTO provider_discovery_limits(provider_id) VALUES($1) ON CONFLICT(provider_id) DO UPDATE SET requests=CASE WHEN provider_discovery_limits.window_start<now()-interval '1 minute' THEN 1 ELSE provider_discovery_limits.requests+1 END,window_start=CASE WHEN provider_discovery_limits.window_start<now()-interval '1 minute' THEN now() ELSE provider_discovery_limits.window_start END RETURNING requests").bind(p.id).fetch_one(&s.pool).await?;
    if requests > 30 {
        return Err(ApiError(StatusCode::TOO_MANY_REQUESTS, "rate_limited"));
    }
    let payload = json!({"provider":p.kind,"token":p.secret,"operation":input.operation,"zone_id":input.zone_id,"zone_name":input.zone_name,"page":input.page});
    let mut response = s
        .client
        .post(format!("{}/v1/discover", s.engine_url))
        .bearer_auth(&s.engine_token)
        .json(&payload)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|_| ApiError(StatusCode::BAD_GATEWAY, "provider_unavailable"))?;
    let success = response.status().is_success();
    let mut bytes = vec![];
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| ApiError(StatusCode::BAD_GATEWAY, "provider_unavailable"))?
    {
        if bytes.len() + chunk.len() > 512 * 1024 {
            return Err(ApiError(StatusCode::BAD_GATEWAY, "provider_unavailable"));
        }
        bytes.extend_from_slice(&chunk);
    }
    if !success {
        let error: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        let code = match error["code"].as_str() {
            Some("auth") => "provider_auth",
            Some("permission") => "provider_permission",
            Some("rate_limit") => "provider_rate_limit",
            Some("conflict" | "not_found") => "provider_conflict",
            _ => "provider_unavailable",
        };
        if code == "provider_rate_limit" {
            let seconds = error["retry_after"].as_i64().unwrap_or(300).clamp(1, 86400) as f64;
            sqlx::query("UPDATE providers SET blocked_until=greatest(coalesce(blocked_until,now()),now()+make_interval(secs=>$2)) WHERE id=$1").bind(p.id).bind(seconds).execute(&s.pool).await?;
            return Err(ApiError(StatusCode::TOO_MANY_REQUESTS, code));
        }
        return Err(ApiError(StatusCode::UNPROCESSABLE_ENTITY, code));
    }
    let data: DiscoveryData = serde_json::from_slice(&bytes)
        .map_err(|_| ApiError(StatusCode::BAD_GATEWAY, "provider_unavailable"))?;
    if data.page != input.page
        || data.next_page.is_some_and(|page| page != input.page + 1)
        || data.records.len() > 1000
        || data.zones.len() > 50
    {
        return Err(ApiError(StatusCode::BAD_GATEWAY, "provider_unavailable"));
    }
    Ok(data)
}
async fn discovery(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<DiscoveryInput>,
) -> Result<Json<DiscoveryData>> {
    let u = auth(&s, &h, "providers.read", true).await?;
    let p = provider(&s, &u, id).await?;
    let data = engine(&s, &p, &input).await?;
    let mut tx = s.pool.begin().await?;
    log(&mut tx, u.id, "providers.discover", Some(id)).await?;
    tx.commit().await?;
    Ok(Json(data))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviewInput {
    provider_id: Uuid,
    zone_id: String,
    zone_name: String,
    config: Value,
    ambiguous_family: Option<String>,
    #[serde(default = "default_interval")]
    interval_seconds: i32,
}
fn default_interval() -> i32 {
    300
}
async fn preview(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(input): Json<PreviewInput>,
) -> Result<(StatusCode, Json<Value>)> {
    let u = auth(&s, &h, "records.create", true).await?;
    if !u.admin {
        return Err(forbidden());
    }
    // Validate the source before any outbound request; parsing never executes options.
    let p = provider(&s, &u, input.provider_id).await?;
    let options = ImportOptions {
        provider_kind: &p.kind,
        zone_id: &input.zone_id,
        zone_name: &input.zone_name,
        ambiguous_family: input.ambiguous_family.as_deref(),
        interval_seconds: input.interval_seconds,
    };
    config_import::analyze(&input.config, &options, &[]).map_err(|_| bad())?;
    if !remote_valid(&input.zone_id) {
        return Err(bad());
    }
    let fetch = async {
        let mut records = vec![];
        let mut page = 1;
        for _ in 0..20 {
            let data = engine(
                &s,
                &p,
                &DiscoveryInput {
                    operation: "records".into(),
                    zone_id: input.zone_id.clone(),
                    zone_name: input.zone_name.clone(),
                    page,
                },
            )
            .await?;
            records.extend(data.records);
            if records.len() > 2000 {
                return Err(ApiError(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "discovery_limit",
                ));
            }
            match data.next_page {
                Some(next) => page = next,
                None => return Ok(records),
            }
        }
        Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "discovery_limit",
        ))
    };
    let remote = tokio::time::timeout(Duration::from_secs(30), fetch)
        .await
        .map_err(|_| ApiError(StatusCode::GATEWAY_TIMEOUT, "provider_unavailable"))??;
    let mut plan = config_import::analyze(&input.config, &options, &remote).map_err(|_| bad())?;
    let existing=sqlx::query("SELECT r.name,r.record_type FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id WHERE z.provider_id=$1 AND z.remote_id=$2").bind(p.id).bind(&input.zone_id).fetch_all(&s.pool).await?;
    let existing: HashSet<(String, String)> = existing
        .iter()
        .map(|row| (row.get("name"), row.get("record_type")))
        .collect();
    for item in &mut plan.items {
        if existing.contains(&(item.name.clone(), item.record_type.clone())) {
            item.errors.push("already_managed".into())
        }
    }
    let value = serde_json::to_value(plan).map_err(|_| internal())?;
    let hash = digest(&value.to_string());
    let id = Uuid::new_v4();
    let mut tx = s.pool.begin().await?;
    sqlx::query("DELETE FROM import_previews WHERE expires_at<now()-interval '1 day'")
        .execute(&mut *tx)
        .await?;
    let expires:chrono::DateTime<chrono::Utc>=sqlx::query_scalar("INSERT INTO import_previews(id,user_id,provider_id,credential_version,plan,plan_digest) VALUES($1,$2,$3,$4,$5,$6) RETURNING expires_at").bind(id).bind(u.id).bind(p.id).bind(p.version).bind(&value).bind(&hash).fetch_one(&mut *tx).await?;
    log(&mut tx, u.id, "imports.preview", Some(id)).await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"id":id,"digest":hash,"expires_at":expires,"plan":value})),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfirmInput {
    digest: String,
    selected_indices: Vec<usize>,
    accept_warnings: bool,
}
async fn confirm(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<ConfirmInput>,
) -> Result<(StatusCode, Json<Value>)> {
    let u = auth(&s, &h, "records.create", true).await?;
    if !u.admin {
        return Err(forbidden());
    }
    let selected: HashSet<usize> = input.selected_indices.iter().copied().collect();
    if !input.accept_warnings
        || selected.is_empty()
        || selected.len() != input.selected_indices.len()
        || selected.len() > 200
    {
        return Err(bad());
    }
    let mut tx = s.pool.begin().await?;
    let draft=sqlx::query("SELECT *,expires_at<=now() AS expired FROM import_previews WHERE id=$1 AND user_id=$2 FOR UPDATE").bind(id).bind(u.id).fetch_optional(&mut *tx).await?.ok_or(ApiError(StatusCode::NOT_FOUND,"not_found"))?;
    if draft
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("consumed_at")
        .is_some()
    {
        return Err(ApiError(StatusCode::CONFLICT, "preview_consumed"));
    }
    if draft.get::<bool, _>("expired") {
        return Err(ApiError(StatusCode::CONFLICT, "preview_expired"));
    }
    if digest(&input.digest) != digest(&draft.get::<String, _>("plan_digest")) {
        return Err(ApiError(StatusCode::CONFLICT, "preview_changed"));
    }
    let provider_id: Uuid = draft.get("provider_id");
    let version: i64 =
        sqlx::query_scalar("SELECT credential_version FROM providers WHERE id=$1 FOR UPDATE")
            .bind(provider_id)
            .fetch_one(&mut *tx)
            .await?;
    if version != draft.get::<i64, _>("credential_version") {
        return Err(ApiError(StatusCode::CONFLICT, "preview_changed"));
    }
    let expired: bool =
        sqlx::query_scalar("SELECT expires_at<=clock_timestamp() FROM import_previews WHERE id=$1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if expired {
        return Err(ApiError(StatusCode::CONFLICT, "preview_expired"));
    }
    let plan: ImportPlan = serde_json::from_value(draft.get("plan")).map_err(|_| internal())?;
    if selected.iter().any(|index| {
        plan.items
            .get(*index)
            .is_none_or(|item| !item.errors.is_empty())
    }) {
        return Err(bad());
    }
    let zone=sqlx::query("INSERT INTO dns_zones(id,provider_id,remote_id,name) VALUES($1,$2,$3,$4) ON CONFLICT(provider_id,remote_id) DO UPDATE SET remote_id=EXCLUDED.remote_id RETURNING id,name").bind(Uuid::new_v4()).bind(provider_id).bind(&plan.zone_id).bind(&plan.zone_name).fetch_one(&mut *tx).await?;
    if zone.get::<String, _>("name") != plan.zone_name {
        return Err(ApiError(StatusCode::CONFLICT, "zone_mismatch"));
    }
    let mut created = vec![];
    for (index, item) in plan.items.iter().enumerate() {
        if !selected.contains(&index) {
            continue;
        }
        let record_id = Uuid::new_v4();
        sqlx::query("INSERT INTO dns_records(id,zone_id,name,record_type,remote_record_id,managed_ip,active,interval_seconds,owner_id) VALUES($1,$2,$3,$4,$5,$6,false,$7,$8)").bind(record_id).bind(zone.get::<Uuid,_>("id")).bind(&item.name).bind(&item.record_type).bind(&item.record_id).bind(&item.managed_ip).bind(item.interval_seconds).bind(u.id).execute(&mut *tx).await?;
        log(&mut tx, u.id, "records.import", Some(record_id)).await?;
        created.push(json!({"index":index,"id":record_id}));
    }
    sqlx::query("UPDATE import_previews SET consumed_at=now() WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    log(&mut tx, u.id, "imports.confirm", Some(id)).await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"created":created,"active":false})),
    ))
}

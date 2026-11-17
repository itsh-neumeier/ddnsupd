use super::{ApiError, Result, auth, authorize_setup, bad, forbidden, internal, lock_empty_setup};
use crate::State;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, FromRequest, Request, State as AppState},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::post,
};
use chrono::{Datelike, Utc};
use ddnsupd_core::{backup, decrypt, encrypt};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use std::path::PathBuf;
use std::time::Duration;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

const MAX_ARCHIVE: usize = 16 * 1024 * 1024;
const TABLES: &[&str] = &[
    "roles",
    "permissions",
    "role_permissions",
    "users",
    "providers",
    "provider_credentials",
    "provider_permissions",
    "dns_zones",
    "dns_records",
    "record_permissions",
    "update_history",
    "audit_logs",
];
const TRANSIENT: &[&str] = &[
    "user_sessions",
    "login_attempts",
    "update_jobs",
    "import_previews",
    "provider_discovery_limits",
];
static MAINTENANCE: RwLock<()> = RwLock::const_new(());
static BACKUP_OPERATION: Semaphore = Semaphore::const_new(1);

pub(super) async fn while_available(request: Request, next: Next) -> Response {
    let Ok(_guard) = MAINTENANCE.try_read() else {
        return ApiError(StatusCode::SERVICE_UNAVAILABLE, "backup_busy").into_response();
    };
    next.run(request).await
}

pub(super) fn router() -> Router<Arc<State>> {
    Router::new()
        .route("/backup", post(export))
        .route(
            "/backup/restore",
            post(restore).layer(DefaultBodyLimit::max(MAX_ARCHIVE * 2 + 4096)),
        )
        .route(
            "/setup/restore",
            post(restore_setup).layer(DefaultBodyLimit::max(MAX_ARCHIVE * 2 + 4096)),
        )
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema: i64,
    source_key: [u8; 32],
    tables: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Passphrase {
    passphrase: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Restore {
    passphrase: String,
    archive: String,
    confirm: bool,
}

fn invalid_archive() -> ApiError {
    ApiError(StatusCode::UNPROCESSABLE_ENTITY, "backup_invalid")
}

fn check_passphrase(passphrase: &str) -> Result<()> {
    if passphrase.chars().count() < 12 || passphrase.len() > 1024 {
        return Err(bad());
    }
    Ok(())
}

fn operation() -> Result<tokio::sync::SemaphorePermit<'static>> {
    BACKUP_OPERATION
        .try_acquire()
        .map_err(|_| ApiError(StatusCode::TOO_MANY_REQUESTS, "backup_busy"))
}

async fn export(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(input): Json<Passphrase>,
) -> Result<Response> {
    let _operation = operation()?;
    let _guard = MAINTENANCE.read().await;
    let user = auth(&s, &h, "backup.create", true).await?;
    if !user.admin {
        return Err(forbidden());
    }
    let archive = export_snapshot(&s, input.passphrase, Some(user.id)).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"ddnsupd.ddnsbackup\"",
            ),
        ],
        archive,
    )
        .into_response())
}

async fn export_snapshot(s: &State, passphrase: String, actor: Option<Uuid>) -> Result<Vec<u8>> {
    check_passphrase(&passphrase)?;
    let mut tx = s.pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *tx)
        .await?;
    let schema = sqlx::query_scalar("SELECT max(version) FROM _sqlx_migrations WHERE success")
        .fetch_one(&mut *tx)
        .await?;
    let mut tables = BTreeMap::new();
    let mut size = 0;
    for table in TABLES {
        let query = format!("SELECT to_jsonb(row) FROM {table} row");
        let mut rows = sqlx::query(&query).fetch(&mut *tx);
        let mut values = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let value: Value = row.try_get(0)?;
            size += value.to_string().len() + 1;
            if size > MAX_ARCHIVE - 4096 {
                return Err(ApiError(StatusCode::PAYLOAD_TOO_LARGE, "backup_too_large"));
            }
            values.push(value);
        }
        tables.insert((*table).to_owned(), Value::Array(values));
    }
    let snapshot = Snapshot {
        schema,
        source_key: s.key,
        tables,
    };
    let archive = tokio::task::spawn_blocking(move || {
        let plaintext = serde_json::to_vec(&snapshot).map_err(|_| internal())?;
        backup::seal(&passphrase, &plaintext).map_err(|_| internal())
    })
    .await
    .map_err(|_| internal())??;
    sqlx::query("INSERT INTO audit_logs(id,user_id,action) VALUES ($1,$2,'backup.export')")
        .bind(Uuid::new_v4())
        .bind(actor)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(archive)
}

pub(crate) fn start_schedule(s: Arc<State>) -> anyhow::Result<()> {
    let dir = std::env::var_os("DDNSUPD_BACKUP_DIR").filter(|value| !value.is_empty());
    let secret =
        std::env::var_os("DDNSUPD_BACKUP_PASSPHRASE_FILE").filter(|value| !value.is_empty());
    match (dir, secret) {
        (None, None) => Ok(()),
        (Some(dir), Some(secret)) => {
            tokio::spawn(schedule(s, dir.into(), secret.into()));
            Ok(())
        }
        _ => anyhow::bail!("Set both DDNSUPD_BACKUP_DIR and DDNSUPD_BACKUP_PASSPHRASE_FILE"),
    }
}

async fn schedule(s: Arc<State>, dir: PathBuf, secret: PathBuf) {
    let mut timer = tokio::time::interval(Duration::from_secs(3600));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        timer.tick().await;
        let today = Utc::now().date_naive();
        let path = dir.join(format!(
            "daily-{}.ddnsbackup",
            today.weekday().num_days_from_monday()
        ));
        if tokio::fs::metadata(&path)
            .await
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .is_some_and(|modified| chrono::DateTime::<Utc>::from(modified).date_naive() == today)
        {
            continue;
        }
        let Ok(_operation) = operation() else {
            continue;
        };
        let _guard = MAINTENANCE.read().await;
        let initialized = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE active AND role_id=$1::uuid)",
        )
        .bind(super::ADMIN)
        .fetch_one(&s.pool)
        .await;
        match initialized {
            Ok(false) => continue,
            Err(_) => {
                tracing::warn!("scheduled backup could not check setup status");
                continue;
            }
            Ok(true) => {}
        }
        let result = async {
            let file = tokio::fs::read_to_string(&secret)
                .await
                .map_err(|_| internal())?;
            let archive =
                export_snapshot(&s, file.trim_end_matches(['\r', '\n']).to_owned(), None).await?;
            let temporary = path.with_extension("ddnsbackup.tmp");
            tokio::fs::write(&temporary, archive)
                .await
                .map_err(|_| internal())?;
            tokio::fs::rename(temporary, &path)
                .await
                .map_err(|_| internal())?;
            Ok::<(), ApiError>(())
        }
        .await;
        if let Err(error) = result {
            tracing::warn!(code = error.1, "scheduled backup failed");
        }
    }
}

async fn restore(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    request: Request,
) -> Result<Json<Value>> {
    let user = auth(&s, &h, "backup.restore", true).await?;
    if !user.admin {
        return Err(forbidden());
    }
    import(&s, &h, restore_input(request).await?, Some(user.id)).await
}

async fn restore_setup(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    request: Request,
) -> Result<Json<Value>> {
    authorize_setup(&s, &h)?;
    if !setup_needed(&s).await? {
        return Err(ApiError(StatusCode::CONFLICT, "setup_complete"));
    }
    import(&s, &h, restore_input(request).await?, None).await
}

async fn restore_input(request: Request) -> Result<Restore> {
    Json::<Restore>::from_request(request, &())
        .await
        .map(|Json(input)| input)
        .map_err(|error| ApiError(error.status(), "invalid_input"))
}

async fn setup_needed(s: &State) -> Result<bool> {
    Ok(sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM users)")
        .fetch_one(&s.pool)
        .await?)
}

async fn import(
    s: &State,
    h: &HeaderMap,
    input: Restore,
    actor: Option<Uuid>,
) -> Result<Json<Value>> {
    check_passphrase(&input.passphrase)?;
    if !input.confirm || input.archive.len() > MAX_ARCHIVE * 2 {
        return Err(bad());
    }
    let _operation = operation()?;
    let mut snapshot = tokio::task::spawn_blocking(move || {
        let archive = hex::decode(input.archive).map_err(|_| invalid_archive())?;
        let plaintext = backup::open(&input.passphrase, &archive).map_err(|_| invalid_archive())?;
        serde_json::from_slice::<Snapshot>(&plaintext).map_err(|_| invalid_archive())
    })
    .await
    .map_err(|_| internal())??;
    if snapshot.tables.len() != TABLES.len()
        || TABLES
            .iter()
            .any(|table| !snapshot.tables.get(*table).is_some_and(Value::is_array))
    {
        return Err(invalid_archive());
    }
    let _guard = MAINTENANCE.write().await;
    if actor.is_some() && !auth(s, h, "backup.restore", true).await?.admin {
        return Err(forbidden());
    }
    let mut tx = s.pool.begin().await?;
    sqlx::query("SET LOCAL lock_timeout = '45s'")
        .execute(&mut *tx)
        .await?;
    if actor.is_none() {
        lock_empty_setup(&mut tx).await?;
    }
    let schema: i64 = sqlx::query_scalar("SELECT max(version) FROM _sqlx_migrations WHERE success")
        .fetch_one(&mut *tx)
        .await?;
    if snapshot.schema != schema {
        return Err(ApiError(StatusCode::CONFLICT, "backup_incompatible"));
    }
    sqlx::query("SELECT pg_advisory_xact_lock(784312)")
        .execute(&mut *tx)
        .await?;
    let tables = [TABLES, TRANSIENT].concat().join(", ");
    sqlx::query(&format!("TRUNCATE {tables}"))
        .execute(&mut *tx)
        .await?;
    for table in TABLES {
        let rows = snapshot.tables.remove(*table).ok_or_else(invalid_archive)?;
        let query = format!(
            "INSERT INTO {table} SELECT * FROM jsonb_populate_recordset(NULL::{table}, $1)"
        );
        sqlx::query(&query)
            .bind(rows)
            .execute(&mut *tx)
            .await
            .map_err(|_| invalid_archive())?;
    }
    let admin: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM users WHERE active AND role_id=$1::uuid ORDER BY id LIMIT 1",
    )
    .bind(super::ADMIN)
    .fetch_optional(&mut *tx)
    .await?;
    admin.ok_or_else(invalid_archive)?;
    let credentials: Vec<(Uuid, Vec<u8>)> =
        sqlx::query_as("SELECT provider_id,ciphertext FROM provider_credentials")
            .fetch_all(&mut *tx)
            .await?;
    for (provider, ciphertext) in credentials {
        let secret =
            decrypt(&snapshot.source_key, provider, &ciphertext).map_err(|_| invalid_archive())?;
        let rekeyed = encrypt(&s.key, provider, &secret).map_err(|_| internal())?;
        sqlx::query("UPDATE provider_credentials SET ciphertext=$1 WHERE provider_id=$2")
            .bind(rekeyed)
            .bind(provider)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("UPDATE dns_records SET active=false")
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO audit_logs(id,user_id,action) VALUES ($1,(SELECT id FROM users WHERE id=$2),'backup.restore')")
        .bind(Uuid::new_v4()).bind(actor).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"restored": true})))
}

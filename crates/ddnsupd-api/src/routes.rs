use crate::State;
use axum::{
    Json, Router,
    extract::{Path, Query, State as AppState},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use ddnsupd_core::{digest, encrypt, hash_password, public_ip, token, valid_name, verify_password};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Postgres, Row, Transaction};
use std::sync::Arc;
use uuid::Uuid;
mod backup;
pub(crate) use backup::start_schedule;
mod management;

const ADMIN: &str = "00000000-0000-0000-0000-000000000001";
const SCOPE: &str = "($2 OR r.owner_id=$1 OR EXISTS(SELECT 1 FROM record_permissions rp WHERE rp.record_id=r.id AND rp.user_id=$1) OR EXISTS(SELECT 1 FROM provider_permissions pp WHERE pp.provider_id=z.provider_id AND pp.user_id=$1))";
const RECORD_SELECT: &str = "SELECT r.*,z.provider_id,z.remote_id AS zone_remote_id,z.name AS zone_name,p.name AS provider_name,p.kind AS provider_kind,p.suspended AS provider_suspended FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id JOIN providers p ON p.id=z.provider_id";

pub struct ApiError(StatusCode, &'static str);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.0,
            Json(json!({"code":self.1,"request_id":Uuid::new_v4()})),
        )
            .into_response()
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db) = error
            && matches!(db.code().as_deref(), Some("23505" | "23503"))
        {
            return Self(StatusCode::CONFLICT, "conflict");
        }
        Self(StatusCode::INTERNAL_SERVER_ERROR, "internal")
    }
}
type Result<T> = std::result::Result<T, ApiError>;
fn bad() -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, "invalid_input")
}
fn forbidden() -> ApiError {
    ApiError(StatusCode::FORBIDDEN, "forbidden")
}
fn internal() -> ApiError {
    ApiError(StatusCode::INTERNAL_SERVER_ERROR, "internal")
}

pub fn router() -> Router<Arc<State>> {
    Router::new()
        .merge(management::router())
        .route("/setup", get(setup_status).post(setup))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
        .route("/auth/password", post(change_password))
        .route("/providers", get(providers).post(create_provider))
        .route(
            "/providers/{id}",
            axum::routing::patch(edit_provider).delete(delete_provider),
        )
        .route("/provider-registry", get(registry))
        .route("/dashboard", get(dashboard))
        .route("/records", get(records).post(create_record))
        .route(
            "/records/{id}",
            axum::routing::patch(edit_record).delete(delete_record),
        )
        .route("/records/{id}/update", post(update_record))
        .route("/records/{id}/create-missing", post(create_missing_record))
        .route("/history", get(history))
        .route("/users", get(users).post(create_user))
        .route("/roles", get(roles).post(create_role))
        .route("/grants", post(grant))
        .route("/audit", get(audit))
        .route("/openapi.json", get(openapi))
        .fallback(|| async { ApiError(StatusCode::NOT_FOUND, "not_found") })
        .layer(axum::middleware::from_fn(backup::while_available))
        .merge(backup::router())
}
pub async fn ready(AppState(s): AppState<Arc<State>>) -> StatusCode {
    if sqlx::query("SELECT 1").execute(&s.pool).await.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
fn origin(s: &State, h: &HeaderMap) -> Result<()> {
    if h.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(&s.origin) {
        return Err(forbidden());
    }
    Ok(())
}
fn cookie_hash(h: &HeaderMap) -> Result<String> {
    let value = h
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|c| {
            c.split(';')
                .find_map(|p| p.trim().strip_prefix("ddnsupd_session="))
        })
        .ok_or(ApiError(StatusCode::UNAUTHORIZED, "unauthorized"))?;
    if value.len() != 64 {
        return Err(ApiError(StatusCode::UNAUTHORIZED, "unauthorized"));
    }
    Ok(digest(value))
}
struct User {
    id: Uuid,
    admin: bool,
    csrf: String,
    username: String,
    permissions: Vec<String>,
}
async fn auth(s: &State, h: &HeaderMap, permission: &str, mutate: bool) -> Result<User> {
    let hash = cookie_hash(h)?;
    let row=sqlx::query("SELECT u.id,u.username,u.role_id,s.csrf_token,ARRAY(SELECT permission FROM role_permissions WHERE role_id=u.role_id) AS permissions FROM user_sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.expires_at>now() AND u.active").bind(hash).fetch_optional(&s.pool).await?.ok_or(ApiError(StatusCode::UNAUTHORIZED,"unauthorized"))?;
    let u = User {
        id: row.get("id"),
        admin: row.get::<Uuid, _>("role_id").to_string() == ADMIN,
        csrf: row.get("csrf_token"),
        username: row.get("username"),
        permissions: row.get("permissions"),
    };
    if !permission.is_empty() && !u.permissions.iter().any(|p| p == permission) {
        return Err(forbidden());
    }
    if mutate {
        origin(s, h)?;
        let supplied = h
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if digest(supplied) != digest(&u.csrf) {
            return Err(forbidden());
        }
    }
    Ok(u)
}
async fn log(
    tx: &mut Transaction<'_, Postgres>,
    user: Uuid,
    action: &str,
    object: Option<Uuid>,
) -> Result<()> {
    sqlx::query("INSERT INTO audit_logs(id,user_id,action,object_id) VALUES($1,$2,$3,$4)")
        .bind(Uuid::new_v4())
        .bind(user)
        .bind(action)
        .bind(object)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
fn credentials_valid(username: &str, password: &str) -> bool {
    (3..=80).contains(&username.len())
        && username
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-@".contains(&b))
        && (12..=256).contains(&password.len())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Login {
    username: String,
    password: String,
}
async fn setup_status(AppState(s): AppState<Arc<State>>) -> Result<Json<Value>> {
    let needed: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM users)")
        .fetch_one(&s.pool)
        .await?;
    Ok(Json(json!({"needs_setup":needed})))
}
async fn setup(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(input): Json<Login>,
) -> Result<Json<Value>> {
    authorize_setup(&s, &h)?;
    if !credentials_valid(&input.username, &input.password) {
        return Err(bad());
    }
    let hash = tokio::task::spawn_blocking(move || hash_password(&input.password))
        .await
        .map_err(|_| internal())?
        .map_err(|_| internal())?;
    let mut tx = s.pool.begin().await?;
    lock_empty_setup(&mut tx).await?;
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users(id,username,password_hash,role_id) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(input.username)
        .bind(hash)
        .bind(Uuid::parse_str(ADMIN).unwrap())
        .execute(&mut *tx)
        .await?;
    log(&mut tx, id, "setup.complete", Some(id)).await?;
    tx.commit().await?;
    Ok(Json(json!({"created":true})))
}

fn authorize_setup(s: &State, h: &HeaderMap) -> Result<()> {
    origin(s, h)?;
    if digest(
        h.get("x-setup-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
    ) != digest(&s.setup_token)
    {
        return Err(forbidden());
    }
    Ok(())
}

async fn lock_empty_setup(tx: &mut Transaction<'_, Postgres>) -> Result<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(314159)")
        .execute(&mut **tx)
        .await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users)")
        .fetch_one(&mut **tx)
        .await?;
    if exists {
        return Err(ApiError(StatusCode::CONFLICT, "setup_complete"));
    }
    Ok(())
}
async fn login(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(input): Json<Login>,
) -> Result<Response> {
    origin(&s, &h)?;
    if input.username.len() > 80 || input.password.len() > 256 {
        return Err(bad());
    }
    // Atomic counters bound attempts before Argon2 work. Do not trust proxy headers.
    sqlx::query("DELETE FROM login_attempts WHERE window_start<now()-interval '10 minutes'")
        .execute(&s.pool)
        .await?;
    for (name, limit) in [
        ("__global".to_string(), 40),
        (input.username.to_lowercase(), 10),
    ] {
        let attempts:i32=sqlx::query_scalar("INSERT INTO login_attempts(username) VALUES($1) ON CONFLICT(username) DO UPDATE SET attempts=CASE WHEN login_attempts.window_start<now()-interval '5 minutes' THEN 1 ELSE login_attempts.attempts+1 END,window_start=CASE WHEN login_attempts.window_start<now()-interval '5 minutes' THEN now() ELSE login_attempts.window_start END RETURNING attempts").bind(name).fetch_one(&s.pool).await?;
        if attempts > limit {
            return Err(ApiError(StatusCode::TOO_MANY_REQUESTS, "rate_limited"));
        }
    }
    let row = sqlx::query("SELECT id,password_hash,active FROM users WHERE username=$1")
        .bind(&input.username)
        .fetch_optional(&s.pool)
        .await?;
    let hash=row.as_ref().map(|r|r.get::<String,_>("password_hash")).unwrap_or_else(||"$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHR2YWx1ZQ$PE8XkzXEq8pPG3Mp0wyj31SA6B9Np3lKNrLfh7mIn7k".into());
    let valid = tokio::task::spawn_blocking(move || verify_password(&hash, &input.password))
        .await
        .map_err(|_| internal())?;
    let Some(row) = row.filter(|r| r.get::<bool, _>("active") && valid) else {
        return Err(ApiError(StatusCode::UNAUTHORIZED, "invalid_login"));
    };
    let user: Uuid = row.get("id");
    let raw = token();
    let csrf = token();
    let mut tx = s.pool.begin().await?;
    sqlx::query("DELETE FROM user_sessions WHERE expires_at<=now()")
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO user_sessions(token_hash,user_id,csrf_token,expires_at) VALUES($1,$2,$3,now()+interval '12 hours')").bind(digest(&raw)).bind(user).bind(&csrf).execute(&mut *tx).await?;
    log(&mut tx, user, "auth.login", Some(user)).await?;
    tx.commit().await?;
    let mut response = Json(json!({"csrf_token":csrf})).into_response();
    let cookie = format!(
        "ddnsupd_session={raw}; Path=/; HttpOnly; SameSite=Strict; Max-Age=43200{}",
        if s.secure_cookie { "; Secure" } else { "" }
    );
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| internal())?,
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}
async fn me(AppState(s): AppState<Arc<State>>, h: HeaderMap) -> Result<Json<Value>> {
    let u = auth(&s, &h, "", false).await?;
    Ok(Json(
        json!({"id":u.id,"username":u.username,"admin":u.admin,"permissions":u.permissions,"csrf_token":u.csrf}),
    ))
}
async fn logout(AppState(s): AppState<Arc<State>>, h: HeaderMap) -> Result<Response> {
    let u = auth(&s, &h, "", true).await?;
    let mut tx = s.pool.begin().await?;
    sqlx::query("DELETE FROM user_sessions WHERE token_hash=$1")
        .bind(cookie_hash(&h)?)
        .execute(&mut *tx)
        .await?;
    log(&mut tx, u.id, "auth.logout", Some(u.id)).await?;
    tx.commit().await?;
    let mut r = StatusCode::NO_CONTENT.into_response();
    r.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("ddnsupd_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0"),
    );
    Ok(r)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Password {
    current: String,
    password: String,
}
async fn change_password(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(i): Json<Password>,
) -> Result<StatusCode> {
    let u = auth(&s, &h, "", true).await?;
    if !(12..=256).contains(&i.password.len()) || i.current.len() > 256 {
        return Err(bad());
    }
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id=$1")
        .bind(u.id)
        .fetch_one(&s.pool)
        .await?;
    let new_hash = tokio::task::spawn_blocking(move || {
        if verify_password(&hash, &i.current) {
            hash_password(&i.password).ok()
        } else {
            None
        }
    })
    .await
    .map_err(|_| internal())?
    .ok_or(forbidden())?;
    let mut tx = s.pool.begin().await?;
    sqlx::query("UPDATE users SET password_hash=$2 WHERE id=$1")
        .bind(u.id)
        .bind(new_hash)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM user_sessions WHERE user_id=$1")
        .bind(u.id)
        .execute(&mut *tx)
        .await?;
    log(&mut tx, u.id, "auth.password", Some(u.id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn registry(AppState(s): AppState<Arc<State>>, h: HeaderMap) -> Result<Json<Value>> {
    auth(&s, &h, "providers.read", false).await?;
    serde_json::from_str(include_str!("../../../provider-engine/registry.json"))
        .map(Json)
        .map_err(|_| internal())
}
async fn providers(AppState(s): AppState<Arc<State>>, h: HeaderMap) -> Result<Json<Value>> {
    let u = auth(&s, &h, "providers.read", false).await?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(p)||jsonb_build_object('has_credentials',EXISTS(SELECT 1 FROM provider_credentials c WHERE c.provider_id=p.id)) FROM providers p WHERE $2 OR EXISTS(SELECT 1 FROM provider_permissions pp WHERE pp.provider_id=p.id AND pp.user_id=$1) ORDER BY p.name LIMIT 500").bind(u.id).bind(u.admin).fetch_all(&s.pool).await?;
    Ok(Json(json!({"items":rows})))
}
async fn dashboard(AppState(s): AppState<Arc<State>>, h: HeaderMap) -> Result<Json<Value>> {
    let u = auth(&s, &h, "dashboard.read", false).await?;
    let value:Value=sqlx::query_scalar(&format!("SELECT jsonb_build_object('total_records',count(*),'verified',count(*) FILTER(WHERE r.status IN ('updated','no_change')),'attention',count(*) FILTER(WHERE r.status='failed' OR p.suspended),'zones',count(DISTINCT z.id),'active',count(*) FILTER(WHERE r.active)) FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id JOIN providers p ON p.id=z.provider_id WHERE {SCOPE}")).bind(u.id).bind(u.admin).fetch_one(&s.pool).await?;
    Ok(Json(value))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderInput {
    kind: String,
    name: String,
    token: String,
}
async fn create_provider(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(i): Json<ProviderInput>,
) -> Result<(StatusCode, Json<Value>)> {
    let u = auth(&s, &h, "providers.manage", true).await?;
    if !u.admin {
        return Err(forbidden());
    }
    if !matches!(i.kind.as_str(), "hetznercloud" | "cloudflare")
        || i.name.trim().is_empty()
        || i.name.len() > 120
        || i.token.is_empty()
        || i.token.len() > 8192
    {
        return Err(bad());
    }
    let id = Uuid::new_v4();
    let ciphertext = encrypt(&s.key, id, &i.token).map_err(|_| internal())?;
    let mut tx = s.pool.begin().await?;
    sqlx::query("INSERT INTO providers(id,kind,name) VALUES($1,$2,$3)")
        .bind(id)
        .bind(i.kind)
        .bind(i.name)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO provider_credentials(provider_id,ciphertext) VALUES($1,$2)")
        .bind(id)
        .bind(ciphertext)
        .execute(&mut *tx)
        .await?;
    log(&mut tx, u.id, "providers.create", Some(id)).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"id":id}))))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderEdit {
    name: String,
    enabled: bool,
    token: Option<String>,
}
async fn edit_provider(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
    Json(i): Json<ProviderEdit>,
) -> Result<StatusCode> {
    let u = auth(&s, &h, "providers.manage", true).await?;
    if !u.admin {
        return Err(forbidden());
    }
    if i.name.trim().is_empty() || i.name.len() > 120 {
        return Err(bad());
    }
    let mut tx = s.pool.begin().await?;
    let changed = sqlx::query("UPDATE providers SET name=$2,enabled=$3 WHERE id=$1")
        .bind(id)
        .bind(i.name)
        .bind(i.enabled)
        .execute(&mut *tx)
        .await?;
    if changed.rows_affected() == 0 {
        return Err(ApiError(StatusCode::NOT_FOUND, "not_found"));
    }
    if let Some(secret) = i.token {
        if secret.is_empty() || secret.len() > 8192 {
            return Err(bad());
        }
        let ciphertext = encrypt(&s.key, id, &secret).map_err(|_| internal())?;
        sqlx::query("UPDATE provider_credentials SET ciphertext=$2 WHERE provider_id=$1")
            .bind(id)
            .bind(ciphertext)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE providers SET credential_version=credential_version+1,suspended=false,blocked_until=NULL WHERE id=$1").bind(id).execute(&mut *tx).await?;
    }
    log(&mut tx, u.id, "providers.edit", Some(id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn delete_provider(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let u = auth(&s, &h, "providers.manage", true).await?;
    if !u.admin {
        return Err(forbidden());
    }
    let mut tx = s.pool.begin().await?;
    sqlx::query("DELETE FROM dns_zones z WHERE provider_id=$1 AND NOT EXISTS(SELECT 1 FROM dns_records r WHERE r.zone_id=z.id)").bind(id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM providers WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    log(&mut tx, u.id, "providers.delete", Some(id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct Page {
    #[serde(default)]
    offset: i64,
    record_id: Option<Uuid>,
}
async fn records(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Query(p): Query<Page>,
) -> Result<Json<Value>> {
    let u = auth(&s, &h, "records.read", false).await?;
    let query = format!(
        "SELECT to_jsonb(item) FROM ({RECORD_SELECT} WHERE {SCOPE} ORDER BY z.name,r.name,r.record_type LIMIT 100 OFFSET $3) item"
    );
    let rows: Vec<Value> = sqlx::query_scalar(&query)
        .bind(u.id)
        .bind(u.admin)
        .bind(p.offset.clamp(0, 1_000_000))
        .fetch_all(&s.pool)
        .await?;
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id WHERE {SCOPE}"
    ))
    .bind(u.id)
    .bind(u.admin)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(
        json!({"items":rows,"total":count,"limit":100,"offset":p.offset.max(0)}),
    ))
}
async fn resource(s: &State, u: &User, id: Uuid) -> Result<()> {
    let yes:bool=sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id WHERE {SCOPE} AND r.id=$3)")).bind(u.id).bind(u.admin).bind(id).fetch_one(&s.pool).await?;
    if !yes {
        return Err(ApiError(StatusCode::NOT_FOUND, "not_found"));
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordInput {
    provider_id: Uuid,
    zone_remote_id: String,
    zone_name: String,
    name: String,
    record_type: String,
    #[serde(default)]
    remote_record_id: String,
    #[serde(default)]
    manual_ip: String,
    #[serde(default)]
    managed_ip: String,
    #[serde(default)]
    active: bool,
    #[serde(default = "default_interval")]
    interval_seconds: i32,
    #[serde(default)]
    site: String,
}
fn default_interval() -> i32 {
    300
}
fn record_valid(name: &str, typ: &str, ip: &str, interval: i32, site: &str) -> bool {
    (name == "@" || name == "*" || valid_name(name, true))
        && matches!(typ, "A" | "AAAA")
        && (ip.is_empty() || public_ip(ip, typ))
        && (60..=86400).contains(&interval)
        && site.len() <= 120
}
fn remote_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
async fn create_record(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(i): Json<RecordInput>,
) -> Result<(StatusCode, Json<Value>)> {
    let u = auth(&s, &h, "records.create", true).await?;
    if !record_valid(
        &i.name,
        &i.record_type,
        &i.manual_ip,
        i.interval_seconds,
        &i.site,
    ) || !valid_name(&i.zone_name, false)
        || !remote_valid(&i.zone_remote_id)
        || (!i.managed_ip.is_empty() && !public_ip(&i.managed_ip, &i.record_type))
    {
        return Err(bad());
    }
    let provider=sqlx::query("SELECT kind FROM providers p WHERE p.id=$1 AND ($3 OR EXISTS(SELECT 1 FROM provider_permissions pp WHERE pp.provider_id=p.id AND pp.user_id=$2))").bind(i.provider_id).bind(u.id).bind(u.admin).fetch_optional(&s.pool).await?.ok_or(forbidden())?;
    if provider.get::<String, _>("kind") == "cloudflare" && !remote_valid(&i.remote_record_id) {
        return Err(bad());
    }
    let mut tx = s.pool.begin().await?;
    let zone_id = Uuid::new_v4();
    let zone=sqlx::query("INSERT INTO dns_zones(id,provider_id,remote_id,name) VALUES($1,$2,$3,$4) ON CONFLICT(provider_id,remote_id) DO UPDATE SET remote_id=EXCLUDED.remote_id RETURNING id,name").bind(zone_id).bind(i.provider_id).bind(i.zone_remote_id).bind(&i.zone_name).fetch_one(&mut *tx).await?;
    if zone.get::<String, _>("name") != i.zone_name {
        return Err(ApiError(StatusCode::CONFLICT, "zone_mismatch"));
    }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO dns_records(id,zone_id,name,record_type,remote_record_id,manual_ip,managed_ip,active,interval_seconds,owner_id,site) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)").bind(id).bind(zone.get::<Uuid,_>("id")).bind(i.name).bind(i.record_type).bind(i.remote_record_id).bind(i.manual_ip).bind(i.managed_ip).bind(i.active).bind(i.interval_seconds).bind(u.id).bind(i.site).execute(&mut *tx).await?;
    log(&mut tx, u.id, "records.create", Some(id)).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"id":id}))))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordEdit {
    active: bool,
    manual_ip: String,
    interval_seconds: i32,
    site: String,
}
async fn edit_record(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
    Json(i): Json<RecordEdit>,
) -> Result<StatusCode> {
    let u = auth(&s, &h, "records.edit", true).await?;
    resource(&s, &u, id).await?;
    let typ: String = sqlx::query_scalar("SELECT record_type FROM dns_records WHERE id=$1")
        .bind(id)
        .fetch_one(&s.pool)
        .await?;
    if !record_valid("@", &typ, &i.manual_ip, i.interval_seconds, &i.site) {
        return Err(bad());
    }
    let mut tx = s.pool.begin().await?;
    sqlx::query("UPDATE dns_records SET active=$2,manual_ip=$3,interval_seconds=$4,site=$5,config_version=config_version+1,next_check=greatest(next_check,now()) WHERE id=$1").bind(id).bind(i.active).bind(i.manual_ip).bind(i.interval_seconds).bind(i.site).execute(&mut *tx).await?;
    log(&mut tx, u.id, "records.edit", Some(id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn delete_record(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let u = auth(&s, &h, "records.delete", true).await?;
    resource(&s, &u, id).await?;
    let mut tx = s.pool.begin().await?;
    sqlx::query("DELETE FROM dns_records WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    log(&mut tx, u.id, "records.delete", Some(id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn update_record(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>)> {
    let u = auth(&s, &h, "updates.trigger", true).await?;
    resource(&s, &u, id).await?;
    let mut tx = s.pool.begin().await?;
    let job:Uuid=sqlx::query_scalar("INSERT INTO update_jobs(id,record_id,trigger,due_at) SELECT gen_random_uuid(),id,'manual',greatest(next_check,now()) FROM dns_records WHERE id=$1 ON CONFLICT(record_id) WHERE state IN ('queued','running') DO UPDATE SET record_id=EXCLUDED.record_id RETURNING id").bind(id).fetch_one(&mut *tx).await?;
    log(&mut tx, u.id, "updates.trigger", Some(id)).await?;
    tx.commit().await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"job_id":job,"cooldown_applies":true})),
    ))
}
async fn create_missing_record(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>)> {
    let u = auth(&s, &h, "records.create", true).await?;
    if !u.permissions.iter().any(|p| p == "updates.trigger") {
        return Err(forbidden());
    }
    let mut tx = s.pool.begin().await?;
    let query = format!(
        "SELECT r.active,p.kind,p.enabled,p.suspended FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id JOIN providers p ON p.id=z.provider_id WHERE r.id=$3 AND {SCOPE} FOR UPDATE OF r"
    );
    let record = sqlx::query(&query)
        .bind(u.id)
        .bind(u.admin)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError(StatusCode::NOT_FOUND, "not_found"))?;
    if record.get::<String, _>("kind") != "hetznercloud"
        || !record.get::<bool, _>("active")
        || !record.get::<bool, _>("enabled")
        || record.get::<bool, _>("suspended")
    {
        return Err(ApiError(StatusCode::CONFLICT, "conflict"));
    }
    let job: Uuid = sqlx::query_scalar(
        "INSERT INTO update_jobs(id,record_id,trigger,due_at)
         SELECT gen_random_uuid(),r.id,'create_missing',greatest(r.next_check,p.blocked_until,now())
         FROM dns_records r JOIN dns_zones z ON z.id=r.zone_id JOIN providers p ON p.id=z.provider_id
         WHERE r.id=$1
         ON CONFLICT(record_id) WHERE state IN ('queued','running')
         DO UPDATE SET trigger='create_missing',due_at=greatest(update_jobs.due_at,EXCLUDED.due_at)
         WHERE update_jobs.state='queued'
         RETURNING id"
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError(StatusCode::CONFLICT, "conflict"))?;
    log(&mut tx, u.id, "dns.create_requested", Some(id)).await?;
    tx.commit().await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"job_id":job,"cooldown_applies":true})),
    ))
}
async fn history(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Query(p): Query<Page>,
) -> Result<Json<Value>> {
    let u = auth(&s, &h, "history.read", false).await?;
    let query = format!(
        "SELECT to_jsonb(h) FROM update_history h LEFT JOIN dns_records r ON r.id=h.record_id LEFT JOIN dns_zones z ON z.id=r.zone_id WHERE {SCOPE} AND ($3::uuid IS NULL OR h.record_id=$3) ORDER BY h.created_at DESC LIMIT 100 OFFSET $4"
    );
    let rows: Vec<Value> = sqlx::query_scalar(&query)
        .bind(u.id)
        .bind(u.admin)
        .bind(p.record_id)
        .bind(p.offset.clamp(0, 1_000_000))
        .fetch_all(&s.pool)
        .await?;
    Ok(Json(
        json!({"items":rows,"limit":100,"offset":p.offset.max(0)}),
    ))
}
async fn users(AppState(s): AppState<Arc<State>>, h: HeaderMap) -> Result<Json<Value>> {
    let u = auth(&s, &h, "users.manage", false).await?;
    if !u.admin {
        return Err(forbidden());
    }
    let rows:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'username',username,'role_id',role_id,'active',active) FROM users ORDER BY username LIMIT 500").fetch_all(&s.pool).await?;
    Ok(Json(json!({"items":rows})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewUser {
    username: String,
    password: String,
    role_id: Uuid,
}
async fn create_user(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(i): Json<NewUser>,
) -> Result<(StatusCode, Json<Value>)> {
    let u = auth(&s, &h, "users.manage", true).await?;
    if !u.admin {
        return Err(forbidden());
    }
    if !credentials_valid(&i.username, &i.password) {
        return Err(bad());
    }
    let hash = tokio::task::spawn_blocking(move || hash_password(&i.password))
        .await
        .map_err(|_| internal())?
        .map_err(|_| internal())?;
    let id = Uuid::new_v4();
    let mut tx = s.pool.begin().await?;
    sqlx::query("INSERT INTO users(id,username,password_hash,role_id) VALUES($1,$2,$3,$4)")
        .bind(id)
        .bind(i.username)
        .bind(hash)
        .bind(i.role_id)
        .execute(&mut *tx)
        .await?;
    log(&mut tx, u.id, "users.create", Some(id)).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"id":id}))))
}
async fn roles(AppState(s): AppState<Arc<State>>, h: HeaderMap) -> Result<Json<Value>> {
    let u = auth(&s, &h, "roles.manage", false).await?;
    if !u.admin {
        return Err(forbidden());
    }
    let rows:Vec<Value>=sqlx::query_scalar("SELECT to_jsonb(r)||jsonb_build_object('permissions',ARRAY(SELECT permission FROM role_permissions WHERE role_id=r.id)) FROM roles r ORDER BY name").fetch_all(&s.pool).await?;
    let permissions: Vec<String> = sqlx::query_scalar("SELECT key FROM permissions ORDER BY key")
        .fetch_all(&s.pool)
        .await?;
    Ok(Json(json!({"items":rows,"permissions":permissions})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewRole {
    name: String,
    permissions: Vec<String>,
}
async fn create_role(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(i): Json<NewRole>,
) -> Result<(StatusCode, Json<Value>)> {
    let u = auth(&s, &h, "roles.manage", true).await?;
    if !u.admin {
        return Err(forbidden());
    }
    if i.name.trim().is_empty() || i.name.len() > 80 || i.permissions.len() > 30 {
        return Err(bad());
    }
    let id = Uuid::new_v4();
    let mut tx = s.pool.begin().await?;
    sqlx::query("INSERT INTO roles(id,name) VALUES($1,$2)")
        .bind(id)
        .bind(i.name)
        .execute(&mut *tx)
        .await?;
    for permission in i.permissions {
        sqlx::query(
            "INSERT INTO role_permissions(role_id,permission) VALUES($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(id)
        .bind(permission)
        .execute(&mut *tx)
        .await?;
    }
    log(&mut tx, u.id, "roles.create", Some(id)).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"id":id}))))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Grant {
    user_id: Uuid,
    provider_id: Option<Uuid>,
    record_id: Option<Uuid>,
    enabled: bool,
}
async fn grant(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Json(i): Json<Grant>,
) -> Result<StatusCode> {
    let u = auth(&s, &h, "roles.manage", true).await?;
    if !u.admin {
        return Err(forbidden());
    }
    let (table, column, id) = match (i.provider_id, i.record_id) {
        (Some(id), None) => ("provider_permissions", "provider_id", id),
        (None, Some(id)) => ("record_permissions", "record_id", id),
        _ => return Err(bad()),
    };
    let mut tx = s.pool.begin().await?;
    if i.enabled {
        sqlx::query(&format!(
            "INSERT INTO {table}({column},user_id) VALUES($1,$2) ON CONFLICT DO NOTHING"
        ))
        .bind(id)
        .bind(i.user_id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(&format!(
            "DELETE FROM {table} WHERE {column}=$1 AND user_id=$2"
        ))
        .bind(id)
        .bind(i.user_id)
        .execute(&mut *tx)
        .await?;
    }
    log(&mut tx, u.id, "grants.change", Some(id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn audit(
    AppState(s): AppState<Arc<State>>,
    h: HeaderMap,
    Query(p): Query<Page>,
) -> Result<Json<Value>> {
    auth(&s, &h, "audit.read", false).await?;
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(a) FROM audit_logs a ORDER BY created_at DESC LIMIT 100 OFFSET $1",
    )
    .bind(p.offset.clamp(0, 1_000_000))
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(json!({"items":rows})))
}
async fn openapi(AppState(s): AppState<Arc<State>>, h: HeaderMap) -> Result<Json<Value>> {
    auth(&s, &h, "", false).await?;
    serde_json::from_str(include_str!("../../../docs/openapi.json"))
        .map(Json)
        .map_err(|_| internal())
}

mod routes;

use axum::{
    Router,
    http::{HeaderValue, header},
    routing::get,
};
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

#[derive(Clone)]
pub struct State {
    pub pool: sqlx::PgPool,
    pub key: [u8; 32],
    pub origin: String,
    pub setup_token: String,
    pub secure_cookie: bool,
    pub engine_url: String,
    pub engine_token: String,
    pub client: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        let response = reqwest::Client::new()
            .get("http://127.0.0.1:8080/readyz")
            .timeout(Duration::from_secs(3))
            .send()
            .await?;
        anyhow::ensure!(response.status().is_success(), "not ready");
        return Ok(());
    }
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("ddnsupd_api=info,ddnsupd_worker=info")
        .init();
    let database_url = std::env::var("DDNSUPD_DATABASE_URL")?;
    let key_bytes = hex::decode(std::env::var("DDNSUPD_SECRET_KEY")?).map_err(|_| {
        anyhow::anyhow!(
            "DDNSUPD_SECRET_KEY must contain exactly 64 hexadecimal characters (0-9, a-f)"
        )
    })?;
    let key: [u8; 32] = key_bytes.try_into().map_err(|_| {
        anyhow::anyhow!(
            "DDNSUPD_SECRET_KEY must contain exactly 64 hexadecimal characters (0-9, a-f)"
        )
    })?;
    let origin = std::env::var("DDNSUPD_EXTERNAL_URL")
        .unwrap_or("http://localhost:8080".into())
        .trim_end_matches('/')
        .to_string();
    let url = reqwest::Url::parse(&origin)?;
    anyhow::ensure!(
        url.path() == "/"
            && url.query().is_none()
            && url.fragment().is_none()
            && url.username().is_empty()
            && url.password().is_none(),
        "external URL must be an origin"
    );
    let private_http = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| match ip {
                    std::net::IpAddr::V4(ip) => ip.is_loopback() || ip.is_private(),
                    std::net::IpAddr::V6(ip) => ip.is_loopback() || ip.is_unique_local(),
                })
    });
    anyhow::ensure!(
        url.scheme() == "https" || (url.scheme() == "http" && private_http),
        "HTTP requires localhost or a private LAN address; configure HTTPS for other origins"
    );
    let setup_token = std::env::var("DDNSUPD_SETUP_TOKEN")?;
    let engine_token = std::env::var("DDNSUPD_ENGINE_TOKEN")?;
    anyhow::ensure!(
        setup_token.len() >= 32 && engine_token.len() >= 32,
        "setup and engine tokens must contain at least 32 characters"
    );
    let pool = PgPoolOptions::new()
        .max_connections(12)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;
    let migrations = std::env::var("DDNSUPD_MIGRATIONS_DIR").unwrap_or("migrations".into());
    sqlx::migrate::Migrator::new(std::path::Path::new(&migrations))
        .await?
        .run(&pool)
        .await?;
    let engine_url =
        std::env::var("DDNSUPD_ENGINE_URL").unwrap_or("http://ddnsupd-worker:8090".into());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(40))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let state = Arc::new(State {
        pool: pool.clone(),
        key,
        secure_cookie: url.scheme() == "https",
        origin,
        setup_token,
        engine_url: engine_url.clone(),
        engine_token: engine_token.clone(),
        client: client.clone(),
    });
    let worker = ddnsupd_worker::Worker {
        pool,
        key,
        engine_url,
        engine_token,
        client,
    };
    tokio::spawn(worker.run());
    routes::start_schedule(state.clone())?;
    let web = std::env::var("DDNSUPD_WEB_DIR").unwrap_or("web/build".into());
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(routes::ready))
        .nest(
            "/api/v1",
            routes::router().layer(SetResponseHeaderLayer::overriding(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-store"),
            )),
        )
        .fallback_service(
            ServeDir::new(&web).not_found_service(ServeFile::new(format!("{web}/index.html"))),
        )
        .layer(axum::extract::DefaultBodyLimit::max(65536))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("same-origin"),
        ))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("DDNSupd listening on :8080");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

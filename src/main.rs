use axum::{
    routing::get,
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Internal application modules
mod config;
mod error;
mod http;
mod state;

// ============================================================================
// DOMAIN MODULES (Strictly mapping to your 19 custom PostgreSQL schemas)
// ============================================================================
mod modules {
    pub mod auth_governance;
    pub mod board;
    pub mod catalog;
    pub mod cms;
    pub mod collab;
    pub mod comms;
    pub mod core;
    pub mod crm;
    pub mod data_governance;
    pub mod dms;
    pub mod event_bus;
    pub mod finance;
    pub mod hr;
    pub mod lms;
    pub mod ops;
    pub mod policy;
    pub mod reference;
    pub mod sis;
    pub mod workflow;
}

use crate::{
    config::Config,
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── 1. Observability ──────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,edusuite_backend=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Bootstrapping EduSuite Enterprise API...");

    // ── 2. Configuration ──────────────────────────────────────────────────
    // Config::from_env() calls dotenvy::dotenv() internally, validates all
    // required variables, and panics with a clear message if any are missing.
    let config = Config::from_env();

    // ── 3. PostgreSQL Connection Pool ─────────────────────────────────────
    // NOTE: If using PgBouncer in Transaction mode, disable prepared
    // statements by appending `?statement_cache_capacity=0` to DATABASE_URL.
    let pool = PgPoolOptions::new()
        .max_connections(50)    // Adjust based on PgBouncer limits
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(&config.database_url)
        .await?;

    tracing::info!("PostgreSQL Connection Pool Established.");

    // ── 4. Database Migrations ────────────────────────────────────────────
    tracing::info!("Running pending database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database schema is up to date.");

    // ── 5. Application State ──────────────────────────────────────────────
    // Both encoding and decoding keys are kept on AppState:
    //   jwt_encoding_key — for signing new tokens (login, refresh)
    //   jwt_decoding_key — for verifying incoming tokens (AuthUser extractor)
    let app_state = AppState {
        db: pool.clone(),
        jwt_encoding_key: std::sync::Arc::new(
            jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        ),
        jwt_decoding_key: std::sync::Arc::new(
            jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        ),
        config,
    };

    // ── 6. Background Workers ─────────────────────────────────────────────
    spawn_event_bus_worker(pool.clone());

    // ── 7. Middleware ─────────────────────────────────────────────────────
    let cors = CorsLayer::new()
        // TODO: restrict to specific frontend domains before production
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Capture port before app_state is consumed by with_state() below.
    let port = app_state.config.port;

    // ── 8. Router ─────────────────────────────────────────────────────────
    // All domain modules are mounted. Auth routes are public (no JWT required
    // — they produce the token). All other /api/v1 routes will require JWT
    // once require_jwt middleware is activated below.
    let api_v1 = Router::new()
        // ── Auth (public — no JWT required) ───────────────────────────────
        .nest("/auth",     modules::auth_governance::router())
        // ── Domain modules (JWT required — uncomment middleware below) ────
        .nest("/board",    modules::board::router())
        .nest("/catalog",  modules::catalog::router())
        .nest("/cms",      modules::cms::router())
        .nest("/collab",   modules::collab::router())
        .nest("/comms",    modules::comms::router())
        .nest("/core",     modules::core::router())
        .nest("/crm",      modules::crm::router())
        .nest("/data-gov", modules::data_governance::router())
        .nest("/dms",      modules::dms::router())
        .nest("/finance",  modules::finance::router())
        .nest("/hr",       modules::hr::router())
        .nest("/lms",      modules::lms::router())
        .nest("/ops",      modules::ops::router())
        .nest("/policy",   modules::policy::router())
        .nest("/reference",modules::reference::router())
        .nest("/sis",      modules::sis::router())
        .nest("/sis",      modules::lms::sis_bridge_router())
        .nest("/workflow", modules::workflow::router())
        // ── JWT middleware (activate once first domain handlers are built) ─
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            http::auth::require_jwt,
        ))
        ;

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state);

    // ── 9. Listen ─────────────────────────────────────────────────────────
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("EduSuite API successfully bound to {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Simple health check for load balancers and Kubernetes liveness probes.
async fn health_check() -> &'static str {
    "EduSuite API is online and healthy."
}

/// Event Bus background worker.
///
/// Polls `event_bus.outbox` every 5 seconds for pending events and dispatches
/// them to the appropriate downstream service. Uses `FOR UPDATE SKIP LOCKED`
/// so multiple worker instances can run without double-processing.
fn spawn_event_bus_worker(_pool: sqlx::PgPool) {
    tokio::spawn(async move {
        tracing::info!("Event Bus Worker started.");
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            // TODO: implement event dispatch when outbox Rust structs are defined
            // 1. SELECT * FROM event_bus.outbox
            //    WHERE status = 'pending' LIMIT 50 FOR UPDATE SKIP LOCKED
            // 2. Dispatch each event to the subscriber service
            // 3. UPDATE event_bus.outbox SET status = 'processed' WHERE id = ...
        }
    });
}
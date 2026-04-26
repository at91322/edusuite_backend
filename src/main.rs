use axum::{
    middleware,
    routing::get,
    Router,
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{env, time::Duration};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Environment & Enterprise Observability (Tracing)
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info,edusuite_backend=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Bootstrapping EduSuite Enterprise API...");

    // 2. Establish PostgreSQL Connection Pool
    // NOTE: If using PgBouncer in Transaction mode, you must disable prepared statements
    // by appending `?statement_cache_capacity=0` to your DATABASE_URL.
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    
    let pool = PgPoolOptions::new()
        .max_connections(50) // Adjust based on PgBouncer limits
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(&database_url)
        .await?;

    tracing::info!("PostgreSQL Connection Pool Established.");

    // 3. Automated Database Migrations
    // This looks for a `migrations/` folder in your project root and runs any pending .sql files.
    // This solves the problem of how your massive schema gets into the database safely.
    tracing::info!("Running pending database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database schema is up to date.");

    // 4. Build Shared Application State
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");
    
    let app_state = AppState { 
        db: pool.clone(),
        jwt_decoding_key: std::sync::Arc::new(jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes())),
        jwt_encoding_key: std::sync::Arc::new(jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes())),
    };

    // 5. Spawn Asynchronous Background Workers (Event Bus)
    // This detaches a thread to handle the `event_bus` outbox without blocking HTTP requests.
    spawn_event_bus_worker(pool.clone());

    // 6. Global Middleware Configuration
    let cors = CorsLayer::new()
        .allow_origin(Any) // In production, restrict to your specific frontend domains
        .allow_methods(Any)
        .allow_headers(Any);

    // 7. Construct the Routing Tree
    // We namespace the API and mount each of your 19 domains cleanly.
    let api_v1 = Router::new()
        .nest("/auth", modules::auth_governance::router())
        .nest("/board", modules::board::router())
        .nest("/catalog", modules::catalog::router())
        .nest("/cms", modules::cms::router())
        .nest("/collab", modules::collab::router())
        .nest("/comms", modules::comms::router())
        .nest("/core", modules::core::router())
        .nest("/crm", modules::crm::router())
        .nest("/data-gov", modules::data_governance::router())
        .nest("/dms", modules::dms::router())
        .nest("/finance", modules::finance::router())
        .nest("/hr", modules::hr::router())
        .nest("/lms", modules::lms::router())
        .nest("/ops", modules::ops::router())
        .nest("/policy", modules::policy::router())
        .nest("/reference", modules::reference::router())
        .nest("/sis", modules::sis::router())
        .nest("/workflow", modules::workflow::router())
        // Apply the JWT Authentication Middleware to ALL /api/v1 routes
        // .route_layer(middleware::from_fn(http::auth::require_jwt))
        ;

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state);

    // 8. Bind to Port and Serve
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    tracing::info!("EduSuite API successfully bound to port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Simple health check endpoint for Load Balancers / Kubernetes
async fn health_check() -> &'static str {
    "EduSuite API is online and healthy."
}

/// The Event Bus Background Worker
/// This runs infinitely in the background, checking your `event_bus` schema for pending outbox messages.
fn spawn_event_bus_worker(pool: sqlx::PgPool) {
    tokio::spawn(async move {
        tracing::info!("Event Bus Worker started.");
        let mut interval = tokio::time::interval(Duration::from_secs(5)); // Poll every 5 seconds
        
        loop {
            interval.tick().await;
            
            // Example logic:
            // 1. SELECT * FROM event_bus.outbox WHERE status = 'pending' LIMIT 50 FOR UPDATE SKIP LOCKED;
            // 2. Loop through events and execute cross-domain logic (e.g., calling Comms API to send email).
            // 3. UPDATE event_bus.outbox SET status = 'processed' WHERE id = ...
            
            // Note: Actual SQL execution is omitted here until we define the Rust structs for the outbox.
        }
    });
}
// src/config.rs
//
// Application configuration loaded from environment variables via dotenvy.
// All values are validated at startup — the server refuses to start if
// required variables are missing or malformed. This surfaces configuration
// errors immediately rather than at the first request that needs them.
//
// REQUIRED environment variables:
//   DATABASE_URL   — PostgreSQL connection string for the edusuite_app role
//   JWT_SECRET     — Secret key for HS256 JWT signing (min 32 bytes)
//
// OPTIONAL environment variables (sensible defaults for local development):
//   PORT                    — HTTP listen port (default: 8081)
//   RUST_LOG                — Tracing filter (default: info)
//   ACCESS_TOKEN_TTL_SECS   — Access token lifetime (default: 900 = 15 min)
//   REFRESH_TOKEN_TTL_SECS  — Refresh token lifetime (default: 2592000 = 30 days)
//   PLATFORM_CLIENT_ID      — OAuth client UUID for password-grant logins
//                             (default: 00000000-0000-0000-0000-000000000001)
 
use std::env;
 
/// Application-wide configuration, validated and loaded once at startup.
/// Stored in `AppState` and shared via Axum's `State` extractor.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
 
    // Token TTLs — used as fallback if tenant_security_policy has no row.
    // In production the DB policy governs; these defaults serve local dev.
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
 
    // The platform-level OAuth client UUID seeded in pre_rls_test_fixes.sql.
    // All password-grant logins create token_families under this client.
    pub platform_client_id: uuid::Uuid,
}
 
impl Config {
    /// Load and validate configuration from the environment.
    /// Panics with a clear message if any required variable is missing.
    pub fn from_env() -> Self {
        // Load .env file if present — silently ignored in production
        // where env vars are injected directly.
        let _ = dotenvy::dotenv();
 
        let database_url = require_env("DATABASE_URL");
 
        let jwt_secret = require_env("JWT_SECRET");
        if jwt_secret.len() < 32 {
            panic!("JWT_SECRET must be at least 32 characters");
        }
 
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8081".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid port number (1–65535)");
 
        let access_token_ttl_secs = env::var("ACCESS_TOKEN_TTL_SECS")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<i64>()
            .expect("ACCESS_TOKEN_TTL_SECS must be a positive integer");
 
        let refresh_token_ttl_secs = env::var("REFRESH_TOKEN_TTL_SECS")
            .unwrap_or_else(|_| "2592000".to_string())
            .parse::<i64>()
            .expect("REFRESH_TOKEN_TTL_SECS must be a positive integer");
 
        let platform_client_id = env::var("PLATFORM_CLIENT_ID")
            .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".to_string())
            .parse::<uuid::Uuid>()
            .expect("PLATFORM_CLIENT_ID must be a valid UUID");
 
        Self {
            database_url,
            jwt_secret,
            port,
            access_token_ttl_secs,
            refresh_token_ttl_secs,
            platform_client_id,
        }
    }
}
 
fn require_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Required environment variable {key} is not set"))
}
 
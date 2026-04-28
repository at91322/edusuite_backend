// src/state.rs

use std::sync::Arc;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

use crate::config::Config;

// Shared application state injected into every Axum handler via `State<AppState>`.
//
// Cloning is cheap — `PgPool`, `Arc`, and `Config` all clone without
// copying the underlying data.
#[derive(Clone)]
pub struct AppState {
    // The PostgreSQL connection pool. All database access goes through here.
    pub db: PgPool,

    // JWT signing key — used to issue new access tokens (login, refresh).
    pub jwt_encoding_key: Arc<EncodingKey>,

    // JWT verification key — used by the AuthUser extractor to validate
    // incoming Bearer tokens. Kept separate from the encoding key so that
    // in future we can switch to RS256 (asymmetric) by replacing only
    // jwt_decoding_key with the public key without changing the signing path.
    pub jwt_decoding_key: Arc<DecodingKey>,

    // Application configuration loaded from environment at startup.
    // Contains token TTLs, port, platform client UUID, and other settings.
    pub config: Config,
}
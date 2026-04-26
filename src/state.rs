use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    
    // We use Arc (Atomic Reference Counting) to safely share the cryptographic keys
    // across thousands of concurrent HTTP requests without cloning the raw bytes.
    pub jwt_decoding_key: Arc<DecodingKey>,
    
    // You will need this later for the auth_governance module to issue tokens
    pub jwt_encoding_key: Arc<EncodingKey>, 
}
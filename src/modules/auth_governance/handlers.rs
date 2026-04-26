use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{extract::State, http::StatusCode, Json};
use jsonwebtoken::{encode, Header};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    http::auth::JwtClaims,
    modules::auth_governance::models::{LoginRequest, LoginResponse},
    state::AppState,
};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    
    // 1. Fetch the user AND their primary tenant link from the database
    // Note: In a production app, you might handle users with multiple tenants differently,
    // but here we grab their active tenant membership.
    let record = sqlx::query!(
        r#"
        SELECT u.id, u.password_hash, tm.tenant_id 
        FROM core.users u
        JOIN core.tenant_memberships tm ON u.id = tm.user_id
        WHERE u.username = $1 AND u.is_active = true
        LIMIT 1
        "#,
        payload.username
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Protect against username enumeration (Return generic 401 if not found)
    let user = match record {
        Some(u) => u,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // 3. Extract the hash (sqlx guarantees this is a String because the column is NOT NULL)
    let hash_string = user.password_hash;

    // 4. Verify the password using Argon2
    let parsed_hash = PasswordHash::new(&hash_string)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err() 
    {
        return Err(StatusCode::UNAUTHORIZED); // Bad password
    }

    // 5. Build the JWT payload
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
        
    let expires_in_seconds = 3600; // 1 Hour

    let claims = JwtClaims {
        sub: user.id,
        tenant_id: user.tenant_id,
        iat: now,
        exp: now + expires_in_seconds, 
    };

    // 6. Cryptographically sign the token using the cached AppState key
    let token = encode(&Header::default(), &claims, &state.jwt_encoding_key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 7. Return the credentials to the client
    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: expires_in_seconds,
    }))
}
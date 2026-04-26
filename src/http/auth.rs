use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{http::extractors::TenantContext, state::AppState};

/// The expected payload inside the JWT Bearer Token
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,        // The standard Subject claim maps to core.users(id)
    pub tenant_id: Uuid,  // Custom claim mapping to core.tenants(id)
    pub exp: usize,       // Expiration timestamp
    pub iat: usize,       // Issued at timestamp
}

/// The Axum Middleware Function
pub async fn require_jwt(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    
    // 1. Extract the "Authorization" header
    let auth_header = req.headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. Enforce "Bearer " prefix format
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = &auth_header[7..];

    // 3. Cryptographically verify the token using the cached State key
    let token_data = decode::<JwtClaims>(
        token,
        &state.jwt_decoding_key,
        &Validation::default(),
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 4. Construct the strictly verified TenantContext
    let tenant_ctx = TenantContext {
        tenant_id: token_data.claims.tenant_id,
        user_id: token_data.claims.sub,
    };

    // 5. Inject the context into the Request extensions
    // Downstream handlers can now simply ask for `Extension(tenant_ctx)`
    req.extensions_mut().insert(tenant_ctx);

    // 6. Proceed to the actual API endpoint
    Ok(next.run(req).await)
}
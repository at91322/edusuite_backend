// src/http/auth.rs
//
// JWT claim structure and the `AuthUser` request extractor.
//
// FLOW FOR PROTECTED ENDPOINTS
// ─────────────────────────────
// Every handler that requires authentication adds `AuthUser` as a parameter:
//
//   pub async fn get_me(
//       State(state): State<AppState>,
//       user: AuthUser,               // ← extracted here
//   ) -> Result<Json<MeResponse>, AppError> { ... }
//
// Axum calls `AuthUser::from_request_parts` automatically. The extractor:
//   1. Pulls the `Authorization: Bearer <token>` header
//   2. Decodes and validates the JWT signature + expiry
//   3. Opens a database transaction with RLS context set
//   4. Attaches the claims to the request
//
// The `TxGuard` inside `AuthUser` holds the open transaction. Handlers
// must call `user.commit()` before returning (or the transaction rolls back).
//
// NOTE ON RLS CONTEXT
// ────────────────────
// `SET LOCAL app.current_tenant_id = '...'` and
// `SET LOCAL app.current_user_id   = '...'`
// are transaction-scoped. They must be set inside a transaction and remain
// set for all subsequent queries in that transaction. SET ROLE must happen
// BEFORE SET LOCAL — switching role clears all session locals.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};
use jsonwebtoken::{decode, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

// ── JWT Claims ────────────────────────────────────────────────────────────────

/// Claims embedded in every access token.
///
/// `sub`       — the user's UUID (core.users.id)
/// `tenant_id` — the tenant context for this session (core.tenants.id)
/// `family_id` — the token family UUID (auth_governance.token_families.id)
///               Included so logout can revoke the family from the JWT alone.
/// `iat` / `exp` — standard issued-at and expiry (Unix seconds)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub:       Uuid,
    pub tenant_id: Uuid,
    pub family_id: Uuid,
    pub iat:       usize,
    pub exp:       usize,
}

// ── AuthUser extractor ────────────────────────────────────────────────────────

/// Axum extractor for authenticated requests.
///
/// Validates the JWT and opens an RLS-scoped transaction. Use this as a
/// parameter in any handler that requires authentication. Access the
/// claims via `user.claims` and the DB transaction via `user.tx`.
///
/// Example:
/// ```rust
/// pub async fn protected(
///     user: AuthUser,
///     State(state): State<AppState>,
/// ) -> Result<impl IntoResponse, AppError> {
///     let row = sqlx::query!("SELECT * FROM ...")
///         .fetch_one(&mut *user.tx)
///         .await?;
///     user.tx.commit().await?;
///     Ok(Json(row))
/// }
/// ```
pub struct AuthUser {
    pub claims: JwtClaims,
    pub tx:     sqlx::Transaction<'static, sqlx::Postgres>,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // ── 1. Extract Bearer token ───────────────────────────────────────
        let token = extract_bearer_token(&parts.headers)?;

        // ── 2. Decode and validate JWT ────────────────────────────────────
        // Use the pre-built DecodingKey from AppState rather than rebuilding per request.
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<JwtClaims>(&token, &state.jwt_decoding_key, &validation)
            .map_err(AppError::from)?;

        let claims = token_data.claims;

        // ── 3. Open transaction with RLS context ──────────────────────────
        let mut tx = begin_rls_transaction(
            &state.db,
            claims.tenant_id,
            claims.sub,
            "api",
        )
        .await?;

        // ── 4. Check token family is still active ─────────────────────────
        // Must run INSIDE the transaction so the RLS context (set_config) is
        // active — token_families has a restrictive tenant_isolation_policy
        // that requires app.current_tenant_id to be set. Without it the row
        // is invisible and we'd incorrectly reject valid tokens.
        //
        // Stateless JWTs remain cryptographically valid after logout until
        // they expire. This check closes that window: logout and theft
        // detection both set family status = 'revoked', so revoked sessions
        // are rejected immediately regardless of the JWT expiry time.
        let family_status = sqlx::query!(
            r#"
            SELECT status::text AS "status!"
            FROM auth_governance.token_families
            WHERE id        = $1
              AND tenant_id = $2
            "#,
            claims.family_id,
            claims.tenant_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Family status check failed: {}", e)))?;

        match family_status {
            None => {
                return Err(AppError::Unauthorized("Session not found".to_string()));
            }
            Some(row) if row.status != "active" => {
                return Err(AppError::Unauthorized(
                    "Session has been revoked. Please log in again.".to_string(),
                ));
            }
            _ => {} // active — proceed
        }

        Ok(AuthUser { claims, tx })
    }
}

// ── AuthUserForLogout extractor ───────────────────────────────────────────────
//
// Identical to AuthUser but skips the family status check so logout remains
// idempotent — calling logout on an already-revoked session still returns 204
// rather than 401. The logout handler revokes the family if active, no-ops if
// already revoked. A valid signed JWT is still required.

pub struct AuthUserForLogout {
    pub claims: JwtClaims,
    pub tx:     sqlx::Transaction<'static, sqlx::Postgres>,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUserForLogout {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token  = extract_bearer_token(&parts.headers)?;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<JwtClaims>(&token, &state.jwt_decoding_key, &validation)
            .map_err(AppError::from)?;

        let claims = token_data.claims;

        // Open RLS transaction — no family status check for logout
        let tx = begin_rls_transaction(
            &state.db,
            claims.tenant_id,
            claims.sub,
            "api",
        )
        .await?;

        Ok(AuthUserForLogout { claims, tx })
    }
}

// ── Helper: extract Bearer token from Authorization header ────────────────────

pub fn extract_bearer_token(headers: &HeaderMap) -> Result<String, AppError> {
    let header = headers
        .get("Authorization")
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Malformed Authorization header".to_string()))?;

    if !header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized(
            "Authorization header must use Bearer scheme".to_string(),
        ));
    }

    Ok(header["Bearer ".len()..].to_string())
}

// ── begin_rls_transaction ─────────────────────────────────────────────────────

/// Open a PostgreSQL transaction and set the RLS session-local variables.
///
/// IMPORTANT: SET LOCAL variables are transaction-scoped and cleared when
/// the transaction ends (commit or rollback). Every query that depends on
/// RLS must run inside this transaction.
///
/// `source_service` is written to audit_logs.source_service so the audit
/// trail can distinguish API calls from background jobs.
pub async fn begin_rls_transaction(
    pool:           &PgPool,
    tenant_id:      Uuid,
    user_id:        Uuid,
    source_service: &str,
) -> Result<sqlx::Transaction<'static, sqlx::Postgres>, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to begin transaction: {}", e)))?;

    // PostgreSQL's prepared statement protocol only allows ONE command per
    // statement. Concatenating SET LOCAL calls with semicolons causes:
    //   "cannot insert multiple commands into a prepared statement"
    //
    // Solution: use set_config(key, value, is_local=true) — a single
    // SQL function call per variable. is_local=true makes it transaction-
    // scoped, identical behaviour to SET LOCAL.
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to set tenant context: {}", e)))?;

    sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
        .bind(user_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to set user context: {}", e)))?;

    sqlx::query("SELECT set_config('app.current_service', $1, true)")
        .bind(source_service)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to set service context: {}", e)))?;

    Ok(tx)
}

// ── Token hashing ─────────────────────────────────────────────────────────────

/// Hash a refresh token value for storage.
///
/// Refresh tokens are stored as SHA-256 hashes, never in plaintext.
/// The raw token is sent to the client once and never stored.
pub fn hash_token(raw: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a cryptographically random refresh token.
///
/// Returns a 64-character hex string (32 bytes of randomness).
/// Uses OsRng directly rather than thread_rng to guarantee OS-level
/// entropy on every call — no risk of reseeding lag on Windows.
/// This is the value sent to the client; the hash is stored.
pub fn generate_refresh_token() -> String {
    use rand::rngs::OsRng;
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}
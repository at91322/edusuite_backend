// src/modules/auth_governance/handlers.rs
//
// Auth handlers: login, refresh, logout, and /me.
//
// SECURITY DESIGN SUMMARY
// ────────────────────────
// All four handlers follow the same principle: fail safe, log everything,
// return the minimum information needed. Specifically:
//
//   • login:   Generic 401 for any credential failure (no enumeration).
//              Lockout checked before password verification.
//              Login events written for every attempt (success + failure).
//              Token family + refresh token created on success.
//
//   • refresh: SHA-256 hash of the provided token is looked up.
//              If found but already consumed → theft detected → revoke family.
//              Atomic rotation via SECURITY DEFINER (no race condition).
//              New access JWT issued with TTL from tenant_security_policy.
//
//   • logout:  Requires valid JWT (AuthUser extractor).
//              Revokes entire token family (all sessions from this login).
//              Writes a session_expired login event.
//              Returns 204 — no body.
//
//   • me:      Demonstrates the AuthUser extractor pattern.
//              All future domain handlers follow this same pattern.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    error::AppError,
    http::auth::{
        begin_rls_transaction, generate_refresh_token, hash_token,
        AuthUser, AuthUserForLogout, JwtClaims,
    },
    modules::auth_governance::{
        models::{LoginRequest, LoginResponse, MeResponse, RefreshRequest, RefreshResponse},
        queries,
    },
    state::AppState,
};

// ── POST /auth/login ──────────────────────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    headers:      HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {

    let ip  = extract_ip(&headers);
    let ua  = extract_ua(&headers);

    // ── 1. Resolve credentials (SECURITY DEFINER — bypasses RLS) ─────────
    let creds = queries::resolve_login_credentials(
        &state.db,
        &payload.username,
        &payload.domain,
    )
    .await?;

    let creds = match creds {
        Some(c) => c,
        None => {
            // Unknown username or domain — log a best-effort event.
            // We can't write to login_events without a tenant_id;
            // the SECURITY DEFINER function handles that gracefully.
            tracing::warn!(
                username = %payload.username,
                domain   = %payload.domain,
                "Login failed: credentials not found"
            );
            // Constant-time delay to prevent user enumeration via timing.
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }
    };

    // ── 2. Read tenant security policy ────────────────────────────────────
    let policy = queries::get_tenant_security_policy(&state.db, creds.tenant_id)
        .await?;

    let access_ttl = policy
        .as_ref()
        .map(|p| p.access_token_ttl_secs as i64)
        .unwrap_or(state.config.access_token_ttl_secs);

    let refresh_ttl = policy
        .as_ref()
        .map(|p| p.refresh_token_ttl_secs as i64)
        .unwrap_or(state.config.refresh_token_ttl_secs);

    let lockout_threshold = policy.as_ref().map(|p| p.lockout_threshold).unwrap_or(5);
    let lockout_duration  = policy.as_ref().map(|p| p.lockout_duration_minutes).unwrap_or(30);

    // ── 3. Check account lockout (SECURITY DEFINER) ───────────────────────
    let lockout = queries::check_account_lockout(
        &state.db, creds.user_id, creds.tenant_id,
    )
    .await?;

    if lockout.is_locked {
        queries::record_login_event(
            &state.db,
            creds.tenant_id,
            Some(creds.user_id),
            "account_locked",
            ip.as_deref(),
            ua.as_deref(),
            None,
        ).await?;
        return Err(AppError::Unauthorized("Account is temporarily locked".to_string()));
    }

    // ── 4. Verify password ────────────────────────────────────────────────
    let hash = argon2::PasswordHash::new(&creds.password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Hash parse error: {}", e)))?;

    let password_ok = argon2::PasswordVerifier::verify_password(
        &argon2::Argon2::default(),
        payload.password.as_bytes(),
        &hash,
    )
    .is_ok();

    if !password_ok {
        // Increment lockout counter
        queries::increment_lockout_counter(
            &state.db,
            creds.user_id,
            creds.tenant_id,
            ip.as_deref(),
            lockout_threshold,
            lockout_duration,
        ).await?;

        queries::record_login_event(
            &state.db,
            creds.tenant_id,
            Some(creds.user_id),
            "invalid_credentials",
            ip.as_deref(),
            ua.as_deref(),
            None,
        ).await?;

        tracing::warn!(
            user_id   = %creds.user_id,
            tenant_id = %creds.tenant_id,
            "Login failed: incorrect password"
        );
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // ── 5. Password correct — open RLS transaction ────────────────────────
    let mut tx = begin_rls_transaction(
        &state.db, creds.tenant_id, creds.user_id, "auth-service",
    )
    .await?;

    // Reset lockout counter on success
    queries::reset_lockout_counter(&state.db, creds.user_id, creds.tenant_id).await?;

    // Update last_accessed_at
    queries::touch_last_accessed(&mut tx, creds.user_id, creds.tenant_id).await?;

    // ── 6. Create token family ────────────────────────────────────────────
    // Must use &mut tx (not &state.db) — the INSERT must run on the same
    // connection where set_config established the RLS tenant context.
    // A pool connection would have no context set -> 42501 RLS violation.
    let family_id = queries::create_token_family(
        &mut tx,
        creds.user_id,
        creds.tenant_id,
        state.config.platform_client_id,
        ip.as_deref(),
        ua.as_deref(),
    )
    .await?;

    // ── 7. Create refresh token ───────────────────────────────────────────
    let raw_refresh  = generate_refresh_token();
    let token_hash   = hash_token(&raw_refresh);
    let expires_at   = Utc::now() + chrono::Duration::seconds(refresh_ttl);

    queries::insert_refresh_token(
        &mut tx,
        family_id,
        creds.tenant_id,
        &token_hash,
        expires_at,
    )
    .await?;

    // ── 8. Commit first, then write login event ──────────────────────────
    // record_login_event references family_id via FK on token_families.
    // token_families was inserted inside tx — committing first makes it
    // visible to the login_events INSERT which runs on a separate connection
    // via the SECURITY DEFINER function.
    tx.commit().await.map_err(AppError::from)?;

    queries::record_login_event(
        &state.db,
        creds.tenant_id,
        Some(creds.user_id),
        "success",
        ip.as_deref(),
        ua.as_deref(),
        Some(family_id),
    ).await?;

    // ── 9. Issue access JWT ───────────────────────────────────────────────
    let access_token = issue_access_jwt(
        creds.user_id,
        creds.tenant_id,
        family_id,
        access_ttl,
        &state.config.jwt_secret,
    )?;

    tracing::info!(
        user_id   = %creds.user_id,
        tenant_id = %creds.tenant_id,
        username  = %payload.username,
        "Login successful"
    );

    Ok(Json(LoginResponse {
        access_token,
        refresh_token: raw_refresh,
        token_type:    "Bearer".to_string(),
        expires_in:    access_ttl,
    }))
}

// ── POST /auth/refresh ────────────────────────────────────────────────────────

pub async fn refresh(
    State(state): State<AppState>,
    headers:      HeaderMap,
    Json(payload): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {

    let ip = extract_ip(&headers);

    // ── 1. Hash the provided token ────────────────────────────────────────
    let old_token_hash = hash_token(&payload.refresh_token);

    // ── 2. Generate the new token before the DB call ─────────────────────
    let new_raw_refresh  = generate_refresh_token();
    let new_token_hash   = hash_token(&new_raw_refresh);

    // ── 3. Read tenant policy to get refresh TTL ──────────────────────────
    // We can't read the policy without a tenant_id, and we get the
    // tenant_id from the rotation call. Use config defaults here;
    // we'll refine after the rotation returns the tenant_id.
    let default_refresh_ttl = state.config.refresh_token_ttl_secs;
    let new_expires_at = Utc::now() + chrono::Duration::seconds(default_refresh_ttl);

    // ── 4. Atomic rotation (SECURITY DEFINER — no RLS needed) ────────────
    let family = queries::rotate_refresh_token(
        &state.db,
        &old_token_hash,
        &new_token_hash,
        new_expires_at,
        ip.as_deref(),
    )
    .await?;

    let family = match family {
        Some(f) => f,
        None => {
            // TOKEN THEFT DETECTED: the token was already consumed.
            // The rotate_refresh_token function has already revoked the family.
            tracing::warn!(
                token_hash_prefix = &old_token_hash[..8],
                "Refresh token theft detected — family revoked"
            );
            return Err(AppError::Unauthorized(
                "Token has already been used. Please log in again.".to_string(),
            ));
        }
    };

    // ── 5. Read policy with the now-known tenant_id ───────────────────────
    let policy = queries::get_tenant_security_policy(&state.db, family.tenant_id).await?;
    let access_ttl = policy
        .as_ref()
        .map(|p| p.access_token_ttl_secs as i64)
        .unwrap_or(state.config.access_token_ttl_secs);

    // ── 6. Issue new access JWT ───────────────────────────────────────────
    let access_token = issue_access_jwt(
        family.user_id,
        family.tenant_id,
        family.family_id,
        access_ttl,
        &state.config.jwt_secret,
    )?;

    tracing::debug!(
        user_id   = %family.user_id,
        tenant_id = %family.tenant_id,
        family_id = %family.family_id,
        "Token rotated successfully"
    );

    Ok(Json(RefreshResponse {
        access_token,
        refresh_token: new_raw_refresh,
        token_type:    "Bearer".to_string(),
        expires_in:    access_ttl,
    }))
}

// ── POST /auth/logout ─────────────────────────────────────────────────────────

pub async fn logout(
    State(state): State<AppState>,
    mut user: AuthUserForLogout,
) -> Result<impl IntoResponse, AppError> {

    // Revoke the family — idempotent if already revoked.
    // Uses AuthUserForLogout which skips the family status check so
    // calling logout on an already-revoked session still returns 204.
    queries::revoke_token_family(
        &mut user.tx,
        user.claims.family_id,
        "user_logout",
    )
    .await?;

    queries::record_login_event(
        &state.db,
        user.claims.tenant_id,
        Some(user.claims.sub),
        "session_expired",
        None,
        None,
        Some(user.claims.family_id),
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        user_id   = %user.claims.sub,
        tenant_id = %user.claims.tenant_id,
        family_id = %user.claims.family_id,
        "User logged out"
    );

    Ok(StatusCode::NO_CONTENT)
}

// ── GET /auth/me ──────────────────────────────────────────────────────────────
//
// Demonstrates the AuthUser extractor pattern for domain handlers.
// Every protected handler in every module will follow this same structure.

pub async fn me(
    mut user: AuthUser,
) -> Result<impl IntoResponse, AppError> {

    let row = sqlx::query!(
        r#"
        SELECT
            u.username   AS "username!: String",
            tm.system_role::text AS "role!: String"
        FROM core.users u
        JOIN core.tenant_memberships tm ON tm.user_id = u.id
        WHERE u.id        = $1
          AND tm.tenant_id = $2
        "#,
        user.claims.sub,
        user.claims.tenant_id,
    )
    .fetch_one(&mut *user.tx)
    .await
    .map_err(AppError::from)?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(MeResponse {
        user_id:   user.claims.sub,
        tenant_id: user.claims.tenant_id,
        username:  row.username,
        role:      row.role,
    }))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn issue_access_jwt(
    user_id:    uuid::Uuid,
    tenant_id:  uuid::Uuid,
    family_id:  uuid::Uuid,
    ttl_secs:   i64,
    jwt_secret: &str,
) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = JwtClaims {
        sub:       user_id,
        tenant_id,
        family_id,
        iat:       now,
        exp:       now + ttl_secs as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(AppError::from)
}

fn extract_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Forwarded-For")
        .or_else(|| headers.get("X-Real-IP"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
}

fn extract_ua(headers: &HeaderMap) -> Option<String> {
    headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
// src/modules/auth_governance/models.rs

use serde::{Deserialize, Serialize};

// ── Login ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    /// The tenant's domain (e.g. "hogwarts.edu").
    /// Identifies which institution the user is logging into.
    /// Required for multi-tenant disambiguation.
    pub domain:   String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token:  String,
    pub refresh_token: String,
    pub token_type:    String,
    /// Seconds until the access token expires.
    pub expires_in:    i64,
}

// ── Refresh ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token:  String,
    pub refresh_token: String,
    pub token_type:    String,
    pub expires_in:    i64,
}

// ── Logout ────────────────────────────────────────────────────────────────────

// POST /auth/logout returns 204 No Content — no body needed.

// ── Me (GET /auth/me — example authenticated endpoint) ───────────────────────

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id:   uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub username:  String,
    pub role:      String,
}
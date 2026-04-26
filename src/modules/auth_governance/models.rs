// src/modules/auth_governance/models.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    /// The tenant's domain (e.g. "hogwarts.edu").
    /// Identifies which school the user is logging into.
    /// Required to resolve the correct tenant_id when a user
    /// belongs to more than one institution.
    pub domain: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token:      String,
    pub token_type: String,
    pub expires_in: usize,
}
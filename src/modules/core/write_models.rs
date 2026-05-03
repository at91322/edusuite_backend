// src/modules/core/write_models.rs
//
// Write request/response types for the core module.
// Step 1: Groups 1 (tenant self-service, departments) and 6 (membership).
// Steps 2–5 types appended in subsequent steps.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 1a — PATCH /core/tenants/me
// ═══════════════════════════════════════════════════════════════════════════════

/// Safe allowlist for tenant self-service updates.
///
/// Deliberately narrow — only fields the tenant admin controls.
/// Platform-locked fields (domain, stripe IDs, tax config, currency,
/// country, fiscal year) require a platform admin action and are not
/// patchable through this endpoint.
#[derive(Debug, Deserialize)]
pub struct PatchTenantRequest {
    /// Institution display name — max 255 chars.
    pub name:                 Option<String>,
    /// ISO 3166-2 subdivision code (e.g. "US-OR").
    /// Must exist in reference.subdivisions if provided.
    pub subdivision_code:     Option<String>,
    /// 1–12. Month the fiscal year ends (default: 6 = June).
    pub fiscal_year_end_month: Option<i16>,
}

impl PatchTenantRequest {
    pub fn has_changes(&self) -> bool {
        self.name.is_some()
            || self.subdivision_code.is_some()
            || self.fiscal_year_end_month.is_some()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }

        if let Some(ref n) = self.name {
            if n.trim().is_empty() {
                errors.push("name cannot be blank".into());
            }
            if n.len() > 255 {
                errors.push("name must be 255 characters or fewer".into());
            }
        }

        if let Some(month) = self.fiscal_year_end_month {
            if !(1..=12).contains(&month) {
                errors.push("fiscal_year_end_month must be between 1 and 12".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ── Group 1b: Department management ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    /// Short identifier code — must be unique within the tenant.
    /// Max 50 chars, uppercase recommended (e.g. "MAGI", "HR", "FIN").
    pub code:         String,
    pub name:         String,
    /// UUID of the core.users row for the department head.
    /// Must be an active member of this tenant.
    pub head_user_id: Uuid,
}

impl CreateDepartmentRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.code.trim().is_empty() {
            errors.push("code is required".into());
        }
        if self.code.len() > 50 {
            errors.push("code must be 50 characters or fewer".into());
        }
        if self.name.trim().is_empty() {
            errors.push("name is required".into());
        }
        if self.name.len() > 255 {
            errors.push("name must be 255 characters or fewer".into());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug, Deserialize)]
pub struct PatchDepartmentRequest {
    pub code:         Option<String>,
    pub name:         Option<String>,
    pub head_user_id: Option<Uuid>,
}

impl PatchDepartmentRequest {
    pub fn has_changes(&self) -> bool {
        self.code.is_some() || self.name.is_some() || self.head_user_id.is_some()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }
        if let Some(ref c) = self.code {
            if c.trim().is_empty() { errors.push("code cannot be blank".into()); }
            if c.len() > 50 { errors.push("code must be 50 characters or fewer".into()); }
        }
        if let Some(ref n) = self.name {
            if n.trim().is_empty() { errors.push("name cannot be blank".into()); }
            if n.len() > 255 { errors.push("name must be 255 characters or fewer".into()); }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Response for both POST and PATCH department operations.
#[derive(Debug, Serialize)]
pub struct DepartmentResponse {
    pub id:           Uuid,
    pub code:         String,
    pub name:         String,
    pub head_user_id: Option<Uuid>,
    pub head_name:    Option<String>,
    pub created_at:   chrono::DateTime<chrono::Utc>,
    pub updated_at:   chrono::DateTime<chrono::Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 6 — MEMBERSHIP MANAGEMENT
// Table: core.tenant_memberships
// ═══════════════════════════════════════════════════════════════════════════════

/// Valid core.system_role enum values.
pub const VALID_SYSTEM_ROLES: &[&str] = &[
    "super_admin",
    "tenant_admin",
    "faculty",
    "student",
    "staff",
    "alumni",
    "guardian",
    "volunteer",
    "user",
];

/// Update a member's institutional_email and/or system_role.
/// Only tenant admins should reach this endpoint (enforced by JWT middleware).
#[derive(Debug, Deserialize)]
pub struct PatchMemberRequest {
    /// Tenant-issued email address — distinct from personal core.user_emails.
    pub institutional_email: Option<String>,
    /// New role within this tenant.
    pub system_role:         Option<String>,
}

impl PatchMemberRequest {
    pub fn has_changes(&self) -> bool {
        self.institutional_email.is_some() || self.system_role.is_some()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }

        if let Some(ref email) = self.institutional_email {
            if email.trim().is_empty() {
                errors.push("institutional_email cannot be blank".into());
            }
            if !email.contains('@') {
                errors.push("institutional_email must be a valid email address".into());
            }
            if email.len() > 255 {
                errors.push("institutional_email must be 255 characters or fewer".into());
            }
        }

        if let Some(ref role) = self.system_role {
            if !VALID_SYSTEM_ROLES.contains(&role.as_str()) {
                errors.push(format!(
                    "system_role '{}' is invalid; must be one of: {}",
                    role,
                    VALID_SYSTEM_ROLES.join(", ")
                ));
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// 200 response for PATCH /core/members/:user_id
#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub membership_id:       Uuid,
    pub user_id:             Uuid,
    pub institutional_email: Option<String>,
    pub system_role:         String,
    pub updated_at:          chrono::DateTime<chrono::Utc>,
}
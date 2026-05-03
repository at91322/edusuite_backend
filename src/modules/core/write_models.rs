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

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 2 — USER IDENTITY WRITES
// ═══════════════════════════════════════════════════════════════════════════════

/// Three-state wrapper for PATCH body fields.
///   Absent  → field not in JSON body — keep current DB value
///   Null    → field present as JSON null — clear the column
///   Value(v) → field present with value — update the column
#[derive(Debug, Deserialize, Default)]
#[serde(untagged)]
pub enum MaybePatch<T> {
    #[default]
    Absent,
    Null,
    Value(T),
}

impl<T> MaybePatch<T> {
    pub fn is_absent(&self) -> bool { matches!(self, MaybePatch::Absent) }
    /// Returns Some(&v) if Value, None otherwise.
    pub fn as_value(&self) -> Option<&T> {
        if let MaybePatch::Value(v) = self { Some(v) } else { None }
    }
}

// ── PATCH /core/users/:id ────────────────────────────────────────────────────

/// Valid core.name_change_reason enum values.
pub const VALID_NAME_CHANGE_REASONS: &[&str] = &[
    "marriage", "divorce", "legal_change", "correction", "gender_transition",
];

/// Partial update for a user's name fields.
///
/// Uses MaybePatch<T> for true three-state semantics on nullable fields:
///   Absent  → keep current value (field not present in JSON)
///   Null    → clear the value (field present as JSON null)
///   Value   → set to new value
///
/// first_name and last_name are NOT NULL in the schema — they use Option<String>
/// but null is rejected in validation.
///
/// When any of first_name, middle_name, or last_name changes, the handler
/// MUST write a user_name_history row with the PRE-CHANGE values and reason.
#[derive(Debug, Deserialize)]
pub struct PatchUserRequest {
    pub first_name:      Option<String>,
    /// Three-state: absent = keep, null = clear, value = set
    pub middle_name:     MaybePatch<String>,
    pub last_name:       Option<String>,
    /// Three-state: absent = keep, null = clear, value = set
    pub preferred_name:  MaybePatch<String>,
    /// Three-state: absent = keep, null = clear, value = set
    pub last_name_suffix: MaybePatch<String>,
    /// Required when first_name, middle_name, or last_name changes.
    /// Must be a valid core.name_change_reason enum value.
    pub name_change_reason: Option<String>,
}

impl PatchUserRequest {
    pub fn has_name_change(&self) -> bool {
        self.first_name.is_some()
            || !matches!(self.middle_name, MaybePatch::Absent)
            || self.last_name.is_some()
    }

    pub fn has_changes(&self) -> bool {
        self.has_name_change()
            || !matches!(self.preferred_name,   MaybePatch::Absent)
            || !matches!(self.last_name_suffix, MaybePatch::Absent)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }

        if let Some(ref v) = self.first_name {
            if v.trim().is_empty() { errors.push("first_name cannot be blank".into()); }
            if v.len() > 100 { errors.push("first_name must be 100 characters or fewer".into()); }
        }
        if let Some(ref v) = self.last_name {
            if v.trim().is_empty() { errors.push("last_name cannot be blank".into()); }
            if v.len() > 255 { errors.push("last_name must be 255 characters or fewer".into()); }
        }
        if let MaybePatch::Value(ref v) = self.middle_name {
            if v.len() > 100 { errors.push("middle_name must be 100 characters or fewer".into()); }
        }
        if let MaybePatch::Value(ref v) = self.preferred_name {
            if v.len() > 100 { errors.push("preferred_name must be 100 characters or fewer".into()); }
        }
        if let MaybePatch::Value(ref v) = self.last_name_suffix {
            if v.len() > 10 { errors.push("last_name_suffix must be 10 characters or fewer".into()); }
        }

        // name_change_reason required when legal name changes
        if self.has_name_change() {
            match &self.name_change_reason {
                None => errors.push(
                    "name_change_reason is required when changing first_name, middle_name, or last_name".into()
                ),
                Some(r) if !VALID_NAME_CHANGE_REASONS.contains(&r.as_str()) => {
                    errors.push(format!(
                        "name_change_reason '{}' is invalid; must be one of: {}",
                        r, VALID_NAME_CHANGE_REASONS.join(", ")
                    ));
                }
                _ => {}
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ── POST /core/users/:id/emails ───────────────────────────────────────────────

/// Valid core.email_type enum values.
pub const VALID_EMAIL_TYPES: &[&str] = &[
    "institutional", "personal", "recovery", "guardian",
];

#[derive(Debug, Deserialize)]
pub struct CreateEmailRequest {
    pub email_address: String,
    /// core.email_type — defaults to "personal"
    pub email_type:    Option<String>,
    /// If true, demotes the current primary before setting this one
    pub is_primary:    Option<bool>,
}

impl CreateEmailRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.email_address.trim().is_empty() {
            errors.push("email_address is required".into());
        }
        if !self.email_address.contains('@') {
            errors.push("email_address must be a valid email address".into());
        }
        if self.email_address.len() > 255 {
            errors.push("email_address must be 255 characters or fewer".into());
        }
        if let Some(ref t) = self.email_type {
            if !VALID_EMAIL_TYPES.contains(&t.as_str()) {
                errors.push(format!(
                    "email_type '{}' is invalid; must be one of: {}",
                    t, VALID_EMAIL_TYPES.join(", ")
                ));
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Response for POST and PATCH email operations.
#[derive(Debug, Serialize)]
pub struct EmailResponse {
    pub id:            uuid::Uuid,
    pub user_id:       uuid::Uuid,
    pub email_address: String,
    pub email_type:    String,
    pub is_primary:    bool,
    pub is_verified:   bool,
    pub created_at:    chrono::DateTime<chrono::Utc>,
}

/// PATCH /core/users/:id/emails/:email_id
/// Only is_primary and email_type are patchable — the address itself is immutable.
#[derive(Debug, Deserialize)]
pub struct PatchEmailRequest {
    pub is_primary:  Option<bool>,
    pub email_type:  Option<String>,
    pub is_verified: Option<bool>,
}

impl PatchEmailRequest {
    pub fn has_changes(&self) -> bool {
        self.is_primary.is_some() || self.email_type.is_some() || self.is_verified.is_some()
    }
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }
        if let Some(ref t) = self.email_type {
            if !VALID_EMAIL_TYPES.contains(&t.as_str()) {
                errors.push(format!(
                    "email_type '{}' is invalid; must be one of: {}",
                    t, VALID_EMAIL_TYPES.join(", ")
                ));
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ── POST /core/users/:id/phones ───────────────────────────────────────────────

/// Valid core.phone_type enum values.
pub const VALID_PHONE_TYPES: &[&str] = &["mobile", "home", "work", "emergency"];

#[derive(Debug, Deserialize)]
pub struct CreatePhoneRequest {
    pub phone_number:    String,
    /// Country dialing code — defaults to "+1"
    pub country_code:    Option<String>,
    /// core.phone_type — defaults to "mobile"
    pub phone_type:      Option<String>,
    pub is_primary:      Option<bool>,
    pub can_receive_sms: Option<bool>,
}

impl CreatePhoneRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.phone_number.trim().is_empty() {
            errors.push("phone_number is required".into());
        }
        if self.phone_number.len() > 50 {
            errors.push("phone_number must be 50 characters or fewer".into());
        }
        if let Some(ref t) = self.phone_type {
            if !VALID_PHONE_TYPES.contains(&t.as_str()) {
                errors.push(format!(
                    "phone_type '{}' is invalid; must be one of: {}",
                    t, VALID_PHONE_TYPES.join(", ")
                ));
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug, Serialize)]
pub struct PhoneResponse {
    pub id:              uuid::Uuid,
    pub user_id:         uuid::Uuid,
    pub phone_number:    String,
    pub country_code:    String,
    pub phone_type:      String,
    pub is_primary:      bool,
    pub can_receive_sms: bool,
    pub created_at:      chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PatchPhoneRequest {
    pub is_primary:      Option<bool>,
    pub can_receive_sms: Option<bool>,
    pub phone_type:      Option<String>,
}

impl PatchPhoneRequest {
    pub fn has_changes(&self) -> bool {
        self.is_primary.is_some() || self.can_receive_sms.is_some() || self.phone_type.is_some()
    }
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }
        if let Some(ref t) = self.phone_type {
            if !VALID_PHONE_TYPES.contains(&t.as_str()) {
                errors.push(format!(
                    "phone_type '{}' is invalid; must be one of: {}",
                    t, VALID_PHONE_TYPES.join(", ")
                ));
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ── POST /core/users/:id/addresses ────────────────────────────────────────────

/// Valid core.address_type enum values.
pub const VALID_ADDRESS_TYPES: &[&str] = &["physical_home", "mailing", "billing"];

#[derive(Debug, Deserialize)]
pub struct CreateAddressRequest {
    /// core.address_type enum
    pub address_type:        String,
    pub street_1:            String,
    pub street_2:            Option<String>,
    pub city:                String,
    pub state_province:      String,
    pub postal_code:         String,
    /// ISO 3166-1 alpha-2 — must exist in reference.countries
    pub country_iso_alpha2:  Option<String>,
    /// ISO 3166-2 — must exist in reference.subdivisions if provided
    pub subdivision_code:    Option<String>,
    /// IANA tz identifier — must exist in reference.timezones if provided
    pub timezone_identifier: Option<String>,
    pub effective_start_date: Option<chrono::NaiveDate>,
}

impl CreateAddressRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !VALID_ADDRESS_TYPES.contains(&self.address_type.as_str()) {
            errors.push(format!(
                "address_type '{}' is invalid; must be one of: {}",
                self.address_type, VALID_ADDRESS_TYPES.join(", ")
            ));
        }
        if self.street_1.trim().is_empty() { errors.push("street_1 is required".into()); }
        if self.city.trim().is_empty()     { errors.push("city is required".into()); }
        if self.state_province.trim().is_empty() { errors.push("state_province is required".into()); }
        if self.postal_code.trim().is_empty()    { errors.push("postal_code is required".into()); }
        if let Some(ref c) = self.country_iso_alpha2 {
            if c.trim().len() != 2 { errors.push("country_iso_alpha2 must be a 2-character ISO code".into()); }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug, Serialize)]
pub struct AddressResponse {
    pub id:                   uuid::Uuid,
    pub user_id:              uuid::Uuid,
    pub address_type:         String,
    pub street_1:             String,
    pub street_2:             Option<String>,
    pub city:                 String,
    pub state_province:       String,
    pub postal_code:          String,
    pub country_iso_alpha2:   Option<String>,
    pub subdivision_code:     Option<String>,
    pub timezone_identifier:  Option<String>,
    pub is_current:           bool,
    pub is_verified:          bool,
    pub effective_start_date: chrono::NaiveDate,
    pub effective_end_date:   Option<chrono::NaiveDate>,
    pub created_at:           chrono::DateTime<chrono::Utc>,
    pub updated_at:           chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PatchAddressRequest {
    pub street_1:            Option<String>,
    pub street_2:            MaybePatch<String>,
    pub city:                Option<String>,
    pub state_province:      Option<String>,
    pub postal_code:         Option<String>,
    pub country_iso_alpha2:  Option<String>,
    pub subdivision_code:    MaybePatch<String>,
    pub timezone_identifier: MaybePatch<String>,
    pub is_current:          Option<bool>,
    pub effective_end_date:  MaybePatch<chrono::NaiveDate>,
}

impl PatchAddressRequest {
    pub fn has_changes(&self) -> bool {
        self.street_1.is_some()
            || !matches!(self.street_2, MaybePatch::Absent)
            || self.city.is_some()
            || self.state_province.is_some()
            || self.postal_code.is_some()
            || self.country_iso_alpha2.is_some()
            || !matches!(self.subdivision_code, MaybePatch::Absent)
            || !matches!(self.timezone_identifier, MaybePatch::Absent)
            || self.is_current.is_some()
            || !matches!(self.effective_end_date, MaybePatch::Absent)
    }
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }
        if let Some(ref v) = self.street_1 {
            if v.trim().is_empty() { errors.push("street_1 cannot be blank".into()); }
        }
        if let Some(ref v) = self.city {
            if v.trim().is_empty() { errors.push("city cannot be blank".into()); }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 3 — EMERGENCY CONTACT WRITES
// POST   /core/users/:id/emergency-contacts
// PATCH  /core/users/:id/emergency-contacts/:contact_id
// DELETE /core/users/:id/emergency-contacts/:contact_id
// ═══════════════════════════════════════════════════════════════════════════════

// ── POST /core/users/:id/emergency-contacts ───────────────────────────────────

/// Create a new emergency contact for a user.
///
/// Actual table columns: first_name, last_name, relationship (varchar),
/// phone_number, email_address (nullable), is_primary.
/// No updated_at, no can_pickup, no notes columns exist.
///
/// If is_primary = true, the existing primary contact is demoted first.
#[derive(Debug, Deserialize)]
pub struct CreateEmergencyContactRequest {
    pub first_name:    String,
    pub last_name:     String,
    /// Free-text relationship description (e.g. "Mother", "Spouse", "Guardian")
    pub relationship:  String,
    pub phone_number:  String,
    pub email_address: Option<String>,
    /// Defaults to false. If true, demotes the current primary first.
    pub is_primary:    Option<bool>,
}

impl CreateEmergencyContactRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.first_name.trim().is_empty() {
            errors.push("first_name is required".into());
        }
        if self.first_name.len() > 100 {
            errors.push("first_name must be 100 characters or fewer".into());
        }
        if self.last_name.trim().is_empty() {
            errors.push("last_name is required".into());
        }
        if self.last_name.len() > 100 {
            errors.push("last_name must be 100 characters or fewer".into());
        }
        if self.relationship.trim().is_empty() {
            errors.push("relationship is required".into());
        }
        if self.relationship.len() > 100 {
            errors.push("relationship must be 100 characters or fewer".into());
        }
        if self.phone_number.trim().is_empty() {
            errors.push("phone_number is required".into());
        }
        if self.phone_number.len() > 50 {
            errors.push("phone_number must be 50 characters or fewer".into());
        }
        if let Some(ref e) = self.email_address {
            if !e.contains('@') {
                errors.push("email_address must be a valid email address".into());
            }
            if e.len() > 255 {
                errors.push("email_address must be 255 characters or fewer".into());
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Response for POST and PATCH emergency contact operations.
/// Mirrors the actual table columns exactly.
#[derive(Debug, Serialize)]
pub struct EmergencyContactResponse {
    pub id:            uuid::Uuid,
    pub user_id:       uuid::Uuid,
    pub first_name:    String,
    pub last_name:     String,
    pub relationship:  String,
    pub phone_number:  String,
    pub email_address: Option<String>,
    pub is_primary:    bool,
    pub created_at:    chrono::DateTime<chrono::Utc>,
}

// ── PATCH /core/users/:id/emergency-contacts/:contact_id ─────────────────────

/// Partial update for an emergency contact.
/// All fields are optional — absent fields retain current values.
/// is_primary promotion demotes the existing primary first.
#[derive(Debug, Deserialize)]
pub struct PatchEmergencyContactRequest {
    pub first_name:    Option<String>,
    pub last_name:     Option<String>,
    pub relationship:  Option<String>,
    pub phone_number:  Option<String>,
    pub email_address: MaybePatch<String>,
    pub is_primary:    Option<bool>,
}

impl PatchEmergencyContactRequest {
    pub fn has_changes(&self) -> bool {
        self.first_name.is_some()
            || self.last_name.is_some()
            || self.relationship.is_some()
            || self.phone_number.is_some()
            || !matches!(self.email_address, MaybePatch::Absent)
            || self.is_primary.is_some()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }
        if let Some(ref v) = self.first_name {
            if v.trim().is_empty() { errors.push("first_name cannot be blank".into()); }
            if v.len() > 100 { errors.push("first_name must be 100 characters or fewer".into()); }
        }
        if let Some(ref v) = self.last_name {
            if v.trim().is_empty() { errors.push("last_name cannot be blank".into()); }
            if v.len() > 100 { errors.push("last_name must be 100 characters or fewer".into()); }
        }
        if let Some(ref v) = self.relationship {
            if v.trim().is_empty() { errors.push("relationship cannot be blank".into()); }
            if v.len() > 100 { errors.push("relationship must be 100 characters or fewer".into()); }
        }
        if let Some(ref v) = self.phone_number {
            if v.trim().is_empty() { errors.push("phone_number cannot be blank".into()); }
            if v.len() > 50 { errors.push("phone_number must be 50 characters or fewer".into()); }
        }
        if let MaybePatch::Value(ref e) = self.email_address {
            if !e.contains('@') {
                errors.push("email_address must be a valid email address".into());
            }
            if e.len() > 255 {
                errors.push("email_address must be 255 characters or fewer".into());
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 4 — ROLE MANAGEMENT WRITES
// POST   /core/users/:id/roles
// DELETE /core/users/:id/roles/:role_name
// ═══════════════════════════════════════════════════════════════════════════════

/// Grant a role to a user within the current tenant.
///
/// user_roles has a composite PK of (tenant_id, user_id, role) — no surrogate id.
/// If the role has previously been granted and revoked, this re-grants it by
/// clearing revoked_at rather than inserting a duplicate row.
#[derive(Debug, Deserialize)]
pub struct GrantRoleRequest {
    /// core.system_role enum value.
    pub role:       String,
    /// Optional expiry. If provided, the role grant automatically lapses at
    /// this timestamp. Must be in the future.
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl GrantRoleRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !VALID_SYSTEM_ROLES.contains(&self.role.as_str()) {
            errors.push(format!(
                "role '{}' is invalid; must be one of: {}",
                self.role,
                VALID_SYSTEM_ROLES.join(", ")
            ));
        }
        if let Some(exp) = self.expires_at {
            if exp <= chrono::Utc::now() {
                errors.push("expires_at must be in the future".into());
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Response for a granted role — mirrors the user_roles row.
#[derive(Debug, Serialize)]
pub struct RoleGrantResponse {
    pub user_id:            uuid::Uuid,
    pub tenant_id:          uuid::Uuid,
    pub role:               String,
    pub granted_at:         chrono::DateTime<chrono::Utc>,
    pub granted_by_user_id: Option<uuid::Uuid>,
    pub expires_at:         Option<chrono::DateTime<chrono::Utc>>,
    pub revoked_at:         Option<chrono::DateTime<chrono::Utc>>,
}
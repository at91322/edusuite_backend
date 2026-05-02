// src/modules/hr/write_models.rs
//
// Request/response models for HR write operations.
// Existing CreateStaffRequest / UpdateStaffRequest preserved; new
// CreateContractRequest appended for POST /hr/staff/:id/contracts.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── MaybePatch (shared pattern) ───────────────────────────────────────────────

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
}

// ── POST /hr/staff (existing) ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ContractInput {
    pub contract_type:  String,
    pub start_date:     NaiveDate,
    pub end_date:       Option<NaiveDate>,
    pub job_title:      String,
    pub annual_salary:  Option<f64>,
    pub hourly_rate:    Option<f64>,
    pub position_id:    Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStaffRequest {
    pub username:              String,
    pub password:              String,
    pub first_name:            String,
    pub middle_name:           Option<String>,
    pub last_name:             String,
    pub preferred_name:        Option<String>,
    pub last_name_suffix:      Option<String>,
    pub institutional_email:   Option<String>,
    pub primary_department_id: Option<Uuid>,
    pub hire_date:             Option<NaiveDate>,
    pub is_tenured:            Option<bool>,
    pub contract:              Option<ContractInput>,
}

impl CreateStaffRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.username.trim().is_empty()   { errors.push("username is required".into()); }
        if self.password.len() < 8           { errors.push("password must be at least 8 characters".into()); }
        if self.first_name.trim().is_empty() { errors.push("first_name is required".into()); }
        if self.last_name.trim().is_empty()  { errors.push("last_name is required".into()); }
        if let Some(ref c) = self.contract {
            validate_contract_input(c, &mut errors);
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ── PATCH /hr/staff/:id (existing) ───────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct UpdateStaffRequest {
    #[serde(default)] pub first_name:            MaybePatch<String>,
    #[serde(default)] pub middle_name:           MaybePatch<String>,
    #[serde(default)] pub last_name:             MaybePatch<String>,
    #[serde(default)] pub preferred_name:        MaybePatch<String>,
    #[serde(default)] pub last_name_suffix:      MaybePatch<String>,
    #[serde(default)] pub institutional_email:   MaybePatch<String>,
    #[serde(default)] pub primary_department_id: MaybePatch<Uuid>,
    #[serde(default)] pub is_tenured:            MaybePatch<bool>,
}

impl UpdateStaffRequest {
    pub fn has_changes(&self) -> bool {
        !self.first_name.is_absent()          ||
        !self.middle_name.is_absent()         ||
        !self.last_name.is_absent()           ||
        !self.preferred_name.is_absent()      ||
        !self.last_name_suffix.is_absent()    ||
        !self.institutional_email.is_absent() ||
        !self.primary_department_id.is_absent() ||
        !self.is_tenured.is_absent()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if let MaybePatch::Value(ref v) = self.first_name {
            if v.trim().is_empty() { errors.push("first_name cannot be blank".into()); }
        }
        if let MaybePatch::Value(ref v) = self.last_name {
            if v.trim().is_empty() { errors.push("last_name cannot be blank".into()); }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// POST /hr/staff/:id/contracts ─────────────────────────────────────────────────

/// Request body for adding a new employment contract to an existing staff member.
///
/// When `deactivate_existing` is true (default), any currently active contract
/// for this staff member will be soft-deactivated (is_active = false) before
/// the new contract is inserted. This models the common HR workflow:
/// "renew contract" or "change role". Set to false only when explicitly adding
/// a secondary/concurrent contract (e.g. an adjunct stipend on top of a
/// full-time salary).
///
/// Compensation fields are nullable:
///   - `annual_salary`  → set for salaried / stipend contracts
///   - `hourly_rate`    → set for hourly contracts
/// Providing both is allowed (some contracts have both a salary floor and an
/// hourly overtime rate); providing neither is allowed for unpaid / stipend
/// contracts where the value is tracked elsewhere.
#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    /// hr.contract_type enum: 'salaried' | 'hourly' | 'stipend' | 'adjunct'
    pub contract_type:       String,
    pub start_date:          NaiveDate,
    /// None = open-ended / permanent; Some = fixed-term
    pub end_date:            Option<NaiveDate>,
    pub job_title:           String,
    pub annual_salary:       Option<f64>,
    pub hourly_rate:         Option<f64>,
    /// Optional link to a budgeted position in hr.budgeted_positions
    pub position_id:         Option<Uuid>,
    /// Deactivate any currently active contract before inserting. Default: true.
    #[serde(default = "default_true")]
    pub deactivate_existing: bool,
}

fn default_true() -> bool { true }

impl CreateContractRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        validate_contract_input_full(self, &mut errors);
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Response body for a newly created contract (201 Created).
#[derive(Debug, Serialize)]
pub struct ContractResponse {
    pub contract_id:    Uuid,
    pub staff_id:       Uuid,
    pub contract_type:  String,
    pub start_date:     NaiveDate,
    pub end_date:       Option<NaiveDate>,
    pub job_title:      String,
    pub annual_salary:  Option<f64>,
    pub hourly_rate:    Option<f64>,
    pub position_id:    Option<Uuid>,
    pub is_active:      bool,
    pub created_at:     chrono::DateTime<chrono::Utc>,
}

// ── Shared validation helpers ─────────────────────────────────────────────────

fn validate_contract_input(c: &ContractInput, errors: &mut Vec<String>) {
    if c.job_title.trim().is_empty() {
        errors.push("contract.job_title is required".into());
    }
    let valid_types = ["salaried", "hourly", "stipend", "adjunct"];
    if !valid_types.contains(&c.contract_type.as_str()) {
        errors.push(format!(
            "contract.contract_type '{}' is invalid; must be one of: {}",
            c.contract_type,
            valid_types.join(", ")
        ));
    }
    if let (Some(start), Some(end)) = (Some(c.start_date), c.end_date) {
        if end <= start {
            errors.push("contract.end_date must be after start_date".into());
        }
    }
}

fn validate_contract_input_full(c: &CreateContractRequest, errors: &mut Vec<String>) {
    if c.job_title.trim().is_empty() {
        errors.push("job_title is required".into());
    }
    let valid_types = ["salaried", "hourly", "stipend", "adjunct"];
    if !valid_types.contains(&c.contract_type.as_str()) {
        errors.push(format!(
            "contract_type '{}' is invalid; must be one of: {}",
            c.contract_type,
            valid_types.join(", ")
        ));
    }
    if let Some(end) = c.end_date {
        if end <= c.start_date {
            errors.push("end_date must be after start_date".into());
        }
    }
    if let Some(sal) = c.annual_salary {
        if sal < 0.0 { errors.push("annual_salary must be non-negative".into()); }
    }
    if let Some(hr) = c.hourly_rate {
        if hr < 0.0 { errors.push("hourly_rate must be non-negative".into()); }
    }
}
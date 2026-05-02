// src/modules/hr/write_models.rs

use serde::Deserialize;
use uuid::Uuid;

// ── Patch helper (duplicated from sis — avoids cross-module coupling) ─────────

#[derive(Debug)]
pub enum MaybePatch<T> {
    Absent,
    Present(T),
}

impl<T> Default for MaybePatch<T> {
    fn default() -> Self { MaybePatch::Absent }
}

pub fn deserialize_optional_field<'de, T, D>(
    deserializer: D,
) -> Result<MaybePatch<T>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    T::deserialize(deserializer).map(MaybePatch::Present)
}

// ── POST /hr/staff ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateStaffRequest {
    pub username:             String,
    pub password:             String,
    pub first_name:           String,
    pub middle_name:          Option<String>,
    pub last_name:            String,
    pub preferred_name:       Option<String>,
    pub last_name_suffix:     Option<String>,
    pub system_role:          String,
    pub institutional_email:  Option<String>,
    pub primary_department_id: Uuid,
    pub hire_date:             chrono::NaiveDate,
    pub is_tenured:            Option<bool>,
    pub contract:              Option<CreateContractRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub contract_type: String,
    pub job_title:     String,
    pub start_date:    chrono::NaiveDate,
    pub end_date:      Option<chrono::NaiveDate>,
    pub annual_salary: Option<f64>,
    pub hourly_rate:   Option<f64>,
}

impl CreateStaffRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let username = self.username.trim();
        if username.len() < 3 {
            errors.push("username must be at least 3 characters".into());
        } else if !username.chars().all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-')) {
            errors.push("username may only contain letters, numbers, '.', '_', '-'".into());
        }
        if self.password.len() < 8 {
            errors.push("password must be at least 8 characters".into());
        }
        if self.first_name.trim().is_empty() {
            errors.push("first_name is required".into());
        }
        if self.last_name.trim().is_empty() {
            errors.push("last_name is required".into());
        }
        let valid_roles = ["faculty", "staff", "tenant_admin"];
        if !valid_roles.contains(&self.system_role.as_str()) {
            errors.push(format!("system_role must be one of: {}", valid_roles.join(", ")));
        }
        if let Some(ref c) = self.contract {
            let valid_types = ["salaried", "hourly", "stipend"];
            if !valid_types.contains(&c.contract_type.as_str()) {
                errors.push(format!("contract.contract_type must be one of: {}", valid_types.join(", ")));
            }
            if c.job_title.trim().is_empty() {
                errors.push("contract.job_title is required".into());
            }
            if c.contract_type == "salaried" && c.annual_salary.is_none() {
                errors.push("contract.annual_salary is required for salaried contracts".into());
            }
            if c.contract_type == "hourly" && c.hourly_rate.is_none() {
                errors.push("contract.hourly_rate is required for hourly contracts".into());
            }
            if let Some(end) = c.end_date {
                if end <= c.start_date {
                    errors.push("contract.end_date must be after start_date".into());
                }
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ── PATCH /hr/staff/:id ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct UpdateStaffRequest {
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub first_name:       MaybePatch<String>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub middle_name:      MaybePatch<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub last_name:        MaybePatch<String>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub preferred_name:   MaybePatch<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub last_name_suffix: MaybePatch<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub institutional_email: MaybePatch<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub primary_department_id: MaybePatch<Uuid>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub hire_date:  MaybePatch<chrono::NaiveDate>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub is_tenured: MaybePatch<Option<bool>>,
}

impl UpdateStaffRequest {
    pub fn has_changes(&self) -> bool {
        !matches!(self.first_name,            MaybePatch::Absent) ||
        !matches!(self.middle_name,           MaybePatch::Absent) ||
        !matches!(self.last_name,             MaybePatch::Absent) ||
        !matches!(self.preferred_name,        MaybePatch::Absent) ||
        !matches!(self.last_name_suffix,      MaybePatch::Absent) ||
        !matches!(self.institutional_email,   MaybePatch::Absent) ||
        !matches!(self.primary_department_id, MaybePatch::Absent) ||
        !matches!(self.hire_date,             MaybePatch::Absent) ||
        !matches!(self.is_tenured,            MaybePatch::Absent)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if let MaybePatch::Present(ref v) = self.first_name {
            if v.trim().is_empty() { errors.push("first_name cannot be blank".into()); }
        }
        if let MaybePatch::Present(ref v) = self.last_name {
            if v.trim().is_empty() { errors.push("last_name cannot be blank".into()); }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
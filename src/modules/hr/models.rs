// src/modules/hr/models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── List params ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListStaffParams {
    /// Filter by department UUID
    pub department_id: Option<Uuid>,
    /// Filter by tenured status
    pub is_tenured: Option<bool>,
    /// Page number, 1-based (default: 1)
    pub page: Option<i64>,
    /// Page size (default: 25, max: 100)
    pub per_page: Option<i64>,
}

impl ListStaffParams {
    pub fn page(&self) -> i64 { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64 { (self.page() - 1) * self.per_page() }
}

// ── List response ─────────────────────────────────────────────────────────────

/// Summary record for the staff list — safe fields only.
/// Salary and hourly_rate are excluded; those are PII restricted to HR roles.
#[derive(Debug, Serialize)]
pub struct StaffSummary {
    pub user_id:         Uuid,
    pub first_name:      String,
    pub last_name:       String,
    pub preferred_name:  Option<String>,
    pub username:        String,
    pub job_title:       Option<String>,
    pub department_name: Option<String>,
    pub hire_date:       chrono::NaiveDate,
    pub is_tenured:      Option<bool>,
    pub contract_type:   Option<String>,
    pub system_role:     String,
}

#[derive(Debug, Serialize)]
pub struct StaffListResponse {
    pub data:        Vec<StaffSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

// ── Detail response ───────────────────────────────────────────────────────────

/// Full staff record. Salary excluded — HR-role gating deferred to future pass.
#[derive(Debug, Serialize)]
pub struct StaffDetail {
    // Identity
    pub user_id:          Uuid,
    pub username:         String,
    pub first_name:       String,
    pub middle_name:      Option<String>,
    pub last_name:        String,
    pub preferred_name:   Option<String>,
    pub last_name_suffix: Option<String>,

    // Institutional relationship
    pub system_role:         String,
    pub joined_at:           chrono::DateTime<chrono::Utc>,
    pub institutional_email: Option<String>,

    // HR profile
    pub hire_date:              chrono::NaiveDate,
    pub is_tenured:             Option<bool>,
    pub primary_department_id:  Uuid,
    pub primary_department:     Option<String>,

    // Active contract (None if no active contract exists)
    pub active_contract: Option<StaffContract>,
}

#[derive(Debug, Serialize)]
pub struct StaffContract {
    pub id:           Uuid,
    pub contract_type: String,
    pub job_title:    String,
    pub start_date:   chrono::NaiveDate,
    pub end_date:     Option<chrono::NaiveDate>,
}
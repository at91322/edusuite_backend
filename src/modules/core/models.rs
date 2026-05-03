// src/modules/core/models.rs
//
// Read models for the core module — Groups 1 and 6 (Step 1).
// Groups 2–5 models will be appended in subsequent steps.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 1a — TENANT SELF-SERVICE
// Tables: core.tenants, core.feature_flags
// ═══════════════════════════════════════════════════════════════════════════════

/// Full tenant configuration returned by GET /core/tenants/me.
/// Only exposes fields safe for the tenant to view and self-service.
/// Platform-internal fields (stripe IDs, tax config) are omitted.
#[derive(Debug, Serialize)]
pub struct TenantDetail {
    pub id:                   Uuid,
    pub name:                 String,
    pub domain:               String,
    pub country_iso_alpha2:   Option<String>,
    pub subdivision_code:     Option<String>,
    pub base_currency_code:   String,
    pub fiscal_year_end_month: i16,
    pub is_tax_exempt:        bool,
    pub created_at:           DateTime<Utc>,
    pub updated_at:           DateTime<Utc>,
}

/// Active module subscription summary.
#[derive(Debug, Serialize)]
pub struct SubscriptionSummary {
    pub id:            Uuid,
    pub module_name:   String,
    pub display_name:  String,
    pub tier:          String,
    pub status:        String,
    pub max_students:  Option<i32>,
    pub max_staff:     Option<i32>,
    pub activated_at:  Option<DateTime<Utc>>,
    pub trial_ends_at: Option<DateTime<Utc>>,
}

/// Feature flag visible to the tenant.
#[derive(Debug, Serialize)]
pub struct FeatureFlag {
    pub id:          Uuid,
    pub flag_name:   String,
    pub is_enabled:  bool,
    pub description: Option<String>,
    pub updated_at:  DateTime<Utc>,
}

// ── Group 1b: Department management ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListDepartmentsParams {
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

impl ListDepartmentsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(50).min(200).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

#[derive(Debug, Serialize)]
pub struct DepartmentSummary {
    pub id:           Uuid,
    pub code:         String,
    pub name:         String,
    pub head_user_id: Option<Uuid>,
    pub head_name:    Option<String>,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DepartmentListResponse {
    pub data:        Vec<DepartmentSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 6 — TENANT MEMBERSHIP MANAGEMENT
// Table: core.tenant_memberships (joined with core.users)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ListMembersParams {
    /// Filter by system_role enum value
    pub system_role:  Option<String>,
    /// Filter by name substring (case-insensitive)
    pub search:       Option<String>,
    pub page:         Option<i64>,
    pub per_page:     Option<i64>,
}

impl ListMembersParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

#[derive(Debug, Serialize)]
pub struct MemberSummary {
    pub membership_id:         Uuid,
    pub user_id:               Uuid,
    pub username:              String,
    pub first_name:            String,
    pub last_name:             String,
    pub preferred_name:        Option<String>,
    pub institutional_email:   Option<String>,
    pub system_role:           String,
    pub joined_at:             DateTime<Utc>,
    pub last_accessed_at:      Option<DateTime<Utc>>,
    pub is_active:             bool,
}

#[derive(Debug, Serialize)]
pub struct MemberListResponse {
    pub data:        Vec<MemberSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 2 — USER IDENTITY (Step 2 — stubs for mod.rs route declarations)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct UserDetail {
    pub id:             Uuid,
    pub username:       String,
    pub first_name:     String,
    pub middle_name:    Option<String>,
    pub last_name:      String,
    pub preferred_name: Option<String>,
    pub suffix:         Option<String>,
    pub is_active:      bool,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct NameHistoryEntry {
    pub id:                   Uuid,
    pub historical_first_name: String,
    pub historical_middle_name: Option<String>,
    pub historical_last_name:  String,
    pub reason:               String,
    pub changed_by_user_id:   Option<Uuid>,
    pub changed_by_name:      Option<String>,
    pub changed_at:           DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserEmail {
    pub id:            Uuid,
    pub email_address: String,
    pub email_type:    String,
    pub is_primary:    bool,
    pub is_verified:   bool,
    pub created_at:    DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserPhone {
    pub id:              Uuid,
    pub phone_number:    String,
    pub country_code:    String,
    pub phone_type:      String,
    pub is_primary:      bool,
    pub can_receive_sms: bool,
    pub created_at:      DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserAddress {
    pub id:                   Uuid,
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
    pub created_at:           DateTime<Utc>,
    pub updated_at:           DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 3 — EMERGENCY CONTACTS (Step 3 — stubs)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct EmergencyContact {
    pub id:               Uuid,
    pub user_id:          Uuid,
    pub name:             String,
    pub relationship:     String,
    pub phone_primary:    String,
    pub phone_secondary:  Option<String>,
    pub email:            Option<String>,
    pub is_primary:       bool,
    pub can_pickup:       bool,
    pub notes:            Option<String>,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 4 — ROLE MANAGEMENT (Step 4 — stubs)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct RoleGrant {
    pub user_id:            Uuid,
    pub tenant_id:          Uuid,
    pub role:               String,
    pub granted_at:         DateTime<Utc>,
    pub granted_by_user_id: Option<Uuid>,
    pub granted_by_name:    Option<String>,
    pub expires_at:         Option<DateTime<Utc>>,
    pub revoked_at:         Option<DateTime<Utc>>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 5 — AUDIT LOG (Step 5 — stubs)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ListAuditLogsParams {
    pub table_name:  Option<String>,
    pub actor_id:    Option<Uuid>,
    pub date_from:   Option<chrono::NaiveDate>,
    pub date_to:     Option<chrono::NaiveDate>,
    pub operation:   Option<String>,
    pub page:        Option<i64>,
    pub per_page:    Option<i64>,
}

impl ListAuditLogsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(50).min(200).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id:           Uuid,
    pub schema_name:  String,
    pub table_name:   String,
    pub operation:    String,
    pub actor_id:     Option<Uuid>,
    pub actor_name:   Option<String>,
    pub record_id:    Option<serde_json::Value>,
    pub old_data:     Option<serde_json::Value>,
    pub new_data:     Option<serde_json::Value>,
    pub created_at:   DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogListResponse {
    pub data:        Vec<AuditLogEntry>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}
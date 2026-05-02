// src/modules/finance/models.rs
//
// Read models for the finance module.
// All amounts are f64 — numeric(x,y) columns are cast to float8 in SQL
// via the established pattern in this codebase (no BigDecimal dependency).
// Dates are chrono::NaiveDate; timestamps are chrono::DateTime<chrono::Utc>.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 1 — STUDENT BILLING
// Tables: finance.student_accounts, finance.student_transactions
// ═══════════════════════════════════════════════════════════════════════════════

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListStudentAccountsParams {
    pub student_id:     Option<Uuid>,
    pub is_hold_active: Option<bool>,
    pub page:           Option<i64>,
    pub per_page:       Option<i64>,
}

impl ListStudentAccountsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

#[derive(Debug, Deserialize)]
pub struct ListTransactionsParams {
    pub term_id:          Option<Uuid>,
    /// Filter by transaction type, e.g. "tuition_charge", "student_payment"
    pub transaction_type: Option<String>,
    pub date_from:        Option<NaiveDate>,
    pub date_to:          Option<NaiveDate>,
    pub page:             Option<i64>,
    pub per_page:         Option<i64>,
}

impl ListTransactionsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(50).min(200).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

// ── Response models ───────────────────────────────────────────────────────────

/// Summary of a student's financial account — balance + hold status.
#[derive(Debug, Serialize)]
pub struct StudentAccountSummary {
    pub id:              Uuid,
    pub student_id:      Uuid,
    pub student_name:    String,
    pub current_balance: f64,
    pub is_hold_active:  bool,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StudentAccountListResponse {
    pub data:        Vec<StudentAccountSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

/// Single immutable transaction entry on a student account.
/// Positive amount = charge; negative amount = credit/payment.
#[derive(Debug, Serialize)]
pub struct StudentTransaction {
    pub id:               Uuid,
    pub account_id:       Uuid,
    pub term_id:          Option<Uuid>,
    pub term_name:        Option<String>,
    pub gl_account_id:    Uuid,
    pub gl_account_name:  String,
    pub transaction_type: String,
    pub amount:           f64,
    pub description:      String,
    pub reference_number: Option<String>,
    pub transaction_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TransactionListResponse {
    pub data:            Vec<StudentTransaction>,
    pub account_balance: f64,
    pub page:            i64,
    pub per_page:        i64,
    pub total:           i64,
    pub total_pages:     i64,
}

/// Simple hold status payload for GET /student-accounts/:id/hold
#[derive(Debug, Serialize)]
pub struct HoldStatus {
    pub account_id:     Uuid,
    pub student_id:     Uuid,
    pub is_hold_active: bool,
    pub current_balance: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 2 — FINANCIAL AID
// Tables: finance.fafsa_records, finance.financial_aid_awards
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ListAidAwardsParams {
    pub term_id:   Option<Uuid>,
    /// Filter by award status: offered, accepted, disbursed, cancelled
    pub status:    Option<String>,
    /// Filter by aid_type: pell_grant, subsidized_loan, unsubsidized_loan, etc.
    pub aid_type:  Option<String>,
    pub page:      Option<i64>,
    pub per_page:  Option<i64>,
}

impl ListAidAwardsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

/// FAFSA record as received from the Department of Education (ISIR).
#[derive(Debug, Serialize)]
pub struct FafsaRecord {
    pub id:                       Uuid,
    pub student_id:               Uuid,
    pub aid_year:                 String,
    pub student_aid_index:        i32,
    pub pell_eligibility_flag:    bool,
    pub verification_required:    bool,
    pub selected_for_verification: bool,
    pub verification_group:       Option<String>,
    pub dependency_status:        String,
    pub c_flag:                   bool,
    pub isir_transaction_number:  Option<i16>,
    pub received_date:            NaiveDate,
    pub updated_at:               DateTime<Utc>,
}

/// Individual aid award within a student's financial aid package.
#[derive(Debug, Serialize)]
pub struct AidAward {
    pub id:                  Uuid,
    pub student_id:          Uuid,
    pub term_id:             Uuid,
    pub term_name:           String,
    pub fund_name:           String,
    pub aid_type:            String,
    pub offered_amount:      f64,
    pub accepted_amount:     f64,
    pub disbursed_amount:    f64,
    pub status:              String,
    /// Loan-specific fields — None for grants/scholarships
    pub nslds_loan_type:     Option<String>,
    pub loan_sequence_number: Option<i16>,
    pub loan_period_begin_date: Option<NaiveDate>,
    pub loan_period_end_date:   Option<NaiveDate>,
    pub grade_level_at_award:  Option<i16>,
    pub created_at:          DateTime<Utc>,
    pub updated_at:          DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AidAwardListResponse {
    pub data:        Vec<AidAward>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 3 — SAP (Satisfactory Academic Progress)
// Tables: finance.sap_policies, finance.sap_evaluations, finance.sap_appeals
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ListSapEvaluationsParams {
    pub term_id:  Option<Uuid>,
    /// Filter by status: satisfactory, warning, suspension, ineligible
    pub status:   Option<String>,
    pub page:     Option<i64>,
    pub per_page: Option<i64>,
}

impl ListSapEvaluationsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

/// SAP policy configuration — thresholds and evaluation rules.
#[derive(Debug, Serialize)]
pub struct SapPolicy {
    pub id:                           Uuid,
    pub name:                         String,
    pub is_default:                   bool,
    pub academic_program_id:          Option<Uuid>,
    pub min_cumulative_gpa:           f64,
    pub min_pace_percentage:          f64,
    pub max_timeframe_multiplier:     f64,
    pub evaluation_frequency:         String,
    pub min_credits_for_evaluation:   f64,
    pub transfer_credits_count_in_pace: bool,
    pub remedial_credits_count_in_pace: bool,
    pub warning_terms_before_suspension: i16,
    pub timeframe_warning_1_pct:      f64,
    pub timeframe_warning_2_pct:      f64,
    pub timeframe_warning_3_pct:      f64,
    pub is_active:                    bool,
    pub created_at:                   DateTime<Utc>,
    pub updated_at:                   DateTime<Utc>,
}

/// SAP evaluation result for one student at the end of one term.
/// Credit snapshots are immutable — the policy thresholds that governed
/// this evaluation are captured at evaluation time.
#[derive(Debug, Serialize)]
pub struct SapEvaluation {
    pub id:                          Uuid,
    pub student_id:                  Uuid,
    pub evaluation_term_id:          Uuid,
    pub term_name:                   String,
    pub snapshot_cumulative_gpa:     f64,
    pub snapshot_attempted_credits:  f64,
    pub snapshot_earned_credits:     f64,
    pub snapshot_pace_percentage:    f64,
    pub snapshot_max_timeframe_credits: Option<f64>,
    pub resulting_sap_status:        String,
    pub gpa_component_met:           Option<bool>,
    pub pace_component_met:          Option<bool>,
    pub max_timeframe_component_met: Option<bool>,
    pub consecutive_warning_terms:   i16,
    pub is_manual_override:          bool,
    pub override_reason:             Option<String>,
    pub sap_policy_id:               Option<Uuid>,
    pub evaluated_at:                DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SapEvaluationListResponse {
    pub data:        Vec<SapEvaluation>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

/// A SAP appeal filed by a student against a failing evaluation.
#[derive(Debug, Serialize)]
pub struct SapAppeal {
    pub id:                  Uuid,
    pub sap_evaluation_id:   Uuid,
    pub student_id:          Uuid,
    pub appeal_reason:       String,
    pub supporting_documents: Option<serde_json::Value>,
    pub status:              String,
    pub reviewer_id:         Option<Uuid>,
    pub reviewer_notes:      Option<String>,
    pub reviewed_at:         Option<DateTime<Utc>>,
    pub probation_expires_term_id: Option<Uuid>,
    pub created_at:          DateTime<Utc>,
    pub updated_at:          DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 4 — VA BENEFITS
// Tables: finance.veteran_profiles, finance.va_certifications
// ═══════════════════════════════════════════════════════════════════════════════

/// VA benefit eligibility record for a student veteran.
#[derive(Debug, Serialize)]
pub struct VeteranProfile {
    pub id:                             Uuid,
    pub student_id:                     Uuid,
    pub student_name:                   String,
    pub va_file_number:                 Option<String>,
    pub primary_chapter:                String,
    pub eligibility_percentage:         Option<i32>,
    pub months_of_entitlement_remaining: Option<f64>,
    pub dd214_on_file:                  bool,
    pub updated_at:                     DateTime<Utc>,
}

/// VA enrollment certification submitted by the School Certifying Official.
#[derive(Debug, Serialize)]
pub struct VaCertification {
    pub id:                       Uuid,
    pub student_id:               Uuid,
    pub student_name:             String,
    pub veteran_profile_id:       Uuid,
    pub term_id:                  Uuid,
    pub term_name:                String,
    pub term_start:               NaiveDate,
    pub term_end:                 NaiveDate,
    pub credits_certified:        f64,
    pub tuition_reported:         f64,
    pub fees_reported:            f64,
    pub certification_date:       NaiveDate,
    pub enrollment_intensity_va:  Option<String>,
    pub training_time_percentage: Option<f64>,
    pub is_amendment:             bool,
    pub amends_certification_id:  Option<Uuid>,
    pub amendment_date:           Option<NaiveDate>,
    pub va_confirmation_number:   Option<String>,
    pub ch33_housing_rate:        Option<f64>,
    pub ch33_book_stipend:        Option<f64>,
    pub program_of_study:         Option<String>,
    pub certified_by:             Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VaCertificationListResponse {
    pub data:        Vec<VaCertification>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListVaCertificationsParams {
    pub term_id:      Option<Uuid>,
    pub is_amendment: Option<bool>,
    pub page:         Option<i64>,
    pub per_page:     Option<i64>,
}

impl ListVaCertificationsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 5 — PAYROLL / STAFF PAY
// Tables: finance.payroll_runs, finance.pay_stubs, finance.pay_stub_line_items,
//         finance.payroll_item_types
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ListPayrollRunsParams {
    /// Filter by status: pending, processing, approved, disbursed
    pub status:    Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to:   Option<NaiveDate>,
    pub page:      Option<i64>,
    pub per_page:  Option<i64>,
}

impl ListPayrollRunsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

#[derive(Debug, Deserialize)]
pub struct ListStaffPayStubsParams {
    pub date_from: Option<NaiveDate>,
    pub date_to:   Option<NaiveDate>,
    pub page:      Option<i64>,
    pub per_page:  Option<i64>,
}

impl ListStaffPayStubsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

/// Summary of a payroll run — used in the list response.
#[derive(Debug, Serialize)]
pub struct PayrollRunSummary {
    pub id:              Uuid,
    pub run_date:        NaiveDate,
    pub total_gross_pay: f64,
    pub gl_account_id:   Uuid,
    pub gl_account_name: String,
    pub status:          String,
    pub stub_count:      i64,
    pub updated_at:      DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PayrollRunListResponse {
    pub data:        Vec<PayrollRunSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

/// Summary of a single pay stub within a run — used in list-by-run.
#[derive(Debug, Serialize)]
pub struct PayStubSummary {
    pub id:              Uuid,
    pub payroll_run_id:  Uuid,
    pub employee_id:     Uuid,
    pub employee_name:   String,
    pub gross_pay:       f64,
    pub net_pay:         f64,
    pub created_at:      DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PayStubListResponse {
    pub data:        Vec<PayStubSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

/// A single line item on a pay stub (earnings, deductions, employer liabilities).
#[derive(Debug, Serialize)]
pub struct PayStubLineItem {
    pub id:             Uuid,
    pub item_type_id:   Uuid,
    pub historical_name: String,
    /// Category: earnings | deduction | employer_liability
    pub category:       String,
    pub amount:         f64,
}

/// Full pay stub detail with all line items.
#[derive(Debug, Serialize)]
pub struct PayStubDetail {
    pub id:              Uuid,
    pub payroll_run_id:  Uuid,
    pub run_date:        NaiveDate,
    pub employee_id:     Uuid,
    pub employee_name:   String,
    pub gross_pay:       f64,
    pub net_pay:         f64,
    pub line_items:      Vec<PayStubLineItem>,
    pub created_at:      DateTime<Utc>,
}

/// A configurable payroll component type (salary, health deduction, FICA, etc.)
#[derive(Debug, Serialize)]
pub struct PayrollItemType {
    pub id:                     Uuid,
    pub name:                   String,
    pub category:               String,
    pub liability_gl_account_id: Uuid,
    pub is_active:              Option<bool>,
    pub created_at:             DateTime<Utc>,
    pub updated_at:             DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 6 — REFERENCE READS
// Tables: finance.fiscal_years, finance.gl_accounts
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ListGlAccountsParams {
    /// Filter by GL account type: revenue, expense, asset, liability, equity
    pub account_type: Option<String>,
    pub is_active:    Option<bool>,
    pub page:         Option<i64>,
    pub per_page:     Option<i64>,
}

impl ListGlAccountsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(50).min(200).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

/// Fiscal year lifecycle record.
#[derive(Debug, Serialize)]
pub struct FiscalYear {
    pub id:                   Uuid,
    pub name:                 String,
    pub start_date:           NaiveDate,
    pub end_date:             NaiveDate,
    pub status:               String,
    pub accounting_framework: Option<String>,
    pub audit_status:         Option<String>,
    pub is_comparative_year:  bool,
    pub period_closed_at:     Option<DateTime<Utc>>,
}

/// GL account summary — used for validation when posting transactions.
#[derive(Debug, Serialize)]
pub struct GlAccountSummary {
    pub id:             Uuid,
    pub account_number: String,
    pub account_name:   String,
    pub account_code:   Option<String>,
    pub account_type:   String,
    pub normal_balance: Option<String>,
    pub current_balance: f64,
    pub is_active:      bool,
    pub is_contra:      bool,
    pub description:    Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GlAccountListResponse {
    pub data:        Vec<GlAccountSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}
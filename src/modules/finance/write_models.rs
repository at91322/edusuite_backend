// src/modules/finance/write_models.rs
//
// Request and response types for Step 2 finance writes:
//   - POST /finance/students/:student_id/account
//   - POST /finance/student-accounts/:id/transactions
//   - PATCH /finance/student-accounts/:id/hold

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════════
// POST /finance/students/:student_id/account
// Creates the student_accounts row. One per student per tenant.
// ═══════════════════════════════════════════════════════════════════════════════

/// No body needed — the student_id comes from the URL path and the tenant
/// from the JWT. Current balance defaults to 0.00 and hold to false.
/// Keeping this as an explicit empty struct (rather than ()) makes it easy
/// to add optional fields later (e.g. opening_balance for migrations).
#[derive(Debug, Deserialize, Default)]
pub struct CreateStudentAccountRequest {
    /// Optional opening balance for data migration scenarios. Defaults to 0.00.
    /// Must be >= 0 — negative opening balances should be posted as transactions.
    pub opening_balance: Option<f64>,
}

impl CreateStudentAccountRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if let Some(b) = self.opening_balance {
            if b < 0.0 {
                errors.push(
                    "opening_balance must be >= 0; post a transaction to record a credit balance"
                    .into()
                );
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// 201 response for account creation.
#[derive(Debug, Serialize)]
pub struct CreateStudentAccountResponse {
    pub account_id:      Uuid,
    pub student_id:      Uuid,
    pub current_balance: f64,
    pub is_hold_active:  bool,
    pub created_at:      chrono::DateTime<chrono::Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// POST /finance/student-accounts/:id/transactions
// Posts an immutable charge or credit to the student ledger.
// ═══════════════════════════════════════════════════════════════════════════════

/// Request body for posting a transaction.
///
/// Sign convention (matches finance.student_transactions schema comment):
///   Positive amount  = charge owed by student  (tuition_charge, fee_charge, etc.)
///   Negative amount  = credit / payment        (student_payment, refund, aid_disbursement)
///
/// The API enforces consistency between type and sign:
///   Charge types     (tuition_charge, fee_charge, room_board_charge)  → amount MUST be > 0
///   Payment/credit   (student_payment, third_party_payment, refund,
///                     aid_disbursement)                                → amount MUST be < 0
///   adjustment       → any non-zero amount (can be positive or negative)
///
/// This keeps the sign convention self-documenting and prevents accidental
/// double-negatives (e.g. posting a payment as a positive amount).
#[derive(Debug, Deserialize)]
pub struct PostTransactionRequest {
    /// finance.transaction_type enum value.
    /// Valid values: tuition_charge | fee_charge | room_board_charge |
    ///               student_payment | third_party_payment | aid_disbursement |
    ///               refund | adjustment
    pub transaction_type: String,

    /// Non-zero dollar amount. Sign must match transaction_type (see above).
    pub amount: f64,

    /// Human-readable description shown on student billing statements. Max 255 chars.
    pub description: String,

    /// Optional GL account to debit/credit. If omitted, the write_query will
    /// look up the default GL account for this transaction type from the tenant
    /// configuration. Providing it explicitly is required for non-standard GL routing.
    pub gl_account_id: Uuid,

    /// Optional term this charge/credit belongs to.
    pub term_id: Option<Uuid>,

    /// Optional external reference (enrollment ID, award ID, check number, etc.)
    /// Stored for audit trail but not validated by the API.
    pub reference_number: Option<String>,

    /// Override the transaction date. Defaults to now(). Useful for backdating
    /// charges during term setup. Must not be in the future.
    pub transaction_date: Option<NaiveDate>,
}

/// The complete set of valid transaction_type values from the DB enum.
pub const VALID_TRANSACTION_TYPES: &[&str] = &[
    "tuition_charge",
    "fee_charge",
    "room_board_charge",
    "student_payment",
    "third_party_payment",
    "aid_disbursement",
    "refund",
    "adjustment",
];

/// Types that must have a positive amount (charges).
pub const CHARGE_TYPES: &[&str] = &[
    "tuition_charge",
    "fee_charge",
    "room_board_charge",
];

/// Types that must have a negative amount (payments / credits).
pub const CREDIT_TYPES: &[&str] = &[
    "student_payment",
    "third_party_payment",
    "aid_disbursement",
    "refund",
];

impl PostTransactionRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // ── transaction_type ────────────────────────────────────────────────
        if !VALID_TRANSACTION_TYPES.contains(&self.transaction_type.as_str()) {
            errors.push(format!(
                "transaction_type '{}' is invalid; must be one of: {}",
                self.transaction_type,
                VALID_TRANSACTION_TYPES.join(", ")
            ));
        }

        // ── amount ──────────────────────────────────────────────────────────
        if self.amount == 0.0 {
            errors.push("amount must be non-zero".into());
        }

        if CHARGE_TYPES.contains(&self.transaction_type.as_str()) && self.amount < 0.0 {
            errors.push(format!(
                "transaction_type '{}' is a charge and requires a positive amount",
                self.transaction_type
            ));
        }

        if CREDIT_TYPES.contains(&self.transaction_type.as_str()) && self.amount > 0.0 {
            errors.push(format!(
                "transaction_type '{}' is a payment/credit and requires a negative amount",
                self.transaction_type
            ));
        }

        // ── description ─────────────────────────────────────────────────────
        if self.description.trim().is_empty() {
            errors.push("description is required".into());
        }
        if self.description.len() > 255 {
            errors.push("description must be 255 characters or fewer".into());
        }

        // ── transaction_date ─────────────────────────────────────────────────
        if let Some(d) = self.transaction_date {
            if d > chrono::Local::now().date_naive() {
                errors.push("transaction_date cannot be in the future".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// 201 response for a posted transaction.
/// Includes the updated account balance so callers don't need a second request.
#[derive(Debug, Serialize)]
pub struct PostTransactionResponse {
    pub transaction_id:      Uuid,
    pub account_id:          Uuid,
    pub transaction_type:    String,
    pub amount:              f64,
    pub description:         String,
    pub reference_number:    Option<String>,
    pub gl_account_id:       Uuid,
    pub term_id:             Option<Uuid>,
    pub transaction_date:    chrono::DateTime<chrono::Utc>,
    /// Updated balance after this transaction was applied.
    pub updated_balance:     f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PATCH /finance/student-accounts/:id/hold
// Sets or clears the registration hold on a student account.
// ═══════════════════════════════════════════════════════════════════════════════

/// Hold changes are always accompanied by a mandatory reason so there is
/// an audit trail for every hold placement and release.
#[derive(Debug, Deserialize)]
pub struct SetHoldRequest {
    pub is_hold_active: bool,
    /// Required explanation — why the hold is being placed or cleared.
    /// Stored in core.audit_log by the handler, not in student_accounts itself
    /// (student_accounts has no hold_reason column).
    pub reason: String,
}

impl SetHoldRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.reason.trim().is_empty() {
            errors.push("reason is required when changing hold status".into());
        }
        if self.reason.len() > 500 {
            errors.push("reason must be 500 characters or fewer".into());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// 200 response after a hold change.
#[derive(Debug, Serialize)]
pub struct SetHoldResponse {
    pub account_id:      Uuid,
    pub student_id:      Uuid,
    pub is_hold_active:  bool,
    pub current_balance: f64,
    pub updated_at:      chrono::DateTime<chrono::Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 3 — FINANCIAL AID WRITES
// POST /finance/students/:student_id/aid-awards
// PATCH /finance/students/:student_id/aid-awards/:id
// ═══════════════════════════════════════════════════════════════════════════════

/// Valid aid_source_type values (finance.aid_source_type enum).
pub const VALID_AID_TYPES: &[&str] = &[
    "federal_grant",
    "federal_loan",
    "state_grant",
    "institutional_scholarship",
    "private_loan",
    "work_study",
];

/// Valid award status values (varchar(50), not a PG enum).
pub const VALID_AWARD_STATUSES: &[&str] = &[
    "offered",
    "accepted",
    "disbursed",
    "cancelled",
];

/// Valid nslds_loan_type values (finance.nslds_loan_type enum).
/// Required for federal_loan and federal_grant awards that originate loans.
pub const VALID_NSLDS_LOAN_TYPES: &[&str] = &[
    "SUB", "UNSUB", "PLUS", "GPLUS", "PERK", "HEAL", "NURS", "INST", "PRIV",
];

// ── POST /finance/students/:student_id/aid-awards ────────────────────────────

/// Package a new aid award for a student for a specific term.
///
/// Aid type drives which fields are required:
///   federal_loan / private_loan  → nslds_loan_type required for federal loans;
///                                  loan_period dates strongly recommended
///   All others                   → loan fields are silently ignored if provided
///
/// offered_amount must be > 0. accepted_amount and disbursed_amount start at
/// 0 and are updated via PATCH. Status defaults to "offered".
#[derive(Debug, Deserialize)]
pub struct CreateAidAwardRequest {
    pub term_id:       Uuid,
    /// Human-readable fund name (e.g. "Federal Pell Grant", "Subsidized Stafford")
    pub fund_name:     String,
    /// finance.aid_source_type enum value
    pub aid_type:      String,
    /// Must be > 0
    pub offered_amount: f64,
    /// Optional: defaults to "offered"
    pub status:        Option<String>,

    // ── Loan-specific fields (required for federal_loan awards) ─────────
    /// finance.nslds_loan_type enum — required when aid_type = "federal_loan"
    pub nslds_loan_type:        Option<String>,
    pub loan_sequence_number:   Option<i16>,
    pub loan_period_begin_date: Option<NaiveDate>,
    pub loan_period_end_date:   Option<NaiveDate>,
    /// 1=Freshman, 2=Sophomore, 3=Junior, 4=Senior, 5=5th year, 6=Graduate
    pub grade_level_at_award:   Option<i16>,
}

impl CreateAidAwardRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.fund_name.trim().is_empty() {
            errors.push("fund_name is required".into());
        }
        if self.fund_name.len() > 100 {
            errors.push("fund_name must be 100 characters or fewer".into());
        }

        if !VALID_AID_TYPES.contains(&self.aid_type.as_str()) {
            errors.push(format!(
                "aid_type '{}' is invalid; must be one of: {}",
                self.aid_type,
                VALID_AID_TYPES.join(", ")
            ));
        }

        if self.offered_amount <= 0.0 {
            errors.push("offered_amount must be greater than zero".into());
        }

        if let Some(ref s) = self.status {
            if !VALID_AWARD_STATUSES.contains(&s.as_str()) {
                errors.push(format!(
                    "status '{}' is invalid; must be one of: {}",
                    s,
                    VALID_AWARD_STATUSES.join(", ")
                ));
            }
            // Cannot create directly in disbursed state
            if s == "disbursed" {
                errors.push(
                    "awards cannot be created with status 'disbursed'; use PATCH to disburse"
                    .into()
                );
            }
        }

        // federal_loan requires nslds_loan_type
        if self.aid_type == "federal_loan" && self.nslds_loan_type.is_none() {
            errors.push(
                "nslds_loan_type is required for federal_loan awards".into()
            );
        }

        if let Some(ref lt) = self.nslds_loan_type {
            if !VALID_NSLDS_LOAN_TYPES.contains(&lt.as_str()) {
                errors.push(format!(
                    "nslds_loan_type '{}' is invalid; must be one of: {}",
                    lt,
                    VALID_NSLDS_LOAN_TYPES.join(", ")
                ));
            }
        }

        if let Some(seq) = self.loan_sequence_number {
            if seq < 1 {
                errors.push("loan_sequence_number must be >= 1".into());
            }
        }

        if let (Some(begin), Some(end)) = (self.loan_period_begin_date, self.loan_period_end_date) {
            if end <= begin {
                errors.push("loan_period_end_date must be after loan_period_begin_date".into());
            }
        }

        if let Some(gl) = self.grade_level_at_award {
            if !(1..=6).contains(&gl) {
                errors.push("grade_level_at_award must be between 1 and 6".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// 201 response for a created aid award.
#[derive(Debug, Serialize)]
pub struct AidAwardResponse {
    pub award_id:              Uuid,
    pub student_id:            Uuid,
    pub term_id:               Uuid,
    pub fund_name:             String,
    pub aid_type:              String,
    pub offered_amount:        f64,
    pub accepted_amount:       f64,
    pub disbursed_amount:      f64,
    pub status:                String,
    pub nslds_loan_type:       Option<String>,
    pub loan_sequence_number:  Option<i16>,
    pub loan_period_begin_date: Option<NaiveDate>,
    pub loan_period_end_date:  Option<NaiveDate>,
    pub grade_level_at_award:  Option<i16>,
    pub created_at:            chrono::DateTime<chrono::Utc>,
    pub updated_at:            chrono::DateTime<chrono::Utc>,
}

// ── PATCH /finance/students/:student_id/aid-awards/:id ──────────────────────

/// Update an existing aid award — primarily drives the status machine.
///
/// Status transitions:
///   offered    → accepted   : set accepted_amount (defaults to offered_amount)
///   accepted   → disbursed  : set disbursed_amount (defaults to accepted_amount)
///                             SIDE EFFECT: posts an aid_disbursement transaction
///                             to the student ledger in the same DB transaction
///   any        → cancelled  : no amount changes required
///   disbursed  → *          : BLOCKED — disbursed awards are immutable
///
/// Only fields present in the request body are updated (partial PATCH semantics).
/// Sending accepted_amount on an already-accepted award revises the acceptance
/// without a status change (useful for award revision workflows).
#[derive(Debug, Deserialize)]
pub struct UpdateAidAwardRequest {
    /// New status. Drives side effects (see above).
    pub status:            Option<String>,
    /// New accepted amount. Required when transitioning to "accepted".
    pub accepted_amount:   Option<f64>,
    /// New disbursed amount. Required when transitioning to "disbursed".
    /// Must be <= accepted_amount.
    pub disbursed_amount:  Option<f64>,
    /// GL account for the aid_disbursement transaction.
    /// Required when status = "disbursed".
    pub disbursement_gl_account_id: Option<Uuid>,
}

impl UpdateAidAwardRequest {
    pub fn has_changes(&self) -> bool {
        self.status.is_some()
            || self.accepted_amount.is_some()
            || self.disbursed_amount.is_some()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }

        if let Some(ref s) = self.status {
            if !VALID_AWARD_STATUSES.contains(&s.as_str()) {
                errors.push(format!(
                    "status '{}' is invalid; must be one of: {}",
                    s,
                    VALID_AWARD_STATUSES.join(", ")
                ));
            }
        }

        if let Some(amt) = self.accepted_amount {
            if amt < 0.0 {
                errors.push("accepted_amount must be >= 0".into());
            }
        }

        if let Some(amt) = self.disbursed_amount {
            if amt < 0.0 {
                errors.push("disbursed_amount must be >= 0".into());
            }
        }

        // If transitioning to disbursed, gl_account_id is required
        if self.status.as_deref() == Some("disbursed")
            && self.disbursement_gl_account_id.is_none()
        {
            errors.push(
                "disbursement_gl_account_id is required when status = 'disbursed'".into()
            );
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 4 — SAP APPEAL + VA CERTIFICATION WRITES
// POST /finance/students/:student_id/sap/:id/appeal
// POST /finance/students/:student_id/va-certifications
// PATCH /finance/students/:student_id/va-certifications/:id
// ═══════════════════════════════════════════════════════════════════════════════

// ── POST /finance/students/:student_id/sap/:id/appeal ────────────────────────

/// Submit a SAP appeal for a failing evaluation.
///
/// sap_appeals.workflow_submission_id is NOT NULL — the write query creates
/// a workflow.submissions row first and links it here. The caller supplies
/// the form_id for the SAP appeal workflow form configured for this tenant.
///
/// appeal_reason is stored in the workflow submission payload_data (JSONB)
/// rather than directly on sap_appeals (which has no text reason column).
#[derive(Debug, Deserialize)]
pub struct CreateSapAppealRequest {
    /// UUID of the workflow.forms row for SAP appeals at this institution.
    pub workflow_form_id:      Uuid,
    /// Narrative explanation of the appeal circumstances.
    pub appeal_reason:         String,
    /// Optional: UUID of a sis.student_documents supporting document.
    pub supporting_document_id: Option<Uuid>,
    /// Optional: term UUID for probationary reinstatement expiry.
    pub probation_expires_term_id: Option<Uuid>,
}

impl CreateSapAppealRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.appeal_reason.trim().is_empty() {
            errors.push("appeal_reason is required".into());
        }
        if self.appeal_reason.len() > 5000 {
            errors.push("appeal_reason must be 5000 characters or fewer".into());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// 201 response for a created SAP appeal.
#[derive(Debug, Serialize)]
pub struct CreateSapAppealResponse {
    pub appeal_id:                 Uuid,
    pub sap_evaluation_id:         Uuid,
    pub student_id:                Uuid,
    pub workflow_submission_id:    Uuid,
    pub supporting_document_id:    Option<Uuid>,
    pub appeal_status:             String,
    pub probation_expires_term_id: Option<Uuid>,
    pub created_at:                chrono::DateTime<chrono::Utc>,
}

// ── POST /finance/students/:student_id/va-certifications ─────────────────────

/// Submit a new VA enrollment certification for a term.
///
/// The certified_by_id is set from the authenticated user's JWT sub claim.
/// Unique constraint: (tenant_id, veteran_profile_id, term_id) blocks
/// duplicate non-amendment certifications.
#[derive(Debug, Deserialize)]
pub struct CreateVaCertificationRequest {
    pub term_id:                  Uuid,
    pub credits_certified:        f64,
    pub tuition_reported:         f64,
    pub fees_reported:            f64,
    pub certification_date:       NaiveDate,
    pub enrollment_intensity_va:  Option<String>,
    pub training_time_percentage: Option<f64>,
    pub ch33_housing_rate:        Option<f64>,
    pub ch33_book_stipend:        Option<f64>,
    pub program_of_study:         Option<String>,
}

/// Valid va_enrollment_intensity values.
pub const VALID_VA_INTENSITIES: &[&str] = &[
    "full_time",
    "three_quarter_time",
    "half_time",
    "less_than_half_time",
    "non_standard",
];

impl CreateVaCertificationRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.credits_certified <= 0.0 {
            errors.push("credits_certified must be greater than zero".into());
        }
        if self.tuition_reported < 0.0 {
            errors.push("tuition_reported must be >= 0".into());
        }
        if self.fees_reported < 0.0 {
            errors.push("fees_reported must be >= 0".into());
        }
        if let Some(ref intensity) = self.enrollment_intensity_va {
            if !VALID_VA_INTENSITIES.contains(&intensity.as_str()) {
                errors.push(format!(
                    "enrollment_intensity_va '{}' is invalid; must be one of: {}",
                    intensity,
                    VALID_VA_INTENSITIES.join(", ")
                ));
            }
        }
        if let Some(pct) = self.training_time_percentage {
            if !(0.0..=100.0).contains(&pct) {
                errors.push("training_time_percentage must be between 0 and 100".into());
            }
        }
        if let Some(rate) = self.ch33_housing_rate {
            if rate < 0.0 {
                errors.push("ch33_housing_rate must be >= 0".into());
            }
        }
        if let Some(stipend) = self.ch33_book_stipend {
            if stipend < 0.0 {
                errors.push("ch33_book_stipend must be >= 0".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// 201 response for a created VA certification.
#[derive(Debug, Serialize)]
pub struct VaCertificationResponse {
    pub certification_id:         Uuid,
    pub veteran_profile_id:       Uuid,
    pub student_id:               Uuid,
    pub term_id:                  Uuid,
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
    pub certified_by_id:          Option<Uuid>,
    pub updated_at:               chrono::DateTime<chrono::Utc>,
}

// ── PATCH /finance/students/:student_id/va-certifications/:id ────────────────

/// Amend an existing VA certification.
///
/// Sets is_amendment = true, links amends_certification_id, and stamps
/// amendment_date. The original certification is NEVER mutated — amendments
/// are new rows linked back. This matches VA-ONCE amendment workflow.
///
/// Any certification field can be corrected via amendment.
#[derive(Debug, Deserialize)]
pub struct AmendVaCertificationRequest {
    pub amendment_date:           NaiveDate,
    pub credits_certified:        Option<f64>,
    pub tuition_reported:         Option<f64>,
    pub fees_reported:            Option<f64>,
    pub enrollment_intensity_va:  Option<String>,
    pub training_time_percentage: Option<f64>,
    pub va_confirmation_number:   Option<String>,
    pub ch33_housing_rate:        Option<f64>,
    pub ch33_book_stipend:        Option<f64>,
    pub program_of_study:         Option<String>,
}

impl AmendVaCertificationRequest {
    pub fn has_changes(&self) -> bool {
        self.credits_certified.is_some()
            || self.tuition_reported.is_some()
            || self.fees_reported.is_some()
            || self.enrollment_intensity_va.is_some()
            || self.training_time_percentage.is_some()
            || self.va_confirmation_number.is_some()
            || self.ch33_housing_rate.is_some()
            || self.ch33_book_stipend.is_some()
            || self.program_of_study.is_some()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if !self.has_changes() {
            errors.push("request body must contain at least one field to amend".into());
        }
        if let Some(c) = self.credits_certified {
            if c <= 0.0 {
                errors.push("credits_certified must be greater than zero".into());
            }
        }
        if let Some(t) = self.tuition_reported {
            if t < 0.0 {
                errors.push("tuition_reported must be >= 0".into());
            }
        }
        if let Some(f) = self.fees_reported {
            if f < 0.0 {
                errors.push("fees_reported must be >= 0".into());
            }
        }
        if let Some(ref intensity) = self.enrollment_intensity_va {
            if !VALID_VA_INTENSITIES.contains(&intensity.as_str()) {
                errors.push(format!(
                    "enrollment_intensity_va '{}' is invalid",
                    intensity
                ));
            }
        }
        if let Some(pct) = self.training_time_percentage {
            if !(0.0..=100.0).contains(&pct) {
                errors.push("training_time_percentage must be between 0 and 100".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
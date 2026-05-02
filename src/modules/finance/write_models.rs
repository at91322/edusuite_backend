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
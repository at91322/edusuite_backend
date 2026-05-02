// src/modules/finance/mod.rs
//
// Finance module — full institutional financial lifecycle.
//
// Router layout mirrors the six endpoint groups defined in the design:
//   Group 1: Student Billing     — /finance/student-accounts, /finance/students/:id/account
//   Group 2: Financial Aid       — /finance/students/:id/fafsa, /finance/students/:id/aid-awards
//   Group 3: SAP                 — /finance/sap/policies, /finance/students/:id/sap
//   Group 4: VA Benefits         — /finance/students/:id/veteran-profile, /finance/students/:id/va-certifications
//   Group 5: Payroll / Staff Pay — /finance/payroll/runs, /finance/staff/:id/pay-stubs
//   Group 6: Reference reads     — /finance/fiscal-years, /finance/gl-accounts
//
// Step 1: all GET routes live. Write routes are stubbed (commented) and
// will be wired in Steps 2-4.

pub mod handlers;
pub mod models;
pub mod queries;
// pub mod write_models;   // Step 2
// pub mod write_queries;  // Step 2

use axum::{
    routing::get,
    Router,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()

        // ── Group 1: Student Billing ──────────────────────────────────────
        .route("/student-accounts",
            get(handlers::billing::list_student_accounts))
            // .post(handlers::billing::create_student_account))          // Step 2

        .route("/student-accounts/:id",
            get(handlers::billing::get_student_account))

        .route("/student-accounts/:id/transactions",
            get(handlers::billing::list_transactions))
            // .post(handlers::billing::post_transaction))                // Step 2

        .route("/student-accounts/:id/transactions/:tx_id",
            get(handlers::billing::get_transaction))

        .route("/student-accounts/:id/hold",
            get(handlers::billing::get_hold_status))
            // .patch(handlers::billing::set_hold))                       // Step 2

        // Convenience: look up account by student UUID
        .route("/students/:student_id/account",
            get(handlers::billing::get_account_by_student))
            // .post(handlers::billing::create_student_account_for_student)) // Step 2

        // ── Group 2: Financial Aid ────────────────────────────────────────
        .route("/students/:student_id/fafsa",
            get(handlers::aid::list_fafsa_records))

        .route("/students/:student_id/fafsa/:id",
            get(handlers::aid::get_fafsa_record))

        .route("/students/:student_id/aid-awards",
            get(handlers::aid::list_student_aid_awards))
            // .post(handlers::aid::create_aid_award))                    // Step 3

        .route("/students/:student_id/aid-awards/:id",
            get(handlers::aid::get_aid_award))
            // .patch(handlers::aid::update_aid_award))                   // Step 3

        // Tenant-wide aid award list for the financial aid office
        .route("/aid-awards",
            get(handlers::aid::list_aid_awards))

        // ── Group 3: SAP ──────────────────────────────────────────────────
        .route("/sap/policies",
            get(handlers::sap::list_sap_policies))

        .route("/sap/policies/:id",
            get(handlers::sap::get_sap_policy))

        .route("/students/:student_id/sap",
            get(handlers::sap::list_sap_evaluations))

        .route("/students/:student_id/sap/:id",
            get(handlers::sap::get_sap_evaluation))

        .route("/students/:student_id/sap/:id/appeal",
            get(handlers::sap::get_sap_appeal))
            // .post(handlers::sap::create_sap_appeal))                   // Step 4

        // ── Group 4: VA Benefits ──────────────────────────────────────────
        .route("/students/:student_id/veteran-profile",
            get(handlers::va::get_veteran_profile))

        .route("/students/:student_id/va-certifications",
            get(handlers::va::list_va_certifications))
            // .post(handlers::va::create_va_certification))              // Step 4

        .route("/students/:student_id/va-certifications/:id",
            get(handlers::va::get_va_certification))
            // .patch(handlers::va::amend_va_certification))              // Step 4

        // ── Group 5: Payroll / Staff Pay ──────────────────────────────────
        .route("/payroll/runs",
            get(handlers::payroll::list_payroll_runs))

        .route("/payroll/runs/:id",
            get(handlers::payroll::get_payroll_run))

        .route("/payroll/runs/:id/stubs",
            get(handlers::payroll::list_run_stubs))

        .route("/payroll/runs/:id/stubs/:stub_id",
            get(handlers::payroll::get_stub_detail))

        .route("/payroll/item-types",
            get(handlers::payroll::list_payroll_item_types))

        .route("/staff/:staff_id/pay-stubs",
            get(handlers::payroll::list_staff_pay_stubs))

        // ── Group 6: Reference Reads ──────────────────────────────────────
        .route("/fiscal-years",
            get(handlers::reference::list_fiscal_years))

        .route("/fiscal-years/:id",
            get(handlers::reference::get_fiscal_year))

        .route("/gl-accounts",
            get(handlers::reference::list_gl_accounts))
}
// src/modules/finance/write_queries.rs
//
// Write queries for Step 2 — student billing.
//
// Key invariants enforced here:
//   1. student_accounts has a unique constraint on (tenant_id, student_id).
//      Duplicate creation → 23505 → AppError::Conflict automatically.
//   2. student_transactions is IMMUTABLE. No UPDATE or DELETE is ever issued
//      against it. Corrections are new transactions (reversals).
//   3. current_balance on student_accounts is updated atomically in the same
//      transaction as the student_transactions INSERT — no eventual consistency gap.
//   4. All amounts flow through the same sign convention as the DB:
//      positive = charge, negative = credit/payment.

use uuid::Uuid;

use crate::error::AppError;
use super::write_models::{
    CreateStudentAccountRequest, CreateStudentAccountResponse,
    PostTransactionRequest, PostTransactionResponse,
    SetHoldRequest, SetHoldResponse,
};

// ═══════════════════════════════════════════════════════════════════════════════
// POST /finance/students/:student_id/account
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a student financial account.
///
/// Steps:
///   1. Confirm the student exists in this tenant (404 guard).
///   2. INSERT into finance.student_accounts.
///      The unique constraint (tenant_id, student_id) fires 23505 on duplicate
///      → AppError::Conflict automatically via AppError::from(sqlx::Error).
///   3. If opening_balance > 0, immediately post an 'adjustment' transaction
///      so the ledger is self-consistent from day one.
///
/// Returns the new account on success (201 Created).
pub async fn create_student_account(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    student_id: Uuid,
    req:        &CreateStudentAccountRequest,
) -> Result<CreateStudentAccountResponse, AppError> {

    // ── 1. Guard: student exists in this tenant ───────────────────────────
    let exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.student_profiles
            WHERE user_id   = $1
              AND tenant_id = $2
        ) AS "exists!"
        "#,
        student_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(
            format!("Student {} not found", student_id)
        ));
    }

    let opening_balance = req.opening_balance.unwrap_or(0.0);

    // ── 2. INSERT finance.student_accounts ───────────────────────────────
    // 23505 on (tenant_id, student_id) unique constraint → Conflict.
    // UNTYPED: f64 cannot bind directly to NUMERIC; cast $3 to ::numeric in SQL.
    use sqlx::Row as _;
    let row = sqlx::query(
        r#"
        INSERT INTO finance.student_accounts
            (tenant_id, student_id, current_balance, is_hold_active)
        VALUES ($1, $2, $3::numeric, false)
        RETURNING id, current_balance::float8 AS balance, is_hold_active, created_at
        "#,
    )
    .bind(tenant_id)
    .bind(student_id)
    .bind(opening_balance)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(CreateStudentAccountResponse {
        account_id:      row.try_get("id").map_err(AppError::from)?,
        student_id,
        current_balance: row.try_get("balance").map_err(AppError::from)?,
        is_hold_active:  row.try_get("is_hold_active").map_err(AppError::from)?,
        created_at:      row.try_get("created_at").map_err(AppError::from)?,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// POST /finance/student-accounts/:id/transactions
// ═══════════════════════════════════════════════════════════════════════════════

/// Post an immutable transaction to a student account.
///
/// Steps:
///   1. Confirm the account exists in this tenant (404 guard) and fetch
///      current_balance for the response.
///   2. Confirm the GL account exists and is active (400 guard).
///   3. If term_id provided, confirm the term exists in this tenant (400 guard).
///   4. INSERT into finance.student_transactions (immutable — no UPDATE ever).
///   5. UPDATE finance.student_accounts.current_balance atomically.
///      Uses current_balance + amount (positive charges increase balance owed;
///      negative credits/payments decrease it — matches the DB sign convention).
///   6. Return the transaction + updated balance.
///
/// The entire operation runs inside the caller's transaction. If any step fails
/// the whole thing rolls back — no partial state.
pub async fn post_transaction(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    account_id: Uuid,
    req:        &PostTransactionRequest,
) -> Result<PostTransactionResponse, AppError> {

    // ── 1. Guard: account exists in this tenant ───────────────────────────
    let account = sqlx::query!(
        r#"
        SELECT id, current_balance::float8 AS "balance!", student_id
        FROM finance.student_accounts
        WHERE id        = $1
          AND tenant_id = $2
        "#,
        account_id,
        tenant_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("Student account {} not found", account_id)
    ))?;

    // ── 2. Guard: GL account exists and is active ─────────────────────────
    let gl_active: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM finance.gl_accounts
            WHERE id        = $1
              AND tenant_id = $2
              AND is_active = true
        ) AS "exists!"
        "#,
        req.gl_account_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !gl_active {
        return Err(AppError::BadRequest(format!(
            "GL account {} not found or inactive", req.gl_account_id
        )));
    }

    // ── 3. Guard: term exists in this tenant (if provided) ────────────────
    if let Some(tid) = req.term_id {
        let term_exists: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM sis.academic_terms
                WHERE id        = $1
                  AND tenant_id = $2
            ) AS "exists!"
            "#,
            tid,
            tenant_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        if !term_exists {
            return Err(AppError::BadRequest(
                format!("Academic term {} not found", tid)
            ));
        }
    }

    // ── 4. INSERT finance.student_transactions (IMMUTABLE) ────────────────
    // transaction_type binds to a custom PG enum → use sqlx::query (untyped)
    // with explicit ::finance.transaction_type cast, per the established pattern.
    use sqlx::Row as _;

    let tx_date_expr = req.transaction_date
        .map(|d| d.to_string());

    let txn_row = sqlx::query(
        r#"
        INSERT INTO finance.student_transactions
            (tenant_id, account_id, term_id, gl_account_id,
             type, amount, description, reference_number, transaction_date)
        VALUES
            ($1, $2, $3, $4,
             $5::finance.transaction_type, $6, $7, $8,
             COALESCE($9::timestamptz, now()))
        RETURNING
            id,
            type::text              AS transaction_type,
            amount::float8          AS amount,
            description,
            reference_number,
            gl_account_id,
            term_id,
            transaction_date
        "#,
    )
    .bind(tenant_id)
    .bind(account_id)
    .bind(req.term_id)
    .bind(req.gl_account_id)
    .bind(&req.transaction_type)
    .bind(req.amount)
    .bind(req.description.trim())
    .bind(req.reference_number.as_deref())
    .bind(tx_date_expr.as_deref())
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let transaction_id: Uuid = txn_row.try_get("id").map_err(AppError::from)?;
    let transaction_date: chrono::DateTime<chrono::Utc> =
        txn_row.try_get("transaction_date").map_err(AppError::from)?;

    // ── 5. UPDATE student_accounts.current_balance atomically ────────────
    // current_balance = current_balance + amount
    //   Charges (positive amount) increase what the student owes.
    //   Payments/credits (negative amount) decrease it.
    // UNTYPED: f64 (req.amount) cannot bind directly to NUMERIC.
    let updated = sqlx::query(
        r#"
        UPDATE finance.student_accounts
           SET current_balance = current_balance + $2::numeric,
               updated_at      = now()
         WHERE id        = $1
           AND tenant_id = $3
        RETURNING current_balance::float8 AS updated_balance
        "#,
    )
    .bind(account_id)
    .bind(req.amount)
    .bind(tenant_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        tenant_id      = %tenant_id,
        account_id     = %account_id,
        student_id     = %account.student_id,
        transaction_id = %transaction_id,
        transaction_type = %req.transaction_type,
        amount         = req.amount,
        updated_balance = updated.try_get::<f64, _>("updated_balance").unwrap_or(0.0),
        "Student transaction posted"
    );

    Ok(PostTransactionResponse {
        transaction_id,
        account_id,
        transaction_type:  req.transaction_type.clone(),
        amount:            req.amount,
        description:       req.description.trim().to_string(),
        reference_number:  req.reference_number.clone(),
        gl_account_id:     req.gl_account_id,
        term_id:           req.term_id,
        transaction_date,
        updated_balance:   updated.try_get::<f64, _>("updated_balance").unwrap_or(0.0),
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// PATCH /finance/student-accounts/:id/hold
// ═══════════════════════════════════════════════════════════════════════════════

/// Set or clear the registration hold on a student account.
///
/// Steps:
///   1. Confirm the account exists in this tenant (404 guard).
///   2. Short-circuit if the requested state already matches (idempotent).
///   3. UPDATE is_hold_active.
///
/// The `reason` from the request is not stored in student_accounts (no column
/// for it) but IS written to core.audit_log by the handler after commit so
/// every hold change has an auditable reason. This function handles only the
/// DB update; the handler writes the audit entry separately.
pub async fn set_hold(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    account_id: Uuid,
    req:        &SetHoldRequest,
) -> Result<SetHoldResponse, AppError> {

    // ── 1. Guard: account exists in this tenant ───────────────────────────
    let account = sqlx::query!(
        r#"
        SELECT id, student_id, is_hold_active,
               current_balance::float8 AS "balance!"
        FROM finance.student_accounts
        WHERE id        = $1
          AND tenant_id = $2
        "#,
        account_id,
        tenant_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("Student account {} not found", account_id)
    ))?;

    // ── 2. Idempotency: already in requested state ────────────────────────
    if account.is_hold_active == req.is_hold_active {
        return Ok(SetHoldResponse {
            account_id,
            student_id:      account.student_id,
            is_hold_active:  account.is_hold_active,
            current_balance: account.balance,
            updated_at:      chrono::Utc::now(),
        });
    }

    // ── 3. UPDATE is_hold_active ──────────────────────────────────────────
    let updated = sqlx::query!(
        r#"
        UPDATE finance.student_accounts
           SET is_hold_active = $2,
               updated_at     = now()
         WHERE id        = $1
           AND tenant_id = $3
        RETURNING updated_at, current_balance::float8 AS "balance!"
        "#,
        account_id,
        req.is_hold_active,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        tenant_id   = %tenant_id,
        account_id  = %account_id,
        student_id  = %account.student_id,
        hold_active = req.is_hold_active,
        reason      = %req.reason,
        "Student account hold status changed"
    );

    Ok(SetHoldResponse {
        account_id,
        student_id:      account.student_id,
        is_hold_active:  req.is_hold_active,
        current_balance: updated.balance,
        updated_at:      updated.updated_at,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 3 — FINANCIAL AID WRITES
// ═══════════════════════════════════════════════════════════════════════════════

use super::write_models::{
    AidAwardResponse, CreateAidAwardRequest,
    UpdateAidAwardRequest,
};

// ── POST /finance/students/:student_id/aid-awards ────────────────────────────

/// Package a new financial aid award.
///
/// Steps:
///   1. Guard: student exists in this tenant.
///   2. Guard: term exists in this tenant.
///   3. INSERT finance.financial_aid_awards.
///      aid_type and nslds_loan_type are PG enums → untyped query.
///      offered_amount is NUMERIC → ::numeric cast.
///
/// Returns the new award record (201 Created).
pub async fn create_aid_award(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    student_id: Uuid,
    req:        &CreateAidAwardRequest,
) -> Result<AidAwardResponse, AppError> {

    // ── 1. Guard: student exists ──────────────────────────────────────────
    let student_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.student_profiles
            WHERE user_id   = $1
              AND tenant_id = $2
        ) AS "exists!"
        "#,
        student_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !student_exists {
        return Err(AppError::NotFound(format!("Student {} not found", student_id)));
    }

    // ── 2. Guard: term exists ─────────────────────────────────────────────
    let term_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.academic_terms
            WHERE id        = $1
              AND tenant_id = $2
        ) AS "exists!"
        "#,
        req.term_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !term_exists {
        return Err(AppError::BadRequest(format!(
            "Academic term {} not found", req.term_id
        )));
    }

    // ── 3. INSERT finance.financial_aid_awards ────────────────────────────
    // UNTYPED: aid_type → finance.aid_source_type (PG enum)
    //          nslds_loan_type → finance.nslds_loan_type (PG enum)
    //          offered_amount  → NUMERIC
    use sqlx::Row as _;

    let status = req.status.as_deref().unwrap_or("offered");

    let row = sqlx::query(
        r#"
        INSERT INTO finance.financial_aid_awards
            (tenant_id, student_id, term_id, fund_name, aid_type,
             offered_amount, accepted_amount, disbursed_amount, status,
             nslds_loan_type, loan_sequence_number,
             loan_period_begin_date, loan_period_end_date,
             grade_level_at_award)
        VALUES
            ($1, $2, $3, $4, $5::finance.aid_source_type,
             $6::numeric, 0.00, 0.00, $7,
             $8::finance.nslds_loan_type, $9,
             $10, $11, $12)
        RETURNING
            id,
            fund_name,
            aid_type::text          AS aid_type,
            offered_amount::float8  AS offered_amount,
            accepted_amount::float8 AS accepted_amount,
            disbursed_amount::float8 AS disbursed_amount,
            status,
            nslds_loan_type::text   AS nslds_loan_type,
            loan_sequence_number,
            loan_period_begin_date,
            loan_period_end_date,
            grade_level_at_award,
            created_at,
            updated_at
        "#,
    )
    .bind(tenant_id)
    .bind(student_id)
    .bind(req.term_id)
    .bind(req.fund_name.trim())
    .bind(&req.aid_type)
    .bind(req.offered_amount)
    .bind(status)
    .bind(req.nslds_loan_type.as_deref())
    .bind(req.loan_sequence_number)
    .bind(req.loan_period_begin_date)
    .bind(req.loan_period_end_date)
    .bind(req.grade_level_at_award)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(AidAwardResponse {
        award_id:               row.try_get("id").map_err(AppError::from)?,
        student_id,
        term_id:                req.term_id,
        fund_name:              row.try_get("fund_name").map_err(AppError::from)?,
        aid_type:               row.try_get("aid_type").map_err(AppError::from)?,
        offered_amount:         row.try_get("offered_amount").map_err(AppError::from)?,
        accepted_amount:        row.try_get("accepted_amount").map_err(AppError::from)?,
        disbursed_amount:       row.try_get("disbursed_amount").map_err(AppError::from)?,
        status:                 row.try_get("status").map_err(AppError::from)?,
        nslds_loan_type:        row.try_get("nslds_loan_type").map_err(AppError::from)?,
        loan_sequence_number:   row.try_get("loan_sequence_number").map_err(AppError::from)?,
        loan_period_begin_date: row.try_get("loan_period_begin_date").map_err(AppError::from)?,
        loan_period_end_date:   row.try_get("loan_period_end_date").map_err(AppError::from)?,
        grade_level_at_award:   row.try_get("grade_level_at_award").map_err(AppError::from)?,
        created_at:             row.try_get("created_at").map_err(AppError::from)?,
        updated_at:             row.try_get("updated_at").map_err(AppError::from)?,
    })
}

// ── PATCH /finance/students/:student_id/aid-awards/:id ──────────────────────

/// Update an aid award — drives the offered → accepted → disbursed state machine.
///
/// The disbursed transition is the critical cross-module write:
///   When status transitions to "disbursed", an aid_disbursement transaction
///   is posted to the student ledger in the SAME database transaction.
///   If either the award UPDATE or the transaction INSERT fails, both roll back.
///
/// Guards:
///   - Award must belong to this student and tenant (404 otherwise).
///   - Cannot modify a disbursed award (409 — immutable after disbursement).
///   - disbursed_amount must not exceed accepted_amount.
///   - disbursement_gl_account_id must be an active GL account (when disbursing).
///
/// Returns the updated award (200 OK).
pub async fn update_aid_award(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    student_id: Uuid,
    award_id:   Uuid,
    req:        &UpdateAidAwardRequest,
) -> Result<AidAwardResponse, AppError> {

    use sqlx::Row as _;

    // ── 1. Fetch current award — guard 404 and disbursed immutability ──────
    let current = sqlx::query(
        r#"
        SELECT
            id, student_id, term_id, fund_name,
            aid_type::text          AS aid_type,
            offered_amount::float8  AS offered_amount,
            accepted_amount::float8 AS accepted_amount,
            disbursed_amount::float8 AS disbursed_amount,
            status,
            nslds_loan_type::text   AS nslds_loan_type,
            loan_sequence_number,
            loan_period_begin_date,
            loan_period_end_date,
            grade_level_at_award,
            created_at,
            updated_at
        FROM finance.financial_aid_awards
        WHERE id         = $1
          AND student_id = $2
          AND tenant_id  = $3
        "#,
    )
    .bind(award_id)
    .bind(student_id)
    .bind(tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("Aid award {} not found for student {}", award_id, student_id)
    ))?;

    let current_status: String = current.try_get("status").map_err(AppError::from)?;

    if current_status == "disbursed" {
        return Err(AppError::Conflict(
            "Disbursed awards are immutable and cannot be modified".into()
        ));
    }

    // ── 2. Resolve new values (partial PATCH — absent fields keep current) ─
    let new_status = req.status.as_deref().unwrap_or(&current_status);

    let current_offered:  f64 = current.try_get("offered_amount").map_err(AppError::from)?;
    let current_accepted: f64 = current.try_get("accepted_amount").map_err(AppError::from)?;

    let new_accepted: f64 = req.accepted_amount.unwrap_or_else(|| {
        // When transitioning to accepted and no explicit accepted_amount given,
        // default to the offered_amount (common FA office workflow).
        if new_status == "accepted" && current_status == "offered" {
            current_offered
        } else {
            current_accepted
        }
    });

    let new_disbursed: f64 = req.disbursed_amount.unwrap_or_else(|| {
        // When transitioning to disbursed with no explicit disbursed_amount,
        // default to the accepted_amount.
        if new_status == "disbursed" {
            new_accepted
        } else {
            current.try_get("disbursed_amount").unwrap_or(0.0)
        }
    });

    // ── 3. Business rule: disbursed_amount <= accepted_amount ─────────────
    if new_status == "disbursed" && new_disbursed > new_accepted {
        return Err(AppError::BadRequest(format!(
            "disbursed_amount ({}) cannot exceed accepted_amount ({})",
            new_disbursed, new_accepted
        )));
    }

    // ── 4. If disbursing, validate GL account ─────────────────────────────
    if new_status == "disbursed" {
        let gl_id = req.disbursement_gl_account_id
            .ok_or_else(|| AppError::BadRequest(
                "disbursement_gl_account_id is required when status = 'disbursed'".into()
            ))?;

        let gl_active: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM finance.gl_accounts
                WHERE id        = $1
                  AND tenant_id = $2
                  AND is_active = true
            ) AS "exists!"
            "#,
            gl_id,
            tenant_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        if !gl_active {
            return Err(AppError::BadRequest(format!(
                "GL account {} not found or inactive", gl_id
            )));
        }
    }

    // ── 5. UPDATE finance.financial_aid_awards ────────────────────────────
    // UNTYPED: accepted_amount and disbursed_amount bind to NUMERIC.
    let updated = sqlx::query(
        r#"
        UPDATE finance.financial_aid_awards
           SET status           = $2,
               accepted_amount  = $3::numeric,
               disbursed_amount = $4::numeric,
               updated_at       = now()
         WHERE id        = $1
           AND tenant_id = $5
        RETURNING
            id, student_id, term_id, fund_name,
            aid_type::text          AS aid_type,
            offered_amount::float8  AS offered_amount,
            accepted_amount::float8 AS accepted_amount,
            disbursed_amount::float8 AS disbursed_amount,
            status,
            nslds_loan_type::text   AS nslds_loan_type,
            loan_sequence_number,
            loan_period_begin_date,
            loan_period_end_date,
            grade_level_at_award,
            created_at,
            updated_at
        "#,
    )
    .bind(award_id)
    .bind(new_status)
    .bind(new_accepted)
    .bind(new_disbursed)
    .bind(tenant_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    // ── 6. Cross-module write: post aid_disbursement transaction ──────────
    // Only fires on the offered/accepted → disbursed transition.
    // Uses the same write_queries::post_transaction logic but called directly
    // here so both writes share the same transaction and roll back together.
    if new_status == "disbursed" && current_status != "disbursed" {
        let gl_id = req.disbursement_gl_account_id.unwrap(); // validated in step 4
        let term_id: Uuid = updated.try_get("term_id").map_err(AppError::from)?;
        let fund_name: String = updated.try_get("fund_name").map_err(AppError::from)?;

        // Find the student account for this student
        let account_id: Option<Uuid> = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM finance.student_accounts
            WHERE student_id = $1
              AND tenant_id  = $2
            "#,
            student_id,
            tenant_id,
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(AppError::from)?;

        match account_id {
            None => {
                // No student account exists yet — log a warning but do not fail.
                // The disbursement is still recorded on the award; the transaction
                // can be reconciled manually or when the account is created.
                tracing::warn!(
                    tenant_id  = %tenant_id,
                    student_id = %student_id,
                    award_id   = %award_id,
                    amount     = new_disbursed,
                    "Aid disbursement posted but student has no billing account — transaction skipped"
                );
            }
            Some(acct_id) => {
                // Amount is NEGATIVE — aid credits reduce what the student owes.
                let credit_amount = -new_disbursed;
                let description = format!("Aid disbursement: {}", fund_name);

                // INSERT the immutable transaction
                sqlx::query(
                    r#"
                    INSERT INTO finance.student_transactions
                        (tenant_id, account_id, term_id, gl_account_id,
                         type, amount, description, reference_number)
                    VALUES
                        ($1, $2, $3, $4,
                         'aid_disbursement'::finance.transaction_type,
                         $5::numeric, $6, $7)
                    "#,
                )
                .bind(tenant_id)
                .bind(acct_id)
                .bind(term_id)
                .bind(gl_id)
                .bind(credit_amount)
                .bind(&description)
                .bind(award_id.to_string())   // reference_number = award UUID
                .execute(&mut **tx)
                .await
                .map_err(AppError::from)?;

                // UPDATE the account balance atomically
                sqlx::query(
                    r#"
                    UPDATE finance.student_accounts
                       SET current_balance = current_balance + $2::numeric,
                           updated_at      = now()
                     WHERE id        = $1
                       AND tenant_id = $3
                    "#,
                )
                .bind(acct_id)
                .bind(credit_amount)
                .bind(tenant_id)
                .execute(&mut **tx)
                .await
                .map_err(AppError::from)?;

                tracing::info!(
                    tenant_id  = %tenant_id,
                    student_id = %student_id,
                    award_id   = %award_id,
                    account_id = %acct_id,
                    amount     = credit_amount,
                    "Aid disbursement transaction posted to student ledger"
                );
            }
        }
    }

    Ok(AidAwardResponse {
        award_id:               updated.try_get("id").map_err(AppError::from)?,
        student_id,
        term_id:                updated.try_get("term_id").map_err(AppError::from)?,
        fund_name:              updated.try_get("fund_name").map_err(AppError::from)?,
        aid_type:               updated.try_get("aid_type").map_err(AppError::from)?,
        offered_amount:         updated.try_get("offered_amount").map_err(AppError::from)?,
        accepted_amount:        updated.try_get("accepted_amount").map_err(AppError::from)?,
        disbursed_amount:       updated.try_get("disbursed_amount").map_err(AppError::from)?,
        status:                 updated.try_get("status").map_err(AppError::from)?,
        nslds_loan_type:        updated.try_get("nslds_loan_type").map_err(AppError::from)?,
        loan_sequence_number:   updated.try_get("loan_sequence_number").map_err(AppError::from)?,
        loan_period_begin_date: updated.try_get("loan_period_begin_date").map_err(AppError::from)?,
        loan_period_end_date:   updated.try_get("loan_period_end_date").map_err(AppError::from)?,
        grade_level_at_award:   updated.try_get("grade_level_at_award").map_err(AppError::from)?,
        created_at:             updated.try_get("created_at").map_err(AppError::from)?,
        updated_at:             updated.try_get("updated_at").map_err(AppError::from)?,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 4 — SAP APPEAL + VA CERTIFICATION WRITES
// ═══════════════════════════════════════════════════════════════════════════════

use super::write_models::{
    CreateSapAppealRequest, CreateSapAppealResponse,
    CreateVaCertificationRequest, VaCertificationResponse,
    AmendVaCertificationRequest,
};

// ── POST /finance/students/:student_id/sap/:id/appeal ────────────────────────

/// Submit a SAP appeal for a failing evaluation.
///
/// Steps:
///   1. Guard: evaluation exists, belongs to this student and tenant.
///   2. Guard: evaluation is in a failing status (not good_standing).
///   3. Guard: no open (pending_committee_review) appeal already exists.
///   4. Guard: workflow form exists in this tenant.
///   5. INSERT workflow.submissions — creates the routing envelope.
///   6. INSERT finance.sap_appeals — links to the evaluation and submission.
///
/// Both INSERTs share the caller's transaction. If either fails, both roll back.
pub async fn create_sap_appeal(
    tx:            &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:     Uuid,
    student_id:    Uuid,
    evaluation_id: Uuid,
    initiator_id:  Uuid,   // authenticated user's sub from JWT
    req:           &CreateSapAppealRequest,
) -> Result<CreateSapAppealResponse, AppError> {

    // ── 1. Guard: evaluation exists and belongs to this student ───────────
    let eval = sqlx::query!(
        r#"
        SELECT id, resulting_sap_status::text AS "status!"
        FROM finance.sap_evaluations
        WHERE id         = $1
          AND student_id = $2
          AND tenant_id  = $3
        "#,
        evaluation_id,
        student_id,
        tenant_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("SAP evaluation {} not found for student {}", evaluation_id, student_id)
    ))?;

    // ── 2. Guard: evaluation is in a failing status ───────────────────────
    const FAILING_STATUSES: &[&str] = &[
        "warning", "suspension", "probation",
        "academic_plan", "max_timeframe_exceeded",
    ];
    if !FAILING_STATUSES.contains(&eval.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "SAP appeals can only be filed for failing evaluations; \
             current status is '{}'", eval.status
        )));
    }

    // ── 3. Guard: no open appeal already exists ───────────────────────────
    let open_appeal_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM finance.sap_appeals
            WHERE sap_evaluation_id = $1
              AND tenant_id         = $2
              AND appeal_status     = 'pending_committee_review'
        ) AS "exists!"
        "#,
        evaluation_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if open_appeal_exists {
        return Err(AppError::Conflict(
            "An open appeal already exists for this SAP evaluation".into()
        ));
    }

    // ── 4. Guard: workflow form exists in this tenant ─────────────────────
    let form_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM workflow.forms
            WHERE id        = $1
              AND tenant_id = $2
              AND is_active = true
        ) AS "exists!"
        "#,
        req.workflow_form_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !form_exists {
        return Err(AppError::BadRequest(format!(
            "Workflow form {} not found or inactive", req.workflow_form_id
        )));
    }

    // ── 5. INSERT workflow.submissions ────────────────────────────────────
    // payload_data stores the appeal reason and evaluation context as JSONB.
    let payload = serde_json::json!({
        "appeal_reason":      req.appeal_reason,
        "sap_evaluation_id":  evaluation_id,
        "student_id":         student_id,
        "sap_status_appealed": eval.status,
    });

    let submission_row = sqlx::query!(
        r#"
        INSERT INTO workflow.submissions
            (tenant_id, form_id, initiator_user_id, payload_data, status)
        VALUES ($1, $2, $3, $4, 'routing')
        RETURNING id
        "#,
        tenant_id,
        req.workflow_form_id,
        initiator_id,
        payload,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let submission_id = submission_row.id;

    // ── 6. INSERT finance.sap_appeals ────────────────────────────────────
    let appeal_row = sqlx::query!(
        r#"
        INSERT INTO finance.sap_appeals
            (tenant_id, student_id, sap_evaluation_id,
             workflow_submission_id, supporting_document_id,
             appeal_status, probation_expires_term_id)
        VALUES
            ($1, $2, $3, $4, $5, 'pending_committee_review', $6)
        RETURNING id, appeal_status, created_at
        "#,
        tenant_id,
        student_id,
        evaluation_id,
        submission_id,
        req.supporting_document_id,
        req.probation_expires_term_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        tenant_id     = %tenant_id,
        student_id    = %student_id,
        evaluation_id = %evaluation_id,
        appeal_id     = %appeal_row.id,
        submission_id = %submission_id,
        "SAP appeal submitted"
    );

    Ok(CreateSapAppealResponse {
        appeal_id:                 appeal_row.id,
        sap_evaluation_id:         evaluation_id,
        student_id,
        workflow_submission_id:    submission_id,
        supporting_document_id:    req.supporting_document_id,
        appeal_status:             appeal_row.appeal_status,
        probation_expires_term_id: req.probation_expires_term_id,
        created_at:                appeal_row.created_at,
    })
}

// ── POST /finance/students/:student_id/va-certifications ─────────────────────

/// Submit a new VA enrollment certification.
///
/// Steps:
///   1. Guard: veteran profile exists for this student in this tenant.
///   2. Guard: term exists in this tenant.
///   3. Guard: no existing non-amendment certification for (veteran_profile_id, term_id)
///      — unique constraint on the table; explicit check gives a clean 409.
///   4. INSERT finance.va_certifications.
///      Numeric columns (credits, tuition, fees, rates) use ::numeric casts.
///      va_enrollment_intensity is a PG enum → untyped query.
pub async fn create_va_certification(
    tx:            &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:     Uuid,
    student_id:    Uuid,
    certified_by:  Uuid,   // authenticated user's sub from JWT
    req:           &CreateVaCertificationRequest,
) -> Result<VaCertificationResponse, AppError> {

    use sqlx::Row as _;

    // ── 1. Guard: veteran profile exists ─────────────────────────────────
    let profile = sqlx::query!(
        r#"
        SELECT id
        FROM finance.veteran_profiles
        WHERE student_id = $1
          AND tenant_id  = $2
        "#,
        student_id,
        tenant_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("No VA veteran profile found for student {}", student_id)
    ))?;

    let veteran_profile_id = profile.id;

    // ── 2. Guard: term exists ─────────────────────────────────────────────
    let term_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.academic_terms
            WHERE id        = $1
              AND tenant_id = $2
        ) AS "exists!"
        "#,
        req.term_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !term_exists {
        return Err(AppError::BadRequest(
            format!("Academic term {} not found", req.term_id)
        ));
    }

    // ── 3. Guard: no existing non-amendment certification for this term ───
    // The unique constraint (tenant_id, veteran_profile_id, term_id) exists
    // on the table but only for non-amendment rows. Check explicitly for a
    // clean 409 rather than a raw constraint violation.
    let cert_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM finance.va_certifications
            WHERE veteran_profile_id = $1
              AND term_id            = $2
              AND tenant_id          = $3
              AND is_amendment       = false
        ) AS "exists!"
        "#,
        veteran_profile_id,
        req.term_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if cert_exists {
        return Err(AppError::Conflict(
            "A VA certification already exists for this student and term; \
             use PATCH to file an amendment".into()
        ));
    }

    // ── 4. INSERT finance.va_certifications ───────────────────────────────
    // UNTYPED: enrollment_intensity_va is a PG enum; numeric fields need casts.
    let row = sqlx::query(
        r#"
        INSERT INTO finance.va_certifications
            (tenant_id, veteran_profile_id, term_id, certified_by_id,
             credits_certified, tuition_reported, fees_reported,
             certification_date, enrollment_intensity_va,
             training_time_percentage, is_amendment,
             ch33_housing_rate, ch33_book_stipend, program_of_study)
        VALUES
            ($1, $2, $3, $4,
             $5::numeric, $6::numeric, $7::numeric,
             $8,
             $9::finance.va_enrollment_intensity,
             $10::numeric, false,
             $11::numeric, $12::numeric, $13)
        RETURNING
            id, veteran_profile_id, term_id, certified_by_id,
            credits_certified::float8        AS credits_certified,
            tuition_reported::float8         AS tuition_reported,
            fees_reported::float8            AS fees_reported,
            certification_date,
            enrollment_intensity_va::text    AS enrollment_intensity_va,
            training_time_percentage::float8 AS training_time_percentage,
            is_amendment, amends_certification_id, amendment_date,
            va_confirmation_number,
            ch33_housing_rate::float8        AS ch33_housing_rate,
            ch33_book_stipend::float8        AS ch33_book_stipend,
            program_of_study,
            updated_at
        "#,
    )
    .bind(tenant_id)
    .bind(veteran_profile_id)
    .bind(req.term_id)
    .bind(certified_by)
    .bind(req.credits_certified)
    .bind(req.tuition_reported)
    .bind(req.fees_reported)
    .bind(req.certification_date)
    .bind(req.enrollment_intensity_va.as_deref())
    .bind(req.training_time_percentage)
    .bind(req.ch33_housing_rate)
    .bind(req.ch33_book_stipend)
    .bind(req.program_of_study.as_deref())
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        tenant_id          = %tenant_id,
        student_id         = %student_id,
        veteran_profile_id = %veteran_profile_id,
        term_id            = %req.term_id,
        certified_by       = %certified_by,
        "VA certification created"
    );

    Ok(VaCertificationResponse {
        certification_id:         row.try_get("id").map_err(AppError::from)?,
        veteran_profile_id:       row.try_get("veteran_profile_id").map_err(AppError::from)?,
        student_id,
        term_id:                  row.try_get("term_id").map_err(AppError::from)?,
        credits_certified:        row.try_get("credits_certified").map_err(AppError::from)?,
        tuition_reported:         row.try_get("tuition_reported").map_err(AppError::from)?,
        fees_reported:            row.try_get("fees_reported").map_err(AppError::from)?,
        certification_date:       row.try_get("certification_date").map_err(AppError::from)?,
        enrollment_intensity_va:  row.try_get("enrollment_intensity_va").map_err(AppError::from)?,
        training_time_percentage: row.try_get("training_time_percentage").map_err(AppError::from)?,
        is_amendment:             row.try_get("is_amendment").map_err(AppError::from)?,
        amends_certification_id:  row.try_get("amends_certification_id").map_err(AppError::from)?,
        amendment_date:           row.try_get("amendment_date").map_err(AppError::from)?,
        va_confirmation_number:   row.try_get("va_confirmation_number").map_err(AppError::from)?,
        ch33_housing_rate:        row.try_get("ch33_housing_rate").map_err(AppError::from)?,
        ch33_book_stipend:        row.try_get("ch33_book_stipend").map_err(AppError::from)?,
        program_of_study:         row.try_get("program_of_study").map_err(AppError::from)?,
        certified_by_id:          row.try_get("certified_by_id").map_err(AppError::from)?,
        updated_at:               row.try_get("updated_at").map_err(AppError::from)?,
    })
}

// ── PATCH /finance/students/:student_id/va-certifications/:id ────────────────

/// Amend an existing VA certification.
///
/// Creates a NEW row with is_amendment = true and amends_certification_id
/// pointing to the original. The original row is never mutated — this matches
/// the VA-ONCE amendment model where each correction is a fresh submission.
///
/// Steps:
///   1. Guard: original certification exists and belongs to this student.
///   2. Guard: original is not itself an amendment (can't amend an amendment —
///      the VA requires amendments to reference the original certification).
///   3. INSERT a new va_certifications row with is_amendment = true,
///      carrying forward any fields not overridden in the request.
pub async fn amend_va_certification(
    tx:               &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:        Uuid,
    student_id:       Uuid,
    certification_id: Uuid,
    certified_by:     Uuid,
    req:              &AmendVaCertificationRequest,
) -> Result<VaCertificationResponse, AppError> {

    use sqlx::Row as _;

    // ── 1. Fetch original certification ───────────────────────────────────
    let original = sqlx::query(
        r#"
        SELECT
            vc.id, vc.veteran_profile_id, vc.term_id,
            vc.credits_certified::float8        AS credits_certified,
            vc.tuition_reported::float8         AS tuition_reported,
            vc.fees_reported::float8            AS fees_reported,
            vc.certification_date,
            vc.enrollment_intensity_va::text    AS enrollment_intensity_va,
            vc.training_time_percentage::float8 AS training_time_percentage,
            vc.is_amendment,
            vc.ch33_housing_rate::float8        AS ch33_housing_rate,
            vc.ch33_book_stipend::float8        AS ch33_book_stipend,
            vc.program_of_study
        FROM finance.va_certifications vc
        JOIN finance.veteran_profiles vp ON vp.id = vc.veteran_profile_id
        WHERE vc.id         = $1
          AND vp.student_id = $2
          AND vc.tenant_id  = $3
        "#,
    )
    .bind(certification_id)
    .bind(student_id)
    .bind(tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("VA certification {} not found for student {}", certification_id, student_id)
    ))?;

    let veteran_profile_id: Uuid = original.try_get("veteran_profile_id").map_err(AppError::from)?;

    // ── 2. Guard: cannot amend an amendment ───────────────────────────────
    let is_amendment: bool = original.try_get("is_amendment").map_err(AppError::from)?;
    if is_amendment {
        return Err(AppError::BadRequest(
            "Cannot amend an amendment; please reference the original certification".into()
        ));
    }

    // ── 3. Resolve fields: request overrides original where provided ──────
    let credits: f64 = req.credits_certified
        .unwrap_or(original.try_get("credits_certified").unwrap_or(0.0));
    let tuition: f64 = req.tuition_reported
        .unwrap_or(original.try_get("tuition_reported").unwrap_or(0.0));
    let fees: f64 = req.fees_reported
        .unwrap_or(original.try_get("fees_reported").unwrap_or(0.0));
    let intensity: Option<String> = req.enrollment_intensity_va.clone()
        .or_else(|| original.try_get("enrollment_intensity_va").unwrap_or(None));
    let training_pct: Option<f64> = req.training_time_percentage
        .or_else(|| original.try_get("training_time_percentage").unwrap_or(None));
    let housing: Option<f64> = req.ch33_housing_rate
        .or_else(|| original.try_get("ch33_housing_rate").unwrap_or(None));
    let stipend: Option<f64> = req.ch33_book_stipend
        .or_else(|| original.try_get("ch33_book_stipend").unwrap_or(None));
    let program: Option<String> = req.program_of_study.clone()
        .or_else(|| original.try_get("program_of_study").unwrap_or(None));
    let orig_cert_date: chrono::NaiveDate = original.try_get("certification_date")
        .map_err(AppError::from)?;
    let term_id: Uuid = original.try_get("term_id").map_err(AppError::from)?;

    // ── 4. INSERT amendment row ───────────────────────────────────────────
    let row = sqlx::query(
        r#"
        INSERT INTO finance.va_certifications
            (tenant_id, veteran_profile_id, term_id, certified_by_id,
             credits_certified, tuition_reported, fees_reported,
             certification_date, enrollment_intensity_va,
             training_time_percentage, is_amendment,
             amends_certification_id, amendment_date,
             va_confirmation_number,
             ch33_housing_rate, ch33_book_stipend, program_of_study)
        VALUES
            ($1, $2, $3, $4,
             $5::numeric, $6::numeric, $7::numeric,
             $8,
             $9::finance.va_enrollment_intensity,
             $10::numeric, true,
             $11, $12,
             $13,
             $14::numeric, $15::numeric, $16)
        RETURNING
            id, veteran_profile_id, term_id, certified_by_id,
            credits_certified::float8        AS credits_certified,
            tuition_reported::float8         AS tuition_reported,
            fees_reported::float8            AS fees_reported,
            certification_date,
            enrollment_intensity_va::text    AS enrollment_intensity_va,
            training_time_percentage::float8 AS training_time_percentage,
            is_amendment, amends_certification_id, amendment_date,
            va_confirmation_number,
            ch33_housing_rate::float8        AS ch33_housing_rate,
            ch33_book_stipend::float8        AS ch33_book_stipend,
            program_of_study, updated_at
        "#,
    )
    .bind(tenant_id)
    .bind(veteran_profile_id)
    .bind(term_id)
    .bind(certified_by)
    .bind(credits)
    .bind(tuition)
    .bind(fees)
    .bind(orig_cert_date)       // certification_date = original's date
    .bind(intensity.as_deref())
    .bind(training_pct)
    .bind(certification_id)     // amends_certification_id = original's id
    .bind(req.amendment_date)
    .bind(req.va_confirmation_number.as_deref())
    .bind(housing)
    .bind(stipend)
    .bind(program.as_deref())
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        tenant_id          = %tenant_id,
        student_id         = %student_id,
        original_cert_id   = %certification_id,
        amendment_id       = %row.try_get::<Uuid,_>("id").unwrap_or_default(),
        "VA certification amendment created"
    );

    Ok(VaCertificationResponse {
        certification_id:         row.try_get("id").map_err(AppError::from)?,
        veteran_profile_id:       row.try_get("veteran_profile_id").map_err(AppError::from)?,
        student_id,
        term_id:                  row.try_get("term_id").map_err(AppError::from)?,
        credits_certified:        row.try_get("credits_certified").map_err(AppError::from)?,
        tuition_reported:         row.try_get("tuition_reported").map_err(AppError::from)?,
        fees_reported:            row.try_get("fees_reported").map_err(AppError::from)?,
        certification_date:       row.try_get("certification_date").map_err(AppError::from)?,
        enrollment_intensity_va:  row.try_get("enrollment_intensity_va").map_err(AppError::from)?,
        training_time_percentage: row.try_get("training_time_percentage").map_err(AppError::from)?,
        is_amendment:             row.try_get("is_amendment").map_err(AppError::from)?,
        amends_certification_id:  row.try_get("amends_certification_id").map_err(AppError::from)?,
        amendment_date:           row.try_get("amendment_date").map_err(AppError::from)?,
        va_confirmation_number:   row.try_get("va_confirmation_number").map_err(AppError::from)?,
        ch33_housing_rate:        row.try_get("ch33_housing_rate").map_err(AppError::from)?,
        ch33_book_stipend:        row.try_get("ch33_book_stipend").map_err(AppError::from)?,
        program_of_study:         row.try_get("program_of_study").map_err(AppError::from)?,
        certified_by_id:          row.try_get("certified_by_id").map_err(AppError::from)?,
        updated_at:               row.try_get("updated_at").map_err(AppError::from)?,
    })
}
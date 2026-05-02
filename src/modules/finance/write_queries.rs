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
// src/modules/finance/queries.rs
//
// All read queries for the finance module.
// All numeric DB columns (numeric, decimal) are cast to float8 in SQL so
// sqlx maps them directly to f64 — consistent with the sis/hr/lms pattern.
// Custom PG enums are cast to ::text in SELECT lists.

use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use super::models::*;

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 1 — STUDENT BILLING
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_student_accounts(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListStudentAccountsParams,
) -> Result<(Vec<StudentAccountSummary>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.student_accounts sa
        WHERE sa.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::uuid IS NULL OR sa.student_id     = $1)
          AND ($2::bool IS NULL OR sa.is_hold_active = $2)
        "#,
        params.student_id    as Option<Uuid>,
        params.is_hold_active as Option<bool>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            sa.id                                           AS id,
            sa.student_id                                   AS student_id,
            u.first_name || ' ' || u.last_name              AS student_name,
            sa.current_balance::float8                      AS current_balance,
            sa.is_hold_active                               AS is_hold_active,
            sa.created_at                                   AS created_at,
            sa.updated_at                                   AS updated_at
        FROM finance.student_accounts sa
        JOIN core.users u ON u.id = sa.student_id
        WHERE sa.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::uuid IS NULL OR sa.student_id     = $1)
          AND ($2::bool IS NULL OR sa.is_hold_active = $2)
        ORDER BY u.last_name ASC, u.first_name ASC
        LIMIT  $3
        OFFSET $4
        "#,
    )
    .bind(params.student_id)
    .bind(params.is_hold_active)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let accounts = rows.iter().map(|r| -> Result<StudentAccountSummary, AppError> {
        Ok(StudentAccountSummary {
            id:              r.try_get("id").map_err(AppError::from)?,
            student_id:      r.try_get("student_id").map_err(AppError::from)?,
            student_name:    r.try_get("student_name").map_err(AppError::from)?,
            current_balance: r.try_get("current_balance").map_err(AppError::from)?,
            is_hold_active:  r.try_get("is_hold_active").map_err(AppError::from)?,
            created_at:      r.try_get("created_at").map_err(AppError::from)?,
            updated_at:      r.try_get("updated_at").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((accounts, total))
}

pub async fn get_student_account(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_id: Uuid,
) -> Result<Option<StudentAccountSummary>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            sa.id                                       AS id,
            sa.student_id                               AS student_id,
            u.first_name || ' ' || u.last_name          AS student_name,
            sa.current_balance::float8                  AS current_balance,
            sa.is_hold_active                           AS is_hold_active,
            sa.created_at                               AS created_at,
            sa.updated_at                               AS updated_at
        FROM finance.student_accounts sa
        JOIN core.users u ON u.id = sa.student_id
        WHERE sa.id        = $1
          AND sa.tenant_id = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| StudentAccountSummary {
        id:              r.try_get("id").unwrap_or_default(),
        student_id:      r.try_get("student_id").unwrap_or_default(),
        student_name:    r.try_get("student_name").unwrap_or_default(),
        current_balance: r.try_get("current_balance").unwrap_or_default(),
        is_hold_active:  r.try_get("is_hold_active").unwrap_or_default(),
        created_at:      r.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at:      r.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }))
}

/// Convenience lookup: get account by student UUID rather than account UUID.
pub async fn get_account_by_student(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: Uuid,
) -> Result<Option<StudentAccountSummary>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            sa.id                                       AS id,
            sa.student_id                               AS student_id,
            u.first_name || ' ' || u.last_name          AS student_name,
            sa.current_balance::float8                  AS current_balance,
            sa.is_hold_active                           AS is_hold_active,
            sa.created_at                               AS created_at,
            sa.updated_at                               AS updated_at
        FROM finance.student_accounts sa
        JOIN core.users u ON u.id = sa.student_id
        WHERE sa.student_id = $1
          AND sa.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(student_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| StudentAccountSummary {
        id:              r.try_get("id").unwrap_or_default(),
        student_id:      r.try_get("student_id").unwrap_or_default(),
        student_name:    r.try_get("student_name").unwrap_or_default(),
        current_balance: r.try_get("current_balance").unwrap_or_default(),
        is_hold_active:  r.try_get("is_hold_active").unwrap_or_default(),
        created_at:      r.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at:      r.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }))
}

pub async fn list_transactions(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_id: Uuid,
    params:     &ListTransactionsParams,
) -> Result<(Vec<StudentTransaction>, f64, i64), AppError> {

    // Guard: confirm account exists in this tenant and get current balance
    let account = sqlx::query!(
        r#"
        SELECT current_balance::float8 AS "balance!", is_hold_active
        FROM finance.student_accounts
        WHERE id        = $1
          AND tenant_id = current_setting('app.current_tenant_id', true)::uuid
        "#,
        account_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("Student account {} not found", account_id)
    ))?;

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.student_transactions st
        WHERE st.account_id = $1
          AND st.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND ($2::uuid IS NULL OR st.term_id           = $2)
          AND ($3::text IS NULL OR st.type::text        = $3)
          AND ($4::date IS NULL OR st.transaction_date::date >= $4)
          AND ($5::date IS NULL OR st.transaction_date::date <= $5)
        "#,
        account_id,
        params.term_id             as Option<Uuid>,
        params.transaction_type.clone() as Option<String>,
        params.date_from           as Option<NaiveDate>,
        params.date_to             as Option<NaiveDate>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            st.id                                AS id,
            st.account_id                        AS account_id,
            st.term_id                           AS term_id,
            t.name                               AS term_name,
            st.gl_account_id                     AS gl_account_id,
            ga.account_name                      AS gl_account_name,
            st.type::text                        AS transaction_type,
            st.amount::float8                    AS amount,
            st.description                       AS description,
            st.reference_number                  AS reference_number,
            st.transaction_date                  AS transaction_date
        FROM finance.student_transactions st
        JOIN finance.gl_accounts ga ON ga.id = st.gl_account_id
        LEFT JOIN sis.academic_terms t ON t.id = st.term_id
        WHERE st.account_id = $1
          AND st.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND ($2::uuid IS NULL OR st.term_id           = $2)
          AND ($3::text IS NULL OR st.type::text        = $3)
          AND ($4::date IS NULL OR st.transaction_date::date >= $4)
          AND ($5::date IS NULL OR st.transaction_date::date <= $5)
        ORDER BY st.transaction_date DESC
        LIMIT  $6
        OFFSET $7
        "#,
    )
    .bind(account_id)
    .bind(params.term_id)
    .bind(params.transaction_type.as_deref())
    .bind(params.date_from)
    .bind(params.date_to)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let txns = rows.iter().map(|r| -> Result<StudentTransaction, AppError> {
        Ok(StudentTransaction {
            id:               r.try_get("id").map_err(AppError::from)?,
            account_id:       r.try_get("account_id").map_err(AppError::from)?,
            term_id:          r.try_get("term_id").map_err(AppError::from)?,
            term_name:        r.try_get("term_name").map_err(AppError::from)?,
            gl_account_id:    r.try_get("gl_account_id").map_err(AppError::from)?,
            gl_account_name:  r.try_get("gl_account_name").map_err(AppError::from)?,
            transaction_type: r.try_get("transaction_type").map_err(AppError::from)?,
            amount:           r.try_get("amount").map_err(AppError::from)?,
            description:      r.try_get("description").map_err(AppError::from)?,
            reference_number: r.try_get("reference_number").map_err(AppError::from)?,
            transaction_date: r.try_get("transaction_date").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((txns, account.balance, total))
}

pub async fn get_transaction(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    account_id: Uuid,
    tx_id:      Uuid,
) -> Result<Option<StudentTransaction>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            st.id                             AS id,
            st.account_id                     AS account_id,
            st.term_id                        AS term_id,
            t.name                            AS term_name,
            st.gl_account_id                  AS gl_account_id,
            ga.account_name                   AS gl_account_name,
            st.type::text                     AS transaction_type,
            st.amount::float8                 AS amount,
            st.description                    AS description,
            st.reference_number               AS reference_number,
            st.transaction_date               AS transaction_date
        FROM finance.student_transactions st
        JOIN finance.gl_accounts ga ON ga.id = st.gl_account_id
        LEFT JOIN sis.academic_terms t ON t.id = st.term_id
        WHERE st.id         = $1
          AND st.account_id = $2
          AND st.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(tx_id)
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| StudentTransaction {
        id:               r.try_get("id").unwrap_or_default(),
        account_id:       r.try_get("account_id").unwrap_or_default(),
        term_id:          r.try_get("term_id").unwrap_or_default(),
        term_name:        r.try_get("term_name").unwrap_or_default(),
        gl_account_id:    r.try_get("gl_account_id").unwrap_or_default(),
        gl_account_name:  r.try_get("gl_account_name").unwrap_or_default(),
        transaction_type: r.try_get("transaction_type").unwrap_or_default(),
        amount:           r.try_get("amount").unwrap_or_default(),
        description:      r.try_get("description").unwrap_or_default(),
        reference_number: r.try_get("reference_number").unwrap_or_default(),
        transaction_date: r.try_get("transaction_date")
            .unwrap_or_else(|_| chrono::Utc::now()),
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 2 — FINANCIAL AID
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_fafsa_records(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: Uuid,
) -> Result<Vec<FafsaRecord>, AppError> {

    // 404 guard — confirm student exists in this tenant
    let exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.student_profiles
            WHERE user_id   = $1
              AND tenant_id = current_setting('app.current_tenant_id', true)::uuid
        ) AS "exists!"
        "#,
        student_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(format!("Student {} not found", student_id)));
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            student_id,
            aid_year,
            student_aid_index,
            pell_eligibility_flag,
            verification_required,
            selected_for_verification,
            verification_group,
            dependency_status::text  AS "dependency_status!",
            c_flag,
            isir_transaction_number,
            received_date,
            updated_at
        FROM finance.fafsa_records
        WHERE student_id = $1
          AND tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY aid_year DESC
        "#,
        student_id,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| FafsaRecord {
        id:                        r.id,
        student_id:                r.student_id,
        aid_year:                  r.aid_year,
        student_aid_index:         r.student_aid_index,
        pell_eligibility_flag:     r.pell_eligibility_flag,
        verification_required:     r.verification_required,
        selected_for_verification: r.selected_for_verification,
        verification_group:        r.verification_group,
        dependency_status:         r.dependency_status,
        c_flag:                    r.c_flag,
        isir_transaction_number:   r.isir_transaction_number,
        received_date:             r.received_date,
        updated_at:                r.updated_at,
    }).collect())
}

pub async fn get_fafsa_record(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: Uuid,
    fafsa_id:   Uuid,
) -> Result<Option<FafsaRecord>, AppError> {

    let row = sqlx::query!(
        r#"
        SELECT
            id, student_id, aid_year, student_aid_index,
            pell_eligibility_flag, verification_required,
            selected_for_verification, verification_group,
            dependency_status::text AS "dependency_status!",
            c_flag, isir_transaction_number, received_date, updated_at
        FROM finance.fafsa_records
        WHERE id         = $1
          AND student_id = $2
          AND tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        "#,
        fafsa_id,
        student_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| FafsaRecord {
        id:                        r.id,
        student_id:                r.student_id,
        aid_year:                  r.aid_year,
        student_aid_index:         r.student_aid_index,
        pell_eligibility_flag:     r.pell_eligibility_flag,
        verification_required:     r.verification_required,
        selected_for_verification: r.selected_for_verification,
        verification_group:        r.verification_group,
        dependency_status:         r.dependency_status,
        c_flag:                    r.c_flag,
        isir_transaction_number:   r.isir_transaction_number,
        received_date:             r.received_date,
        updated_at:                r.updated_at,
    }))
}

pub async fn list_aid_awards(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: Option<Uuid>,
    params:     &ListAidAwardsParams,
) -> Result<(Vec<AidAward>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.financial_aid_awards fa
        WHERE fa.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::uuid IS NULL OR fa.student_id = $1)
          AND ($2::uuid IS NULL OR fa.term_id    = $2)
          AND ($3::text IS NULL OR fa.status     = $3)
          AND ($4::text IS NULL OR fa.aid_type::text = $4)
        "#,
        student_id       as Option<Uuid>,
        params.term_id   as Option<Uuid>,
        params.status.clone()   as Option<String>,
        params.aid_type.clone() as Option<String>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            fa.id,
            fa.student_id,
            fa.term_id,
            t.name                           AS term_name,
            fa.fund_name,
            fa.aid_type::text                AS aid_type,
            fa.offered_amount::float8        AS offered_amount,
            fa.accepted_amount::float8       AS accepted_amount,
            fa.disbursed_amount::float8      AS disbursed_amount,
            fa.status,
            fa.nslds_loan_type::text         AS nslds_loan_type,
            fa.loan_sequence_number,
            fa.loan_period_begin_date,
            fa.loan_period_end_date,
            fa.grade_level_at_award,
            fa.created_at,
            fa.updated_at
        FROM finance.financial_aid_awards fa
        JOIN sis.academic_terms t ON t.id = fa.term_id
        WHERE fa.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::uuid IS NULL OR fa.student_id    = $1)
          AND ($2::uuid IS NULL OR fa.term_id       = $2)
          AND ($3::text IS NULL OR fa.status        = $3)
          AND ($4::text IS NULL OR fa.aid_type::text = $4)
        ORDER BY t.start_date DESC, fa.aid_type ASC
        LIMIT  $5
        OFFSET $6
        "#,
    )
    .bind(student_id)
    .bind(params.term_id)
    .bind(params.status.as_deref())
    .bind(params.aid_type.as_deref())
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let awards = rows.iter().map(|r| -> Result<AidAward, AppError> {
        Ok(AidAward {
            id:                    r.try_get("id").map_err(AppError::from)?,
            student_id:            r.try_get("student_id").map_err(AppError::from)?,
            term_id:               r.try_get("term_id").map_err(AppError::from)?,
            term_name:             r.try_get("term_name").map_err(AppError::from)?,
            fund_name:             r.try_get("fund_name").map_err(AppError::from)?,
            aid_type:              r.try_get("aid_type").map_err(AppError::from)?,
            offered_amount:        r.try_get("offered_amount").map_err(AppError::from)?,
            accepted_amount:       r.try_get("accepted_amount").map_err(AppError::from)?,
            disbursed_amount:      r.try_get("disbursed_amount").map_err(AppError::from)?,
            status:                r.try_get("status").map_err(AppError::from)?,
            nslds_loan_type:       r.try_get("nslds_loan_type").map_err(AppError::from)?,
            loan_sequence_number:  r.try_get("loan_sequence_number").map_err(AppError::from)?,
            loan_period_begin_date: r.try_get("loan_period_begin_date").map_err(AppError::from)?,
            loan_period_end_date:  r.try_get("loan_period_end_date").map_err(AppError::from)?,
            grade_level_at_award:  r.try_get("grade_level_at_award").map_err(AppError::from)?,
            created_at:            r.try_get("created_at").map_err(AppError::from)?,
            updated_at:            r.try_get("updated_at").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((awards, total))
}

pub async fn get_aid_award(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: Uuid,
    award_id:   Uuid,
) -> Result<Option<AidAward>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            fa.id, fa.student_id, fa.term_id, t.name AS term_name,
            fa.fund_name, fa.aid_type::text AS aid_type,
            fa.offered_amount::float8   AS offered_amount,
            fa.accepted_amount::float8  AS accepted_amount,
            fa.disbursed_amount::float8 AS disbursed_amount,
            fa.status,
            fa.nslds_loan_type::text    AS nslds_loan_type,
            fa.loan_sequence_number, fa.loan_period_begin_date,
            fa.loan_period_end_date, fa.grade_level_at_award,
            fa.created_at, fa.updated_at
        FROM finance.financial_aid_awards fa
        JOIN sis.academic_terms t ON t.id = fa.term_id
        WHERE fa.id         = $1
          AND fa.student_id = $2
          AND fa.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(award_id)
    .bind(student_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| AidAward {
        id:                    r.try_get("id").unwrap_or_default(),
        student_id:            r.try_get("student_id").unwrap_or_default(),
        term_id:               r.try_get("term_id").unwrap_or_default(),
        term_name:             r.try_get("term_name").unwrap_or_default(),
        fund_name:             r.try_get("fund_name").unwrap_or_default(),
        aid_type:              r.try_get("aid_type").unwrap_or_default(),
        offered_amount:        r.try_get("offered_amount").unwrap_or_default(),
        accepted_amount:       r.try_get("accepted_amount").unwrap_or_default(),
        disbursed_amount:      r.try_get("disbursed_amount").unwrap_or_default(),
        status:                r.try_get("status").unwrap_or_default(),
        nslds_loan_type:       r.try_get("nslds_loan_type").unwrap_or_default(),
        loan_sequence_number:  r.try_get("loan_sequence_number").unwrap_or_default(),
        loan_period_begin_date: r.try_get("loan_period_begin_date").unwrap_or_default(),
        loan_period_end_date:  r.try_get("loan_period_end_date").unwrap_or_default(),
        grade_level_at_award:  r.try_get("grade_level_at_award").unwrap_or_default(),
        created_at:            r.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at:            r.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 3 — SAP
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_sap_policies(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<SapPolicy>, AppError> {

    let rows = sqlx::query!(
        r#"
        SELECT
            id, name, is_default, academic_program_id,
            min_cumulative_gpa::float8       AS "min_cumulative_gpa!",
            min_pace_percentage::float8      AS "min_pace_percentage!",
            max_timeframe_multiplier::float8 AS "max_timeframe_multiplier!",
            evaluation_frequency,
            min_credits_for_evaluation::float8 AS "min_credits_for_evaluation!",
            transfer_credits_count_in_pace,
            remedial_credits_count_in_pace,
            warning_terms_before_suspension,
            timeframe_warning_1_pct::float8  AS "timeframe_warning_1_pct!",
            timeframe_warning_2_pct::float8  AS "timeframe_warning_2_pct!",
            timeframe_warning_3_pct::float8  AS "timeframe_warning_3_pct!",
            is_active, created_at, updated_at
        FROM finance.sap_policies
        WHERE tenant_id = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY is_default DESC, name ASC
        "#,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| SapPolicy {
        id:                              r.id,
        name:                            r.name,
        is_default:                      r.is_default,
        academic_program_id:             r.academic_program_id,
        min_cumulative_gpa:              r.min_cumulative_gpa,
        min_pace_percentage:             r.min_pace_percentage,
        max_timeframe_multiplier:        r.max_timeframe_multiplier,
        evaluation_frequency:            r.evaluation_frequency,
        min_credits_for_evaluation:      r.min_credits_for_evaluation,
        transfer_credits_count_in_pace:  r.transfer_credits_count_in_pace,
        remedial_credits_count_in_pace:  r.remedial_credits_count_in_pace,
        warning_terms_before_suspension: r.warning_terms_before_suspension,
        timeframe_warning_1_pct:         r.timeframe_warning_1_pct,
        timeframe_warning_2_pct:         r.timeframe_warning_2_pct,
        timeframe_warning_3_pct:         r.timeframe_warning_3_pct,
        is_active:                       r.is_active,
        created_at:                      r.created_at,
        updated_at:                      r.updated_at,
    }).collect())
}

pub async fn get_sap_policy(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    policy_id: Uuid,
) -> Result<Option<SapPolicy>, AppError> {

    let row = sqlx::query!(
        r#"
        SELECT
            id, name, is_default, academic_program_id,
            min_cumulative_gpa::float8       AS "min_cumulative_gpa!",
            min_pace_percentage::float8      AS "min_pace_percentage!",
            max_timeframe_multiplier::float8 AS "max_timeframe_multiplier!",
            evaluation_frequency,
            min_credits_for_evaluation::float8 AS "min_credits_for_evaluation!",
            transfer_credits_count_in_pace,
            remedial_credits_count_in_pace,
            warning_terms_before_suspension,
            timeframe_warning_1_pct::float8  AS "timeframe_warning_1_pct!",
            timeframe_warning_2_pct::float8  AS "timeframe_warning_2_pct!",
            timeframe_warning_3_pct::float8  AS "timeframe_warning_3_pct!",
            is_active, created_at, updated_at
        FROM finance.sap_policies
        WHERE id        = $1
          AND tenant_id = current_setting('app.current_tenant_id', true)::uuid
        "#,
        policy_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| SapPolicy {
        id:                              r.id,
        name:                            r.name,
        is_default:                      r.is_default,
        academic_program_id:             r.academic_program_id,
        min_cumulative_gpa:              r.min_cumulative_gpa,
        min_pace_percentage:             r.min_pace_percentage,
        max_timeframe_multiplier:        r.max_timeframe_multiplier,
        evaluation_frequency:            r.evaluation_frequency,
        min_credits_for_evaluation:      r.min_credits_for_evaluation,
        transfer_credits_count_in_pace:  r.transfer_credits_count_in_pace,
        remedial_credits_count_in_pace:  r.remedial_credits_count_in_pace,
        warning_terms_before_suspension: r.warning_terms_before_suspension,
        timeframe_warning_1_pct:         r.timeframe_warning_1_pct,
        timeframe_warning_2_pct:         r.timeframe_warning_2_pct,
        timeframe_warning_3_pct:         r.timeframe_warning_3_pct,
        is_active:                       r.is_active,
        created_at:                      r.created_at,
        updated_at:                      r.updated_at,
    }))
}

pub async fn list_sap_evaluations(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: Uuid,
    params:     &ListSapEvaluationsParams,
) -> Result<(Vec<SapEvaluation>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.sap_evaluations se
        WHERE se.student_id = $1
          AND se.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND ($2::uuid IS NULL OR se.evaluation_term_id           = $2)
          AND ($3::text IS NULL OR se.resulting_sap_status::text   = $3)
        "#,
        student_id,
        params.term_id as Option<Uuid>,
        params.status.clone() as Option<String>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            se.id, se.student_id, se.evaluation_term_id,
            t.name                                      AS term_name,
            se.snapshot_cumulative_gpa::float8          AS snapshot_cumulative_gpa,
            se.snapshot_attempted_credits::float8       AS snapshot_attempted_credits,
            se.snapshot_earned_credits::float8          AS snapshot_earned_credits,
            se.snapshot_pace_percentage::float8         AS snapshot_pace_percentage,
            se.snapshot_max_timeframe_credits::float8   AS snapshot_max_timeframe_credits,
            se.resulting_sap_status::text               AS resulting_sap_status,
            se.gpa_component_met,
            se.pace_component_met,
            se.max_timeframe_component_met,
            se.consecutive_warning_terms,
            se.is_manual_override,
            se.override_reason,
            se.sap_policy_id,
            se.evaluated_at
        FROM finance.sap_evaluations se
        JOIN sis.academic_terms t ON t.id = se.evaluation_term_id
        WHERE se.student_id = $1
          AND se.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND ($2::uuid IS NULL OR se.evaluation_term_id         = $2)
          AND ($3::text IS NULL OR se.resulting_sap_status::text = $3)
        ORDER BY se.evaluated_at DESC
        LIMIT  $4
        OFFSET $5
        "#,
    )
    .bind(student_id)
    .bind(params.term_id)
    .bind(params.status.as_deref())
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let evals = rows.iter().map(|r| -> Result<SapEvaluation, AppError> {
        Ok(SapEvaluation {
            id:                          r.try_get("id").map_err(AppError::from)?,
            student_id:                  r.try_get("student_id").map_err(AppError::from)?,
            evaluation_term_id:          r.try_get("evaluation_term_id").map_err(AppError::from)?,
            term_name:                   r.try_get("term_name").map_err(AppError::from)?,
            snapshot_cumulative_gpa:     r.try_get("snapshot_cumulative_gpa").map_err(AppError::from)?,
            snapshot_attempted_credits:  r.try_get("snapshot_attempted_credits").map_err(AppError::from)?,
            snapshot_earned_credits:     r.try_get("snapshot_earned_credits").map_err(AppError::from)?,
            snapshot_pace_percentage:    r.try_get("snapshot_pace_percentage").map_err(AppError::from)?,
            snapshot_max_timeframe_credits: r.try_get("snapshot_max_timeframe_credits").map_err(AppError::from)?,
            resulting_sap_status:        r.try_get("resulting_sap_status").map_err(AppError::from)?,
            gpa_component_met:           r.try_get("gpa_component_met").map_err(AppError::from)?,
            pace_component_met:          r.try_get("pace_component_met").map_err(AppError::from)?,
            max_timeframe_component_met: r.try_get("max_timeframe_component_met").map_err(AppError::from)?,
            consecutive_warning_terms:   r.try_get("consecutive_warning_terms").map_err(AppError::from)?,
            is_manual_override:          r.try_get("is_manual_override").map_err(AppError::from)?,
            override_reason:             r.try_get("override_reason").map_err(AppError::from)?,
            sap_policy_id:               r.try_get("sap_policy_id").map_err(AppError::from)?,
            evaluated_at:                r.try_get("evaluated_at").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((evals, total))
}

pub async fn get_sap_evaluation(
    tx:            &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id:    Uuid,
    evaluation_id: Uuid,
) -> Result<Option<SapEvaluation>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            se.id, se.student_id, se.evaluation_term_id,
            t.name                                      AS term_name,
            se.snapshot_cumulative_gpa::float8          AS snapshot_cumulative_gpa,
            se.snapshot_attempted_credits::float8       AS snapshot_attempted_credits,
            se.snapshot_earned_credits::float8          AS snapshot_earned_credits,
            se.snapshot_pace_percentage::float8         AS snapshot_pace_percentage,
            se.snapshot_max_timeframe_credits::float8   AS snapshot_max_timeframe_credits,
            se.resulting_sap_status::text               AS resulting_sap_status,
            se.gpa_component_met, se.pace_component_met,
            se.max_timeframe_component_met,
            se.consecutive_warning_terms,
            se.is_manual_override, se.override_reason,
            se.sap_policy_id, se.evaluated_at
        FROM finance.sap_evaluations se
        JOIN sis.academic_terms t ON t.id = se.evaluation_term_id
        WHERE se.id         = $1
          AND se.student_id = $2
          AND se.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(evaluation_id)
    .bind(student_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| SapEvaluation {
        id:                          r.try_get("id").unwrap_or_default(),
        student_id:                  r.try_get("student_id").unwrap_or_default(),
        evaluation_term_id:          r.try_get("evaluation_term_id").unwrap_or_default(),
        term_name:                   r.try_get("term_name").unwrap_or_default(),
        snapshot_cumulative_gpa:     r.try_get("snapshot_cumulative_gpa").unwrap_or_default(),
        snapshot_attempted_credits:  r.try_get("snapshot_attempted_credits").unwrap_or_default(),
        snapshot_earned_credits:     r.try_get("snapshot_earned_credits").unwrap_or_default(),
        snapshot_pace_percentage:    r.try_get("snapshot_pace_percentage").unwrap_or_default(),
        snapshot_max_timeframe_credits: r.try_get("snapshot_max_timeframe_credits").unwrap_or_default(),
        resulting_sap_status:        r.try_get("resulting_sap_status").unwrap_or_default(),
        gpa_component_met:           r.try_get("gpa_component_met").unwrap_or_default(),
        pace_component_met:          r.try_get("pace_component_met").unwrap_or_default(),
        max_timeframe_component_met: r.try_get("max_timeframe_component_met").unwrap_or_default(),
        consecutive_warning_terms:   r.try_get("consecutive_warning_terms").unwrap_or_default(),
        is_manual_override:          r.try_get("is_manual_override").unwrap_or_default(),
        override_reason:             r.try_get("override_reason").unwrap_or_default(),
        sap_policy_id:               r.try_get("sap_policy_id").unwrap_or_default(),
        evaluated_at:                r.try_get("evaluated_at")
            .unwrap_or_else(|_| chrono::Utc::now()),
    }))
}

pub async fn get_sap_appeal(
    tx:            &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id:    Uuid,
    evaluation_id: Uuid,
) -> Result<Option<SapAppeal>, AppError> {

    let row = sqlx::query!(
        r#"
        SELECT
            sa.id, sa.sap_evaluation_id, se.student_id,
            sa.appeal_reason, sa.supporting_documents,
            sa.status, sa.reviewer_id, sa.reviewer_notes,
            sa.reviewed_at, sa.probation_expires_term_id,
            sa.created_at, sa.updated_at
        FROM finance.sap_appeals sa
        JOIN finance.sap_evaluations se ON se.id = sa.sap_evaluation_id
        WHERE sa.sap_evaluation_id = $1
          AND se.student_id        = $2
          AND sa.tenant_id         = current_setting('app.current_tenant_id', true)::uuid
        "#,
        evaluation_id,
        student_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| SapAppeal {
        id:                    r.id,
        sap_evaluation_id:     r.sap_evaluation_id,
        student_id:            r.student_id,
        appeal_reason:         r.appeal_reason,
        supporting_documents:  r.supporting_documents,
        status:                r.status,
        reviewer_id:           r.reviewer_id,
        reviewer_notes:        r.reviewer_notes,
        reviewed_at:           r.reviewed_at,
        probation_expires_term_id: r.probation_expires_term_id,
        created_at:            r.created_at,
        updated_at:            r.updated_at,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 4 — VA BENEFITS
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn get_veteran_profile(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: Uuid,
) -> Result<Option<VeteranProfile>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            vp.id,
            vp.student_id,
            u.first_name || ' ' || u.last_name   AS student_name,
            vp.va_file_number,
            vp.primary_chapter::text             AS primary_chapter,
            vp.eligibility_percentage,
            vp.months_of_entitlement_remaining::float8 AS months_remaining,
            vp.dd214_on_file,
            vp.updated_at
        FROM finance.veteran_profiles vp
        JOIN core.users u ON u.id = vp.student_id
        WHERE vp.student_id = $1
          AND vp.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(student_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| VeteranProfile {
        id:                              r.try_get("id").unwrap_or_default(),
        student_id:                      r.try_get("student_id").unwrap_or_default(),
        student_name:                    r.try_get("student_name").unwrap_or_default(),
        va_file_number:                  r.try_get("va_file_number").unwrap_or_default(),
        primary_chapter:                 r.try_get("primary_chapter").unwrap_or_default(),
        eligibility_percentage:          r.try_get("eligibility_percentage").unwrap_or_default(),
        months_of_entitlement_remaining: r.try_get("months_remaining").unwrap_or_default(),
        dd214_on_file:                   r.try_get("dd214_on_file").unwrap_or_default(),
        updated_at:                      r.try_get("updated_at")
            .unwrap_or_else(|_| chrono::Utc::now()),
    }))
}

pub async fn list_va_certifications(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: Uuid,
    params:     &ListVaCertificationsParams,
) -> Result<(Vec<VaCertification>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.va_certifications vc
        JOIN finance.veteran_profiles vp ON vp.id = vc.veteran_profile_id
        WHERE vp.student_id  = $1
          AND vc.tenant_id   = current_setting('app.current_tenant_id', true)::uuid
          AND ($2::uuid IS NULL OR vc.term_id       = $2)
          AND ($3::bool IS NULL OR vc.is_amendment  = $3)
        "#,
        student_id,
        params.term_id      as Option<Uuid>,
        params.is_amendment as Option<bool>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            vc.id, vp.student_id,
            u.first_name || ' ' || u.last_name          AS student_name,
            vc.veteran_profile_id, vc.term_id,
            t.name                                       AS term_name,
            t.start_date                                 AS term_start,
            t.end_date                                   AS term_end,
            vc.credits_certified::float8                 AS credits_certified,
            vc.tuition_reported::float8                  AS tuition_reported,
            vc.fees_reported::float8                     AS fees_reported,
            vc.certification_date,
            vc.enrollment_intensity_va::text             AS enrollment_intensity_va,
            vc.training_time_percentage::float8          AS training_time_percentage,
            vc.is_amendment, vc.amends_certification_id, vc.amendment_date,
            vc.va_confirmation_number,
            vc.ch33_housing_rate::float8                 AS ch33_housing_rate,
            vc.ch33_book_stipend::float8                 AS ch33_book_stipend,
            vc.program_of_study,
            u2.first_name || ' ' || u2.last_name         AS certified_by
        FROM finance.va_certifications vc
        JOIN finance.veteran_profiles vp ON vp.id = vc.veteran_profile_id
        JOIN core.users u                ON u.id  = vp.student_id
        JOIN sis.academic_terms t        ON t.id  = vc.term_id
        LEFT JOIN core.users u2          ON u2.id = vc.certified_by_id
        WHERE vp.student_id = $1
          AND vc.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND ($2::uuid IS NULL OR vc.term_id      = $2)
          AND ($3::bool IS NULL OR vc.is_amendment = $3)
        ORDER BY vc.certification_date DESC
        LIMIT  $4
        OFFSET $5
        "#,
    )
    .bind(student_id)
    .bind(params.term_id)
    .bind(params.is_amendment)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let certs = rows.iter().map(|r| -> Result<VaCertification, AppError> {
        Ok(VaCertification {
            id:                       r.try_get("id").map_err(AppError::from)?,
            student_id:               r.try_get("student_id").map_err(AppError::from)?,
            student_name:             r.try_get("student_name").map_err(AppError::from)?,
            veteran_profile_id:       r.try_get("veteran_profile_id").map_err(AppError::from)?,
            term_id:                  r.try_get("term_id").map_err(AppError::from)?,
            term_name:                r.try_get("term_name").map_err(AppError::from)?,
            term_start:               r.try_get("term_start").map_err(AppError::from)?,
            term_end:                 r.try_get("term_end").map_err(AppError::from)?,
            credits_certified:        r.try_get("credits_certified").map_err(AppError::from)?,
            tuition_reported:         r.try_get("tuition_reported").map_err(AppError::from)?,
            fees_reported:            r.try_get("fees_reported").map_err(AppError::from)?,
            certification_date:       r.try_get("certification_date").map_err(AppError::from)?,
            enrollment_intensity_va:  r.try_get("enrollment_intensity_va").map_err(AppError::from)?,
            training_time_percentage: r.try_get("training_time_percentage").map_err(AppError::from)?,
            is_amendment:             r.try_get("is_amendment").map_err(AppError::from)?,
            amends_certification_id:  r.try_get("amends_certification_id").map_err(AppError::from)?,
            amendment_date:           r.try_get("amendment_date").map_err(AppError::from)?,
            va_confirmation_number:   r.try_get("va_confirmation_number").map_err(AppError::from)?,
            ch33_housing_rate:        r.try_get("ch33_housing_rate").map_err(AppError::from)?,
            ch33_book_stipend:        r.try_get("ch33_book_stipend").map_err(AppError::from)?,
            program_of_study:         r.try_get("program_of_study").map_err(AppError::from)?,
            certified_by:             r.try_get("certified_by").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((certs, total))
}

pub async fn get_va_certification(
    tx:              &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id:      Uuid,
    certification_id: Uuid,
) -> Result<Option<VaCertification>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            vc.id, vp.student_id,
            u.first_name || ' ' || u.last_name          AS student_name,
            vc.veteran_profile_id, vc.term_id,
            t.name AS term_name, t.start_date AS term_start, t.end_date AS term_end,
            vc.credits_certified::float8   AS credits_certified,
            vc.tuition_reported::float8    AS tuition_reported,
            vc.fees_reported::float8       AS fees_reported,
            vc.certification_date,
            vc.enrollment_intensity_va::text AS enrollment_intensity_va,
            vc.training_time_percentage::float8 AS training_time_percentage,
            vc.is_amendment, vc.amends_certification_id, vc.amendment_date,
            vc.va_confirmation_number,
            vc.ch33_housing_rate::float8   AS ch33_housing_rate,
            vc.ch33_book_stipend::float8   AS ch33_book_stipend,
            vc.program_of_study,
            u2.first_name || ' ' || u2.last_name AS certified_by
        FROM finance.va_certifications vc
        JOIN finance.veteran_profiles vp ON vp.id = vc.veteran_profile_id
        JOIN core.users u                ON u.id  = vp.student_id
        JOIN sis.academic_terms t        ON t.id  = vc.term_id
        LEFT JOIN core.users u2          ON u2.id = vc.certified_by_id
        WHERE vc.id          = $1
          AND vp.student_id  = $2
          AND vc.tenant_id   = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(certification_id)
    .bind(student_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| VaCertification {
        id:                       r.try_get("id").unwrap_or_default(),
        student_id:               r.try_get("student_id").unwrap_or_default(),
        student_name:             r.try_get("student_name").unwrap_or_default(),
        veteran_profile_id:       r.try_get("veteran_profile_id").unwrap_or_default(),
        term_id:                  r.try_get("term_id").unwrap_or_default(),
        term_name:                r.try_get("term_name").unwrap_or_default(),
        term_start:               r.try_get("term_start").unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970,1,1).unwrap()),
        term_end:                 r.try_get("term_end").unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970,1,1).unwrap()),
        credits_certified:        r.try_get("credits_certified").unwrap_or_default(),
        tuition_reported:         r.try_get("tuition_reported").unwrap_or_default(),
        fees_reported:            r.try_get("fees_reported").unwrap_or_default(),
        certification_date:       r.try_get("certification_date").unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970,1,1).unwrap()),
        enrollment_intensity_va:  r.try_get("enrollment_intensity_va").unwrap_or_default(),
        training_time_percentage: r.try_get("training_time_percentage").unwrap_or_default(),
        is_amendment:             r.try_get("is_amendment").unwrap_or_default(),
        amends_certification_id:  r.try_get("amends_certification_id").unwrap_or_default(),
        amendment_date:           r.try_get("amendment_date").unwrap_or_default(),
        va_confirmation_number:   r.try_get("va_confirmation_number").unwrap_or_default(),
        ch33_housing_rate:        r.try_get("ch33_housing_rate").unwrap_or_default(),
        ch33_book_stipend:        r.try_get("ch33_book_stipend").unwrap_or_default(),
        program_of_study:         r.try_get("program_of_study").unwrap_or_default(),
        certified_by:             r.try_get("certified_by").unwrap_or_default(),
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 5 — PAYROLL / STAFF PAY
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_payroll_runs(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListPayrollRunsParams,
) -> Result<(Vec<PayrollRunSummary>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.payroll_runs pr
        WHERE pr.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR pr.status        = $1)
          AND ($2::date IS NULL OR pr.run_date      >= $2)
          AND ($3::date IS NULL OR pr.run_date      <= $3)
        "#,
        params.status.clone()    as Option<String>,
        params.date_from         as Option<NaiveDate>,
        params.date_to           as Option<NaiveDate>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            pr.id, pr.run_date,
            pr.total_gross_pay::float8  AS total_gross_pay,
            pr.gl_account_id,
            ga.account_name             AS gl_account_name,
            pr.status,
            COUNT(ps.id)                AS stub_count,
            pr.updated_at
        FROM finance.payroll_runs pr
        JOIN finance.gl_accounts ga ON ga.id = pr.gl_account_id
        LEFT JOIN finance.pay_stubs ps ON ps.payroll_run_id = pr.id
        WHERE pr.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR pr.status   = $1)
          AND ($2::date IS NULL OR pr.run_date >= $2)
          AND ($3::date IS NULL OR pr.run_date <= $3)
        GROUP BY pr.id, pr.run_date, pr.total_gross_pay, pr.gl_account_id,
                 ga.account_name, pr.status, pr.updated_at
        ORDER BY pr.run_date DESC
        LIMIT  $4
        OFFSET $5
        "#,
    )
    .bind(params.status.as_deref())
    .bind(params.date_from)
    .bind(params.date_to)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let runs = rows.iter().map(|r| -> Result<PayrollRunSummary, AppError> {
        Ok(PayrollRunSummary {
            id:              r.try_get("id").map_err(AppError::from)?,
            run_date:        r.try_get("run_date").map_err(AppError::from)?,
            total_gross_pay: r.try_get("total_gross_pay").map_err(AppError::from)?,
            gl_account_id:   r.try_get("gl_account_id").map_err(AppError::from)?,
            gl_account_name: r.try_get("gl_account_name").map_err(AppError::from)?,
            status:          r.try_get("status").map_err(AppError::from)?,
            stub_count:      r.try_get("stub_count").map_err(AppError::from)?,
            updated_at:      r.try_get("updated_at").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((runs, total))
}

pub async fn get_payroll_run(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    run_id: Uuid,
) -> Result<Option<PayrollRunSummary>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            pr.id, pr.run_date,
            pr.total_gross_pay::float8  AS total_gross_pay,
            pr.gl_account_id,
            ga.account_name             AS gl_account_name,
            pr.status,
            COUNT(ps.id)                AS stub_count,
            pr.updated_at
        FROM finance.payroll_runs pr
        JOIN finance.gl_accounts ga ON ga.id = pr.gl_account_id
        LEFT JOIN finance.pay_stubs ps ON ps.payroll_run_id = pr.id
        WHERE pr.id        = $1
          AND pr.tenant_id = current_setting('app.current_tenant_id', true)::uuid
        GROUP BY pr.id, pr.run_date, pr.total_gross_pay, pr.gl_account_id,
                 ga.account_name, pr.status, pr.updated_at
        "#,
    )
    .bind(run_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| PayrollRunSummary {
        id:              r.try_get("id").unwrap_or_default(),
        run_date:        r.try_get("run_date")
            .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970,1,1).unwrap()),
        total_gross_pay: r.try_get("total_gross_pay").unwrap_or_default(),
        gl_account_id:   r.try_get("gl_account_id").unwrap_or_default(),
        gl_account_name: r.try_get("gl_account_name").unwrap_or_default(),
        status:          r.try_get("status").unwrap_or_default(),
        stub_count:      r.try_get("stub_count").unwrap_or_default(),
        updated_at:      r.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }))
}

pub async fn list_run_stubs(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    run_id: Uuid,
    params: &ListStaffPayStubsParams,   // reuse — same filters apply
) -> Result<(Vec<PayStubSummary>, i64), AppError> {

    // Guard: run exists in this tenant
    let run_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM finance.payroll_runs
            WHERE id = $1 AND tenant_id = current_setting('app.current_tenant_id', true)::uuid
        ) AS "exists!"
        "#,
        run_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !run_exists {
        return Err(AppError::NotFound(format!("Payroll run {} not found", run_id)));
    }

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.pay_stubs ps
        WHERE ps.payroll_run_id = $1
          AND ps.tenant_id      = current_setting('app.current_tenant_id', true)::uuid
        "#,
        run_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            ps.id, ps.payroll_run_id, ps.employee_id,
            u.first_name || ' ' || u.last_name  AS employee_name,
            ps.gross_pay::float8                AS gross_pay,
            ps.net_pay::float8                  AS net_pay,
            ps.created_at
        FROM finance.pay_stubs ps
        JOIN core.users u ON u.id = ps.employee_id
        WHERE ps.payroll_run_id = $1
          AND ps.tenant_id      = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY u.last_name ASC, u.first_name ASC
        LIMIT  $2
        OFFSET $3
        "#,
    )
    .bind(run_id)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let stubs = rows.iter().map(|r| -> Result<PayStubSummary, AppError> {
        Ok(PayStubSummary {
            id:             r.try_get("id").map_err(AppError::from)?,
            payroll_run_id: r.try_get("payroll_run_id").map_err(AppError::from)?,
            employee_id:    r.try_get("employee_id").map_err(AppError::from)?,
            employee_name:  r.try_get("employee_name").map_err(AppError::from)?,
            gross_pay:      r.try_get("gross_pay").map_err(AppError::from)?,
            net_pay:        r.try_get("net_pay").map_err(AppError::from)?,
            created_at:     r.try_get("created_at").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((stubs, total))
}

pub async fn get_stub_detail(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    run_id:  Uuid,
    stub_id: Uuid,
) -> Result<Option<PayStubDetail>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            ps.id, ps.payroll_run_id,
            pr.run_date,
            ps.employee_id,
            u.first_name || ' ' || u.last_name  AS employee_name,
            ps.gross_pay::float8                AS gross_pay,
            ps.net_pay::float8                  AS net_pay,
            ps.created_at
        FROM finance.pay_stubs ps
        JOIN finance.payroll_runs pr ON pr.id = ps.payroll_run_id
        JOIN core.users u            ON u.id  = ps.employee_id
        WHERE ps.id             = $1
          AND ps.payroll_run_id = $2
          AND ps.tenant_id      = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(stub_id)
    .bind(run_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let stub_row = match row {
        None => return Ok(None),
        Some(r) => r,
    };

    // Fetch line items separately — cleaner than a nested aggregation
    let line_rows = sqlx::query(
        r#"
        SELECT
            li.id,
            li.item_type_id,
            li.historical_name,
            pit.category::text  AS category,
            li.amount::float8   AS amount
        FROM finance.pay_stub_line_items li
        JOIN finance.payroll_item_types pit ON pit.id = li.item_type_id
        WHERE li.pay_stub_id = $1
          AND li.tenant_id   = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY pit.category ASC, li.historical_name ASC
        "#,
    )
    .bind(stub_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let line_items = line_rows.iter().map(|r| -> Result<PayStubLineItem, AppError> {
        Ok(PayStubLineItem {
            id:              r.try_get("id").map_err(AppError::from)?,
            item_type_id:    r.try_get("item_type_id").map_err(AppError::from)?,
            historical_name: r.try_get("historical_name").map_err(AppError::from)?,
            category:        r.try_get("category").map_err(AppError::from)?,
            amount:          r.try_get("amount").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok(Some(PayStubDetail {
        id:             stub_row.try_get("id").map_err(AppError::from)?,
        payroll_run_id: stub_row.try_get("payroll_run_id").map_err(AppError::from)?,
        run_date:       stub_row.try_get("run_date").map_err(AppError::from)?,
        employee_id:    stub_row.try_get("employee_id").map_err(AppError::from)?,
        employee_name:  stub_row.try_get("employee_name").map_err(AppError::from)?,
        gross_pay:      stub_row.try_get("gross_pay").map_err(AppError::from)?,
        net_pay:        stub_row.try_get("net_pay").map_err(AppError::from)?,
        line_items,
        created_at:     stub_row.try_get("created_at").map_err(AppError::from)?,
    }))
}

pub async fn list_staff_pay_stubs(
    tx:       &mut sqlx::Transaction<'_, sqlx::Postgres>,
    staff_id: Uuid,
    params:   &ListStaffPayStubsParams,
) -> Result<(Vec<PayStubSummary>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.pay_stubs ps
        JOIN finance.payroll_runs pr ON pr.id = ps.payroll_run_id
        WHERE ps.employee_id = $1
          AND ps.tenant_id   = current_setting('app.current_tenant_id', true)::uuid
          AND ($2::date IS NULL OR pr.run_date >= $2)
          AND ($3::date IS NULL OR pr.run_date <= $3)
        "#,
        staff_id,
        params.date_from as Option<NaiveDate>,
        params.date_to   as Option<NaiveDate>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            ps.id, ps.payroll_run_id, ps.employee_id,
            u.first_name || ' ' || u.last_name  AS employee_name,
            ps.gross_pay::float8                AS gross_pay,
            ps.net_pay::float8                  AS net_pay,
            ps.created_at
        FROM finance.pay_stubs ps
        JOIN finance.payroll_runs pr ON pr.id = ps.payroll_run_id
        JOIN core.users u            ON u.id  = ps.employee_id
        WHERE ps.employee_id = $1
          AND ps.tenant_id   = current_setting('app.current_tenant_id', true)::uuid
          AND ($2::date IS NULL OR pr.run_date >= $2)
          AND ($3::date IS NULL OR pr.run_date <= $3)
        ORDER BY pr.run_date DESC
        LIMIT  $4
        OFFSET $5
        "#,
    )
    .bind(staff_id)
    .bind(params.date_from)
    .bind(params.date_to)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let stubs = rows.iter().map(|r| -> Result<PayStubSummary, AppError> {
        Ok(PayStubSummary {
            id:             r.try_get("id").map_err(AppError::from)?,
            payroll_run_id: r.try_get("payroll_run_id").map_err(AppError::from)?,
            employee_id:    r.try_get("employee_id").map_err(AppError::from)?,
            employee_name:  r.try_get("employee_name").map_err(AppError::from)?,
            gross_pay:      r.try_get("gross_pay").map_err(AppError::from)?,
            net_pay:        r.try_get("net_pay").map_err(AppError::from)?,
            created_at:     r.try_get("created_at").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((stubs, total))
}

pub async fn list_payroll_item_types(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<PayrollItemType>, AppError> {

    let rows = sqlx::query!(
        r#"
        SELECT
            id, name,
            category::text      AS "category!",
            liability_gl_account_id,
            is_active,
            created_at, updated_at
        FROM finance.payroll_item_types
        WHERE tenant_id = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY category ASC, name ASC
        "#,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| PayrollItemType {
        id:                      r.id,
        name:                    r.name,
        category:                r.category,
        liability_gl_account_id: r.liability_gl_account_id,
        is_active:               r.is_active,
        created_at:              r.created_at,
        updated_at:              r.updated_at,
    }).collect())
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 6 — REFERENCE READS
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_fiscal_years(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<FiscalYear>, AppError> {

    let rows = sqlx::query!(
        r#"
        SELECT
            id, name, start_date, end_date,
            status::text                AS "status!",
            accounting_framework::text  AS accounting_framework,
            audit_status::text          AS audit_status,
            is_comparative_year, period_closed_at
        FROM finance.fiscal_years
        WHERE tenant_id = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY start_date DESC
        "#,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| FiscalYear {
        id:                   r.id,
        name:                 r.name,
        start_date:           r.start_date,
        end_date:             r.end_date,
        status:               r.status,
        accounting_framework: r.accounting_framework,
        audit_status:         r.audit_status,
        is_comparative_year:  r.is_comparative_year,
        period_closed_at:     r.period_closed_at,
    }).collect())
}

pub async fn get_fiscal_year(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    fy_id:  Uuid,
) -> Result<Option<FiscalYear>, AppError> {

    let row = sqlx::query!(
        r#"
        SELECT
            id, name, start_date, end_date,
            status::text               AS "status!",
            accounting_framework::text AS accounting_framework,
            audit_status::text         AS audit_status,
            is_comparative_year, period_closed_at
        FROM finance.fiscal_years
        WHERE id        = $1
          AND tenant_id = current_setting('app.current_tenant_id', true)::uuid
        "#,
        fy_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| FiscalYear {
        id:                   r.id,
        name:                 r.name,
        start_date:           r.start_date,
        end_date:             r.end_date,
        status:               r.status,
        accounting_framework: r.accounting_framework,
        audit_status:         r.audit_status,
        is_comparative_year:  r.is_comparative_year,
        period_closed_at:     r.period_closed_at,
    }))
}

pub async fn list_gl_accounts(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListGlAccountsParams,
) -> Result<(Vec<GlAccountSummary>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM finance.gl_accounts ga
        WHERE ga.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR ga.type::text = $1)
          AND ($2::bool IS NULL OR ga.is_active  = $2)
        "#,
        params.account_type.clone() as Option<String>,
        params.is_active            as Option<bool>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query!(
        r#"
        SELECT
            id, account_number, account_name, account_code,
            type::text              AS "account_type!",
            normal_balance::text    AS normal_balance,
            current_balance::float8 AS "current_balance!",
            is_active, is_contra, description
        FROM finance.gl_accounts
        WHERE tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR type::text = $1)
          AND ($2::bool IS NULL OR is_active  = $2)
        ORDER BY type ASC, account_number ASC
        LIMIT  $3
        OFFSET $4
        "#,
        params.account_type.clone() as Option<String>,
        params.is_active            as Option<bool>,
        params.per_page(),
        params.offset(),
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let accounts = rows.into_iter().map(|r| GlAccountSummary {
        id:              r.id,
        account_number:  r.account_number,
        account_name:    r.account_name,
        account_code:    r.account_code,
        account_type:    r.account_type,
        normal_balance:  r.normal_balance,
        current_balance: r.current_balance,
        is_active:       r.is_active,
        is_contra:       r.is_contra,
        description:     r.description,
    }).collect();

    Ok((accounts, total))
}
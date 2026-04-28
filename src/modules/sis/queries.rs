// src/modules/sis/queries.rs

use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use super::models::{ListStudentsParams, StudentSummary};

/// Fetch a paginated list of students for the authenticated tenant.
///
/// Joins:
///   core.users             — name, preferred_name, username
///   sis.student_profiles   — GPA, enrollment year, academic standing
///   sis.student_programs   — primary program name (is_primary + is_current)
///   sis.academic_programs  — program name text
///
/// RLS on all four tables is enforced by the transaction context set in
/// begin_rls_transaction(). No explicit tenant_id filter is needed in the
/// WHERE clause — RLS handles it — but we add it anyway for query clarity
/// and to keep the query planner from choosing a full-scan plan.
///
/// student_demographics is intentionally excluded from this list endpoint.
/// date_of_birth, race, SSN fragments etc. are FERPA-restricted and belong
/// in a dedicated single-student detail endpoint with role checks.
pub async fn list_students(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListStudentsParams,
) -> Result<(Vec<StudentSummary>, i64), AppError> {

    // ── Total count (same filters, no pagination) ─────────────────────────────
    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM sis.student_profiles sp
        JOIN core.users u ON u.id = sp.user_id
        WHERE sp.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text    IS NULL OR sp.academic_standing_status::text = $1)
          AND ($2::uuid    IS NULL OR EXISTS (
                SELECT 1
                FROM sis.student_programs prg
                WHERE prg.student_id = sp.user_id
                  AND prg.program_id = $2
                  AND prg.is_primary = true
                  AND prg.is_current = true
              ))
        "#,
        params.standing.clone()   as Option<String>,
        params.program_id as Option<Uuid>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    // ── Paginated rows ────────────────────────────────────────────────────────
    // Uses sqlx::query (untyped) because the optional LEFT JOIN result columns
    // (program name, expected_grad_year) are Option<String>/Option<i32> and
    // the typed macro struggles with nullable columns from outer joins.
    let rows = sqlx::query(
        r#"
        SELECT
            sp.user_id                          AS user_id,
            u.first_name                        AS first_name,
            u.last_name                         AS last_name,
            u.preferred_name                    AS preferred_name,
            u.username                          AS username,
            sp.enrollment_year                  AS enrollment_year,
            sp.cumulative_gpa::float8           AS cumulative_gpa,
            sp.term_gpa::float8                 AS term_gpa,
            sp.academic_standing_status::text   AS academic_standing,
            sp.expected_graduation_year         AS expected_grad_year,
            ap.name                             AS primary_program
        FROM sis.student_profiles sp
        JOIN core.users u ON u.id = sp.user_id
        LEFT JOIN sis.student_programs prg
               ON prg.student_id = sp.user_id
              AND prg.is_primary  = true
              AND prg.is_current  = true
        LEFT JOIN sis.academic_programs ap
               ON ap.id = prg.program_id
        WHERE sp.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR sp.academic_standing_status::text = $1)
          AND ($2::uuid IS NULL OR (prg.program_id = $2))
        ORDER BY u.last_name ASC, u.first_name ASC
        LIMIT  $3
        OFFSET $4
        "#,
    )
    .bind(&params.standing)
    .bind(params.program_id)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let students = rows
        .iter()
        .map(|row| map_row(row))
        .collect::<Result<Vec<_>, _>>()?;

    Ok((students, total))
}

fn map_row(row: &PgRow) -> Result<StudentSummary, AppError> {
    // cumulative_gpa and term_gpa are cast to float8 in the SQL query,
    // so sqlx maps them directly to Option<f64> without needing rust_decimal.
    let cumulative_gpa: Option<f64> = row.try_get("cumulative_gpa").map_err(AppError::from)?;
    let term_gpa:       Option<f64> = row.try_get("term_gpa").map_err(AppError::from)?;

    Ok(StudentSummary {
        user_id:            row.try_get("user_id").map_err(AppError::from)?,
        first_name:         row.try_get("first_name").map_err(AppError::from)?,
        last_name:          row.try_get("last_name").map_err(AppError::from)?,
        preferred_name:     row.try_get("preferred_name").map_err(AppError::from)?,
        username:           row.try_get("username").map_err(AppError::from)?,
        enrollment_year:    row.try_get("enrollment_year").map_err(AppError::from)?,
        cumulative_gpa,
        term_gpa,
        academic_standing:  row.try_get("academic_standing").map_err(AppError::from)?,
        primary_program:    row.try_get("primary_program").map_err(AppError::from)?,
        expected_grad_year: row.try_get("expected_grad_year").map_err(AppError::from)?,
    })
}
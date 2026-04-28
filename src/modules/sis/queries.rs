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
        .map(|row| -> Result<StudentSummary, AppError> { map_row(row) })
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

// ── Student detail queries ────────────────────────────────────────────────────

use super::models::{StudentDemographics, StudentDetail, StudentProgram};

/// Fetch the full student record for a single student.
///
/// Returns None if the student_id does not exist in this tenant — the handler
/// maps this to 404. Three separate queries run in the same transaction:
///   1. Profile + identity + membership — 404 if not found
///   2. Current demographics (LEFT JOIN style via fetch_optional)
///   3. All current programs with program name
pub async fn get_student(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: uuid::Uuid,
) -> Result<Option<StudentDetail>, crate::error::AppError> {

    // ── 1. Core profile + identity + membership ───────────────────────────
    let profile_row = sqlx::query(
        r#"
        SELECT
            u.id                            AS user_id,
            u.username                      AS username,
            u.first_name                    AS first_name,
            u.middle_name                   AS middle_name,
            u.last_name                     AS last_name,
            u.preferred_name                AS preferred_name,
            u.last_name_suffix              AS last_name_suffix,
            tm.system_role::text            AS system_role,
            tm.joined_at                    AS joined_at,
            tm.institutional_email          AS institutional_email,
            sp.enrollment_year              AS enrollment_year,
            sp.expected_graduation_year     AS expected_graduation_year,
            sp.cumulative_gpa::float8       AS cumulative_gpa,
            sp.term_gpa::float8             AS term_gpa,
            sp.gpa_last_calculated_at       AS gpa_last_calculated_at,
            sp.academic_standing_status::text AS academic_standing,
            sp.cumulative_credits_attempted::float8 AS cumulative_credits_attempted,
            sp.cumulative_credits_earned::float8    AS cumulative_credits_earned,
            sp.current_timeframe_pct::float8 AS current_timeframe_pct,
            sp.is_nsc_opted_out             AS is_nsc_opted_out
        FROM sis.student_profiles sp
        JOIN core.users u            ON u.id  = sp.user_id
        JOIN core.tenant_memberships tm
             ON tm.user_id   = sp.user_id
            AND tm.tenant_id = sp.tenant_id
        WHERE sp.user_id   = $1
          AND sp.tenant_id = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(student_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(crate::error::AppError::from)?;

    let profile_row = match profile_row {
        None    => return Ok(None),
        Some(r) => r,
    };

    // ── 2. Current demographics (optional — may not exist) ────────────────
    let demo_row = sqlx::query(
        r#"
        SELECT
            date_of_birth,
            legal_sex,
            gender_identity,
            hispanic_or_latino,
            race_categories,
            primary_language,
            requires_iep_or_504,
            housing_status::text    AS housing_status,
            first_generation_student
        FROM sis.student_demographics
        WHERE student_id = $1
          AND tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND is_current = true
        "#,
    )
    .bind(student_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(crate::error::AppError::from)?;

    let demographics = demo_row.map(|r| {
        StudentDemographics {
            date_of_birth:           r.try_get("date_of_birth").unwrap_or_default(),
            legal_sex:               r.try_get("legal_sex").unwrap_or_default(),
            gender_identity:         r.try_get("gender_identity").ok().flatten(),
            hispanic_or_latino:      r.try_get("hispanic_or_latino").ok().flatten(),
            race_categories:         r.try_get("race_categories").ok().flatten(),
            primary_language:        r.try_get("primary_language").ok().flatten(),
            requires_iep_or_504:     r.try_get("requires_iep_or_504").ok().flatten(),
            housing_status:          r.try_get("housing_status").ok().flatten(),
            first_generation_student: r.try_get("first_generation_student").ok().flatten(),
        }
    });

    // ── 3. Current programs ───────────────────────────────────────────────
    let programs = get_student_programs(tx, student_id).await?;

    // ── Assemble detail ───────────────────────────────────────────────────
    let detail = StudentDetail {
        user_id:          profile_row.try_get("user_id").map_err(crate::error::AppError::from)?,
        username:         profile_row.try_get("username").map_err(crate::error::AppError::from)?,
        first_name:       profile_row.try_get("first_name").map_err(crate::error::AppError::from)?,
        middle_name:      profile_row.try_get("middle_name").map_err(crate::error::AppError::from)?,
        last_name:        profile_row.try_get("last_name").map_err(crate::error::AppError::from)?,
        preferred_name:   profile_row.try_get("preferred_name").map_err(crate::error::AppError::from)?,
        last_name_suffix: profile_row.try_get("last_name_suffix").map_err(crate::error::AppError::from)?,
        system_role:      profile_row.try_get("system_role").map_err(crate::error::AppError::from)?,
        joined_at:        profile_row.try_get("joined_at").map_err(crate::error::AppError::from)?,
        institutional_email: profile_row.try_get("institutional_email").map_err(crate::error::AppError::from)?,
        enrollment_year:              profile_row.try_get("enrollment_year").map_err(crate::error::AppError::from)?,
        expected_graduation_year:     profile_row.try_get("expected_graduation_year").map_err(crate::error::AppError::from)?,
        cumulative_gpa:               profile_row.try_get("cumulative_gpa").map_err(crate::error::AppError::from)?,
        term_gpa:                     profile_row.try_get("term_gpa").map_err(crate::error::AppError::from)?,
        gpa_last_calculated_at:       profile_row.try_get("gpa_last_calculated_at").map_err(crate::error::AppError::from)?,
        academic_standing:            profile_row.try_get("academic_standing").map_err(crate::error::AppError::from)?,
        cumulative_credits_attempted: profile_row.try_get("cumulative_credits_attempted").map_err(crate::error::AppError::from)?,
        cumulative_credits_earned:    profile_row.try_get("cumulative_credits_earned").map_err(crate::error::AppError::from)?,
        current_timeframe_pct:        profile_row.try_get("current_timeframe_pct").map_err(crate::error::AppError::from)?,
        is_nsc_opted_out:             profile_row.try_get("is_nsc_opted_out").map_err(crate::error::AppError::from)?,
        demographics,
        programs,
    };

    Ok(Some(detail))
}

async fn get_student_programs(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: uuid::Uuid,
) -> Result<Vec<StudentProgram>, crate::error::AppError> {
    let rows = sqlx::query(
        r#"
        SELECT
            sp.id                   AS id,
            sp.program_id           AS program_id,
            ap.name                 AS program_name,
            sp.is_primary           AS is_primary,
            sp.priority             AS priority,
            sp.status::text         AS status,
            sp.declared_on          AS declared_on
        FROM sis.student_programs sp
        JOIN sis.academic_programs ap ON ap.id = sp.program_id
        WHERE sp.student_id = $1
          AND sp.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND sp.is_current = true
        ORDER BY sp.is_primary DESC, sp.priority ASC
        "#,
    )
    .bind(student_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(crate::error::AppError::from)?;

    rows.iter().map(|r| {
        Ok(StudentProgram {
            id:           r.try_get("id").map_err(crate::error::AppError::from)?,
            program_id:   r.try_get("program_id").map_err(crate::error::AppError::from)?,
            program_name: r.try_get("program_name").map_err(crate::error::AppError::from)?,
            is_primary:   r.try_get("is_primary").map_err(crate::error::AppError::from)?,
            priority:     r.try_get("priority").map_err(crate::error::AppError::from)?,
            status:       r.try_get("status").map_err(crate::error::AppError::from)?,
            declared_on:  r.try_get("declared_on").map_err(crate::error::AppError::from)?,
        })
    }).collect()
}


// ── Course queries ────────────────────────────────────────────────────────────

use super::models::{
    CourseDetail, CourseSummary, ListCoursesParams,
    StudentEnrollment,
};

pub async fn list_courses(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListCoursesParams,
) -> Result<(Vec<CourseSummary>, i64), crate::error::AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM sis.courses c
        WHERE c.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND c.is_current = true
          AND ($1::uuid IS NULL OR c.department_id = $1)
          AND ($2::text IS NULL OR c.subject = $2)
          AND ($3::bool IS NULL OR c.is_active = $3)
        "#,
        params.department_id as Option<uuid::Uuid>,
        params.subject.clone() as Option<String>,
        params.is_active as Option<bool>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(crate::error::AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            c.id                        AS id,
            c.subject                   AS subject,
            c.course                    AS course,
            c.title                     AS title,
            c.credits::float8           AS credits,
            c.is_active                 AS is_active,
            c.grading_basis::text       AS grading_basis,
            c.course_level              AS course_level,
            d.name                      AS department_name
        FROM sis.courses c
        LEFT JOIN core.departments d ON d.id = c.department_id
        WHERE c.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND c.is_current = true
          AND ($1::uuid IS NULL OR c.department_id = $1)
          AND ($2::text IS NULL OR c.subject = $2)
          AND ($3::bool IS NULL OR c.is_active = $3)
        ORDER BY c.subject ASC, c.course ASC
        LIMIT  $4
        OFFSET $5
        "#,
    )
    .bind(params.department_id)
    .bind(params.subject.clone())
    .bind(params.is_active)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(crate::error::AppError::from)?;

    let courses = rows.iter().map(|r| -> Result<CourseSummary, crate::error::AppError> {
        Ok(CourseSummary {
            id:              r.try_get("id").map_err(crate::error::AppError::from)?,
            subject:         r.try_get("subject").map_err(crate::error::AppError::from)?,
            course:          r.try_get("course").map_err(crate::error::AppError::from)?,
            title:           r.try_get("title").map_err(crate::error::AppError::from)?,
            credits:         r.try_get("credits").map_err(crate::error::AppError::from)?,
            is_active:       r.try_get("is_active").map_err(crate::error::AppError::from)?,
            grading_basis:   r.try_get("grading_basis").map_err(crate::error::AppError::from)?,
            course_level:    r.try_get("course_level").map_err(crate::error::AppError::from)?,
            department_name: r.try_get("department_name").map_err(crate::error::AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((courses, total))
}

pub async fn get_course(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    course_id: uuid::Uuid,
) -> Result<Option<CourseDetail>, crate::error::AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            c.id                                AS id,
            c.subject                           AS subject,
            c.course                            AS course,
            c.title                             AS title,
            c.description                       AS description,
            c.credits::float8                   AS credits,
            c.is_active                         AS is_active,
            c.department_id                     AS department_id,
            d.name                              AS department_name,
            c.grading_basis::text               AS grading_basis,
            c.course_level                      AS course_level,
            c.lecture_hours::float8             AS lecture_hours,
            c.lab_hours::float8                 AS lab_hours,
            c.clinical_hours::float8            AS clinical_hours,
            c.independent_study_hours::float8   AS independent_study_hours,
            c.total_contact_hours::float8       AS total_contact_hours,
            c.is_repeatable                     AS is_repeatable,
            c.max_repeat_attempts               AS max_repeat_attempts,
            c.effective_start_date              AS effective_start_date,
            c.catalog_year                      AS catalog_year
        FROM sis.courses c
        LEFT JOIN core.departments d ON d.id = c.department_id
        WHERE c.id        = $1
          AND c.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND c.is_current = true
        "#,
    )
    .bind(course_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(crate::error::AppError::from)?;

    Ok(row.map(|r| CourseDetail {
        id:                      r.try_get("id").unwrap_or_default(),
        subject:                 r.try_get("subject").unwrap_or_default(),
        course:                  r.try_get("course").unwrap_or_default(),
        title:                   r.try_get("title").unwrap_or_default(),
        description:             r.try_get("description").ok().flatten(),
        credits:                 r.try_get("credits").unwrap_or_default(),
        is_active:               r.try_get("is_active").ok().flatten(),
        department_id:           r.try_get("department_id").unwrap_or_default(),
        department_name:         r.try_get("department_name").ok().flatten(),
        grading_basis:           r.try_get("grading_basis").unwrap_or_default(),
        course_level:            r.try_get("course_level").ok().flatten(),
        lecture_hours:           r.try_get("lecture_hours").unwrap_or_default(),
        lab_hours:               r.try_get("lab_hours").unwrap_or_default(),
        clinical_hours:          r.try_get("clinical_hours").unwrap_or_default(),
        independent_study_hours: r.try_get("independent_study_hours").unwrap_or_default(),
        total_contact_hours:     r.try_get("total_contact_hours").ok().flatten(),
        is_repeatable:           r.try_get("is_repeatable").unwrap_or(true),
        max_repeat_attempts:     r.try_get("max_repeat_attempts").ok().flatten(),
        effective_start_date:    r.try_get("effective_start_date").unwrap_or_default(),
        catalog_year:            r.try_get("catalog_year").ok().flatten(),
    }))
}

// ── Student enrollments ───────────────────────────────────────────────────────

pub async fn get_student_enrollments(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student_id: uuid::Uuid,
) -> Result<Vec<StudentEnrollment>, crate::error::AppError> {

    // First confirm student exists in this tenant (404 guard)
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
    .map_err(crate::error::AppError::from)?;

    if !exists {
        return Err(crate::error::AppError::NotFound(
            format!("Student {} not found", student_id)
        ));
    }

    let rows = sqlx::query(
        r#"
        SELECT
            e.id                            AS id,
            e.section_id                    AS section_id,
            e.status::text                  AS status,
            e.enrolled_at                   AS enrolled_at,
            e.credit_hours_enrolled::float8 AS credit_hours_enrolled,
            c.subject                       AS course_subject,
            c.course                        AS course_number,
            c.title                         AS course_title,
            t.name                          AS term_name,
            t.id                            AS term_id,
            cs.section_number               AS section_number,
            u.first_name || ' ' || u.last_name AS instructor_name
        FROM sis.enrollments e
        JOIN sis.course_sections cs  ON cs.id = e.section_id
        JOIN sis.courses c           ON c.id  = cs.course_id
        JOIN sis.academic_terms t    ON t.id  = cs.term_id
        JOIN core.users u            ON u.id  = cs.instructor_id
        WHERE e.student_id = $1
          AND e.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY t.start_date DESC, c.subject ASC, c.course ASC
        "#,
    )
    .bind(student_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(crate::error::AppError::from)?;

    rows.iter().map(|r| -> Result<StudentEnrollment, crate::error::AppError> {
        Ok(StudentEnrollment {
            id:                   r.try_get("id").map_err(crate::error::AppError::from)?,
            section_id:           r.try_get("section_id").map_err(crate::error::AppError::from)?,
            status:               r.try_get("status").map_err(crate::error::AppError::from)?,
            enrolled_at:          r.try_get("enrolled_at").map_err(crate::error::AppError::from)?,
            credit_hours_enrolled: r.try_get("credit_hours_enrolled").map_err(crate::error::AppError::from)?,
            course_subject:       r.try_get("course_subject").map_err(crate::error::AppError::from)?,
            course_number:        r.try_get("course_number").map_err(crate::error::AppError::from)?,
            course_title:         r.try_get("course_title").map_err(crate::error::AppError::from)?,
            term_name:            r.try_get("term_name").map_err(crate::error::AppError::from)?,
            term_id:              r.try_get("term_id").map_err(crate::error::AppError::from)?,
            section_number:       r.try_get("section_number").map_err(crate::error::AppError::from)?,
            instructor_name:      r.try_get("instructor_name").ok(),
        })
    }).collect()
}
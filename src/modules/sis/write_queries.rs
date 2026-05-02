// src/modules/sis/write_queries.rs
//
// All SIS write operations. Each function receives an open transaction and
// performs its work within it — the caller (handler) is responsible for
// committing or rolling back.
//
// NEW additions in this revision:
//   • create_enrollment   — POST /sis/students/:id/enrollments
//   • delete_enrollment   — DELETE /sis/students/:id/enrollments/:enrollment_id
//   • cross_module_enroll — POST /sis/enrollments  (SIS + LMS in one tx)

use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher};
use uuid::Uuid;

use crate::error::AppError;
use super::write_models::{
    CreateStudentRequest, CrossModuleEnrollRequest, CrossModuleEnrollResponse,
    CreateEnrollmentRequest, EnrollmentResponse, MaybePatch, UpdateStudentRequest,
};

// ── POST /sis/students ────────────────────────────────────────────────────────

/// Creates a new student atomically in a single transaction:
///   1. core.users            — global identity record
///   2. core.tenant_memberships — ties user to this tenant as 'student'
///   3. sis.student_profiles  — academic profile
pub async fn create_student(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    req:       &CreateStudentRequest,
) -> Result<Uuid, AppError> {

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("password hashing failed: {e}")))?
        .to_string();

    let user_id: Uuid = sqlx::query_scalar!(
        r#"
        INSERT INTO core.users
            (username, password_hash, first_name, middle_name,
             last_name, preferred_name, last_name_suffix, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        RETURNING id
        "#,
        req.username.trim(),
        hash,
        req.first_name.trim(),
        req.middle_name.as_deref().map(str::trim),
        req.last_name.trim(),
        req.preferred_name.as_deref().map(str::trim),
        req.last_name_suffix.as_deref().map(str::trim),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    sqlx::query(
        r#"
        INSERT INTO core.tenant_memberships
            (tenant_id, user_id, system_role, institutional_email)
        VALUES ($1, $2, 'student'::core.system_role, $3)
        "#,
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(req.institutional_email.as_deref())
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    sqlx::query!(
        r#"
        INSERT INTO sis.student_profiles
            (user_id, tenant_id, enrollment_year, expected_graduation_year)
        VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        tenant_id,
        req.enrollment_year,
        req.expected_graduation_year,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(user_id)
}

// ── PATCH /sis/students/:id ───────────────────────────────────────────────────

pub async fn update_student(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    student_id: Uuid,
    req:       &UpdateStudentRequest,
) -> Result<(), AppError> {

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
        return Err(AppError::NotFound(format!("Student {} not found", student_id)));
    }

    let users_changed =
        !matches!(req.first_name,       MaybePatch::Absent) ||
        !matches!(req.middle_name,      MaybePatch::Absent) ||
        !matches!(req.last_name,        MaybePatch::Absent) ||
        !matches!(req.preferred_name,   MaybePatch::Absent) ||
        !matches!(req.last_name_suffix, MaybePatch::Absent);

    if users_changed {
        let current = sqlx::query!(
            r#"SELECT first_name, middle_name, last_name,
                      preferred_name, last_name_suffix
               FROM core.users WHERE id = $1"#,
            student_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        let first_name = match &req.first_name {
            MaybePatch::Value(v) => v.trim().to_string(),
            _ => current.first_name,
        };
        let middle_name = match &req.middle_name {
            MaybePatch::Value(v) => Some(v.trim().to_string()),
            MaybePatch::Null     => None,
            MaybePatch::Absent   => current.middle_name,
        };
        let last_name = match &req.last_name {
            MaybePatch::Value(v) => v.trim().to_string(),
            _ => current.last_name,
        };
        let preferred_name = match &req.preferred_name {
            MaybePatch::Value(v) => Some(v.trim().to_string()),
            MaybePatch::Null     => None,
            MaybePatch::Absent   => current.preferred_name,
        };
        let last_name_suffix = match &req.last_name_suffix {
            MaybePatch::Value(v) => Some(v.trim().to_string()),
            MaybePatch::Null     => None,
            MaybePatch::Absent   => current.last_name_suffix,
        };

        sqlx::query!(
            r#"
            UPDATE core.users
               SET first_name        = $2,
                   middle_name       = $3,
                   last_name         = $4,
                   preferred_name    = $5,
                   last_name_suffix  = $6,
                   updated_at        = now()
             WHERE id = $1
            "#,
            student_id,
            first_name,
            middle_name,
            last_name,
            preferred_name,
            last_name_suffix,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    let profile_changed =
        !matches!(req.institutional_email,      MaybePatch::Absent) ||
        !matches!(req.expected_graduation_year, MaybePatch::Absent) ||
        !matches!(req.academic_standing_status, MaybePatch::Absent);

    if profile_changed {
        if let MaybePatch::Value(ref email) = req.institutional_email {
            sqlx::query!(
                r#"
                UPDATE core.tenant_memberships
                   SET institutional_email = $3
                 WHERE user_id   = $1
                   AND tenant_id = $2
                "#,
                student_id,
                tenant_id,
                email.trim(),
            )
            .execute(&mut **tx)
            .await
            .map_err(AppError::from)?;
        }

        let current_profile = sqlx::query!(
            r#"SELECT expected_graduation_year, academic_standing_status::text AS standing
               FROM sis.student_profiles
               WHERE user_id = $1 AND tenant_id = $2"#,
            student_id,
            tenant_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        let grad_year = match &req.expected_graduation_year {
            MaybePatch::Value(v) => Some(*v),
            MaybePatch::Null     => None,
            MaybePatch::Absent   => current_profile.expected_graduation_year,
        };
        let standing = match &req.academic_standing_status {
            MaybePatch::Value(v) => v.clone(),
            _ => current_profile.standing.unwrap_or_else(|| "good_standing".to_string()),
        };

        sqlx::query(
            r#"
            UPDATE sis.student_profiles
               SET expected_graduation_year  = $3,
                   academic_standing_status  = $4::sis.academic_standing_status,
                   updated_at               = now()
             WHERE user_id   = $1
               AND tenant_id = $2
            "#,
        )
        .bind(student_id)
        .bind(tenant_id)
        .bind(grad_year)
        .bind(&standing)
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// POST /sis/students/:id/enrollments
// ═══════════════════════════════════════════════════════════════════════════════

/// Enroll a student into a section (SIS side only — no LMS side-effects).
///
/// Business rules enforced:
///   1. Student must exist in this tenant.
///   2. Section must exist, be current, and be published.
///   3. No duplicate active enrollment (tenant_student_section_unique).
///   4. Capacity check: if enrolled count >= max_enrollment, auto-promote
///      to 'waitlisted' unless the caller explicitly passed status=waitlisted.
///      Returns AppError::Conflict if both enrolled and waitlist are full.
pub async fn create_enrollment(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    student_id: Uuid,
    req:        &CreateEnrollmentRequest,
) -> Result<EnrollmentResponse, AppError> {

    // ── 1. Guard: student exists in tenant ───────────────────────────────
    let student_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.student_profiles
            WHERE user_id = $1 AND tenant_id = $2
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

    // ── 2. Guard: section exists, is current, and is published ───────────
    let section = sqlx::query!(
        r#"
        SELECT id, max_enrollment, waitlist_capacity, is_published
        FROM sis.course_sections
        WHERE id        = $1
          AND tenant_id = $2
          AND is_current = true
        "#,
        req.section_id,
        tenant_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Section {} not found", req.section_id)))?;

    if !section.is_published {
        return Err(AppError::BadRequest(
            format!("Section {} is not published and cannot accept enrollments", req.section_id)
        ));
    }

    // ── 3. Capacity check ────────────────────────────────────────────────
    let enrolled_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM sis.enrollments
        WHERE section_id = $1
          AND tenant_id  = $2
          AND status     = 'enrolled'
        "#,
        req.section_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let waitlist_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM sis.enrollments
        WHERE section_id = $1
          AND tenant_id  = $2
          AND status     = 'waitlisted'
        "#,
        req.section_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let requested_status = req.status.as_deref().unwrap_or("enrolled");

    let resolved_status: &str = if requested_status == "waitlisted" {
        // Caller explicitly requested waitlist — honour it.
        if waitlist_count >= section.waitlist_capacity as i64 {
            return Err(AppError::Conflict(
                "Section waitlist is full".to_string()
            ));
        }
        "waitlisted"
    } else {
        // Caller wants enrolled; auto-demote to waitlist if section is full.
        if enrolled_count >= section.max_enrollment as i64 {
            if waitlist_count >= section.waitlist_capacity as i64 {
                return Err(AppError::Conflict(
                    "Section is full and the waitlist is full".to_string()
                ));
            }
            "waitlisted"
        } else {
            "enrolled"
        }
    };

    // ── 4. Insert enrollment ─────────────────────────────────────────────
    // UNTYPED — $4 binds to sis.enrollment_status (custom PG enum).
    // sqlx::query! cannot bind &str to a custom enum at compile time.
    // The unique constraint (tenant_student_section_unique) fires 23505
    // on duplicate → AppError::Conflict automatically.
    use sqlx::Row as _;
    let row = sqlx::query(
        r#"
        INSERT INTO sis.enrollments
            (tenant_id, student_id, section_id, status,
             enrollment_effective_date, credit_hours_enrolled,
             is_title_iv_eligible, fulfilled_catalog_requirement_id)
        VALUES
            ($1, $2, $3, $4::sis.enrollment_status,
             $5, $6, $7, $8)
        RETURNING
            id,
            status::text                AS status,
            enrolled_at,
            enrollment_effective_date,
            credit_hours_enrolled::float8 AS credit_hours_enrolled,
            is_title_iv_eligible
        "#,
    )
    .bind(tenant_id)
    .bind(student_id)
    .bind(req.section_id)
    .bind(resolved_status)
    .bind(req.enrollment_effective_date)
    .bind(req.credit_hours_enrolled)
    .bind(req.is_title_iv_eligible.unwrap_or(true))
    .bind(req.fulfilled_catalog_requirement_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(EnrollmentResponse {
        enrollment_id:             row.try_get("id").map_err(AppError::from)?,
        student_id,
        section_id:                req.section_id,
        status:                    row.try_get("status").map_err(AppError::from)?,
        enrolled_at:               row.try_get("enrolled_at").map_err(AppError::from)?,
        enrollment_effective_date: row.try_get("enrollment_effective_date").map_err(AppError::from)?,
        credit_hours_enrolled:     row.try_get("credit_hours_enrolled").map_err(AppError::from)?,
        is_title_iv_eligible:      row.try_get("is_title_iv_eligible").map_err(AppError::from)?,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// DELETE /sis/students/:id/enrollments/:enrollment_id
// ═══════════════════════════════════════════════════════════════════════════════

/// Soft-delete an enrollment by transitioning its status to 'dropped'.
///
/// Hard deletes are never performed — the enrollment row is retained for audit
/// and financial aid history. Only enrollments currently in status 'enrolled'
/// or 'waitlisted' may be dropped through this endpoint. Attempting to drop
/// an already-dropped, withdrawn, or prereq_drop record returns 409 Conflict.
///
/// Returns the updated enrollment_id on success.
pub async fn drop_enrollment(
    tx:            &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:     Uuid,
    student_id:    Uuid,
    enrollment_id: Uuid,
) -> Result<Uuid, AppError> {

    // Fetch the enrollment, confirming it belongs to this student + tenant.
    let row = sqlx::query!(
        r#"
        SELECT id, status::text AS "status!"
        FROM sis.enrollments
        WHERE id         = $1
          AND student_id = $2
          AND tenant_id  = $3
        "#,
        enrollment_id,
        student_id,
        tenant_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("Enrollment {} not found for student {}", enrollment_id, student_id)
    ))?;

    match row.status.as_str() {
        "enrolled" | "waitlisted" => {}
        other => {
            return Err(AppError::Conflict(format!(
                "Cannot drop enrollment with status '{}'; only 'enrolled' or 'waitlisted' enrollments can be dropped",
                other
            )));
        }
    }

    sqlx::query!(
        r#"
        UPDATE sis.enrollments
           SET status            = 'dropped'::sis.enrollment_status,
               status_updated_at = now(),
               updated_at        = now()
         WHERE id         = $1
           AND tenant_id  = $2
        "#,
        enrollment_id,
        tenant_id,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(enrollment_id)
}

// ═══════════════════════════════════════════════════════════════════════════════
// POST /sis/enrollments  (cross-module SIS + LMS)
// ═══════════════════════════════════════════════════════════════════════════════

/// Atomic cross-module enrollment.
///
/// This is the realistic registrar workflow operation. A single database
/// transaction touches two schemas:
///
///   sis.enrollments              — authoritative academic record
///   lms.section_grade_summaries  — LMS gradebook provisioning
///
/// Business rules are identical to create_enrollment above, with the addition
/// of the LMS INSERT. If either INSERT fails the entire transaction rolls back.
///
/// Design note on LMS coupling:
///   The LMS grade summary row must exist before any grade can be computed.
///   Provisioning it here (not lazily) means the gradebook is ready the
///   instant the student is enrolled — no eventual-consistency gap.
pub async fn cross_module_enroll(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    req:       &CrossModuleEnrollRequest,
) -> Result<CrossModuleEnrollResponse, AppError> {

    // ── 1. Guard: student exists in tenant ───────────────────────────────
    let student_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.student_profiles
            WHERE user_id = $1 AND tenant_id = $2
        ) AS "exists!"
        "#,
        req.student_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !student_exists {
        return Err(AppError::NotFound(format!("Student {} not found", req.student_id)));
    }

    // ── 2. Guard: section exists, is current, and is published ───────────
    let section = sqlx::query!(
        r#"
        SELECT id, max_enrollment, waitlist_capacity, is_published
        FROM sis.course_sections
        WHERE id        = $1
          AND tenant_id = $2
          AND is_current = true
        "#,
        req.section_id,
        tenant_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Section {} not found", req.section_id)))?;

    if !section.is_published {
        return Err(AppError::BadRequest(
            format!("Section {} is not published", req.section_id)
        ));
    }

    // ── 3. Capacity check ────────────────────────────────────────────────
    let enrolled_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM sis.enrollments
        WHERE section_id = $1
          AND tenant_id  = $2
          AND status     = 'enrolled'
        "#,
        req.section_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let waitlist_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM sis.enrollments
        WHERE section_id = $1
          AND tenant_id  = $2
          AND status     = 'waitlisted'
        "#,
        req.section_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let requested_status = req.status.as_deref().unwrap_or("enrolled");

    let resolved_status: &str = if requested_status == "waitlisted" {
        if waitlist_count >= section.waitlist_capacity as i64 {
            return Err(AppError::Conflict("Section waitlist is full".to_string()));
        }
        "waitlisted"
    } else {
        if enrolled_count >= section.max_enrollment as i64 {
            if waitlist_count >= section.waitlist_capacity as i64 {
                return Err(AppError::Conflict(
                    "Section is full and the waitlist is full".to_string()
                ));
            }
            "waitlisted"
        } else {
            "enrolled"
        }
    };

    // ── 4. INSERT sis.enrollments ────────────────────────────────────────
    // UNTYPED — $4 binds to sis.enrollment_status (custom PG enum).
    use sqlx::Row as _;
    let enrollment = sqlx::query(
        r#"
        INSERT INTO sis.enrollments
            (tenant_id, student_id, section_id, status,
             enrollment_effective_date, credit_hours_enrolled,
             is_title_iv_eligible, fulfilled_catalog_requirement_id)
        VALUES
            ($1, $2, $3, $4::sis.enrollment_status,
             $5, $6, $7, $8)
        RETURNING
            id,
            status::text                  AS status,
            enrolled_at,
            enrollment_effective_date,
            credit_hours_enrolled::float8 AS credit_hours_enrolled,
            is_title_iv_eligible
        "#,
    )
    .bind(tenant_id)
    .bind(req.student_id)
    .bind(req.section_id)
    .bind(resolved_status)
    .bind(req.enrollment_effective_date)
    .bind(req.credit_hours_enrolled)
    .bind(req.is_title_iv_eligible.unwrap_or(true))
    .bind(req.fulfilled_catalog_requirement_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let enrollment_id:             Uuid    = enrollment.try_get("id").map_err(AppError::from)?;
    let enrollment_status:         String  = enrollment.try_get("status").map_err(AppError::from)?;
    let enrolled_at:               chrono::DateTime<chrono::Utc> = enrollment.try_get("enrolled_at").map_err(AppError::from)?;
    let enrollment_effective_date: Option<chrono::NaiveDate>     = enrollment.try_get("enrollment_effective_date").map_err(AppError::from)?;
    let credit_hours_enrolled:     Option<f64>                   = enrollment.try_get("credit_hours_enrolled").map_err(AppError::from)?;
    let is_title_iv_eligible:      bool    = enrollment.try_get("is_title_iv_eligible").map_err(AppError::from)?;

    // ── 5. INSERT lms.section_grade_summaries ────────────────────────────
    // Only provision the LMS row for actively enrolled students; waitlisted
    // students are not yet in the gradebook.
    let lms_summary_id: Uuid = if resolved_status == "enrolled" {
        sqlx::query_scalar!(
            r#"
            INSERT INTO lms.section_grade_summaries
                (tenant_id, enrollment_id, section_id, student_id,
                 graded_assignment_count, total_assignment_count,
                 points_earned, points_possible)
            VALUES ($1, $2, $3, $4, 0, 0, 0, 0)
            RETURNING id
            "#,
            tenant_id,
            enrollment_id,
            req.section_id,
            req.student_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?
    } else {
        // Placeholder row for waitlisted students — is_excused = true signals
        // the gradebook service to ignore it until the waitlist clears.
        sqlx::query_scalar!(
            r#"
            INSERT INTO lms.section_grade_summaries
                (tenant_id, enrollment_id, section_id, student_id,
                 graded_assignment_count, total_assignment_count,
                 points_earned, points_possible,
                 is_excused)
            VALUES ($1, $2, $3, $4, 0, 0, 0, 0, true)
            RETURNING id
            "#,
            tenant_id,
            enrollment_id,
            req.section_id,
            req.student_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?
    };

    Ok(CrossModuleEnrollResponse {
        enrollment_id,
        student_id:                req.student_id,
        section_id:                req.section_id,
        status:                    enrollment_status,
        enrolled_at,
        enrollment_effective_date,
        credit_hours_enrolled,
        is_title_iv_eligible,
        lms_grade_summary_id:      lms_summary_id,
    })
}
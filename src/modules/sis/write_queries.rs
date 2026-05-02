// src/modules/sis/write_queries.rs

use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher};
use uuid::Uuid;

use crate::error::AppError;
use super::write_models::{CreateStudentRequest, MaybePatch, UpdateStudentRequest};

// ── POST /sis/students ────────────────────────────────────────────────────────

/// Creates a new student atomically in a single transaction:
///   1. core.users           — global identity record
///   2. core.tenant_memberships — ties user to this tenant as 'student'
///   3. sis.student_profiles — academic profile
///
/// Returns the new user_id on success.
/// Errors:
///   - AppError::Conflict if username already exists (unique index violation)
///   - AppError::Internal if hashing fails
pub async fn create_student(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    req:        &CreateStudentRequest,
) -> Result<Uuid, AppError> {

    // ── Hash password ─────────────────────────────────────────────────────
    let salt   = SaltString::generate(&mut OsRng);
    let hash   = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("password hashing failed: {e}")))?
        .to_string();

    // ── 1. Insert core.users ──────────────────────────────────────────────
    // username has a global unique index — sqlx maps 23505 → AppError::Conflict
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

    // ── 2. Insert core.tenant_memberships ────────────────────────────────
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

    // ── 3. Insert sis.student_profiles ───────────────────────────────────
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

/// Partially updates a student's profile.
///
/// Only fields explicitly present in the request body are updated.
/// Runs three conditional UPDATE statements in the same transaction:
///   - core.users if any identity field changed
///   - core.tenant_memberships if institutional_email changed
///   - sis.student_profiles if any academic field changed
///
/// Returns AppError::NotFound if student_id doesn't exist in this tenant.
pub async fn update_student(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    student_id: Uuid,
    req:        &UpdateStudentRequest,
) -> Result<(), AppError> {

    // Guard: confirm student exists in this tenant before any UPDATE
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

    // ── Update core.users if any identity field is present ────────────────
    let users_changed =
        !matches!(req.first_name,       MaybePatch::Absent) ||
        !matches!(req.middle_name,      MaybePatch::Absent) ||
        !matches!(req.last_name,        MaybePatch::Absent) ||
        !matches!(req.preferred_name,   MaybePatch::Absent) ||
        !matches!(req.last_name_suffix, MaybePatch::Absent);

    if users_changed {
        // Read current values so we can COALESCE — avoids dynamic SQL
        let current = sqlx::query!(
            r#"
            SELECT first_name, middle_name, last_name,
                   preferred_name, last_name_suffix
            FROM core.users WHERE id = $1
            "#,
            student_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        let new_first = match &req.first_name {
            MaybePatch::Present(v) => v.trim().to_string(),
            MaybePatch::Absent     => current.first_name.clone(),
        };
        let new_middle = match &req.middle_name {
            MaybePatch::Present(v) => v.as_deref().map(str::trim).map(String::from),
            MaybePatch::Absent     => current.middle_name.clone(),
        };
        let new_last = match &req.last_name {
            MaybePatch::Present(v) => v.trim().to_string(),
            MaybePatch::Absent     => current.last_name.clone(),
        };
        let new_preferred = match &req.preferred_name {
            MaybePatch::Present(v) => v.as_deref().map(str::trim).map(String::from),
            MaybePatch::Absent     => current.preferred_name.clone(),
        };
        let new_suffix = match &req.last_name_suffix {
            MaybePatch::Present(v) => v.as_deref().map(str::trim).map(String::from),
            MaybePatch::Absent     => current.last_name_suffix.clone(),
        };

        sqlx::query!(
            r#"
            UPDATE core.users
            SET first_name       = $2,
                middle_name      = $3,
                last_name        = $4,
                preferred_name   = $5,
                last_name_suffix = $6,
                updated_at       = now()
            WHERE id = $1
            "#,
            student_id,
            new_first,
            new_middle,
            new_last,
            new_preferred,
            new_suffix,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    // ── Update core.tenant_memberships if institutional_email changed ─────
    if let MaybePatch::Present(ref email) = req.institutional_email {
        sqlx::query!(
            r#"
            UPDATE core.tenant_memberships
            SET institutional_email = $3,
                updated_at          = now()
            WHERE tenant_id = $1
              AND user_id   = $2
            "#,
            tenant_id,
            student_id,
            email.as_deref(),
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    // ── Update sis.student_profiles if any academic field changed ─────────
    let profile_changed =
        !matches!(req.enrollment_year,          MaybePatch::Absent) ||
        !matches!(req.expected_graduation_year, MaybePatch::Absent) ||
        !matches!(req.academic_standing_status, MaybePatch::Absent);

    if profile_changed {
        let current = sqlx::query!(
            r#"
            SELECT enrollment_year, expected_graduation_year,
                   academic_standing_status::text AS academic_standing_status
            FROM sis.student_profiles
            WHERE user_id   = $1
              AND tenant_id = $2
            "#,
            student_id,
            tenant_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        let new_enroll = match &req.enrollment_year {
            MaybePatch::Present(v) => *v,
            MaybePatch::Absent     => current.enrollment_year,
        };
        let new_grad = match &req.expected_graduation_year {
            MaybePatch::Present(v) => *v,
            MaybePatch::Absent     => current.expected_graduation_year,
        };
        let new_standing = match &req.academic_standing_status {
            MaybePatch::Present(v) => v.clone(),
            MaybePatch::Absent     => current.academic_standing_status,
        };

        sqlx::query(
            r#"
            UPDATE sis.student_profiles
            SET enrollment_year          = $3,
                expected_graduation_year = $4,
                academic_standing_status = $5::sis.academic_standing_status,
                updated_at               = now()
            WHERE user_id   = $1
              AND tenant_id = $2
            "#,
        )
        .bind(student_id)
        .bind(tenant_id)
        .bind(new_enroll)
        .bind(new_grad)
        .bind(new_standing)
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    Ok(())
}
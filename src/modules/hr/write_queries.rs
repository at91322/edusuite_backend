// src/modules/hr/write_queries.rs

use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher};
use uuid::Uuid;

use crate::error::AppError;
use super::write_models::{CreateStaffRequest, MaybePatch, UpdateStaffRequest};

// ── POST /hr/staff ────────────────────────────────────────────────────────────

pub async fn create_staff_member(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    req:       &CreateStaffRequest,
) -> Result<Uuid, AppError> {

    // ── Hash password ─────────────────────────────────────────────────────
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("password hashing failed: {e}")))?
        .to_string();

    // ── 1. core.users ─────────────────────────────────────────────────────
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

    // ── 2. core.tenant_memberships ────────────────────────────────────────
    sqlx::query(
        r#"
        INSERT INTO core.tenant_memberships
            (tenant_id, user_id, system_role, institutional_email)
        VALUES ($1, $2, $3::core.system_role, $4)
        "#,
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(&req.system_role)
    .bind(req.institutional_email.as_deref())
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    // ── 3. hr.staff_profiles ──────────────────────────────────────────────
    sqlx::query!(
        r#"
        INSERT INTO hr.staff_profiles
            (user_id, tenant_id, primary_department_id, hire_date, is_tenured)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        user_id,
        tenant_id,
        req.primary_department_id,
        req.hire_date,
        req.is_tenured,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    // ── 4. hr.employment_contracts (optional) ─────────────────────────────
    if let Some(ref c) = req.contract {
        sqlx::query(
            r#"
            INSERT INTO hr.employment_contracts
                (tenant_id, staff_id, type, start_date, end_date,
                 job_title, annual_salary, hourly_rate, is_active)
            VALUES ($1, $2, $3::hr.contract_type, $4, $5, $6, $7, $8, true)
            "#,
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(&c.contract_type)
        .bind(c.start_date)
        .bind(c.end_date)
        .bind(c.job_title.trim())
        .bind(c.annual_salary)
        .bind(c.hourly_rate)
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    Ok(user_id)
}

// ── PATCH /hr/staff/:id ───────────────────────────────────────────────────────

pub async fn update_staff_member(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    staff_id:  Uuid,
    req:       &UpdateStaffRequest,
) -> Result<(), AppError> {

    // Guard: confirm staff exists in this tenant
    let exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM hr.staff_profiles
            WHERE user_id   = $1
              AND tenant_id = $2
        ) AS "exists!"
        "#,
        staff_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(
            format!("Staff member {} not found", staff_id)
        ));
    }

    // ── Update core.users ─────────────────────────────────────────────────
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
            staff_id,
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
            staff_id,
            new_first, new_middle, new_last, new_preferred, new_suffix,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    // ── Update core.tenant_memberships ────────────────────────────────────
    if let MaybePatch::Present(ref email) = req.institutional_email {
        sqlx::query!(
            r#"
            UPDATE core.tenant_memberships
            SET institutional_email = $3, updated_at = now()
            WHERE tenant_id = $1 AND user_id = $2
            "#,
            tenant_id, staff_id, email.as_deref(),
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    // ── Update hr.staff_profiles ──────────────────────────────────────────
    let profile_changed =
        !matches!(req.primary_department_id, MaybePatch::Absent) ||
        !matches!(req.hire_date,             MaybePatch::Absent) ||
        !matches!(req.is_tenured,            MaybePatch::Absent);

    if profile_changed {
        let current = sqlx::query!(
            r#"SELECT primary_department_id, hire_date, is_tenured
               FROM hr.staff_profiles
               WHERE user_id = $1 AND tenant_id = $2"#,
            staff_id, tenant_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        let new_dept = match &req.primary_department_id {
            MaybePatch::Present(v) => *v,
            MaybePatch::Absent     => current.primary_department_id,
        };
        let new_hire = match &req.hire_date {
            MaybePatch::Present(v) => *v,
            MaybePatch::Absent     => current.hire_date,
        };
        let new_tenured = match &req.is_tenured {
            MaybePatch::Present(v) => *v,
            MaybePatch::Absent     => current.is_tenured,
        };

        sqlx::query!(
            r#"
            UPDATE hr.staff_profiles
            SET primary_department_id = $3,
                hire_date             = $4,
                is_tenured            = $5,
                updated_at            = now()
            WHERE user_id = $1 AND tenant_id = $2
            "#,
            staff_id, tenant_id, new_dept, new_hire, new_tenured,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    Ok(())
}
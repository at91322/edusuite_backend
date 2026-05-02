// src/modules/hr/write_queries.rs
//
// All HR write operations.
// Existing create_staff_member / update_staff_member preserved.
// create_contract — POST /hr/staff/:id/contracts

use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher};
use uuid::Uuid;

use crate::error::AppError;
use super::write_models::{
    ContractResponse, CreateContractRequest, CreateStaffRequest,
    MaybePatch, UpdateStaffRequest,
};

// ── POST /hr/staff ────────────────────────────────────────────────────────────

pub async fn create_staff_member(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    req:       &CreateStaffRequest,
) -> Result<Uuid, AppError> {

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
        VALUES ($1, $2, 'staff'::core.system_role, $3)
        "#,
    )
    .bind(tenant_id)
    .bind(user_id)
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
        return Err(AppError::NotFound(format!("Staff member {} not found", staff_id)));
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
               SET first_name       = $2,
                   middle_name      = $3,
                   last_name        = $4,
                   preferred_name   = $5,
                   last_name_suffix = $6,
                   updated_at       = now()
             WHERE id = $1
            "#,
            staff_id,
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

    // ── Update hr.staff_profiles ──────────────────────────────────────────
    let profile_changed =
        !matches!(req.institutional_email,   MaybePatch::Absent) ||
        !matches!(req.primary_department_id, MaybePatch::Absent) ||
        !matches!(req.is_tenured,            MaybePatch::Absent);

    if profile_changed {
        if let MaybePatch::Value(ref email) = req.institutional_email {
            sqlx::query!(
                r#"
                UPDATE core.tenant_memberships
                   SET institutional_email = $3
                 WHERE user_id   = $1
                   AND tenant_id = $2
                "#,
                staff_id,
                tenant_id,
                email.trim(),
            )
            .execute(&mut **tx)
            .await
            .map_err(AppError::from)?;
        }

        let current_profile = sqlx::query!(
            r#"SELECT primary_department_id, is_tenured
               FROM hr.staff_profiles
               WHERE user_id = $1 AND tenant_id = $2"#,
            staff_id,
            tenant_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        // primary_department_id is NOT NULL in hr.staff_profiles — the column
        // cannot be cleared, so Null is treated the same as Absent (no change).
        let dept_id: Uuid = match &req.primary_department_id {
            MaybePatch::Value(v) => *v,
            MaybePatch::Null | MaybePatch::Absent => current_profile.primary_department_id,
        };
        let is_tenured = match &req.is_tenured {
            MaybePatch::Value(v) => *v,
            _ => current_profile.is_tenured.unwrap_or(false),
        };

        sqlx::query!(
            r#"
            UPDATE hr.staff_profiles
               SET primary_department_id = $3,
                   is_tenured            = $4,
                   updated_at            = now()
             WHERE user_id   = $1
               AND tenant_id = $2
            "#,
            staff_id,
            tenant_id,
            dept_id,
            is_tenured,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// POST /hr/staff/:id/contracts
// ═══════════════════════════════════════════════════════════════════════════════

/// Add a new employment contract to an existing staff member.
///
/// Steps:
///   1. Confirm the staff member exists in this tenant (404 guard).
///   2. If `deactivate_existing = true`, soft-deactivate any contract where
///      is_active = true for this staff member. Historical contracts are never
///      deleted.
///   3. Insert the new contract with is_active = true.
///   4. Return the full contract record.
///
/// The caller (handler) owns the transaction and commits on success.
pub async fn create_contract(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    staff_id:  Uuid,
    req:       &CreateContractRequest,
) -> Result<ContractResponse, AppError> {

    // ── 1. Guard: staff exists in tenant ─────────────────────────────────
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

    // ── 2. Deactivate existing active contracts if requested ──────────────
    if req.deactivate_existing {
        let deactivated = sqlx::query_scalar!(
            r#"
            UPDATE hr.employment_contracts
               SET is_active  = false,
                   updated_at = now()
             WHERE staff_id   = $1
               AND tenant_id  = $2
               AND is_active  = true
            RETURNING id
            "#,
            staff_id,
            tenant_id,
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(AppError::from)?;

        if !deactivated.is_empty() {
            tracing::info!(
                tenant_id = %tenant_id,
                staff_id  = %staff_id,
                count     = deactivated.len(),
                "Deactivated existing contracts before inserting new one"
            );
        }
    }

    // ── 3. Insert new contract ────────────────────────────────────────────
    // UNTYPED (sqlx::query, not sqlx::query!) because $3 binds to the custom
    // hr.contract_type PG enum. sqlx::query! cannot bind &str to a custom enum
    // at compile time. The explicit ::hr.contract_type cast lets Postgres
    // validate the value; an invalid string returns a DB error → AppError::Internal.
    use sqlx::Row as _;
    let row = sqlx::query(
        r#"
        INSERT INTO hr.employment_contracts
            (tenant_id, staff_id, type, start_date, end_date,
             job_title, annual_salary, hourly_rate, position_id, is_active)
        VALUES
            ($1, $2, $3::hr.contract_type, $4, $5,
             $6, $7, $8, $9, true)
        RETURNING
            id,
            type::text  AS contract_type,
            start_date,
            end_date,
            job_title,
            annual_salary::float8  AS annual_salary,
            hourly_rate::float8    AS hourly_rate,
            position_id,
            is_active,
            created_at
        "#,
    )
    .bind(tenant_id)
    .bind(staff_id)
    .bind(&req.contract_type)
    .bind(req.start_date)
    .bind(req.end_date)
    .bind(req.job_title.trim())
    .bind(req.annual_salary)
    .bind(req.hourly_rate)
    .bind(req.position_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(ContractResponse {
        contract_id:   row.try_get("id").map_err(AppError::from)?,
        staff_id,
        contract_type: row.try_get("contract_type").map_err(AppError::from)?,
        start_date:    row.try_get("start_date").map_err(AppError::from)?,
        end_date:      row.try_get("end_date").map_err(AppError::from)?,
        job_title:     row.try_get("job_title").map_err(AppError::from)?,
        annual_salary: row.try_get("annual_salary").map_err(AppError::from)?,
        hourly_rate:   row.try_get("hourly_rate").map_err(AppError::from)?,
        position_id:   row.try_get("position_id").map_err(AppError::from)?,
        is_active:     row.try_get("is_active").map_err(AppError::from)?,
        created_at:    row.try_get("created_at").map_err(AppError::from)?,
    })
}
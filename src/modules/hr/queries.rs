// src/modules/hr/queries.rs

use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use super::models::{ListStaffParams, StaffContract, StaffDetail, StaffSummary};

// ── List ──────────────────────────────────────────────────────────────────────

pub async fn list_staff(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListStaffParams,
) -> Result<(Vec<StaffSummary>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM hr.staff_profiles sp
        JOIN core.users u ON u.id = sp.user_id
        WHERE sp.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::uuid IS NULL OR sp.primary_department_id = $1)
          AND ($2::bool IS NULL OR sp.is_tenured = $2)
        "#,
        params.department_id as Option<Uuid>,
        params.is_tenured    as Option<bool>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            sp.user_id                      AS user_id,
            u.first_name                    AS first_name,
            u.last_name                     AS last_name,
            u.preferred_name                AS preferred_name,
            u.username                      AS username,
            sp.hire_date                    AS hire_date,
            sp.is_tenured                   AS is_tenured,
            tm.system_role::text            AS system_role,
            d.name                          AS department_name,
            ec.job_title                    AS job_title,
            ec.type::text                   AS contract_type
        FROM hr.staff_profiles sp
        JOIN core.users u            ON u.id  = sp.user_id
        JOIN core.tenant_memberships tm
               ON tm.user_id   = sp.user_id
              AND tm.tenant_id = sp.tenant_id
        LEFT JOIN core.departments d ON d.id = sp.primary_department_id
        LEFT JOIN hr.employment_contracts ec
               ON ec.staff_id  = sp.user_id
              AND ec.tenant_id = sp.tenant_id
              AND ec.is_active = true
        WHERE sp.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::uuid IS NULL OR sp.primary_department_id = $1)
          AND ($2::bool IS NULL OR sp.is_tenured = $2)
        ORDER BY u.last_name ASC, u.first_name ASC
        LIMIT  $3
        OFFSET $4
        "#,
    )
    .bind(params.department_id)
    .bind(params.is_tenured)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let staff = rows.iter().map(|r| -> Result<StaffSummary, AppError> {
        Ok(StaffSummary {
            user_id:         r.try_get("user_id").map_err(AppError::from)?,
            first_name:      r.try_get("first_name").map_err(AppError::from)?,
            last_name:       r.try_get("last_name").map_err(AppError::from)?,
            preferred_name:  r.try_get("preferred_name").map_err(AppError::from)?,
            username:        r.try_get("username").map_err(AppError::from)?,
            hire_date:       r.try_get("hire_date").map_err(AppError::from)?,
            is_tenured:      r.try_get("is_tenured").map_err(AppError::from)?,
            system_role:     r.try_get("system_role").map_err(AppError::from)?,
            department_name: r.try_get("department_name").map_err(AppError::from)?,
            job_title:       r.try_get("job_title").map_err(AppError::from)?,
            contract_type:   r.try_get("contract_type").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((staff, total))
}

// ── Detail ────────────────────────────────────────────────────────────────────

pub async fn get_staff_member(
    tx:       &mut sqlx::Transaction<'_, sqlx::Postgres>,
    staff_id: Uuid,
) -> Result<Option<StaffDetail>, AppError> {

    let row = sqlx::query(
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
            sp.hire_date                    AS hire_date,
            sp.is_tenured                   AS is_tenured,
            sp.primary_department_id        AS primary_department_id,
            d.name                          AS primary_department
        FROM hr.staff_profiles sp
        JOIN core.users u            ON u.id  = sp.user_id
        JOIN core.tenant_memberships tm
               ON tm.user_id   = sp.user_id
              AND tm.tenant_id = sp.tenant_id
        LEFT JOIN core.departments d ON d.id = sp.primary_department_id
        WHERE sp.user_id   = $1
          AND sp.tenant_id = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(staff_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let row = match row {
        None    => return Ok(None),
        Some(r) => r,
    };

    // Active contract (optional — staff may not have one yet)
    let contract_row = sqlx::query(
        r#"
        SELECT id, type::text AS contract_type, job_title, start_date, end_date
        FROM hr.employment_contracts
        WHERE staff_id  = $1
          AND tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND is_active = true
        ORDER BY start_date DESC
        LIMIT 1
        "#,
    )
    .bind(staff_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let active_contract = contract_row.map(|r| StaffContract {
        id:            r.try_get("id").unwrap_or_default(),
        contract_type: r.try_get("contract_type").unwrap_or_default(),
        job_title:     r.try_get("job_title").unwrap_or_default(),
        start_date:    r.try_get("start_date").unwrap_or_default(),
        end_date:      r.try_get("end_date").ok().flatten(),
    });

    Ok(Some(StaffDetail {
        user_id:               row.try_get("user_id").map_err(AppError::from)?,
        username:              row.try_get("username").map_err(AppError::from)?,
        first_name:            row.try_get("first_name").map_err(AppError::from)?,
        middle_name:           row.try_get("middle_name").map_err(AppError::from)?,
        last_name:             row.try_get("last_name").map_err(AppError::from)?,
        preferred_name:        row.try_get("preferred_name").map_err(AppError::from)?,
        last_name_suffix:      row.try_get("last_name_suffix").map_err(AppError::from)?,
        system_role:           row.try_get("system_role").map_err(AppError::from)?,
        joined_at:             row.try_get("joined_at").map_err(AppError::from)?,
        institutional_email:   row.try_get("institutional_email").map_err(AppError::from)?,
        hire_date:             row.try_get("hire_date").map_err(AppError::from)?,
        is_tenured:            row.try_get("is_tenured").map_err(AppError::from)?,
        primary_department_id: row.try_get("primary_department_id").map_err(AppError::from)?,
        primary_department:    row.try_get("primary_department").map_err(AppError::from)?,
        active_contract,
    }))
}
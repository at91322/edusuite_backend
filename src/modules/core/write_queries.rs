// src/modules/core/write_queries.rs
//
// Write queries for the core module — Step 1 (Groups 1 and 6).
// Groups 2–5 write queries appended in subsequent steps.

use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use super::write_models::*;

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 1a — PATCH /core/tenants/me
// ═══════════════════════════════════════════════════════════════════════════════

/// Update tenant self-service fields.
///
/// Only the three fields in the safe allowlist are patchable.
/// Uses coalesce logic: absent fields keep their current value.
/// subdivision_code is validated against reference.subdivisions if provided.
pub async fn patch_tenant(
    tx:  &mut sqlx::Transaction<'_, sqlx::Postgres>,
    req: &PatchTenantRequest,
) -> Result<super::models::TenantDetail, AppError> {

    // Validate subdivision_code exists in reference table if provided
    if let Some(ref code) = req.subdivision_code {
        let exists: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM reference.subdivisions WHERE code = $1
            ) AS "exists!"
            "#,
            code,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        if !exists {
            return Err(AppError::BadRequest(format!(
                "subdivision_code '{}' not found in reference.subdivisions", code
            )));
        }
    }

    let row = sqlx::query!(
        r#"
        UPDATE core.tenants SET
            name                  = COALESCE($1, name),
            subdivision_code      = COALESCE($2, subdivision_code),
            fiscal_year_end_month = COALESCE($3, fiscal_year_end_month),
            updated_at            = now()
        WHERE id = current_setting('app.current_tenant_id', true)::uuid
        RETURNING
            id, name, domain,
            country_iso_alpha2,
            subdivision_code,
            base_currency_code,
            fiscal_year_end_month,
            is_tax_exempt,
            created_at, updated_at
        "#,
        req.name.as_deref(),
        req.subdivision_code.as_deref(),
        req.fiscal_year_end_month,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(super::models::TenantDetail {
        id:                    row.id,
        name:                  row.name,
        domain:                row.domain,
        country_iso_alpha2:    row.country_iso_alpha2.map(|c| c.trim().to_string()),
        subdivision_code:      row.subdivision_code,
        base_currency_code:    row.base_currency_code.trim().to_string(),
        fiscal_year_end_month: row.fiscal_year_end_month,
        is_tax_exempt:         row.is_tax_exempt,
        created_at:            row.created_at,
        updated_at:            row.updated_at,
    })
}

// ── Group 1b: Department management ──────────────────────────────────────────

/// Create a new department.
///
/// Guards:
///   - code must be unique within the tenant (unique constraint on (tenant_id, code))
///   - head_user_id must be an active member of this tenant
pub async fn create_department(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    req:       &CreateDepartmentRequest,
) -> Result<DepartmentResponse, AppError> {

    // Guard: head_user_id is a member of this tenant
    let head_is_member: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM core.tenant_memberships
            WHERE user_id   = $1
              AND tenant_id = $2
        ) AS "exists!"
        "#,
        req.head_user_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !head_is_member {
        return Err(AppError::BadRequest(format!(
            "User {} is not a member of this tenant and cannot be set as department head",
            req.head_user_id
        )));
    }

    // INSERT — (tenant_id, code) unique constraint fires 23505 → Conflict
    let row = sqlx::query(
        r#"
        INSERT INTO core.departments
            (tenant_id, code, name, head_user_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, created_at, updated_at
        "#,
    )
    .bind(tenant_id)
    .bind(req.code.trim().to_uppercase())
    .bind(req.name.trim())
    .bind(req.head_user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    // Fetch head name for response
    let head_name: Option<String> = sqlx::query_scalar!(
        r#"
        SELECT first_name || ' ' || last_name
        FROM core.users WHERE id = $1
        "#,
        req.head_user_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .flatten();

    Ok(DepartmentResponse {
        id:           row.try_get("id").map_err(AppError::from)?,
        code:         req.code.trim().to_uppercase(),
        name:         req.name.trim().to_string(),
        head_user_id: Some(req.head_user_id),
        head_name,
        created_at:   row.try_get("created_at").map_err(AppError::from)?,
        updated_at:   row.try_get("updated_at").map_err(AppError::from)?,
    })
}

/// Update an existing department.
///
/// Partial PATCH — absent fields retain current values via COALESCE.
/// If head_user_id is being changed, validates the new head is a tenant member.
pub async fn patch_department(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    dept_id:   Uuid,
    req:       &PatchDepartmentRequest,
) -> Result<DepartmentResponse, AppError> {

    // Guard: department exists in this tenant
    let exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM core.departments
            WHERE id = $1 AND tenant_id = $2
        ) AS "exists!"
        "#,
        dept_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(format!("Department {} not found", dept_id)));
    }

    // Guard: new head_user_id must be a tenant member
    if let Some(head_id) = req.head_user_id {
        let is_member: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM core.tenant_memberships
                WHERE user_id = $1 AND tenant_id = $2
            ) AS "exists!"
            "#,
            head_id,
            tenant_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        if !is_member {
            return Err(AppError::BadRequest(format!(
                "User {} is not a member of this tenant and cannot be set as department head",
                head_id
            )));
        }
    }

    let code_upper = req.code.as_deref().map(|c| c.trim().to_uppercase());

    let row = sqlx::query(
        r#"
        UPDATE core.departments SET
            code         = COALESCE($1, code),
            name         = COALESCE($2, name),
            head_user_id = COALESCE($3, head_user_id),
            updated_at   = now()
        WHERE id = $4 AND tenant_id = $5
        RETURNING id, code, name, head_user_id, created_at, updated_at
        "#,
    )
    .bind(code_upper.as_deref())
    .bind(req.name.as_deref().map(|n| n.trim()))
    .bind(req.head_user_id)
    .bind(dept_id)
    .bind(tenant_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let head_id: Option<Uuid> = row.try_get("head_user_id").map_err(AppError::from)?;

    let head_name: Option<String> = if let Some(hid) = head_id {
        sqlx::query_scalar!(
            "SELECT first_name || ' ' || last_name FROM core.users WHERE id = $1",
            hid,
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(AppError::from)?
        .flatten()
    } else {
        None
    };

    Ok(DepartmentResponse {
        id:           row.try_get("id").map_err(AppError::from)?,
        code:         row.try_get("code").map_err(AppError::from)?,
        name:         row.try_get("name").map_err(AppError::from)?,
        head_user_id: head_id,
        head_name,
        created_at:   row.try_get("created_at").map_err(AppError::from)?,
        updated_at:   row.try_get("updated_at").map_err(AppError::from)?,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 6 — MEMBERSHIP MANAGEMENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Update a member's institutional_email and/or system_role.
///
/// Uses COALESCE so absent fields keep their current values.
/// system_role casts to the core.system_role enum via explicit ::text cast
/// on the input — untyped query required.
pub async fn patch_member(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    user_id:   Uuid,
    req:       &PatchMemberRequest,
) -> Result<MemberResponse, AppError> {

    // Guard: membership exists
    let exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM core.tenant_memberships
            WHERE user_id = $1 AND tenant_id = $2
        ) AS "exists!"
        "#,
        user_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(
            format!("Member {} not found in this tenant", user_id)
        ));
    }

    // system_role is a PG enum — use untyped query with explicit cast
    let row = sqlx::query(
        r#"
        UPDATE core.tenant_memberships SET
            institutional_email = COALESCE($1, institutional_email),
            system_role         = COALESCE($2::core.system_role, system_role),
            updated_at          = now()
        WHERE user_id   = $3
          AND tenant_id = $4
        RETURNING id, institutional_email, system_role::text AS system_role, updated_at
        "#,
    )
    .bind(req.institutional_email.as_deref())
    .bind(req.system_role.as_deref())
    .bind(user_id)
    .bind(tenant_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(MemberResponse {
        membership_id:       row.try_get("id").map_err(AppError::from)?,
        user_id,
        institutional_email: row.try_get("institutional_email").map_err(AppError::from)?,
        system_role:         row.try_get("system_role").map_err(AppError::from)?,
        updated_at:          row.try_get("updated_at").map_err(AppError::from)?,
    })
}

/// Revoke a member's access to this tenant.
///
/// Soft-deletes the tenant_memberships row by removing it entirely.
/// The core.users record is NOT deleted — the user may exist in other tenants.
/// Returns 409 if attempting to remove the last tenant_admin.
pub async fn delete_member(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    user_id:   Uuid,
) -> Result<(), AppError> {

    // Guard: membership exists
    let exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM core.tenant_memberships
            WHERE user_id = $1 AND tenant_id = $2
        ) AS "exists!"
        "#,
        user_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(
            format!("Member {} not found in this tenant", user_id)
        ));
    }

    // Guard: cannot remove the last tenant_admin
    let is_last_admin: bool = sqlx::query_scalar!(
        r#"
        SELECT (
            (SELECT system_role::text FROM core.tenant_memberships
             WHERE user_id = $1 AND tenant_id = $2) = 'tenant_admin'
            AND
            (SELECT COUNT(*) FROM core.tenant_memberships
             WHERE tenant_id = $2 AND system_role::text = 'tenant_admin') <= 1
        ) AS "is_last!"
        "#,
        user_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if is_last_admin {
        return Err(AppError::Conflict(
            "Cannot remove the last tenant_admin — assign another admin first".into()
        ));
    }

    sqlx::query!(
        r#"
        DELETE FROM core.tenant_memberships
        WHERE user_id = $1 AND tenant_id = $2
        "#,
        user_id,
        tenant_id,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        tenant_id = %tenant_id,
        user_id   = %user_id,
        "Tenant membership revoked"
    );

    Ok(())
}
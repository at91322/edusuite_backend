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

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 2 — USER IDENTITY WRITES
// ═══════════════════════════════════════════════════════════════════════════════

use super::write_models::{
    PatchUserRequest,
    CreateEmailRequest, PatchEmailRequest, EmailResponse,
    CreatePhoneRequest, PatchPhoneRequest, PhoneResponse,
    CreateAddressRequest, PatchAddressRequest, AddressResponse,
};
use super::models::UserDetail;
// ── PATCH /core/users/:id ────────────────────────────────────────────────────

/// Update a user's name and/or non-name fields.
///
/// Steps:
///   1. Guard: user is a member of this tenant (RLS enforces visibility;
///      this gives a clean 404 rather than a cryptic empty result).
///   2. Fetch current name values — needed for the history entry and for
///      MaybePatch::Absent merging.
///   3. If any legal name field is changing, INSERT into user_name_history
///      FIRST (capturing pre-change values) then UPDATE core.users.
///      Both writes share the transaction — if the UPDATE fails, the history
///      entry rolls back too.
///   4. Update non-name fields (preferred_name, last_name_suffix) in the
///      same UPDATE statement.
pub async fn patch_user(
    tx:          &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:   uuid::Uuid,
    user_id:     uuid::Uuid,
    actor_id:    uuid::Uuid,   // JWT sub — the person making the change
    req:         &PatchUserRequest,
) -> Result<UserDetail, AppError> {

    // ── 1. Guard: user is a member of this tenant ─────────────────────────
    let current = sqlx::query!(
        r#"
        SELECT u.first_name, u.middle_name, u.last_name,
               u.preferred_name, u.last_name_suffix,
               u.username, u.is_active, u.created_at, u.updated_at
        FROM core.users u
        JOIN core.tenant_memberships tm
          ON tm.user_id = u.id AND tm.tenant_id = $2
        WHERE u.id = $1
        "#,
        user_id,
        tenant_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

    // ── 2. Merge MaybePatch fields ────────────────────────────────────────
    let new_first = req.first_name.as_deref()
        .unwrap_or(&current.first_name)
        .trim().to_string();
    let new_last  = req.last_name.as_deref()
        .unwrap_or(&current.last_name)
        .trim().to_string();
    let new_middle: Option<String> = match &req.middle_name {
        MaybePatch::Value(v) => Some(v.trim().to_string()),
        MaybePatch::Null     => None,
        MaybePatch::Absent   => current.middle_name.clone(),
    };
    let new_preferred: Option<String> = match &req.preferred_name {
        MaybePatch::Value(v) => Some(v.trim().to_string()),
        MaybePatch::Null     => None,
        MaybePatch::Absent   => current.preferred_name.clone(),
    };
    let new_suffix: Option<String> = match &req.last_name_suffix {
        MaybePatch::Value(v) => Some(v.trim().to_string()),
        MaybePatch::Null     => None,
        MaybePatch::Absent   => current.last_name_suffix.clone(),
    };

    // ── 3. Write name history BEFORE updating (captures pre-change values) ─
    if req.has_name_change() {
        let reason = req.name_change_reason.as_deref().unwrap(); // validated earlier

        // Insert the current (pre-change) name as the historical record
// UNTYPED: reason binds to core.name_change_reason PG enum
        sqlx::query(
            r#"
            INSERT INTO core.user_name_history
                (user_id, historical_first_name, historical_middle_name,
                 historical_last_name, reason, changed_by_user_id)
            VALUES ($1, $2, $3, $4, $5::core.name_change_reason, $6)
            "#,
        )
        .bind(user_id)
        .bind(&current.first_name)
        .bind(&current.middle_name)
        .bind(&current.last_name)
        .bind(reason)
        .bind(actor_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    // ── 4. UPDATE core.users ──────────────────────────────────────────────
    let row = sqlx::query!(
        r#"
        UPDATE core.users SET
            first_name       = $2,
            middle_name      = $3,
            last_name        = $4,
            preferred_name   = $5,
            last_name_suffix = $6,
            updated_at       = now()
        WHERE id = $1
        RETURNING id, username, first_name, middle_name, last_name,
                  preferred_name, last_name_suffix AS suffix,
                  is_active, created_at, updated_at
        "#,
        user_id,
        new_first,
        new_middle,
        new_last,
        new_preferred,
        new_suffix,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        tenant_id = %tenant_id,
        user_id   = %user_id,
        actor_id  = %actor_id,
        name_change = req.has_name_change(),
        "User profile updated"
    );

    Ok(UserDetail {
        id:             row.id,
        username:       row.username,
        first_name:     row.first_name,
        middle_name:    row.middle_name,
        last_name:      row.last_name,
        preferred_name: row.preferred_name,
        suffix:         row.suffix,
        is_active:      row.is_active.unwrap_or(true),
        created_at:     row.created_at,
        updated_at:     row.updated_at,
    })
}

// ── POST /core/users/:id/emails ───────────────────────────────────────────────

/// Add an email address to a user.
///
/// The global unique constraint on email_address means a 23505 violation
/// → AppError::Conflict automatically — no duplicate across any user.
///
/// If is_primary = true, clears is_primary on all existing emails first,
/// then inserts the new primary. Both run in the same transaction.
pub async fn create_email(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
    req:     &CreateEmailRequest,
) -> Result<EmailResponse, AppError> {

    let is_primary  = req.is_primary.unwrap_or(false);
    let email_type  = req.email_type.as_deref().unwrap_or("personal");

    // Guard: user exists (RLS already scopes this, but gives clean 404)
    let user_exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM core.users WHERE id = $1) AS "exists!""#,
        user_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !user_exists {
        return Err(AppError::NotFound(format!("User {} not found", user_id)));
    }

    // Demote existing primary if needed
    if is_primary {
        sqlx::query!(
            "UPDATE core.user_emails SET is_primary = false WHERE user_id = $1 AND is_primary = true",
            user_id,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    // INSERT — 23505 on unique_global_email → Conflict
    let row = sqlx::query(
        r#"
        INSERT INTO core.user_emails (user_id, email_address, type, is_primary)
        VALUES ($1, $2, $3::core.email_type, $4)
        RETURNING id, email_address, type::text AS email_type,
                  is_primary, is_verified, created_at
        "#,
    )
    .bind(user_id)
    .bind(req.email_address.trim().to_lowercase())
    .bind(email_type)
    .bind(is_primary)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    use sqlx::Row as _;
    Ok(EmailResponse {
        id:            row.try_get("id").map_err(AppError::from)?,
        user_id,
        email_address: row.try_get("email_address").map_err(AppError::from)?,
        email_type:    row.try_get("email_type").map_err(AppError::from)?,
        is_primary:    row.try_get("is_primary").map_err(AppError::from)?,
        is_verified:   row.try_get("is_verified").map_err(AppError::from)?,
        created_at:    row.try_get("created_at").map_err(AppError::from)?,
    })
}

/// Update email metadata (type, primary flag, verified status).
/// The email_address itself is immutable — changes require DELETE + POST.
pub async fn patch_email(
    tx:       &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id:  uuid::Uuid,
    email_id: uuid::Uuid,
    req:      &PatchEmailRequest,
) -> Result<EmailResponse, AppError> {

    // Guard: email belongs to this user
    let exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM core.user_emails WHERE id = $1 AND user_id = $2) AS "exists!""#,
        email_id, user_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(
            format!("Email {} not found for user {}", email_id, user_id)
        ));
    }

    // Demote current primary if promoting this one
    if req.is_primary == Some(true) {
        sqlx::query!(
            "UPDATE core.user_emails SET is_primary = false WHERE user_id = $1 AND is_primary = true",
            user_id,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    // Guard: cannot demote primary without a replacement
    if req.is_primary == Some(false) {
        let is_currently_primary: bool = sqlx::query_scalar!(
            r#"SELECT is_primary FROM core.user_emails WHERE id = $1"#,
            email_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        if is_currently_primary {
            return Err(AppError::Conflict(
                "Cannot remove is_primary from this address — set another address as primary first".into()
            ));
        }
    }

    let row = sqlx::query(
        r#"
        UPDATE core.user_emails SET
            type       = COALESCE($3::core.email_type, type),
            is_primary = COALESCE($4, is_primary),
            is_verified = COALESCE($5, is_verified)
        WHERE id = $1 AND user_id = $2
        RETURNING id, email_address, type::text AS email_type,
                  is_primary, is_verified, created_at
        "#,
    )
    .bind(email_id)
    .bind(user_id)
    .bind(req.email_type.as_deref())
    .bind(req.is_primary)
    .bind(req.is_verified)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    use sqlx::Row as _;
    Ok(EmailResponse {
        id:            row.try_get("id").map_err(AppError::from)?,
        user_id,
        email_address: row.try_get("email_address").map_err(AppError::from)?,
        email_type:    row.try_get("email_type").map_err(AppError::from)?,
        is_primary:    row.try_get("is_primary").map_err(AppError::from)?,
        is_verified:   row.try_get("is_verified").map_err(AppError::from)?,
        created_at:    row.try_get("created_at").map_err(AppError::from)?,
    })
}

/// Delete an email address.
/// Blocks deletion of the primary email — caller must promote another first.
pub async fn delete_email(
    tx:       &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id:  uuid::Uuid,
    email_id: uuid::Uuid,
) -> Result<(), AppError> {

    let row = sqlx::query!(
        "SELECT is_primary FROM core.user_emails WHERE id = $1 AND user_id = $2",
        email_id, user_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("Email {} not found for user {}", email_id, user_id)
    ))?;

    if row.is_primary {
        return Err(AppError::Conflict(
            "Cannot delete the primary email address — promote another address first".into()
        ));
    }

    sqlx::query!("DELETE FROM core.user_emails WHERE id = $1 AND user_id = $2", email_id, user_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;

    Ok(())
}

// ── POST /core/users/:id/phones ───────────────────────────────────────────────

pub async fn create_phone(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
    req:     &CreatePhoneRequest,
) -> Result<PhoneResponse, AppError> {

    let is_primary   = req.is_primary.unwrap_or(false);
    let phone_type   = req.phone_type.as_deref().unwrap_or("mobile");
    let country_code = req.country_code.as_deref().unwrap_or("+1");

    if is_primary {
        sqlx::query!(
            "UPDATE core.user_phones SET is_primary = false WHERE user_id = $1 AND is_primary = true",
            user_id,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    let row = sqlx::query(
        r#"
        INSERT INTO core.user_phones
            (user_id, phone_number, country_code, type, is_primary, can_receive_sms)
        VALUES ($1, $2, $3, $4::core.phone_type, $5, $6)
        RETURNING id, phone_number, country_code, type::text AS phone_type,
                  is_primary, can_receive_sms, created_at
        "#,
    )
    .bind(user_id)
    .bind(req.phone_number.trim())
    .bind(country_code)
    .bind(phone_type)
    .bind(is_primary)
    .bind(req.can_receive_sms.unwrap_or(false))
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    use sqlx::Row as _;
    Ok(PhoneResponse {
        id:              row.try_get("id").map_err(AppError::from)?,
        user_id,
        phone_number:    row.try_get("phone_number").map_err(AppError::from)?,
        country_code:    row.try_get("country_code").map_err(AppError::from)?,
        phone_type:      row.try_get("phone_type").map_err(AppError::from)?,
        is_primary:      row.try_get("is_primary").map_err(AppError::from)?,
        can_receive_sms: row.try_get("can_receive_sms").map_err(AppError::from)?,
        created_at:      row.try_get("created_at").map_err(AppError::from)?,
    })
}

pub async fn patch_phone(
    tx:       &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id:  uuid::Uuid,
    phone_id: uuid::Uuid,
    req:      &PatchPhoneRequest,
) -> Result<PhoneResponse, AppError> {

    let exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM core.user_phones WHERE id = $1 AND user_id = $2) AS "exists!""#,
        phone_id, user_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(
            format!("Phone {} not found for user {}", phone_id, user_id)
        ));
    }

    if req.is_primary == Some(true) {
        sqlx::query!(
            "UPDATE core.user_phones SET is_primary = false WHERE user_id = $1 AND is_primary = true",
            user_id,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    let row = sqlx::query(
        r#"
        UPDATE core.user_phones SET
            type            = COALESCE($3::core.phone_type, type),
            is_primary      = COALESCE($4, is_primary),
            can_receive_sms = COALESCE($5, can_receive_sms)
        WHERE id = $1 AND user_id = $2
        RETURNING id, phone_number, country_code, type::text AS phone_type,
                  is_primary, can_receive_sms, created_at
        "#,
    )
    .bind(phone_id)
    .bind(user_id)
    .bind(req.phone_type.as_deref())
    .bind(req.is_primary)
    .bind(req.can_receive_sms)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    use sqlx::Row as _;
    Ok(PhoneResponse {
        id:              row.try_get("id").map_err(AppError::from)?,
        user_id,
        phone_number:    row.try_get("phone_number").map_err(AppError::from)?,
        country_code:    row.try_get("country_code").map_err(AppError::from)?,
        phone_type:      row.try_get("phone_type").map_err(AppError::from)?,
        is_primary:      row.try_get("is_primary").map_err(AppError::from)?,
        can_receive_sms: row.try_get("can_receive_sms").map_err(AppError::from)?,
        created_at:      row.try_get("created_at").map_err(AppError::from)?,
    })
}

pub async fn delete_phone(
    tx:       &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id:  uuid::Uuid,
    phone_id: uuid::Uuid,
) -> Result<(), AppError> {

    let row = sqlx::query!(
        "SELECT is_primary FROM core.user_phones WHERE id = $1 AND user_id = $2",
        phone_id, user_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(
        format!("Phone {} not found for user {}", phone_id, user_id)
    ))?;

    if row.is_primary {
        return Err(AppError::Conflict(
            "Cannot delete the primary phone number — promote another number first".into()
        ));
    }

    sqlx::query!("DELETE FROM core.user_phones WHERE id = $1 AND user_id = $2", phone_id, user_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;

    Ok(())
}

// ── POST /core/users/:id/addresses ────────────────────────────────────────────

/// Add an address. Validates country/subdivision/timezone FKs if provided.
/// Sets is_current = true and effective_start_date = today by default.
pub async fn create_address(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
    req:     &CreateAddressRequest,
) -> Result<AddressResponse, AppError> {

    // Validate country FK
    if let Some(ref country) = req.country_iso_alpha2 {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM reference.countries WHERE iso_alpha2 = $1) AS "exists!""#,
            country.as_str(),
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;
        if !exists {
            return Err(AppError::BadRequest(
                format!("country_iso_alpha2 '{}' not found in reference.countries", country)
            ));
        }
    }

    // Validate subdivision FK
    if let Some(ref sub) = req.subdivision_code {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM reference.subdivisions WHERE code = $1) AS "exists!""#,
            sub.as_str(),
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;
        if !exists {
            return Err(AppError::BadRequest(
                format!("subdivision_code '{}' not found in reference.subdivisions", sub)
            ));
        }
    }

    // Validate timezone FK
    if let Some(ref tz) = req.timezone_identifier {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM reference.timezones WHERE tz_identifier = $1) AS "exists!""#,
            tz.as_str(),
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;
        if !exists {
            return Err(AppError::BadRequest(
                format!("timezone_identifier '{}' not found in reference.timezones", tz)
            ));
        }
    }

    let effective_start = req.effective_start_date
        .unwrap_or_else(|| chrono::Local::now().date_naive());

    let row = sqlx::query(
        r#"
        INSERT INTO core.user_addresses
            (user_id, type, street_1, street_2, city, state_province,
             postal_code, country_iso_alpha2, subdivision_code,
             timezone_identifier, effective_start_date, is_current)
        VALUES ($1, $2::core.address_type, $3, $4, $5, $6, $7, $8, $9, $10, $11, true)
        RETURNING id, type::text AS address_type, street_1, street_2,
                  city, state_province, postal_code,
                  country_iso_alpha2, subdivision_code, timezone_identifier,
                  is_current, is_verified,
                  effective_start_date, effective_end_date,
                  created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(req.address_type.as_str())
    .bind(req.street_1.trim())
    .bind(req.street_2.as_deref())
    .bind(req.city.trim())
    .bind(req.state_province.trim())
    .bind(req.postal_code.trim())
    .bind(req.country_iso_alpha2.as_deref())
    .bind(req.subdivision_code.as_deref())
    .bind(req.timezone_identifier.as_deref())
    .bind(effective_start)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    use sqlx::Row as _;
    Ok(AddressResponse {
        id:                   row.try_get("id").map_err(AppError::from)?,
        user_id,
        address_type:         row.try_get("address_type").map_err(AppError::from)?,
        street_1:             row.try_get("street_1").map_err(AppError::from)?,
        street_2:             row.try_get("street_2").map_err(AppError::from)?,
        city:                 row.try_get("city").map_err(AppError::from)?,
        state_province:       row.try_get("state_province").map_err(AppError::from)?,
        postal_code:          row.try_get("postal_code").map_err(AppError::from)?,
        country_iso_alpha2:   row.try_get::<Option<&str>, _>("country_iso_alpha2")
                                  .map_err(AppError::from)?
                                  .map(|s| s.trim().to_string()),
        subdivision_code:     row.try_get("subdivision_code").map_err(AppError::from)?,
        timezone_identifier:  row.try_get("timezone_identifier").map_err(AppError::from)?,
        is_current:           row.try_get("is_current").map_err(AppError::from)?,
        is_verified:          row.try_get("is_verified").map_err(AppError::from)?,
        effective_start_date: row.try_get("effective_start_date").map_err(AppError::from)?,
        effective_end_date:   row.try_get("effective_end_date").map_err(AppError::from)?,
        created_at:           row.try_get("created_at").map_err(AppError::from)?,
        updated_at:           row.try_get("updated_at").map_err(AppError::from)?,
    })
}

pub async fn patch_address(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
    addr_id: uuid::Uuid,
    req:     &PatchAddressRequest,
) -> Result<AddressResponse, AppError> {

    let exists: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM core.user_addresses WHERE id = $1 AND user_id = $2) AS "exists!""#,
        addr_id, user_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(
            format!("Address {} not found for user {}", addr_id, user_id)
        ));
    }

    // Validate subdivision if changing
    if let Some(ref sub) = req.subdivision_code.as_value() {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM reference.subdivisions WHERE code = $1) AS "exists!""#,
            sub.as_str(),
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;
        if !exists {
            return Err(AppError::BadRequest(
                format!("subdivision_code '{}' not found in reference.subdivisions", sub)
            ));
        }
    }

    // Fetch current for MaybePatch merging
    let current = sqlx::query!(
        r#"SELECT street_1, street_2, city, state_province, postal_code,
                  country_iso_alpha2, subdivision_code, timezone_identifier,
                  is_current, effective_end_date
           FROM core.user_addresses WHERE id = $1"#,
        addr_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let new_street_2: Option<String> = match &req.street_2 {
        MaybePatch::Value(v) => Some(v.clone()),
        MaybePatch::Null     => None,
        MaybePatch::Absent   => current.street_2.clone(),
    };
    let new_subdivision: Option<String> = match &req.subdivision_code {
        MaybePatch::Value(v) => Some(v.clone()),
        MaybePatch::Null     => None,
        MaybePatch::Absent   => current.subdivision_code.clone(),
    };
    let new_timezone: Option<String> = match &req.timezone_identifier {
        MaybePatch::Value(v) => Some(v.clone()),
        MaybePatch::Null     => None,
        MaybePatch::Absent   => current.timezone_identifier.clone(),
    };
    let new_eff_end: Option<chrono::NaiveDate> = match &req.effective_end_date {
        MaybePatch::Value(v) => Some(*v),
        MaybePatch::Null     => None,
        MaybePatch::Absent   => current.effective_end_date,
    };

    let row = sqlx::query(
        r#"
        UPDATE core.user_addresses SET
            street_1            = COALESCE($3, street_1),
            street_2            = $4,
            city                = COALESCE($5, city),
            state_province      = COALESCE($6, state_province),
            postal_code         = COALESCE($7, postal_code),
            country_iso_alpha2  = COALESCE($8, country_iso_alpha2),
            subdivision_code    = $9,
            timezone_identifier = $10,
            is_current          = COALESCE($11, is_current),
            effective_end_date  = $12,
            updated_at          = now()
        WHERE id = $1 AND user_id = $2
        RETURNING id, type::text AS address_type, street_1, street_2,
                  city, state_province, postal_code,
                  country_iso_alpha2, subdivision_code, timezone_identifier,
                  is_current, is_verified,
                  effective_start_date, effective_end_date,
                  created_at, updated_at
        "#,
    )
    .bind(addr_id)
    .bind(user_id)
    .bind(req.street_1.as_deref())
    .bind(new_street_2)
    .bind(req.city.as_deref())
    .bind(req.state_province.as_deref())
    .bind(req.postal_code.as_deref())
    .bind(req.country_iso_alpha2.as_deref())
    .bind(new_subdivision)
    .bind(new_timezone)
    .bind(req.is_current)
    .bind(new_eff_end)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    use sqlx::Row as _;
    Ok(AddressResponse {
        id:                   row.try_get("id").map_err(AppError::from)?,
        user_id,
        address_type:         row.try_get("address_type").map_err(AppError::from)?,
        street_1:             row.try_get("street_1").map_err(AppError::from)?,
        street_2:             row.try_get("street_2").map_err(AppError::from)?,
        city:                 row.try_get("city").map_err(AppError::from)?,
        state_province:       row.try_get("state_province").map_err(AppError::from)?,
        postal_code:          row.try_get("postal_code").map_err(AppError::from)?,
        country_iso_alpha2:   row.try_get::<Option<&str>, _>("country_iso_alpha2")
                                  .map_err(AppError::from)?
                                  .map(|s| s.trim().to_string()),
        subdivision_code:     row.try_get("subdivision_code").map_err(AppError::from)?,
        timezone_identifier:  row.try_get("timezone_identifier").map_err(AppError::from)?,
        is_current:           row.try_get("is_current").map_err(AppError::from)?,
        is_verified:          row.try_get("is_verified").map_err(AppError::from)?,
        effective_start_date: row.try_get("effective_start_date").map_err(AppError::from)?,
        effective_end_date:   row.try_get("effective_end_date").map_err(AppError::from)?,
        created_at:           row.try_get("created_at").map_err(AppError::from)?,
        updated_at:           row.try_get("updated_at").map_err(AppError::from)?,
    })
}

pub async fn delete_address(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
    addr_id: uuid::Uuid,
) -> Result<(), AppError> {

    let rows_deleted = sqlx::query!(
        "DELETE FROM core.user_addresses WHERE id = $1 AND user_id = $2",
        addr_id, user_id,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?
    .rows_affected();

    if rows_deleted == 0 {
        return Err(AppError::NotFound(
            format!("Address {} not found for user {}", addr_id, user_id)
        ));
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// STEP 3 — EMERGENCY CONTACT WRITES
// ═══════════════════════════════════════════════════════════════════════════════

use super::write_models::{
    CreateEmergencyContactRequest, PatchEmergencyContactRequest, EmergencyContactResponse,
};

// ── POST /core/users/:id/emergency-contacts ───────────────────────────────────

/// Add an emergency contact for a user.
///
/// RLS note: emergency_contacts uses cross_tenant_user_isolation — there is no
/// tenant_id column on the table. RLS visibility is controlled by the user's
/// tenant membership. The write query confirms the user exists in the current
/// tenant context before inserting.
///
/// If is_primary = true, demotes the existing primary contact first.
pub async fn create_emergency_contact(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
    req:     &CreateEmergencyContactRequest,
) -> Result<EmergencyContactResponse, AppError> {

    // Guard: user exists in the current tenant context (RLS-scoped query)
    let user_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM core.users u
            JOIN core.tenant_memberships tm
              ON tm.user_id = u.id
              AND tm.tenant_id = current_setting('app.current_tenant_id', true)::uuid
            WHERE u.id = $1
        ) AS "exists!"
        "#,
        user_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !user_exists {
        return Err(AppError::NotFound(format!("User {} not found", user_id)));
    }

    let is_primary = req.is_primary.unwrap_or(false);

    // Demote existing primary before promoting the new one
    if is_primary {
        sqlx::query!(
            "UPDATE core.emergency_contacts SET is_primary = false WHERE user_id = $1 AND is_primary = true",
            user_id,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    let row = sqlx::query!(
        r#"
        INSERT INTO core.emergency_contacts
            (user_id, first_name, last_name, relationship, phone_number, email_address, is_primary)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, first_name, last_name, relationship, phone_number,
                  email_address, is_primary, created_at
        "#,
        user_id,
        req.first_name.trim(),
        req.last_name.trim(),
        req.relationship.trim(),
        req.phone_number.trim(),
        req.email_address.as_deref(),
        is_primary,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        user_id    = %user_id,
        contact_id = %row.id,
        is_primary,
        "Emergency contact created"
    );

    Ok(EmergencyContactResponse {
        id:            row.id,
        user_id,
        first_name:    row.first_name,
        last_name:     row.last_name,
        relationship:  row.relationship,
        phone_number:  row.phone_number,
        email_address: row.email_address,
        is_primary:    row.is_primary,
        created_at:    row.created_at,
    })
}

// ── PATCH /core/users/:id/emergency-contacts/:contact_id ─────────────────────

/// Update an emergency contact.
///
/// Uses COALESCE for simple fields. email_address uses MaybePatch three-state:
///   Absent  → keep current value
///   Null    → clear the email (set to NULL)
///   Value   → set to new email
///
/// is_primary promotion demotes the existing primary first, then updates.
pub async fn patch_emergency_contact(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id:    uuid::Uuid,
    contact_id: uuid::Uuid,
    req:        &PatchEmergencyContactRequest,
) -> Result<EmergencyContactResponse, AppError> {

    // Guard: contact exists and belongs to this user
    let exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM core.emergency_contacts
            WHERE id = $1 AND user_id = $2
        ) AS "exists!"
        "#,
        contact_id,
        user_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !exists {
        return Err(AppError::NotFound(
            format!("Emergency contact {} not found for user {}", contact_id, user_id)
        ));
    }

    // Demote existing primary if promoting this contact
    if req.is_primary == Some(true) {
        sqlx::query!(
            "UPDATE core.emergency_contacts SET is_primary = false WHERE user_id = $1 AND is_primary = true",
            user_id,
        )
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;
    }

    // Fetch current for MaybePatch merge on email_address
    let current_email: Option<String> = sqlx::query_scalar!(
        "SELECT email_address FROM core.emergency_contacts WHERE id = $1",
        contact_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let new_email: Option<String> = match &req.email_address {
        super::write_models::MaybePatch::Value(v) => Some(v.clone()),
        super::write_models::MaybePatch::Null      => None,
        super::write_models::MaybePatch::Absent    => current_email,
    };

    let row = sqlx::query!(
        r#"
        UPDATE core.emergency_contacts SET
            first_name    = COALESCE($3, first_name),
            last_name     = COALESCE($4, last_name),
            relationship  = COALESCE($5, relationship),
            phone_number  = COALESCE($6, phone_number),
            email_address = $7,
            is_primary    = COALESCE($8, is_primary)
        WHERE id = $1 AND user_id = $2
        RETURNING id, first_name, last_name, relationship, phone_number,
                  email_address, is_primary, created_at
        "#,
        contact_id,
        user_id,
        req.first_name.as_deref(),
        req.last_name.as_deref(),
        req.relationship.as_deref(),
        req.phone_number.as_deref(),
        new_email,
        req.is_primary,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(EmergencyContactResponse {
        id:            row.id,
        user_id,
        first_name:    row.first_name,
        last_name:     row.last_name,
        relationship:  row.relationship,
        phone_number:  row.phone_number,
        email_address: row.email_address,
        is_primary:    row.is_primary,
        created_at:    row.created_at,
    })
}

// ── DELETE /core/users/:id/emergency-contacts/:contact_id ────────────────────

/// Delete an emergency contact.
///
/// No soft-delete — emergency contacts are not compliance records.
/// No primary-deletion guard: unlike emails/phones, there is no system
/// requirement for a user to always have an emergency contact on file
/// (especially relevant for adult Higher Ed users). The caller may delete
/// whichever contact they choose, including the primary.
pub async fn delete_emergency_contact(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id:    uuid::Uuid,
    contact_id: uuid::Uuid,
) -> Result<(), AppError> {

    let rows_deleted = sqlx::query!(
        "DELETE FROM core.emergency_contacts WHERE id = $1 AND user_id = $2",
        contact_id,
        user_id,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?
    .rows_affected();

    if rows_deleted == 0 {
        return Err(AppError::NotFound(
            format!("Emergency contact {} not found for user {}", contact_id, user_id)
        ));
    }

    tracing::info!(
        user_id    = %user_id,
        contact_id = %contact_id,
        "Emergency contact deleted"
    );

    Ok(())
}
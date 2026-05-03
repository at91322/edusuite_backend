// src/modules/core/queries.rs
//
// Read queries for the core module — Step 1 (Groups 1 and 6).
// Groups 2–5 queries appended in subsequent steps.

use chrono::NaiveDate;
use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use super::models::*;

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 1a — TENANT SELF-SERVICE
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn get_tenant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Option<TenantDetail>, AppError> {

    let row = sqlx::query!(
        r#"
        SELECT
            id, name, domain,
            country_iso_alpha2,
            subdivision_code,
            base_currency_code,
            fiscal_year_end_month,
            is_tax_exempt,
            created_at, updated_at
        FROM core.tenants
        WHERE id = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| TenantDetail {
        id:                    r.id,
        name:                  r.name,
        domain:                r.domain,
        country_iso_alpha2:    r.country_iso_alpha2.map(|c| c.trim().to_string()),
        subdivision_code:      r.subdivision_code,
        base_currency_code:    r.base_currency_code.trim().to_string(),
        fiscal_year_end_month: r.fiscal_year_end_month,
        is_tax_exempt:         r.is_tax_exempt,
        created_at:            r.created_at,
        updated_at:            r.updated_at,
    }))
}

pub async fn get_tenant_subscriptions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<SubscriptionSummary>, AppError> {

    let rows = sqlx::query(
        r#"
        SELECT
            ts.id,
            ts.module_name::text    AS module_name,
            mc.display_name,
            ts.tier::text           AS tier,
            ts.status::text         AS status,
            ts.max_students,
            ts.max_staff,
            ts.activated_at,
            ts.trial_ends_at
        FROM core.tenant_subscriptions ts
        JOIN core.module_catalog mc ON mc.module_name = ts.module_name
        WHERE ts.tenant_id = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY mc.display_order ASC, mc.display_name ASC
        "#,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    rows.iter().map(|r| -> Result<SubscriptionSummary, AppError> {
        Ok(SubscriptionSummary {
            id:            r.try_get("id").map_err(AppError::from)?,
            module_name:   r.try_get("module_name").map_err(AppError::from)?,
            display_name:  r.try_get("display_name").map_err(AppError::from)?,
            tier:          r.try_get("tier").map_err(AppError::from)?,
            status:        r.try_get("status").map_err(AppError::from)?,
            max_students:  r.try_get("max_students").map_err(AppError::from)?,
            max_staff:     r.try_get("max_staff").map_err(AppError::from)?,
            activated_at:  r.try_get("activated_at").map_err(AppError::from)?,
            trial_ends_at: r.try_get("trial_ends_at").map_err(AppError::from)?,
        })
    }).collect()
}

pub async fn get_feature_flags(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<FeatureFlag>, AppError> {

    let rows = sqlx::query!(
        r#"
        SELECT id, flag_name, is_enabled, description, updated_at
        FROM core.feature_flags
        WHERE tenant_id = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY flag_name ASC
        "#,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| FeatureFlag {
        id:          r.id,
        flag_name:   r.flag_name,
        is_enabled:  r.is_enabled,
        description: r.description,
        updated_at:  r.updated_at,
    }).collect())
}

// ── Group 1b: Departments ─────────────────────────────────────────────────────

pub async fn list_departments(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListDepartmentsParams,
) -> Result<(Vec<DepartmentSummary>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM core.departments
        WHERE tenant_id = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            d.id, d.code, d.name, d.head_user_id,
            u.first_name || ' ' || u.last_name AS head_name,
            d.created_at, d.updated_at
        FROM core.departments d
        LEFT JOIN core.users u ON u.id = d.head_user_id
        WHERE d.tenant_id = current_setting('app.current_tenant_id', true)::uuid
        ORDER BY d.code ASC
        LIMIT  $1
        OFFSET $2
        "#,
    )
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let depts = rows.iter().map(|r| -> Result<DepartmentSummary, AppError> {
        Ok(DepartmentSummary {
            id:           r.try_get("id").map_err(AppError::from)?,
            code:         r.try_get("code").map_err(AppError::from)?,
            name:         r.try_get("name").map_err(AppError::from)?,
            head_user_id: r.try_get("head_user_id").map_err(AppError::from)?,
            head_name:    r.try_get("head_name").map_err(AppError::from)?,
            created_at:   r.try_get("created_at").map_err(AppError::from)?,
            updated_at:   r.try_get("updated_at").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((depts, total))
}

pub async fn get_department(
    tx:   &mut sqlx::Transaction<'_, sqlx::Postgres>,
    dept_id: Uuid,
) -> Result<Option<DepartmentSummary>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            d.id, d.code, d.name, d.head_user_id,
            u.first_name || ' ' || u.last_name AS head_name,
            d.created_at, d.updated_at
        FROM core.departments d
        LEFT JOIN core.users u ON u.id = d.head_user_id
        WHERE d.id        = $1
          AND d.tenant_id = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(dept_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| DepartmentSummary {
        id:           r.try_get("id").unwrap_or_default(),
        code:         r.try_get("code").unwrap_or_default(),
        name:         r.try_get("name").unwrap_or_default(),
        head_user_id: r.try_get("head_user_id").unwrap_or_default(),
        head_name:    r.try_get("head_name").unwrap_or_default(),
        created_at:   r.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
        updated_at:   r.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now()),
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 6 — TENANT MEMBERSHIP MANAGEMENT
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_members(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListMembersParams,
) -> Result<(Vec<MemberSummary>, i64), AppError> {

    let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s.to_lowercase()));

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM core.tenant_memberships tm
        JOIN core.users u ON u.id = tm.user_id
        WHERE tm.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR tm.system_role::text = $1)
          AND ($2::text IS NULL OR (
              lower(u.first_name) LIKE $2
              OR lower(u.last_name) LIKE $2
              OR lower(u.username) LIKE $2
          ))
        "#,
        params.system_role.clone() as Option<String>,
        search_pattern.clone()     as Option<String>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            tm.id                       AS membership_id,
            tm.user_id,
            u.username,
            u.first_name,
            u.last_name,
            u.preferred_name,
            tm.institutional_email,
            tm.system_role::text        AS system_role,
            tm.joined_at,
            tm.last_accessed_at,
            u.is_active
        FROM core.tenant_memberships tm
        JOIN core.users u ON u.id = tm.user_id
        WHERE tm.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR tm.system_role::text = $1)
          AND ($2::text IS NULL OR (
              lower(u.first_name) LIKE $2
              OR lower(u.last_name) LIKE $2
              OR lower(u.username) LIKE $2
          ))
        ORDER BY u.last_name ASC, u.first_name ASC
        LIMIT  $3
        OFFSET $4
        "#,
    )
    .bind(params.system_role.as_deref())
    .bind(search_pattern.as_deref())
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let members = rows.iter().map(|r| -> Result<MemberSummary, AppError> {
        Ok(MemberSummary {
            membership_id:       r.try_get("membership_id").map_err(AppError::from)?,
            user_id:             r.try_get("user_id").map_err(AppError::from)?,
            username:            r.try_get("username").map_err(AppError::from)?,
            first_name:          r.try_get("first_name").map_err(AppError::from)?,
            last_name:           r.try_get("last_name").map_err(AppError::from)?,
            preferred_name:      r.try_get("preferred_name").map_err(AppError::from)?,
            institutional_email: r.try_get("institutional_email").map_err(AppError::from)?,
            system_role:         r.try_get("system_role").map_err(AppError::from)?,
            joined_at:           r.try_get("joined_at").map_err(AppError::from)?,
            last_accessed_at:    r.try_get("last_accessed_at").map_err(AppError::from)?,
            is_active:           r.try_get("is_active").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((members, total))
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 2 — USER IDENTITY (Step 2)
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn get_user(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Option<UserDetail>, AppError> {

    let row = sqlx::query!(
        r#"
        SELECT
            u.id, u.username, u.first_name, u.middle_name,
            u.last_name, u.preferred_name, u.last_name_suffix AS suffix,
            u.is_active, u.created_at, u.updated_at
        FROM core.users u
        JOIN core.tenant_memberships tm
          ON tm.user_id = u.id
          AND tm.tenant_id = current_setting('app.current_tenant_id', true)::uuid
        WHERE u.id = $1
        "#,
        user_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| UserDetail {
        id:             r.id,
        username:       r.username,
        first_name:     r.first_name,
        middle_name:    r.middle_name,
        last_name:      r.last_name,
        preferred_name: r.preferred_name,
        suffix:         r.suffix,
        is_active:      r.is_active,
        created_at:     r.created_at,
        updated_at:     r.updated_at,
    }))
}

pub async fn get_name_history(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Vec<NameHistoryEntry>, AppError> {

    let rows = sqlx::query(
        r#"
        SELECT
            nh.id,
            nh.historical_first_name,
            nh.historical_middle_name,
            nh.historical_last_name,
            nh.reason::text                          AS reason,
            nh.changed_by_user_id,
            u.first_name || ' ' || u.last_name       AS changed_by_name,
            nh.changed_at
        FROM core.user_name_history nh
        LEFT JOIN core.users u ON u.id = nh.changed_by_user_id
        WHERE nh.user_id = $1
        ORDER BY nh.changed_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    rows.iter().map(|r| -> Result<NameHistoryEntry, AppError> {
        Ok(NameHistoryEntry {
            id:                    r.try_get("id").map_err(AppError::from)?,
            historical_first_name: r.try_get("historical_first_name").map_err(AppError::from)?,
            historical_middle_name: r.try_get("historical_middle_name").map_err(AppError::from)?,
            historical_last_name:  r.try_get("historical_last_name").map_err(AppError::from)?,
            reason:                r.try_get("reason").map_err(AppError::from)?,
            changed_by_user_id:    r.try_get("changed_by_user_id").map_err(AppError::from)?,
            changed_by_name:       r.try_get("changed_by_name").map_err(AppError::from)?,
            changed_at:            r.try_get("changed_at").map_err(AppError::from)?,
        })
    }).collect()
}

pub async fn list_emails(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Vec<UserEmail>, AppError> {

    let rows = sqlx::query!(
        r#"
        SELECT id, email_address, type::text AS "email_type!", is_primary, is_verified, created_at
        FROM core.user_emails
        WHERE user_id = $1
        ORDER BY is_primary DESC, created_at ASC
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| UserEmail {
        id:            r.id,
        email_address: r.email_address,
        email_type:    r.email_type,
        is_primary:    r.is_primary,
        is_verified:   r.is_verified,
        created_at:    r.created_at,
    }).collect())
}

pub async fn list_phones(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Vec<UserPhone>, AppError> {

    let rows = sqlx::query!(
        r#"
        SELECT
            id, phone_number, country_code,
            type::text AS "phone_type!",
            is_primary, can_receive_sms, created_at
        FROM core.user_phones
        WHERE user_id = $1
        ORDER BY is_primary DESC, created_at ASC
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| UserPhone {
        id:              r.id,
        phone_number:    r.phone_number,
        country_code:    r.country_code,
        phone_type:      r.phone_type,
        is_primary:      r.is_primary,
        can_receive_sms: r.can_receive_sms,
        created_at:      r.created_at,
    }).collect())
}

pub async fn list_addresses(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Vec<UserAddress>, AppError> {

    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            type::text          AS "address_type!",
            street_1, street_2, city, state_province, postal_code,
            country_iso_alpha2, subdivision_code, timezone_identifier,
            is_current, is_verified,
            effective_start_date, effective_end_date,
            created_at, updated_at
        FROM core.user_addresses
        WHERE user_id = $1
        ORDER BY is_current DESC, effective_start_date DESC
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| UserAddress {
        id:                   r.id,
        address_type:         r.address_type,
        street_1:             r.street_1,
        street_2:             r.street_2,
        city:                 r.city,
        state_province:       r.state_province,
        postal_code:          r.postal_code,
        country_iso_alpha2:   r.country_iso_alpha2.map(|c| c.trim().to_string()),
        subdivision_code:     r.subdivision_code,
        timezone_identifier:  r.timezone_identifier,
        is_current:           r.is_current,
        is_verified:          r.is_verified,
        effective_start_date: r.effective_start_date,
        effective_end_date:   r.effective_end_date,
        created_at:           r.created_at,
        updated_at:           r.updated_at,
    }).collect())
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 3 — EMERGENCY CONTACTS (Step 3)
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_emergency_contacts(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Vec<EmergencyContact>, AppError> {

    // Actual columns: id, user_id, first_name, last_name, relationship (varchar),
    // phone_number, email_address, is_primary, created_at
    // No: name, phone_primary/secondary, can_pickup, notes, updated_at
    let rows = sqlx::query!(
        r#"
        SELECT
            id, user_id, first_name, last_name,
            relationship,
            phone_number, email_address,
            is_primary, created_at
        FROM core.emergency_contacts
        WHERE user_id = $1
        ORDER BY is_primary DESC, created_at ASC
        "#,
        user_id,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(rows.into_iter().map(|r| EmergencyContact {
        id:           r.id,
        user_id:      r.user_id,
        first_name:   r.first_name,
        last_name:    r.last_name,
        relationship: r.relationship,
        phone_number: r.phone_number,
        email_address: r.email_address,
        is_primary:   r.is_primary,
        created_at:   r.created_at,
    }).collect())
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 4 — ROLE MANAGEMENT (Step 4)
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_roles(
    tx:      &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Vec<RoleGrant>, AppError> {

    let rows = sqlx::query(
        r#"
        SELECT
            ur.user_id, ur.tenant_id,
            ur.role::text       AS role,
            ur.granted_at, ur.granted_by_user_id,
            u.first_name || ' ' || u.last_name AS granted_by_name,
            ur.expires_at, ur.revoked_at
        FROM core.user_roles ur
        LEFT JOIN core.users u ON u.id = ur.granted_by_user_id
        WHERE ur.user_id   = $1
          AND ur.tenant_id = current_setting('app.current_tenant_id', true)::uuid
          AND ur.revoked_at IS NULL
        ORDER BY ur.granted_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    rows.iter().map(|r| -> Result<RoleGrant, AppError> {
        Ok(RoleGrant {
            user_id:            r.try_get("user_id").map_err(AppError::from)?,
            tenant_id:          r.try_get("tenant_id").map_err(AppError::from)?,
            role:               r.try_get("role").map_err(AppError::from)?,
            granted_at:         r.try_get("granted_at").map_err(AppError::from)?,
            granted_by_user_id: r.try_get("granted_by_user_id").map_err(AppError::from)?,
            granted_by_name:    r.try_get("granted_by_name").map_err(AppError::from)?,
            expires_at:         r.try_get("expires_at").map_err(AppError::from)?,
            revoked_at:         r.try_get("revoked_at").map_err(AppError::from)?,
        })
    }).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP 5 — AUDIT LOG (Step 5)
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_audit_logs(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListAuditLogsParams,
) -> Result<(Vec<AuditLogEntry>, i64), AppError> {

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM core.audit_logs
        WHERE tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR target_table     = $1)
          AND ($2::uuid IS NULL OR actor_id         = $2)
          AND ($3::text IS NULL OR action::text      = $3)
          AND ($4::date IS NULL OR created_at::date >= $4)
          AND ($5::date IS NULL OR created_at::date <= $5)
        "#,
        params.table_name.clone() as Option<String>,
        params.actor_id           as Option<Uuid>,
        params.operation.clone()  as Option<String>,
        params.date_from          as Option<NaiveDate>,
        params.date_to            as Option<NaiveDate>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            al.id, al.schema_name, al.target_table,
            al.action::text         AS operation,
            al.actor_id,
            u.first_name || ' ' || u.last_name AS actor_name,
            al.target_record_id, al.old_payload, al.new_payload,
            al.created_at
        FROM core.audit_logs al
        LEFT JOIN core.users u ON u.id = al.actor_id
        WHERE al.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND ($1::text IS NULL OR al.target_table    = $1)
          AND ($2::uuid IS NULL OR al.actor_id        = $2)
          AND ($3::text IS NULL OR al.action::text    = $3)
          AND ($4::date IS NULL OR al.created_at::date >= $4)
          AND ($5::date IS NULL OR al.created_at::date <= $5)
        ORDER BY al.created_at DESC
        LIMIT  $6
        OFFSET $7
        "#,
    )
    .bind(params.table_name.as_deref())
    .bind(params.actor_id)
    .bind(params.operation.as_deref())
    .bind(params.date_from)
    .bind(params.date_to)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let logs = rows.iter().map(|r| -> Result<AuditLogEntry, AppError> {
        Ok(AuditLogEntry {
            id:          r.try_get("id").map_err(AppError::from)?,
            schema_name: r.try_get("schema_name").map_err(AppError::from)?,
            table_name:  r.try_get("table_name").map_err(AppError::from)?,
            operation:   r.try_get("operation").map_err(AppError::from)?,
            actor_id:    r.try_get("actor_id").map_err(AppError::from)?,
            actor_name:  r.try_get("actor_name").map_err(AppError::from)?,
            record_id:   r.try_get("target_record_id").map_err(AppError::from)?,
            old_data:    r.try_get("old_payload").map_err(AppError::from)?,
            new_data:    r.try_get("new_payload").map_err(AppError::from)?,
            created_at:  r.try_get("created_at").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((logs, total))
}

pub async fn get_audit_log(
    tx:  &mut sqlx::Transaction<'_, sqlx::Postgres>,
    log_id: Uuid,
) -> Result<Option<AuditLogEntry>, AppError> {

    let row = sqlx::query(
        r#"
        SELECT
            al.id, al.schema_name, al.target_table,
            al.action::text         AS operation,
            al.actor_id,
            u.first_name || ' ' || u.last_name AS actor_name,
            al.target_record_id, al.old_payload, al.new_payload,
            al.created_at
        FROM core.audit_logs al
        LEFT JOIN core.users u ON u.id = al.actor_id
        WHERE al.id         = $1
          AND al.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
        "#,
    )
    .bind(log_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| AuditLogEntry {
        id:          r.try_get("id").unwrap_or_default(),
        schema_name: r.try_get("schema_name").unwrap_or_default(),
        table_name:  r.try_get("table_name").unwrap_or_default(),
        operation:   r.try_get("operation").unwrap_or_default(),
        actor_id:    r.try_get("actor_id").unwrap_or_default(),
        actor_name:  r.try_get("actor_name").unwrap_or_default(),
        record_id:   r.try_get("record_id").unwrap_or_default(),
        old_data:    r.try_get("old_data").unwrap_or_default(),
        new_data:    r.try_get("new_data").unwrap_or_default(),
        created_at:  r.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
    }))
}
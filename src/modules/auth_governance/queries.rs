// src/modules/auth_governance/queries.rs
//
// All database queries for the auth_governance module.
//
// INET COLUMN STRATEGY
// ─────────────────────
// PostgreSQL INET columns require Option<IpNetwork> when using sqlx::query!
// (the typed macro). Rather than import ipnetwork and parse every IP string
// in Rust, functions that write INET columns use sqlx::query (the untyped
// builder) and pass the IP as Option<&str> with an explicit ::inet cast in
// the SQL. PostgreSQL does the parse; invalid IPs return a DB error which
// maps to AppError::Internal. This is safe because all IP strings come from
// HTTP headers extracted by the server, not from user-supplied JSON.
//
// CUSTOM ENUM STRATEGY
// ─────────────────────
// sqlx::query! cannot bind &str to a PostgreSQL custom ENUM at compile time.
// Functions that pass ENUM values also use sqlx::query with an explicit cast
// (e.g. $3::auth_governance.login_event_outcome). The DB validates the value.
//
// RULE: use sqlx::query! everywhere except functions marked "untyped" below.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AppError;

// ── Credential resolution (pre-auth, SECURITY DEFINER) ───────────────────────

pub struct ResolvedCredentials {
    pub user_id:       Uuid,
    pub password_hash: String,
    pub tenant_id:     Uuid,
    pub system_role:   String,
}

pub async fn resolve_login_credentials(
    pool:     &PgPool,
    username: &str,
    domain:   &str,
) -> Result<Option<ResolvedCredentials>, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT
            user_id       AS "user_id!: Uuid",
            password_hash AS "password_hash!: String",
            tenant_id     AS "tenant_id!: Uuid",
            system_role   AS "system_role!: String"
        FROM core.resolve_login_credentials($1, $2)
        "#,
        username,
        domain,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| ResolvedCredentials {
        user_id:       r.user_id,
        password_hash: r.password_hash,
        tenant_id:     r.tenant_id,
        system_role:   r.system_role,
    }))
}

// ── Account lockout (pre-auth, SECURITY DEFINER) ──────────────────────────────

pub struct LockoutStatus {
    pub is_locked:            bool,
    pub locked_until:         Option<DateTime<Utc>>,
    pub failed_attempt_count: i16,
}

pub async fn check_account_lockout(
    pool:      &PgPool,
    user_id:   Uuid,
    tenant_id: Uuid,
) -> Result<LockoutStatus, AppError> {
    // fetch_optional: the SECURITY DEFINER function returns zero rows when
    // no lockout record exists for this user (first login, clean account).
    // In that case we return the safe default: not locked, zero failures.
    let row = sqlx::query!(
        r#"
        SELECT
            is_locked            AS "is_locked!: bool",
            locked_until         AS "locked_until?: DateTime<Utc>",
            failed_attempt_count AS "failed_attempt_count!: i16"
        FROM core.check_account_lockout($1, $2)
        "#,
        user_id,
        tenant_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    match row {
        Some(r) => Ok(LockoutStatus {
            is_locked:            r.is_locked,
            locked_until:         r.locked_until,
            failed_attempt_count: r.failed_attempt_count,
        }),
        None => Ok(LockoutStatus {
            is_locked:            false,
            locked_until:         None,
            failed_attempt_count: 0,
        }),
    }
}

// UNTYPED — INET param ($3)
pub async fn increment_lockout_counter(
    pool:                  &PgPool,
    user_id:               Uuid,
    tenant_id:             Uuid,
    ip_address:            Option<&str>,
    lockout_threshold:     i16,
    lockout_duration_mins: i16,
) -> Result<(), AppError> {
    sqlx::query(
        "SELECT core.increment_lockout_counter($1, $2, $3::inet, $4, $5)",
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind(ip_address)
    .bind(lockout_threshold)
    .bind(lockout_duration_mins)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

pub async fn reset_lockout_counter(
    pool:      &PgPool,
    user_id:   Uuid,
    tenant_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        "SELECT core.reset_lockout_counter($1, $2)",
        user_id,
        tenant_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

// ── Tenant security policy ────────────────────────────────────────────────────

pub struct TenantSecurityPolicy {
    pub access_token_ttl_secs:   i32,
    pub refresh_token_ttl_secs:  i32,
    pub lockout_threshold:        i16,
    pub lockout_duration_minutes: i16,
    pub mfa_required:             bool,
}

pub async fn get_tenant_security_policy(
    pool:      &PgPool,
    tenant_id: Uuid,
) -> Result<Option<TenantSecurityPolicy>, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT
            access_token_ttl_secs    AS "access_token_ttl_secs!: i32",
            refresh_token_ttl_secs   AS "refresh_token_ttl_secs!: i32",
            lockout_threshold         AS "lockout_threshold!: i16",
            lockout_duration_minutes  AS "lockout_duration_minutes!: i16",
            mfa_required              AS "mfa_required!: bool"
        FROM auth_governance.tenant_security_policy
        WHERE tenant_id = $1
        "#,
        tenant_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|r| TenantSecurityPolicy {
        access_token_ttl_secs:   r.access_token_ttl_secs,
        refresh_token_ttl_secs:  r.refresh_token_ttl_secs,
        lockout_threshold:        r.lockout_threshold,
        lockout_duration_minutes: r.lockout_duration_minutes,
        mfa_required:             r.mfa_required,
    }))
}

// ── Login events (SECURITY DEFINER) ──────────────────────────────────────────
// UNTYPED — custom ENUM param ($3) and INET param ($4)

pub async fn record_login_event(
    pool:       &PgPool,
    tenant_id:  Uuid,
    user_id:    Option<Uuid>,
    outcome:    &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    family_id:  Option<Uuid>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"SELECT core.record_login_event(
            $1, $2,
            $3::auth_governance.login_event_outcome,
            $4::inet,
            $5, $6
        )"#,
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(outcome)
    .bind(ip_address)
    .bind(user_agent)
    .bind(family_id)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

// ── Token family creation ─────────────────────────────────────────────────────
// UNTYPED — INET param ($4 created_ip)
// Takes a transaction so the INSERT runs inside the RLS context set by
// begin_rls_transaction(). Using &PgPool here would grab a different
// connection with no tenant context, causing the WITH CHECK to fail (42501).

pub async fn create_token_family(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id:   Uuid,
    tenant_id: Uuid,
    client_id: Uuid,
    ip:        Option<&str>,
    ua:        Option<&str>,
) -> Result<Uuid, AppError> {
    let row = sqlx::query(
        r#"INSERT INTO auth_governance.token_families
            (user_id, tenant_id, client_id, grant_type, created_ip, created_ua)
        VALUES ($1, $2, $3, 'password', $4::inet, $5)
        RETURNING id::text AS id"#,
    )
    .bind(user_id)
    .bind(tenant_id)
    .bind(client_id)
    .bind(ip)
    .bind(ua)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let id_str: String = row.try_get("id").map_err(AppError::from)?;
    let id = id_str.parse::<Uuid>()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Bad UUID from token_families insert: {}", e)))?;
    Ok(id)
}

// ── Refresh token insertion ───────────────────────────────────────────────────

// Takes a transaction so the INSERT runs inside the RLS context.
pub async fn insert_refresh_token(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    family_id:  Uuid,
    tenant_id:  Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO auth_governance.refresh_tokens
            (family_id, tenant_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        family_id,
        tenant_id,
        token_hash,
        expires_at,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

// ── Token rotation (SECURITY DEFINER) ────────────────────────────────────────
// UNTYPED — INET param ($4 consumed_ip)

pub struct RotatedFamily {
    pub family_id:  Uuid,
    pub tenant_id:  Uuid,
    pub user_id:    Uuid,
}

/// Atomically rotate a refresh token.
///
/// Returns Ok(Some(family)) — valid token, rotation succeeded.
/// Returns Ok(None)         — token already consumed, THEFT DETECTED.
///                            The DB function has already revoked the family.
/// Returns Err              — token not found or expired.
pub async fn rotate_refresh_token(
    pool:           &PgPool,
    old_token_hash: &str,
    new_token_hash: &str,
    new_expires_at: DateTime<Utc>,
    consumed_ip:    Option<&str>,
) -> Result<Option<RotatedFamily>, AppError> {
    // Cast UUIDs to text explicitly — when the SECURITY DEFINER function
    // returns NULL UUIDs, the untyped driver can receive them as empty
    // strings rather than SQL NULL, causing uuid parse errors. Receiving
    // as Option<String> and parsing manually avoids this entirely.
    let row = sqlx::query(
        r#"SELECT
            family_id::text  AS family_id,
            tenant_id::text  AS tenant_id,
            user_id::text    AS user_id,
            was_theft
        FROM core.rotate_refresh_token($1, $2, $3, $4::inet)"#,
    )
    .bind(old_token_hash)
    .bind(new_token_hash)
    .bind(new_expires_at)
    .bind(consumed_ip)
    .fetch_optional(pool)   // fetch_optional: zero rows = token not found
    .await
    .map_err(AppError::from)?;

    // Zero rows = token not found or already expired
    let row = match row {
        Some(r) => r,
        None => return Err(AppError::Unauthorized(
            "Refresh token not found or expired".to_string(),
        )),
    };

    let was_theft: bool             = row.try_get("was_theft").map_err(AppError::from)?;
    let family_id: Option<String>   = row.try_get("family_id").map_err(AppError::from)?;
    let tenant_id: Option<String>   = row.try_get("tenant_id").map_err(AppError::from)?;
    let user_id:   Option<String>   = row.try_get("user_id").map_err(AppError::from)?;

    if was_theft {
        return Ok(None);
    }

    // Parse the text UUIDs — empty string or missing = token not usable
    let parse_uuid = |s: Option<String>, field: &str| -> Result<Option<Uuid>, AppError> {
        match s {
            None => Ok(None),
            Some(v) if v.is_empty() => Ok(None),
            Some(v) => v.parse::<Uuid>().map(Some).map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Bad {} from rotate_refresh_token: {}", field, e))
            }),
        }
    };

    let family_id = parse_uuid(family_id, "family_id")?;
    let tenant_id = parse_uuid(tenant_id, "tenant_id")?;
    let user_id   = parse_uuid(user_id,   "user_id")?;

    match (family_id, tenant_id, user_id) {
        (Some(fid), Some(tid), Some(uid)) => Ok(Some(RotatedFamily {
            family_id: fid,
            tenant_id: tid,
            user_id:   uid,
        })),
        _ => Err(AppError::Unauthorized(
            "Refresh token not found or expired".to_string(),
        )),
    }
}

// ── Token family revocation ───────────────────────────────────────────────────

pub async fn revoke_token_family(
    tx:            &mut sqlx::Transaction<'_, sqlx::Postgres>,
    family_id:     Uuid,
    revoke_reason: &str,
) -> Result<(), AppError> {
    // WHERE status != 'revoked' makes this idempotent — calling revoke on an
    // already-revoked family is a no-op rather than an error, so logout can
    // be called multiple times safely.
    sqlx::query!(
        r#"
        UPDATE auth_governance.token_families
        SET status        = 'revoked',
            revoked_at    = now(),
            revoke_reason = $2,
            updated_at    = now()
        WHERE id     = $1
          AND status != 'revoked'
        "#,
        family_id,
        revoke_reason,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    sqlx::query!(
        r#"
        UPDATE auth_governance.refresh_tokens
        SET consumed_at  = now(),
            is_valid_use = false
        WHERE family_id     = $1
          AND consumed_at IS NULL
        "#,
        family_id,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(())
}

// ── last_accessed_at update ───────────────────────────────────────────────────

pub async fn touch_last_accessed(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id:   Uuid,
    tenant_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        UPDATE core.tenant_memberships
        SET last_accessed_at = now()
        WHERE user_id   = $1
          AND tenant_id = $2
        "#,
        user_id,
        tenant_id,
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;
    Ok(())
}
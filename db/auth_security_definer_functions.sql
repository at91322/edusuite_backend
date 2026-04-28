-- =============================================================================
-- eduSuite :: auth_security_definer_functions.sql
-- =============================================================================
-- Run as postgres (superuser) in pgAdmin.
-- Safe to re-run: all functions use CREATE OR REPLACE.
--
-- These functions are called by the Rust auth service to perform operations
-- that must bypass RLS because they occur before (or during failure of) the
-- login flow -- before a tenant context can be established.
--
-- All functions:
--   - Are SECURITY DEFINER (run as schema owner, bypass RLS)
--   - Have search_path pinned to prevent schema injection attacks
--   - Are REVOKE'd from PUBLIC and GRANT'd only to edusuite_app
--   - Return the minimum data needed
--
-- REVISION HISTORY
-- v1 -- initial implementation
-- v2 -- rotate_refresh_token: qualified all column refs with table aliases
--        to resolve "column reference family_id is ambiguous"
-- v3 -- rotate_refresh_token: initial lookup JOINs token_families with
--        tf.status = 'active' so hash collisions against consumed tokens
--        in revoked families do not false-trigger theft detection
-- =============================================================================

BEGIN;

-- =============================================================================
-- 1. check_account_lockout
-- =============================================================================
CREATE OR REPLACE FUNCTION core.check_account_lockout(
    p_user_id   uuid,
    p_tenant_id uuid
)
RETURNS TABLE (
    is_locked            boolean,
    locked_until         timestamptz,
    failed_attempt_count smallint
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = auth_governance, core, public
AS $$
BEGIN
    RETURN QUERY
    SELECT
        CASE
            WHEN al.locked_until IS NOT NULL AND al.locked_until > now()
            THEN true
            ELSE false
        END                     AS is_locked,
        al.locked_until,
        al.failed_attempt_count
    FROM auth_governance.account_lockouts al
    WHERE al.user_id   = p_user_id
      AND al.tenant_id = p_tenant_id;

    IF NOT FOUND THEN
        RETURN QUERY SELECT false::boolean, NULL::timestamptz, 0::smallint;
    END IF;
END;
$$;

REVOKE EXECUTE ON FUNCTION core.check_account_lockout(uuid, uuid) FROM PUBLIC;
GRANT  EXECUTE ON FUNCTION core.check_account_lockout(uuid, uuid) TO edusuite_app;

-- =============================================================================
-- 2. increment_lockout_counter
-- =============================================================================
CREATE OR REPLACE FUNCTION core.increment_lockout_counter(
    p_user_id               uuid,
    p_tenant_id             uuid,
    p_ip                    inet,
    p_lockout_threshold     smallint,
    p_lockout_duration_mins smallint
)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = auth_governance, core, public
AS $$
DECLARE
    v_new_count smallint;
BEGIN
    INSERT INTO auth_governance.account_lockouts
        (user_id, tenant_id, failed_attempt_count, last_failed_at, last_failed_ip)
    VALUES
        (p_user_id, p_tenant_id, 1, now(), p_ip)
    ON CONFLICT (user_id, tenant_id) DO UPDATE
        SET failed_attempt_count = account_lockouts.failed_attempt_count + 1,
            last_failed_at       = now(),
            last_failed_ip       = p_ip,
            updated_at           = now()
    RETURNING failed_attempt_count INTO v_new_count;

    IF v_new_count >= p_lockout_threshold THEN
        UPDATE auth_governance.account_lockouts
        SET locked_until = now() + (p_lockout_duration_mins || ' minutes')::interval,
            updated_at   = now()
        WHERE user_id   = p_user_id
          AND tenant_id = p_tenant_id;
    END IF;
END;
$$;

REVOKE EXECUTE ON FUNCTION core.increment_lockout_counter(uuid, uuid, inet, smallint, smallint) FROM PUBLIC;
GRANT  EXECUTE ON FUNCTION core.increment_lockout_counter(uuid, uuid, inet, smallint, smallint) TO edusuite_app;

-- =============================================================================
-- 3. reset_lockout_counter
-- =============================================================================
CREATE OR REPLACE FUNCTION core.reset_lockout_counter(
    p_user_id   uuid,
    p_tenant_id uuid
)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = auth_governance, core, public
AS $$
BEGIN
    UPDATE auth_governance.account_lockouts
    SET failed_attempt_count = 0,
        locked_until         = NULL,
        last_reset_at        = now(),
        updated_at           = now()
    WHERE user_id   = p_user_id
      AND tenant_id = p_tenant_id;
END;
$$;

REVOKE EXECUTE ON FUNCTION core.reset_lockout_counter(uuid, uuid) FROM PUBLIC;
GRANT  EXECUTE ON FUNCTION core.reset_lockout_counter(uuid, uuid) TO edusuite_app;

-- =============================================================================
-- 4. record_login_event
-- =============================================================================
CREATE OR REPLACE FUNCTION core.record_login_event(
    p_tenant_id  uuid,
    p_user_id    uuid,
    p_outcome    auth_governance.login_event_outcome,
    p_ip         inet,
    p_user_agent text,
    p_family_id  uuid
)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = auth_governance, core, public
AS $$
BEGIN
    INSERT INTO auth_governance.login_events
        (tenant_id, user_id, outcome, ip_address, user_agent, family_id, grant_type)
    VALUES
        (p_tenant_id, p_user_id, p_outcome, p_ip, p_user_agent, p_family_id,
         CASE WHEN p_family_id IS NOT NULL
              THEN 'password'::auth_governance.session_grant_type
              ELSE NULL
         END);
END;
$$;

REVOKE EXECUTE ON FUNCTION
    core.record_login_event(uuid, uuid, auth_governance.login_event_outcome, inet, text, uuid)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION
    core.record_login_event(uuid, uuid, auth_governance.login_event_outcome, inet, text, uuid)
    TO edusuite_app;

-- =============================================================================
-- 5. rotate_refresh_token  (v3)
-- =============================================================================
CREATE OR REPLACE FUNCTION core.rotate_refresh_token(
    p_old_hash    varchar(64),
    p_new_hash    varchar(64),
    p_new_expires timestamptz,
    p_consumed_ip inet
)
RETURNS TABLE (
    family_id uuid,
    tenant_id uuid,
    user_id   uuid,
    was_theft boolean
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = auth_governance, core, public
AS $$
DECLARE
    v_token  auth_governance.refresh_tokens%ROWTYPE;
    v_family auth_governance.token_families%ROWTYPE;
BEGIN
    -- Join on tf.status = 'active' so hash collisions against tokens in
    -- revoked families (e.g. after logout + re-login) cannot false-trigger
    -- theft detection. All columns qualified with rt./tf. to avoid ambiguity
    -- with the RETURNS TABLE output column names.
    SELECT rt.* INTO v_token
    FROM auth_governance.refresh_tokens rt
    JOIN auth_governance.token_families tf ON tf.id = rt.family_id
    WHERE rt.token_hash = p_old_hash
      AND rt.expires_at > now()
      AND tf.status     = 'active'
    FOR UPDATE OF rt;

    IF NOT FOUND THEN
        RETURN;
    END IF;

    -- Theft detection: token in active family but already consumed
    IF v_token.consumed_at IS NOT NULL THEN
        SELECT * INTO v_family
        FROM auth_governance.token_families tf
        WHERE tf.id = v_token.family_id;

        UPDATE auth_governance.token_families tf
        SET status        = 'revoked',
            revoked_at    = now(),
            revoke_reason = 'token_theft_detected',
            updated_at    = now()
        WHERE tf.id = v_token.family_id;

        UPDATE auth_governance.refresh_tokens rt
        SET consumed_at  = now(),
            is_valid_use = false
        WHERE rt.family_id  = v_token.family_id
          AND rt.consumed_at IS NULL;

        RETURN QUERY SELECT
            v_family.id        AS family_id,
            v_family.tenant_id AS tenant_id,
            v_family.user_id   AS user_id,
            true::boolean      AS was_theft;
        RETURN;
    END IF;

    -- Valid rotation
    SELECT * INTO v_family
    FROM auth_governance.token_families tf
    WHERE tf.id     = v_token.family_id
      AND tf.status = 'active';

    IF NOT FOUND THEN
        RETURN;
    END IF;

    UPDATE auth_governance.refresh_tokens rt
    SET consumed_at  = now(),
        consumed_ip  = p_consumed_ip,
        is_valid_use = true
    WHERE rt.id = v_token.id;

    UPDATE auth_governance.token_families tf
    SET last_rotated_at = now(),
        updated_at      = now()
    WHERE tf.id = v_family.id;

    INSERT INTO auth_governance.refresh_tokens
        (family_id, tenant_id, token_hash, expires_at)
    VALUES
        (v_family.id, v_family.tenant_id, p_new_hash, p_new_expires);

    RETURN QUERY SELECT
        v_family.id        AS family_id,
        v_family.tenant_id AS tenant_id,
        v_family.user_id   AS user_id,
        false::boolean     AS was_theft;
END;
$$;

REVOKE EXECUTE ON FUNCTION core.rotate_refresh_token(varchar, varchar, timestamptz, inet) FROM PUBLIC;
GRANT  EXECUTE ON FUNCTION core.rotate_refresh_token(varchar, varchar, timestamptz, inet) TO edusuite_app;

COMMIT;

-- Verify all five functions are SECURITY DEFINER
SELECT routine_name, security_type
FROM information_schema.routines
WHERE routine_schema = 'core'
  AND routine_name IN (
      'check_account_lockout', 'increment_lockout_counter',
      'reset_lockout_counter', 'record_login_event', 'rotate_refresh_token'
  )
ORDER BY routine_name;
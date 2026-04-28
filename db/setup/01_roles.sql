-- =============================================================================
-- eduSuite :: 01_roles.sql
-- =============================================================================
-- Creates the application role and grants all necessary privileges.
-- Run as postgres (superuser) BEFORE applying the schema.
--
-- Idempotent: DO blocks skip gracefully if the role already exists.
-- =============================================================================

-- ── Create application role ───────────────────────────────────────────────────
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'edusuite_app') THEN
        CREATE ROLE edusuite_app WITH LOGIN PASSWORD 'change_in_production';
        RAISE NOTICE 'Role edusuite_app created.';
    ELSE
        RAISE NOTICE 'Role edusuite_app already exists — skipping.';
    END IF;
END $$;

-- ── Schema usage ──────────────────────────────────────────────────────────────
GRANT USAGE ON SCHEMA
    auth_governance, board, catalog, cms, collab, comms, core,
    crm, data_governance, dms, event_bus, finance, hr, lms,
    ops, policy, public, reference, sis, workflow
TO edusuite_app;

-- ── Table privileges (applied to existing tables) ─────────────────────────────
-- Run after the schema is applied. If tables are created later, the
-- ALTER DEFAULT PRIVILEGES block below handles them automatically.
DO $$
DECLARE
    s text;
BEGIN
    FOR s IN SELECT unnest(ARRAY[
        'auth_governance','board','catalog','cms','collab','comms','core',
        'crm','data_governance','dms','event_bus','finance','hr','lms',
        'ops','policy','reference','sis','workflow'
    ]) LOOP
        EXECUTE format(
            'GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA %I TO edusuite_app', s
        );
        EXECUTE format(
            'GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA %I TO edusuite_app', s
        );
        EXECUTE format(
            'GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA %I TO edusuite_app', s
        );
    END LOOP;
END $$;

-- ── Default privileges (applies to future tables) ─────────────────────────────
DO $$
DECLARE
    s text;
BEGIN
    FOR s IN SELECT unnest(ARRAY[
        'auth_governance','board','catalog','cms','collab','comms','core',
        'crm','data_governance','dms','event_bus','finance','hr','lms',
        'ops','policy','reference','sis','workflow'
    ]) LOOP
        EXECUTE format(
            'ALTER DEFAULT PRIVILEGES FOR ROLE postgres IN SCHEMA %I
             GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO edusuite_app', s
        );
        EXECUTE format(
            'ALTER DEFAULT PRIVILEGES FOR ROLE postgres IN SCHEMA %I
             GRANT USAGE, SELECT ON SEQUENCES TO edusuite_app', s
        );
    END LOOP;
END $$;

-- ── Security definer function grants ─────────────────────────────────────────
-- These are created by auth_security_definer_functions.sql and must be
-- explicitly granted since SECURITY DEFINER functions are REVOKE'd from PUBLIC.
-- Re-run this after applying auth_security_definer_functions.sql if needed.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
               WHERE n.nspname = 'core' AND p.proname = 'check_account_lockout') THEN
        GRANT EXECUTE ON FUNCTION core.check_account_lockout(uuid, uuid)                              TO edusuite_app;
        GRANT EXECUTE ON FUNCTION core.increment_lockout_counter(uuid, uuid, inet, smallint, smallint) TO edusuite_app;
        GRANT EXECUTE ON FUNCTION core.reset_lockout_counter(uuid, uuid)                              TO edusuite_app;
        GRANT EXECUTE ON FUNCTION core.resolve_login_credentials(text, text)                          TO edusuite_app;
        RAISE NOTICE 'Security definer grants applied.';
    ELSE
        RAISE NOTICE 'Security definer functions not yet created — run auth_security_definer_functions.sql first.';
    END IF;
END $$;
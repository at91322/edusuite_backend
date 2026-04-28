-- =============================================================================
-- eduSuite :: 02_guc_defaults.sql
-- =============================================================================
-- Sets sentinel UUID defaults for the RLS GUC variables at the database level.
-- Run as postgres (superuser).
--
-- WHY THIS IS NECESSARY
-- ──────────────────────
-- Every RLS policy on tenant-scoped tables evaluates:
--     tenant_id = current_setting('app.current_tenant_id', true)::uuid
--
-- The is_missing=true flag (second argument) means current_setting returns
-- an empty string '' rather than NULL when the variable is not set.
-- Casting '' to uuid throws:
--     ERROR: invalid input syntax for type uuid: ""
--
-- This affects any query on an RLS-protected table running on a pool
-- connection that hasn't had set_config() called yet (e.g. pre-auth queries).
--
-- The sentinel value '00000000-0000-0000-0000-000000000000' is not a valid
-- tenant UUID, so RLS correctly returns zero rows on unscoped connections
-- rather than throwing an error.
--
-- The application overrides these with set_config(key, value, true) inside
-- every authenticated transaction, scoped to that transaction only.
--
-- RECONNECT REQUIRED
-- ───────────────────
-- Existing connections will not see these defaults. Restart the Rust server
-- after running this script to ensure all pool connections pick them up.
-- =============================================================================

ALTER DATABASE edusuite SET "app.current_tenant_id" = '00000000-0000-0000-0000-000000000000';
ALTER DATABASE edusuite SET "app.current_user_id"   = '00000000-0000-0000-0000-000000000000';
ALTER DATABASE edusuite SET "app.current_service"   = 'none';

-- Verify
SELECT unnest(setconfig) AS config
FROM pg_db_role_setting drs
JOIN pg_database db ON db.oid = drs.setdatabase
WHERE db.datname = current_database()
  AND drs.setrole = 0;
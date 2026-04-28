# eduSuite Database Setup

Complete setup instructions for a fresh development environment.
All scripts in this folder must be run as the `postgres` superuser in pgAdmin
or via `psql -U postgres`.

---

## Prerequisites

- PostgreSQL 15+ with the following extensions installed in the target database:
  - `uuid-ossp` (`CREATE EXTENSION IF NOT EXISTS "uuid-ossp";`)
  - `postgis` (for spatial data in ops schema)
  - `pg_trgm` (for full-text search indexes)
- A database named `edusuite` (`CREATE DATABASE edusuite;`)

---

## Step-by-step setup

Run these in order. Each step depends on the previous one.

### Step 1 — Apply the schema

Run the main schema dump in pgAdmin as `postgres`:

```
db/edusuite_schema_backup_<date>.sql
```

This creates all 19 schemas, ~300 tables, types, indexes, triggers, and
the `_sqlx_migrations` tracking table.

### Step 2 — Create role and grant privileges

```
db/setup/01_roles.sql
```

Creates `edusuite_app` with `LOGIN` privilege and grants `SELECT`, `INSERT`,
`UPDATE`, `DELETE` on all tables across all schemas. Also sets `ALTER DEFAULT
PRIVILEGES` so future tables are automatically accessible.

> **Note:** After creating the role, update your `.env` file:
> ```
> DATABASE_URL=postgres://edusuite_app:change_in_production@localhost:5432/edusuite
> ```

### Step 3 — Apply security definer functions

```
db/auth_security_definer_functions.sql
```

Creates the five SECURITY DEFINER functions used by the auth layer:
`check_account_lockout`, `increment_lockout_counter`, `reset_lockout_counter`,
`record_login_event`, `rotate_refresh_token`. Each is `REVOKE`'d from PUBLIC
and `GRANT`'d only to `edusuite_app`.

### Step 4 — Set GUC sentinel defaults

```
db/setup/02_guc_defaults.sql
```

Sets `app.current_tenant_id`, `app.current_user_id`, and `app.current_service`
to safe sentinel values at the database level. Required to prevent
`invalid input syntax for type uuid: ""` errors on unscoped pool connections.

**Restart the Rust server after this step** so pool connections pick up the
new defaults.

### Step 5 — Apply RLS permissive policies

```
db/setup/03_rls_permissive_policies.sql
```

Adds `CREATE POLICY app_access FOR ALL TO edusuite_app USING (true) WITH CHECK (true)`
to all 359 RLS-enabled tables across the 17 non-auth schemas. PostgreSQL
requires at least one permissive policy per table — the restrictive
`tenant_isolation_policy` alone denies everything.

`core` and `auth_governance` policies were created separately and are
excluded from this file (already present in the live database).

### Step 6 — Seed development data

```
db/setup/04_seed_data.sql
```

Inserts 17 tenants, 17 admin users, and 17 tenant memberships with
sequential UUIDs for easy identification. All users share the password
`password` (Argon2id hash).

**Do not run in production.**

| Username | Domain | Password |
|---|---|---|
| admin_springfield | springfield.edu | password |
| admin_southpark | southpark.edu | password |
| admin_hogwarts | hogwarts.edu | password |
| admin_prek | sunshineprek.com | password |
| admin_k12 | lincolnk12.edu | password |
| admin_k8 | oakridgek8.edu | password |
| admin_6_8 | washingtonms.edu | password |
| admin_9_12 | roosevelths.edu | password |
| admin_6_9 | jeffersonjh.edu | password |
| admin_10_12 | kennedysrh.edu | password |
| admin_cc | riversidecc.edu | password |
| admin_state_u | stateu.edu | password |
| admin_tech_u | techu.edu | password |
| admin_elite_u | eliteu.edu | password |
| admin_liberal | heritageliberal.edu | password |
| admin_law | prestigelaw.edu | password |
| admin_med | ghmed.edu | password |

---

## Quick verification

After all six steps, run this in pgAdmin to confirm everything is in place:

```sql
-- Role exists
SELECT rolname, rolcanlogin FROM pg_roles WHERE rolname = 'edusuite_app';

-- GUC defaults set
SELECT unnest(setconfig) AS config
FROM pg_db_role_setting drs
JOIN pg_database db ON db.oid = drs.setdatabase
WHERE db.datname = 'edusuite' AND drs.setrole = 0;

-- Permissive policies applied (expect 370+)
SELECT COUNT(*) FROM pg_policy WHERE polname = 'app_access';

-- Seed data present
SELECT 'tenants' AS tbl, COUNT(*) FROM core.tenants
UNION ALL
SELECT 'users', COUNT(*) FROM core.users
UNION ALL
SELECT 'memberships', COUNT(*) FROM core.tenant_memberships
UNION ALL
SELECT 'oauth_clients', COUNT(*) FROM auth_governance.oauth_clients;
```

Expected results: role exists with `rolcanlogin = true`, three GUC entries,
370+ permissive policies, 17/17/17/1 row counts.

---

## Rust application setup

After the database is ready:

1. Copy `.env.example` to `.env` and fill in `DATABASE_URL` and `JWT_SECRET`
2. `cargo run` — sqlx migrations in `migrations/` run automatically on startup
3. Verify with `GET http://localhost:8081/health`

---

## Notes on migration vs setup scripts

| Location | Role | Purpose |
|---|---|---|
| `migrations/*.sql` | `edusuite_app` | Schema evolution (tables, indexes) |
| `db/setup/*.sql` | `postgres` | One-time superuser setup (roles, RLS, GUCs) |

sqlx runs `migrations/` automatically on `cargo run`. The `db/setup/` scripts
require manual execution because they need superuser privileges that
`edusuite_app` does not have.
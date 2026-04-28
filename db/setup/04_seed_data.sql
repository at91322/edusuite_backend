-- =============================================================================
-- eduSuite :: 04_seed_data.sql
-- =============================================================================
-- Development seed data: 17 tenants, 17 admin users, 17 memberships.
-- All users share the same password: 'password' (Argon2id hash below).
-- Run as postgres (superuser) after applying the schema.
--
-- Idempotent: ON CONFLICT DO NOTHING skips rows that already exist.
-- Safe to re-run on a database that was partially seeded.
--
-- DO NOT USE IN PRODUCTION. These are fixed UUIDs and a known password.
--
-- BE SURE TO SEED A ROW IN TENANT_MEMBERSHIPS FOR EACH SEED USER OTHERWISE
-- THEY WILL BE INVISIBLE TO THE core.users TABLE RLS POLICY
-- =============================================================================

BEGIN;

-- ── Set audit context ─────────────────────────────────────────────────────────
-- The write_audit_log() trigger fires on core.users INSERT and reads
-- app.current_tenant_id / app.current_user_id from the session. Without
-- these being set, tenant_id in audit_logs would be NULL (NOT NULL violation).
-- We use a sentinel "system" actor UUID and no specific tenant for seed inserts
-- since this data spans all tenants.
-- Using postgres superuser bypasses RLS; the audit trigger still fires.
SELECT set_config('app.current_tenant_id', '00000000-0000-0000-0000-000000000000', true);
SELECT set_config('app.current_user_id',   '00000000-0000-0000-0000-000000000000', true);
SELECT set_config('app.current_service',   'seed-script', true);

-- ── oauth_clients: platform client (required for token_families FK) ───────────
INSERT INTO auth_governance.oauth_clients
    (id, client_name, client_type, grant_types, tenant_id)
VALUES
    ('00000000-0000-0000-0000-000000000001',
     'eduSuite Web Application',
     'confidential',
     ARRAY['password']::auth_governance.grant_type[],
     NULL)
ON CONFLICT (id) DO NOTHING;

-- ── Tenants ───────────────────────────────────────────────────────────────────
INSERT INTO core.tenants (id, name, domain) VALUES
    ('a0000000-0000-0000-0000-000000000001', 'Springfield Elementary', 'springfield.edu'),
    ('a0000000-0000-0000-0000-000000000002', 'South Park Elementary', 'southpark.edu'),
    ('a0000000-0000-0000-0000-000000000003', 'Hogwarts School', 'hogwarts.edu'),
    ('a0000000-0000-0000-0000-000000000004', 'Sunshine Pre-K', 'sunshineprek.com'),
    ('a0000000-0000-0000-0000-000000000005', 'Lincoln K-12 District', 'lincolnk12.edu'),
    ('a0000000-0000-0000-0000-000000000006', 'Oakridge K-8 Academy', 'oakridgek8.edu'),
    ('a0000000-0000-0000-0000-000000000007', 'Washington Middle School', 'washingtonms.edu'),
    ('a0000000-0000-0000-0000-000000000008', 'Roosevelt High School', 'roosevelths.edu'),
    ('a0000000-0000-0000-0000-000000000009', 'Jefferson Jr High', 'jeffersonjh.edu'),
    ('a0000000-0000-0000-0000-000000000010', 'Kennedy Sr High', 'kennedysrh.edu'),
    ('a0000000-0000-0000-0000-000000000011', 'Riverside Community College', 'riversidecc.edu'),
    ('a0000000-0000-0000-0000-000000000012', 'State University', 'stateu.edu'),
    ('a0000000-0000-0000-0000-000000000013', 'Tech University', 'techu.edu'),
    ('a0000000-0000-0000-0000-000000000014', 'Elite Private University', 'eliteu.edu'),
    ('a0000000-0000-0000-0000-000000000015', 'Heritage Liberal Arts', 'heritageliberal.edu'),
    ('a0000000-0000-0000-0000-000000000016', 'Prestige Law', 'prestigelaw.edu'),
    ('a0000000-0000-0000-0000-000000000017', 'General Hospital Med', 'ghmed.edu')
ON CONFLICT (id) DO NOTHING;

-- ── Users (password: 'password') ────────────────────────────────────────────
INSERT INTO core.users (id, username, password_hash, first_name, last_name, is_active) VALUES
    ('b0000000-0000-0000-0000-000000000001', 'admin_springfield', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'Seymour', 'Skinner', true),
    ('b0000000-0000-0000-0000-000000000002', 'admin_southpark', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'Peter', 'Victoria', true),
    ('b0000000-0000-0000-0000-000000000003', 'admin_hogwarts', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'Albus', 'Dumbledore', true),
    ('b0000000-0000-0000-0000-000000000004', 'admin_prek', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'PreK', 'Director', true),
    ('b0000000-0000-0000-0000-000000000005', 'admin_k12', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'K12', 'Superintendent', true),
    ('b0000000-0000-0000-0000-000000000006', 'admin_k8', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'K8', 'Headmaster', true),
    ('b0000000-0000-0000-0000-000000000007', 'admin_6_8', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'Middle', 'Principal', true),
    ('b0000000-0000-0000-0000-000000000008', 'admin_9_12', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'High', 'Principal', true),
    ('b0000000-0000-0000-0000-000000000009', 'admin_6_9', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'JrHigh', 'Principal', true),
    ('b0000000-0000-0000-0000-000000000010', 'admin_10_12', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'SrHigh', 'Principal', true),
    ('b0000000-0000-0000-0000-000000000011', 'admin_cc', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'CC', 'President', true),
    ('b0000000-0000-0000-0000-000000000012', 'admin_state_u', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'StateU', 'Chancellor', true),
    ('b0000000-0000-0000-0000-000000000013', 'admin_tech_u', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'TechU', 'President', true),
    ('b0000000-0000-0000-0000-000000000014', 'admin_elite_u', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'EliteU', 'Provost', true),
    ('b0000000-0000-0000-0000-000000000015', 'admin_liberal', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'LiberalArts', 'Dean', true),
    ('b0000000-0000-0000-0000-000000000016', 'admin_law', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'Law', 'Dean', true),
    ('b0000000-0000-0000-0000-000000000017', 'admin_med', $pw$$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQh$DqHGwv6NQV0VcaJi7jeF1E8IpfMXmXcpq4r2kKyqpXk$pw$, 'Med', 'Dean', true)
ON CONFLICT (id) DO NOTHING;

-- ── Tenant memberships ───────────────────────────────────────────────────────
-- This is vitally important, every seeded user needs a tenant_memberships row or they'll be invisible under core.users RLS
INSERT INTO core.tenant_memberships (id, tenant_id, user_id, system_role) VALUES
    ('c0000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'b0000000-0000-0000-0000-000000000001', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000002', 'a0000000-0000-0000-0000-000000000002', 'b0000000-0000-0000-0000-000000000002', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000003', 'a0000000-0000-0000-0000-000000000003', 'b0000000-0000-0000-0000-000000000003', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000004', 'a0000000-0000-0000-0000-000000000004', 'b0000000-0000-0000-0000-000000000004', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000005', 'a0000000-0000-0000-0000-000000000005', 'b0000000-0000-0000-0000-000000000005', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000006', 'a0000000-0000-0000-0000-000000000006', 'b0000000-0000-0000-0000-000000000006', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000007', 'a0000000-0000-0000-0000-000000000007', 'b0000000-0000-0000-0000-000000000007', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000008', 'a0000000-0000-0000-0000-000000000008', 'b0000000-0000-0000-0000-000000000008', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000009', 'a0000000-0000-0000-0000-000000000009', 'b0000000-0000-0000-0000-000000000009', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000010', 'a0000000-0000-0000-0000-000000000010', 'b0000000-0000-0000-0000-000000000010', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000011', 'a0000000-0000-0000-0000-000000000011', 'b0000000-0000-0000-0000-000000000011', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000012', 'a0000000-0000-0000-0000-000000000012', 'b0000000-0000-0000-0000-000000000012', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000013', 'a0000000-0000-0000-0000-000000000013', 'b0000000-0000-0000-0000-000000000013', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000014', 'a0000000-0000-0000-0000-000000000014', 'b0000000-0000-0000-0000-000000000014', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000015', 'a0000000-0000-0000-0000-000000000015', 'b0000000-0000-0000-0000-000000000015', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000016', 'a0000000-0000-0000-0000-000000000016', 'b0000000-0000-0000-0000-000000000016', 'tenant_admin'::core.system_role),
    ('c0000000-0000-0000-0000-000000000017', 'a0000000-0000-0000-0000-000000000017', 'b0000000-0000-0000-0000-000000000017', 'tenant_admin'::core.system_role)
ON CONFLICT (id) DO NOTHING;

COMMIT;

-- Verify
SELECT 'tenants'            AS tbl, COUNT(*) FROM core.tenants
UNION ALL
SELECT 'users'              AS tbl, COUNT(*) FROM core.users
UNION ALL
SELECT 'tenant_memberships' AS tbl, COUNT(*) FROM core.tenant_memberships
UNION ALL
SELECT 'oauth_clients'      AS tbl, COUNT(*) FROM auth_governance.oauth_clients;
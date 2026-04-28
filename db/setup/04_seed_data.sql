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

-- Disable audit triggers during seed inserts.
-- write_audit_log() requires valid FK references we don't have during seeding.
ALTER TABLE core.users              DISABLE TRIGGER ALL;
ALTER TABLE core.tenant_memberships DISABLE TRIGGER ALL;
ALTER TABLE auth_governance.oauth_clients DISABLE TRIGGER ALL;
ALTER TABLE core.departments        DISABLE TRIGGER ALL;
ALTER TABLE hr.staff_profiles       DISABLE TRIGGER ALL;
ALTER TABLE hr.employment_contracts DISABLE TRIGGER ALL;
ALTER TABLE sis.student_profiles    DISABLE TRIGGER ALL;
ALTER TABLE sis.academic_terms      DISABLE TRIGGER ALL;
ALTER TABLE sis.courses             DISABLE TRIGGER ALL;
ALTER TABLE sis.course_sections     DISABLE TRIGGER ALL;
ALTER TABLE sis.enrollments         DISABLE TRIGGER ALL;

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

-- =============================================================================
-- DEPARTMENTS
-- =============================================================================
INSERT INTO core.departments (id, tenant_id, code, name, head_user_id) VALUES
    ('f0000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003','MAGI','Magical Arts','b0000000-0000-0000-0000-000000000003'),
    ('f0000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003','DFDA','Defence Against the Dark Arts','b0000000-0000-0000-0000-000000000003'),
    ('f0000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000003','POT','Potions and Sciences','b0000000-0000-0000-0000-000000000003'),
    ('f0000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000002','GEN','General Education','b0000000-0000-0000-0000-000000000002'),
    ('f0000000-0000-0000-0000-000000000005','a0000000-0000-0000-0000-000000000002','SPED','Special Education','b0000000-0000-0000-0000-000000000002')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- STAFF + STUDENT USERS (password: 'password')
-- =============================================================================
INSERT INTO core.users (id, username, password_hash, first_name, last_name, is_active) VALUES
    ('f1000000-0000-0000-0000-000000000001','albus.dumbledore','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Albus','Dumbledore',true),
    ('f1000000-0000-0000-0000-000000000002','minerva.mcgonagall','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Minerva','McGonagall',true),
    ('f1000000-0000-0000-0000-000000000003','severus.snape','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Severus','Snape',true),
    ('f1000000-0000-0000-0000-000000000004','remus.lupin','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Remus','Lupin',true),
    ('f1000000-0000-0000-0000-000000000005','herbert.garrison','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Herbert','Garrison',true),
    ('f1000000-0000-0000-0000-000000000006','pc.principal','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','PC','Principal',true),
    ('f2000000-0000-0000-0000-000000000001','ron.weasley','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Ron','Weasley',true),
    ('f2000000-0000-0000-0000-000000000002','neville.longbottom','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Neville','Longbottom',true),
    ('f2000000-0000-0000-0000-000000000003','luna.lovegood','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Luna','Lovegood',true),
    ('f2000000-0000-0000-0000-000000000004','eric.cartman','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Eric','Cartman',true),
    ('f2000000-0000-0000-0000-000000000005','kyle.broflovski','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Kyle','Broflovski',true),
    ('f2000000-0000-0000-0000-000000000006','stan.marsh','$argon2id$v=19$m=19456,t=2,p=1$YmFkIHNhbHQgZG9uJ3QgdXNl$QKb2tB3yHEPzMG7rGolpJNhfG7GhGGvIqzNFzqJVgMk','Stan','Marsh',true)
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- TENANT MEMBERSHIPS
-- Every user needs a membership or core.users RLS hides them.
-- =============================================================================
INSERT INTO core.tenant_memberships (id, tenant_id, user_id, system_role) VALUES
    ('f6000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000001','tenant_admin'),
    ('f6000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000002','faculty'),
    ('f6000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000003','faculty'),
    ('f6000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000004','faculty'),
    ('f6000000-0000-0000-0000-000000000005','a0000000-0000-0000-0000-000000000002','f1000000-0000-0000-0000-000000000005','faculty'),
    ('f6000000-0000-0000-0000-000000000006','a0000000-0000-0000-0000-000000000002','f1000000-0000-0000-0000-000000000006','staff'),
    ('f6000000-0000-0000-0000-000000000007','a0000000-0000-0000-0000-000000000003','f2000000-0000-0000-0000-000000000001','student'),
    ('f6000000-0000-0000-0000-000000000008','a0000000-0000-0000-0000-000000000003','f2000000-0000-0000-0000-000000000002','student'),
    ('f6000000-0000-0000-0000-000000000009','a0000000-0000-0000-0000-000000000003','f2000000-0000-0000-0000-000000000003','student'),
    ('f6000000-0000-0000-0000-000000000010','a0000000-0000-0000-0000-000000000002','f2000000-0000-0000-0000-000000000004','student'),
    ('f6000000-0000-0000-0000-000000000011','a0000000-0000-0000-0000-000000000002','f2000000-0000-0000-0000-000000000005','student'),
    ('f6000000-0000-0000-0000-000000000012','a0000000-0000-0000-0000-000000000002','f2000000-0000-0000-0000-000000000006','student')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- HR: STAFF PROFILES
-- =============================================================================
INSERT INTO hr.staff_profiles (user_id, tenant_id, primary_department_id, hire_date, is_tenured) VALUES
    ('f1000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003','f0000000-0000-0000-0000-000000000001','1956-09-01',true),
    ('f1000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003','f0000000-0000-0000-0000-000000000001','1956-09-01',true),
    ('f1000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000003','f0000000-0000-0000-0000-000000000003','1981-09-01',true),
    ('f1000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000003','f0000000-0000-0000-0000-000000000002','1993-09-01',false),
    ('f1000000-0000-0000-0000-000000000005','a0000000-0000-0000-0000-000000000002','f0000000-0000-0000-0000-000000000004','1997-09-01',false),
    ('f1000000-0000-0000-0000-000000000006','a0000000-0000-0000-0000-000000000002','f0000000-0000-0000-0000-000000000004','2014-09-01',false)
ON CONFLICT (user_id) DO NOTHING;

-- =============================================================================
-- HR: EMPLOYMENT CONTRACTS  (contract_type: 'salaried' | 'hourly' | 'stipend')
-- =============================================================================
INSERT INTO hr.employment_contracts (id, tenant_id, staff_id, type, start_date, job_title, annual_salary, is_active) VALUES
    ('f7000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000001','salaried','1956-09-01','Headmaster',120000.00,true),
    ('f7000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000002','salaried','1956-09-01','Professor of Transfiguration',95000.00,true),
    ('f7000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000003','salaried','1981-09-01','Professor of Potions',92000.00,true),
    ('f7000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000004','salaried','1993-09-01','Professor of Defence Against the Dark Arts',88000.00,true),
    ('f7000000-0000-0000-0000-000000000005','a0000000-0000-0000-0000-000000000002','f1000000-0000-0000-0000-000000000005','salaried','1997-09-01','Third Grade Teacher',58000.00,true),
    ('f7000000-0000-0000-0000-000000000006','a0000000-0000-0000-0000-000000000002','f1000000-0000-0000-0000-000000000006','salaried','2014-09-01','Principal',95000.00,true)
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- SIS: STUDENT PROFILES
-- =============================================================================
INSERT INTO sis.student_profiles (user_id, tenant_id, enrollment_year, cumulative_gpa, academic_standing_status) VALUES
    ('f2000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003',2024,2.85,'good_standing'),
    ('f2000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003',2024,2.50,'good_standing'),
    ('f2000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000003',2024,3.90,'good_standing'),
    ('f2000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000002',2024,1.20,'academic_probation'),
    ('f2000000-0000-0000-0000-000000000005','a0000000-0000-0000-0000-000000000002',2024,3.80,'good_standing'),
    ('f2000000-0000-0000-0000-000000000006','a0000000-0000-0000-0000-000000000002',2024,3.20,'good_standing')
ON CONFLICT (user_id) DO NOTHING;

-- =============================================================================
-- SIS: ACADEMIC TERMS
-- No is_active or calendar_type columns — uses status (sis.term_status enum)
-- =============================================================================
INSERT INTO sis.academic_terms (id, tenant_id, name, alias, academic_year, start_date, end_date, status) VALUES
    ('f3000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003','Fall 2024','FA2024',2024,'2024-09-02','2024-12-20','archived'),
    ('f3000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003','Spring 2025','SP2025',2025,'2025-01-13','2025-05-09','active'),
    ('f3000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000002','Fall 2024','FA2024',2024,'2024-08-26','2024-12-19','archived'),
    ('f3000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000002','Spring 2025','SP2025',2025,'2025-01-06','2025-05-22','active')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- SIS: COURSES
-- grading_basis enum: 'graded_letter' | 'pass_fail' | 'audit' | 'credit_no_credit'
-- =============================================================================
INSERT INTO sis.courses (id, tenant_id, subject, course, title, credits, is_active, is_current, department_id, grading_basis, effective_start_date, course_level) VALUES
    ('f4000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003','DFDA','101','Defence Against the Dark Arts I',3.0,true,true,'f0000000-0000-0000-0000-000000000002','graded_letter','2024-09-01','undergraduate'),
    ('f4000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003','POT','101','Introduction to Potions',3.0,true,true,'f0000000-0000-0000-0000-000000000003','graded_letter','2024-09-01','undergraduate'),
    ('f4000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000003','MAGI','201','Transfiguration I',3.0,true,true,'f0000000-0000-0000-0000-000000000001','graded_letter','2024-09-01','undergraduate'),
    ('f4000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000002','ENG','101','English Composition',3.0,true,true,'f0000000-0000-0000-0000-000000000004','graded_letter','2024-08-01','undergraduate'),
    ('f4000000-0000-0000-0000-000000000005','a0000000-0000-0000-0000-000000000002','MATH','101','Elementary Mathematics',3.0,true,true,'f0000000-0000-0000-0000-000000000004','graded_letter','2024-08-01','undergraduate')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- SIS: COURSE SECTIONS (Fall 2024)
-- =============================================================================
INSERT INTO sis.course_sections (id, tenant_id, course_id, term_id, instructor_id, section_number, max_enrollment, is_published, is_current, instructional_format, effective_start_date) VALUES
    ('f5000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003','f4000000-0000-0000-0000-000000000001','f3000000-0000-0000-0000-000000000001','f1000000-0000-0000-0000-000000000004','01',30,true,true,'in_person','2024-09-02'),
    ('f5000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003','f4000000-0000-0000-0000-000000000002','f3000000-0000-0000-0000-000000000001','f1000000-0000-0000-0000-000000000003','01',24,true,true,'in_person','2024-09-02'),
    ('f5000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000003','f4000000-0000-0000-0000-000000000003','f3000000-0000-0000-0000-000000000001','f1000000-0000-0000-0000-000000000002','01',28,true,true,'in_person','2024-09-02'),
    ('f5000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000002','f4000000-0000-0000-0000-000000000004','f3000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000005','01',30,true,true,'in_person','2024-08-26'),
    ('f5000000-0000-0000-0000-000000000005','a0000000-0000-0000-0000-000000000002','f4000000-0000-0000-0000-000000000005','f3000000-0000-0000-0000-000000000003','f1000000-0000-0000-0000-000000000005','01',30,true,true,'in_person','2024-08-26')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- SIS: ENROLLMENTS
-- =============================================================================
INSERT INTO sis.enrollments (id, tenant_id, student_id, section_id, status, credit_hours_enrolled) VALUES
    ('f8000000-0000-0000-0000-000000000001','a0000000-0000-0000-0000-000000000003','d0000000-0000-0000-0000-000000000001','f5000000-0000-0000-0000-000000000001','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000002','a0000000-0000-0000-0000-000000000003','d0000000-0000-0000-0000-000000000001','f5000000-0000-0000-0000-000000000002','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000003','a0000000-0000-0000-0000-000000000003','d0000000-0000-0000-0000-000000000002','f5000000-0000-0000-0000-000000000001','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000004','a0000000-0000-0000-0000-000000000003','d0000000-0000-0000-0000-000000000002','f5000000-0000-0000-0000-000000000002','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000005','a0000000-0000-0000-0000-000000000003','d0000000-0000-0000-0000-000000000002','f5000000-0000-0000-0000-000000000003','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000006','a0000000-0000-0000-0000-000000000003','f2000000-0000-0000-0000-000000000001','f5000000-0000-0000-0000-000000000001','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000007','a0000000-0000-0000-0000-000000000003','f2000000-0000-0000-0000-000000000001','f5000000-0000-0000-0000-000000000003','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000008','a0000000-0000-0000-0000-000000000003','f2000000-0000-0000-0000-000000000002','f5000000-0000-0000-0000-000000000002','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000009','a0000000-0000-0000-0000-000000000002','f2000000-0000-0000-0000-000000000004','f5000000-0000-0000-0000-000000000004','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000010','a0000000-0000-0000-0000-000000000002','f2000000-0000-0000-0000-000000000005','f5000000-0000-0000-0000-000000000004','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000011','a0000000-0000-0000-0000-000000000002','f2000000-0000-0000-0000-000000000005','f5000000-0000-0000-0000-000000000005','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000012','a0000000-0000-0000-0000-000000000002','f2000000-0000-0000-0000-000000000006','f5000000-0000-0000-0000-000000000004','enrolled',3.0),
    ('f8000000-0000-0000-0000-000000000013','a0000000-0000-0000-0000-000000000002','f2000000-0000-0000-0000-000000000006','f5000000-0000-0000-0000-000000000005','enrolled',3.0)
ON CONFLICT (id) DO NOTHING;

-- Re-enable audit triggers
ALTER TABLE core.users ENABLE TRIGGER ALL;
ALTER TABLE core.tenant_memberships ENABLE TRIGGER ALL;
ALTER TABLE auth_governance.oauth_clients ENABLE TRIGGER ALL;
ALTER TABLE core.departments        DISABLE TRIGGER ALL;
ALTER TABLE hr.staff_profiles       DISABLE TRIGGER ALL;
ALTER TABLE hr.employment_contracts DISABLE TRIGGER ALL;
ALTER TABLE sis.student_profiles    DISABLE TRIGGER ALL;
ALTER TABLE sis.academic_terms      DISABLE TRIGGER ALL;
ALTER TABLE sis.courses             DISABLE TRIGGER ALL;
ALTER TABLE sis.course_sections     DISABLE TRIGGER ALL;
ALTER TABLE sis.enrollments         DISABLE TRIGGER ALL;

COMMIT;

-- Verify
SELECT 'tenants'            AS tbl, COUNT(*) FROM core.tenants
UNION ALL
SELECT 'users'              AS tbl, COUNT(*) FROM core.users
UNION ALL
SELECT 'tenant_memberships' AS tbl, COUNT(*) FROM core.tenant_memberships
UNION ALL
SELECT 'oauth_clients'      AS tbl, COUNT(*) FROM auth_governance.oauth_clients;
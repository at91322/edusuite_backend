-- =============================================================================
-- Migration: RLS baseline setup
-- =============================================================================
-- Two things that must exist before the application can function correctly:
--
-- 1. SENTINEL GUC DEFAULTS
--    The RLS policies cast current_setting('app.current_tenant_id', true)::uuid.
--    When a pool connection has no context set, current_setting returns ''
--    (empty string, not NULL — the is_missing=true flag returns '' not NULL).
--    Casting '' to uuid throws "invalid input syntax for type uuid".
--    Setting a sentinel UUID (all zeros, not a valid tenant) as the database
--    default means unscoped connections get 0 rows from RLS rather than an
--    error. The application overrides these with set_config() inside every
--    transaction.
--
-- 2. PERMISSIVE RLS POLICIES FOR edusuite_app
--    PostgreSQL RLS requires at least one PERMISSIVE policy to grant access.
--    The tenant_isolation_policy on every table is RESTRICTIVE — it scopes
--    rows to the current tenant but cannot grant access by itself. Without a
--    permissive policy, RESTRICTIVE-only = deny everything for edusuite_app.
--    These policies allow all rows; the restrictive policy then scopes them.
-- =============================================================================

-- ── 1. Sentinel GUC defaults ──────────────────────────────────────────────────
-- These must be applied by a superuser. sqlx migrations run as the DATABASE_URL
-- role (edusuite_app) which cannot ALTER DATABASE. Run this migration as
-- postgres, or apply these three lines manually in pgAdmin if the migration
-- role lacks superuser.
--
-- ALTER DATABASE edusuite SET "app.current_tenant_id" = '00000000-0000-0000-0000-000000000000';
-- ALTER DATABASE edusuite SET "app.current_user_id"   = '00000000-0000-0000-0000-000000000000';
-- ALTER DATABASE edusuite SET "app.current_service"   = 'none';
--
-- The lines above are commented out because sqlx runs migrations as edusuite_app
-- which lacks ALTER DATABASE privilege. They have been applied manually and are
-- documented here for reproducibility on fresh environments.
-- On a fresh setup: run those three lines as postgres before cargo run.

-- ── 2. Permissive RLS policies — all schemas ──────────────────────────────────
-- Generated from:
--   SELECT 'CREATE POLICY app_access ON ' || nsp.nspname || '.' || cls.relname ||
--          ' FOR ALL TO edusuite_app USING (true) WITH CHECK (true);'
--   FROM pg_class cls
--   JOIN pg_namespace nsp ON nsp.oid = cls.relnamespace
--   WHERE cls.relrowsecurity = true AND cls.relkind = 'r'
--   AND NOT EXISTS (SELECT 1 FROM pg_policy pol WHERE pol.polrelid = cls.oid AND pol.polname = 'app_access')
--   ORDER BY nsp.nspname, cls.relname;
--
-- core and auth_governance were applied manually earlier in development.
-- This migration covers all remaining schemas.
-- ON CONFLICT DO NOTHING equivalent: each CREATE POLICY will error if the
-- policy already exists, so wrap in DO blocks for idempotency.

DO $$ BEGIN
  -- ── sis ────────────────────────────────────────────────────────────────────
  EXECUTE 'CREATE POLICY app_access ON sis.academic_programs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.academic_standing_policies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.academic_terms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.advising_appointments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.articulation_rules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.athlete_term_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.athletic_teams FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.attendance_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.awarded_credential_majors FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.awarded_credentials FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.corequisite_rules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.course_attributes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.course_repeat_policies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.course_sections FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.courses FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.cpos_overrides FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.cpos_program_rules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.cpos_sweep_results FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.cpos_sweep_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.credential_types FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.disciplinary_incidents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.enrollment_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.enrollments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.faculty_sections FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.grade_scales FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.grade_scale_entries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.graduation_applications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.holds FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.nsc_submissions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.prerequisite_rules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.program_accreditations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.sap_appeals FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.sap_evaluation_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.sap_policies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.sap_results FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.section_meetings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.staff_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_athletes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_course_plans FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_demographics FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_documents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_external_terms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_programs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_risk_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_timeframe_snapshots FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.transfer_courses FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.transfer_institutions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.va_certifications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.waitlist_entries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── lms ───────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.announcements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessment_pools FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessment_sections FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assignments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_item_responses FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_score_breakdowns FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.caliper_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.course_modules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.direct_messages FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.discussion_posts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.discussion_topics FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.item_statistics FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.lti_score_passbacks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.lti_tools FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.originality_reports FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.proctoring_sessions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.question_bank_members FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.question_banks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.question_standards FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.question_tags FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.question_versions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.questions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.rubric_criteria FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.rubric_criterion_scores FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.rubrics FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.scorm_tracking FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.section_grade_summaries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.section_grading_config FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.student_accommodations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.student_attempts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.student_group_members FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.student_groups FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.student_mastery_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.submissions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.xapi_states FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.xapi_statements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── finance ───────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.bank_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.bank_reconciliations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.bank_transactions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.budget_lines FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.budget_transfers FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.currency_exchange_rates FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.deferred_revenue_schedules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.endowment_distributions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.endowment_funds FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.expense_line_items FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.expense_reports FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.fafsa_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.financial_aid_awards FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.financial_aid_disbursements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.fiscal_years FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.fixed_assets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.fixed_asset_depreciation FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.fund_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.gl_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.gl_entries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.grant_budget_categories FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.grants FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.multi_state_nexus FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.pay_stub_line_items FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.pay_stubs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.payroll_items FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.payroll_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.purchase_order_lines FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.purchase_orders FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.student_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.student_transactions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tax_1099_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tax_rates FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.travel_authorizations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.va_benefit_certifications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.vendor_invoices FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.vendor_payments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.vendor_tax_details FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.vendors FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── hr ────────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.applicant_tracking FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.budgeted_positions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.employment_contracts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.interview_feedback FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.job_applications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.job_postings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.leave_balances FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.leave_requests FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.offer_letters FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.performance_evaluations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.professional_credentials FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.salary_grid FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.staff_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.timesheets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── crm ───────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.admissions_applications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.alumni_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.constituent_interactions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.donor_pledges FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.gifts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.matriculation_batches FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.mentorship_matches FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.onboarding_tasks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.prospects FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.recruitment_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.requirement_completions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── ops ───────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.asset_checkouts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.assets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.buildings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.campuses FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.energy_bills FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.event_tickets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.fleet_inspections FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.fleet_vehicles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.housing_applications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.housing_assignments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.maintenance_schedules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.meter_readings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.orders FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.order_items FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.products FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.reservations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.rooms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.service_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.storefronts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.transit_routes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.transit_stop_assignments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.utility_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.utility_meters FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.vehicle_odometer_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.work_orders FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── cms ───────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.content_blocks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.form_submissions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.forms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.media_asset_subjects FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.media_assets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.navigation_menus FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.page_subjects FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.page_tags FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.page_versions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.pages FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.privacy_notices FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.redirects FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.sites FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.tags FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── collab ────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.milestones FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.sprint_tasks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.sprints FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.task_comments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.task_dependencies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.task_lms_links FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.tasks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.time_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.workspace_members FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.workspaces FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── comms ─────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.delivery_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.journey_enrollments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.journeys FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.suppression_list FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.templates FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.unsubscribe_tokens FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.user_devices FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.user_preferences FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.webhook_delivery_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.webhook_endpoints FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.webhook_subscriptions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── dms ───────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.document_versions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.documents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.folder_acls FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.folders FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── board ─────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.agendas FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.attachments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.committees FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.meetings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.members FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.public_comments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── policy ────────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.attestation_campaigns FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.attestation_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.categories FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.documents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.manuals FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.review_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.review_schedules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.versions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── workflow ──────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.esignatures FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.form_stages FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.forms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.routing_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.steps FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.submissions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── catalog ───────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON catalog.editions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON catalog.entries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON catalog.publications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── data_governance ───────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.anonymisation_jobs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.breach_affected_subjects FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.breach_incidents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.consent_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.consent_types FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.consent_withdrawal_actions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.consent_withdrawal_completions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.cookie_categories FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.cookie_consent_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.cookie_policies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.data_disclosures FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.data_sharing_agreements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.data_subject_requests FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.destruction_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.dsr_export_jobs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.field_anonymisation_rules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.legitimate_interest_assessments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.media_consent_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.parental_consent_requests FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.pii_field_registry FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.privacy_impact_assessments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.retention_holds FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.retention_schedules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── event_bus ─────────────────────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.dead_letters FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.subscriptions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
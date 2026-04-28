-- =============================================================================
-- RLS permissive policies for edusuite_app — all remaining schemas
-- Views excluded (cannot have RLS policies).
-- Run as postgres (superuser) in pgAdmin.
-- core and auth_governance already done — skipped here.
-- =============================================================================

-- ── board (7) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.agendas FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.attachments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.committees FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.meetings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.members FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.public_comments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON board.votes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── catalog (5) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON catalog.course_listings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON catalog.curriculum_proposals FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON catalog.editions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON catalog.program_listings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON catalog.sections FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── cms (13) ──────────────────────────────────────────────────
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
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.sites FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.tags FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON cms.url_redirects FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── collab (9) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.milestones FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.sprints FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.task_comments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.task_dependencies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.task_lms_links FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.tasks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.time_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.workspace_members FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON collab.workspaces FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── comms (10) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.delivery_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.journey_steps FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.journeys FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.templates FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.user_devices FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.user_preferences FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.webhook_delivery_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.webhook_endpoints FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON comms.webhook_subscriptions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── crm (19) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.alumni_chapters FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.alumni_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.application_requirements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.application_reviews FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.applications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.chapter_members FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.constituent_interactions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.donor_pledges FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.event_registrations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.fundraising_campaigns FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.gifts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.matriculation_batches FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.mentorship_matches FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.onboarding_template_tasks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.onboarding_templates FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.prospect_journey_states FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.prospect_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.recruitment_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON crm.student_onboarding_tasks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── data_governance (13) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.breach_affected_subjects FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.breach_incidents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.consent_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.consent_types FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.data_disclosures FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.data_sharing_agreements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.data_subject_requests FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.destruction_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.pii_field_registry FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.privacy_impact_assessments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.retention_holds FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.retention_schedules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON data_governance.tenant_regulations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── dms (7) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.document_acls FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.document_links FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.document_versions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.documents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.folder_acls FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.folders FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON dms.retention_policies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── event_bus (18) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.dead_letters FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.event_types FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m04 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m05 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m06 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m07 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m08 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m09 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m10 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m11 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2026m12 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2027m01 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2027m02 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.outbox_y2027m03 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.saga_instances FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.saga_steps FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON event_bus.subscriptions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── finance (57) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.accounting_configurations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.ar_invoices FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.ar_payments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.asset_depreciation_schedules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.bank_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.bank_reconciliations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.budget_lines FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.budget_transfers FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.charts_of_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.currency_exchange_rates FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.deferred_revenue FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.depreciation_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.economic_nexus_monitoring FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.endowment_fund_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.expense_line_items FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.expense_reports FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.fafsa_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.financial_aid_awards FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.fiscal_years FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.fixed_assets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.funds FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.gl_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.grant_budgets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.inter_fund_transfers FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.journal_entries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.journal_lines FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.ledger_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.nslds_reporting_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.pay_stub_line_items FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.pay_stubs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.payroll_item_types FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.payroll_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.purchase_orders FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.receiving_reports FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.sap_appeals FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.sap_evaluation_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.sap_evaluations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.sap_policies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.sponsored_grants FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.student_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.student_transactions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tax_1099_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tax_filing_periods FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tax_jurisdictions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tax_nexus FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tax_product_rules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tax_rates FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.tenant_tax_registrations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.transaction_tax_lines FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.travel_authorizations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.trial_balance_snapshots FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.va_certifications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.vendor_invoices FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.vendor_payments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.vendor_tax_details FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.vendors FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON finance.veteran_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── hr (17) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.budgeted_positions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.employee_supervisors FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.employment_contracts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.interview_panelists FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.interviews FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.job_applications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.job_offers FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.job_postings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.job_requisitions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.leave_balances FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.leave_requests FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.performance_evaluations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.salary_grid_nodes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.salary_grids FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.staff_credentials FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.staff_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON hr.timesheets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── lms (71) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.announcements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.answer_choices FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessment_accommodations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessment_analytics_snapshots FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessment_attempts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessment_items FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessment_pools FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessment_sections FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assessments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assignment_groups FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assignment_standards FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.assignments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_events_y2026m05 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_events_y2026m06 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_events_y2026m07 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_events_y2026m08 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_events_y2026m09 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_events_y2026m10 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_proctoring_sessions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_responses FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.attempt_score_breakdowns FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.caliper_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.caliper_events_y2026m05 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.caliper_events_y2026m06 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.caliper_events_y2026m07 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.caliper_events_y2026m08 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.caliper_events_y2026m09 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.caliper_events_y2026m10 FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.conversation_participants FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.conversations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.course_modules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.discussion_posts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.discussion_topics FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.grade_history FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.grade_roster_entries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.grade_roster_submissions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.grades FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.grading_scale_entries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.grading_scales FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.group_sets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.item_statistics FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.learning_packages FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.learning_standards FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.lti_deployments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.lti_launches FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.lti_registered_tools FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.lti_resource_links FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.lti_score_passbacks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.lti_tools FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.messages FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.originality_reports FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
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
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.student_group_members FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.student_groups FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.student_mastery_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.submissions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.xapi_statements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON lms.xapi_states FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── ops (29) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.asset_categories FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.assets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.buildings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.campus_events FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.campuses FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.event_ticket_types FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.event_tickets FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.housing_applications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.meter_readings FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.pm_schedules FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.reservations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.residential_rooms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.retail_order_items FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.retail_orders FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.retail_products FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.retail_storefronts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.rooms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.student_route_assignments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.transit_route_stops FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.transit_routes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.transit_stops FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.utility_accounts FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.utility_bills FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.utility_meters FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.vehicle_inspections FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.vehicle_odometer_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.vehicle_service_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.vehicles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON ops.work_orders FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── policy (7) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.attestation_campaigns FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.categories FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.documents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.manuals FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.review_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.user_acknowledgements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON policy.versions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── reference (13) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.accreditors FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.address_formats FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.cip_codes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.countries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.external_organizations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.ficm_codes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.ipeds_race_ethnicity FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.languages FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.ncaa_sports FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.postal_codes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.standardized_tests FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.subdivisions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON reference.timezones FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── sis (58) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.academic_programs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
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
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.enrollments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.external_courses FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.external_grade_entries FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.external_grading_scales FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.external_institution_scale_assignments FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.external_institutions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.gpa_recalculation_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.grade_crosswalks FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.graduation_applications FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.intervention_cases FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.materials_catalog FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.ncaa_academic_evaluations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.nsc_submission_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.prerequisite_groups FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.prerequisite_overrides FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.prerequisite_requirements FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.prerequisite_sweep_results FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.prerequisite_sweep_runs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.program_accreditations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.repeat_rule_overrides FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.section_materials FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.section_meeting_times FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.section_syllabi FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_academic_standing FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_athletes FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_course_plans FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_demographics FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_documents FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_external_terms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_programs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_risk_profiles FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.student_timeframe_snapshots FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.transcript_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.transfer_course_records FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.transfer_evaluations FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON sis.transfer_grade_policies FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- ── workflow (6) ──────────────────────────────────────────────────
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.esignatures FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.form_stages FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.forms FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.routing_logs FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.steps FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN EXECUTE 'CREATE POLICY app_access ON workflow.submissions FOR ALL TO edusuite_app USING (true) WITH CHECK (true)'; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- Total: 359 tables
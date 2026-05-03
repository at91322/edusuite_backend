// src/modules/core/mod.rs
//
// Core module — institutional identity layer.
// Every other module depends on this one.
//
// Handler sub-modules mirror the group structure from the endpoint plan:
//   handlers/tenant.rs  — Group 1: tenant self-service + departments
//   handlers/user.rs    — Group 2: user identity (Step 2)
//   handlers/contact.rs — Group 3: emergency contacts (Step 3)
//   handlers/role.rs    — Group 4: role grants (Step 4)
//   handlers/audit.rs   — Group 5: audit log reads (Step 5)
//   handlers/member.rs  — Group 6: tenant membership management
//
// Step 1 activates Groups 1 and 6.
// Steps 2–5 are stubbed as comments at their insertion points.

pub mod handlers;
pub mod models;
pub mod queries;
pub mod write_models;
pub mod write_queries;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()

        // ── Group 1a: Tenant self-service ─────────────────────────────────
        .route("/tenants/me",
            get(handlers::tenant::get_tenant_me)
           .patch(handlers::tenant::patch_tenant_me))

        .route("/tenants/me/subscriptions",
            get(handlers::tenant::get_tenant_subscriptions))

        .route("/tenants/me/feature-flags",
            get(handlers::tenant::get_feature_flags))

        // ── Group 1b: Department management ──────────────────────────────
        .route("/tenants/me/departments",
            get(handlers::tenant::list_departments)
           .post(handlers::tenant::create_department))

        .route("/tenants/me/departments/:id",
            get(handlers::tenant::get_department)
           .patch(handlers::tenant::update_department))

        // ── Group 2: User identity ────────────────────────────────────────
        // Step 2
        .route("/users/:id",
            get(handlers::user::get_user)
           .patch(handlers::user::patch_user))

        .route("/users/:id/name-history",
            get(handlers::user::get_name_history))

        .route("/users/:id/emails",
            get(handlers::user::list_emails)
           .post(handlers::user::create_email))

        .route("/users/:id/emails/:email_id",
            patch(handlers::user::patch_email)
           .delete(handlers::user::delete_email))

        .route("/users/:id/phones",
            get(handlers::user::list_phones)
           .post(handlers::user::create_phone))

        .route("/users/:id/phones/:phone_id",
            patch(handlers::user::patch_phone)
           .delete(handlers::user::delete_phone))

        .route("/users/:id/addresses",
            get(handlers::user::list_addresses)
           .post(handlers::user::create_address))

        .route("/users/:id/addresses/:addr_id",
            patch(handlers::user::patch_address)
           .delete(handlers::user::delete_address))

        // ── Group 3: Emergency contacts ───────────────────────────────────
        // Step 3
        .route("/users/:id/emergency-contacts",
            get(handlers::contact::list_emergency_contacts)
           .post(handlers::contact::create_emergency_contact))

        .route("/users/:id/emergency-contacts/:contact_id",
            patch(handlers::contact::patch_emergency_contact)
           .delete(handlers::contact::delete_emergency_contact))

        // ── Group 4: Role management ──────────────────────────────────────
        // Step 4
        .route("/users/:id/roles",
            get(handlers::role::list_roles)
           .post(handlers::role::grant_role))

        .route("/users/:id/roles/:role_name",
            delete(handlers::role::revoke_role))

        // ── Group 5: Audit log ────────────────────────────────────────────
        // Step 5
        .route("/audit-logs",
            get(handlers::audit::list_audit_logs))

        .route("/audit-logs/:id",
            get(handlers::audit::get_audit_log))

        // ── Group 6: Tenant membership management ─────────────────────────
        .route("/members",
            get(handlers::member::list_members))

        .route("/members/:user_id",
            patch(handlers::member::patch_member)
           .delete(handlers::member::delete_member))
}
// src/modules/lms/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;
pub mod write_models;
pub mod write_queries;

use axum::{routing::{get, patch, post}, Router};
use crate::state::AppState;

/// LMS routes — mount at /lms
pub fn router() -> Router<AppState> {
    Router::new()
        // ── Sections (read) ──────────────────────────────────────────────────
        .route("/sections",     get(handlers::list_sections))
        .route("/sections/:id", get(handlers::get_section))

        // ── Section content writes ───────────────────────────────────────────
        .route("/sections/:id/modules",
            post(handlers::create_module))
        .route("/sections/:id/assignments",
            post(handlers::create_assignment))

        // ── Grade entry workflow ─────────────────────────────────────────────
        .route("/grade-roster-entries/:id",
            patch(handlers::update_grade_entry))

        // ── Grade roster lifecycle ───────────────────────────────────────────
        .route("/grade-roster-submissions/:id/submit",
            post(handlers::submit_grade_roster))
        .route("/grade-roster-submissions/:id/post",
            post(handlers::post_grade_roster))
}

/// SIS-prefixed routes served by the LMS module.
/// Mount at /sis in main.rs alongside sis::router().
/// Resolves to: GET /sis/courses/:id/sections
pub fn sis_bridge_router() -> Router<AppState> {
    Router::new()
        .route("/courses/:id/sections", get(handlers::list_sections_for_course))
}
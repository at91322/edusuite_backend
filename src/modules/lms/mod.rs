// src/modules/lms/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;

use axum::{routing::get, Router};
use crate::state::AppState;

/// LMS-scoped routes: /lms/...
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sections",     get(handlers::list_sections))
        .route("/sections/:id", get(handlers::get_section))     // NEW
}

/// Routes that mount under the SIS prefix (/sis/...) but are served by the
/// LMS module because the response shape is LMS-enriched.
///
/// Mount in main.rs as:
///   .nest("/sis", lms::sis_bridge_router())
///
/// This keeps the SIS router clean while co-locating the section-query logic
/// with the rest of the LMS section code.
pub fn sis_bridge_router() -> Router<AppState> {
    Router::new()
        .route("/courses/:id/sections",
            get(handlers::list_sections_for_course))             // NEW
}
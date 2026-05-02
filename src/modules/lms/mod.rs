// src/modules/lms/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;

use axum::{routing::get, Router};
use crate::state::AppState;

/// LMS routes — mount at /lms
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sections",     get(handlers::list_sections))
        .route("/sections/:id", get(handlers::get_section))
}

/// SIS-prefixed routes served by the LMS module.
/// Mount at /sis in main.rs alongside sis::router().
/// Resolves to: GET /sis/courses/:id/sections
pub fn sis_bridge_router() -> Router<AppState> {
    Router::new()
        .route("/courses/:id/sections", get(handlers::list_sections_for_course))
}
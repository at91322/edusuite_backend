// src/modules/sis/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;

use axum::{routing::get, Router};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/students",                    get(handlers::list_students))
        .route("/students/:id",                get(handlers::get_student))
        .route("/students/:id/enrollments",    get(handlers::get_student_enrollments))
        .route("/courses",                     get(handlers::list_courses))
        .route("/courses/:id",                 get(handlers::get_course))
}
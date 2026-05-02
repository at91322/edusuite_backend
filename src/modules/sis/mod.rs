// src/modules/sis/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;
pub mod write_models;
pub mod write_queries;

use axum::{routing::get, Router};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/students",                    get(handlers::list_students)
                                              .post(handlers::create_student))
        .route("/students/:id",                get(handlers::get_student)
                                              .patch(handlers::update_student))
        .route("/students/:id/enrollments",    get(handlers::get_student_enrollments))
        .route("/courses",                     get(handlers::list_courses))
        .route("/courses/:id",                 get(handlers::get_course))
}
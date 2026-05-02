// src/modules/sis/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;
pub mod write_models;
pub mod write_queries;

use axum::{
    routing::{delete, get, post},
    Router,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Students ──────────────────────────────────────────────────────
        .route("/students",
            get(handlers::list_students)
           .post(handlers::create_student))

        .route("/students/:id",
            get(handlers::get_student)
           .patch(handlers::update_student))

        // ── Student enrollments (student-scoped) ──────────────────────────
        .route("/students/:id/enrollments",
            get(handlers::get_student_enrollments)           // existing
           .post(handlers::create_student_enrollment))       // NEW

        .route("/students/:id/enrollments/:enrollment_id",
            delete(handlers::delete_student_enrollment))     // NEW

        // ── Cross-module enrollment (registrar workflow) ───────────────────
        .route("/enrollments",
            post(handlers::cross_module_enroll))             // NEW

        // ── Courses ───────────────────────────────────────────────────────
        .route("/courses",
            get(handlers::list_courses))

        .route("/courses/:id",
            get(handlers::get_course))
}
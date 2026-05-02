// src/modules/hr/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;
pub mod write_models;
pub mod write_queries;

use axum::{
    routing::{get, post},
    Router,
};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/staff",
            get(handlers::list_staff)
           .post(handlers::create_staff_member))

        .route("/staff/:id",
            get(handlers::get_staff_member)
           .patch(handlers::update_staff_member))

        // NEW: employment contracts for an existing staff member
        .route("/staff/:id/contracts",
            post(handlers::create_staff_contract))
}
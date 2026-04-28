// src/modules/hr/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;

use axum::{routing::get, Router};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/staff",     get(handlers::list_staff))
        .route("/staff/:id", get(handlers::get_staff_member))
}
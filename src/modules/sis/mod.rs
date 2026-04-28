// src/modules/sis/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;

use axum::{routing::get, Router};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/students", get(handlers::list_students))
}
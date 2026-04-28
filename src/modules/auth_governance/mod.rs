// src/modules/auth_governance/mod.rs

pub mod handlers;
pub mod models;
pub mod queries;
 
use axum::{routing::{get, post}, Router};
use crate::state::AppState;

// Build the auth router. Mount at /auth in main.rs.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login",   post(handlers::login))
        .route("/refresh", post(handlers::refresh))
        .route("/logout",  post(handlers::logout))
        .route("/me",      get(handlers::me))
}
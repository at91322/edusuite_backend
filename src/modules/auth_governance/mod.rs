use axum::{routing::post, Router};
use crate::state::AppState;

mod handlers;
mod models;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(handlers::login))
}
use axum::Router;git
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
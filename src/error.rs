// src/error.rs
//
// Centralised error type for the entire API.
//
// Every handler returns `Result<T, AppError>`. Axum's `IntoResponse` impl
// converts each variant to the appropriate HTTP status and a consistent JSON
// body so the client always knows what shape to expect:
//
//   { "error": "unauthorized", "message": "Invalid or expired token" }
//
// The `message` field is safe to return to clients — it never exposes
// internal implementation details, stack traces, or database error text.
// Internal details are logged at ERROR level for the operator.
 
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
 
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 400 — the request body or parameters are malformed or fail validation.
    #[error("Bad request: {0}")]
    BadRequest(String),
 
    /// 401 — authentication failed or the token is missing/expired.
    /// Message is deliberately vague to avoid credential enumeration.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
 
    /// 403 — authenticated but not permitted to perform this action.
    #[error("Forbidden: {0}")]
    Forbidden(String),
 
    /// 404 — the requested resource does not exist (or RLS hides it).
    #[error("Not found: {0}")]
    NotFound(String),
 
    /// 409 — the request conflicts with existing state (duplicate, etc.).
    #[error("Conflict: {0}")]
    Conflict(String),
 
    /// 500 — unexpected internal error. Details are logged, not returned.
    #[error("Internal server error")]
    Internal(#[source] anyhow::Error),
}
 
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "bad_request", msg.clone())
            }
            AppError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone())
            }
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, "forbidden", msg.clone())
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "not_found", msg.clone())
            }
            AppError::Conflict(msg) => {
                (StatusCode::CONFLICT, "conflict", msg.clone())
            }
            AppError::Internal(err) => {
                // Log the real error; return a generic message to the client.
                tracing::error!(error = %err, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error",
                    "An unexpected error occurred".to_string(),
                )
            }
        };
 
        let body = Json(json!({
            "error":   code,
            "message": message,
        }));
 
        (status, body).into_response()
    }
}
 
// ── From conversions ──────────────────────────────────────────────────────────
 
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        // Map well-known database errors to domain errors where possible.
        match &err {
            sqlx::Error::RowNotFound => {
                AppError::NotFound("Resource not found".to_string())
            }
            sqlx::Error::Database(db_err) => {
                // PostgreSQL unique violation (code 23505) → Conflict
                if db_err.code().as_deref() == Some("23505") {
                    return AppError::Conflict(
                        "A resource with these values already exists".to_string(),
                    );
                }
                // RLS policy violation (code 42501) → the caller shouldn't know
                // whether the resource exists; treat as 404.
                if db_err.code().as_deref() == Some("42501") {
                    return AppError::NotFound("Resource not found".to_string());
                }
                tracing::error!(db_error = %db_err, "Database error");
                AppError::Internal(anyhow::anyhow!("Database error: {}", db_err))
            }
            _ => {
                tracing::error!(sqlx_error = %err, "SQLx error");
                AppError::Internal(anyhow::anyhow!(err))
            }
        }
    }
}
 
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => {
                AppError::Unauthorized("Token has expired".to_string())
            }
            ErrorKind::InvalidToken
            | ErrorKind::InvalidSignature
            | ErrorKind::InvalidAlgorithm => {
                AppError::Unauthorized("Invalid token".to_string())
            }
            _ => {
                tracing::error!(jwt_error = %err, "JWT error");
                AppError::Internal(anyhow::anyhow!(err))
            }
        }
    }
}
 
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}
 
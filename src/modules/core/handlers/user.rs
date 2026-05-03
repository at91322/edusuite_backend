#![allow(unused_mut)]
// src/modules/core/handlers/user.rs
// Group 2 — User identity management (Step 2)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::core::queries;

// ── GET /core/users/:id ───────────────────────────────────────────────────────

pub async fn get_user(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let detail = queries::get_user(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    match detail {
        None    => Err(AppError::NotFound(format!("User {} not found", user_id))),
        Some(u) => Ok(Json(u)),
    }
}

// ── PATCH /core/users/:id ─────────────────────────────────────────────────────

pub async fn patch_user(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(_user_id): Path<Uuid>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── GET /core/users/:id/name-history ─────────────────────────────────────────

pub async fn get_name_history(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let history = queries::get_name_history(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(history))
}

// ── GET /core/users/:id/emails ────────────────────────────────────────────────

pub async fn list_emails(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let emails = queries::list_emails(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(emails))
}

// ── POST /core/users/:id/emails ───────────────────────────────────────────────

pub async fn create_email(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(_user_id): Path<Uuid>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── PATCH /core/users/:id/emails/:email_id ────────────────────────────────────

pub async fn patch_email(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _email_id)): Path<(Uuid, Uuid)>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── DELETE /core/users/:id/emails/:email_id ───────────────────────────────────

pub async fn delete_email(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _email_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── GET /core/users/:id/phones ────────────────────────────────────────────────

pub async fn list_phones(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let phones = queries::list_phones(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(phones))
}

// ── POST /core/users/:id/phones ───────────────────────────────────────────────

pub async fn create_phone(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(_user_id): Path<Uuid>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── PATCH /core/users/:id/phones/:phone_id ────────────────────────────────────

pub async fn patch_phone(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _phone_id)): Path<(Uuid, Uuid)>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── DELETE /core/users/:id/phones/:phone_id ───────────────────────────────────

pub async fn delete_phone(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _phone_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── GET /core/users/:id/addresses ─────────────────────────────────────────────

pub async fn list_addresses(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let addresses = queries::list_addresses(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(addresses))
}

// ── POST /core/users/:id/addresses ────────────────────────────────────────────

pub async fn create_address(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(_user_id): Path<Uuid>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── PATCH /core/users/:id/addresses/:addr_id ──────────────────────────────────

pub async fn patch_address(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _addr_id)): Path<(Uuid, Uuid)>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}

// ── DELETE /core/users/:id/addresses/:addr_id ─────────────────────────────────

pub async fn delete_address(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _addr_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 2: not yet implemented".into()))
}
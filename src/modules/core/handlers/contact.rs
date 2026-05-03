#![allow(unused_mut)]
// src/modules/core/handlers/contact.rs
// Group 3 — Emergency contacts (Step 3)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::core::queries;

pub async fn list_emergency_contacts(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let contacts = queries::list_emergency_contacts(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(contacts))
}

pub async fn create_emergency_contact(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(_user_id): Path<Uuid>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 3: not yet implemented".into()))
}

pub async fn patch_emergency_contact(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _contact_id)): Path<(Uuid, Uuid)>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 3: not yet implemented".into()))
}

pub async fn delete_emergency_contact(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _contact_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 3: not yet implemented".into()))
}
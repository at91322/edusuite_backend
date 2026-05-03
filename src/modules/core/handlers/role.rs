#![allow(unused_mut)]
// src/modules/core/handlers/role.rs
// Group 4 — Role management (Step 4)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::core::queries;

pub async fn list_roles(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let roles = queries::list_roles(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(roles))
}

pub async fn grant_role(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(_user_id): Path<Uuid>,
    _body: axum::body::Body,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 4: not yet implemented".into()))
}

pub async fn revoke_role(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((_user_id, _role_name)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user; // stub — transaction rolls back automatically
    Err::<StatusCode, _>(AppError::BadRequest("Step 4: not yet implemented".into()))
}
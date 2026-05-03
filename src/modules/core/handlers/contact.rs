// src/modules/core/handlers/contact.rs
// Group 3 — Emergency contacts (reads + writes)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::core::{
    queries,
    write_models::{CreateEmergencyContactRequest, PatchEmergencyContactRequest},
    write_queries,
};

// ── GET /core/users/:id/emergency-contacts ────────────────────────────────────

pub async fn list_emergency_contacts(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let contacts = queries::list_emergency_contacts(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(contacts))
}

// ── POST /core/users/:id/emergency-contacts ───────────────────────────────────

pub async fn create_emergency_contact(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(req): Json<CreateEmergencyContactRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let result = write_queries::create_emergency_contact(
        &mut user.tx, user_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(result)))
}

// ── PATCH /core/users/:id/emergency-contacts/:contact_id ──────────────────────

pub async fn patch_emergency_contact(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, contact_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<PatchEmergencyContactRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let result = write_queries::patch_emergency_contact(
        &mut user.tx, user_id, contact_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(result))
}

// ── DELETE /core/users/:id/emergency-contacts/:contact_id ─────────────────────

pub async fn delete_emergency_contact(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, contact_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {

    write_queries::delete_emergency_contact(
        &mut user.tx, user_id, contact_id,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
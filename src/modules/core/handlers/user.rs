// src/modules/core/handlers/user.rs
// Group 2 — User identity management (reads + writes)

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
    write_models::{
        PatchUserRequest,
        CreateEmailRequest, PatchEmailRequest,
        CreatePhoneRequest, PatchPhoneRequest,
        CreateAddressRequest, PatchAddressRequest,
    },
    write_queries,
};

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
    Path(user_id): Path<Uuid>,
    Json(req): Json<PatchUserRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;
    let actor_id  = user.claims.sub;

    let updated = write_queries::patch_user(
        &mut user.tx, tenant_id, user_id, actor_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(updated))
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
    Path(user_id): Path<Uuid>,
    Json(req): Json<CreateEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;
    let result = write_queries::create_email(&mut user.tx, user_id, &req).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(result)))
}

// ── PATCH /core/users/:id/emails/:email_id ────────────────────────────────────

pub async fn patch_email(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, email_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<PatchEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;
    let result = write_queries::patch_email(&mut user.tx, user_id, email_id, &req).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(result))
}

// ── DELETE /core/users/:id/emails/:email_id ───────────────────────────────────

pub async fn delete_email(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, email_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    write_queries::delete_email(&mut user.tx, user_id, email_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(StatusCode::NO_CONTENT)
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
    Path(user_id): Path<Uuid>,
    Json(req): Json<CreatePhoneRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;
    let result = write_queries::create_phone(&mut user.tx, user_id, &req).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(result)))
}

// ── PATCH /core/users/:id/phones/:phone_id ────────────────────────────────────

pub async fn patch_phone(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, phone_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<PatchPhoneRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;
    let result = write_queries::patch_phone(&mut user.tx, user_id, phone_id, &req).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(result))
}

// ── DELETE /core/users/:id/phones/:phone_id ───────────────────────────────────

pub async fn delete_phone(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, phone_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    write_queries::delete_phone(&mut user.tx, user_id, phone_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(StatusCode::NO_CONTENT)
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
    Path(user_id): Path<Uuid>,
    Json(req): Json<CreateAddressRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;
    let result = write_queries::create_address(&mut user.tx, user_id, &req).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(result)))
}

// ── PATCH /core/users/:id/addresses/:addr_id ──────────────────────────────────

pub async fn patch_address(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, addr_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<PatchAddressRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;
    let result = write_queries::patch_address(&mut user.tx, user_id, addr_id, &req).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(result))
}

// ── DELETE /core/users/:id/addresses/:addr_id ─────────────────────────────────

pub async fn delete_address(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, addr_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    write_queries::delete_address(&mut user.tx, user_id, addr_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
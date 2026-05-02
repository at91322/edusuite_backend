// src/modules/hr/handlers.rs

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use super::{models::{ListStaffParams, StaffListResponse}, queries};

pub async fn list_staff(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListStaffParams>,
) -> Result<impl IntoResponse, AppError> {

    let (staff, total) = queries::list_staff(&mut user.tx, &params).await?;

    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::debug!(
        tenant_id = %user.claims.tenant_id,
        count = staff.len(),
        total,
        page,
        "GET /hr/staff"
    );

    Ok(Json(StaffListResponse { data: staff, page, per_page, total, total_pages }))
}

pub async fn get_staff_member(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(staff_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let member = queries::get_staff_member(&mut user.tx, staff_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match member {
        None    => Err(AppError::NotFound(format!("Staff member {} not found", staff_id))),
        Some(s) => {
            tracing::debug!(
                tenant_id = %user.claims.tenant_id,
                staff_id  = %staff_id,
                "GET /hr/staff/:id"
            );
            Ok(Json(s))
        }
    }
}

// ── POST /hr/staff ────────────────────────────────────────────────────────────

pub async fn create_staff_member(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Json(req): Json<super::write_models::CreateStaffRequest>,
) -> Result<impl IntoResponse, AppError> {
    use axum::http::StatusCode;

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;
    let user_id   = super::write_queries::create_staff_member(
        &mut user.tx, tenant_id, &req,
    ).await?;

    let detail = queries::get_staff_member(&mut user.tx, user_id)
        .await?
        .ok_or_else(|| AppError::Internal(
            anyhow::anyhow!("Staff member created but not found on re-fetch")
        ))?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id = %tenant_id,
        user_id   = %user_id,
        username  = %req.username,
        "Staff member created"
    );

    Ok((StatusCode::CREATED, Json(detail)))
}

// ── PATCH /hr/staff/:id ───────────────────────────────────────────────────────

pub async fn update_staff_member(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(staff_id): Path<Uuid>,
    Json(req): Json<super::write_models::UpdateStaffRequest>,
) -> Result<impl IntoResponse, AppError> {

    if !req.has_changes() {
        return Err(AppError::BadRequest(
            "Request body contains no fields to update".into()
        ));
    }

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;
    super::write_queries::update_staff_member(
        &mut user.tx, tenant_id, staff_id, &req,
    ).await?;

    let detail = queries::get_staff_member(&mut user.tx, staff_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Staff member {} not found", staff_id)))?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id = %tenant_id,
        staff_id  = %staff_id,
        "Staff member updated"
    );

    Ok(Json(detail))
}
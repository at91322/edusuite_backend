// src/modules/hr/handlers.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use super::{models::{ListStaffParams, StaffListResponse}, queries};

// ── GET /hr/staff ─────────────────────────────────────────────────────────────

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

// ── GET /hr/staff/:id ─────────────────────────────────────────────────────────

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

// POST /hr/staff/:id/contracts ─────────────────────────────────────────────────

/// Add a new employment contract for an existing staff member.
///
/// If `deactivate_existing` is true (the default), any currently active
/// contract is soft-deactivated before the new one is inserted. This is the
/// standard "renew contract" / "role change" workflow. Pass
/// `deactivate_existing: false` to layer a second concurrent contract
/// (e.g. adjunct stipend alongside a full-time appointment).
pub async fn create_staff_contract(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(staff_id): Path<Uuid>,
    Json(req): Json<super::write_models::CreateContractRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;

    let contract = super::write_queries::create_contract(
        &mut user.tx, tenant_id, staff_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id   = %tenant_id,
        staff_id    = %staff_id,
        contract_id = %contract.contract_id,
        contract_type = %contract.contract_type,
        deactivated_existing = req.deactivate_existing,
        "Employment contract created"
    );

    Ok((StatusCode::CREATED, Json(contract)))
}
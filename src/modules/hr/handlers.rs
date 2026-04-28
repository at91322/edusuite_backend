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
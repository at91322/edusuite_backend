// src/modules/finance/handlers/va.rs
// Group 4 — VA Benefits

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::finance::{
    models::{ListVaCertificationsParams, VaCertificationListResponse},
    queries,
};

// ── GET /finance/students/:student_id/veteran-profile ────────────────────────

pub async fn get_veteran_profile(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let profile = queries::get_veteran_profile(&mut user.tx, student_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match profile {
        None    => Err(AppError::NotFound(
            format!("No VA veteran profile found for student {}", student_id)
        )),
        Some(p) => Ok(Json(p)),
    }
}

// ── GET /finance/students/:student_id/va-certifications ──────────────────────

pub async fn list_va_certifications(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
    Query(params): Query<ListVaCertificationsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (certs, total) = queries::list_va_certifications(
        &mut user.tx, student_id, &params,
    ).await?;

    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(VaCertificationListResponse { data: certs, page, per_page, total, total_pages }))
}

// ── GET /finance/students/:student_id/va-certifications/:id ──────────────────

pub async fn get_va_certification(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((student_id, cert_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {

    let cert = queries::get_va_certification(&mut user.tx, student_id, cert_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match cert {
        None    => Err(AppError::NotFound(
            format!("VA certification {} not found for student {}", cert_id, student_id)
        )),
        Some(c) => Ok(Json(c)),
    }
}
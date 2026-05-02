// src/modules/finance/handlers/va.rs
// Group 4 — VA Benefits (reads + writes)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::finance::{
    models::{ListVaCertificationsParams, VaCertificationListResponse},
    queries,
    write_models::{AmendVaCertificationRequest, CreateVaCertificationRequest},
    write_queries,
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

// ── POST /finance/students/:student_id/va-certifications ─────────────────────

pub async fn create_va_certification(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
    Json(req): Json<CreateVaCertificationRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id    = user.claims.tenant_id;
    let certified_by = user.claims.sub;

    let response = write_queries::create_va_certification(
        &mut user.tx, tenant_id, student_id, certified_by, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(response)))
}

// ── PATCH /finance/students/:student_id/va-certifications/:id ────────────────

pub async fn amend_va_certification(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((student_id, cert_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<AmendVaCertificationRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id    = user.claims.tenant_id;
    let certified_by = user.claims.sub;

    let response = write_queries::amend_va_certification(
        &mut user.tx, tenant_id, student_id, cert_id, certified_by, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(response)))
}
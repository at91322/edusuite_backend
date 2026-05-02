// src/modules/finance/handlers/aid.rs
// Group 2 — Financial Aid (reads + writes)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::finance::{
    models::{AidAwardListResponse, ListAidAwardsParams},
    queries,
    write_models::{CreateAidAwardRequest, UpdateAidAwardRequest},
    write_queries,
};

// ── GET /finance/students/:student_id/fafsa ───────────────────────────────────

pub async fn list_fafsa_records(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let records = queries::list_fafsa_records(&mut user.tx, student_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(records))
}

// ── GET /finance/students/:student_id/fafsa/:id ───────────────────────────────

pub async fn get_fafsa_record(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((student_id, fafsa_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let record = queries::get_fafsa_record(&mut user.tx, student_id, fafsa_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    match record {
        None    => Err(AppError::NotFound(
            format!("FAFSA record {} not found for student {}", fafsa_id, student_id)
        )),
        Some(r) => Ok(Json(r)),
    }
}

// ── GET /finance/students/:student_id/aid-awards ──────────────────────────────

pub async fn list_student_aid_awards(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
    Query(params): Query<ListAidAwardsParams>,
) -> Result<impl IntoResponse, AppError> {
    let (awards, total) = queries::list_aid_awards(
        &mut user.tx, Some(student_id), &params,
    ).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(AidAwardListResponse { data: awards, page, per_page, total, total_pages }))
}

// ── GET /finance/students/:student_id/aid-awards/:id ─────────────────────────

pub async fn get_aid_award(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((student_id, award_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let award = queries::get_aid_award(&mut user.tx, student_id, award_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    match award {
        None    => Err(AppError::NotFound(
            format!("Aid award {} not found for student {}", award_id, student_id)
        )),
        Some(a) => Ok(Json(a)),
    }
}

// ── GET /finance/aid-awards (tenant-wide FA office view) ──────────────────────

pub async fn list_aid_awards(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListAidAwardsParams>,
) -> Result<impl IntoResponse, AppError> {
    let (awards, total) = queries::list_aid_awards(&mut user.tx, None, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(AidAwardListResponse { data: awards, page, per_page, total, total_pages }))
}

// ── POST /finance/students/:student_id/aid-awards ────────────────────────────

pub async fn create_aid_award(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
    Json(req): Json<CreateAidAwardRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;

    let response = write_queries::create_aid_award(
        &mut user.tx, tenant_id, student_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id  = %tenant_id,
        student_id = %student_id,
        award_id   = %response.award_id,
        aid_type   = %response.aid_type,
        amount     = response.offered_amount,
        "Aid award created"
    );

    Ok((StatusCode::CREATED, Json(response)))
}

// ── PATCH /finance/students/:student_id/aid-awards/:id ──────────────────────

pub async fn update_aid_award(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((student_id, award_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateAidAwardRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;

    let response = write_queries::update_aid_award(
        &mut user.tx, tenant_id, student_id, award_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id  = %tenant_id,
        student_id = %student_id,
        award_id   = %award_id,
        status     = %response.status,
        "Aid award updated"
    );

    Ok(Json(response))
}
// src/modules/finance/handlers/sap.rs
// Group 3 — Satisfactory Academic Progress

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::finance::{
    models::{ListSapEvaluationsParams, SapEvaluationListResponse},
    queries,
};

// ── GET /finance/sap/policies ─────────────────────────────────────────────────

pub async fn list_sap_policies(
    State(_state): State<AppState>,
    mut user: AuthUser,
) -> Result<impl IntoResponse, AppError> {

    let policies = queries::list_sap_policies(&mut user.tx).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(policies))
}

// ── GET /finance/sap/policies/:id ────────────────────────────────────────────

pub async fn get_sap_policy(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(policy_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let policy = queries::get_sap_policy(&mut user.tx, policy_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match policy {
        None    => Err(AppError::NotFound(format!("SAP policy {} not found", policy_id))),
        Some(p) => Ok(Json(p)),
    }
}

// ── GET /finance/students/:student_id/sap ─────────────────────────────────────

pub async fn list_sap_evaluations(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
    Query(params): Query<ListSapEvaluationsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (evals, total) = queries::list_sap_evaluations(
        &mut user.tx, student_id, &params,
    ).await?;

    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(SapEvaluationListResponse { data: evals, page, per_page, total, total_pages }))
}

// ── GET /finance/students/:student_id/sap/:id ────────────────────────────────

pub async fn get_sap_evaluation(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((student_id, eval_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {

    let eval = queries::get_sap_evaluation(&mut user.tx, student_id, eval_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match eval {
        None    => Err(AppError::NotFound(
            format!("SAP evaluation {} not found for student {}", eval_id, student_id)
        )),
        Some(e) => Ok(Json(e)),
    }
}

// ── GET /finance/students/:student_id/sap/:id/appeal ─────────────────────────

pub async fn get_sap_appeal(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((student_id, eval_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {

    let appeal = queries::get_sap_appeal(&mut user.tx, student_id, eval_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match appeal {
        None    => Err(AppError::NotFound(
            format!("No SAP appeal found for evaluation {}", eval_id)
        )),
        Some(a) => Ok(Json(a)),
    }
}
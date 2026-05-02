// src/modules/finance/handlers/payroll.rs
// Group 5 — Payroll / Staff Pay

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::finance::{
    models::{ListPayrollRunsParams, ListStaffPayStubsParams, PayrollRunListResponse, PayStubListResponse},
    queries,
};

// ── GET /finance/payroll/runs ─────────────────────────────────────────────────

pub async fn list_payroll_runs(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListPayrollRunsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (runs, total) = queries::list_payroll_runs(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(PayrollRunListResponse { data: runs, page, per_page, total, total_pages }))
}

// ── GET /finance/payroll/runs/:id ─────────────────────────────────────────────

pub async fn get_payroll_run(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(run_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let run = queries::get_payroll_run(&mut user.tx, run_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match run {
        None    => Err(AppError::NotFound(format!("Payroll run {} not found", run_id))),
        Some(r) => Ok(Json(r)),
    }
}

// ── GET /finance/payroll/runs/:id/stubs ───────────────────────────────────────

pub async fn list_run_stubs(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(run_id): Path<Uuid>,
    Query(params): Query<ListStaffPayStubsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (stubs, total) = queries::list_run_stubs(&mut user.tx, run_id, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(PayStubListResponse { data: stubs, page, per_page, total, total_pages }))
}

// ── GET /finance/payroll/runs/:id/stubs/:stub_id ──────────────────────────────

pub async fn get_stub_detail(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((run_id, stub_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {

    let stub = queries::get_stub_detail(&mut user.tx, run_id, stub_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match stub {
        None    => Err(AppError::NotFound(
            format!("Pay stub {} not found in run {}", stub_id, run_id)
        )),
        Some(s) => Ok(Json(s)),
    }
}

// ── GET /finance/payroll/item-types ───────────────────────────────────────────

pub async fn list_payroll_item_types(
    State(_state): State<AppState>,
    mut user: AuthUser,
) -> Result<impl IntoResponse, AppError> {

    let types = queries::list_payroll_item_types(&mut user.tx).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(types))
}

// ── GET /finance/staff/:staff_id/pay-stubs ────────────────────────────────────

pub async fn list_staff_pay_stubs(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(staff_id): Path<Uuid>,
    Query(params): Query<ListStaffPayStubsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (stubs, total) = queries::list_staff_pay_stubs(
        &mut user.tx, staff_id, &params,
    ).await?;

    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(PayStubListResponse { data: stubs, page, per_page, total, total_pages }))
}
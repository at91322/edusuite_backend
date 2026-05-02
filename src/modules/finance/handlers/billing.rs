// src/modules/finance/handlers/billing.rs
// Group 1 — Student Billing (reads + writes)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::finance::{
    models::{
        ListStudentAccountsParams, ListTransactionsParams,
        StudentAccountListResponse, TransactionListResponse,
    },
    queries,
    write_models::{
        CreateStudentAccountRequest, PostTransactionRequest, SetHoldRequest,
    },
    write_queries,
};

// ── GET /finance/student-accounts ────────────────────────────────────────────

pub async fn list_student_accounts(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListStudentAccountsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (accounts, total) = queries::list_student_accounts(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(StudentAccountListResponse { data: accounts, page, per_page, total, total_pages }))
}

// ── GET /finance/student-accounts/:id ────────────────────────────────────────

pub async fn get_student_account(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let account = queries::get_student_account(&mut user.tx, account_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match account {
        None    => Err(AppError::NotFound(format!("Student account {} not found", account_id))),
        Some(a) => Ok(Json(a)),
    }
}

// ── GET /finance/students/:student_id/account ────────────────────────────────

pub async fn get_account_by_student(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let account = queries::get_account_by_student(&mut user.tx, student_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match account {
        None    => Err(AppError::NotFound(
            format!("No financial account found for student {}", student_id)
        )),
        Some(a) => Ok(Json(a)),
    }
}

// ── GET /finance/student-accounts/:id/transactions ───────────────────────────

pub async fn list_transactions(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(params): Query<ListTransactionsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (txns, balance, total) = queries::list_transactions(
        &mut user.tx, account_id, &params,
    ).await?;

    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(TransactionListResponse {
        data:            txns,
        account_balance: balance,
        page,
        per_page,
        total,
        total_pages,
    }))
}

// ── GET /finance/student-accounts/:id/transactions/:tx_id ────────────────────

pub async fn get_transaction(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((account_id, tx_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {

    let txn = queries::get_transaction(&mut user.tx, account_id, tx_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match txn {
        None    => Err(AppError::NotFound(
            format!("Transaction {} not found on account {}", tx_id, account_id)
        )),
        Some(t) => Ok(Json(t)),
    }
}

// ── GET /finance/student-accounts/:id/hold ───────────────────────────────────

pub async fn get_hold_status(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let account = queries::get_student_account(&mut user.tx, account_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match account {
        None => Err(AppError::NotFound(
            format!("Student account {} not found", account_id)
        )),
        Some(a) => Ok(Json(crate::modules::finance::models::HoldStatus {
            account_id:      a.id,
            student_id:      a.student_id,
            is_hold_active:  a.is_hold_active,
            current_balance: a.current_balance,
        })),
    }
}

// ── POST /finance/students/:student_id/account ───────────────────────────────

pub async fn create_student_account(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<Uuid>,
    body: Option<Json<CreateStudentAccountRequest>>,
) -> Result<impl IntoResponse, AppError> {

    let req = body.map(|Json(r)| r).unwrap_or_default();
    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;

    let account = write_queries::create_student_account(
        &mut user.tx, tenant_id, student_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id  = %tenant_id,
        student_id = %student_id,
        account_id = %account.account_id,
        "Student financial account created"
    );

    Ok((StatusCode::CREATED, Json(account)))
}

// ── POST /finance/student-accounts/:id/transactions ──────────────────────────

pub async fn post_transaction(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(req): Json<PostTransactionRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;

    let response = write_queries::post_transaction(
        &mut user.tx, tenant_id, account_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(response)))
}

// ── PATCH /finance/student-accounts/:id/hold ─────────────────────────────────

pub async fn set_hold(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(req): Json<SetHoldRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;

    let response = write_queries::set_hold(
        &mut user.tx, tenant_id, account_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id   = %tenant_id,
        account_id  = %account_id,
        hold_active = req.is_hold_active,
        reason      = %req.reason,
        "Student account hold updated"
    );

    Ok(Json(response))
}
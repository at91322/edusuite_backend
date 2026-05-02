// src/modules/finance/handlers/reference.rs
// Group 6 — Reference Reads (fiscal years, GL accounts)

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::finance::{
    models::{GlAccountListResponse, ListGlAccountsParams},
    queries,
};

// ── GET /finance/fiscal-years ─────────────────────────────────────────────────

pub async fn list_fiscal_years(
    State(_state): State<AppState>,
    mut user: AuthUser,
) -> Result<impl IntoResponse, AppError> {

    let years = queries::list_fiscal_years(&mut user.tx).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(years))
}

// ── GET /finance/fiscal-years/:id ────────────────────────────────────────────

pub async fn get_fiscal_year(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(fy_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let year = queries::get_fiscal_year(&mut user.tx, fy_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match year {
        None    => Err(AppError::NotFound(format!("Fiscal year {} not found", fy_id))),
        Some(y) => Ok(Json(y)),
    }
}

// ── GET /finance/gl-accounts ──────────────────────────────────────────────────

pub async fn list_gl_accounts(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListGlAccountsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (accounts, total) = queries::list_gl_accounts(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(GlAccountListResponse { data: accounts, page, per_page, total, total_pages }))
}
// src/modules/core/handlers/audit.rs
// Group 5 — Audit log reads

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::core::{
    models::{AuditLogListResponse, ListAuditLogsParams},
    queries,
};

// ── GET /core/audit-logs ──────────────────────────────────────────────────────

pub async fn list_audit_logs(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListAuditLogsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (logs, total) = queries::list_audit_logs(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(AuditLogListResponse { data: logs, page, per_page, total, total_pages }))
}

// ── GET /core/audit-logs/:id ──────────────────────────────────────────────────

pub async fn get_audit_log(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(log_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let log = queries::get_audit_log(&mut user.tx, log_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match log {
        None    => Err(AppError::NotFound(format!("Audit log {} not found", log_id))),
        Some(l) => Ok(Json(l)),
    }
}
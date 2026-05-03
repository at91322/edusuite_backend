// src/modules/core/handlers/member.rs
// Group 6 — Tenant membership management

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::core::{
    models::{ListMembersParams, MemberListResponse},
    queries,
    write_models::PatchMemberRequest,
    write_queries,
};

// ── GET /core/members ─────────────────────────────────────────────────────────

pub async fn list_members(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListMembersParams>,
) -> Result<impl IntoResponse, AppError> {

    let (members, total) = queries::list_members(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(MemberListResponse { data: members, page, per_page, total, total_pages }))
}

// ── PATCH /core/members/:user_id ──────────────────────────────────────────────

pub async fn patch_member(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(target_user_id): Path<Uuid>,
    Json(req): Json<PatchMemberRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;
    let response = write_queries::patch_member(
        &mut user.tx, tenant_id, target_user_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id      = %tenant_id,
        target_user_id = %target_user_id,
        actor_id       = %user.claims.sub,
        "Tenant membership updated"
    );

    Ok(Json(response))
}

// ── DELETE /core/members/:user_id ─────────────────────────────────────────────

pub async fn delete_member(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(target_user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let tenant_id = user.claims.tenant_id;

    write_queries::delete_member(&mut user.tx, tenant_id, target_user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id      = %tenant_id,
        target_user_id = %target_user_id,
        actor_id       = %user.claims.sub,
        "Tenant membership revoked"
    );

    Ok(StatusCode::NO_CONTENT)
}
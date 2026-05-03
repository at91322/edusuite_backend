// src/modules/core/handlers/role.rs
// Group 4 — Role management (reads + writes)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::core::{
    queries,
    write_models::GrantRoleRequest,
    write_queries,
};

// ── GET /core/users/:id/roles ─────────────────────────────────────────────────

pub async fn list_roles(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let roles = queries::list_roles(&mut user.tx, user_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;
    Ok(Json(roles))
}

// ── POST /core/users/:id/roles ────────────────────────────────────────────────

pub async fn grant_role(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(req): Json<GrantRoleRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id  = user.claims.tenant_id;
    let grantor_id = user.claims.sub;

    let result = write_queries::grant_role(
        &mut user.tx, tenant_id, user_id, grantor_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(result)))
}

// ── DELETE /core/users/:id/roles/:role_name ───────────────────────────────────

pub async fn revoke_role(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((user_id, role_name)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {

    let tenant_id = user.claims.tenant_id;

    write_queries::revoke_role(
        &mut user.tx, tenant_id, user_id, &role_name,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
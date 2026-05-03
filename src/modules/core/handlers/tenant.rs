// src/modules/core/handlers/tenant.rs
// Group 1 — Tenant self-service + Department management

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use crate::modules::core::{
    models::{ListDepartmentsParams, DepartmentListResponse},
    queries,
    write_models::{CreateDepartmentRequest, PatchDepartmentRequest, PatchTenantRequest},
    write_queries,
};

// ── GET /core/tenants/me ──────────────────────────────────────────────────────

pub async fn get_tenant_me(
    State(_state): State<AppState>,
    mut user: AuthUser,
) -> Result<impl IntoResponse, AppError> {

    let tenant = queries::get_tenant(&mut user.tx).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match tenant {
        None    => Err(AppError::NotFound("Tenant not found".into())),
        Some(t) => Ok(Json(t)),
    }
}

// ── PATCH /core/tenants/me ────────────────────────────────────────────────────

pub async fn patch_tenant_me(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Json(req): Json<PatchTenantRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let updated = write_queries::patch_tenant(&mut user.tx, &req).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id = %user.claims.tenant_id,
        "Tenant self-service update applied"
    );

    Ok(Json(updated))
}

// ── GET /core/tenants/me/subscriptions ───────────────────────────────────────

pub async fn get_tenant_subscriptions(
    State(_state): State<AppState>,
    mut user: AuthUser,
) -> Result<impl IntoResponse, AppError> {

    let subs = queries::get_tenant_subscriptions(&mut user.tx).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(subs))
}

// ── GET /core/tenants/me/feature-flags ───────────────────────────────────────

pub async fn get_feature_flags(
    State(_state): State<AppState>,
    mut user: AuthUser,
) -> Result<impl IntoResponse, AppError> {

    let flags = queries::get_feature_flags(&mut user.tx).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(flags))
}

// ── GET /core/tenants/me/departments ─────────────────────────────────────────

pub async fn list_departments(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListDepartmentsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (depts, total) = queries::list_departments(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(DepartmentListResponse { data: depts, page, per_page, total, total_pages }))
}

// ── GET /core/tenants/me/departments/:id ─────────────────────────────────────

pub async fn get_department(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(dept_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let dept = queries::get_department(&mut user.tx, dept_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match dept {
        None    => Err(AppError::NotFound(format!("Department {} not found", dept_id))),
        Some(d) => Ok(Json(d)),
    }
}

// ── POST /core/tenants/me/departments ────────────────────────────────────────

pub async fn create_department(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;
    let dept = write_queries::create_department(&mut user.tx, tenant_id, &req).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id  = %tenant_id,
        dept_id    = %dept.id,
        code       = %dept.code,
        "Department created"
    );

    Ok((StatusCode::CREATED, Json(dept)))
}

// ── PATCH /core/tenants/me/departments/:id ────────────────────────────────────

pub async fn update_department(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(dept_id): Path<Uuid>,
    Json(req): Json<PatchDepartmentRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;
    let dept = write_queries::patch_department(&mut user.tx, tenant_id, dept_id, &req).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id = %tenant_id,
        dept_id   = %dept_id,
        "Department updated"
    );

    Ok(Json(dept))
}
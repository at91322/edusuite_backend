// src/modules/lms/handlers.rs

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use super::{
    models::{CourseSectionsParams, ListSectionsParams, SectionListResponse},
    queries,
};

// ── GET /lms/sections (existing, unchanged) ───────────────────────────────────

pub async fn list_sections(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListSectionsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (sections, total) = queries::list_sections(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::debug!(
        tenant_id = %user.claims.tenant_id,
        count = sections.len(),
        total,
        page,
        "GET /lms/sections"
    );

    Ok(Json(SectionListResponse { data: sections, page, per_page, total, total_pages }))
}

// GET /lms/sections/:id ────────────────────────────────────────────────────────

pub async fn get_section(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(section_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let section = queries::get_section(&mut user.tx, section_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match section {
        None => Err(AppError::NotFound(
            format!("Section {} not found", section_id)
        )),
        Some(s) => {
            tracing::debug!(
                tenant_id  = %user.claims.tenant_id,
                section_id = %section_id,
                "GET /lms/sections/:id"
            );
            Ok(Json(s))
        }
    }
}

// GET /sis/courses/:id/sections ────────────────────────────────────────────────
// (handler lives in lms because the response shape is LMS-enriched SectionSummary)

pub async fn list_sections_for_course(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(course_id): Path<Uuid>,
    Query(params): Query<CourseSectionsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (sections, total) = queries::list_sections_for_course(
        &mut user.tx, course_id, &params,
    ).await?;

    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::debug!(
        tenant_id = %user.claims.tenant_id,
        course_id = %course_id,
        count = sections.len(),
        total,
        page,
        "GET /sis/courses/:id/sections"
    );

    Ok(Json(SectionListResponse { data: sections, page, per_page, total, total_pages }))
}
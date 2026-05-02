// src/modules/lms/handlers.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use super::{
    models::{CourseSectionsParams, ListSectionsParams, SectionListResponse},
    queries,
    write_models::{
        CreateAssignmentRequest, CreateModuleRequest, UpdateGradeEntryRequest,
    },
    write_queries,
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

// POST /lms/sections/:id/modules ─────────────────────────────────────────────

pub async fn create_module(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(section_id): Path<Uuid>,
    Json(req): Json<CreateModuleRequest>,
) -> Result<impl IntoResponse, AppError> {

    let tenant_id = user.claims.tenant_id;

    let module = write_queries::create_module(
        &mut user.tx, tenant_id, section_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id  = %tenant_id,
        section_id = %section_id,
        module_id  = %module.module_id,
        "Course module created"
    );

    Ok((StatusCode::CREATED, Json(module)))
}

// POST /lms/sections/:id/assignments ──────────────────────────────────────────

pub async fn create_assignment(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(section_id): Path<Uuid>,
    Json(req): Json<CreateAssignmentRequest>,
) -> Result<impl IntoResponse, AppError> {

    let tenant_id = user.claims.tenant_id;

    let assignment = write_queries::create_assignment(
        &mut user.tx, tenant_id, section_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id     = %tenant_id,
        section_id    = %section_id,
        assignment_id = %assignment.assignment_id,
        "Assignment created"
    );

    Ok((StatusCode::CREATED, Json(assignment)))
}

// PATCH /lms/grade-roster-entries/:id ─────────────────────────────────────────

pub async fn update_grade_entry(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(entry_id): Path<Uuid>,
    Json(req): Json<UpdateGradeEntryRequest>,
) -> Result<impl IntoResponse, AppError> {

    let tenant_id = user.claims.tenant_id;
    let user_id   = user.claims.sub;

    let entry = write_queries::update_grade_entry(
        &mut user.tx, tenant_id, entry_id, user_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::debug!(
        tenant_id = %tenant_id,
        entry_id  = %entry_id,
        "Grade roster entry updated"
    );

    Ok(Json(entry))
}

// POST /lms/grade-roster-submissions/:id/submit ───────────────────────────────

pub async fn submit_grade_roster(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(roster_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let tenant_id = user.claims.tenant_id;
    let user_id   = user.claims.sub;

    let response = write_queries::submit_grade_roster(
        &mut user.tx, tenant_id, roster_id, user_id,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id = %tenant_id,
        roster_id = %roster_id,
        entry_count = response.entry_count,
        "Grade roster submitted"
    );

    Ok(Json(response))
}

// POST /lms/grade-roster-submissions/:id/post ─────────────────────────────────

pub async fn post_grade_roster(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(roster_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let tenant_id = user.claims.tenant_id;
    let user_id   = user.claims.sub;

    let response = write_queries::post_grade_roster(
        &mut user.tx, tenant_id, roster_id, user_id,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id                  = %tenant_id,
        roster_id                  = %roster_id,
        transcript_records_written = response.transcript_records_written,
        incomplete_entries         = response.incomplete_entries,
        excused_entries            = response.excused_entries,
        "Grade roster posted; transcript records written"
    );

    Ok(Json(response))
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
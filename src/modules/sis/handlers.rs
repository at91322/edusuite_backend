// src/modules/sis/handlers.rs
//
// All SIS HTTP handlers. Each handler:
//   1. Extracts AuthUser (which opens the RLS-scoped transaction)
//   2. Delegates all DB work to queries:: or write_queries::
//   3. Commits on success — the extractor rolls back on drop if not committed
//
// Role-gating (staff/admin only)
// will be added in a future middleware pass over all SIS detail endpoints.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use super::{models::*, queries};

// ── GET /sis/students ─────────────────────────────────────────────────────────

pub async fn list_students(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListStudentsParams>,
) -> Result<impl IntoResponse, AppError> {

    let (students, total) = queries::list_students(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(StudentListResponse { data: students, page, per_page, total, total_pages }))
}

// ── GET /sis/students/:id ─────────────────────────────────────────────────────

pub async fn get_student(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let student = queries::get_student(&mut user.tx, student_id).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    match student {
        None => Err(AppError::NotFound(
            format!("Student {} not found", student_id)
        )),
        Some(s) => {
            tracing::debug!(
                tenant_id  = %user.claims.tenant_id,
                student_id = %student_id,
                "GET /sis/students/:id"
            );
            Ok(Json(s))
        }
    }
}

// ── GET /sis/courses ──────────────────────────────────────────────────────────

pub async fn list_courses(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Query(params): Query<ListCoursesParams>,
) -> Result<impl IntoResponse, AppError> {

    let (courses, total) = queries::list_courses(&mut user.tx, &params).await?;
    let per_page    = params.per_page();
    let page        = params.page();
    let total_pages = (total + per_page - 1) / per_page;

    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(CourseListResponse { data: courses, page, per_page, total, total_pages }))
}

// ── GET /sis/courses/:id ──────────────────────────────────────────────────────

pub async fn get_course(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(course_id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let course = queries::get_course(&mut user.tx, course_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    match course {
        None    => Err(AppError::NotFound(format!("Course {} not found", course_id))),
        Some(c) => Ok(Json(c)),
    }
}

// ── GET /sis/students/:id/enrollments ────────────────────────────────────────

pub async fn get_student_enrollments(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {

    let enrollments = queries::get_student_enrollments(&mut user.tx, student_id).await?;
    user.tx.commit().await.map_err(AppError::from)?;

    Ok(Json(enrollments))
}

// ── POST /sis/students ────────────────────────────────────────────────────────

pub async fn create_student(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Json(req): Json<super::write_models::CreateStudentRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;
    let user_id   = super::write_queries::create_student(
        &mut user.tx, tenant_id, &req,
    ).await?;

    let detail = queries::get_student(&mut user.tx, user_id)
        .await?
        .ok_or_else(|| AppError::Internal(
            anyhow::anyhow!("Student created but not found on re-fetch")
        ))?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id = %tenant_id,
        user_id   = %user_id,
        username  = %req.username,
        "Student created"
    );

    Ok((StatusCode::CREATED, Json(detail)))
}

// ── PATCH /sis/students/:id ───────────────────────────────────────────────────

pub async fn update_student(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<uuid::Uuid>,
    Json(req): Json<super::write_models::UpdateStudentRequest>,
) -> Result<impl IntoResponse, AppError> {

    if !req.has_changes() {
        return Err(AppError::BadRequest(
            "Request body contains no fields to update".into()
        ));
    }

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;
    super::write_queries::update_student(
        &mut user.tx, tenant_id, student_id, &req,
    ).await?;

    let detail = queries::get_student(&mut user.tx, student_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Student {} not found", student_id)))?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id  = %tenant_id,
        student_id = %student_id,
        "Student updated"
    );

    Ok(Json(detail))
}

// POST /sis/students/:id/enrollments ───────────────────────────────────────────

pub async fn create_student_enrollment(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path(student_id): Path<uuid::Uuid>,
    Json(req): Json<super::write_models::CreateEnrollmentRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;

    let response = super::write_queries::create_enrollment(
        &mut user.tx, tenant_id, student_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id     = %tenant_id,
        student_id    = %student_id,
        section_id    = %req.section_id,
        status        = %response.status,
        enrollment_id = %response.enrollment_id,
        "Student enrollment created"
    );

    Ok((StatusCode::CREATED, Json(response)))
}

// DELETE /sis/students/:id/enrollments/:enrollment_id ──────────────────────────

pub async fn delete_student_enrollment(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Path((student_id, enrollment_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<impl IntoResponse, AppError> {

    let tenant_id = user.claims.tenant_id;

    super::write_queries::drop_enrollment(
        &mut user.tx, tenant_id, student_id, enrollment_id,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id     = %tenant_id,
        student_id    = %student_id,
        enrollment_id = %enrollment_id,
        "Enrollment dropped"
    );

    // 204 No Content — the record still exists in the DB but is now 'dropped'
    Ok(StatusCode::NO_CONTENT)
}

// POST /sis/enrollments  (cross-module) ────────────────────────────────────────

/// Cross-module enrollment endpoint — touches sis.enrollments AND
/// lms.section_grade_summaries in one atomic transaction.
///
/// This is the primary registrar-facing enrollment workflow. For bulk
/// imports or self-service registration the same function is reused.
pub async fn cross_module_enroll(
    State(_state): State<AppState>,
    mut user: AuthUser,
    Json(req): Json<super::write_models::CrossModuleEnrollRequest>,
) -> Result<impl IntoResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let tenant_id = user.claims.tenant_id;

    let response = super::write_queries::cross_module_enroll(
        &mut user.tx, tenant_id, &req,
    ).await?;

    user.tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        tenant_id          = %tenant_id,
        student_id         = %req.student_id,
        section_id         = %req.section_id,
        status             = %response.status,
        enrollment_id      = %response.enrollment_id,
        lms_summary_id     = %response.lms_grade_summary_id,
        "Cross-module enrollment created (SIS + LMS)"
    );

    Ok((StatusCode::CREATED, Json(response)))
}
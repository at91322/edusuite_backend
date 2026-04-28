// src/modules/sis/handlers.rs

use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};

use crate::{
    error::AppError,
    http::auth::AuthUser,
    state::AppState,
    modules::sis::{
        models::{ListStudentsParams, StudentListResponse},
        queries,
    },
};

// ── GET /sis/students ─────────────────────────────────────────────────────────
//
// Returns a paginated list of students for the authenticated tenant.
//
// This handler is intentionally narrow — it returns safe summary fields only.
// FERPA-restricted data (date_of_birth, SSN fragment, race/ethnicity) is
// excluded. A future GET /sis/students/:id endpoint with role enforcement
// will expose the full student record.
//
// Query parameters:
//   ?standing=good_standing|academic_probation|academic_suspension|...
//   ?program_id=<uuid>
//   ?page=1
//   ?per_page=25  (max 100)
//
// Example:
//   GET /api/v1/sis/students?standing=academic_probation&page=1&per_page=50

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

    tracing::debug!(
        tenant_id  = %user.claims.tenant_id,
        user_id    = %user.claims.sub,
        count      = students.len(),
        total      = total,
        page       = page,
        "GET /sis/students"
    );

    Ok(Json(StudentListResponse {
        data: students,
        page,
        per_page,
        total,
        total_pages,
    }))
}

// ── GET /sis/students/:id ─────────────────────────────────────────────────────
//
// Returns the full student record for a single student.
// Returns 404 if the student_id does not belong to the authenticated tenant
// (RLS ensures cross-tenant data is invisible, not a 403).
//
// FERPA: demographics fields are included. Role-gating (staff/admin only)
// will be added in a future middleware pass over all SIS detail endpoints.

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
    Query(params): Query<crate::modules::sis::models::ListCoursesParams>,
) -> Result<impl IntoResponse, AppError> {
    use crate::modules::sis::models::CourseListResponse;

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
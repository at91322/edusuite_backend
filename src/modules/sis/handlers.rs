// src/modules/sis/handlers.rs

use axum::{extract::{Query, State}, response::IntoResponse, Json};

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
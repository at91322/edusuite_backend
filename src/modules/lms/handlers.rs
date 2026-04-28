// src/modules/lms/handlers.rs

use axum::{extract::{Query, State}, response::IntoResponse, Json};

use crate::{error::AppError, http::auth::AuthUser, state::AppState};
use super::{models::{ListSectionsParams, SectionListResponse}, queries};

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
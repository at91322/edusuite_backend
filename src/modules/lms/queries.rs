// src/modules/lms/queries.rs

use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use super::models::{ListSectionsParams, SectionSummary};

pub async fn list_sections(
    tx:     &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &ListSectionsParams,
) -> Result<(Vec<SectionSummary>, i64), AppError> {

    // Enrolled count uses sis.enrollments — active enrollments only
    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM sis.course_sections cs
        WHERE cs.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND cs.is_current = true
          AND ($1::uuid IS NULL OR cs.term_id       = $1)
          AND ($2::uuid IS NULL OR cs.course_id     = $2)
          AND ($3::uuid IS NULL OR cs.instructor_id = $3)
          AND ($4::bool IS NULL OR cs.is_published  = $4)
        "#,
        params.term_id       as Option<Uuid>,
        params.course_id     as Option<Uuid>,
        params.instructor_id as Option<Uuid>,
        params.is_published  as Option<bool>,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let rows = sqlx::query(
        r#"
        SELECT
            cs.id                               AS id,
            cs.section_number                   AS section_number,
            cs.course_id                        AS course_id,
            c.subject                           AS course_subject,
            c.course                            AS course_number,
            c.title                             AS course_title,
            cs.term_id                          AS term_id,
            t.name                              AS term_name,
            t.status::text                      AS term_status,
            cs.instructor_id                    AS instructor_id,
            u.first_name || ' ' || u.last_name  AS instructor_name,
            cs.instructional_format::text       AS instructional_format,
            cs.max_enrollment                   AS max_enrollment,
            cs.is_published                     AS is_published,
            COUNT(e.id)                         AS enrolled_count
        FROM sis.course_sections cs
        JOIN sis.courses c       ON c.id  = cs.course_id
        JOIN sis.academic_terms t ON t.id = cs.term_id
        JOIN core.users u        ON u.id  = cs.instructor_id
        LEFT JOIN sis.enrollments e
               ON e.section_id = cs.id
              AND e.status = 'enrolled'
        WHERE cs.tenant_id  = current_setting('app.current_tenant_id', true)::uuid
          AND cs.is_current = true
          AND ($1::uuid IS NULL OR cs.term_id       = $1)
          AND ($2::uuid IS NULL OR cs.course_id     = $2)
          AND ($3::uuid IS NULL OR cs.instructor_id = $3)
          AND ($4::bool IS NULL OR cs.is_published  = $4)
        GROUP BY
            cs.id, cs.section_number, cs.course_id,
            c.subject, c.course, c.title,
            cs.term_id, t.name, t.status, t.start_date,
            cs.instructor_id, u.first_name, u.last_name,
            cs.instructional_format, cs.max_enrollment, cs.is_published
        ORDER BY t.start_date DESC, c.subject ASC, c.course ASC, cs.section_number ASC
        LIMIT  $5
        OFFSET $6
        "#,
    )
    .bind(params.term_id)
    .bind(params.course_id)
    .bind(params.instructor_id)
    .bind(params.is_published)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let sections = rows.iter().map(|r| -> Result<SectionSummary, AppError> {
        Ok(SectionSummary {
            id:                   r.try_get("id").map_err(AppError::from)?,
            section_number:       r.try_get("section_number").map_err(AppError::from)?,
            course_id:            r.try_get("course_id").map_err(AppError::from)?,
            course_subject:       r.try_get("course_subject").map_err(AppError::from)?,
            course_number:        r.try_get("course_number").map_err(AppError::from)?,
            course_title:         r.try_get("course_title").map_err(AppError::from)?,
            term_id:              r.try_get("term_id").map_err(AppError::from)?,
            term_name:            r.try_get("term_name").map_err(AppError::from)?,
            term_status:          r.try_get("term_status").map_err(AppError::from)?,
            instructor_id:        r.try_get("instructor_id").map_err(AppError::from)?,
            instructor_name:      r.try_get("instructor_name").map_err(AppError::from)?,
            instructional_format: r.try_get("instructional_format").map_err(AppError::from)?,
            max_enrollment:       r.try_get("max_enrollment").map_err(AppError::from)?,
            is_published:         r.try_get("is_published").map_err(AppError::from)?,
            enrolled_count:       r.try_get("enrolled_count").map_err(AppError::from)?,
        })
    }).collect::<Result<Vec<_>, _>>()?;

    Ok((sections, total))
}
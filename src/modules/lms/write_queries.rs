// src/modules/lms/write_queries.rs
//
// Write queries for the LMS module:
//   POST /lms/sections/:id/modules           → create_module
//   POST /lms/sections/:id/assignments       → create_assignment
//   PATCH /lms/grade-roster-entries/:id      → update_grade_entry
//   POST /lms/grade-roster-submissions/:id/submit → submit_grade_roster
//   POST /lms/grade-roster-submissions/:id/post   → post_grade_roster
//
// Grade roster → transcript cross-module write invariants:
//   • One DB transaction covers the entire post operation — partial writes
//     are impossible; either all transcript records are written or none are.
//   • Incomplete entries are NOT written to the transcript at posting time.
//     They are deferred until the incomplete is resolved (separate endpoint).
//   • Excused entries produce no transcript record.
//   • repeat_instance is derived from MAX(repeat_instance) for the same
//     (tenant, student, course, term) tuple — handles course repeats correctly.

use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use super::write_models::{
    CreateModuleRequest,     CreateModuleResponse,
    CreateAssignmentRequest, CreateAssignmentResponse,
    UpdateGradeEntryRequest, GradeEntryResponse,
    GradeRosterSubmitResponse,
    GradeRosterPostResponse,
};

// ── POST /lms/sections/:id/modules ───────────────────────────────────────────

pub async fn create_module(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    section_id: Uuid,
    req:        &CreateModuleRequest,
) -> Result<CreateModuleResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let section_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.course_sections
            WHERE id        = $1
              AND tenant_id = $2
              AND is_current = true
        ) AS "exists!"
        "#,
        section_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !section_exists {
        return Err(AppError::NotFound(format!("Section {} not found", section_id)));
    }

    let row = sqlx::query(
        r#"
        INSERT INTO lms.course_modules
            (tenant_id, section_id, title, description, order_index, is_published)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, order_index, is_published, created_at
        "#,
    )
    .bind(tenant_id)
    .bind(section_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.order_index.unwrap_or(0))
    .bind(req.is_published.unwrap_or(false))
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(CreateModuleResponse {
        module_id:    row.try_get("id").map_err(AppError::from)?,
        section_id,
        title:        req.title.clone(),
        description:  req.description.clone(),
        order_index:  row.try_get("order_index").map_err(AppError::from)?,
        is_published: row.try_get("is_published").map_err(AppError::from)?,
        created_at:   row.try_get("created_at").map_err(AppError::from)?,
    })
}

// ── POST /lms/sections/:id/assignments ───────────────────────────────────────

pub async fn create_assignment(
    tx:         &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id:  Uuid,
    section_id: Uuid,
    req:        &CreateAssignmentRequest,
) -> Result<CreateAssignmentResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    let section_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM sis.course_sections
            WHERE id        = $1
              AND tenant_id = $2
              AND is_current = true
        ) AS "exists!"
        "#,
        section_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !section_exists {
        return Err(AppError::NotFound(format!("Section {} not found", section_id)));
    }

    // Guard: module must belong to this section
    let module_exists: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM lms.course_modules
            WHERE id         = $1
              AND section_id = $2
              AND tenant_id  = $3
        ) AS "exists!"
        "#,
        req.module_id,
        section_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if !module_exists {
        return Err(AppError::NotFound(format!(
            "Module {} not found in section {}",
            req.module_id, section_id
        )));
    }

    let assignment_type = req.assignment_type.as_deref().unwrap_or("homework");
    let category        = req.category.as_deref().unwrap_or("formative");
    let max_score       = req.max_score.unwrap_or(100.0);

    let row = sqlx::query(
        r#"
        INSERT INTO lms.assignments
            (tenant_id, section_id, module_id, title, description,
             type, category, max_score, due_date,
             allow_late_submissions, is_published,
             rubric_id, lti_resource_link_id, lti_line_item_id,
             learning_package_id, group_id, assessment_id)
        VALUES ($1, $2, $3, $4, $5,
                $6::lms.assignment_type, $7::lms.assignment_category, $8::numeric, $9,
                $10, $11,
                $12, $13, $14,
                $15, $16, $17)
        RETURNING id,
                  type::text     AS assignment_type,
                  category::text AS category,
                  max_score::float8,
                  allow_late_submissions,
                  is_published,
                  created_at
        "#,
    )
    .bind(tenant_id)
    .bind(section_id)
    .bind(req.module_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(assignment_type)
    .bind(category)
    .bind(max_score)
    .bind(req.due_date)
    .bind(req.allow_late_submissions.unwrap_or(false))
    .bind(req.is_published.unwrap_or(false))
    .bind(req.rubric_id)
    .bind(req.lti_resource_link_id)
    .bind(req.lti_line_item_id.as_deref())
    .bind(req.learning_package_id)
    .bind(req.group_id)
    .bind(req.assessment_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(CreateAssignmentResponse {
        assignment_id:          row.try_get("id").map_err(AppError::from)?,
        section_id,
        module_id:              req.module_id,
        title:                  req.title.clone(),
        description:            req.description.clone(),
        assignment_type:        row.try_get("assignment_type").map_err(AppError::from)?,
        category:               row.try_get("category").map_err(AppError::from)?,
        max_score:              row.try_get("max_score").map_err(AppError::from)?,
        due_date:               req.due_date,
        allow_late_submissions: row.try_get("allow_late_submissions").map_err(AppError::from)?,
        is_published:           row.try_get("is_published").map_err(AppError::from)?,
        created_at:             row.try_get("created_at").map_err(AppError::from)?,
    })
}

// ── PATCH /lms/grade-roster-entries/:id ──────────────────────────────────────

/// Update a single grade roster entry using merge semantics.
///
/// Reads the current row, merges supplied fields, validates the merged state
/// against DB constraints, then writes the full merged state in one UPDATE.
/// This ensures the DB constraint check (grade_roster_entry_must_have_outcome
/// etc.) is reflected in a human-readable API error before the DB rejects it.
pub async fn update_grade_entry(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    entry_id:  Uuid,
    user_id:   Uuid,
    req:       &UpdateGradeEntryRequest,
) -> Result<GradeEntryResponse, AppError> {

    req.validate().map_err(|errs| AppError::BadRequest(errs.join("; ")))?;

    // Fetch current state plus the parent roster's status in one query.
    let existing = sqlx::query(
        r#"
        SELECT
            gre.id,
            gre.roster_id,
            gre.enrollment_id,
            gre.student_id,
            gre.final_letter_grade,
            gre.final_quality_points::float8  AS final_quality_points,
            gre.final_percentage::float8       AS final_percentage,
            gre.is_incomplete,
            gre.incomplete_deadline,
            gre.incomplete_default_grade,
            gre.is_excused,
            gre.is_registrar_override,
            gre.override_reason,
            gre.entered_by_user_id,
            gre.entered_at,
            gre.updated_at,
            grs.status::text                   AS roster_status
        FROM lms.grade_roster_entries gre
        JOIN lms.grade_roster_submissions grs ON grs.id = gre.roster_id
        WHERE gre.id        = $1
          AND gre.tenant_id = $2
        "#,
    )
    .bind(entry_id)
    .bind(tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let existing = match existing {
        None    => return Err(AppError::NotFound(
            format!("Grade roster entry {} not found", entry_id)
        )),
        Some(r) => r,
    };

    let roster_status: String = existing.try_get("roster_status").map_err(AppError::from)?;
    if roster_status != "open" {
        return Err(AppError::BadRequest(format!(
            "Grade roster is '{}'; only 'open' rosters accept grade changes",
            roster_status
        )));
    }

    // Merge request fields over existing values.
    let cur_letter:    Option<String>     = existing.try_get("final_letter_grade").map_err(AppError::from)?;
    let cur_qp:        Option<f64>        = existing.try_get("final_quality_points").map_err(AppError::from)?;
    let cur_pct:       Option<f64>        = existing.try_get("final_percentage").map_err(AppError::from)?;
    let cur_incomplete: bool              = existing.try_get("is_incomplete").map_err(AppError::from)?;
    let cur_deadline:  Option<chrono::NaiveDate> = existing.try_get("incomplete_deadline").map_err(AppError::from)?;
    let cur_def_grade: Option<String>     = existing.try_get("incomplete_default_grade").map_err(AppError::from)?;
    let cur_excused:   bool               = existing.try_get("is_excused").map_err(AppError::from)?;
    let cur_override:  bool               = existing.try_get("is_registrar_override").map_err(AppError::from)?;
    let cur_reason:    Option<String>     = existing.try_get("override_reason").map_err(AppError::from)?;

    let new_letter     = req.final_letter_grade.as_ref().or(cur_letter.as_ref()).cloned();
    let new_qp         = req.final_quality_points.or(cur_qp);
    let new_pct        = req.final_percentage.or(cur_pct);
    let new_incomplete = req.is_incomplete.unwrap_or(cur_incomplete);
    let new_deadline   = req.incomplete_deadline.or(cur_deadline);
    let new_def_grade  = req.incomplete_default_grade.as_ref().or(cur_def_grade.as_ref()).cloned();
    let new_excused    = req.is_excused.unwrap_or(cur_excused);
    let new_override   = req.is_registrar_override.unwrap_or(cur_override);
    let new_reason     = req.override_reason.as_ref().or(cur_reason.as_ref()).cloned();

    // Validate merged state against DB constraints before issuing the UPDATE.
    if new_letter.is_none() && !new_incomplete && !new_excused {
        return Err(AppError::BadRequest(
            "Grade entry must have an outcome: set final_letter_grade, is_incomplete, or is_excused".into()
        ));
    }
    if new_incomplete && (new_deadline.is_none() || new_def_grade.is_none()) {
        return Err(AppError::BadRequest(
            "Incomplete entries require both incomplete_deadline and incomplete_default_grade".into()
        ));
    }
    if new_override && new_reason.is_none() {
        return Err(AppError::BadRequest(
            "Registrar override requires an override_reason".into()
        ));
    }

    let row = sqlx::query(
        r#"
        UPDATE lms.grade_roster_entries SET
            final_letter_grade       = $1,
            final_quality_points     = $2::numeric,
            final_percentage         = $3::numeric,
            is_incomplete            = $4,
            incomplete_deadline      = $5,
            incomplete_default_grade = $6,
            is_excused               = $7,
            is_registrar_override    = $8,
            override_reason          = $9,
            entered_by_user_id       = $10,
            entered_at               = now(),
            updated_at               = now()
        WHERE id        = $11
          AND tenant_id = $12
        RETURNING
            id,
            roster_id,
            enrollment_id,
            student_id,
            final_letter_grade,
            final_quality_points::float8  AS final_quality_points,
            final_percentage::float8      AS final_percentage,
            is_incomplete,
            incomplete_deadline,
            incomplete_default_grade,
            is_excused,
            is_registrar_override,
            override_reason,
            entered_by_user_id,
            entered_at,
            updated_at
        "#,
    )
    .bind(&new_letter)
    .bind(new_qp)
    .bind(new_pct)
    .bind(new_incomplete)
    .bind(new_deadline)
    .bind(&new_def_grade)
    .bind(new_excused)
    .bind(new_override)
    .bind(&new_reason)
    .bind(user_id)
    .bind(entry_id)
    .bind(tenant_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(GradeEntryResponse {
        entry_id:                 row.try_get("id").map_err(AppError::from)?,
        roster_id:                row.try_get("roster_id").map_err(AppError::from)?,
        enrollment_id:            row.try_get("enrollment_id").map_err(AppError::from)?,
        student_id:               row.try_get("student_id").map_err(AppError::from)?,
        final_letter_grade:       row.try_get("final_letter_grade").map_err(AppError::from)?,
        final_quality_points:     row.try_get("final_quality_points").map_err(AppError::from)?,
        final_percentage:         row.try_get("final_percentage").map_err(AppError::from)?,
        is_incomplete:            row.try_get("is_incomplete").map_err(AppError::from)?,
        incomplete_deadline:      row.try_get("incomplete_deadline").map_err(AppError::from)?,
        incomplete_default_grade: row.try_get("incomplete_default_grade").map_err(AppError::from)?,
        is_excused:               row.try_get("is_excused").map_err(AppError::from)?,
        is_registrar_override:    row.try_get("is_registrar_override").map_err(AppError::from)?,
        override_reason:          row.try_get("override_reason").map_err(AppError::from)?,
        entered_by_user_id:       row.try_get("entered_by_user_id").map_err(AppError::from)?,
        entered_at:               row.try_get("entered_at").map_err(AppError::from)?,
        updated_at:               row.try_get("updated_at").map_err(AppError::from)?,
    })
}

// ── POST /lms/grade-roster-submissions/:id/submit ────────────────────────────

/// Transition a grade roster from 'open' → 'submitted'.
///
/// Rejects the transition if any entries still lack an outcome so the
/// instructor is forced to resolve every student before submission.
pub async fn submit_grade_roster(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    roster_id: Uuid,
    user_id:   Uuid,
) -> Result<GradeRosterSubmitResponse, AppError> {

    let roster = sqlx::query(
        r#"
        SELECT id, section_id, term_id, status::text AS status
        FROM lms.grade_roster_submissions
        WHERE id = $1 AND tenant_id = $2
        "#,
    )
    .bind(roster_id)
    .bind(tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let roster = match roster {
        None    => return Err(AppError::NotFound(
            format!("Grade roster {} not found", roster_id)
        )),
        Some(r) => r,
    };

    let current_status: String = roster.try_get("status").map_err(AppError::from)?;
    if current_status != "open" {
        return Err(AppError::BadRequest(format!(
            "Grade roster is '{}'; only 'open' rosters can be submitted",
            current_status
        )));
    }

    // Every entry must have an outcome before submission is allowed.
    let ungraded_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM lms.grade_roster_entries
        WHERE roster_id = $1
          AND tenant_id = $2
          AND final_letter_grade IS NULL
          AND is_incomplete = false
          AND is_excused    = false
        "#,
        roster_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    if ungraded_count > 0 {
        return Err(AppError::BadRequest(format!(
            "{} {} without an outcome (grade, incomplete, or excused); all entries must be resolved before submission",
            ungraded_count,
            if ungraded_count == 1 { "entry" } else { "entries" }
        )));
    }

    let entry_count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint AS "count!"
        FROM lms.grade_roster_entries
        WHERE roster_id = $1 AND tenant_id = $2
        "#,
        roster_id,
        tenant_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let submitted_at = chrono::Utc::now();

    sqlx::query(
        r#"
        UPDATE lms.grade_roster_submissions SET
            status               = 'submitted'::lms.grade_roster_status,
            submitted_at         = $1,
            submitted_by_user_id = $2,
            updated_at           = now()
        WHERE id = $3 AND tenant_id = $4
        "#,
    )
    .bind(submitted_at)
    .bind(user_id)
    .bind(roster_id)
    .bind(tenant_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(GradeRosterSubmitResponse {
        roster_id,
        section_id:           roster.try_get("section_id").map_err(AppError::from)?,
        term_id:              roster.try_get("term_id").map_err(AppError::from)?,
        status:               "submitted".to_string(),
        submitted_at,
        submitted_by_user_id: user_id,
        entry_count,
    })
}

// ── POST /lms/grade-roster-submissions/:id/post ──────────────────────────────

/// Transition a grade roster from 'submitted' → 'posted' and write
/// sis.transcript_records for every graded enrollment.
///
/// This is the primary cross-module write in the system. The entire operation
/// runs inside one transaction — all transcript records are written atomically
/// or none are (the roster stays in 'submitted' on failure).
pub async fn post_grade_roster(
    tx:        &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    roster_id: Uuid,
    user_id:   Uuid,
) -> Result<GradeRosterPostResponse, AppError> {

    // Fetch roster with course credit info from the section → course join.
    let roster = sqlx::query(
        r#"
        SELECT
            grs.id,
            grs.section_id,
            grs.term_id,
            grs.status::text          AS status,
            c.id                      AS course_id,
            COALESCE(c.credits::float8, 0.0) AS course_credits
        FROM lms.grade_roster_submissions grs
        JOIN sis.course_sections cs ON cs.id = grs.section_id
        JOIN sis.courses c          ON c.id  = cs.course_id
        WHERE grs.id        = $1
          AND grs.tenant_id = $2
        "#,
    )
    .bind(roster_id)
    .bind(tenant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let roster = match roster {
        None    => return Err(AppError::NotFound(
            format!("Grade roster {} not found", roster_id)
        )),
        Some(r) => r,
    };

    let current_status: String = roster.try_get("status").map_err(AppError::from)?;
    if current_status != "submitted" {
        return Err(AppError::BadRequest(format!(
            "Grade roster is '{}'; only 'submitted' rosters can be posted",
            current_status
        )));
    }

    let course_id: Uuid   = roster.try_get("course_id").map_err(AppError::from)?;
    let term_id:   Uuid   = roster.try_get("term_id").map_err(AppError::from)?;
    let section_id: Uuid  = roster.try_get("section_id").map_err(AppError::from)?;
    let course_credits: f64 = roster
        .try_get::<f64, _>("course_credits")
        .or_else(|_| roster.try_get::<i32, _>("course_credits").map(|v| v as f64))
        .unwrap_or(0.0);

    // Load all entries for this roster in one query.
    let entries = sqlx::query(
        r#"
        SELECT
            id,
            student_id,
            final_letter_grade,
            final_quality_points::float8 AS final_quality_points,
            is_incomplete,
            is_excused
        FROM lms.grade_roster_entries
        WHERE roster_id = $1
          AND tenant_id = $2
        "#,
    )
    .bind(roster_id)
    .bind(tenant_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(AppError::from)?;

    let mut transcript_records_written: i64 = 0;
    let mut incomplete_entries:         i64 = 0;
    let mut excused_entries:            i64 = 0;

    for entry in &entries {
        let entry_id:    Uuid           = entry.try_get("id").map_err(AppError::from)?;
        let student_id:  Uuid           = entry.try_get("student_id").map_err(AppError::from)?;
        let is_excused:  bool           = entry.try_get("is_excused").map_err(AppError::from)?;
        let is_incomplete: bool         = entry.try_get("is_incomplete").map_err(AppError::from)?;
        let letter_grade: Option<String> = entry.try_get("final_letter_grade").map_err(AppError::from)?;
        let qp_per_credit: Option<f64>  = entry.try_get("final_quality_points").map_err(AppError::from)?;

        if is_excused {
            excused_entries += 1;
            continue;
        }

        if is_incomplete {
            // Transcript record is deferred until the incomplete is resolved.
            incomplete_entries += 1;
            continue;
        }

        // Compute transcript values.
        //   credits_earned: quality_points = 0 means failing → no credit earned.
        //   quality_points: total for this course = credits_attempted × QP/credit.
        let qp_rate         = qp_per_credit.unwrap_or(0.0);
        let credits_attempted = course_credits;
        let credits_earned  = if qp_rate > 0.0 { credits_attempted } else { 0.0 };
        let total_quality_points = credits_attempted * qp_rate;

        // Detect prior attempts for the same student/course/term to set repeat_instance.
        let repeat_row = sqlx::query(
            r#"
            SELECT COALESCE(MAX(repeat_instance), 0)::int AS max_repeat
            FROM sis.transcript_records
            WHERE student_id = $1
              AND course_id  = $2
              AND term_id    = $3
              AND tenant_id  = $4
            "#,
        )
        .bind(student_id)
        .bind(course_id)
        .bind(term_id)
        .bind(tenant_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        let max_repeat: i32    = repeat_row.try_get("max_repeat").map_err(AppError::from)?;
        let repeat_instance: i16 = (max_repeat + 1) as i16;

        // INSERT the transcript record.
        let transcript_row = sqlx::query(
            r#"
            INSERT INTO sis.transcript_records
                (tenant_id, student_id, course_id, term_id,
                 credits_attempted, credits_earned, final_letter_grade,
                 quality_points, record_type, repeat_instance, is_locked)
            VALUES ($1, $2, $3, $4,
                    $5::numeric, $6::numeric, $7,
                    $8::numeric, 'institutional'::sis.transcript_record_type, $9,
                    false)
            RETURNING id
            "#,
        )
        .bind(tenant_id)
        .bind(student_id)
        .bind(course_id)
        .bind(term_id)
        .bind(credits_attempted)
        .bind(credits_earned)
        .bind(&letter_grade)
        .bind(total_quality_points)
        .bind(repeat_instance)
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::from)?;

        let transcript_record_id: Uuid = transcript_row.try_get("id").map_err(AppError::from)?;

        // Link the transcript record back to the grade roster entry.
        sqlx::query(
            r#"
            UPDATE lms.grade_roster_entries SET
                transcript_record_id = $1,
                updated_at           = now()
            WHERE id        = $2
              AND tenant_id = $3
            "#,
        )
        .bind(transcript_record_id)
        .bind(entry_id)
        .bind(tenant_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::from)?;

        transcript_records_written += 1;
    }

    // Transition the roster to 'posted'.
    let posted_at = chrono::Utc::now();

    sqlx::query(
        r#"
        UPDATE lms.grade_roster_submissions SET
            status             = 'posted'::lms.grade_roster_status,
            posted_at          = $1,
            posted_by_user_id  = $2,
            updated_at         = now()
        WHERE id = $3 AND tenant_id = $4
        "#,
    )
    .bind(posted_at)
    .bind(user_id)
    .bind(roster_id)
    .bind(tenant_id)
    .execute(&mut **tx)
    .await
    .map_err(AppError::from)?;

    Ok(GradeRosterPostResponse {
        roster_id,
        section_id,
        term_id,
        status:                     "posted".to_string(),
        posted_at,
        posted_by_user_id:          user_id,
        transcript_records_written,
        incomplete_entries,
        excused_entries,
    })
}

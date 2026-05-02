// src/modules/lms/write_models.rs

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════════
// POST /lms/sections/:id/modules
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct CreateModuleRequest {
    pub title:        String,
    pub description:  Option<String>,
    pub order_index:  Option<i32>,
    pub is_published: Option<bool>,
}

impl CreateModuleRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.title.trim().is_empty() {
            errors.push("title is required".into());
        }
        if self.title.len() > 255 {
            errors.push("title must be 255 characters or fewer".into());
        }
        if let Some(idx) = self.order_index {
            if idx < 0 {
                errors.push("order_index must be >= 0".into());
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateModuleResponse {
    pub module_id:    Uuid,
    pub section_id:   Uuid,
    pub title:        String,
    pub description:  Option<String>,
    pub order_index:  i32,
    pub is_published: bool,
    pub created_at:   DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// POST /lms/sections/:id/assignments
// ═══════════════════════════════════════════════════════════════════════════════

/// Valid lms.assignment_type enum values.
pub const VALID_ASSIGNMENT_TYPES: &[&str] = &[
    "homework", "quiz", "exam", "essay", "project", "discussion",
];

/// Valid lms.assignment_category enum values.
pub const VALID_ASSIGNMENT_CATEGORIES: &[&str] = &[
    "formative", "summative",
];

#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    /// UUID of the lms.course_modules row this assignment belongs to.
    /// The module must belong to the same section as the path :id.
    pub module_id:             Uuid,
    pub title:                 String,
    pub description:           Option<String>,
    /// lms.assignment_type — defaults to "homework"
    pub assignment_type:       Option<String>,
    /// lms.assignment_category — defaults to "formative"
    pub category:              Option<String>,
    /// Defaults to 100.00. Must be > 0 and <= 999.99 (numeric(5,2) column).
    pub max_score:             Option<f64>,
    pub due_date:              DateTime<Utc>,
    pub allow_late_submissions: Option<bool>,
    pub is_published:          Option<bool>,
    // Optional linkage fields
    pub rubric_id:             Option<Uuid>,
    pub lti_resource_link_id:  Option<Uuid>,
    /// Max 255 chars (varchar(255) column).
    pub lti_line_item_id:      Option<String>,
    pub learning_package_id:   Option<Uuid>,
    pub group_id:              Option<Uuid>,
    pub assessment_id:         Option<Uuid>,
}

impl CreateAssignmentRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.title.trim().is_empty() {
            errors.push("title is required".into());
        }
        if self.title.len() > 255 {
            errors.push("title must be 255 characters or fewer".into());
        }

        if let Some(ref t) = self.assignment_type {
            if !VALID_ASSIGNMENT_TYPES.contains(&t.as_str()) {
                errors.push(format!(
                    "assignment_type '{}' is invalid; must be one of: {}",
                    t, VALID_ASSIGNMENT_TYPES.join(", ")
                ));
            }
        }

        if let Some(ref c) = self.category {
            if !VALID_ASSIGNMENT_CATEGORIES.contains(&c.as_str()) {
                errors.push(format!(
                    "category '{}' is invalid; must be one of: {}",
                    c, VALID_ASSIGNMENT_CATEGORIES.join(", ")
                ));
            }
        }

        if let Some(score) = self.max_score {
            if score <= 0.0 {
                errors.push("max_score must be greater than zero".into());
            }
            if score > 999.99 {
                errors.push("max_score cannot exceed 999.99".into());
            }
        }

        if let Some(ref id) = self.lti_line_item_id {
            if id.len() > 255 {
                errors.push("lti_line_item_id must be 255 characters or fewer".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateAssignmentResponse {
    pub assignment_id:          Uuid,
    pub section_id:             Uuid,
    pub module_id:              Uuid,
    pub title:                  String,
    pub description:            Option<String>,
    pub assignment_type:        String,
    pub category:               String,
    pub max_score:              f64,
    pub due_date:               DateTime<Utc>,
    pub allow_late_submissions: bool,
    pub is_published:           bool,
    pub created_at:             DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PATCH /lms/grade-roster-entries/:id
// ═══════════════════════════════════════════════════════════════════════════════

/// Partial update for a single grade roster entry.
///
/// DB constraints enforced after merging with existing row state:
///   grade_roster_entry_must_have_outcome:
///     final_letter_grade IS NOT NULL OR is_incomplete OR is_excused
///   grade_roster_incomplete_requires_deadline:
///     NOT is_incomplete OR (incomplete_deadline IS NOT NULL AND incomplete_default_grade IS NOT NULL)
///   grade_roster_override_requires_reason:
///     NOT is_registrar_override OR override_reason IS NOT NULL
///
/// Omitted fields retain their current values in the database.
/// The roster must be in 'open' status — submitted/posted/locked entries reject updates.
#[derive(Debug, Deserialize)]
pub struct UpdateGradeEntryRequest {
    pub final_letter_grade:      Option<String>,
    pub final_quality_points:    Option<f64>,
    pub final_percentage:        Option<f64>,
    pub is_incomplete:           Option<bool>,
    pub incomplete_deadline:     Option<NaiveDate>,
    pub incomplete_default_grade: Option<String>,
    pub is_excused:              Option<bool>,
    pub is_registrar_override:   Option<bool>,
    pub override_reason:         Option<String>,
}

impl UpdateGradeEntryRequest {
    pub fn has_changes(&self) -> bool {
        self.final_letter_grade.is_some()
            || self.final_quality_points.is_some()
            || self.final_percentage.is_some()
            || self.is_incomplete.is_some()
            || self.incomplete_deadline.is_some()
            || self.incomplete_default_grade.is_some()
            || self.is_excused.is_some()
            || self.is_registrar_override.is_some()
            || self.override_reason.is_some()
    }

    /// Validates individual field ranges. Merged-state constraint validation
    /// (outcome, incomplete, override) is done in the write query after
    /// reading the current row.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if !self.has_changes() {
            errors.push("request body contains no fields to update".into());
        }

        if let Some(ref g) = self.final_letter_grade {
            let trimmed = g.trim();
            if trimmed.is_empty() || trimmed.len() > 5 {
                errors.push("final_letter_grade must be 1–5 characters".into());
            }
        }

        if let Some(qp) = self.final_quality_points {
            if qp < 0.0 {
                errors.push("final_quality_points must be >= 0".into());
            }
        }

        if let Some(pct) = self.final_percentage {
            if !(0.0..=999.99).contains(&pct) {
                errors.push("final_percentage must be between 0 and 999.99".into());
            }
        }

        if let Some(ref g) = self.incomplete_default_grade {
            let trimmed = g.trim();
            if trimmed.is_empty() || trimmed.len() > 5 {
                errors.push("incomplete_default_grade must be 1–5 characters".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

#[derive(Debug, Serialize)]
pub struct GradeEntryResponse {
    pub entry_id:                 Uuid,
    pub roster_id:                Uuid,
    pub enrollment_id:            Uuid,
    pub student_id:               Uuid,
    pub final_letter_grade:       Option<String>,
    pub final_quality_points:     Option<f64>,
    pub final_percentage:         Option<f64>,
    pub is_incomplete:            bool,
    pub incomplete_deadline:      Option<NaiveDate>,
    pub incomplete_default_grade: Option<String>,
    pub is_excused:               bool,
    pub is_registrar_override:    bool,
    pub override_reason:          Option<String>,
    pub entered_by_user_id:       Option<Uuid>,
    pub entered_at:               DateTime<Utc>,
    pub updated_at:               DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// POST /lms/grade-roster-submissions/:id/submit
// ═══════════════════════════════════════════════════════════════════════════════

/// Transitions a grade roster from 'open' → 'submitted'.
///
/// No request body — the transition is keyed on the roster ID in the path
/// and the authenticated user from JWT. The write query verifies:
///   1. Roster is 'open'
///   2. Every entry has an outcome (letter grade, incomplete, or excused)
#[derive(Debug, Serialize)]
pub struct GradeRosterSubmitResponse {
    pub roster_id:            Uuid,
    pub section_id:           Uuid,
    pub term_id:              Uuid,
    pub status:               String,
    pub submitted_at:         DateTime<Utc>,
    pub submitted_by_user_id: Uuid,
    pub entry_count:          i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// POST /lms/grade-roster-submissions/:id/post
// ═══════════════════════════════════════════════════════════════════════════════

/// Transitions a grade roster from 'submitted' → 'posted' and writes
/// sis.transcript_records for every graded enrollment.
///
/// Cross-module write sequence (all in one transaction):
///   1. Verify roster is 'submitted'
///   2. For each entry with a final_letter_grade:
///      a. Resolve credits_attempted from sis.courses via section → course
///      b. Compute credits_earned (0 for QP=0 / failing, full for passing)
///      c. Detect prior attempts (same student/course/term) for repeat_instance
///      d. INSERT sis.transcript_records
///      e. UPDATE grade_roster_entries.transcript_record_id
///   3. Entries with is_incomplete = true → deferred (transcript written at resolution)
///   4. Entries with is_excused = true → no transcript record
///   5. Transition roster status → 'posted', stamp posted_at
#[derive(Debug, Serialize)]
pub struct GradeRosterPostResponse {
    pub roster_id:                  Uuid,
    pub section_id:                 Uuid,
    pub term_id:                    Uuid,
    pub status:                     String,
    pub posted_at:                  DateTime<Utc>,
    pub posted_by_user_id:          Uuid,
    pub transcript_records_written: i64,
    pub incomplete_entries:         i64,
    pub excused_entries:            i64,
}

// src/modules/lms/models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Existing list params (unchanged) ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListSectionsParams {
    pub term_id:       Option<Uuid>,
    pub course_id:     Option<Uuid>,
    pub instructor_id: Option<Uuid>,
    pub is_published:  Option<bool>,
    pub page:          Option<i64>,
    pub per_page:      Option<i64>,
}

impl ListSectionsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}

/// A course section as seen from the LMS perspective — bridges sis.course_sections
/// with instructor identity, course info, and term info.
#[derive(Debug, Serialize)]
pub struct SectionSummary {
    pub id:                   Uuid,
    pub section_number:       String,
    pub course_id:            Uuid,
    pub course_subject:       String,
    pub course_number:        String,
    pub course_title:         String,
    pub term_id:              Uuid,
    pub term_name:            String,
    pub term_status:          String,
    pub instructor_id:        Uuid,
    pub instructor_name:      String,
    pub instructional_format: String,
    pub max_enrollment:       i32,
    pub is_published:         bool,
    pub enrolled_count:       i64,
}

#[derive(Debug, Serialize)]
pub struct SectionListResponse {
    pub data:        Vec<SectionSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

// GET /lms/sections/:id ────────────────────────────────────────────────────────

/// Full section detail as seen from the LMS. Extends SectionSummary with
/// additional fields that are expensive to include in list responses:
///   - Meeting times (days of week, start/end time, location)
///   - Waitlisted count (separate from enrolled count)
///   - Assignment and module counts for the section dashboard
///   - Section grading config summary (whether weighted groups are active)
#[derive(Debug, Serialize)]
pub struct SectionDetail {
    // ── Core identity ─────────────────────────────────────────────────────
    pub id:                   Uuid,
    pub section_number:       String,

    // ── Course ────────────────────────────────────────────────────────────
    pub course_id:            Uuid,
    pub course_subject:       String,
    pub course_number:        String,
    pub course_title:         String,
    pub course_credits:       f64,

    // ── Term ──────────────────────────────────────────────────────────────
    pub term_id:              Uuid,
    pub term_name:            String,
    pub term_status:          String,
    pub term_start_date:      chrono::NaiveDate,
    pub term_end_date:        chrono::NaiveDate,

    // ── Instructor ────────────────────────────────────────────────────────
    pub instructor_id:        Uuid,
    pub instructor_name:      String,

    // ── Section metadata ──────────────────────────────────────────────────
    pub instructional_format: String,
    pub instruction_language: String,
    pub max_enrollment:       i32,
    pub waitlist_capacity:    i32,
    pub is_published:         bool,
    pub effective_start_date: chrono::NaiveDate,

    // ── Live counts ───────────────────────────────────────────────────────
    pub enrolled_count:       i64,
    pub waitlisted_count:     i64,
    pub module_count:         i64,
    pub assignment_count:     i64,
}

// GET /sis/courses/:id/sections  (query params for this sub-resource) ──────────

/// Query params for listing sections under a specific course.
/// Mirrors ListSectionsParams but drops `course_id` (it's in the path).
#[derive(Debug, Deserialize)]
pub struct CourseSectionsParams {
    pub term_id:       Option<Uuid>,
    pub instructor_id: Option<Uuid>,
    /// Only published sections? Default: true
    pub is_published:  Option<bool>,
    pub page:          Option<i64>,
    pub per_page:      Option<i64>,
}

impl CourseSectionsParams {
    pub fn page(&self) -> i64     { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64   { (self.page() - 1) * self.per_page() }
}
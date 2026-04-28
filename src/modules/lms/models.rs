// src/modules/lms/models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListSectionsParams {
    /// Filter by term UUID
    pub term_id:       Option<Uuid>,
    /// Filter by course UUID
    pub course_id:     Option<Uuid>,
    /// Filter by instructor UUID
    pub instructor_id: Option<Uuid>,
    /// Only published sections (default: true)
    pub is_published:  Option<bool>,
    pub page:          Option<i64>,
    pub per_page:      Option<i64>,
}

impl ListSectionsParams {
    pub fn page(&self) -> i64 { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64 { (self.page() - 1) * self.per_page() }
}

/// A course section as seen from the LMS perspective — bridges sis.course_sections
/// with instructor identity, course info, and term info.
#[derive(Debug, Serialize)]
pub struct SectionSummary {
    pub id:                  Uuid,
    pub section_number:      String,
    pub course_id:           Uuid,
    pub course_subject:      String,
    pub course_number:       String,
    pub course_title:        String,
    pub term_id:             Uuid,
    pub term_name:           String,
    pub term_status:         String,
    pub instructor_id:       Uuid,
    pub instructor_name:     String,
    pub instructional_format: String,
    pub max_enrollment:      i32,
    pub is_published:        bool,
    pub enrolled_count:      i64,
}

#[derive(Debug, Serialize)]
pub struct SectionListResponse {
    pub data:        Vec<SectionSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}
// src/modules/sis/models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Query parameters ──────────────────────────────────────────────────────────

/// Query parameters for GET /sis/students
///
/// All fields are optional — omitting them returns all students for the
/// authenticated tenant (subject to pagination).
#[derive(Debug, Deserialize)]
pub struct ListStudentsParams {
    /// Filter by academic standing status
    pub standing: Option<String>,
    /// Filter by primary program ID
    pub program_id: Option<Uuid>,
    /// Page number, 1-based (default: 1)
    pub page: Option<i64>,
    /// Page size (default: 25, max: 100)
    pub per_page: Option<i64>,
}

impl ListStudentsParams {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(25).min(100).max(1)
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }
}

// ── Response types ────────────────────────────────────────────────────────────

/// Summary record returned in the student list.
///
/// Deliberately excludes FERPA-restricted fields (date_of_birth, SSN,
/// race/ethnicity) — those belong in a dedicated GET /sis/students/:id
/// endpoint with stricter role-based access control.
#[derive(Debug, Serialize)]
pub struct StudentSummary {
    /// The student's user UUID (same as core.users.id)
    pub user_id:            Uuid,
    pub first_name:         String,
    pub last_name:          String,
    pub preferred_name:     Option<String>,
    pub username:           String,
    pub enrollment_year:    i32,
    pub cumulative_gpa:     Option<f64>,
    pub term_gpa:           Option<f64>,
    pub academic_standing:  Option<String>,
    pub primary_program:    Option<String>,
    pub expected_grad_year: Option<i32>,
}

/// Paginated list response wrapper
#[derive(Debug, Serialize)]
pub struct StudentListResponse {
    pub data:       Vec<StudentSummary>,
    pub page:       i64,
    pub per_page:   i64,
    pub total:      i64,
    pub total_pages: i64,
}
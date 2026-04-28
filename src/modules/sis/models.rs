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

// ── Student detail ────────────────────────────────────────────────────────────

/// Full student record returned by GET /sis/students/:id.
///
/// FERPA NOTICE
/// ─────────────
/// This response includes fields classified as FERPA-restricted PII:
/// date_of_birth, legal_sex, gender_identity, hispanic_or_latino,
/// race_categories, and first_generation_student. These are present in the
/// response now for development; a future role-enforcement middleware pass
/// will gate them behind staff/faculty/admin roles.
///
/// ssn_last_four is intentionally excluded — it has no legitimate display
/// use case and must never appear in an API response.
#[derive(Debug, Serialize)]
pub struct StudentDetail {
    // ── Identity (core.users) ─────────────────────────────────────────────
    pub user_id:          Uuid,
    pub username:         String,
    pub first_name:       String,
    pub middle_name:      Option<String>,
    pub last_name:        String,
    pub preferred_name:   Option<String>,
    pub last_name_suffix: Option<String>,

    // ── Institutional relationship (core.tenant_memberships) ──────────────
    pub system_role:          String,
    pub joined_at:            chrono::DateTime<chrono::Utc>,
    pub institutional_email:  Option<String>,

    // ── Academic profile (sis.student_profiles) ───────────────────────────
    pub enrollment_year:              i32,
    pub expected_graduation_year:     Option<i32>,
    pub cumulative_gpa:               Option<f64>,
    pub term_gpa:                     Option<f64>,
    pub gpa_last_calculated_at:       Option<chrono::DateTime<chrono::Utc>>,
    pub academic_standing:            Option<String>,
    pub cumulative_credits_attempted: f64,
    pub cumulative_credits_earned:    f64,
    pub current_timeframe_pct:        Option<f64>,
    pub is_nsc_opted_out:             bool,

    // ── Demographics (sis.student_demographics, FERPA-restricted) ─────────
    /// None if no demographics record has been created yet.
    pub demographics: Option<StudentDemographics>,

    // ── Programs (sis.student_programs, all current) ──────────────────────
    pub programs: Vec<StudentProgram>,
}

/// FERPA-restricted demographic fields.
/// Excluded: ssn_last_four — never returned in any API response.
#[derive(Debug, Serialize)]
pub struct StudentDemographics {
    pub date_of_birth:          chrono::NaiveDate,
    pub legal_sex:               String,
    pub gender_identity:         Option<String>,
    pub hispanic_or_latino:      Option<bool>,
    pub race_categories:         Option<Vec<String>>,
    pub primary_language:        Option<String>,
    pub requires_iep_or_504:     Option<bool>,
    pub housing_status:          Option<String>,
    pub first_generation_student: Option<bool>,
}

/// One entry in the student's program history (current records only).
#[derive(Debug, Serialize)]
pub struct StudentProgram {
    pub id:           Uuid,
    pub program_id:   Uuid,
    pub program_name: String,
    pub is_primary:   bool,
    pub priority:     i32,
    pub status:       String,
    pub declared_on:  chrono::NaiveDate,
}

// ── Courses ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListCoursesParams {
    pub department_id: Option<uuid::Uuid>,
    pub subject:       Option<String>,
    pub is_active:     Option<bool>,
    pub page:          Option<i64>,
    pub per_page:      Option<i64>,
}

impl ListCoursesParams {
    pub fn page(&self) -> i64 { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> i64 { self.per_page.unwrap_or(25).min(100).max(1) }
    pub fn offset(&self) -> i64 { (self.page() - 1) * self.per_page() }
}

#[derive(Debug, Serialize)]
pub struct CourseSummary {
    pub id:              uuid::Uuid,
    pub subject:         String,
    pub course:          String,
    pub title:           String,
    pub credits:         f64,
    pub is_active:       Option<bool>,
    pub department_name: Option<String>,
    pub grading_basis:   String,
    pub course_level:    Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CourseListResponse {
    pub data:        Vec<CourseSummary>,
    pub page:        i64,
    pub per_page:    i64,
    pub total:       i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize)]
pub struct CourseDetail {
    pub id:                     uuid::Uuid,
    pub subject:                String,
    pub course:                 String,
    pub title:                  String,
    pub description:            Option<String>,
    pub credits:                f64,
    pub is_active:              Option<bool>,
    pub department_id:          uuid::Uuid,
    pub department_name:        Option<String>,
    pub grading_basis:          String,
    pub course_level:           Option<String>,
    pub lecture_hours:          f64,
    pub lab_hours:              f64,
    pub clinical_hours:         f64,
    pub independent_study_hours: f64,
    pub total_contact_hours:    Option<f64>,
    pub is_repeatable:          bool,
    pub max_repeat_attempts:    Option<i16>,
    pub effective_start_date:   chrono::NaiveDate,
    pub catalog_year:           Option<i32>,
}

// ── Enrollments ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct StudentEnrollment {
    pub id:                   uuid::Uuid,
    pub section_id:           uuid::Uuid,
    pub status:               String,
    pub enrolled_at:          chrono::DateTime<chrono::Utc>,
    pub credit_hours_enrolled: Option<f64>,
    pub course_subject:       String,
    pub course_number:        String,
    pub course_title:         String,
    pub term_name:            String,
    pub term_id:              uuid::Uuid,
    pub instructor_name:      Option<String>,
    pub section_number:       String,
}
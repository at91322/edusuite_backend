// src/modules/sis/write_models.rs
//
// Request/response models for SIS write operations.
// Existing CreateStudentRequest and UpdateStudentRequest are preserved below;
// new types for enrollment writes are appended.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── MaybePatch helper (unchanged) ─────────────────────────────────────────────

/// Three-state wrapper used in PATCH bodies:
///   - Absent   → field not in JSON; leave DB value alone
///   - Null     → field present as `null`; clear the column
///   - Value(v) → field present with a value; update the column
#[derive(Debug, Deserialize, Default)]
#[serde(untagged)]
pub enum MaybePatch<T> {
    #[default]
    Absent,
    Null,
    Value(T),
}

impl<T> MaybePatch<T> {
    pub fn is_absent(&self) -> bool { matches!(self, MaybePatch::Absent) }
}

// ── POST /sis/students (unchanged) ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateStudentRequest {
    pub username:                   String,
    pub password:                   String,
    pub first_name:                 String,
    pub middle_name:                Option<String>,
    pub last_name:                  String,
    pub preferred_name:             Option<String>,
    pub last_name_suffix:           Option<String>,
    pub institutional_email:        Option<String>,
    pub enrollment_year:            i32,
    pub expected_graduation_year:   Option<i32>,
}

impl CreateStudentRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.username.trim().is_empty()   { errors.push("username is required".into()); }
        if self.password.len() < 8           { errors.push("password must be at least 8 characters".into()); }
        if self.first_name.trim().is_empty() { errors.push("first_name is required".into()); }
        if self.last_name.trim().is_empty()  { errors.push("last_name is required".into()); }
        if self.enrollment_year < 1900 || self.enrollment_year > 2100 {
            errors.push("enrollment_year is out of range".into());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ── PATCH /sis/students/:id (unchanged) ──────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct UpdateStudentRequest {
    #[serde(default)] pub first_name:               MaybePatch<String>,
    #[serde(default)] pub middle_name:              MaybePatch<String>,
    #[serde(default)] pub last_name:                MaybePatch<String>,
    #[serde(default)] pub preferred_name:           MaybePatch<String>,
    #[serde(default)] pub last_name_suffix:         MaybePatch<String>,
    #[serde(default)] pub institutional_email:      MaybePatch<String>,
    #[serde(default)] pub expected_graduation_year: MaybePatch<i32>,
    #[serde(default)] pub academic_standing_status: MaybePatch<String>,
}

impl UpdateStudentRequest {
    pub fn has_changes(&self) -> bool {
        !self.first_name.is_absent()               ||
        !self.middle_name.is_absent()              ||
        !self.last_name.is_absent()                ||
        !self.preferred_name.is_absent()           ||
        !self.last_name_suffix.is_absent()         ||
        !self.institutional_email.is_absent()      ||
        !self.expected_graduation_year.is_absent() ||
        !self.academic_standing_status.is_absent()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if let MaybePatch::Value(ref v) = self.first_name {
            if v.trim().is_empty() { errors.push("first_name cannot be blank".into()); }
        }
        if let MaybePatch::Value(ref v) = self.last_name {
            if v.trim().is_empty() { errors.push("last_name cannot be blank".into()); }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}


// POST /sis/students/:id/enrollments ───────────────────────────────────────────

/// Request body for enrolling a student into a section.
///
/// The section must belong to this tenant (enforced by RLS).
/// Capacity and duplicate checks are performed in write_queries before INSERT.
///
/// `status` defaults to `"enrolled"`. Registrars may pass `"waitlisted"` to
/// explicitly waitlist when the section is full rather than let the server
/// auto-promote. Any other status values (dropped, withdrawn, etc.) are
/// rejected — those go through separate lifecycle endpoints.
#[derive(Debug, Deserialize)]
pub struct CreateEnrollmentRequest {
    pub section_id:                    Uuid,
    /// "enrolled" | "waitlisted"  (default: "enrolled")
    pub status:                        Option<String>,
    pub enrollment_effective_date:     Option<NaiveDate>,
    pub credit_hours_enrolled:         Option<f64>,
    pub is_title_iv_eligible:          Option<bool>,
    /// Optional: link this enrollment to a fulfilled catalog requirement
    pub fulfilled_catalog_requirement_id: Option<Uuid>,
}

impl CreateEnrollmentRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(ref s) = self.status {
            match s.as_str() {
                "enrolled" | "waitlisted" => {}
                other => errors.push(format!(
                    "status '{}' is not valid for a new enrollment; use 'enrolled' or 'waitlisted'",
                    other
                )),
            }
        }

        if let Some(ch) = self.credit_hours_enrolled {
            if ch < 0.0 || ch > 99.0 {
                errors.push("credit_hours_enrolled must be between 0 and 99".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Response body returned as 201 Created.
#[derive(Debug, Serialize)]
pub struct EnrollmentResponse {
    pub enrollment_id:             Uuid,
    pub student_id:                Uuid,
    pub section_id:                Uuid,
    pub status:                    String,
    pub enrolled_at:               chrono::DateTime<chrono::Utc>,
    pub enrollment_effective_date: Option<NaiveDate>,
    pub credit_hours_enrolled:     Option<f64>,
    pub is_title_iv_eligible:      bool,
}

// POST /sis/enrollments  (cross-module: SIS + LMS) ─────────────────────────────

/// Full cross-module enrollment request.
///
/// Identical semantics to CreateEnrollmentRequest but designed for the
/// standalone `/sis/enrollments` endpoint which touches both SIS and LMS
/// in a single atomic transaction:
///
///   1. Capacity / duplicate guard
///   2. INSERT sis.enrollments
///   3. INSERT lms.section_grade_summaries  ← LMS side effect
///   4. Audit log event published
///
/// The student_id is required here because the URL is not scoped to a
/// student (unlike /sis/students/:id/enrollments).
#[derive(Debug, Deserialize)]
pub struct CrossModuleEnrollRequest {
    pub student_id:                    Uuid,
    pub section_id:                    Uuid,
    /// "enrolled" | "waitlisted"  (default: "enrolled")
    pub status:                        Option<String>,
    pub enrollment_effective_date:     Option<NaiveDate>,
    pub credit_hours_enrolled:         Option<f64>,
    pub is_title_iv_eligible:          Option<bool>,
    pub fulfilled_catalog_requirement_id: Option<Uuid>,
}

impl CrossModuleEnrollRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(ref s) = self.status {
            match s.as_str() {
                "enrolled" | "waitlisted" => {}
                other => errors.push(format!(
                    "status '{}' is not valid; use 'enrolled' or 'waitlisted'",
                    other
                )),
            }
        }

        if let Some(ch) = self.credit_hours_enrolled {
            if ch < 0.0 || ch > 99.0 {
                errors.push("credit_hours_enrolled must be between 0 and 99".into());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Richer 201 response for the cross-module path — includes LMS side-effect
/// confirmation so callers can see both records were created.
#[derive(Debug, Serialize)]
pub struct CrossModuleEnrollResponse {
    pub enrollment_id:             Uuid,
    pub student_id:                Uuid,
    pub section_id:                Uuid,
    pub status:                    String,
    pub enrolled_at:               chrono::DateTime<chrono::Utc>,
    pub enrollment_effective_date: Option<NaiveDate>,
    pub credit_hours_enrolled:     Option<f64>,
    pub is_title_iv_eligible:      bool,
    /// Confirms the LMS grade summary row was provisioned.
    pub lms_grade_summary_id:      Uuid,
}
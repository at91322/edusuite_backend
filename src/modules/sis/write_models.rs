// src/modules/sis/write_models.rs

use serde::Deserialize;

// ── Patch helper ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum MaybePatch<T> {
    Absent,
    Present(T),
}

impl<T> Default for MaybePatch<T> {
    fn default() -> Self { MaybePatch::Absent }
}

pub fn deserialize_optional_field<'de, T, D>(
    deserializer: D,
) -> Result<MaybePatch<T>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    T::deserialize(deserializer).map(MaybePatch::Present)
}

// ── POST /sis/students ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct CreateStudentRequest {
    pub username:                 String,
    pub password:                 String,
    pub first_name:               String,
    #[serde(default)]
    pub middle_name:              Option<String>,
    pub last_name:                String,
    #[serde(default)]
    pub preferred_name:           Option<String>,
    #[serde(default)]
    pub last_name_suffix:         Option<String>,
    #[serde(default)]
    pub institutional_email:      Option<String>,
    pub enrollment_year:          i32,
    #[serde(default)]
    pub expected_graduation_year: Option<i32>,
}

impl CreateStudentRequest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let username = self.username.trim();
        if username.len() < 3 {
            errors.push("username must be at least 3 characters".into());
        } else if !username.chars().all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-')) {
            errors.push("username may only contain letters, numbers, '.', '_', '-'".into());
        }
        if self.password.len() < 8 {
            errors.push("password must be at least 8 characters".into());
        }
        if self.first_name.trim().is_empty() {
            errors.push("first_name is required".into());
        }
        if self.last_name.trim().is_empty() {
            errors.push("last_name is required".into());
        }
        if self.enrollment_year < 1900 || self.enrollment_year > 2100 {
            errors.push("enrollment_year must be a valid year".into());
        }
        if let Some(g) = self.expected_graduation_year {
            if g < self.enrollment_year {
                errors.push("expected_graduation_year must be >= enrollment_year".into());
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// ── PATCH /sis/students/:id ───────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct UpdateStudentRequest {
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub first_name:       MaybePatch<String>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub middle_name:      MaybePatch<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub last_name:        MaybePatch<String>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub preferred_name:   MaybePatch<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub last_name_suffix: MaybePatch<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub institutional_email: MaybePatch<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub enrollment_year:          MaybePatch<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub expected_graduation_year: MaybePatch<Option<i32>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub academic_standing_status: MaybePatch<Option<String>>,
}

impl UpdateStudentRequest {
    pub fn has_changes(&self) -> bool {
        !matches!(self.first_name,               MaybePatch::Absent) ||
        !matches!(self.middle_name,              MaybePatch::Absent) ||
        !matches!(self.last_name,                MaybePatch::Absent) ||
        !matches!(self.preferred_name,           MaybePatch::Absent) ||
        !matches!(self.last_name_suffix,         MaybePatch::Absent) ||
        !matches!(self.institutional_email,      MaybePatch::Absent) ||
        !matches!(self.enrollment_year,          MaybePatch::Absent) ||
        !matches!(self.expected_graduation_year, MaybePatch::Absent) ||
        !matches!(self.academic_standing_status, MaybePatch::Absent)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if let MaybePatch::Present(ref v) = self.first_name {
            if v.trim().is_empty() { errors.push("first_name cannot be blank".into()); }
        }
        if let MaybePatch::Present(ref v) = self.last_name {
            if v.trim().is_empty() { errors.push("last_name cannot be blank".into()); }
        }
        if let MaybePatch::Present(ref y) = self.enrollment_year {
            if *y < 1900 || *y > 2100 {
                errors.push("enrollment_year must be a valid year".into());
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
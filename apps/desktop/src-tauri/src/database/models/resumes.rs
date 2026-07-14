use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeProfile {
    pub id: String,
    pub name: String,
    pub file_path: Option<String>,
    pub file_format: Option<String>,
    pub parsed_text: String,
    pub personal_info: String,
    pub education_background: String,
    pub internship_experience: String,
    pub project_experience: String,
    pub professional_skills: String,
    pub academic_achievements: String,
    pub skill_certificates: String,
    pub target_direction: String,
    pub notes: String,
    pub parent_profile_id: Option<String>,
    pub linked_application_count: i64,
    pub assessment_count: i64,
    pub interview_count: i64,
    pub offer_count: i64,
    pub is_primary: bool,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResumeProfileInput {
    pub name: String,
    pub file_path: Option<String>,
    pub file_format: Option<String>,
    pub parsed_text: Option<String>,
    pub personal_info: Option<String>,
    pub education_background: Option<String>,
    pub internship_experience: Option<String>,
    pub project_experience: Option<String>,
    pub professional_skills: Option<String>,
    pub academic_achievements: Option<String>,
    pub skill_certificates: Option<String>,
    pub target_direction: Option<String>,
    pub notes: Option<String>,
    pub parent_profile_id: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResumeProfileInput {
    pub name: String,
    pub personal_info: String,
    pub education_background: String,
    pub internship_experience: String,
    pub project_experience: String,
    pub professional_skills: String,
    pub academic_achievements: String,
    pub skill_certificates: String,
    pub target_direction: String,
    pub notes: String,
}

use crate::{
    db::{CreateResumeProfileInput, Database, ResumeProfile, UpdateResumeProfileInput},
    resume, resume_ai,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportResumeProfileInput {
    path: String,
    #[serde(default)]
    confirm_ai_send: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResumeImportResult {
    profile: ResumeProfile,
    ai_status: String,
    warning: Option<String>,
}

#[tauri::command]
pub(crate) fn list_resume_profiles(
    database: tauri::State<'_, Database>,
) -> Result<Vec<ResumeProfile>, String> {
    database.list_resume_profiles()
}

#[tauri::command]
pub(crate) async fn import_resume_profile(
    database: tauri::State<'_, Database>,
    input: ImportResumeProfileInput,
) -> Result<ResumeImportResult, String> {
    let profile = resume::import_resume(&database, &input.path)?;
    let ai_settings = database.get_provider_settings()?.ai;
    if !ai_settings.allow_resume {
        return Ok(ResumeImportResult {
            profile,
            ai_status: "skipped".into(),
            warning: Some("未授权向 AI 服务发送简历，当前仅使用本地规则解析".into()),
        });
    }
    if ai_settings.prompt_before_send && !input.confirm_ai_send {
        return Ok(ResumeImportResult {
            profile,
            ai_status: "skipped".into(),
            warning: Some("尚未确认向 AI 服务发送简历，当前仅使用本地规则解析".into()),
        });
    }
    let value = match resume_ai::structure_resume(&database, &profile.parsed_text).await {
        Ok(value) => value,
        Err(error) => {
            return Ok(ResumeImportResult {
                profile,
                ai_status: "failed".into(),
                warning: Some(format!("AI 结构化解析失败，已保留本地解析草稿：{error}")),
            })
        }
    };
    let serialize = |key: &str| {
        value
            .get(key)
            .map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| serde_json::to_string(item).unwrap_or_default())
            })
            .unwrap_or_default()
    };
    let profile = database.update_resume_profile(
        &profile.id,
        UpdateResumeProfileInput {
            name: profile.name.clone(),
            personal_info: serialize("personal"),
            education_background: serialize("education"),
            internship_experience: serialize("internships"),
            project_experience: serialize("projects"),
            professional_skills: serialize("skills"),
            academic_achievements: serialize("academics"),
            skill_certificates: serialize("certificates"),
            target_direction: profile.target_direction.clone(),
            notes: profile.notes.clone(),
        },
    )?;
    Ok(ResumeImportResult {
        profile,
        ai_status: "succeeded".into(),
        warning: None,
    })
}

#[tauri::command]
pub(crate) fn update_resume_profile(
    database: tauri::State<'_, Database>,
    id: String,
    input: UpdateResumeProfileInput,
) -> Result<ResumeProfile, String> {
    database.update_resume_profile(&id, input)
}

#[tauri::command]
pub(crate) fn set_primary_resume_profile(
    database: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    database.set_primary_resume_profile(&id)
}

#[tauri::command]
pub(crate) fn delete_resume_profile(
    database: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    database.delete_resume_profile(&id)
}

#[tauri::command]
pub(crate) fn duplicate_resume_profile(
    database: tauri::State<'_, Database>,
    id: String,
) -> Result<ResumeProfile, String> {
    database.duplicate_resume_profile(&id)
}

#[tauri::command]
pub(crate) fn set_resume_profile_archived(
    database: tauri::State<'_, Database>,
    id: String,
    archived: bool,
) -> Result<(), String> {
    database.set_resume_profile_archived(&id, archived)
}

#[tauri::command]
pub(crate) fn create_blank_resume_profile(
    database: tauri::State<'_, Database>,
    name: String,
) -> Result<ResumeProfile, String> {
    database.create_resume_profile(CreateResumeProfileInput {
        name,
        file_path: None,
        file_format: None,
        parsed_text: None,
        personal_info: Some("{}".into()),
        education_background: Some("[]".into()),
        internship_experience: Some("[]".into()),
        project_experience: Some("[]".into()),
        professional_skills: None,
        academic_achievements: Some("[]".into()),
        skill_certificates: Some("[]".into()),
        target_direction: None,
        notes: None,
        parent_profile_id: None,
        is_primary: false,
    })
}

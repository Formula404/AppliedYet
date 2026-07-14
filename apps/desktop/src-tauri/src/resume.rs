use crate::{
    db::{CreateResumeProfileInput, Database, ResumeProfile},
    document,
};
use std::path::Path;

pub fn import_resume(database: &Database, path: &str) -> Result<ResumeProfile, String> {
    let path_ref = Path::new(path);
    let (format, text) = document::extract_document(path_ref)?;
    if text.trim().is_empty() {
        return Err("简历中没有提取到文字；如果是扫描版 PDF，请先进行 OCR 后再导入".into());
    }
    let name = path_ref
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("未命名简历")
        .to_string();
    let sections = split_sections(&text);
    database.create_resume_profile(CreateResumeProfileInput {
        name,
        file_path: Some(path.to_string()),
        file_format: Some(format),
        parsed_text: Some(text),
        personal_info: Some(sections.personal_info),
        education_background: Some(sections.education_background),
        internship_experience: Some(sections.internship_experience),
        project_experience: Some(sections.project_experience),
        professional_skills: Some(sections.professional_skills),
        academic_achievements: Some(sections.academic_achievements),
        skill_certificates: Some(sections.skill_certificates),
        target_direction: None,
        notes: None,
        parent_profile_id: None,
        is_primary: true,
    })
}

struct ResumeSections {
    personal_info: String,
    education_background: String,
    internship_experience: String,
    project_experience: String,
    professional_skills: String,
    academic_achievements: String,
    skill_certificates: String,
}

fn split_sections(text: &str) -> ResumeSections {
    let mut current = "personal_info";
    let mut sections = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for line in text.lines() {
        let normalized = line.trim().to_ascii_lowercase();
        let next = if contains_any(&normalized, &["教育背景", "教育经历", "education"]) {
            Some("education_background")
        } else if contains_any(
            &normalized,
            &["实习经历", "工作经历", "实习", "experience", "internship"],
        ) {
            Some("internship_experience")
        } else if contains_any(
            &normalized,
            &["项目经历", "项目经验", "projects", "project experience"],
        ) {
            Some("project_experience")
        } else if contains_any(
            &normalized,
            &["专业技能", "技能", "skills", "technical skills"],
        ) {
            Some("professional_skills")
        } else if contains_any(
            &normalized,
            &["学术成果", "科研成果", "论文", "publication", "research"],
        ) {
            Some("academic_achievements")
        } else if contains_any(
            &normalized,
            &["技能证书", "证书", "certification", "certificates"],
        ) {
            Some("skill_certificates")
        } else {
            None
        };
        if let Some(section) = next {
            current = section;
            continue;
        }
        sections.entry(current).or_default().push(line);
    }
    let get = |key: &str| {
        sections
            .get(key)
            .map(|lines| lines.join("\n"))
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    ResumeSections {
        personal_info: get("personal_info"),
        education_background: get("education_background"),
        internship_experience: get("internship_experience"),
        project_experience: get("project_experience"),
        professional_skills: get("professional_skills"),
        academic_achievements: get("academic_achievements"),
        skill_certificates: get("skill_certificates"),
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

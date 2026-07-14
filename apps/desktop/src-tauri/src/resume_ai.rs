use crate::{
    ai::{candidate_models, request_text},
    credentials,
    db::Database,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StructuredResume {
    personal: Personal,
    education: Vec<Education>,
    internships: Vec<Internship>,
    projects: Vec<Project>,
    skills: String,
    academics: Vec<Academic>,
    certificates: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Personal {
    name: String,
    birthday: String,
    contact: String,
    links: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Education {
    start_date: String,
    end_date: String,
    school: String,
    degree: String,
    major: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Internship {
    company: String,
    role: String,
    start_date: String,
    end_date: String,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    name: String,
    role: String,
    start_date: String,
    end_date: String,
    description: String,
    technologies: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Academic {
    title: String,
    kind: String,
    date: String,
    description: String,
    link: String,
}

pub async fn structure_resume(database: &Database, text: &str) -> Result<Value, String> {
    let settings = database.get_provider_settings()?.ai;
    if !settings.allow_resume {
        return Err("未允许向 AI 服务发送简历内容".into());
    }
    let api_key = credentials::get_secret("ai_api_key")?;
    if text.trim().is_empty() {
        return Err("简历中没有可解析文本".into());
    }

    let prompt = format!(
        "将 <resume> 中的简历纯文本准确整理为结构化数据。\n\
         - 每家公司或组织必须生成一条独立 internships 记录，不得把多家公司合并。\n\
         - 每个项目必须生成一条独立 projects 记录，不得塞入实习 description。\n\
         - 教育、技能、证书、个人信息只能放入对应字段。\n\
         - description 保留职责与成果的语义，可用换行分隔要点。\n\
         - 只提取原文明确出现的信息；缺失字段输出空字符串，不推测、不补写。\n\
         - 日期统一为 YYYY-MM；“至今”的 endDate 输出空字符串。\n\n\
         <resume>\n{}\n</resume>",
        text
    );
    let mut last_error = String::new();
    for model in candidate_models(&settings) {
        for attempt in 0..2 {
            let schema_fallback = attempt > 0 && schema_transport_is_unsupported(&last_error);
            let repair = if attempt == 0 || last_error.is_empty() {
                String::new()
            } else if schema_fallback {
                format!(
                    "\n当前 AI 服务不支持 JSON Schema 请求参数，请直接按以下 Schema 输出纯 JSON：\n{}",
                    resume_schema()
                )
            } else {
                format!(
                    "\n上一次结果未通过校验：{}。请根据该错误重新解析全文，并严格按 Schema 输出。",
                    last_error
                )
            };
            match request_text(
                &settings,
                &api_key,
                &model,
                "你是简历结构化解析器。不得把教育、项目、技能、证书或个人信息放进实习经历。只输出 JSON。",
                &format!("{prompt}{repair}"),
                (!schema_fallback).then(resume_schema),
            ).await {
                Ok(output) => match serde_json::from_str::<StructuredResume>(strip_json_fence(&output)) {
                    Ok(value) => match validate_resume(&value, text) {
                        Ok(()) => return serde_json::to_value(value).map_err(|error| error.to_string()),
                        Err(error) => last_error = error,
                    },
                    Err(error) => last_error = format!("简历结构校验失败: {error}"),
                },
                Err(error) => last_error = error,
            }
        }
    }
    Err(if last_error.is_empty() {
        "AI 未返回有效简历结构".into()
    } else {
        last_error
    })
}

fn schema_transport_is_unsupported(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("response_format")
        || error.contains("json_schema")
        || error.contains("structured output")
        || error.contains("结构化输出不支持")
}

fn strip_json_fence(value: &str) -> &str {
    let value = value.trim();
    let value = value
        .strip_prefix("```json")
        .or_else(|| value.strip_prefix("```"))
        .unwrap_or(value);
    value.strip_suffix("```").unwrap_or(value).trim()
}

fn validate_resume(resume: &StructuredResume, source: &str) -> Result<(), String> {
    let has_content = !resume.personal.name.trim().is_empty()
        || !resume.personal.contact.trim().is_empty()
        || !resume.education.is_empty()
        || !resume.internships.is_empty()
        || !resume.projects.is_empty()
        || !resume.skills.trim().is_empty()
        || !resume.academics.is_empty()
        || !resume.certificates.is_empty();
    if !has_content {
        return Err("AI 返回了空简历".into());
    }
    if resume
        .education
        .iter()
        .any(|item| item.school.trim().is_empty())
    {
        return Err("教育经历中存在缺少院校的空条目".into());
    }
    if resume
        .internships
        .iter()
        .any(|item| item.company.trim().is_empty() && item.role.trim().is_empty())
    {
        return Err("工作或实习经历中存在无法识别公司和职位的条目".into());
    }
    if resume
        .projects
        .iter()
        .any(|item| item.name.trim().is_empty())
    {
        return Err("项目经历中存在缺少项目名称的条目".into());
    }
    if resume
        .academics
        .iter()
        .any(|item| item.title.trim().is_empty())
    {
        return Err("学术成果中存在缺少名称的条目".into());
    }

    let section_markers = [
        "教育背景",
        "教育经历",
        "实习经历",
        "工作经历",
        "项目经历",
        "项目经验",
        "专业技能",
        "技能证书",
        "学术成果",
        "education",
        "experience",
        "projects",
        "skills",
    ];
    let source_lower = source.to_ascii_lowercase();
    let marker_count = section_markers
        .iter()
        .filter(|marker| source_lower.contains(*marker))
        .count();
    let source_len = source.chars().count().max(1);
    let looks_like_everything_in_one_field = resume
        .internships
        .iter()
        .map(|item| item.description.chars().count())
        .chain(
            resume
                .projects
                .iter()
                .map(|item| item.description.chars().count()),
        )
        .any(|length| marker_count >= 3 && length > 500 && length * 2 > source_len);
    if looks_like_everything_in_one_field {
        return Err("检测到大部分简历内容被合并进单个经历，请按栏目和经历边界拆分".into());
    }
    Ok(())
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}

fn resume_schema() -> Value {
    let string = || json!({"type":"string"});
    json!({
        "type":"object", "additionalProperties":false,
        "properties":{
            "personal":object(json!({"name":string(),"birthday":string(),"contact":string(),"links":string()}), &["name","birthday","contact","links"]),
            "education":{"type":"array","items":object(json!({"startDate":string(),"endDate":string(),"school":string(),"degree":string(),"major":string()}), &["startDate","endDate","school","degree","major"])},
            "internships":{"type":"array","items":object(json!({"company":string(),"role":string(),"startDate":string(),"endDate":string(),"description":string()}), &["company","role","startDate","endDate","description"])},
            "projects":{"type":"array","items":object(json!({"name":string(),"role":string(),"startDate":string(),"endDate":string(),"description":string(),"technologies":string()}), &["name","role","startDate","endDate","description","technologies"])},
            "skills":string(),
            "academics":{"type":"array","items":object(json!({"title":string(),"kind":string(),"date":string(),"description":string(),"link":string()}), &["title","kind","date","description","link"])},
            "certificates":{"type":"array","items":string()}
        },
        "required":["personal","education","internships","projects","skills","academics","certificates"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_schema_defines_every_array_item() {
        let schema = resume_schema();
        assert!(schema["properties"]["projects"]["items"]["properties"].is_object());
        assert!(schema["properties"]["academics"]["items"]["properties"].is_object());
    }

    #[test]
    fn rejects_a_resume_collapsed_into_one_description() {
        let resume = StructuredResume {
            personal: Personal {
                name: "张三".into(),
                birthday: String::new(),
                contact: String::new(),
                links: String::new(),
            },
            education: vec![],
            internships: vec![Internship {
                company: "某公司".into(),
                role: "开发".into(),
                start_date: String::new(),
                end_date: String::new(),
                description: "内容".repeat(300),
            }],
            projects: vec![],
            skills: String::new(),
            academics: vec![],
            certificates: vec![],
        };
        let source = format!(
            "教育背景\n实习经历\n项目经历\n专业技能\n{}",
            "内容".repeat(400)
        );
        assert!(validate_resume(&resume, &source).is_err());
    }
}

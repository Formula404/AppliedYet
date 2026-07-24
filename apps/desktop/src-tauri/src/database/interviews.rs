use super::{db_error, Database};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInterviewQuestion {
    pub prompt: String,
    pub source: String,
    #[serde(default)]
    pub answer: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterviewQuestionRecord {
    pub id: String,
    pub prompt: String,
    pub source: String,
    pub answer: String,
    pub score: Option<i64>,
    pub evaluation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterviewSessionRecord {
    pub id: String,
    pub application_id: String,
    pub session_type: String,
    pub round: String,
    pub created_at: String,
    pub duration: String,
    pub status: String,
    pub current_question_index: i64,
    pub review_summary: Option<String>,
    pub questions: Vec<InterviewQuestionRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterviewQuestionReview {
    pub question_id: String,
    pub score: i64,
    pub evaluation: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuestionBankInput {
    #[serde(default)]
    pub query: String,
    #[serde(default = "default_bank_status")]
    pub status: String,
    #[serde(default)]
    pub review_state: Option<String>,
    #[serde(default)]
    pub mastery: Vec<String>,
    #[serde(default = "default_bank_sort")]
    pub sort: String,
    #[serde(default = "default_bank_direction")]
    pub direction: String,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionBankItem {
    pub id: String,
    pub prompt: String,
    pub category: String,
    pub best_answer: String,
    pub mastery: String,
    pub system_mastery: String,
    pub manual_mastery: Option<String>,
    pub membership_status: String,
    pub real_interview_count: i64,
    pub asked_count: i64,
    pub practice_count: i64,
    pub reference_count: i64,
    pub company_count: i64,
    pub legacy_count: i64,
    pub last_real_asked_at: Option<String>,
    pub last_practiced_at: Option<String>,
    pub next_review_at: Option<String>,
    pub created_at: String,
    pub sources: Vec<String>,
    pub needs_review: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionBankFacets {
    pub active: i64,
    pub due: i64,
    pub pending_matches: i64,
    pub archived: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionBankPage {
    pub items: Vec<QuestionBankItem>,
    pub total: i64,
    pub next_cursor: Option<String>,
    pub facets: QuestionBankFacets,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionEvidence {
    pub id: String,
    pub event_type: String,
    pub source_type: String,
    pub source_id: String,
    pub source_item_id: String,
    pub prompt: String,
    pub company: Option<String>,
    pub position: Option<String>,
    pub round: Option<String>,
    pub occurred_at: String,
    pub verification_state: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionBankDetail {
    #[serde(flatten)]
    pub item: QuestionBankItem,
    pub variants: Vec<String>,
    pub evidence: Vec<QuestionEvidence>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionMatchCandidate {
    pub question: QuestionBankItem,
    pub score: f64,
    pub reason: String,
}

fn default_bank_status() -> String { "active".into() }
fn default_bank_sort() -> String { "review_priority".into() }
fn default_bank_direction() -> String { "desc".into() }
fn default_page_size() -> i64 { 30 }

impl Database {
    pub fn list_interview_sessions(&self) -> Result<Vec<InterviewSessionRecord>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection
            .prepare("SELECT id FROM interview_sessions ORDER BY created_at DESC, rowid DESC")
            .map_err(db_error)?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(db_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        ids.iter()
            .map(|id| load_interview_session(&connection, id))
            .collect()
    }

    pub fn get_interview_session(&self, id: &str) -> Result<InterviewSessionRecord, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        load_interview_session(&connection, id)
    }

    pub fn create_interview_session(
        &self,
        application_id: &str,
        session_type: &str,
        round: &str,
        status: &str,
        questions: &[CreateInterviewQuestion],
    ) -> Result<InterviewSessionRecord, String> {
        if questions.is_empty() || questions.len() > crate::MAX_INTERVIEW_QUESTIONS {
            return Err("面试会话的问题数量必须在 1 到 30 之间".into());
        }
        if !matches!(session_type, "模拟面试" | "真实面试")
            || !matches!(status, "进行中" | "待复盘" | "复盘完成")
            || round.trim().is_empty()
        {
            return Err("面试会话类型、轮次或状态无效".into());
        }
        if questions.iter().any(|question| {
            question.prompt.trim().is_empty()
                || question.prompt.chars().count() > 10_000
                || question.answer.chars().count() > 100_000
                || !matches!(
                    question.source.as_str(),
                    "面经" | "AI 简历题" | "真实面试" | "个人题库"
                )
        }) {
            return Err("面试问题的内容、来源或长度无效".into());
        }
        let id = Uuid::new_v4().to_string();
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let application_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM applications
                    WHERE id=?1 AND deleted_at IS NULL
                      AND (
                        ?2='真实面试'
                        OR (
                          archived_at IS NULL
                          AND current_stage NOT LIKE '%拒绝%'
                          AND lower(current_stage) NOT LIKE '%offer%'
                          AND current_stage NOT LIKE '%人才库%'
                          AND current_stage NOT IN ('流程暂停','流程结束','主动放弃')
                        )
                      )
                 )",
                params![application_id, session_type],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        if !application_exists {
            return Err(if session_type == "真实面试" {
                "投递记录不存在或已经删除"
            } else {
                "只能为使用中的投递创建模拟面试"
            }
            .into());
        }
        transaction
            .execute(
                "INSERT INTO interview_sessions(id,application_id,session_type,round,status,completed_at)
                 VALUES (?1,?2,?3,?4,?5,CASE WHEN ?5='进行中' THEN NULL ELSE strftime('%Y-%m-%dT%H:%M:%fZ','now') END)",
                params![id, application_id, session_type, round, status],
            )
            .map_err(db_error)?;
        for (position, question) in questions.iter().enumerate() {
            let prompt = question.prompt.trim();
            transaction
                .execute(
                    "INSERT INTO interview_session_questions(id,session_id,position,prompt,source,answer)
                     VALUES (?1,?2,?3,?4,?5,?6)",
                    params![
                        Uuid::new_v4().to_string(),
                        id,
                        position as i64,
                        prompt,
                        question.source,
                        question.answer.trim()
                    ],
                )
                .map_err(db_error)?;
            upsert_question_bank(&transaction, prompt, &question.source)?;
        }
        transaction.commit().map_err(db_error)?;
        load_interview_session(&connection, &id)
    }

    pub fn update_interview_session_answer(
        &self,
        session_id: &str,
        question_id: &str,
        answer: &str,
    ) -> Result<(), String> {
        if answer.chars().count() > 100_000 {
            return Err("单题回答不能超过 10 万字".into());
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute(
                "UPDATE interview_session_questions SET answer=?3,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id=?2 AND session_id=?1
                   AND EXISTS(SELECT 1 FROM interview_sessions WHERE id=?1 AND status='进行中')",
                params![session_id, question_id, answer],
            )
            .map_err(db_error)?;
        if changed == 0 {
            let in_progress: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM interview_sessions WHERE id=?1 AND status='进行中')",
                    [session_id],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            if in_progress {
                return Err("面试问题不存在".into());
            }
            return Err("面试会话已结束，无法再更新答案".into());
        }
        connection
            .execute(
                "UPDATE interview_sessions SET updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
                [session_id],
            )
            .map_err(db_error)?;
        Ok(())
    }

    pub fn update_interview_session_progress(
        &self,
        id: &str,
        question_index: i64,
    ) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let question_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM interview_session_questions WHERE session_id=?1",
                [id],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        if question_index < 0 || question_index >= question_count {
            return Err("面试问题进度超出范围".into());
        }
        let changed = connection
            .execute(
                "UPDATE interview_sessions SET current_question_index=MAX(0,?2),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id=?1 AND status='进行中'",
                params![id, question_index],
            )
            .map_err(db_error)?;
        if changed == 0 {
            return Err("可继续作答的面试会话不存在".into());
        }
        Ok(())
    }

    pub fn complete_interview_session(&self, id: &str) -> Result<InterviewSessionRecord, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute(
                "UPDATE interview_sessions
                 SET status='待复盘',completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),
                     duration_seconds=MAX(1,CAST((julianday('now')-julianday(created_at))*86400 AS INTEGER)),
                     updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id=?1 AND status='进行中'",
                [id],
            )
            .map_err(db_error)?;
        if changed == 0 {
            return Err("进行中的面试会话不存在".into());
        }
        load_interview_session(&connection, id)
    }

    pub fn save_interview_session_review(
        &self,
        id: &str,
        summary: &str,
        reviews: &[InterviewQuestionReview],
    ) -> Result<InterviewSessionRecord, String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        for review in reviews {
            if !(0..=100).contains(&review.score) || review.evaluation.trim().is_empty() {
                return Err("复盘评分或评价内容无效".into());
            }
            let changed = transaction
                .execute(
                    "UPDATE interview_session_questions SET score=?3,evaluation=?4,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE session_id=?1 AND id=?2",
                    params![id, review.question_id, review.score, review.evaluation.trim()],
                )
                .map_err(db_error)?;
            if changed == 0 {
                return Err("复盘结果包含未知问题".into());
            }
            let question: (String, String) = transaction
                .query_row(
                    "SELECT prompt,answer FROM interview_session_questions WHERE id=?1",
                    [&review.question_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(db_error)?;
            let mastery = mastery_from_score(review.score);
            transaction
                .execute(
                    "UPDATE question_bank_items
                     SET mastery=?2,best_answer=CASE WHEN best_answer='' AND ?3>=80 AND ?4<>'' THEN ?4 ELSE best_answer END,
                         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE normalized_key=?1",
                    params![normalize_question(&question.0), mastery, review.score, question.1],
                )
                .map_err(db_error)?;
        }
        transaction
            .execute(
                "UPDATE interview_sessions SET status='复盘完成',review_summary=?2,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
                params![id, summary.trim()],
            )
            .map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        load_interview_session(&connection, id)
    }

    pub fn delete_interview_session(&self, id: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute("DELETE FROM interview_sessions WHERE id=?1", [id])
            .map_err(db_error)?;
        if changed == 0 {
            return Err("面试会话不存在".into());
        }
        Ok(())
    }

    pub fn list_question_bank_items(&self) -> Result<Vec<QuestionBankItem>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT id,prompt,category,best_answer,mastery,source,occurrence_count,last_seen_at
                 FROM question_bank_items ORDER BY last_seen_at DESC,updated_at DESC",
            )
            .map_err(db_error)?;
        let items = statement
            .query_map([], question_bank_row)
            .map_err(db_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        Ok(items)
    }

    pub fn save_question_bank_item(
        &self,
        id: Option<&str>,
        prompt: &str,
        category: &str,
        best_answer: &str,
        mastery: &str,
    ) -> Result<QuestionBankItem, String> {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return Err("问题不能为空".into());
        }
        if prompt.chars().count() > 10_000
            || best_answer.chars().count() > 100_000
            || category.chars().count() > 100
        {
            return Err("题库问题、类型或参考回答过长".into());
        }
        if !matches!(mastery, "待加强" | "练习中" | "熟悉" | "掌握") {
            return Err("掌握程度无效".into());
        }
        let normalized = normalize_question(prompt);
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let item_id = id
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        if id.is_some() {
            let changed = connection
                .execute(
                    "UPDATE question_bank_items SET normalized_key=?2,prompt=?3,category=?4,best_answer=?5,mastery=?6,
                     updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
                    params![item_id, normalized, prompt, category.trim(), best_answer.trim(), mastery],
                )
                .map_err(|error| match error {
                    rusqlite::Error::SqliteFailure(_, Some(message)) if message.contains("UNIQUE") => "题库中已存在相同问题".into(),
                    other => db_error(other),
                })?;
            if changed == 0 {
                return Err("题库问题不存在".into());
            }
        } else {
            connection
                .execute(
                    "INSERT INTO question_bank_items(id,normalized_key,prompt,category,best_answer,mastery,source)
                     VALUES (?1,?2,?3,?4,?5,?6,'手动')",
                    params![item_id, normalized, prompt, category.trim(), best_answer.trim(), mastery],
                )
                .map_err(|error| match error {
                    rusqlite::Error::SqliteFailure(_, Some(message)) if message.contains("UNIQUE") => "题库中已存在相同问题".into(),
                    other => db_error(other),
                })?;
        }
        connection
            .query_row(
                "SELECT id,prompt,category,best_answer,mastery,source,occurrence_count,last_seen_at FROM question_bank_items WHERE id=?1",
                [&item_id],
                question_bank_row,
            )
            .map_err(db_error)
    }

    pub fn delete_question_bank_item(&self, id: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute("DELETE FROM question_bank_items WHERE id=?1", [id])
            .map_err(db_error)?;
        if changed == 0 {
            return Err("题库问题不存在".into());
        }
        Ok(())
    }
}

fn load_interview_session(
    connection: &Connection,
    id: &str,
) -> Result<InterviewSessionRecord, String> {
    let row = connection
        .query_row(
            "SELECT id,application_id,session_type,round,created_at,duration_seconds,status,current_question_index,review_summary
             FROM interview_sessions WHERE id=?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            },
        )
        .optional()
        .map_err(db_error)?
        .ok_or_else(|| "面试会话不存在".to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT id,prompt,source,answer,score,evaluation FROM interview_session_questions
             WHERE session_id=?1 ORDER BY position",
        )
        .map_err(db_error)?;
    let questions = statement
        .query_map([id], |row| {
            Ok(InterviewQuestionRecord {
                id: row.get(0)?,
                prompt: row.get(1)?,
                source: row.get(2)?,
                answer: row.get(3)?,
                score: row.get(4)?,
                evaluation: row.get(5)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(InterviewSessionRecord {
        id: row.0,
        application_id: row.1,
        session_type: row.2,
        round: row.3,
        created_at: row.4,
        duration: row.5.map(format_duration).unwrap_or_else(|| {
            if row.6 == "进行中" {
                "进行中".into()
            } else {
                "未记录".into()
            }
        }),
        status: row.6,
        current_question_index: row.7,
        review_summary: row.8,
        questions,
    })
}

fn upsert_question_bank(connection: &Connection, prompt: &str, source: &str) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO question_bank_items(id,normalized_key,prompt,category,source)
             VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(normalized_key) DO UPDATE SET occurrence_count=occurrence_count+1,
                 last_seen_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            params![Uuid::new_v4().to_string(), normalize_question(prompt), prompt, infer_category(prompt), source],
        )
        .map_err(db_error)?;
    Ok(())
}

fn question_bank_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<QuestionBankItem> {
    Ok(QuestionBankItem {
        id: row.get(0)?,
        prompt: row.get(1)?,
        category: row.get(2)?,
        best_answer: row.get(3)?,
        mastery: row.get(4)?,
        source: row.get(5)?,
        occurrence_count: row.get(6)?,
        last_seen_at: row.get(7)?,
    })
}

fn normalize_question(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            !character.is_whitespace() && !"，。！？?；;：:、,.".contains(*character)
        })
        .flat_map(char::to_lowercase)
        .collect()
}

fn infer_category(prompt: &str) -> &'static str {
    if ["冲突", "协作", "团队", "失败", "困难", "经历", "STAR"]
        .iter()
        .any(|value| prompt.contains(value))
    {
        "行为面试"
    } else if ["为什么选择", "为什么加入", "职业规划", "工作方式", "公司吗"]
        .iter()
        .any(|value| prompt.contains(value))
    {
        "岗位动机"
    } else if ["项目", "负责", "贡献", "指标", "重构"]
        .iter()
        .any(|value| prompt.contains(value))
    {
        "项目深挖"
    } else {
        "专业知识"
    }
}

fn mastery_from_score(score: i64) -> &'static str {
    match score {
        0..=59 => "待加强",
        60..=74 => "练习中",
        75..=89 => "熟悉",
        _ => "掌握",
    }
}

fn format_duration(seconds: i64) -> String {
    let minutes = (seconds.max(1) + 59) / 60;
    format!("{minutes} 分钟")
}

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeQuestionInput {
    pub target_id: String,
    pub source_id: String,
    pub display_prompt: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitQuestionInput {
    pub question_id: String,
    pub observation_ids: Vec<String>,
    pub display_prompt: String,
}

fn default_bank_status() -> String {
    "active".into()
}
fn default_bank_sort() -> String {
    "review_priority".into()
}
fn default_bank_direction() -> String {
    "desc".into()
}
fn default_page_size() -> i64 {
    30
}

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
            let question_id = Uuid::new_v4().to_string();
            let canonical_id =
                find_or_create_question(&transaction, prompt, infer_category(prompt))?;
            transaction
                .execute(
                    "INSERT INTO interview_session_questions(id,session_id,position,prompt,source,answer,canonical_question_id)
                     VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        question_id,
                        id,
                        position as i64,
                        prompt,
                        question.source,
                        question.answer.trim(),
                        canonical_id
                    ],
                )
                .map_err(db_error)?;
            if session_type == "真实面试" {
                observe_interview_question(
                    &transaction,
                    &canonical_id,
                    &question_id,
                    &InterviewObservationContext {
                        session_id: &id,
                        application_id,
                        round,
                        event_type: "real_asked",
                        verification_state: "inferred",
                    },
                )?;
                ensure_membership(&transaction, &canonical_id, "real_interview")?;
            } else if !question.answer.trim().is_empty() {
                observe_interview_question(
                    &transaction,
                    &canonical_id,
                    &question_id,
                    &InterviewObservationContext {
                        session_id: &id,
                        application_id,
                        round,
                        event_type: "mock_answered",
                        verification_state: "confirmed",
                    },
                )?;
                ensure_membership(&transaction, &canonical_id, "mock_practice")?;
            }
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
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let changed = transaction
            .execute(
                "UPDATE interview_session_questions SET answer=?3,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id=?2 AND session_id=?1
                   AND EXISTS(SELECT 1 FROM interview_sessions WHERE id=?1 AND status='进行中')",
                params![session_id, question_id, answer],
            )
            .map_err(db_error)?;
        if changed == 0 {
            let in_progress: bool = transaction
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
        if !answer.trim().is_empty() {
            let context: Option<(String, String, String, String)> = transaction
                .query_row(
                    "SELECT q.canonical_question_id,s.session_type,s.application_id,s.round
                     FROM interview_session_questions q
                     JOIN interview_sessions s ON s.id=q.session_id
                     WHERE q.id=?1 AND q.session_id=?2",
                    params![question_id, session_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .optional()
                .map_err(db_error)?;
            if let Some((canonical_id, session_type, application_id, round)) = context {
                let (event_type, verification) = if session_type == "真实面试" {
                    ("real_asked", "confirmed")
                } else {
                    ("mock_answered", "confirmed")
                };
                observe_interview_question(
                    &transaction,
                    &canonical_id,
                    question_id,
                    &InterviewObservationContext {
                        session_id,
                        application_id: &application_id,
                        round: &round,
                        event_type,
                        verification_state: verification,
                    },
                )?;
                ensure_membership(
                    &transaction,
                    &canonical_id,
                    if event_type == "real_asked" {
                        "real_interview"
                    } else {
                        "mock_practice"
                    },
                )?;
            }
        }
        transaction
            .execute(
                "UPDATE interview_sessions SET updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
                [session_id],
            )
            .map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
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
            let question: (String, String, String) = transaction
                .query_row(
                    "SELECT prompt,answer,canonical_question_id FROM interview_session_questions WHERE id=?1",
                    [&review.question_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(db_error)?;
            let scores = transaction
                .prepare(
                    "SELECT score FROM interview_session_questions
                 WHERE canonical_question_id=?1 AND score IS NOT NULL AND answer<>''
                 ORDER BY updated_at DESC LIMIT 3",
                )
                .map_err(db_error)?
                .query_map([&question.2], |row| row.get::<_, i64>(0))
                .map_err(db_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(db_error)?;
            let mastery = mastery_from_scores(&scores);
            transaction
                .execute(
                    "UPDATE canonical_questions
                     SET system_mastery=?2,best_answer=CASE WHEN best_answer='' AND ?3>=80 AND ?4<>'' THEN ?4 ELSE best_answer END,
                         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE id=?1",
                    params![question.2, mastery, review.score, question.1],
                )
                .map_err(db_error)?;
            transaction
                .execute(
                    "UPDATE question_observations SET verification_state='confirmed',
                 updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE source_type='interview_session' AND source_id=?1 AND source_item_id=?2
                   AND event_type='real_asked'",
                    params![id, review.question_id],
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

    pub fn list_question_bank_items(
        &self,
        input: &ListQuestionBankInput,
    ) -> Result<QuestionBankPage, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let page_size = input.page_size.clamp(1, 100);
        let offset = decode_cursor(input.cursor.as_deref())?;
        let status = if input.status == "archived" {
            "archived"
        } else {
            "active"
        };
        let keyword = format!("%{}%", input.query.trim().to_lowercase());
        let mut where_sql = "m.status=?1".to_string();
        let mut values: Vec<rusqlite::types::Value> = vec![status.to_string().into()];
        if !input.query.trim().is_empty() {
            where_sql.push_str(
                " AND (lower(cq.display_prompt) LIKE ?2 OR EXISTS(
                    SELECT 1 FROM question_variants qv
                    WHERE qv.question_id=cq.id AND lower(qv.raw_prompt) LIKE ?2
                ))",
            );
            values.push(keyword.into());
        }
        if input.review_state.as_deref() == Some("due") {
            where_sql.push_str(
                " AND (
                    (cq.next_review_at IS NOT NULL AND cq.next_review_at<=strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                    OR (
                        COALESCE(cq.manual_mastery,cq.system_mastery) IN ('待加强','练习中')
                        AND NOT EXISTS(
                            SELECT 1 FROM question_observations review_o
                            WHERE review_o.question_id=cq.id
                              AND review_o.event_type='mock_answered'
                              AND review_o.verification_state='confirmed'
                              AND review_o.occurred_at>strftime('%Y-%m-%dT%H:%M:%fZ','now','-60 days')
                        )
                    )
                )",
            );
        }
        if !input.mastery.is_empty() {
            let placeholders = (0..input.mastery.len())
                .map(|index| format!("?{}", values.len() + index + 1))
                .collect::<Vec<_>>()
                .join(",");
            where_sql.push_str(&format!(
                " AND COALESCE(cq.manual_mastery,cq.system_mastery) IN ({placeholders})"
            ));
            values.extend(input.mastery.iter().cloned().map(Into::into));
        }
        let total: i64 = connection.query_row(
            &format!("SELECT COUNT(*) FROM canonical_questions cq JOIN question_bank_memberships m ON m.question_id=cq.id WHERE cq.redirect_to_id IS NULL AND {where_sql}"),
            rusqlite::params_from_iter(values.iter()), |row| row.get(0),
        ).map_err(db_error)?;
        let direction = if input.direction == "asc" {
            "ASC"
        } else {
            "DESC"
        };
        let sort = match input.sort.as_str() {
            "real_frequency" => "(SELECT COUNT(DISTINCT source_id) FROM question_observations WHERE question_id=cq.id AND event_type='real_asked' AND verification_state='confirmed')",
            "reference_frequency" => "(SELECT COUNT(DISTINCT source_id) FROM question_observations WHERE question_id=cq.id AND event_type='reference_mentioned' AND verification_state<>'rejected')",
            "last_real_asked" => "(SELECT MAX(occurred_at) FROM question_observations WHERE question_id=cq.id AND event_type='real_asked' AND verification_state='confirmed')",
            "last_practiced" => "(SELECT MAX(occurred_at) FROM question_observations WHERE question_id=cq.id AND event_type='mock_answered' AND verification_state='confirmed')",
            "created_at" => "cq.created_at",
            _ => "CASE WHEN cq.next_review_at IS NOT NULL AND cq.next_review_at<=strftime('%Y-%m-%dT%H:%M:%fZ','now') THEN 0 ELSE 1 END, CASE COALESCE(cq.manual_mastery,cq.system_mastery) WHEN '待加强' THEN 0 WHEN '练习中' THEN 1 WHEN '熟悉' THEN 2 ELSE 3 END",
        };
        let mut page_values = values.clone();
        page_values.push((page_size + 1).into());
        page_values.push(offset.into());
        let sql = format!(
            "SELECT cq.id FROM canonical_questions cq
             JOIN question_bank_memberships m ON m.question_id=cq.id
             WHERE cq.redirect_to_id IS NULL AND {where_sql}
             ORDER BY {sort} {direction},cq.id {direction}
             LIMIT ?{} OFFSET ?{}",
            page_values.len() - 1,
            page_values.len()
        );
        let mut statement = connection.prepare(&sql).map_err(db_error)?;
        let mut ids = statement
            .query_map(rusqlite::params_from_iter(page_values.iter()), |row| {
                row.get::<_, String>(0)
            })
            .map_err(db_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        let has_more = ids.len() > page_size as usize;
        ids.truncate(page_size as usize);
        let items = ids
            .iter()
            .map(|id| load_question_summary(&connection, id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(QuestionBankPage {
            items,
            total,
            next_cursor: has_more.then(|| encode_cursor(offset + page_size)),
            facets: load_question_facets(&connection)?,
        })
    }

    pub fn save_question_bank_item(
        &self,
        id: Option<&str>,
        prompt: &str,
        category: &str,
        best_answer: &str,
        mastery: &str,
        force_new: bool,
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
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let item_id = id
            .map(str::to_string)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        if id.is_some() {
            let changed = transaction
                .execute(
                    "UPDATE canonical_questions SET display_prompt=?2,question_type=?3,best_answer=?4,
                     manual_mastery=?5,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE id=?1 AND redirect_to_id IS NULL",
                    params![item_id, prompt, category.trim(), best_answer.trim(), mastery],
                ).map_err(db_error)?;
            if changed == 0 {
                return Err("题库问题不存在".into());
            }
            let variant_id = ensure_variant(&transaction, &item_id, prompt, true)?;
            ensure_membership(&transaction, &item_id, "manual")?;
            transaction.execute(
                "INSERT INTO question_observations(
                    id,question_id,variant_id,event_type,source_type,source_id,source_item_id,occurred_at
                 ) VALUES (?1,?2,?3,'manual_saved','manual',?2,?2,strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT(event_type,source_type,source_id,source_item_id) DO NOTHING",
                params![Uuid::new_v4().to_string(),item_id,variant_id],
            ).map_err(db_error)?;
        } else {
            if !force_new
                && find_question_by_normalized(&transaction, &normalize_question(prompt))?.is_some()
            {
                return Err("题库中已存在相同问题；请编辑已有题或从候选中选择关联".into());
            }
            transaction.execute(
                "INSERT INTO canonical_questions(id,display_prompt,question_type,manual_mastery,best_answer)
                 VALUES (?1,?2,?3,?4,?5)",
                params![item_id,prompt,category.trim(),mastery,best_answer.trim()],
            ).map_err(db_error)?;
            let variant_id = ensure_variant(&transaction, &item_id, prompt, true)?;
            ensure_membership(&transaction, &item_id, "manual")?;
            transaction.execute(
                "INSERT INTO question_observations(
                    id,question_id,variant_id,event_type,source_type,source_id,source_item_id,occurred_at
                 ) VALUES (?1,?2,?3,'manual_saved','manual',?2,?2,strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                 ON CONFLICT(event_type,source_type,source_id,source_item_id) DO NOTHING",
                params![Uuid::new_v4().to_string(),item_id,variant_id],
            ).map_err(db_error)?;
        }
        transaction.commit().map_err(db_error)?;
        load_question_summary(&connection, &item_id)
    }

    pub fn delete_question_bank_item(&self, id: &str) -> Result<(), String> {
        self.archive_question_bank_item(id)
    }

    pub fn archive_question_bank_item(&self, id: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute(
                "UPDATE question_bank_memberships SET status='archived',
                 archived_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE question_id=?1 AND status='active'",
                [id],
            )
            .map_err(db_error)?;
        if changed == 0 {
            return Err("题库问题不存在或已归档".into());
        }
        Ok(())
    }

    pub fn restore_question_bank_item(&self, id: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute(
                "UPDATE question_bank_memberships SET status='active',archived_at=NULL
             WHERE question_id=?1 AND status='archived'",
                [id],
            )
            .map_err(db_error)?;
        if changed == 0 {
            return Err("归档题目不存在".into());
        }
        Ok(())
    }

    pub fn get_question_bank_item(&self, id: &str) -> Result<QuestionBankDetail, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let item = load_question_summary(&connection, id)?;
        let variants = connection
            .prepare(
                "SELECT qv.raw_prompt FROM question_variants qv
             JOIN canonical_questions owner ON owner.id=qv.question_id
             WHERE qv.question_id=?1 OR owner.redirect_to_id=?1
             ORDER BY qv.last_seen_at DESC",
            )
            .map_err(db_error)?
            .query_map([id], |row| row.get(0))
            .map_err(db_error)?
            .collect::<Result<Vec<String>, _>>()
            .map_err(db_error)?;
        let evidence = connection.prepare(
            "SELECT o.id,o.event_type,o.source_type,o.source_id,o.source_item_id,
                    COALESCE(v.raw_prompt,cq.display_prompt),c.name,p.title,o.round,o.occurred_at,o.verification_state
             FROM question_observations o
             JOIN canonical_questions cq ON cq.id=o.question_id
             LEFT JOIN question_variants v ON v.id=o.variant_id
             LEFT JOIN companies c ON c.id=o.company_id
             LEFT JOIN positions p ON p.id=o.position_id
             WHERE o.question_id=?1 ORDER BY o.occurred_at DESC,o.id"
        ).map_err(db_error)?.query_map([id], |row| Ok(QuestionEvidence {
            id: row.get(0)?, event_type: row.get(1)?, source_type: row.get(2)?,
            source_id: row.get(3)?, source_item_id: row.get(4)?, prompt: row.get(5)?,
            company: row.get(6)?, position: row.get(7)?, round: row.get(8)?,
            occurred_at: row.get(9)?, verification_state: row.get(10)?,
        })).map_err(db_error)?.collect::<Result<Vec<_>, _>>().map_err(db_error)?;
        Ok(QuestionBankDetail {
            item,
            variants,
            evidence,
        })
    }

    pub fn sync_experience_question_observations(&self, source_id: &str) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let context: (String, String, String, String, String) = transaction
            .query_row(
                "SELECT e.questions_json,e.application_id,p.company_id,p.id,e.updated_at
             FROM interview_experience_sources e
             JOIN applications a ON a.id=e.application_id
             JOIN positions p ON p.id=a.position_id WHERE e.id=?1",
                [source_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .map_err(db_error)?;
        let questions: Vec<String> =
            serde_json::from_str(&context.0).map_err(|error| error.to_string())?;
        transaction
            .execute(
                "DELETE FROM question_observations WHERE source_type='experience' AND source_id=?1",
                [source_id],
            )
            .map_err(db_error)?;
        let mut seen = std::collections::HashSet::new();
        for prompt in questions {
            let canonical_id =
                find_or_create_question(&transaction, &prompt, infer_category(&prompt))?;
            if !seen.insert(canonical_id.clone()) {
                continue;
            }
            let variant_id = ensure_variant(&transaction, &canonical_id, &prompt, false)?;
            transaction.execute(
                "INSERT INTO question_observations(
                    id,question_id,variant_id,event_type,source_type,source_id,source_item_id,
                    application_id,company_id,position_id,occurred_at,verification_state
                 ) VALUES (?1,?2,?3,'reference_mentioned','experience',?4,?2,?5,?6,?7,?8,'confirmed')",
                params![
                    Uuid::new_v4().to_string(),canonical_id,variant_id,source_id,
                    context.1,context.2,context.3,context.4
                ],
            ).map_err(db_error)?;
        }
        transaction.commit().map_err(db_error)
    }

    pub fn list_question_match_candidates(
        &self,
        prompt: &str,
        exclude_id: Option<&str>,
    ) -> Result<Vec<QuestionMatchCandidate>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT cq.id,cq.display_prompt FROM canonical_questions cq
             JOIN question_bank_memberships m ON m.question_id=cq.id
             WHERE cq.redirect_to_id IS NULL AND m.status='active' AND (?1 IS NULL OR cq.id<>?1)
               AND NOT EXISTS(
                 SELECT 1 FROM question_match_decisions d
                 WHERE d.allow_resuggest=0
                   AND ((d.left_question_id=?1 AND d.right_question_id=cq.id)
                     OR (d.right_question_id=?1 AND d.left_question_id=cq.id))
               )
             ORDER BY cq.updated_at DESC LIMIT 2000",
            )
            .map_err(db_error)?;
        let candidates = statement
            .query_map([exclude_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(db_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        let mut scored = Vec::new();
        for (id, candidate_prompt) in candidates {
            let (score, reason, conflict) = question_similarity(prompt, &candidate_prompt);
            if score < 0.2 {
                continue;
            }
            let mut question = load_question_summary(&connection, &id)?;
            if conflict {
                question.needs_review = false;
            }
            scored.push(QuestionMatchCandidate {
                question,
                score: if conflict { score.min(0.49) } else { score },
                reason: if conflict {
                    format!("存在冲突项：{reason}")
                } else {
                    reason
                },
            });
        }
        scored.sort_by(|left, right| right.score.total_cmp(&left.score));
        scored.truncate(5);
        Ok(scored)
    }

    pub fn resolve_question_match(
        &self,
        left_id: &str,
        right_id: &str,
        action: &str,
        reason: &str,
    ) -> Result<Option<String>, String> {
        if left_id == right_id {
            return Err("不能对同一道题执行匹配操作".into());
        }
        if action == "merge" {
            let prompt = {
                let connection = self
                    .connection
                    .lock()
                    .map_err(|_| "数据库连接锁已损坏".to_string())?;
                connection
                    .query_row(
                        "SELECT display_prompt FROM canonical_questions WHERE id=?1",
                        [left_id],
                        |row| row.get::<_, String>(0),
                    )
                    .map_err(db_error)?
            };
            return self
                .merge_question_bank_items(&MergeQuestionInput {
                    target_id: left_id.into(),
                    source_id: right_id.into(),
                    display_prompt: prompt,
                    reason: reason.into(),
                })
                .map(Some);
        }
        if !matches!(action, "same_topic" | "keep_separate") {
            return Err("匹配处理方式无效".into());
        }
        let (left, right) = ordered_pair(left_id, right_id);
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute(
            "INSERT INTO question_match_decisions(
                id,left_question_id,right_question_id,system_decision,reason,user_action,allow_resuggest
             ) VALUES (?1,?2,?3,'pending',?4,?5,0)
             ON CONFLICT(left_question_id,right_question_id) DO UPDATE SET
                reason=excluded.reason,user_action=excluded.user_action,allow_resuggest=0",
            params![Uuid::new_v4().to_string(),left,right,reason,action],
        ).map_err(db_error)?;
        Ok(None)
    }

    pub fn merge_question_bank_items(&self, input: &MergeQuestionInput) -> Result<String, String> {
        if input.target_id == input.source_id || input.display_prompt.trim().is_empty() {
            return Err("合并参数无效".into());
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let target_exists: bool = transaction
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM canonical_questions
                    WHERE id=?1 AND redirect_to_id IS NULL
                )",
                [&input.target_id],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        if !target_exists {
            return Err("合并目标不存在或已经合并到其他问题".into());
        }
        let source: (String,String,String,Option<String>,String,String) = transaction.query_row(
            "SELECT display_prompt,question_type,system_mastery,manual_mastery,best_answer,
                    COALESCE((SELECT status||':'||added_reason FROM question_bank_memberships WHERE question_id=?1),'active:manual')
             FROM canonical_questions WHERE id=?1 AND redirect_to_id IS NULL",
            [&input.source_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).map_err(db_error)?;
        let variants = collect_ids(
            &transaction,
            "SELECT id FROM question_variants WHERE question_id=?1",
            &input.source_id,
        )?;
        let observations = collect_ids(
            &transaction,
            "SELECT id FROM question_observations WHERE question_id=?1",
            &input.source_id,
        )?;
        let snapshot = serde_json::json!({
            "source": source, "variants": variants, "observations": observations
        })
        .to_string();
        let audit_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO question_merge_audits(id,target_question_id,source_question_id,snapshot_json,reason)
             VALUES (?1,?2,?3,?4,?5)",
            params![audit_id,input.target_id,input.source_id,snapshot,input.reason],
        ).map_err(db_error)?;
        for variant_id in &variants {
            let variant: (String, String) = transaction
                .query_row(
                    "SELECT normalized_prompt,raw_prompt FROM question_variants WHERE id=?1",
                    [variant_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(db_error)?;
            if let Some(target_variant) = transaction.query_row(
                "SELECT id FROM question_variants WHERE question_id=?1 AND normalized_prompt=?2",
                params![input.target_id,variant.0], |row| row.get::<_, String>(0),
            ).optional().map_err(db_error)? {
                transaction.execute(
                    "UPDATE question_observations SET variant_id=?1 WHERE variant_id=?2",
                    params![target_variant,variant_id],
                ).map_err(db_error)?;
            } else {
                transaction.execute(
                    "UPDATE question_variants SET question_id=?1,confirmed_equivalent=1 WHERE id=?2",
                    params![input.target_id,variant_id],
                ).map_err(db_error)?;
            }
        }
        transaction
            .execute(
                "UPDATE question_observations SET question_id=?1 WHERE question_id=?2",
                params![input.target_id, input.source_id],
            )
            .map_err(db_error)?;
        transaction
            .execute(
                "UPDATE question_answer_versions SET question_id=?1 WHERE question_id=?2",
                params![input.target_id, input.source_id],
            )
            .map_err(db_error)?;
        ensure_membership(&transaction, &input.target_id, "manual")?;
        transaction
            .execute(
                "DELETE FROM question_bank_memberships WHERE question_id=?1",
                [&input.source_id],
            )
            .map_err(db_error)?;
        transaction.execute(
            "UPDATE canonical_questions SET display_prompt=?2,
             best_answer=CASE WHEN best_answer='' THEN (SELECT best_answer FROM canonical_questions WHERE id=?3) ELSE best_answer END,
             updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            params![input.target_id,input.display_prompt.trim(),input.source_id],
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE canonical_questions SET redirect_to_id=?1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?2",
            params![input.target_id,input.source_id],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        Ok(audit_id)
    }

    pub fn undo_question_merge(&self, audit_id: &str) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (source_id, snapshot): (String, String) = transaction
            .query_row(
                "SELECT source_question_id,snapshot_json FROM question_merge_audits
             WHERE id=?1 AND undone_at IS NULL",
                [audit_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(db_error)?;
        let value: serde_json::Value =
            serde_json::from_str(&snapshot).map_err(|error| error.to_string())?;
        let variants = json_ids(&value, "variants")?;
        let observations = json_ids(&value, "observations")?;
        transaction
            .execute(
                "UPDATE canonical_questions SET redirect_to_id=NULL WHERE id=?1",
                [&source_id],
            )
            .map_err(db_error)?;
        for id in variants {
            transaction
                .execute(
                    "UPDATE question_variants SET question_id=?1 WHERE id=?2",
                    params![source_id, id],
                )
                .map_err(db_error)?;
        }
        for id in observations {
            transaction
                .execute(
                    "UPDATE question_observations
                     SET question_id=?1,
                         variant_id=COALESCE(
                             (
                                 SELECT source_variant.id
                                 FROM question_variants current_variant
                                 JOIN question_variants source_variant
                                   ON source_variant.question_id=?1
                                  AND source_variant.normalized_prompt=current_variant.normalized_prompt
                                 WHERE current_variant.id=question_observations.variant_id
                                 LIMIT 1
                             ),
                             variant_id
                         )
                     WHERE id=?2",
                    params![source_id, id],
                )
                .map_err(db_error)?;
        }
        let membership = value["source"][5].as_str().unwrap_or("active:manual");
        let mut parts = membership.split(':');
        let status = parts.next().unwrap_or("active");
        let reason = parts.next().unwrap_or("manual");
        transaction.execute(
            "INSERT OR REPLACE INTO question_bank_memberships(question_id,status,added_reason,archived_at)
             VALUES (?1,?2,?3,CASE WHEN ?2='archived' THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') END)",
            params![source_id,status,reason],
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE question_merge_audits SET undone_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            [audit_id],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)
    }

    pub fn split_question_bank_item(
        &self,
        input: &SplitQuestionInput,
    ) -> Result<QuestionBankItem, String> {
        if input.observation_ids.is_empty() || input.display_prompt.trim().is_empty() {
            return Err("拆分时必须选择来源事件并填写新标准题".into());
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let category: String = transaction.query_row(
            "SELECT question_type FROM canonical_questions WHERE id=?1 AND redirect_to_id IS NULL",
            [&input.question_id], |row| row.get(0),
        ).map_err(db_error)?;
        let new_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO canonical_questions(id,display_prompt,question_type) VALUES (?1,?2,?3)",
            params![new_id,input.display_prompt.trim(),category],
        ).map_err(db_error)?;
        let new_variant = ensure_variant(&transaction, &new_id, input.display_prompt.trim(), true)?;
        ensure_membership(&transaction, &new_id, "manual")?;
        for observation_id in &input.observation_ids {
            let changed = transaction
                .execute(
                    "UPDATE question_observations SET question_id=?1,variant_id=?2
                 WHERE id=?3 AND question_id=?4",
                    params![new_id, new_variant, observation_id, input.question_id],
                )
                .map_err(db_error)?;
            if changed == 0 {
                return Err("拆分来源不属于原题".into());
            }
        }
        let (left, right) = ordered_pair(&input.question_id, &new_id);
        transaction.execute(
            "INSERT INTO question_match_decisions(
                id,left_question_id,right_question_id,system_decision,reason,user_action,allow_resuggest
             ) VALUES (?1,?2,?3,'different','用户拆分题目','keep_separate',0)",
            params![Uuid::new_v4().to_string(),left,right],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        load_question_summary(&connection, &new_id)
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

fn normalize_question(value: &str) -> String {
    let normalized: String = value.nfkc().collect();
    let mut normalized = normalized
        .chars()
        .map(|character| match character {
            '，' => ',',
            '。' => '.',
            '！' => '!',
            '？' => '?',
            '；' => ';',
            '：' => ':',
            '（' => '(',
            '）' => ')',
            other => other,
        })
        .flat_map(char::to_lowercase)
        .collect::<String>();
    normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized = normalized
        .trim_matches(|character: char| {
            character.is_ascii_punctuation() || "、。！？；：".contains(character)
        })
        .to_string();
    for suffix in ["呢", "吗"] {
        if normalized.ends_with(suffix) {
            normalized.truncate(normalized.len() - suffix.len());
        }
    }
    normalized = normalized.trim_end().to_string();
    let trimmed = normalized.trim_start();
    let without_number = trimmed.trim_start_matches(|character: char| {
        character.is_ascii_digit() || "一二三四五六七八九十、.．:：问题 ".contains(character)
    });
    if without_number.is_empty() {
        normalized
    } else {
        without_number.trim().to_string()
    }
}

fn find_question_by_normalized(
    connection: &Connection,
    normalized: &str,
) -> Result<Option<String>, String> {
    let exact = connection
        .query_row(
            "SELECT cq.id FROM question_variants qv
         JOIN canonical_questions cq ON cq.id=qv.question_id
         WHERE qv.normalized_prompt=?1 AND cq.redirect_to_id IS NULL
         ORDER BY qv.confirmed_equivalent DESC,qv.created_at LIMIT 1",
            [normalized],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)?;
    if exact.is_some() {
        return Ok(exact);
    }
    let mut statement = connection
        .prepare(
            "SELECT cq.id,qv.raw_prompt FROM question_variants qv
             JOIN canonical_questions cq ON cq.id=qv.question_id
             WHERE cq.redirect_to_id IS NULL",
        )
        .map_err(db_error)?;
    let variants = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(db_error)?;
    for variant in variants {
        let (id, raw_prompt) = variant.map_err(db_error)?;
        if normalize_question(&raw_prompt) == normalized {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

fn find_or_create_question(
    connection: &Connection,
    prompt: &str,
    category: &str,
) -> Result<String, String> {
    let normalized = normalize_question(prompt);
    if let Some(id) = find_question_by_normalized(connection, &normalized)? {
        ensure_variant(connection, &id, prompt, false)?;
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO canonical_questions(id,display_prompt,question_type) VALUES (?1,?2,?3)",
            params![id, prompt, category],
        )
        .map_err(db_error)?;
    ensure_variant(connection, &id, prompt, false)?;
    Ok(id)
}

fn ensure_variant(
    connection: &Connection,
    question_id: &str,
    prompt: &str,
    confirmed: bool,
) -> Result<String, String> {
    let normalized = normalize_question(prompt);
    let existing = connection
        .query_row(
            "SELECT id FROM question_variants WHERE question_id=?1 AND normalized_prompt=?2",
            params![question_id, normalized],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)?;
    if let Some(id) = existing {
        connection
            .execute(
                "UPDATE question_variants SET last_seen_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),
             confirmed_equivalent=MAX(confirmed_equivalent,?2) WHERE id=?1",
                params![id, confirmed],
            )
            .map_err(db_error)?;
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO question_variants(id,question_id,raw_prompt,normalized_prompt,language,confirmed_equivalent)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![id,question_id,prompt,normalized,detect_language(prompt),confirmed],
    ).map_err(db_error)?;
    Ok(id)
}

fn ensure_membership(
    connection: &Connection,
    question_id: &str,
    reason: &str,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO question_bank_memberships(question_id,status,added_reason)
         VALUES (?1,'active',?2)
         ON CONFLICT(question_id) DO UPDATE SET status='active',archived_at=NULL",
            params![question_id, reason],
        )
        .map_err(db_error)?;
    Ok(())
}

struct InterviewObservationContext<'a> {
    session_id: &'a str,
    application_id: &'a str,
    round: &'a str,
    event_type: &'a str,
    verification_state: &'a str,
}

fn observe_interview_question(
    connection: &Connection,
    question_id: &str,
    source_item_id: &str,
    context: &InterviewObservationContext<'_>,
) -> Result<(), String> {
    let variant_id: String = connection.query_row(
        "SELECT id FROM question_variants WHERE question_id=?1 ORDER BY last_seen_at DESC LIMIT 1",
        [question_id], |row| row.get(0),
    ).map_err(db_error)?;
    connection
        .execute(
            "INSERT INTO question_observations(
            id,question_id,variant_id,event_type,source_type,source_id,source_item_id,
            application_id,company_id,position_id,round,occurred_at,verification_state
         )
         SELECT ?1,?2,?3,?4,'interview_session',?5,?6,a.id,p.company_id,p.id,?7,
                COALESCE(s.completed_at,s.created_at),?8
         FROM applications a JOIN positions p ON p.id=a.position_id
         JOIN interview_sessions s ON s.id=?5
         WHERE a.id=?9
         ON CONFLICT(event_type,source_type,source_id,source_item_id) DO UPDATE SET
            verification_state=excluded.verification_state,
            occurred_at=excluded.occurred_at,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",
            params![
                Uuid::new_v4().to_string(),
                question_id,
                variant_id,
                context.event_type,
                context.session_id,
                source_item_id,
                context.round,
                context.verification_state,
                context.application_id
            ],
        )
        .map_err(db_error)?;
    Ok(())
}

fn load_question_summary(connection: &Connection, id: &str) -> Result<QuestionBankItem, String> {
    connection.query_row(
        "SELECT cq.id,cq.display_prompt,cq.question_type,cq.best_answer,
                COALESCE(cq.manual_mastery,cq.system_mastery),cq.system_mastery,cq.manual_mastery,m.status,
                COUNT(DISTINCT CASE WHEN o.event_type='real_asked' AND o.verification_state='confirmed' THEN o.source_id END),
                SUM(CASE WHEN o.event_type='real_asked' AND o.verification_state='confirmed' THEN 1 ELSE 0 END),
                COUNT(DISTINCT CASE WHEN o.event_type='mock_answered' AND o.verification_state='confirmed' THEN o.source_item_id END),
                COUNT(DISTINCT CASE WHEN o.event_type='reference_mentioned' AND o.verification_state<>'rejected' THEN o.source_id END),
                COUNT(DISTINCT CASE WHEN o.event_type='real_asked' AND o.verification_state='confirmed' THEN o.company_id END),
                COALESCE(SUM(CASE WHEN o.event_type='imported_legacy' THEN o.legacy_count ELSE 0 END),0),
                MAX(CASE WHEN o.event_type='real_asked' AND o.verification_state='confirmed' THEN o.occurred_at END),
                MAX(CASE WHEN o.event_type='mock_answered' AND o.verification_state='confirmed' THEN o.occurred_at END),
                cq.next_review_at,cq.created_at,
                GROUP_CONCAT(DISTINCT CASE o.event_type
                    WHEN 'real_asked' THEN '真实面试' WHEN 'mock_answered' THEN '模拟练习'
                    WHEN 'reference_mentioned' THEN '面经' WHEN 'manual_saved' THEN '手动'
                    WHEN 'ai_generated' THEN 'AI 生成' WHEN 'imported_legacy' THEN '旧版记录' END)
         FROM canonical_questions cq JOIN question_bank_memberships m ON m.question_id=cq.id
         LEFT JOIN question_observations o ON o.question_id=cq.id
         WHERE cq.id=?1 GROUP BY cq.id",
        [id], |row| {
            let mastery: String = row.get(4)?;
            let next_review_at: Option<String> = row.get(16)?;
            let last_practiced_at: Option<String> = row.get(15)?;
            let sources: Option<String> = row.get(18)?;
            let low_mastery = matches!(mastery.as_str(), "待加强" | "练习中");
            let needs_review = next_review_at.as_deref().is_some_and(|value| {
                value <= chrono::Utc::now().to_rfc3339().as_str()
            }) || low_mastery && last_practiced_at.as_deref().is_none_or(|value| {
                chrono::DateTime::parse_from_rfc3339(value)
                    .map(|date| chrono::Utc::now().signed_duration_since(date.with_timezone(&chrono::Utc)).num_days() >= 60)
                    .unwrap_or(true)
            });
            Ok(QuestionBankItem {
                id: row.get(0)?, prompt: row.get(1)?, category: row.get(2)?,
                best_answer: row.get(3)?, mastery, system_mastery: row.get(5)?,
                manual_mastery: row.get(6)?, membership_status: row.get(7)?,
                real_interview_count: row.get(8)?, asked_count: row.get(9)?,
                practice_count: row.get(10)?, reference_count: row.get(11)?,
                company_count: row.get(12)?, legacy_count: row.get(13)?,
                last_real_asked_at: row.get(14)?, last_practiced_at,
                next_review_at, created_at: row.get(17)?,
                sources: sources.unwrap_or_default().split(',').filter(|v| !v.is_empty()).map(str::to_string).collect(),
                needs_review,
            })
        },
    ).optional().map_err(db_error)?.ok_or_else(|| "题库问题不存在".to_string())
}

fn load_question_facets(connection: &Connection) -> Result<QuestionBankFacets, String> {
    connection.query_row(
        "SELECT
            SUM(CASE WHEN status='active' THEN 1 ELSE 0 END),
            SUM(CASE WHEN status='active' AND (
                (cq.next_review_at IS NOT NULL AND cq.next_review_at<=strftime('%Y-%m-%dT%H:%M:%fZ','now'))
                OR (
                    COALESCE(cq.manual_mastery,cq.system_mastery) IN ('待加强','练习中')
                    AND NOT EXISTS(
                        SELECT 1 FROM question_observations review_o
                        WHERE review_o.question_id=cq.id
                          AND review_o.event_type='mock_answered'
                          AND review_o.verification_state='confirmed'
                          AND review_o.occurred_at>strftime('%Y-%m-%dT%H:%M:%fZ','now','-60 days')
                    )
                )
            ) THEN 1 ELSE 0 END),
            (SELECT COUNT(*) FROM question_match_decisions WHERE user_action IS NULL),
            SUM(CASE WHEN status='archived' THEN 1 ELSE 0 END)
         FROM question_bank_memberships m JOIN canonical_questions cq ON cq.id=m.question_id
         WHERE cq.redirect_to_id IS NULL",
        [], |row| Ok(QuestionBankFacets {
            active: row.get::<_, Option<i64>>(0)?.unwrap_or(0),
            due: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
            pending_matches: row.get(2)?,
            archived: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
        }),
    ).map_err(db_error)
}

fn encode_cursor(offset: i64) -> String {
    URL_SAFE_NO_PAD.encode(offset.to_string())
}

fn decode_cursor(cursor: Option<&str>) -> Result<i64, String> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| "分页游标无效".to_string())?;
    let value = String::from_utf8(bytes)
        .map_err(|_| "分页游标无效".to_string())?
        .parse::<i64>()
        .map_err(|_| "分页游标无效".to_string())?;
    if value < 0 {
        return Err("分页游标无效".into());
    }
    Ok(value)
}

fn detect_language(value: &str) -> &'static str {
    if value
        .chars()
        .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character))
    {
        "zh"
    } else {
        "en"
    }
}

fn ordered_pair<'a>(left: &'a str, right: &'a str) -> (&'a str, &'a str) {
    if left < right {
        (left, right)
    } else {
        (right, left)
    }
}

fn collect_ids(connection: &Connection, sql: &str, id: &str) -> Result<Vec<String>, String> {
    connection
        .prepare(sql)
        .map_err(db_error)?
        .query_map([id], |row| row.get(0))
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)
}

fn json_ids(value: &serde_json::Value, key: &str) -> Result<Vec<String>, String> {
    value[key]
        .as_array()
        .ok_or_else(|| "合并审计快照损坏".to_string())?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| "合并审计快照损坏".to_string())
        })
        .collect()
}

fn question_similarity(left: &str, right: &str) -> (f64, String, bool) {
    let left_normalized = normalize_question(left);
    let right_normalized = normalize_question(right);
    if left_normalized == right_normalized {
        return (1.0, "标准化文本一致".into(), false);
    }
    let conflict_terms = [
        ("不支持", "支持"),
        ("不是", "是"),
        ("kafka", "rabbitmq"),
        ("是什么", "如何"),
        ("哪些", "如何配置"),
        ("比较", "介绍"),
    ];
    for (first, second) in conflict_terms {
        if (left_normalized.contains(first) && right_normalized.contains(second))
            || (left_normalized.contains(second) && right_normalized.contains(first))
        {
            return (0.45, format!("“{first}”与“{second}”的意图或实体冲突"), true);
        }
    }
    let left_grams = ngrams(&left_normalized);
    let right_grams = ngrams(&right_normalized);
    let intersection = left_grams.intersection(&right_grams).count() as f64;
    let union = left_grams.union(&right_grams).count().max(1) as f64;
    let score = intersection / union;
    (
        score,
        format!("本地字符二元组相似度 {:.0}%", score * 100.0),
        false,
    )
}

fn ngrams(value: &str) -> std::collections::HashSet<String> {
    let chars = value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<Vec<_>>();
    if chars.len() < 2 {
        return [value.to_string()].into_iter().collect();
    }
    chars.windows(2).map(|pair| pair.iter().collect()).collect()
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

fn mastery_from_scores(scores: &[i64]) -> &'static str {
    if scores.is_empty() {
        return "待加强";
    }
    let weights = [3_i64, 2, 1];
    let weighted_total: i64 = scores
        .iter()
        .zip(weights)
        .map(|(score, weight)| score * weight)
        .sum();
    let weight_total: i64 = weights.into_iter().take(scores.len()).sum();
    match weighted_total / weight_total {
        0..=59 => "待加强",
        60..=74 => "练习中",
        75..=89 => "熟悉",
        _ if scores.len() >= 2 => "掌握",
        _ => "熟悉",
    }
}

fn format_duration(seconds: i64) -> String {
    let minutes = (seconds.max(1) + 59) / 60;
    format!("{minutes} 分钟")
}

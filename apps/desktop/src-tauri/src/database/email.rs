use super::{
    clean_timestamp, create_application_record, db_error, query_tasks, required,
    validate_task_times, ApplicationListItem, ApplicationTask, CreateApplicationInput, Database,
};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RawEmail {
    pub account: String,
    pub mailbox: String,
    pub uid: u32,
    pub message_id: Option<String>,
    pub sender: String,
    pub subject: String,
    pub received_at: String,
    pub body_text: String,
    pub links: Vec<EmailLink>,
}

#[derive(Debug, Clone)]
pub struct EmailSyncFailure {
    pub uid: u32,
    pub reason: String,
    pub permanently_skipped: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct EmailSyncCursor {
    pub last_uid: u32,
    pub uid_validity: Option<u32>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailMessage {
    pub id: String,
    pub sender: String,
    pub subject: String,
    pub received_at: String,
    pub snippet: String,
    pub body_text: String,
    pub links: Vec<EmailLink>,
    pub category: String,
    pub suggested_stage: Option<String>,
    pub status: String,
    pub matched_application_id: Option<String>,
    pub company: Option<String>,
    pub role: Option<String>,
    pub current_stage: Option<String>,
    pub confidence: i64,
    pub reasons: Vec<String>,
    pub calendar_task_created: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailStats {
    pub this_week: i64,
    pub pending: i64,
    pub confirmed: i64,
    pub unmatched: i64,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub fetched: usize,
    pub recognized: usize,
    pub matched: usize,
}

struct Classification {
    category: &'static str,
    stage: Option<&'static str>,
}

struct MatchCandidate {
    id: String,
    score: i64,
    reasons: Vec<String>,
    safe_to_attach: bool,
}

type EmailConfirmationData = (
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    String,
);

impl Database {
    pub fn email_sync_cursor(&self, account: &str) -> Result<EmailSyncCursor, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let value: Option<(i64, Option<i64>)> = connection
            .query_row(
                "SELECT last_uid,uid_validity FROM email_sync_state WHERE account=?1 AND mailbox='INBOX'",
                [account],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (last_uid, uid_validity) = value.unwrap_or((0, None));
        Ok(EmailSyncCursor {
            last_uid: last_uid.max(0) as u32,
            uid_validity: uid_validity.map(|value| value.max(0) as u32),
        })
    }

    #[cfg(test)]
    pub fn latest_email_uid(&self, account: &str) -> Result<u32, String> {
        self.email_sync_cursor(account)
            .map(|cursor| cursor.last_uid)
    }

    pub fn retryable_email_uids(&self, account: &str) -> Result<Vec<u32>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT uid FROM email_sync_failures
                 WHERE account=?1 AND mailbox='INBOX' AND permanently_skipped=0
                 ORDER BY last_attempt_at LIMIT 20",
            )
            .map_err(db_error)?;
        let rows = statement
            .query_map([account], |row| row.get::<_, i64>(0))
            .map_err(db_error)?;
        rows.map(|row| row.map(|uid| uid.max(0) as u32).map_err(db_error))
            .collect()
    }

    #[cfg(test)]
    pub fn ingest_emails(&self, messages: Vec<RawEmail>) -> Result<SyncResult, String> {
        let fetched = messages.len();
        let cursor = messages
            .iter()
            .map(|item| (item.account.clone(), item.uid))
            .max_by_key(|(_, uid)| *uid);
        self.ingest_emails_with_cursor(messages, cursor, None, None, Vec::new(), fetched)
    }

    pub fn ingest_emails_through(
        &self,
        messages: Vec<RawEmail>,
        account: &str,
        highest_uid: Option<u32>,
        uid_validity: Option<u32>,
        failures: Vec<EmailSyncFailure>,
        scanned: usize,
    ) -> Result<SyncResult, String> {
        let cursor = highest_uid.map(|uid| (account.to_string(), uid));
        self.ingest_emails_with_cursor(
            messages,
            cursor,
            Some(account),
            uid_validity,
            failures,
            scanned,
        )
    }

    fn ingest_emails_with_cursor(
        &self,
        messages: Vec<RawEmail>,
        cursor: Option<(String, u32)>,
        failure_account: Option<&str>,
        uid_validity: Option<u32>,
        failures: Vec<EmailSyncFailure>,
        fetched: usize,
    ) -> Result<SyncResult, String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let mut result = SyncResult {
            fetched,
            recognized: 0,
            matched: 0,
        };
        if let (Some(account), Some(uid_validity)) = (failure_account, uid_validity) {
            transaction
                .execute(
                    "DELETE FROM email_sync_failures
                     WHERE account=?1
                       AND EXISTS(
                         SELECT 1 FROM email_sync_state
                         WHERE account=?1 AND mailbox='INBOX'
                           AND uid_validity IS NOT NULL AND uid_validity<>?2
                       )",
                    params![account, uid_validity],
                )
                .map_err(db_error)?;
        }

        for message in messages {
            transaction
                .execute(
                    "DELETE FROM email_sync_failures
                     WHERE account=?1 AND mailbox='INBOX' AND uid=?2",
                    params![message.account, message.uid],
                )
                .map_err(db_error)?;
            let combined = format!(
                "{}\n{}",
                message.subject,
                primary_email_content(&message.body_text)
            );
            let Some(classification) = classify(&message.subject, &message.body_text) else {
                continue;
            };
            result.recognized += 1;
            let candidate = best_match(&transaction, &combined, &message.received_at)?;
            if candidate.as_ref().is_some_and(|item| item.safe_to_attach) {
                result.matched += 1;
            }
            let matched = candidate.as_ref().filter(|item| item.safe_to_attach);
            let confidence = candidate
                .as_ref()
                .map(|item| item.score.clamp(0, 100))
                .unwrap_or(0);
            let reasons = candidate
                .as_ref()
                .map(|item| item.reasons.clone())
                .unwrap_or_else(|| vec!["未找到公司或岗位信息足够接近的投递".into()]);
            let snippet = compact(&message.body_text, 180);
            transaction.execute(
                "INSERT INTO email_messages(id,account,mailbox,uid,message_id,sender,subject,received_at,body_text,snippet,category,suggested_stage,status,matched_application_id,confidence,reasons_json,links_json)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)
                 ON CONFLICT(account,mailbox,uid) DO UPDATE SET
                    message_id=excluded.message_id,sender=excluded.sender,subject=excluded.subject,
                    received_at=excluded.received_at,body_text=excluded.body_text,snippet=excluded.snippet,
                    links_json=excluded.links_json,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                params![Uuid::new_v4().to_string(), message.account, message.mailbox, message.uid, message.message_id,
                    message.sender, message.subject, message.received_at, message.body_text, snippet,
                    classification.category, classification.stage,
                    if matched.is_some() { "pending" } else { "unmatched" }, matched.map(|item| item.id.as_str()), confidence,
                    serde_json::to_string(&reasons).map_err(|error| error.to_string())?,
                    serde_json::to_string(&message.links).map_err(|error| error.to_string())?],
            ).map_err(db_error)?;
        }
        for failure in failures {
            let account = failure_account.unwrap_or_default();
            transaction
                .execute(
                    "INSERT INTO email_sync_failures(account,mailbox,uid,reason,permanently_skipped)
                     VALUES (?1,'INBOX',?2,?3,?4)
                     ON CONFLICT(account,mailbox,uid) DO UPDATE SET
                       reason=excluded.reason,retry_count=retry_count+1,
                       permanently_skipped=excluded.permanently_skipped,
                       last_attempt_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                    params![
                        account,
                        failure.uid,
                        failure.reason,
                        failure.permanently_skipped
                    ],
                )
                .map_err(db_error)?;
        }
        if let Some((account, uid)) = cursor {
            transaction.execute(
                "INSERT INTO email_sync_state(account,mailbox,last_uid,uid_validity) VALUES (?1,'INBOX',?2,?3)
                 ON CONFLICT(account,mailbox) DO UPDATE SET
                   last_uid=CASE
                     WHEN email_sync_state.uid_validity IS NOT excluded.uid_validity THEN excluded.last_uid
                     ELSE MAX(email_sync_state.last_uid,excluded.last_uid)
                   END,
                   uid_validity=excluded.uid_validity,
                   updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                params![account, uid, uid_validity],
            ).map_err(db_error)?;
        }
        transaction.commit().map_err(db_error)?;
        Ok(result)
    }

    pub fn list_email_messages(&self) -> Result<Vec<EmailMessage>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection.prepare(
            "SELECT e.id,e.sender,e.subject,e.received_at,e.snippet,e.body_text,e.links_json,e.category,e.suggested_stage,e.status,e.matched_application_id,
                    c.name,p.title,a.current_stage,e.confidence,e.reasons_json,
                    EXISTS(SELECT 1 FROM tasks t WHERE t.source_type='email' AND t.source_id=e.id AND t.deleted_at IS NULL)
             FROM email_messages e
             LEFT JOIN applications a ON a.id=e.matched_application_id
             LEFT JOIN positions p ON p.id=a.position_id
             LEFT JOIN companies c ON c.id=p.company_id
             ORDER BY e.received_at DESC, e.created_at DESC LIMIT 500"
        ).map_err(db_error)?;
        let rows = statement
            .query_map([], |row| {
                let links_json: String = row.get(6)?;
                let reasons_json: String = row.get(15)?;
                let sender: String = row.get(1)?;
                let subject: String = row.get(2)?;
                let body_text: String = row.get(5)?;
                let matched_company: Option<String> = row.get(11)?;
                let matched_role: Option<String> = row.get(12)?;
                Ok(EmailMessage {
                    id: row.get(0)?,
                    sender: sender.clone(),
                    subject: subject.clone(),
                    received_at: row.get(3)?,
                    snippet: row.get(4)?,
                    body_text: body_text.clone(),
                    links: serde_json::from_str(&links_json).unwrap_or_default(),
                    category: row.get(7)?,
                    suggested_stage: row.get(8)?,
                    status: row.get(9)?,
                    matched_application_id: row.get(10)?,
                    company: matched_company
                        .or_else(|| extract_company(&sender, &subject, &body_text)),
                    role: matched_role.or_else(|| extract_role(&subject, &body_text)),
                    current_stage: row.get(13)?,
                    confidence: row.get(14)?,
                    reasons: serde_json::from_str(&reasons_json).unwrap_or_default(),
                    calendar_task_created: row.get(16)?,
                })
            })
            .map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn email_stats(&self) -> Result<EmailStats, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection
            .query_row(
                "SELECT SUM(received_at >= strftime('%Y-%m-%dT%H:%M:%fZ','now','-7 days')),
                    SUM(status='pending'), SUM(status='confirmed'), SUM(status='unmatched'),
                    (SELECT MAX(updated_at) FROM email_sync_state)
             FROM email_messages",
                [],
                |row| {
                    Ok(EmailStats {
                        this_week: row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                        pending: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                        confirmed: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                        unmatched: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                        last_synced_at: row.get(4)?,
                    })
                },
            )
            .map_err(db_error)
    }

    pub fn ignore_email(&self, id: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection.execute("UPDATE email_messages SET status='ignored',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status!='confirmed'", [id]).map_err(db_error)?;
        if changed == 0 {
            return Err("邮件不存在或已更新流程".into());
        }
        Ok(())
    }

    pub fn rematch_email(&self, id: &str) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (subject, body, received, status): (String, String, String, String) = transaction
            .query_row(
                "SELECT subject,body_text,received_at,status FROM email_messages WHERE id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "邮件不存在".to_string())?;
        if status == "confirmed" {
            let event: Option<(String, Option<String>, Option<String>)> = transaction
                .query_row(
                    "SELECT id,stage_after,reverted_at FROM application_events
                     WHERE source_type='email' AND source_id=?1
                     ORDER BY created_at DESC LIMIT 1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(db_error)?;
            match event {
                Some((_, Some(_), None)) => {
                    return Err(
                        "该邮件已经更新投递阶段，请先在投递详情的时间线中撤销对应邮件事件，再重新识别"
                            .into(),
                    );
                }
                Some((event_id, None, None)) => {
                    // 该确认事件只记录了邮件，没有改变投递阶段。重新识别不会破坏
                    // 阶段历史，因此直接将旧事件标为已撤销，避免再次确认后留下两条有效记录。
                    transaction
                        .execute(
                            "UPDATE application_events
                             SET reverted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                             WHERE id=?1",
                            [event_id],
                        )
                        .map_err(db_error)?;
                }
                Some((_, _, Some(_))) => {}
                None => return Err("该邮件缺少对应的投递事件，无法安全地重新识别".into()),
            }
        }
        // 已进入本地招聘邮件索引的记录，即使新规则无法判断具体阶段，也保留为
        // “待人工判断”，继续匹配投递，但绝不建议修改投递阶段。
        let classification = classify_existing(&subject, &body);
        let text = format!("{subject}\n{}", primary_email_content(&body));
        let candidate = best_match(&transaction, &text, &received)?;
        let matched = candidate.as_ref().filter(|item| item.safe_to_attach);
        let reasons = candidate
            .as_ref()
            .map(|item| item.reasons.clone())
            .unwrap_or_else(|| vec!["未找到公司或岗位信息足够接近的投递".into()]);
        transaction.execute(
            "UPDATE email_messages SET category=?2,suggested_stage=?3,matched_application_id=?4,confidence=?5,reasons_json=?6,status=?7,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            params![id, classification.category, classification.stage, matched.map(|item| item.id.as_str()), candidate.as_ref().map(|item| item.score.clamp(0,100)).unwrap_or(0), serde_json::to_string(&reasons).map_err(|error| error.to_string())?, if matched.is_some() { "pending" } else { "unmatched" }],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)
    }

    pub fn attach_email_to_application(
        &self,
        email_id: &str,
        application_id: &str,
    ) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        attach_email_to_application_in_transaction(&transaction, email_id, application_id)?;
        transaction.commit().map_err(db_error)
    }

    pub fn review_email(
        &self,
        email_id: &str,
        application_id: &str,
        category: &str,
        suggested_stage: Option<&str>,
    ) -> Result<(), String> {
        const CATEGORIES: &[&str] = &[
            "投递反馈 · 投递成功",
            "测评邀请",
            "笔试邀请",
            "面试邀请",
            "结果通知 · 进入下一轮",
            "结果通知 · 流程进展",
            "结果通知 · Offer",
            "结果通知 · 未通过",
            "HR 沟通",
            "招聘邮件",
            "待人工判断",
        ];
        const STAGES: &[&str] = &[
            "已投递",
            "等待结果",
            "在线测评",
            "笔试",
            "面试中",
            "HR 面试",
            "已获Offer",
            "已拒绝",
            "进入人才库",
            "主动放弃",
        ];
        let category = category.trim();
        if !CATEGORIES.contains(&category) {
            return Err("邮件分类无效".into());
        }
        let suggested_stage = suggested_stage
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if suggested_stage.is_some_and(|stage| !STAGES.contains(&stage)) {
            return Err("建议阶段无效".into());
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        attach_email_to_application_in_transaction(&transaction, email_id, application_id)?;
        let mut review_reasons = vec!["用户手动选择了这条投递", "用户手动指定邮件分类"];
        if suggested_stage.is_some() {
            review_reasons.push("用户手动指定邮件阶段");
        }
        transaction
            .execute(
                "UPDATE email_messages
                 SET category=?2,suggested_stage=?3,reasons_json=?4,status='pending',
                     updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id=?1",
                params![
                    email_id,
                    category,
                    suggested_stage,
                    serde_json::to_string(&review_reasons).map_err(|error| error.to_string())?
                ],
            )
            .map_err(db_error)?;
        transaction.commit().map_err(db_error)
    }

    pub fn confirm_email_match(&self, id: &str) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        confirm_email_match_in_transaction(&transaction, id)?;
        transaction.commit().map_err(db_error)
    }

    pub fn create_email_calendar_task(
        &self,
        email_id: &str,
        title: String,
        due_at: String,
        remind_at: Option<String>,
    ) -> Result<ApplicationTask, String> {
        let title = required(title, "任务标题")?;
        let due_at = clean_timestamp(Some(due_at), "日历时间")?
            .ok_or_else(|| "日历时间不能为空".to_string())?;
        let remind_at = clean_timestamp(remind_at, "提醒时间")?;
        validate_task_times(Some(&due_at), remind_at.as_deref())?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (status, application_id, subject): (String, Option<String>, String) = transaction
            .query_row(
                "SELECT status,matched_application_id,subject FROM email_messages WHERE id=?1",
                [email_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "邮件不存在".to_string())?;
        if status != "confirmed" {
            return Err("请先确认邮件关联，再创建日历任务".into());
        }
        let application_id = application_id.ok_or_else(|| "邮件尚未关联到投递".to_string())?;
        let existing: bool = transaction
            .query_row(
                "SELECT EXISTS(
                   SELECT 1 FROM tasks
                   WHERE source_type='email' AND source_id=?1 AND deleted_at IS NULL
                 )",
                [email_id],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        if existing {
            return Err("这封邮件已经创建过日历任务".into());
        }
        let task_id = Uuid::new_v4().to_string();
        transaction
            .execute(
                "INSERT INTO tasks(
                   id,application_id,title,description,priority,due_at,remind_at,
                   application_stage,source_type,source_id
                 ) VALUES (?1,?2,?3,?4,2,?5,?6,NULL,'email',?7)",
                params![
                    task_id,
                    application_id,
                    title,
                    format!("来自招聘邮件：{subject}"),
                    due_at,
                    remind_at,
                    email_id
                ],
            )
            .map_err(db_error)?;
        transaction
            .execute(
                "INSERT INTO application_events(
                   id,application_id,event_type,title,content,source_type,source_id
                 ) VALUES (?1,?2,'task_created','从邮件创建日历任务',?3,'email',?4)",
                params![Uuid::new_v4().to_string(), application_id, title, task_id],
            )
            .map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        drop(connection);
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        query_tasks(&connection, &application_id)?
            .into_iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| "创建日历任务后无法读取记录".to_string())
    }

    pub fn create_application_from_email(
        &self,
        email_id: &str,
        input: CreateApplicationInput,
    ) -> Result<ApplicationListItem, String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let received_at: String = transaction
            .query_row(
                "SELECT received_at FROM email_messages WHERE id=?1",
                [email_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "邮件不存在".to_string())?;
        if let Some(applied_at) = input.applied_at.as_deref() {
            let later: bool = transaction
                .query_row(
                    "SELECT date(?1) > date(?2)",
                    params![applied_at, received_at],
                    |row| row.get(0),
                )
                .map_err(db_error)?;
            if later {
                return Err("投递日期不能晚于这封招聘邮件的接收日期".into());
            }
        }
        let application_id = create_application_record(&transaction, input)?;
        attach_email_to_application_in_transaction(&transaction, email_id, &application_id)?;
        confirm_email_match_in_transaction(&transaction, email_id)?;
        transaction.commit().map_err(db_error)?;
        drop(connection);
        self.get_application(&application_id)?
            .ok_or_else(|| "从邮件创建投递后无法读取记录".to_string())
    }
}

fn attach_email_to_application_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    email_id: &str,
    application_id: &str,
) -> Result<(), String> {
    let email_status: String = transaction
        .query_row(
            "SELECT status FROM email_messages WHERE id=?1",
            [email_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)?
        .ok_or_else(|| "邮件不存在".to_string())?;
    if email_status == "confirmed" {
        return Err("该邮件已经写入投递流程".into());
    }
    let application_exists: bool = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM applications WHERE id=?1 AND deleted_at IS NULL)",
            [application_id],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if !application_exists {
        return Err("选择的投递不存在或已经删除".into());
    }
    transaction
        .execute(
            "UPDATE email_messages
             SET matched_application_id=?2,confidence=0,reasons_json=?3,status='pending',
                 updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id=?1",
            params![
                email_id,
                application_id,
                serde_json::to_string(&vec!["用户手动选择了这条投递"])
                    .map_err(|error| error.to_string())?
            ],
        )
        .map_err(db_error)?;
    Ok(())
}

fn confirm_email_match_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    id: &str,
) -> Result<(), String> {
    let data: Option<EmailConfirmationData> = transaction.query_row(
            "SELECT e.status,e.matched_application_id,e.category,e.subject,e.received_at,e.suggested_stage,e.reasons_json FROM email_messages e WHERE e.id=?1",
            [id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?)),
        ).optional().map_err(db_error)?;
    let (status, application_id, category, subject, received_at, suggested_stage, reasons_json) =
        data.ok_or_else(|| "邮件不存在".to_string())?;
    if status == "confirmed" {
        return Ok(());
    }
    let application_exists: bool = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM applications WHERE id=?1 AND deleted_at IS NULL)",
            [&application_id],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if !application_exists {
        return Err("匹配的投递不存在".to_string());
    }
    // 邮件的状态判断以邮件到达时的历史阶段为准，而不是以用户确认邮件时的
    // 当前阶段为准。这样晚确认旧邮件时，事件仍能插入正确的时间位置。
    let stage_at_email_time: String = transaction
        .query_row(
            "SELECT COALESCE(
                (SELECT stage_after FROM application_events
                 WHERE application_id=?1 AND stage_after IS NOT NULL AND reverted_at IS NULL
                   AND julianday(happened_at) <= julianday(?2)
                 ORDER BY happened_at DESC,created_at DESC,rowid DESC LIMIT 1),
                '已投递')",
            params![application_id, received_at],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    // 只有用户在审核表单中明确指定阶段时才允许覆盖阶段保护。
    // 单纯手动关联投递不应让自动建议回退已有流程。
    let manual_stage_override = serde_json::from_str::<Vec<String>>(&reasons_json)
        .unwrap_or_default()
        .iter()
        .any(|reason| reason == "用户手动指定邮件阶段");
    let next_stage = suggested_stage.filter(|stage| {
        if manual_stage_override {
            stage != &stage_at_email_time
        } else {
            should_advance(&stage_at_email_time, stage)
        }
    });
    transaction.execute(
            "INSERT INTO application_events(id,application_id,event_type,title,content,source_type,source_id,stage_before,stage_after,happened_at,reversible)
             VALUES (?1,?2,'email_status',?3,?4,'email',?5,?6,?7,?8,?9)",
            params![Uuid::new_v4().to_string(), application_id, category, subject, id, stage_at_email_time, next_stage.as_deref(), received_at, next_stage.is_some()],
        ).map_err(db_error)?;
    // 插入可能早于当前流程的邮件事件后，按完整有效时间线重新计算当前阶段。
    // 因此旧邮件使用原始接收时间，但不会覆盖时间上更晚的人工或邮件节点。
    let effective: Option<(String, String)> = transaction
        .query_row(
            "SELECT stage_after,happened_at FROM application_events
                 WHERE application_id=?1 AND stage_after IS NOT NULL AND reverted_at IS NULL
                 ORDER BY happened_at DESC,created_at DESC,rowid DESC LIMIT 1",
            [&application_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(db_error)?;
    let (effective_stage, effective_time) =
        effective.unwrap_or_else(|| ("已投递".to_string(), received_at.clone()));
    transaction
        .execute(
            "UPDATE applications
             SET current_stage=?2,status_updated_at=?3,
                 updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id=?1",
            params![application_id, effective_stage, effective_time],
        )
        .map_err(db_error)?;
    transaction.execute("UPDATE email_messages SET status='confirmed',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1", [id]).map_err(db_error)?;
    Ok(())
}

fn best_match(
    connection: &rusqlite::Connection,
    text: &str,
    received_at: &str,
) -> Result<Option<MatchCandidate>, String> {
    let normalized = normalize(text);
    let mut statement = connection.prepare(
        "SELECT a.id,c.name,p.title,COALESCE(p.job_code,''),COALESCE(a.applied_at,a.created_at)
         FROM applications a JOIN positions p ON p.id=a.position_id JOIN companies c ON c.id=p.company_id
         WHERE a.deleted_at IS NULL AND a.archived_at IS NULL AND p.deleted_at IS NULL AND c.deleted_at IS NULL"
    ).map_err(db_error)?;
    let candidates = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(db_error)?;
    let mut ranked = Vec::new();
    for row in candidates {
        let (id, company, role, job_code, applied_at) = row.map_err(db_error)?;
        let mut score = 0;
        let mut reasons = Vec::new();
        let mut company_hit = false;
        let mut role_hit = false;
        let mut code_hit = false;
        if contains_meaningful(&normalized, &normalize(&company)) {
            score += 50;
            company_hit = true;
            reasons.push(format!("邮件中出现公司名称“{company}”"));
        }
        if contains_meaningful(&normalized, &normalize(&role)) {
            score += 30;
            role_hit = true;
            reasons.push(format!("邮件中出现岗位名称“{role}”"));
        } else {
            let overlap = keyword_overlap(&normalized, &normalize(&role));
            if overlap >= 2 {
                score += 18;
                role_hit = true;
                reasons.push("岗位关键词与已投岗位相符".into());
            }
        }
        if !job_code.trim().is_empty() && normalized.contains(&normalize(&job_code)) {
            score += 25;
            code_hit = true;
            reasons.push(format!("岗位编号 {job_code} 一致"));
        }
        if received_at >= applied_at.as_str() {
            score += 8;
            reasons.push("邮件时间晚于投递时间".into());
        }
        ranked.push((id, score, reasons, company_hit, role_hit, code_hit));
    }
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let Some((id, score, mut reasons, company_hit, role_hit, code_hit)) = ranked.first().cloned()
    else {
        return Ok(None);
    };
    let margin = ranked
        .get(1)
        .map(|runner_up| score - runner_up.1)
        .unwrap_or(score);
    let evidence_is_specific = role_hit || code_hit;
    let safe_to_attach = score >= 70 && margin >= 20 && evidence_is_specific;
    if !evidence_is_specific && company_hit {
        reasons.push("仅命中公司，无法区分同公司下的不同岗位".into());
    }
    if score < 70 {
        reasons.push("匹配分不足 70，需要人工选择投递".into());
    } else if margin < 20 {
        reasons.push(format!("前两名仅相差 {margin} 分，存在岗位歧义"));
    }
    Ok(Some(MatchCandidate {
        id,
        score,
        reasons,
        safe_to_attach,
    }))
}

fn classify(subject: &str, body: &str) -> Option<Classification> {
    let subject = subject.to_lowercase();
    let body = primary_email_content(body).to_lowercase();
    let value = format!("{subject}\n{body}");
    let has = |words: &[&str]| words.iter().any(|word| value.contains(word));
    let subject_has = |words: &[&str]| words.iter().any(|word| subject.contains(word));
    if has(&[
        "验证码",
        "verification code",
        "账单",
        "newsletter",
        "广告退订",
    ]) {
        return None;
    }
    let rejection_words = &[
        "很遗憾",
        "遗憾地通知",
        "不得不遗憾",
        "未能通过",
        "不予录用",
        "投递流程已结束",
        "申请流程已结束",
        "应聘流程已结束",
        "招聘流程已结束",
        "rejected",
        "not move forward",
        "other candidates",
    ];
    if rejection_words
        .iter()
        .any(|word| contains_unconditional_phrase(&value, word))
    {
        return Some(Classification {
            category: "结果通知 · 未通过",
            stage: Some("已拒绝"),
        });
    }
    let strong_offer = subject_has(&[
        "录用通知",
        "录取通知",
        "入职邀请",
        "正式录用",
        "聘用通知",
        "offer letter",
        "job offer",
        "employment offer",
        "正式offer",
        "正式 offer",
        "offer通知",
        "offer 通知",
    ]) || has(&[
        "we are pleased to offer you",
        "很高兴向您发出录用",
        "向您发放offer",
        "向你发放offer",
        "发放 offer",
        "发放offer",
        "接受 offer",
        "接受offer",
    ]);
    if strong_offer || contains_ascii_word(&subject, "offer") {
        return Some(Classification {
            category: "结果通知 · Offer",
            stage: Some("已获Offer"),
        });
    }
    if has(&[
        "面试通过",
        "通过本轮",
        "通过了本轮",
        "进入下一轮",
        "下一轮面试",
        "next round interview",
    ]) {
        return Some(Classification {
            category: "结果通知 · 进入下一轮",
            stage: Some("面试中"),
        });
    }
    // “面试结果及后续安排”必须归为结果，而不是因为包含“面试”误判为邀请。
    if has(&[
        "结果通知",
        "面试结果",
        "测评结果",
        "笔试结果",
        "后续安排",
        "application status",
        "interview result",
        "assessment result",
        "under review",
    ]) {
        return Some(Classification {
            category: "结果通知 · 流程进展",
            stage: Some("等待结果"),
        });
    }
    // 明确的下一步行动优先于正文页脚中的招聘宣传等宽泛词。
    if subject_has(&["在线测评", "测评", "assessment", "测评邀请", "测评提醒"])
        || has(&[
            "测评链接",
            "assessment link",
            "complete your assessment",
            "完成在线测评",
            "参加在线测评",
            "邀请您参加在线测评",
        ])
    {
        return Some(Classification {
            category: "测评邀请",
            stage: Some("在线测评"),
        });
    }
    if subject_has(&["笔试", "机试", "coding test", "written test"])
        || has(&[
            "笔试链接",
            "机试链接",
            "参加笔试",
            "参加机试",
            "完成笔试",
            "完成机试",
            "邀请您参加笔试",
            "邀请您参加机试",
        ])
    {
        return Some(Classification {
            category: "笔试邀请",
            stage: Some("笔试"),
        });
    }
    if subject_has(&[
        "面试",
        "interview",
        "视频沟通",
        "面谈",
        "技术沟通",
        "hr沟通",
        "沟通邀请",
    ]) || has(&[
        "邀请您参加面试",
        "邀请你参加面试",
        "诚邀您参加面试",
        "诚邀你参加面试",
        "诚邀参加面试",
        "invite you to interview",
        "interview invitation",
    ]) {
        return Some(Classification {
            category: "面试邀请",
            stage: Some("面试中"),
        });
    }
    if has(&["通过筛选", "等待结果", "简历筛选中"]) {
        return Some(Classification {
            category: "结果通知 · 流程进展",
            stage: Some("等待结果"),
        });
    }
    if has(&[
        "投递成功",
        "简历投递成功",
        "申请成功",
        "收到您的简历",
        "已收到您的申请",
        "我们已收到您的申请",
        "application received",
        "application submitted",
        "thank you for applying",
        "we received your application",
        "申请已提交",
    ]) {
        return Some(Classification {
            category: "投递反馈 · 投递成功",
            stage: Some("已投递"),
        });
    }
    if has(&[
        "招聘",
        "应聘",
        "岗位申请",
        "求职",
        "recruiting",
        "recruitment",
        "application",
    ]) {
        return Some(Classification {
            category: "招聘邮件",
            stage: None,
        });
    }
    None
}

fn classify_existing(subject: &str, body: &str) -> Classification {
    classify(subject, body).unwrap_or(Classification {
        category: "待人工判断",
        stage: None,
    })
}

fn primary_email_content(body: &str) -> String {
    const QUOTE_MARKERS: &[&str] = &[
        "-----original message-----",
        "----- 原始邮件 -----",
        "发件人:",
        "from:",
    ];
    let body_lines = body.lines().collect::<Vec<_>>();
    let mut lines = Vec::new();
    for (index, line) in body_lines.iter().enumerate() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        let quoted_block = trimmed.starts_with('>')
            && (body_lines
                .iter()
                .skip(index + 1)
                .find(|next| !next.trim().is_empty())
                .is_some_and(|next| next.trim().starts_with('>'))
                || ["> from:", "> 发件人:", "> on "]
                    .iter()
                    .any(|marker| lower.starts_with(marker)));
        let reply_marker = QUOTE_MARKERS
            .iter()
            .take(2)
            .any(|marker| lower.starts_with(marker))
            || (!lines.is_empty()
                && QUOTE_MARKERS
                    .iter()
                    .skip(2)
                    .any(|marker| lower.starts_with(marker)));
        if quoted_block
            || reply_marker
            || (lower.starts_with("on ") && lower.ends_with(" wrote:"))
            || (trimmed.starts_with("在 ") && trimmed.ends_with("写道："))
        {
            break;
        }
        if matches!(
            lower.as_str(),
            "此致" | "best regards" | "kind regards" | "sent from my iphone"
        ) {
            break;
        }
        lines.push(*line);
    }
    lines.join("\n")
}

fn contains_unconditional_phrase(value: &str, phrase: &str) -> bool {
    value.match_indices(phrase).any(|(index, _)| {
        let prefix = value[..index]
            .chars()
            .rev()
            .take(24)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        let clause = prefix
            .rsplit(['。', '！', '？', ';', '；', '\n'])
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        ![
            "如果", "假如", "倘若", "若您", "若你", "若未", "如您", "如你", "如未", "if ",
            "unless ",
        ]
        .iter()
        .any(|marker| clause.contains(marker))
    })
}

fn contains_ascii_word(value: &str, word: &str) -> bool {
    value.match_indices(word).any(|(index, matched)| {
        let before = value[..index].chars().next_back();
        let after = value[index + matched.len()..].chars().next();
        before.is_none_or(|c| !c.is_ascii_alphanumeric())
            && after.is_none_or(|c| !c.is_ascii_alphanumeric())
    })
}

fn should_advance(current: &str, suggested: &str) -> bool {
    if current == suggested {
        return false;
    }
    if matches!(
        current,
        "已获Offer" | "已拒绝" | "进入人才库" | "流程暂停" | "流程结束" | "主动放弃"
    ) {
        return false;
    }
    // “等待结果”是面试轮次之间的可循环状态，不是高于面试的终态。
    if current.contains("等待") && (suggested.contains('面') || suggested.contains("HR")) {
        return true;
    }
    // “面试中”是泛化阶段，不能覆盖 HR 面、终面等更具体的面试轮次。
    if suggested == "面试中" && (current.contains('面') || current.contains("HR")) {
        return false;
    }
    let rank = |stage: &str| {
        if stage.contains("拒绝")
            || stage.to_ascii_lowercase().contains("offer")
            || stage.contains("人才库")
        {
            6
        } else if stage.contains("等待") {
            5
        } else if stage.contains('面') || stage.contains("HR") {
            4
        } else if stage.contains("笔试") {
            3
        } else if stage.contains("测评") {
            2
        } else if stage.contains("投递") || stage.contains("准备") {
            1
        } else {
            0
        }
    };
    rank(suggested) >= rank(current) || suggested == "已拒绝"
}

fn extract_company(sender: &str, subject: &str, body: &str) -> Option<String> {
    for (prefix, suffix) in [
        ("感谢您对", "的关注"),
        ("感谢你对", "的关注"),
        ("感谢您对", "的认可"),
        ("感谢你对", "的认可"),
    ] {
        if let Some((_, rest)) = body.split_once(prefix) {
            if let Some((company, _)) = rest.split_once(suffix) {
                let company = company.trim();
                if company.chars().count() >= 2
                    && company.chars().count() <= 40
                    && !company
                        .chars()
                        .any(|character| matches!(character, '，' | '。' | '！' | '；' | '\n'))
                {
                    return Some(company.to_string());
                }
            }
        }
    }
    let display = sender
        .split('<')
        .next()
        .unwrap_or(sender)
        .trim()
        .trim_matches(['"', '\'']);
    let mut candidate = display.to_string();
    loop {
        let before = candidate.clone();
        for suffix in [
            "人才招聘",
            "校园招聘",
            "招聘团队",
            "招聘中心",
            "校招",
            "招聘",
            "Recruitment",
            "Recruiting",
            "Careers",
            "Talent",
            "HR",
        ] {
            candidate = candidate
                .trim_end_matches(|character: char| character.is_whitespace() || character == '-')
                .trim_end_matches(suffix)
                .trim()
                .to_string();
        }
        if candidate == before {
            break;
        }
    }
    let normalized = candidate.to_ascii_lowercase();
    let generic = [
        "",
        "no-reply",
        "noreply",
        "jobs",
        "job",
        "campus",
        "recruiting",
        "recruitment",
        "talent",
        "hr",
    ];
    if !candidate.contains('@')
        && candidate.chars().count() >= 2
        && candidate.chars().count() <= 40
        && !generic.contains(&normalized.as_str())
    {
        return Some(candidate);
    }
    if let Some((_, rest)) = subject.split_once('【') {
        if let Some((bracketed, _)) = rest.split_once('】') {
            let bracketed = bracketed.trim();
            if bracketed.chars().count() >= 2
                && bracketed.chars().count() <= 40
                && ![
                    "通知",
                    "邀请",
                    "测评",
                    "笔试",
                    "面试",
                    "验证码",
                    "工程师",
                    "开发",
                    "研发",
                    "算法",
                    "产品",
                    "运营",
                    "设计",
                    "测试",
                    "实习",
                ]
                .iter()
                .any(|word| bracketed.contains(word))
            {
                return Some(bracketed.to_string());
            }
        }
    }
    None
}

fn extract_role(subject: &str, body: &str) -> Option<String> {
    for text in [subject, body] {
        if let Some((left, _)) = text.split_once("的投递流程") {
            let candidate = ["您本次", "你本次", "本次"]
                .iter()
                .find_map(|marker| left.rsplit_once(marker).map(|(_, value)| value))
                .unwrap_or_else(|| {
                    left.rsplit(|character: char| {
                        matches!(character, '，' | '。' | '！' | '；' | '：' | '、' | '\n')
                    })
                    .find(|part| !part.trim().is_empty())
                    .unwrap_or("")
                })
                .trim_start_matches("您本次")
                .trim_start_matches("你本次")
                .trim_start_matches("本次")
                .trim();
            if looks_like_role(candidate) {
                return Some(candidate.to_string());
            }
        }
    }
    for text in [subject, body] {
        if let Some((_, rest)) = text.split_once("投递") {
            if let Some((role, _)) = rest.split_once("岗位") {
                let role = role
                    .trim_matches(|character: char| {
                        character.is_whitespace()
                            || matches!(character, '的' | '“' | '”' | '"' | ':' | '：')
                    })
                    .to_string();
                if looks_like_role(&role) {
                    return Some(role);
                }
            }
        }
    }
    let mut candidate = subject
        .split('】')
        .next_back()
        .unwrap_or(subject)
        .trim()
        .to_string();
    for suffix in [
        "面试通过及后续安排",
        "技术一面安排",
        "技术二面安排",
        "技术三面安排",
        "业务一面安排",
        "业务二面安排",
        "HR 面试安排",
        "HR面试安排",
        "一面安排",
        "二面安排",
        "三面安排",
        "终面安排",
        "视频面试安排",
        "技术面试安排",
        "面试安排通知",
        "面试安排",
        "线上面试邀请",
        "视频面试邀请",
        "面试邀请",
        "面试通知",
        "技术一面",
        "技术二面",
        "技术三面",
        "一面",
        "二面",
        "三面",
        "终面",
        "技术面",
        "业务面",
        "HR 面",
        "HR面",
        "视频沟通",
        "技术沟通",
        "在线测评邀请",
        "在线测评通知",
        "测评通知",
        "测评邀请",
        "在线测评",
        "机试邀请",
        "机试通知",
        "笔试邀请",
        "笔试通知",
        "沟通邀请",
        "沟通安排",
        "后续安排",
        "招聘流程更新",
        "招聘进展通知",
        "申请状态更新",
        "申请进展通知",
        "结果通知",
        "申请成功",
        "投递成功",
    ] {
        candidate = candidate
            .trim_end_matches(suffix)
            .trim_end_matches(|character: char| {
                character.is_whitespace()
                    || matches!(
                        character,
                        '-' | '–' | '—' | '_' | '·' | '|' | '｜' | ':' | '：' | '/' | '\\'
                    )
            })
            .to_string();
    }
    candidate = candidate
        .trim_start_matches("关于")
        .trim_start_matches("应聘")
        .trim_end_matches('的')
        .trim()
        .to_string();
    if looks_like_role(&candidate) {
        Some(candidate)
    } else {
        None
    }
}

fn looks_like_role(candidate: &str) -> bool {
    let candidate = candidate.trim();
    let length = candidate.chars().count();
    if !(2..=40).contains(&length)
        || candidate.contains(['\n', '\r', '@'])
        || candidate.to_lowercase().contains("http")
        || candidate
            .chars()
            .any(|character| matches!(character, '。' | '！' | '？' | '；' | '，'))
        || candidate.chars().all(|character| {
            character.is_ascii_digit()
                || character.is_whitespace()
                || matches!(character, '-' | '_' | '/' | '\\')
        })
    {
        return false;
    }

    let normalized = normalize(candidate);
    let noise_phrases = [
        "通知",
        "邀请",
        "面试",
        "面试通知",
        "面试邀请",
        "面试安排",
        "在线测评",
        "测评通知",
        "测评邀请",
        "笔试通知",
        "笔试邀请",
        "招聘",
        "招聘通知",
        "招聘进展",
        "招聘流程",
        "招聘流程更新",
        "应聘反馈",
        "投递反馈",
        "投递进展",
        "申请状态",
        "申请状态更新",
        "申请进展",
        "流程更新",
        "结果通知",
        "后续安排",
        "申请成功",
        "投递成功",
        "录用通知",
        "候选人中心",
        "消息提醒",
    ];
    let sentence_prefixes = [
        "您好",
        "你好",
        "恭喜",
        "感谢",
        "请您",
        "请你",
        "邀请您",
        "邀请你",
        "我们",
        "您的",
        "你的",
    ];
    let company_suffixes = [
        "有限公司",
        "股份公司",
        "集团",
        "公司",
        "科技",
        "银行",
        "证券",
        "大学",
        "学院",
        "学校",
        "医院",
    ];

    !noise_phrases
        .iter()
        .any(|noise| normalized == normalize(noise))
        && !sentence_prefixes
            .iter()
            .any(|prefix| candidate.starts_with(prefix))
        && !company_suffixes
            .iter()
            .any(|suffix| candidate.ends_with(suffix))
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
fn contains_meaningful(haystack: &str, needle: &str) -> bool {
    needle.chars().count() >= 2 && haystack.contains(needle)
}
fn keyword_overlap(text: &str, role: &str) -> usize {
    [
        "前端",
        "后端",
        "开发",
        "研发",
        "算法",
        "数据",
        "产品",
        "运营",
        "测试",
        "java",
        "python",
        "工程师",
    ]
    .iter()
    .filter(|word| role.contains(**word) && text.contains(**word))
    .count()
}
fn compact(value: &str, limit: usize) -> String {
    let clean = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = clean.chars();
    let result: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{result}…")
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify, classify_existing, extract_company, extract_role, should_advance, RawEmail,
    };
    use crate::db::{CreateApplicationInput, Database};

    #[test]
    fn recognizes_common_recruitment_stages() {
        assert_eq!(
            classify("邀请您参加在线测评", "").and_then(|item| item.stage),
            Some("在线测评")
        );
        assert_eq!(
            classify("笔试邀请", "请查收笔试链接").and_then(|item| item.stage),
            Some("笔试")
        );
        assert_eq!(
            classify("Interview invitation", "").and_then(|item| item.stage),
            Some("面试中")
        );
        assert_eq!(
            classify("恭喜，录用通知 Offer", "").and_then(|item| item.stage),
            Some("已获Offer")
        );
        assert_eq!(
            classify("很遗憾，您未能通过", "").and_then(|item| item.stage),
            Some("已拒绝")
        );
        assert_eq!(
            classify(
                "招聘结果通知",
                "很遗憾，您未能通过本轮筛选。常见问题：如果未通过，能否再次申请？"
            )
            .and_then(|item| item.stage),
            Some("已拒绝")
        );
        assert_ne!(
            classify(
                "招聘流程说明",
                "如果您未能通过筛选，我们会保留您的申请资料。"
            )
            .and_then(|item| item.stage),
            Some("已拒绝")
        );
        assert_eq!(
            classify("招聘结果", "我们决定向您发放Offer，请于三日内接受Offer。")
                .and_then(|item| item.stage),
            Some("已获Offer")
        );
        let result = classify("面试结果及后续安排", "恭喜通过本轮技术面试").unwrap();
        assert_eq!(
            (result.category, result.stage),
            ("结果通知 · 进入下一轮", Some("面试中"))
        );
        let receipt = classify(
            "申请已提交",
            "我们已收到您的申请，后续可能安排测评、笔试或面试。",
        )
        .unwrap();
        assert_eq!(
            (receipt.category, receipt.stage),
            ("投递反馈 · 投递成功", Some("已投递"))
        );
        assert_eq!(
            classify("招聘进展 - 示例科技", "请于3天内完成在线测评").and_then(|item| item.stage),
            Some("在线测评")
        );
        assert_eq!(
            classify("申请状态更新", "请按时参加笔试/机试").and_then(|item| item.stage),
            Some("笔试")
        );
        let interview = classify("招聘进展", "恭喜通过筛选，诚邀您参加面试").unwrap();
        assert_eq!(
            (interview.category, interview.stage),
            ("面试邀请", Some("面试中"))
        );
    }

    #[test]
    fn ignores_non_recruitment_security_mail() {
        assert!(classify("Your verification code 验证码 123456", "").is_none());
    }

    #[test]
    fn loreal_screening_mail_is_assessment_even_if_footer_mentions_offer() {
        let subject = "【2026欧莱雅（中国）科技青年咖】恭喜你通过第一轮筛选，请查收线上测评链接";
        let result = classify(
            subject,
            "Complete the assessment to continue. Future job offer terms may apply.",
        )
        .unwrap();
        assert_eq!(
            (result.category, result.stage),
            ("测评邀请", Some("在线测评"))
        );
    }

    #[test]
    fn extracts_company_and_role_for_unmatched_email_prefill() {
        assert_eq!(
            extract_company(
                "示例科技招聘 <jobs@example.com>",
                "后端开发工程师面试邀请",
                ""
            )
            .as_deref(),
            Some("示例科技")
        );
        assert_eq!(
            extract_role("后端开发工程师面试邀请", "").as_deref(),
            Some("后端开发工程师")
        );
        assert_eq!(
            extract_company("京东招聘Recruiting <jobs@example.com>", "招聘进展", "").as_deref(),
            Some("京东")
        );
        assert_eq!(
            extract_company("jobs@example.com", "【后端开发工程师】面试邀请", ""),
            None
        );
        assert_eq!(
            extract_role(
                "应聘反馈",
                "我们不得不遗憾地通知您，您本次 Java 开发工程师 的投递流程已结束。"
            )
            .as_deref(),
            Some("Java 开发工程师")
        );
        assert_eq!(
            extract_role("前端开发工程师技术一面", "").as_deref(),
            Some("前端开发工程师")
        );
        assert_eq!(
            extract_role("Java后端一面", "").as_deref(),
            Some("Java后端")
        );
        assert_eq!(
            extract_role("数据分析实习生在线测评邀请", "").as_deref(),
            Some("数据分析实习生")
        );
        assert_eq!(
            extract_role("全栈架构师技术沟通", "").as_deref(),
            Some("全栈架构师")
        );
        assert_eq!(
            extract_role("财务管培生一面安排", "").as_deref(),
            Some("财务管培生")
        );
        assert_eq!(
            extract_role("市场营销岗测评通知", "").as_deref(),
            Some("市场营销岗")
        );
        assert_eq!(
            extract_role("关于高级会计的面试邀请", "").as_deref(),
            Some("高级会计")
        );
        assert_eq!(
            extract_role("供应链采购专员 - 笔试通知", "").as_deref(),
            Some("供应链采购专员")
        );
        assert_eq!(
            extract_role("申请进展", "您已投递临床研究岗位，请等待后续通知。").as_deref(),
            Some("临床研究")
        );
        assert_eq!(extract_role("招聘流程更新", ""), None);
        assert_eq!(extract_role("面试邀请", ""), None);
        assert_eq!(extract_role("示例科技面试邀请", ""), None);
        assert_eq!(extract_role("恭喜您进入下一轮面试邀请", ""), None);
    }

    #[test]
    fn shein_closed_application_is_rejection_and_prefills_company_and_role() {
        let body = "程旭升，您好！非常感谢您对SHEIN的关注和支持，您优秀的履历给我们留下了深刻的印象。结合目前的招聘进展，并经过对您本次申请材料与岗位契合度的慎重评估，我们不得不遗憾地通知您，您本次 GPT实习生 的投递流程已结束。您的相关资料已录入人才库，当其他职位合适时，将优先向您发出邀请！";
        let result = classify("应聘反馈", body).unwrap();
        assert_eq!(
            (result.category, result.stage),
            ("结果通知 · 未通过", Some("已拒绝"))
        );
        assert_eq!(
            extract_company("Moka <notice@moka.example>", "应聘反馈", body).as_deref(),
            Some("SHEIN")
        );
        assert_eq!(extract_role("应聘反馈", body).as_deref(), Some("GPT实习生"));
    }

    #[test]
    fn existing_indexed_mail_without_stage_signal_is_kept_for_manual_review() {
        let result = classify_existing("来自候选人中心的消息", "请登录账户查看最新信息");
        assert_eq!((result.category, result.stage), ("待人工判断", None));
    }

    #[test]
    fn stage_guard_prevents_regression_and_final_overwrite() {
        assert!(should_advance("已投递", "面试中"));
        assert!(!should_advance("面试中", "在线测评"));
        assert!(!should_advance("已获Offer", "已拒绝"));
        assert!(should_advance("在线测评", "笔试"));
        assert!(should_advance("等待结果", "面试中"));
        assert!(!should_advance("进入人才库", "面试中"));
        assert!(!should_advance("HR 面试", "在线测评"));
        assert!(!should_advance("HR 面试", "面试中"));
        assert!(!should_advance("终面", "面试中"));
        assert!(should_advance("面试中", "HR 面试"));
        assert!(!should_advance("主动放弃", "面试中"));
    }

    #[test]
    fn quoted_rejection_does_not_override_a_new_interview_invitation() {
        let result = classify(
            "二面邀请",
            "恭喜进入下一轮，请参加二面。\n-----Original Message-----\n很遗憾，您此前申请的岗位未能通过。",
        )
        .unwrap();
        assert_eq!(
            (result.category, result.stage),
            ("结果通知 · 进入下一轮", Some("面试中"))
        );
    }

    #[test]
    fn decorative_separator_and_single_angle_line_keep_current_content() {
        let result = classify(
            "招聘流程更新",
            "________________________\n> 请在今晚之前完成在线测评\n测评链接：https://example.com",
        )
        .unwrap();
        assert_eq!(
            (result.category, result.stage),
            ("测评邀请", Some("在线测评"))
        );
    }

    #[test]
    fn manual_link_keeps_stage_guard_but_explicit_stage_review_can_override() {
        let db = Database::in_memory().unwrap();
        let application = db
            .create_application(CreateApplicationInput {
                company_name: "阶段科技".into(),
                company_short_name: None,
                industry: None,
                company_type: None,
                website: None,
                company_notes: None,
                position_title: "后端工程师".into(),
                department: None,
                location: None,
                recruitment_type: None,
                job_code: None,
                source_url: None,
                channel: Some("官网".into()),
                applied_at: Some("2026-07-01".into()),
                priority: Some(2),
                jd_raw: None,
                resume_profile_id: None,
            })
            .unwrap();
        db.update_application_stage(&application.id, "HR 面试")
            .unwrap();
        db.connection
            .lock()
            .unwrap()
            .execute(
                "UPDATE application_events
                 SET happened_at='2026-07-19T09:00:00Z'
                 WHERE application_id=?1 AND stage_after='HR 面试'",
                [&application.id],
            )
            .unwrap();
        db.ingest_emails(vec![
            RawEmail {
                account: "me@example.com".into(),
                mailbox: "INBOX".into(),
                uid: 78,
                message_id: Some("manual-link".into()),
                sender: "陌生招聘平台 <jobs@unknown.example>".into(),
                subject: "在线测评邀请".into(),
                received_at: "2026-07-20T09:00:00Z".into(),
                body_text: "请完成在线测评".into(),
                links: Vec::new(),
            },
            RawEmail {
                account: "me@example.com".into(),
                mailbox: "INBOX".into(),
                uid: 79,
                message_id: Some("manual-review".into()),
                sender: "另一招聘平台 <jobs@other.example>".into(),
                subject: "招聘流程更新".into(),
                received_at: "2026-07-21T09:00:00Z".into(),
                body_text: "请查看招聘流程更新".into(),
                links: Vec::new(),
            },
        ])
        .unwrap();
        let emails = db.list_email_messages().unwrap();
        let linked = emails
            .iter()
            .find(|email| email.subject == "在线测评邀请")
            .unwrap();
        db.attach_email_to_application(&linked.id, &application.id)
            .unwrap();
        db.confirm_email_match(&linked.id).unwrap();
        assert_eq!(
            db.get_application_detail(&application.id)
                .unwrap()
                .current_stage,
            "HR 面试"
        );

        let reviewed = emails
            .iter()
            .find(|email| email.subject == "招聘流程更新")
            .unwrap();
        db.review_email(&reviewed.id, &application.id, "测评邀请", Some("在线测评"))
            .unwrap();
        db.confirm_email_match(&reviewed.id).unwrap();
        assert_eq!(
            db.get_application_detail(&application.id)
                .unwrap()
                .current_stage,
            "在线测评"
        );
    }

    #[test]
    fn company_only_match_is_left_for_manual_selection() {
        let db = Database::in_memory().unwrap();
        let create = |role: &str| CreateApplicationInput {
            company_name: "示例科技".into(),
            company_short_name: None,
            industry: None,
            company_type: None,
            website: None,
            company_notes: None,
            position_title: role.into(),
            department: None,
            location: None,
            recruitment_type: None,
            job_code: None,
            source_url: None,
            channel: None,
            applied_at: Some("2026-07-01".into()),
            priority: Some(2),
            jd_raw: None,
            resume_profile_id: None,
        };
        db.create_application(create("后端开发")).unwrap();
        db.create_application(create("产品经理")).unwrap();
        db.ingest_emails(vec![RawEmail {
            account: "me@example.com".into(),
            mailbox: "INBOX".into(),
            uid: 77,
            message_id: None,
            sender: "示例科技招聘 <jobs@example.com>".into(),
            subject: "示例科技校园招聘进展".into(),
            received_at: "2026-07-10T09:00:00Z".into(),
            body_text: "请登录候选人中心查看最新消息".into(),
            links: Vec::new(),
        }])
        .unwrap();
        let email = db.list_email_messages().unwrap().remove(0);
        assert_eq!(email.status, "unmatched");
        assert!(email.matched_application_id.is_none());
        assert!(email
            .reasons
            .iter()
            .any(|reason| reason.contains("仅命中公司")));
    }

    #[test]
    fn recognized_email_matches_and_updates_application_after_confirmation() {
        let db = Database::in_memory().unwrap();
        let application = db
            .create_application(CreateApplicationInput {
                company_name: "示例科技".into(),
                company_short_name: None,
                industry: None,
                company_type: None,
                website: None,
                company_notes: None,
                position_title: "后端开发工程师".into(),
                department: None,
                location: Some("上海".into()),
                recruitment_type: None,
                job_code: Some("BE-2026".into()),
                source_url: None,
                channel: Some("官网".into()),
                applied_at: Some("2026-07-01T08:00:00Z".into()),
                priority: Some(2),
                jd_raw: None,
                resume_profile_id: None,
            })
            .unwrap();
        let result = db
            .ingest_emails(vec![RawEmail {
                account: "me@example.com".into(),
                mailbox: "INBOX".into(),
                uid: 10,
                message_id: Some("m10".into()),
                links: Vec::new(),
                sender: "示例科技招聘 <jobs@example.com>".into(),
                subject: "后端开发工程师面试邀请 BE-2026".into(),
                received_at: "2026-07-10T09:00:00Z".into(),
                body_text: "邀请您参加示例科技技术面试".into(),
            }])
            .unwrap();
        assert_eq!((result.recognized, result.matched), (1, 1));
        let messages = db.list_email_messages().unwrap();
        assert_eq!(messages[0].status, "pending");
        assert_eq!(
            messages[0].matched_application_id.as_deref(),
            Some(application.id.as_str())
        );
        db.confirm_email_match(&messages[0].id).unwrap();
        assert_eq!(
            db.get_application_detail(&application.id)
                .unwrap()
                .current_stage,
            "面试中"
        );
        assert_eq!(db.list_email_messages().unwrap()[0].status, "confirmed");
        let calendar = db
            .get_dashboard(
                "2026-07-01T00:00:00Z",
                "2026-08-01T00:00:00Z",
                "2026-07-10T00:00:00Z",
                "2026-07-11T00:00:00Z",
            )
            .unwrap();
        assert!(calendar.events.iter().any(|event| {
            event.application_id == application.id
                && event.title == "进入面试中"
                && event.scheduled_at == "2026-07-10T09:00:00Z"
                && event.kind == "milestone"
        }));
    }

    #[test]
    fn skipped_email_still_advances_the_sync_cursor() {
        let db = Database::in_memory().unwrap();
        let result = db
            .ingest_emails_through(
                Vec::new(),
                "me@example.com",
                Some(23),
                Some(42),
                Vec::new(),
                0,
            )
            .unwrap();
        assert_eq!(result.fetched, 0);
        assert_eq!(result.recognized, 0);
        assert_eq!(db.latest_email_uid("me@example.com").unwrap(), 23);
    }

    #[test]
    fn unmatched_email_can_be_attached_to_new_application_and_confirmed() {
        let db = Database::in_memory().unwrap();
        db.ingest_emails(vec![RawEmail {
            account: "me@example.com".into(),
            mailbox: "INBOX".into(),
            uid: 24,
            message_id: Some("m24".into()),
            links: Vec::new(),
            sender: "星云科技招聘 <jobs@nebula.example>".into(),
            subject: "后端工程师面试邀请".into(),
            received_at: "2026-07-10T09:00:00Z".into(),
            body_text: "邀请您参加线上技术面试".into(),
        }])
        .unwrap();
        let email = db.list_email_messages().unwrap().remove(0);
        assert_eq!(email.status, "unmatched");
        assert_eq!(email.company.as_deref(), Some("星云科技"));

        let application = db
            .create_application(CreateApplicationInput {
                company_name: "星云科技".into(),
                company_short_name: None,
                industry: None,
                company_type: None,
                website: None,
                company_notes: None,
                position_title: "后端工程师".into(),
                department: None,
                location: None,
                recruitment_type: None,
                job_code: None,
                source_url: None,
                channel: Some("邮件识别".into()),
                applied_at: Some("2026-07-01".into()),
                priority: Some(2),
                jd_raw: None,
                resume_profile_id: None,
            })
            .unwrap();
        db.attach_email_to_application(&email.id, &application.id)
            .unwrap();
        db.confirm_email_match(&email.id).unwrap();

        let updated = db.list_email_messages().unwrap().remove(0);
        assert_eq!(updated.status, "confirmed");
        assert_eq!(
            db.get_application_detail(&application.id)
                .unwrap()
                .current_stage,
            "面试中"
        );
        let task = db
            .create_email_calendar_task(
                &email.id,
                "技术面试".into(),
                "2026-07-20T14:00:00Z".into(),
                Some("2026-07-20T13:30:00Z".into()),
            )
            .unwrap();
        assert_eq!(task.source_type, "email");
        assert!(db
            .create_email_calendar_task(
                &email.id,
                "重复面试".into(),
                "2026-07-20T14:00:00Z".into(),
                None,
            )
            .unwrap_err()
            .contains("已经创建过"));
        let detail = db.get_application_detail(&application.id).unwrap();
        let email_event = detail
            .events
            .iter()
            .find(|event| event.source_type == "email" && event.reversible)
            .unwrap();
        db.revert_application_event(&email_event.id).unwrap();
        assert_eq!(
            db.list_email_messages().unwrap().remove(0).status,
            "pending"
        );
    }

    #[test]
    fn confirmed_email_without_stage_change_can_be_safely_rematched() {
        let db = Database::in_memory().unwrap();
        let application = db
            .create_application(CreateApplicationInput {
                company_name: "示例科技".into(),
                company_short_name: None,
                industry: None,
                company_type: None,
                website: None,
                company_notes: None,
                position_title: "后端工程师".into(),
                department: None,
                location: None,
                recruitment_type: None,
                job_code: None,
                source_url: None,
                channel: Some("官网".into()),
                applied_at: Some("2026-07-01".into()),
                priority: Some(2),
                jd_raw: None,
                resume_profile_id: None,
            })
            .unwrap();
        db.ingest_emails(vec![RawEmail {
            account: "me@example.com".into(),
            mailbox: "INBOX".into(),
            uid: 25,
            message_id: Some("m25".into()),
            links: Vec::new(),
            sender: "示例科技招聘 <jobs@example.com>".into(),
            subject: "示例科技后端工程师招聘进展".into(),
            received_at: "2026-07-10T09:00:00Z".into(),
            body_text: "请登录招聘系统查看岗位申请信息".into(),
        }])
        .unwrap();
        let email = db.list_email_messages().unwrap().remove(0);
        assert_eq!(
            email.matched_application_id.as_deref(),
            Some(application.id.as_str())
        );
        assert_eq!(email.suggested_stage, None);
        db.confirm_email_match(&email.id).unwrap();
        assert_eq!(db.list_email_messages().unwrap()[0].status, "confirmed");

        db.rematch_email(&email.id).unwrap();
        assert_eq!(db.list_email_messages().unwrap()[0].status, "pending");
    }

    #[test]
    fn old_email_uses_received_time_without_overwriting_later_stage() {
        let db = Database::in_memory().unwrap();
        let application = db
            .create_application(CreateApplicationInput {
                company_name: "时序科技".into(),
                company_short_name: None,
                industry: None,
                company_type: None,
                website: None,
                company_notes: None,
                position_title: "后端工程师".into(),
                department: None,
                location: None,
                recruitment_type: None,
                job_code: None,
                source_url: None,
                channel: Some("官网".into()),
                applied_at: Some("2026-07-01".into()),
                priority: Some(2),
                jd_raw: None,
                resume_profile_id: None,
            })
            .unwrap();
        db.update_application_stage(&application.id, "面试中")
            .unwrap();
        let stage_event_id = db
            .get_application_detail(&application.id)
            .unwrap()
            .events
            .into_iter()
            .find(|event| event.event_type == "stage_changed")
            .unwrap()
            .id;
        db.update_application_event_time(&stage_event_id, "2026-07-15T10:00:00Z")
            .unwrap();
        db.ingest_emails(vec![RawEmail {
            account: "me@example.com".into(),
            mailbox: "INBOX".into(),
            uid: 26,
            message_id: Some("m26".into()),
            links: Vec::new(),
            sender: "时序科技招聘 <jobs@example.com>".into(),
            subject: "时序科技后端工程师在线测评邀请".into(),
            received_at: "2026-07-10T09:30:00Z".into(),
            body_text: "请完成在线测评".into(),
        }])
        .unwrap();
        let email = db.list_email_messages().unwrap().remove(0);
        db.confirm_email_match(&email.id).unwrap();

        let detail = db.get_application_detail(&application.id).unwrap();
        assert_eq!(detail.current_stage, "面试中");
        let event = detail
            .events
            .iter()
            .find(|event| event.source_type == "email")
            .unwrap();
        assert_eq!(event.happened_at, "2026-07-10T09:30:00Z");
        assert_eq!(event.stage_before.as_deref(), Some("已投递"));
        assert_eq!(event.stage_after.as_deref(), Some("在线测评"));
    }

    #[test]
    fn create_from_email_rolls_back_application_when_confirmation_fails() {
        let db = Database::in_memory().unwrap();
        db.ingest_emails(vec![RawEmail {
            account: "me@example.com".into(),
            mailbox: "INBOX".into(),
            uid: 27,
            message_id: Some("m27".into()),
            links: Vec::new(),
            sender: "原子科技招聘 <jobs@example.com>".into(),
            subject: "原子科技后端工程师面试邀请".into(),
            received_at: "2026-07-10T09:30:00Z".into(),
            body_text: "诚邀您参加面试".into(),
        }])
        .unwrap();
        let email = db.list_email_messages().unwrap().remove(0);
        {
            let connection = db.connection.lock().unwrap();
            connection
                .execute_batch(
                    "CREATE TRIGGER fail_email_confirmation
                     BEFORE INSERT ON application_events
                     WHEN NEW.event_type='email_status'
                     BEGIN SELECT RAISE(ABORT, 'forced confirmation failure'); END;",
                )
                .unwrap();
        }
        let result = db.create_application_from_email(
            &email.id,
            CreateApplicationInput {
                company_name: "原子科技".into(),
                company_short_name: None,
                industry: None,
                company_type: None,
                website: None,
                company_notes: None,
                position_title: "后端工程师".into(),
                department: None,
                location: None,
                recruitment_type: None,
                job_code: None,
                source_url: None,
                channel: Some("邮件识别".into()),
                applied_at: Some("2026-07-01".into()),
                priority: Some(2),
                jd_raw: None,
                resume_profile_id: None,
            },
        );
        assert!(result.is_err());
        assert!(db.list_applications().unwrap().is_empty());
        let unchanged = db.list_email_messages().unwrap().remove(0);
        assert_eq!(unchanged.status, "unmatched");
        assert_eq!(unchanged.matched_application_id, None);
    }

    #[test]
    fn confirmation_falls_back_when_no_effective_stage_event_remains() {
        let db = Database::in_memory().unwrap();
        let application = db
            .create_application(CreateApplicationInput {
                company_name: "回退科技".into(),
                company_short_name: None,
                industry: None,
                company_type: None,
                website: None,
                company_notes: None,
                position_title: "后端工程师".into(),
                department: None,
                location: None,
                recruitment_type: None,
                job_code: None,
                source_url: None,
                channel: Some("官网".into()),
                applied_at: Some("2026-07-01".into()),
                priority: Some(2),
                jd_raw: None,
                resume_profile_id: None,
            })
            .unwrap();
        db.ingest_emails(vec![RawEmail {
            account: "me@example.com".into(),
            mailbox: "INBOX".into(),
            uid: 28,
            message_id: Some("m28".into()),
            links: Vec::new(),
            sender: "回退科技招聘 <jobs@example.com>".into(),
            subject: "回退科技后端工程师招聘进展".into(),
            received_at: "2026-07-10T09:30:00Z".into(),
            body_text: "请登录招聘系统查看岗位申请信息".into(),
        }])
        .unwrap();
        let email = db.list_email_messages().unwrap().remove(0);
        {
            let connection = db.connection.lock().unwrap();
            connection
                .execute(
                    "UPDATE application_events
                     SET reverted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                     WHERE application_id=?1",
                    [&application.id],
                )
                .unwrap();
        }

        db.confirm_email_match(&email.id).unwrap();
        assert_eq!(db.list_email_messages().unwrap()[0].status, "confirmed");
        assert_eq!(
            db.get_application_detail(&application.id)
                .unwrap()
                .current_stage,
            "已投递"
        );
    }
}

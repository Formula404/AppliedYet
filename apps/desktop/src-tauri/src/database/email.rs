use super::{db_error, Database};
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailStats {
    pub this_week: i64,
    pub pending: i64,
    pub confirmed: i64,
    pub unmatched: i64,
}

#[derive(Debug, Serialize)]
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
}

impl Database {
    pub fn latest_email_uid(&self, account: &str) -> Result<u32, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let value: i64 = connection
            .query_row(
                "SELECT COALESCE((SELECT last_uid FROM email_sync_state WHERE account=?1 AND mailbox='INBOX'), 0)",
                [account],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        Ok(value.max(0) as u32)
    }

    #[cfg(test)]
    pub fn ingest_emails(&self, messages: Vec<RawEmail>) -> Result<SyncResult, String> {
        let fetched = messages.len();
        let cursor = messages
            .iter()
            .map(|item| (item.account.clone(), item.uid))
            .max_by_key(|(_, uid)| *uid);
        self.ingest_emails_with_cursor(messages, cursor, fetched)
    }

    pub fn ingest_emails_through(
        &self,
        messages: Vec<RawEmail>,
        account: &str,
        highest_uid: Option<u32>,
        scanned: usize,
    ) -> Result<SyncResult, String> {
        let cursor = highest_uid.map(|uid| (account.to_string(), uid));
        self.ingest_emails_with_cursor(messages, cursor, scanned)
    }

    fn ingest_emails_with_cursor(
        &self,
        messages: Vec<RawEmail>,
        cursor: Option<(String, u32)>,
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

        for message in messages {
            let combined = format!("{}\n{}", message.subject, message.body_text);
            let Some(classification) = classify(&message.subject, &message.body_text) else {
                continue;
            };
            result.recognized += 1;
            let candidate = best_match(&transaction, &combined, &message.received_at)?;
            if candidate.as_ref().is_some_and(|item| item.score >= 45) {
                result.matched += 1;
            }
            let matched = candidate.as_ref().filter(|item| item.score >= 45);
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
        if let Some((account, uid)) = cursor {
            transaction.execute(
                "INSERT INTO email_sync_state(account,mailbox,last_uid) VALUES (?1,'INBOX',?2)
                 ON CONFLICT(account,mailbox) DO UPDATE SET last_uid=MAX(last_uid,excluded.last_uid),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                params![account, uid],
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
                    c.name,p.title,a.current_stage,e.confidence,e.reasons_json
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
                Ok(EmailMessage {
                    id: row.get(0)?,
                    sender: row.get(1)?,
                    subject: row.get(2)?,
                    received_at: row.get(3)?,
                    snippet: row.get(4)?,
                    body_text: row.get(5)?,
                    links: serde_json::from_str(&links_json).unwrap_or_default(),
                    category: row.get(7)?,
                    suggested_stage: row.get(8)?,
                    status: row.get(9)?,
                    matched_application_id: row.get(10)?,
                    company: row.get(11)?,
                    role: row.get(12)?,
                    current_stage: row.get(13)?,
                    confidence: row.get(14)?,
                    reasons: serde_json::from_str(&reasons_json).unwrap_or_default(),
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
        connection.query_row(
            "SELECT SUM(received_at >= strftime('%Y-%m-%dT%H:%M:%fZ','now','-7 days')),
                    SUM(status='pending'), SUM(status='confirmed'), SUM(status='unmatched') FROM email_messages",
            [], |row| Ok(EmailStats { this_week: row.get::<_, Option<i64>>(0)?.unwrap_or(0), pending: row.get::<_, Option<i64>>(1)?.unwrap_or(0), confirmed: row.get::<_, Option<i64>>(2)?.unwrap_or(0), unmatched: row.get::<_, Option<i64>>(3)?.unwrap_or(0) })
        ).map_err(db_error)
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
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let (subject, body, received, status): (String, String, String, String) = connection
            .query_row(
                "SELECT subject,body_text,received_at,status FROM email_messages WHERE id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "邮件不存在".to_string())?;
        if status == "confirmed" {
            let reverted: bool = connection.query_row(
                "SELECT reverted_at IS NOT NULL FROM application_events WHERE source_type='email' AND source_id=?1 ORDER BY created_at DESC LIMIT 1",
                [id], |row| row.get(0),
            ).optional().map_err(db_error)?.unwrap_or(false);
            if !reverted {
                return Err(
                    "该邮件已经更新投递阶段，请先在投递详情的时间线中撤销对应邮件事件，再重新识别"
                        .into(),
                );
            }
        }
        // 已进入本地招聘邮件索引的记录，即使新规则无法判断具体阶段，也保留为
        // “待人工判断”，继续匹配投递，但绝不建议修改投递阶段。
        let classification = classify_existing(&subject, &body);
        let text = format!("{subject}\n{body}");
        let candidate = best_match(&connection, &text, &received)?;
        let matched = candidate.as_ref().filter(|item| item.score >= 45);
        let reasons = candidate
            .as_ref()
            .map(|item| item.reasons.clone())
            .unwrap_or_else(|| vec!["未找到公司或岗位信息足够接近的投递".into()]);
        connection.execute(
            "UPDATE email_messages SET category=?2,suggested_stage=?3,matched_application_id=?4,confidence=?5,reasons_json=?6,status=?7,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            params![id, classification.category, classification.stage, matched.map(|item| item.id.as_str()), candidate.as_ref().map(|item| item.score.clamp(0,100)).unwrap_or(0), serde_json::to_string(&reasons).map_err(|error| error.to_string())?, if matched.is_some() { "pending" } else { "unmatched" }],
        ).map_err(db_error)?;
        Ok(())
    }

    pub fn confirm_email_match(&self, id: &str) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let data: Option<(String, String, String, String, String, Option<String>)> = transaction.query_row(
            "SELECT e.status,e.matched_application_id,e.category,e.subject,e.received_at,e.suggested_stage FROM email_messages e WHERE e.id=?1",
            [id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).optional().map_err(db_error)?;
        let (status, application_id, category, subject, received_at, suggested_stage) =
            data.ok_or_else(|| "邮件不存在".to_string())?;
        if status == "confirmed" {
            return Ok(());
        }
        let current_stage: String = transaction
            .query_row(
                "SELECT current_stage FROM applications WHERE id=?1 AND deleted_at IS NULL",
                [&application_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "匹配的投递不存在".to_string())?;
        let next_stage = suggested_stage.filter(|stage| should_advance(&current_stage, stage));
        let effective_time: String = transaction.query_row(
            "SELECT CASE
                WHEN julianday(?2) >= julianday(COALESCE(MAX(happened_at), '1970-01-01T00:00:00Z')) THEN ?2
                ELSE strftime('%Y-%m-%dT%H:%M:%fZ','now') END
             FROM application_events WHERE application_id=?1 AND reverted_at IS NULL",
            params![application_id, received_at],
            |row| row.get(0),
        ).map_err(db_error)?;
        if let Some(stage) = next_stage.as_deref() {
            transaction.execute("UPDATE applications SET current_stage=?2,status_updated_at=?3,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1", params![application_id, stage, effective_time]).map_err(db_error)?;
        }
        transaction.execute(
            "INSERT INTO application_events(id,application_id,event_type,title,content,source_type,source_id,stage_before,stage_after,happened_at,reversible)
             VALUES (?1,?2,'email_status',?3,?4,'email',?5,?6,?7,?8,?9)",
            params![Uuid::new_v4().to_string(), application_id, category, subject, id, current_stage, next_stage.as_deref(), effective_time, next_stage.is_some()],
        ).map_err(db_error)?;
        transaction.execute("UPDATE email_messages SET status='confirmed',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1", [id]).map_err(db_error)?;
        transaction.commit().map_err(db_error)
    }
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
    let mut best: Option<MatchCandidate> = None;
    for row in candidates {
        let (id, company, role, job_code, applied_at) = row.map_err(db_error)?;
        let mut score = 0;
        let mut reasons = Vec::new();
        if contains_meaningful(&normalized, &normalize(&company)) {
            score += 50;
            reasons.push(format!("邮件中出现公司名称“{company}”"));
        }
        if contains_meaningful(&normalized, &normalize(&role)) {
            score += 30;
            reasons.push(format!("邮件中出现岗位名称“{role}”"));
        } else {
            let overlap = keyword_overlap(&normalized, &normalize(&role));
            if overlap >= 2 {
                score += 18;
                reasons.push("岗位关键词与已投岗位相符".into());
            }
        }
        if !job_code.trim().is_empty() && normalized.contains(&normalize(&job_code)) {
            score += 25;
            reasons.push(format!("岗位编号 {job_code} 一致"));
        }
        if received_at >= applied_at.as_str() {
            score += 8;
            reasons.push("邮件时间晚于投递时间".into());
        }
        let item = MatchCandidate { id, score, reasons };
        if best.as_ref().is_none_or(|old| item.score > old.score) {
            best = Some(item);
        }
    }
    Ok(best)
}

fn classify(subject: &str, body: &str) -> Option<Classification> {
    let subject = subject.to_lowercase();
    let value = format!("{subject}\n{}", body.to_lowercase());
    let has = |words: &[&str]| words.iter().any(|word| value.contains(word));
    if has(&[
        "验证码",
        "verification code",
        "账单",
        "newsletter",
        "广告退订",
    ]) {
        return None;
    }
    if has(&[
        "很遗憾",
        "未能通过",
        "不予录用",
        "rejected",
        "not move forward",
        "other candidates",
    ]) {
        return Some(Classification {
            category: "未通过",
            stage: Some("已拒绝"),
        });
    }
    // 具体的下一步行动优先于正文页脚或模板中偶然出现的 Offer 等宽泛词。
    if has(&[
        "在线测评",
        "测评链接",
        "assessment link",
        "online assessment",
        "测评邀请",
        "测评提醒",
    ]) {
        return Some(Classification {
            category: "测评",
            stage: Some("在线测评"),
        });
    }
    if has(&[
        "笔试链接",
        "笔试",
        "机试",
        "coding test",
        "written test",
        "笔试邀请",
        "机试邀请",
    ]) {
        return Some(Classification {
            category: "笔试",
            stage: Some("笔试"),
        });
    }
    if has(&[
        "面试",
        "interview",
        "视频沟通",
        "面谈",
        "技术沟通",
        "hr沟通",
        "沟通邀请",
    ]) {
        return Some(Classification {
            category: "面试",
            stage: Some("面试中"),
        });
    }
    let strong_offer = has(&[
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
        "发放offer",
        "发放 offer",
        "接受offer",
        "接受 offer",
        "offer通知",
        "offer 通知",
    ]);
    if strong_offer || contains_ascii_word(&subject, "offer") {
        return Some(Classification {
            category: "Offer",
            stage: Some("已获Offer"),
        });
    }
    if has(&[
        "通过筛选",
        "进入下一轮",
        "后续安排",
        "等待结果",
        "under review",
    ]) {
        return Some(Classification {
            category: "流程进展",
            stage: Some("等待结果"),
        });
    }
    if has(&[
        "投递成功",
        "申请成功",
        "收到您的简历",
        "application received",
        "申请已提交",
    ]) {
        return Some(Classification {
            category: "投递回执",
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
    use super::{classify, classify_existing, should_advance, RawEmail};
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
        assert_eq!((result.category, result.stage), ("测评", Some("在线测评")));
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
        assert!(!should_advance("进入人才库", "面试中"));
        assert!(!should_advance("HR 面试", "在线测评"));
        assert!(!should_advance("主动放弃", "面试中"));
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
            .ingest_emails_through(Vec::new(), "me@example.com", Some(23), 0)
            .unwrap();
        assert_eq!(result.fetched, 0);
        assert_eq!(result.recognized, 0);
        assert_eq!(db.latest_email_uid("me@example.com").unwrap(), 23);
    }
}

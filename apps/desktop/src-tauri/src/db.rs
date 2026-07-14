use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::Mutex};
use uuid::Uuid;

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "001_init", include_str!("../migrations/001_init.sql")),
    (
        2,
        "002_application_detail",
        include_str!("../migrations/002_application_detail.sql"),
    ),
];

pub struct Database {
    connection: Mutex<Connection>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationInput {
    pub company_name: String,
    pub position_title: String,
    pub location: Option<String>,
    pub channel: Option<String>,
    pub applied_at: Option<String>,
    pub jd_raw: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationListItem {
    pub id: String,
    pub company: String,
    pub company_mark: String,
    pub role: String,
    pub city: String,
    pub stage: String,
    pub stage_tone: String,
    pub priority: String,
    pub next_step: String,
    pub next_time: String,
    pub progress: i64,
    pub updated: String,
    pub risk: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationDetail {
    pub id: String,
    pub company_name: String,
    pub company_short_name: Option<String>,
    pub industry: Option<String>,
    pub company_type: Option<String>,
    pub website: Option<String>,
    pub company_notes: Option<String>,
    pub position_title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub recruitment_type: Option<String>,
    pub job_code: Option<String>,
    pub source_url: Option<String>,
    pub jd_raw: Option<String>,
    pub applied_at: Option<String>,
    pub channel: Option<String>,
    pub priority: i64,
    pub current_stage: String,
    pub next_action: Option<String>,
    pub next_action_due_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tasks: Vec<ApplicationTask>,
    pub events: Vec<ApplicationEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApplicationDetailInput {
    pub company_name: String,
    pub company_short_name: Option<String>,
    pub industry: Option<String>,
    pub company_type: Option<String>,
    pub website: Option<String>,
    pub company_notes: Option<String>,
    pub position_title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub recruitment_type: Option<String>,
    pub job_code: Option<String>,
    pub source_url: Option<String>,
    pub jd_raw: Option<String>,
    pub applied_at: Option<String>,
    pub channel: Option<String>,
    pub priority: i64,
    pub current_stage: String,
    pub next_action: Option<String>,
    pub next_action_due_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationTask {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: i64,
    pub status: String,
    pub due_at: Option<String>,
    pub remind_at: Option<String>,
    pub application_stage: Option<String>,
    pub source_type: String,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: i64,
    pub due_at: Option<String>,
    pub remind_at: Option<String>,
    pub application_stage: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationEvent {
    pub id: String,
    pub event_type: String,
    pub title: String,
    pub content: Option<String>,
    pub source_type: String,
    pub stage_before: Option<String>,
    pub stage_after: Option<String>,
    pub happened_at: String,
    pub reversible: bool,
    pub reverted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventInput {
    pub title: String,
    pub content: Option<String>,
    pub happened_at: Option<String>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("无法创建数据目录: {error}"))?;
        }
        let mut connection = Connection::open(path).map_err(db_error)?;
        Self::configure(&connection)?;
        Self::migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, String> {
        let mut connection = Connection::open_in_memory().map_err(db_error)?;
        Self::configure(&connection)?;
        Self::migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn configure(connection: &Connection) -> Result<(), String> {
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;",
            )
            .map_err(db_error)
    }

    fn migrate(connection: &mut Connection) -> Result<(), String> {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')));",
        ).map_err(db_error)?;

        for (version, name, sql) in MIGRATIONS {
            let applied = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
                    [version],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(db_error)?;
            if applied {
                continue;
            }

            let transaction = connection.transaction().map_err(db_error)?;
            transaction.execute_batch(sql).map_err(db_error)?;
            transaction
                .execute(
                    "INSERT INTO schema_migrations(version, name) VALUES (?1, ?2)",
                    params![version, name],
                )
                .map_err(db_error)?;
            transaction.commit().map_err(db_error)?;
        }

        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(db_error)?;
        if integrity != "ok" {
            return Err(format!("数据库完整性检查失败: {integrity}"));
        }
        Ok(())
    }

    pub fn list_applications(&self) -> Result<Vec<ApplicationListItem>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection.prepare(
            "SELECT a.id, c.name, p.title, COALESCE(p.location, ''), a.current_stage, a.priority, COALESCE(a.next_action, '待安排'), COALESCE(a.next_action_due_at, '待安排'), a.updated_at
             FROM applications a JOIN positions p ON p.id = a.position_id JOIN companies c ON c.id = p.company_id
             WHERE a.deleted_at IS NULL AND p.deleted_at IS NULL AND c.deleted_at IS NULL
             ORDER BY a.updated_at DESC",
        ).map_err(db_error)?;

        let rows = statement
            .query_map([], |row| {
                let company: String = row.get(1)?;
                let stage: String = row.get(4)?;
                let priority: i64 = row.get(5)?;
                Ok(ApplicationListItem {
                    id: row.get(0)?,
                    company_mark: company.chars().next().unwrap_or('?').to_string(),
                    company,
                    role: row.get(2)?,
                    city: row.get(3)?,
                    stage_tone: stage_tone(&stage).to_string(),
                    priority: priority_label(priority).to_string(),
                    next_step: row.get(6)?,
                    next_time: row.get(7)?,
                    progress: stage_progress(&stage),
                    updated: row.get(8)?,
                    risk: None,
                    stage,
                })
            })
            .map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn create_application(
        &self,
        input: CreateApplicationInput,
    ) -> Result<ApplicationListItem, String> {
        let company_name = required(input.company_name, "公司名称")?;
        let position_title = required(input.position_title, "岗位名称")?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;

        let company_id = transaction
            .query_row(
                "SELECT id FROM companies WHERE name = ?1 COLLATE NOCASE AND deleted_at IS NULL",
                [&company_name],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(db_error)?
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        transaction
            .execute(
                "INSERT OR IGNORE INTO companies(id, name) VALUES (?1, ?2)",
                params![company_id, company_name],
            )
            .map_err(db_error)?;

        let position_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO positions(id, company_id, title, location, jd_raw) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![position_id, company_id, position_title, clean(input.location), clean(input.jd_raw)],
        ).map_err(db_error)?;

        let application_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO applications(id, position_id, applied_at, channel) VALUES (?1, ?2, ?3, ?4)",
            params![application_id, position_id, clean(input.applied_at), clean(input.channel)],
        ).map_err(db_error)?;
        transaction.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, source_type, stage_after) VALUES (?1, ?2, 'application_created', '创建投递', 'manual', '已投递')",
            params![Uuid::new_v4().to_string(), application_id],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        drop(connection);

        self.get_application(&application_id)?
            .ok_or_else(|| "创建投递后无法读取记录".to_string())
    }

    pub fn update_application_stage(&self, id: &str, stage: &str) -> Result<(), String> {
        let stage = required(stage.to_string(), "投递阶段")?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let before = transaction
            .query_row(
                "SELECT current_stage FROM applications WHERE id = ?1 AND deleted_at IS NULL",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "投递记录不存在".to_string())?;
        if before == stage {
            return Ok(());
        }
        transaction.execute(
            "UPDATE applications SET current_stage = ?2, status_updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, stage],
        ).map_err(db_error)?;
        transaction.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, source_type, stage_before, stage_after, reversible) VALUES (?1, ?2, 'stage_changed', '更新投递阶段', 'manual', ?3, ?4, 1)",
            params![Uuid::new_v4().to_string(), id, before, stage],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)
    }

    pub fn get_application_detail(&self, id: &str) -> Result<ApplicationDetail, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut detail = connection
            .query_row(
                "SELECT a.id, c.name, c.short_name, c.industry, c.company_type, c.website, c.notes,
                        p.title, p.department, p.location, p.recruitment_type, p.job_code, p.source_url, p.jd_raw,
                        a.applied_at, a.channel, a.priority, a.current_stage, a.next_action, a.next_action_due_at,
                        a.created_at, a.updated_at
                 FROM applications a
                 JOIN positions p ON p.id = a.position_id
                 JOIN companies c ON c.id = p.company_id
                 WHERE a.id = ?1 AND a.deleted_at IS NULL AND p.deleted_at IS NULL AND c.deleted_at IS NULL",
                [id],
                |row| {
                    Ok(ApplicationDetail {
                        id: row.get(0)?,
                        company_name: row.get(1)?,
                        company_short_name: row.get(2)?,
                        industry: row.get(3)?,
                        company_type: row.get(4)?,
                        website: row.get(5)?,
                        company_notes: row.get(6)?,
                        position_title: row.get(7)?,
                        department: row.get(8)?,
                        location: row.get(9)?,
                        recruitment_type: row.get(10)?,
                        job_code: row.get(11)?,
                        source_url: row.get(12)?,
                        jd_raw: row.get(13)?,
                        applied_at: row.get(14)?,
                        channel: row.get(15)?,
                        priority: row.get(16)?,
                        current_stage: row.get(17)?,
                        next_action: row.get(18)?,
                        next_action_due_at: row.get(19)?,
                        created_at: row.get(20)?,
                        updated_at: row.get(21)?,
                        tasks: Vec::new(),
                        events: Vec::new(),
                    })
                },
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "投递记录不存在".to_string())?;
        detail.tasks = query_tasks(&connection, id)?;
        detail.events = query_events(&connection, id)?;
        Ok(detail)
    }

    pub fn update_application_detail(
        &self,
        id: &str,
        input: UpdateApplicationDetailInput,
    ) -> Result<ApplicationDetail, String> {
        let company_name = required(input.company_name, "公司名称")?;
        let position_title = required(input.position_title, "岗位名称")?;
        let current_stage = required(input.current_stage, "当前阶段")?;
        if !(1..=3).contains(&input.priority) {
            return Err("优先级必须在 1 到 3 之间".to_string());
        }

        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (company_id, position_id, stage_before) = transaction
            .query_row(
                "SELECT p.company_id, a.position_id, a.current_stage FROM applications a JOIN positions p ON p.id = a.position_id WHERE a.id = ?1 AND a.deleted_at IS NULL",
                [id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "投递记录不存在".to_string())?;

        transaction.execute(
            "UPDATE companies SET name = ?2, short_name = ?3, industry = ?4, company_type = ?5, website = ?6, notes = ?7, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![company_id, company_name, clean(input.company_short_name), clean(input.industry), clean(input.company_type), clean(input.website), clean(input.company_notes)],
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE positions SET title = ?2, department = ?3, location = ?4, recruitment_type = ?5, job_code = ?6, source_url = ?7, jd_raw = ?8, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![position_id, position_title, clean(input.department), clean(input.location), clean(input.recruitment_type), clean(input.job_code), clean(input.source_url), clean(input.jd_raw)],
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE applications SET applied_at = ?2, channel = ?3, priority = ?4, current_stage = ?5, next_action = ?6, next_action_due_at = ?7,
                    status_updated_at = CASE WHEN current_stage <> ?5 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE status_updated_at END,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, clean(input.applied_at), clean(input.channel), input.priority, current_stage, clean(input.next_action), clean(input.next_action_due_at)],
        ).map_err(db_error)?;

        let stage_changed = stage_before != current_stage;
        transaction.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, content, source_type, stage_before, stage_after, reversible)
             VALUES (?1, ?2, ?3, ?4, ?5, 'manual', ?6, ?7, ?8)",
            params![
                Uuid::new_v4().to_string(), id,
                if stage_changed { "stage_changed" } else { "detail_updated" },
                if stage_changed { "更新投递阶段" } else { "更新岗位与投递资料" },
                if stage_changed { Some(format!("{stage_before} → {current_stage}")) } else { None },
                if stage_changed { Some(stage_before) } else { None },
                if stage_changed { Some(current_stage) } else { None },
                if stage_changed { 1 } else { 0 },
            ],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        drop(connection);
        self.get_application_detail(id)
    }

    pub fn create_task(
        &self,
        application_id: &str,
        input: CreateTaskInput,
    ) -> Result<ApplicationTask, String> {
        let title = required(input.title, "任务标题")?;
        if !(1..=3).contains(&input.priority) {
            return Err("优先级必须在 1 到 3 之间".to_string());
        }
        let task_id = Uuid::new_v4().to_string();
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        transaction.execute(
            "INSERT INTO tasks(id, application_id, title, description, priority, due_at, remind_at, application_stage, source_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'manual')",
            params![task_id, application_id, title, clean(input.description), input.priority, clean(input.due_at), clean(input.remind_at), clean(input.application_stage)],
        ).map_err(db_error)?;
        transaction.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, content, source_type, source_id)
             VALUES (?1, ?2, 'task_created', '新增任务', ?3, 'manual', ?4)",
            params![Uuid::new_v4().to_string(), application_id, title, task_id],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        drop(connection);
        self.get_application_detail(application_id)?
            .tasks
            .into_iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| "创建任务后无法读取记录".to_string())
    }

    pub fn set_task_status(&self, task_id: &str, status: &str) -> Result<ApplicationTask, String> {
        if !matches!(status, "todo" | "doing" | "done" | "canceled") {
            return Err("无效的任务状态".to_string());
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (application_id, title, status_before) = transaction.query_row(
            "SELECT application_id, title, status FROM tasks WHERE id = ?1 AND deleted_at IS NULL",
            [task_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        ).optional().map_err(db_error)?.ok_or_else(|| "任务不存在".to_string())?;
        transaction.execute(
            "UPDATE tasks SET status = ?2,
                    completed_at = CASE WHEN ?2 = 'done' THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE NULL END,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![task_id, status],
        ).map_err(db_error)?;
        if status_before != status {
            transaction.execute(
                "INSERT INTO application_events(id, application_id, event_type, title, content, source_type, source_id)
                 VALUES (?1, ?2, 'task_status_changed', ?3, ?4, 'manual', ?5)",
                params![Uuid::new_v4().to_string(), application_id,
                    if status == "done" { "完成任务" } else { "更新任务状态" },
                    title, task_id],
            ).map_err(db_error)?;
        }
        transaction.commit().map_err(db_error)?;
        drop(connection);
        self.get_application_detail(&application_id)?
            .tasks
            .into_iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| "更新任务后无法读取记录".to_string())
    }

    pub fn create_event(
        &self,
        application_id: &str,
        input: CreateEventInput,
    ) -> Result<ApplicationEvent, String> {
        let title = required(input.title, "事件标题")?;
        let event_id = Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, content, source_type, happened_at)
             VALUES (?1, ?2, 'manual_note', ?3, ?4, 'manual', COALESCE(?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')))",
            params![event_id, application_id, title, clean(input.content), clean(input.happened_at)],
        ).map_err(db_error)?;
        query_events(&connection, application_id)?
            .into_iter()
            .find(|event| event.id == event_id)
            .ok_or_else(|| "创建事件后无法读取记录".to_string())
    }

    fn get_application(&self, id: &str) -> Result<Option<ApplicationListItem>, String> {
        Ok(self
            .list_applications()?
            .into_iter()
            .find(|item| item.id == id))
    }
}

fn query_tasks(
    connection: &Connection,
    application_id: &str,
) -> Result<Vec<ApplicationTask>, String> {
    let mut statement = connection.prepare(
        "SELECT id, title, description, priority, status, due_at, remind_at, application_stage, source_type, completed_at, created_at
         FROM tasks WHERE application_id = ?1 AND deleted_at IS NULL
         ORDER BY CASE status WHEN 'todo' THEN 0 WHEN 'doing' THEN 1 WHEN 'done' THEN 2 ELSE 3 END, due_at IS NULL, due_at, created_at DESC",
    ).map_err(db_error)?;
    let rows = statement
        .query_map([application_id], |row| {
            Ok(ApplicationTask {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                status: row.get(4)?,
                due_at: row.get(5)?,
                remind_at: row.get(6)?,
                application_stage: row.get(7)?,
                source_type: row.get(8)?,
                completed_at: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(db_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
}

fn query_events(
    connection: &Connection,
    application_id: &str,
) -> Result<Vec<ApplicationEvent>, String> {
    let mut statement = connection.prepare(
        "SELECT id, event_type, title, content, source_type, stage_before, stage_after, happened_at, reversible, reverted_at
         FROM application_events WHERE application_id = ?1 ORDER BY happened_at DESC, created_at DESC",
    ).map_err(db_error)?;
    let rows = statement
        .query_map([application_id], |row| {
            Ok(ApplicationEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                source_type: row.get(4)?,
                stage_before: row.get(5)?,
                stage_after: row.get(6)?,
                happened_at: row.get(7)?,
                reversible: row.get(8)?,
                reverted_at: row.get(9)?,
            })
        })
        .map_err(db_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}
fn required(value: String, field: &str) -> Result<String, String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(format!("{field}不能为空"))
    } else {
        Ok(value)
    }
}
fn db_error(error: rusqlite::Error) -> String {
    format!("数据库操作失败: {error}")
}
fn priority_label(priority: i64) -> &'static str {
    match priority {
        3 => "高",
        2 => "中",
        _ => "普通",
    }
}
fn stage_tone(stage: &str) -> &'static str {
    if stage.contains("拒绝") {
        "red"
    } else if stage.to_lowercase().contains("offer") {
        "green"
    } else if stage.contains("面") || stage.contains("HR") {
        "purple"
    } else if stage.contains("测评") {
        "orange"
    } else if stage.contains("等待") {
        "gray"
    } else {
        "blue"
    }
}
fn stage_progress(stage: &str) -> i64 {
    if stage.contains("拒绝") || stage.to_lowercase().contains("offer") {
        5
    } else if stage.contains("等待") {
        4
    } else if stage.contains("面") || stage.contains("HR") {
        3
    } else if stage.contains("测评") {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CreateApplicationInput {
        CreateApplicationInput {
            company_name: "测试公司".into(),
            position_title: "后端工程师".into(),
            location: Some("深圳".into()),
            channel: Some("官网".into()),
            applied_at: Some("2026-07-14".into()),
            jd_raw: None,
        }
    }

    fn detail_input() -> UpdateApplicationDetailInput {
        UpdateApplicationDetailInput {
            company_name: "测试公司科技".into(),
            company_short_name: Some("测试".into()),
            industry: Some("互联网".into()),
            company_type: Some("民营".into()),
            website: Some("https://example.com".into()),
            company_notes: Some("重点关注".into()),
            position_title: "高级后端工程师".into(),
            department: Some("平台研发部".into()),
            location: Some("广州".into()),
            recruitment_type: Some("校招".into()),
            job_code: Some("BE-001".into()),
            source_url: Some("https://example.com/jobs/BE-001".into()),
            jd_raw: Some("负责核心服务研发".into()),
            applied_at: Some("2026-07-14".into()),
            channel: Some("内推".into()),
            priority: 3,
            current_stage: "业务面试".into(),
            next_action: Some("准备技术面试".into()),
            next_action_due_at: Some("2026-07-16T02:00:00.000Z".into()),
        }
    }

    #[test]
    fn migrates_and_creates_application() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        assert_eq!(created.company, "测试公司");
        assert_eq!(created.stage, "已投递");
        assert_eq!(db.list_applications().unwrap().len(), 1);
    }

    #[test]
    fn stage_update_is_persisted() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        db.update_application_stage(&created.id, "面试中").unwrap();
        let item = db.list_applications().unwrap().remove(0);
        assert_eq!(item.stage, "面试中");
        assert_eq!(item.stage_tone, "purple");
    }

    #[test]
    fn detail_update_tasks_and_events_are_persisted() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();

        let updated = db
            .update_application_detail(&created.id, detail_input())
            .unwrap();
        assert_eq!(updated.company_name, "测试公司科技");
        assert_eq!(updated.position_title, "高级后端工程师");
        assert_eq!(updated.current_stage, "业务面试");

        let task = db
            .create_task(
                &created.id,
                CreateTaskInput {
                    title: "准备技术面试".into(),
                    description: Some("复习项目难点".into()),
                    priority: 3,
                    due_at: Some("2026-07-16T02:00:00.000Z".into()),
                    remind_at: None,
                    application_stage: Some("业务面试".into()),
                },
            )
            .unwrap();
        assert_eq!(task.status, "todo");
        let completed = db.set_task_status(&task.id, "done").unwrap();
        assert_eq!(completed.status, "done");
        assert!(completed.completed_at.is_some());

        db.create_event(
            &created.id,
            CreateEventInput {
                title: "HR 电话沟通".into(),
                content: Some("确认面试时间".into()),
                happened_at: None,
            },
        )
        .unwrap();

        let detail = db.get_application_detail(&created.id).unwrap();
        assert_eq!(detail.tasks.len(), 1);
        assert!(detail
            .events
            .iter()
            .any(|event| event.event_type == "stage_changed"));
        assert!(detail
            .events
            .iter()
            .any(|event| event.event_type == "task_created"));
        assert!(detail
            .events
            .iter()
            .any(|event| event.event_type == "task_status_changed"));
        assert!(detail
            .events
            .iter()
            .any(|event| event.event_type == "manual_note"));
    }
}

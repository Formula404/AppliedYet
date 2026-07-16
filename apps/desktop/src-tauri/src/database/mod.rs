use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use uuid::Uuid;

mod email;
mod interviews;
mod migrations;
pub mod models;
pub use email::{EmailLink, EmailMessage, EmailStats, RawEmail, SyncResult};
pub use interviews::{
    CreateInterviewQuestion, InterviewQuestionReview, InterviewSessionRecord, QuestionBankItem,
};
use migrations::MIGRATIONS;
pub use models::{
    AiApplicationContext, AiCallSummary, AiProviderSettings, AsrProviderSettings,
    CreateResumeProfileInput, EmailSettings, ProcessingJobResult, ProviderSettings,
    ResumeAiContext, ResumeProfile, StoredInterviewPreparation, UpdateResumeProfileInput,
};

pub struct Database {
    connection: Mutex<Connection>,
    path: Mutex<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationInput {
    pub company_name: String,
    #[serde(default)]
    pub company_short_name: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub company_type: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub company_notes: Option<String>,
    pub position_title: String,
    #[serde(default)]
    pub department: Option<String>,
    pub location: Option<String>,
    #[serde(default)]
    pub recruitment_type: Option<String>,
    #[serde(default)]
    pub job_code: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    pub channel: Option<String>,
    pub applied_at: Option<String>,
    #[serde(default)]
    pub priority: Option<i64>,
    pub jd_raw: Option<String>,
    pub resume_profile_id: Option<String>,
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
    pub archived: bool,
    pub resume_profile_id: Option<String>,
    pub resume_name: Option<String>,
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
    pub archived_at: Option<String>,
    pub resume_profile_id: Option<String>,
    pub resume_name: Option<String>,
    pub resume_file_format: Option<String>,
    pub resume_target_direction: Option<String>,
    pub tasks: Vec<ApplicationTask>,
    pub events: Vec<ApplicationEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApplicationDetailInput {
    pub company_name: String,
    #[serde(default)]
    pub company_short_name: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub company_type: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub company_notes: Option<String>,
    pub position_title: String,
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub recruitment_type: Option<String>,
    #[serde(default)]
    pub job_code: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub jd_raw: Option<String>,
    #[serde(default)]
    pub applied_at: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    pub priority: i64,
    pub current_stage: String,
    #[serde(default)]
    pub next_action: Option<String>,
    #[serde(default)]
    pub next_action_due_at: Option<String>,
    #[serde(default)]
    pub resume_profile_id: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterviewExperienceSource {
    pub id: String,
    pub application_id: String,
    pub source: String,
    pub url: Option<String>,
    pub title: String,
    pub status: String,
    pub questions: Vec<String>,
    pub error_message: Option<String>,
    pub imported_at: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskInput {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub priority: i64,
    #[serde(default)]
    pub due_at: Option<String>,
    #[serde(default)]
    pub remind_at: Option<String>,
    #[serde(default)]
    pub application_stage: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DueTaskReminder {
    pub task_id: String,
    pub application_id: String,
    pub title: String,
    pub company: String,
    pub role: String,
    pub due_at: Option<String>,
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
    pub updated_at: String,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub summary: DashboardSummary,
    pub tasks: Vec<DashboardTask>,
    pub events: Vec<DashboardEvent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub total: i64,
    pub active: i64,
    pub assessments: i64,
    pub interviews: i64,
    pub waiting: i64,
    pub offers: i64,
    pub rejected: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardTask {
    pub id: String,
    pub application_id: String,
    pub title: String,
    pub company: String,
    pub role: String,
    pub due_at: String,
    pub priority: i64,
    pub status: String,
    pub overdue: bool,
    pub tone: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardEvent {
    pub id: String,
    pub application_id: String,
    pub title: String,
    pub company: String,
    pub role: String,
    pub scheduled_at: String,
    pub kind: String,
    pub tone: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySummary {
    pub streak_days: i64,
    pub this_week_applications: i64,
    pub previous_week_applications: i64,
    pub daily_activity: Vec<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsData {
    pub total: i64,
    pub this_month: i64,
    pub previous_month: i64,
    pub assessments: i64,
    pub interviews: i64,
    pub offers: i64,
    pub average_feedback_days: Option<f64>,
    pub daily: Vec<AnalyticsPeriod>,
    pub weekly: Vec<AnalyticsPeriod>,
    pub directions: Vec<AnalyticsDirection>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsPeriod {
    pub label: String,
    pub applications: i64,
    pub interviews: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDirection {
    pub name: String,
    pub count: i64,
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
            path: Mutex::new(path.to_path_buf()),
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, String> {
        let mut connection = Connection::open_in_memory().map_err(db_error)?;
        Self::configure(&connection)?;
        Self::migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
            path: Mutex::new(PathBuf::from(":memory:")),
        })
    }

    pub fn storage_path(&self) -> Result<String, String> {
        Ok(self
            .path
            .lock()
            .map_err(|_| "数据路径锁已损坏".to_string())?
            .to_string_lossy()
            .into_owned())
    }

    pub fn relocate<F>(&self, directory: &Path, persist_location: F) -> Result<String, String>
    where
        F: FnOnce(&Path) -> Result<(), String>,
    {
        fs::create_dir_all(directory).map_err(|error| format!("无法创建数据目录: {error}"))?;
        let target = directory.join("applied-yet.sqlite3");
        // Keep both guards for the complete checkpoint/copy/swap operation. Every database
        // reader/writer takes `connection`, so no write can land in the old WAL after the
        // checkpoint and before the copied database becomes active.
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut path = self
            .path
            .lock()
            .map_err(|_| "数据路径锁已损坏".to_string())?;
        let current = path.clone();
        if current == target {
            persist_location(&target)?;
            return Ok(target.to_string_lossy().into_owned());
        }
        if target.exists() {
            return Err("所选目录已存在同名数据库，请选择其他目录".into());
        }
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(db_error)?;
        fs::copy(&current, &target).map_err(|error| {
            remove_database_files(&target);
            format!("移动数据失败: {error}")
        })?;
        let replacement = match Database::open(&target) {
            Ok(database) => database,
            Err(error) => {
                remove_database_files(&target);
                return Err(error);
            }
        };
        let replacement_connection = match replacement.connection.into_inner() {
            Ok(connection) => connection,
            Err(_) => {
                remove_database_files(&target);
                return Err("新数据库连接锁已损坏".to_string());
            }
        };
        if let Err(error) = persist_location(&target) {
            drop(replacement_connection);
            remove_database_files(&target);
            return Err(error);
        }
        *connection = replacement_connection;
        *path = target.clone();
        Ok(target.to_string_lossy().into_owned())
    }

    pub fn backup_to(&self, target: &Path) -> Result<String, String> {
        let parent = target
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| "备份路径无效".to_string())?;
        fs::create_dir_all(parent).map_err(|error| format!("无法创建备份目录: {error}"))?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let path = self
            .path
            .lock()
            .map_err(|_| "数据路径锁已损坏".to_string())?;
        if *path == target {
            return Err("不能用当前数据库文件覆盖自身".into());
        }
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(db_error)?;
        let temporary = parent.join(format!(".applied-yet-backup-{}.tmp", Uuid::new_v4()));
        if let Err(error) = fs::copy(path.as_path(), &temporary) {
            return Err(format!("创建备份失败: {error}"));
        }
        if let Err(error) = validate_database_file(&temporary) {
            remove_database_files(&temporary);
            return Err(error);
        }
        if let Err(error) = fs::rename(&temporary, target) {
            remove_database_files(&temporary);
            return Err(format!("保存备份失败: {error}"));
        }
        Ok(target.to_string_lossy().into_owned())
    }

    pub fn restore_from<F>(&self, source: &Path, persist_location: F) -> Result<String, String>
    where
        F: FnOnce(&Path) -> Result<(), String>,
    {
        if !source.is_file() {
            return Err("所选备份文件不存在".into());
        }
        validate_database_file(source)?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut path = self
            .path
            .lock()
            .map_err(|_| "数据路径锁已损坏".to_string())?;
        let source =
            fs::canonicalize(source).map_err(|error| format!("无法读取备份路径: {error}"))?;
        let current = fs::canonicalize(path.as_path()).unwrap_or_else(|_| path.clone());
        if source == current {
            return Err("不能从当前正在使用的数据库恢复".into());
        }
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(db_error)?;
        let directory = path
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .ok_or_else(|| "当前数据目录无效".to_string())?;
        let target = directory.join(format!("applied-yet-restored-{}.sqlite3", Uuid::new_v4()));
        fs::copy(&source, &target).map_err(|error| format!("复制备份失败: {error}"))?;
        let replacement = match Database::open(&target) {
            Ok(database) => database,
            Err(error) => {
                remove_database_files(&target);
                return Err(format!("无法恢复备份: {error}"));
            }
        };
        let replacement_connection = match replacement.connection.into_inner() {
            Ok(value) => value,
            Err(_) => {
                remove_database_files(&target);
                return Err("恢复后的数据库连接锁已损坏".to_string());
            }
        };
        if let Err(error) = persist_location(&target) {
            drop(replacement_connection);
            remove_database_files(&target);
            return Err(error);
        }
        *connection = replacement_connection;
        *path = target.clone();
        Ok(target.to_string_lossy().into_owned())
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
            if *version == 9 {
                let has_recorded_at: bool = transaction.query_row(
                    "SELECT EXISTS(SELECT 1 FROM pragma_table_info('application_events') WHERE name='recorded_at')",
                    [], |row| row.get(0),
                ).map_err(db_error)?;
                if !has_recorded_at {
                    transaction
                        .execute(
                            "ALTER TABLE application_events ADD COLUMN recorded_at TEXT",
                            [],
                        )
                        .map_err(db_error)?;
                }
            }
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
            "SELECT a.id, c.name, p.title, COALESCE(p.location, ''), a.current_stage, a.priority, COALESCE(a.next_action, '待安排'), COALESCE(a.next_action_due_at, '待安排'), a.updated_at, a.archived_at IS NOT NULL,
                    a.resume_profile_id,rp.name
             FROM applications a JOIN positions p ON p.id = a.position_id JOIN companies c ON c.id = p.company_id
             LEFT JOIN resume_profiles rp ON rp.id=a.resume_profile_id
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
                    archived: row.get(9)?,
                    resume_profile_id: row.get(10)?,
                    resume_name: row.get(11)?,
                    stage,
                })
            })
            .map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn get_activity_summary(&self) -> Result<ActivitySummary, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let streak_days = connection.query_row(
            "WITH RECURSIVE activity(day) AS (
                 SELECT DISTINCT date(happened_at, 'localtime') FROM application_events e
                 JOIN applications a ON a.id=e.application_id WHERE a.deleted_at IS NULL
             ), start(day) AS (
                 SELECT CASE WHEN EXISTS(SELECT 1 FROM activity WHERE day=date('now','localtime')) THEN date('now','localtime')
                             WHEN EXISTS(SELECT 1 FROM activity WHERE day=date('now','localtime','-1 day')) THEN date('now','localtime','-1 day') END
             ), streak(day,n) AS (
                 SELECT day,1 FROM start WHERE day IS NOT NULL
                 UNION ALL SELECT date(streak.day,'-1 day'),n+1 FROM streak
                 WHERE EXISTS(SELECT 1 FROM activity WHERE day=date(streak.day,'-1 day'))
             ) SELECT COALESCE(MAX(n),0) FROM streak",
            [], |row| row.get(0),
        ).map_err(db_error)?;
        let week_start = "date('now','localtime','-' || ((CAST(strftime('%w','now','localtime') AS INTEGER)+6)%7) || ' days')";
        let this_week_applications = connection.query_row(
            &format!("SELECT COUNT(*) FROM applications WHERE deleted_at IS NULL AND date(COALESCE(applied_at,created_at),'localtime') >= {week_start}"),
            [], |row| row.get(0),
        ).map_err(db_error)?;
        let previous_week_applications = connection.query_row(
            &format!("SELECT COUNT(*) FROM applications WHERE deleted_at IS NULL AND date(COALESCE(applied_at,created_at),'localtime') >= date({week_start},'-7 days') AND date(COALESCE(applied_at,created_at),'localtime') < {week_start}"),
            [], |row| row.get(0),
        ).map_err(db_error)?;
        let mut statement = connection.prepare(
            "WITH RECURSIVE offsets(value) AS (SELECT 13 UNION ALL SELECT value-1 FROM offsets WHERE value>0)
             SELECT COUNT(a.id) FROM offsets
             LEFT JOIN application_events e ON date(e.happened_at,'localtime')=date('now','localtime','-' || offsets.value || ' days')
             LEFT JOIN applications a ON a.id=e.application_id AND a.deleted_at IS NULL
             GROUP BY offsets.value ORDER BY offsets.value DESC",
        ).map_err(db_error)?;
        let daily_activity = statement
            .query_map([], |row| row.get(0))
            .map_err(db_error)?
            .collect::<Result<Vec<i64>, _>>()
            .map_err(db_error)?;
        Ok(ActivitySummary {
            streak_days,
            this_week_applications,
            previous_week_applications,
            daily_activity,
        })
    }

    pub fn export_applications_excel(&self, path: &str) -> Result<usize, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut workbook = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<?mso-application progid=\"Excel.Sheet\"?>\n<Workbook xmlns=\"urn:schemas-microsoft-com:office:spreadsheet\" xmlns:ss=\"urn:schemas-microsoft-com:office:spreadsheet\"><Styles><Style ss:ID=\"Header\"><Font ss:Bold=\"1\"/><Interior ss:Color=\"#DCE6F1\" ss:Pattern=\"Solid\"/></Style><Style ss:ID=\"Timeline\"><Alignment ss:Vertical=\"Top\" ss:WrapText=\"1\"/></Style></Styles>\n",
        );

        workbook.push_str("<Worksheet ss:Name=\"投递记录\"><Table><Column ss:Index=\"22\" ss:Width=\"420\" ss:StyleID=\"Timeline\"/>");
        push_excel_row(
            &mut workbook,
            &[
                "投递ID",
                "公司",
                "公司简称",
                "行业",
                "公司性质",
                "岗位",
                "部门",
                "地点",
                "招聘类型",
                "岗位编号",
                "投递渠道",
                "投递日期",
                "优先级",
                "当前阶段",
                "下一步行动",
                "下一步时间",
                "关联简历",
                "招聘链接",
                "公司官网",
                "JD原文",
                "公司备注",
                "事件时间线",
                "是否归档",
                "创建时间",
                "更新时间",
            ],
            true,
        );
        let mut application_count = 0usize;
        {
            let mut statement = connection.prepare(
                "SELECT a.id,c.name,COALESCE(c.short_name,''),COALESCE(c.industry,''),COALESCE(c.company_type,''),p.title,COALESCE(p.department,''),COALESCE(p.location,''),COALESCE(p.recruitment_type,''),COALESCE(p.job_code,''),COALESCE(a.channel,''),COALESCE(a.applied_at,''),a.priority,a.current_stage,COALESCE(a.next_action,''),COALESCE(a.next_action_due_at,''),COALESCE(rp.name,''),COALESCE(p.source_url,''),COALESCE(c.website,''),COALESCE(p.jd_raw,''),COALESCE(c.notes,''),
                        COALESCE((SELECT group_concat(timeline_line, char(10)) FROM (
                            SELECT e.happened_at || '｜' || CASE WHEN e.reverted_at IS NULL THEN '' ELSE '[已撤销]' END || e.title ||
                                   CASE WHEN e.stage_before IS NOT NULL AND e.stage_after IS NOT NULL THEN '（' || e.stage_before || ' → ' || e.stage_after || '）' ELSE '' END ||
                                   CASE WHEN e.content IS NOT NULL AND trim(e.content) <> '' THEN '：' || replace(replace(e.content, char(13), ' '), char(10), ' ') ELSE '' END AS timeline_line
                            FROM application_events e WHERE e.application_id=a.id
                            ORDER BY e.happened_at,e.created_at,e.rowid
                        )),''),
                        CASE WHEN a.archived_at IS NULL THEN '否' ELSE '是' END,a.created_at,a.updated_at
                 FROM applications a JOIN positions p ON p.id=a.position_id JOIN companies c ON c.id=p.company_id LEFT JOIN resume_profiles rp ON rp.id=a.resume_profile_id
                 WHERE a.deleted_at IS NULL AND p.deleted_at IS NULL AND c.deleted_at IS NULL ORDER BY a.created_at DESC",
            ).map_err(db_error)?;
            let rows = statement
                .query_map([], |row| {
                    let priority: i64 = row.get(12)?;
                    Ok(vec![
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                        row.get(10)?,
                        row.get(11)?,
                        priority_label(priority).to_string(),
                        row.get(13)?,
                        row.get(14)?,
                        row.get(15)?,
                        row.get(16)?,
                        row.get(17)?,
                        row.get(18)?,
                        row.get(19)?,
                        row.get(20)?,
                        row.get(21)?,
                        row.get(22)?,
                        row.get(23)?,
                        row.get(24)?,
                    ])
                })
                .map_err(db_error)?;
            for row in rows {
                let row = row.map_err(db_error)?;
                let cells: Vec<&str> = row.iter().map(String::as_str).collect();
                push_excel_row(&mut workbook, &cells, false);
                application_count += 1;
            }
        }
        workbook.push_str("</Table></Worksheet>\n");

        workbook.push_str("<Worksheet ss:Name=\"流程与状态变更\"><Table>");
        push_excel_row(
            &mut workbook,
            &[
                "事件ID",
                "投递ID",
                "公司",
                "岗位",
                "事件类型",
                "事件标题",
                "详细内容",
                "来源",
                "变更前阶段",
                "变更后阶段",
                "发生时间",
                "首次记录时间",
                "最后编辑时间",
                "是否可撤销",
                "是否已撤销",
                "撤销时间",
            ],
            true,
        );
        {
            let mut statement = connection.prepare(
                "SELECT e.id,e.application_id,c.name,p.title,e.event_type,e.title,COALESCE(e.content,''),e.source_type,COALESCE(e.stage_before,''),COALESCE(e.stage_after,''),e.happened_at,e.created_at,COALESCE(e.updated_at,e.created_at),CASE WHEN e.reversible=1 THEN '是' ELSE '否' END,CASE WHEN e.reverted_at IS NULL THEN '否' ELSE '是' END,COALESCE(e.reverted_at,'')
                 FROM application_events e JOIN applications a ON a.id=e.application_id JOIN positions p ON p.id=a.position_id JOIN companies c ON c.id=p.company_id
                 WHERE a.deleted_at IS NULL ORDER BY e.happened_at DESC,e.created_at DESC,e.rowid DESC",
            ).map_err(db_error)?;
            let rows = statement
                .query_map([], |row| {
                    (0..16)
                        .map(|index| row.get::<_, String>(index))
                        .collect::<Result<Vec<_>, _>>()
                })
                .map_err(db_error)?;
            for row in rows {
                let row = row.map_err(db_error)?;
                let cells: Vec<&str> = row.iter().map(String::as_str).collect();
                push_excel_row(&mut workbook, &cells, false);
            }
        }
        workbook.push_str("</Table></Worksheet></Workbook>");
        fs::write(path, workbook.as_bytes())
            .map_err(|error| format!("导出 Excel 失败: {error}"))?;
        Ok(application_count)
    }

    pub fn create_application(
        &self,
        input: CreateApplicationInput,
    ) -> Result<ApplicationListItem, String> {
        let company_name = required(input.company_name, "公司名称")?;
        let position_title = required(input.position_title, "岗位名称")?;
        let website = clean_external_url(input.website, "公司官网")?;
        let source_url = clean_external_url(input.source_url, "招聘链接")?;
        let priority = input.priority.unwrap_or(2);
        let applied_at = clean_date(input.applied_at, "投递日期")?;
        if !(1..=3).contains(&priority) {
            return Err("优先级必须在 1 到 3 之间".to_string());
        }
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
        transaction.execute(
            "UPDATE companies SET short_name=COALESCE(?2,short_name),industry=COALESCE(?3,industry),company_type=COALESCE(?4,company_type),website=COALESCE(?5,website),notes=COALESCE(?6,notes),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            params![company_id, clean(input.company_short_name), clean(input.industry), clean(input.company_type), website, clean(input.company_notes)],
        ).map_err(db_error)?;

        let position_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO positions(id, company_id, title, department, location, recruitment_type, job_code, source_url, jd_raw) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![position_id, company_id, position_title, clean(input.department), clean(input.location), clean(input.recruitment_type), clean(input.job_code), source_url, clean(input.jd_raw)],
        ).map_err(db_error)?;

        let resume_profile_id = match clean(input.resume_profile_id) {
            Some(id) => transaction
                .query_row(
                    "SELECT id FROM resume_profiles WHERE id=?1 AND deleted_at IS NULL AND archived_at IS NULL",
                    [&id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(db_error)?
                .ok_or_else(|| "选择的简历不存在或已归档".to_string())?
                .into(),
            None => None,
        };
        let application_id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO applications(id, position_id, applied_at, channel, priority, resume_profile_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![application_id, position_id, applied_at.clone(), clean(input.channel), priority, resume_profile_id],
        ).map_err(db_error)?;
        transaction.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, source_type, stage_after, happened_at)
             VALUES (?1, ?2, 'application_created', '创建投递', 'manual', '已投递',
                     COALESCE(?3 || 'T00:00:00.000Z', strftime('%Y-%m-%dT%H:%M:%fZ', 'now')))",
            params![Uuid::new_v4().to_string(), application_id, applied_at],
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
        let recent_drag_event = transaction
            .query_row(
                "SELECT id, stage_before FROM application_events
                 WHERE application_id=?1 AND event_type='stage_changed' AND source_id='board_drag'
                   AND reverted_at IS NULL AND julianday(COALESCE(recorded_at, created_at)) >= julianday('now', '-5 minutes')
                 ORDER BY COALESCE(recorded_at, created_at) DESC, rowid DESC LIMIT 1",
                [id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        transaction.execute(
            "UPDATE applications SET current_stage = ?2, status_updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, stage],
        ).map_err(db_error)?;
        if let Some((event_id, original_stage)) = recent_drag_event {
            if original_stage == stage {
                // The coalesced A -> ... -> A operation has no visible milestone, but keep
                // the superseded event as reverted audit data instead of destroying history.
                transaction
                    .execute(
                        "UPDATE application_events
                     SET reverted_at=strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                         updated_at=strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                         recorded_at=strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                     WHERE id=?1",
                        [&event_id],
                    )
                    .map_err(db_error)?;
            } else {
                transaction.execute(
                    "UPDATE application_events SET stage_after=?2, happened_at=strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_at=strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), recorded_at=strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id=?1",
                    params![event_id, stage],
                ).map_err(db_error)?;
            }
        } else {
            transaction.execute(
                "INSERT INTO application_events(id, application_id, event_type, title, source_type, source_id, stage_before, stage_after, reversible)
                 VALUES (?1, ?2, 'stage_changed', '更新投递阶段', 'manual', 'board_drag', ?3, ?4, 1)",
                params![Uuid::new_v4().to_string(), id, before, stage],
            ).map_err(db_error)?;
        }
        let (effective_stage, effective_time) = transaction.query_row(
            "SELECT stage_after, happened_at FROM application_events
             WHERE application_id=?1 AND stage_after IS NOT NULL AND reverted_at IS NULL AND event_type <> 'event_reverted'
             ORDER BY happened_at DESC, created_at DESC, rowid DESC LIMIT 1",
            [id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE applications SET current_stage=?2, status_updated_at=?3, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            params![id, effective_stage, effective_time],
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
                        a.created_at, a.updated_at, a.archived_at,
                        a.resume_profile_id,rp.name,rp.file_format,rp.target_direction
                 FROM applications a
                 JOIN positions p ON p.id = a.position_id
                 JOIN companies c ON c.id = p.company_id
                 LEFT JOIN resume_profiles rp ON rp.id=a.resume_profile_id AND rp.deleted_at IS NULL
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
                        archived_at: row.get(22)?,
                        resume_profile_id: row.get(23)?,
                        resume_name: row.get(24)?,
                        resume_file_format: row.get(25)?,
                        resume_target_direction: row.get(26)?,
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
        let website = clean_external_url(input.website, "公司官网")?;
        let source_url = clean_external_url(input.source_url, "招聘链接")?;
        let applied_at = clean_date(input.applied_at, "投递日期")?;
        let next_action_due_at = clean_timestamp(input.next_action_due_at, "下一步时间")?;
        if !(1..=3).contains(&input.priority) {
            return Err("优先级必须在 1 到 3 之间".to_string());
        }

        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (company_id, position_id, stage_before, resume_before, resume_name_before, applied_before) =
            transaction
                .query_row(
                    "SELECT p.company_id,a.position_id,a.current_stage,a.resume_profile_id,rp.name,a.applied_at
                 FROM applications a JOIN positions p ON p.id=a.position_id
                 LEFT JOIN resume_profiles rp ON rp.id=a.resume_profile_id
                 WHERE a.id=?1 AND a.deleted_at IS NULL",
                    [id],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, Option<String>>(3)?,
                            row.get::<_, Option<String>>(4)?,
                            row.get::<_, Option<String>>(5)?,
                        ))
                    },
                )
                .optional()
                .map_err(db_error)?
                .ok_or_else(|| "投递记录不存在".to_string())?;

        let resume_profile_id = clean(input.resume_profile_id);
        let resume_name_after = if resume_profile_id == resume_before {
            resume_name_before.clone()
        } else if let Some(resume_id) = resume_profile_id.as_deref() {
            Some(transaction.query_row(
                "SELECT name FROM resume_profiles WHERE id=?1 AND deleted_at IS NULL AND archived_at IS NULL",
                [resume_id],
                |row| row.get::<_, String>(0),
            ).optional().map_err(db_error)?.ok_or_else(|| "选择的简历不存在或已归档".to_string())?)
        } else {
            None
        };

        transaction.execute(
            "UPDATE companies SET name = ?2, short_name = ?3, industry = ?4, company_type = ?5, website = ?6, notes = ?7, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![company_id, company_name, clean(input.company_short_name), clean(input.industry), clean(input.company_type), website, clean(input.company_notes)],
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE positions SET title = ?2, department = ?3, location = ?4, recruitment_type = ?5, job_code = ?6, source_url = ?7, jd_raw = ?8, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![position_id, position_title, clean(input.department), clean(input.location), clean(input.recruitment_type), clean(input.job_code), source_url, clean(input.jd_raw)],
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE applications SET applied_at = ?2, channel = ?3, priority = ?4, current_stage = ?5, next_action = ?6, next_action_due_at = ?7, resume_profile_id=?8,
                    status_updated_at = CASE WHEN current_stage <> ?5 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE status_updated_at END,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, applied_at.clone(), clean(input.channel), input.priority, current_stage, clean(input.next_action), next_action_due_at, resume_profile_id],
        ).map_err(db_error)?;

        if applied_before != applied_at {
            if let Some(first_stage_time) = transaction
                .query_row(
                    "SELECT happened_at FROM application_events
                 WHERE application_id=?1 AND event_type='stage_changed' AND reverted_at IS NULL
                 ORDER BY rowid LIMIT 1",
                    [id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(db_error)?
            {
                if let Some(date) = applied_at.as_deref() {
                    let after_first_stage: bool = transaction
                        .query_row(
                            "SELECT julianday(?1 || 'T00:00:00.000Z') > julianday(?2)",
                            params![date, first_stage_time],
                            |row| row.get(0),
                        )
                        .map_err(db_error)?;
                    if after_first_stage {
                        return Err(format!(
                            "投递日期不能晚于首个状态变更（{first_stage_time}）"
                        ));
                    }
                }
            }
            transaction.execute(
                "UPDATE application_events
                 SET happened_at=COALESCE(?2 || 'T00:00:00.000Z', created_at), updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE application_id=?1 AND event_type='application_created'",
                params![id, applied_at],
            ).map_err(db_error)?;
        }

        let stage_changed = stage_before != current_stage;
        let resume_changed = resume_before != resume_profile_id;
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
        if resume_changed {
            transaction.execute(
                "INSERT INTO application_events(id,application_id,event_type,title,content,source_type)
                 VALUES (?1,?2,'resume_changed','更换关联简历',?3,'manual')",
                params![Uuid::new_v4().to_string(),id,format!("{} → {}",resume_name_before.as_deref().unwrap_or("未关联"),resume_name_after.as_deref().unwrap_or("未关联"))],
            ).map_err(db_error)?;
        }
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
        let due_at = clean_timestamp(input.due_at, "任务截止时间")?;
        let remind_at = clean_timestamp(input.remind_at, "任务提醒时间")?;
        validate_task_times(due_at.as_deref(), remind_at.as_deref())?;
        let task_id = Uuid::new_v4().to_string();
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        transaction.execute(
            "INSERT INTO tasks(id, application_id, title, description, priority, due_at, remind_at, application_stage, source_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'manual')",
            params![task_id, application_id, title, clean(input.description), input.priority, due_at, remind_at, clean(input.application_stage)],
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

    pub fn update_task(
        &self,
        task_id: &str,
        input: UpdateTaskInput,
    ) -> Result<ApplicationTask, String> {
        let title = required(input.title, "任务标题")?;
        if !(1..=3).contains(&input.priority) {
            return Err("优先级必须在 1 到 3 之间".to_string());
        }
        let due_at = clean_timestamp(input.due_at, "任务截止时间")?;
        let remind_at = clean_timestamp(input.remind_at, "任务提醒时间")?;
        validate_task_times(due_at.as_deref(), remind_at.as_deref())?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (application_id, old_title) = transaction
            .query_row(
                "SELECT application_id, title FROM tasks WHERE id = ?1 AND deleted_at IS NULL",
                [task_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "任务不存在".to_string())?;
        transaction.execute(
            "UPDATE tasks SET title = ?2, description = ?3, priority = ?4, due_at = ?5, remind_at = ?6, application_stage = ?7,
                    reminder_notified_at = CASE WHEN COALESCE(remind_at, '') <> COALESCE(?6, '') THEN NULL ELSE reminder_notified_at END,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![task_id, title, clean(input.description), input.priority, due_at, remind_at, clean(input.application_stage)],
        ).map_err(db_error)?;
        transaction.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, content, source_type, source_id)
             VALUES (?1, ?2, 'task_updated', '编辑任务', ?3, 'manual', ?4)",
            params![Uuid::new_v4().to_string(), application_id, format!("{old_title} → {title}"), task_id],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        drop(connection);
        self.get_application_detail(&application_id)?
            .tasks
            .into_iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| "更新任务后无法读取记录".to_string())
    }

    pub fn delete_task(&self, task_id: &str) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (application_id, title) = transaction
            .query_row(
                "SELECT application_id, title FROM tasks WHERE id = ?1 AND deleted_at IS NULL",
                [task_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "任务不存在".to_string())?;
        transaction.execute(
            "UPDATE tasks SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            [task_id],
        ).map_err(db_error)?;
        transaction.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, content, source_type, source_id)
             VALUES (?1, ?2, 'task_deleted', '删除任务', ?3, 'manual', ?4)",
            params![Uuid::new_v4().to_string(), application_id, title, task_id],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)
    }

    pub fn set_application_archived(&self, id: &str, archived: bool) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let exists = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM applications WHERE id = ?1 AND deleted_at IS NULL)",
                [id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(db_error)?;
        if !exists {
            return Err("投递记录不存在".to_string());
        }
        transaction.execute(
            "UPDATE applications SET archived_at = CASE WHEN ?2 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') ELSE NULL END,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, archived],
        ).map_err(db_error)?;
        transaction
            .execute(
                "INSERT INTO application_events(id, application_id, event_type, title, source_type)
             VALUES (?1, ?2, ?3, ?4, 'manual')",
                params![
                    Uuid::new_v4().to_string(),
                    id,
                    if archived {
                        "application_archived"
                    } else {
                        "application_restored"
                    },
                    if archived {
                        "归档投递"
                    } else {
                        "恢复投递"
                    }
                ],
            )
            .map_err(db_error)?;
        transaction.commit().map_err(db_error)
    }

    pub fn delete_archived_application(&self, id: &str) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let archived = transaction
            .query_row(
                "SELECT archived_at IS NOT NULL FROM applications WHERE id=?1 AND deleted_at IS NULL",
                [id],
                |row| row.get::<_, bool>(0),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "投递记录不存在或已经删除".to_string())?;
        if !archived {
            return Err("只能删除已归档的投递，请先归档后再删除".to_string());
        }
        transaction
            .execute(
                "INSERT INTO application_events(id, application_id, event_type, title, source_type)
             VALUES (?1, ?2, 'application_deleted', '删除已归档投递', 'manual')",
                params![Uuid::new_v4().to_string(), id],
            )
            .map_err(db_error)?;
        transaction.execute(
            "UPDATE applications SET deleted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'), updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            [id],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)
    }

    pub fn revert_application_event(&self, event_id: &str) -> Result<ApplicationDetail, String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let (application_id, stage_before, stage_after, reversible, reverted_at) = transaction.query_row(
            "SELECT application_id, stage_before, stage_after, reversible, reverted_at FROM application_events WHERE id = ?1",
            [event_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, bool>(3)?, row.get::<_, Option<String>>(4)?)),
        ).optional().map_err(db_error)?.ok_or_else(|| "事件不存在".to_string())?;
        if !reversible || reverted_at.is_some() {
            return Err("该事件不可撤销或已经撤销".to_string());
        }
        let _stage_before = stage_before.ok_or_else(|| "事件缺少原阶段".to_string())?;
        let _stage_after = stage_after.ok_or_else(|| "事件缺少目标阶段".to_string())?;
        transaction.execute(
            "UPDATE application_events SET reverted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1", [event_id],
        ).map_err(db_error)?;
        // 允许撤销任意一条仍有效的阶段事件，并从剩余有效历史重算当前阶段。
        // 这样既能清理连续误拖产生的中间记录，也不会因撤销较早事件而破坏后续状态。
        let (effective_stage, effective_time) = transaction.query_row(
            "SELECT stage_after, happened_at FROM application_events
             WHERE application_id=?1 AND stage_after IS NOT NULL AND reverted_at IS NULL AND event_type <> 'event_reverted'
             ORDER BY happened_at DESC,created_at DESC,rowid DESC LIMIT 1",
            [&application_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE applications SET current_stage = ?2, status_updated_at = ?3, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![application_id, effective_stage, effective_time],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        drop(connection);
        self.get_application_detail(&application_id)
    }

    pub fn update_application_event_time(
        &self,
        event_id: &str,
        happened_at: &str,
    ) -> Result<ApplicationDetail, String> {
        let happened_at = clean_timestamp(Some(happened_at.to_string()), "发生时间")?
            .ok_or_else(|| "发生时间不能为空".to_string())?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        let valid: bool = transaction
            .query_row("SELECT julianday(?1) IS NOT NULL", [&happened_at], |row| {
                row.get(0)
            })
            .map_err(db_error)?;
        if !valid {
            return Err("发生时间格式无效".to_string());
        }
        let (row_id, application_id, event_type, current_time, reverted_at) = transaction
            .query_row(
                "SELECT rowid, application_id, event_type, happened_at, reverted_at FROM application_events WHERE id=?1",
                [event_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, Option<String>>(4)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "事件不存在".to_string())?;
        if event_type != "stage_changed" {
            return Err("只能修改状态变更事件的发生时间".to_string());
        }
        if reverted_at.is_some() {
            return Err("已撤销的状态事件不能修改时间".to_string());
        }
        let previous_time = transaction.query_row(
            "SELECT happened_at FROM application_events
             WHERE application_id=?1 AND id<>?2 AND reverted_at IS NULL AND stage_after IS NOT NULL
               AND event_type IN ('application_created','stage_changed')
               AND (julianday(happened_at)<julianday(?3) OR (julianday(happened_at)=julianday(?3) AND rowid<?4))
             ORDER BY happened_at DESC, created_at DESC, rowid DESC LIMIT 1",
            params![application_id, event_id, current_time, row_id], |row| row.get::<_, String>(0),
        ).optional().map_err(db_error)?;
        let next_time = transaction.query_row(
            "SELECT happened_at FROM application_events
             WHERE application_id=?1 AND id<>?2 AND reverted_at IS NULL AND event_type='stage_changed'
               AND (julianday(happened_at)>julianday(?3) OR (julianday(happened_at)=julianday(?3) AND rowid>?4))
             ORDER BY happened_at, created_at, rowid LIMIT 1",
            params![application_id, event_id, current_time, row_id], |row| row.get::<_, String>(0),
        ).optional().map_err(db_error)?;
        if let Some(previous) = previous_time.as_deref() {
            let before_previous: bool = transaction
                .query_row(
                    "SELECT julianday(?1) < julianday(?2)",
                    params![happened_at, previous],
                    |row| row.get(0),
                )
                .map_err(db_error)?;
            if before_previous {
                return Err(format!("发生时间不能早于上一流程节点（{previous}）"));
            }
        }
        if let Some(next) = next_time.as_deref() {
            let after_next: bool = transaction
                .query_row(
                    "SELECT julianday(?1) > julianday(?2)",
                    params![happened_at, next],
                    |row| row.get(0),
                )
                .map_err(db_error)?;
            if after_next {
                return Err(format!("发生时间不能晚于下一流程节点（{next}）"));
            }
        }
        transaction.execute(
            "UPDATE application_events SET happened_at=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            params![event_id, happened_at],
        ).map_err(db_error)?;
        let (effective_stage, effective_time) = transaction.query_row(
            "SELECT stage_after, happened_at FROM application_events
             WHERE application_id=?1 AND stage_after IS NOT NULL AND reverted_at IS NULL AND event_type <> 'event_reverted'
             ORDER BY happened_at DESC, created_at DESC, rowid DESC LIMIT 1",
            [&application_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        ).map_err(db_error)?;
        transaction.execute(
            "UPDATE applications SET current_stage=?2, status_updated_at=?3, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            params![application_id, effective_stage, effective_time],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        drop(connection);
        self.get_application_detail(&application_id)
    }

    pub fn list_due_task_reminders(&self, now: &str) -> Result<Vec<DueTaskReminder>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection.prepare(
            "SELECT t.id, a.id, t.title, c.name, p.title, t.due_at
             FROM tasks t JOIN applications a ON a.id = t.application_id JOIN positions p ON p.id = a.position_id JOIN companies c ON c.id = p.company_id
             WHERE t.deleted_at IS NULL AND t.status IN ('todo', 'doing') AND t.remind_at IS NOT NULL
               AND t.remind_at <= ?1 AND t.reminder_notified_at IS NULL AND a.deleted_at IS NULL AND a.archived_at IS NULL
               AND a.current_stage NOT LIKE '%拒绝%' AND lower(a.current_stage) NOT LIKE '%offer%'
               AND a.current_stage NOT LIKE '%人才库%' AND a.current_stage NOT IN ('流程暂停','流程结束','主动放弃')
             ORDER BY t.remind_at LIMIT 20",
        ).map_err(db_error)?;
        let rows = statement
            .query_map([now], |row| {
                Ok(DueTaskReminder {
                    task_id: row.get(0)?,
                    application_id: row.get(1)?,
                    title: row.get(2)?,
                    company: row.get(3)?,
                    role: row.get(4)?,
                    due_at: row.get(5)?,
                })
            })
            .map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn mark_task_reminder_delivered(
        &self,
        task_id: &str,
        notified_at: &str,
    ) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection.execute(
            "UPDATE tasks SET reminder_notified_at = ?2 WHERE id = ?1 AND deleted_at IS NULL AND reminder_notified_at IS NULL",
            params![task_id, notified_at],
        ).map_err(db_error)?;
        if changed == 0 {
            return Err("提醒任务不存在或已经发送".to_string());
        }
        Ok(())
    }

    pub fn release_task_reminder_delivery(
        &self,
        task_id: &str,
        notified_at: &str,
    ) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection
            .execute(
                "UPDATE tasks SET reminder_notified_at = NULL
             WHERE id = ?1 AND deleted_at IS NULL AND reminder_notified_at = ?2",
                params![task_id, notified_at],
            )
            .map_err(db_error)?;
        Ok(())
    }

    pub fn get_provider_settings(&self) -> Result<ProviderSettings, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let ai_json = connection
            .query_row(
                "SELECT config_json FROM provider_settings WHERE provider = 'ai'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(db_error)?;
        let asr_json = connection
            .query_row(
                "SELECT config_json FROM provider_settings WHERE provider = 'asr'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(db_error)?;
        let ai: AiProviderSettings = ai_json
            .map(|value| serde_json::from_str(&value).map_err(json_error))
            .transpose()?
            .unwrap_or_default();
        let asr = asr_json
            .map(|value| serde_json::from_str(&value).map_err(json_error))
            .transpose()?
            .unwrap_or_default();
        let email_json = connection
            .query_row(
                "SELECT value_json FROM app_settings WHERE key='email'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(db_error)?;
        let email = email_json
            .map(|value| serde_json::from_str(&value).map_err(json_error))
            .transpose()?
            .unwrap_or_default();
        Ok(ProviderSettings { ai, asr, email })
    }

    pub fn save_ai_settings(&self, settings: AiProviderSettings) -> Result<(), String> {
        let base_url = settings.base_url.trim();
        if settings.provider.chars().count() > 200
            || settings.model.chars().count() > 500
            || settings
                .fallback_model
                .as_deref()
                .is_some_and(|value| value.chars().count() > 500)
            || base_url.chars().count() > 2048
        {
            return Err("AI 服务商、模型或 API 地址过长".to_string());
        }
        if !is_allowed_provider_url(base_url) {
            return Err("API 地址必须使用 HTTPS；仅本机服务允许 HTTP".to_string());
        }
        if settings.model.trim().is_empty() {
            return Err("模型名称不能为空".to_string());
        }
        if !matches!(
            settings.protocol.as_str(),
            "responses" | "chat" | "anthropic"
        ) {
            return Err("AI 接口协议无效".to_string());
        }
        if !(256..=32768).contains(&settings.max_output_tokens) {
            return Err("最大输出必须在 256 到 32768 Token 之间".to_string());
        }
        if !(5..=300).contains(&settings.timeout_seconds) {
            return Err("超时时间必须在 5 到 300 秒之间".to_string());
        }
        self.save_provider_json("ai", &settings)
    }

    pub fn save_asr_settings(&self, settings: AsrProviderSettings) -> Result<(), String> {
        if settings.provider.chars().count() > 200
            || settings.model.chars().count() > 500
            || settings.language.chars().count() > 100
            || settings.base_url.chars().count() > 2048
        {
            return Err("ASR 服务商、模型、语言或 API 地址过长".to_string());
        }
        if settings.provider.trim().is_empty()
            || settings.language.trim().is_empty()
            || settings.model.trim().is_empty()
        {
            return Err("ASR 服务商、模型和语言不能为空".to_string());
        }
        if !is_allowed_provider_url(settings.base_url.trim()) {
            return Err("ASR API 地址必须使用 HTTPS；仅本机服务允许 HTTP".to_string());
        }
        if !(30..=1800).contains(&settings.segment_seconds) {
            return Err("分片长度必须在 30 到 1800 秒之间".to_string());
        }
        if !(1..=2048).contains(&settings.file_limit_mb) {
            return Err("文件限制必须在 1 到 2048 MB 之间".to_string());
        }
        self.save_provider_json("asr", &settings)
    }

    pub fn save_email_settings(&self, settings: EmailSettings) -> Result<(), String> {
        if settings.provider.chars().count() > 200
            || settings.email_address.chars().count() > 320
            || settings.username.chars().count() > 500
            || settings.oauth_client_id.chars().count() > 500
            || settings.oauth_tenant.chars().count() > 500
        {
            return Err("邮箱设置中的文本字段过长".into());
        }
        if settings.enabled
            && (settings.email_address.trim().is_empty()
                || settings.imap_host.trim().is_empty()
                || settings.username.trim().is_empty())
        {
            return Err("启用邮箱检查前，请填写邮箱地址、IMAP 服务器和用户名".into());
        }
        if !(1..=65535).contains(&settings.imap_port) {
            return Err("IMAP 端口无效".into());
        }
        if !(1..=1440).contains(&settings.polling_minutes) {
            return Err("检查间隔必须在 1 到 1440 分钟之间".into());
        }
        if !matches!(settings.auth_method.as_str(), "password" | "oauth2") {
            return Err("邮箱认证方式无效".into());
        }
        if settings.enabled
            && settings.auth_method == "oauth2"
            && settings.oauth_client_id.trim().is_empty()
        {
            return Err("使用 OAuth2 前请填写桌面应用 Client ID".into());
        }
        let host = settings.imap_host.trim();
        if !host.is_empty() && !valid_network_host(host) {
            return Err("IMAP 服务器地址无效".into());
        }
        if !host.is_empty() && !settings.use_tls && !is_local_network_host(host) {
            return Err("远程 IMAP 连接必须启用 TLS，避免凭据和邮件明文传输".into());
        }
        let value = serde_json::to_string(&settings).map_err(json_error)?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute("INSERT INTO app_settings(key,value_json) VALUES ('email',?1) ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')", [value]).map_err(db_error)?;
        Ok(())
    }

    fn save_provider_json<T: Serialize>(&self, provider: &str, settings: &T) -> Result<(), String> {
        let value = serde_json::to_string(settings).map_err(json_error)?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute(
            "INSERT INTO provider_settings(provider, config_json) VALUES (?1, ?2)
             ON CONFLICT(provider) DO UPDATE SET config_json = excluded.config_json, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            params![provider, value],
        ).map_err(db_error)?;
        Ok(())
    }

    pub fn get_ai_application_context(
        &self,
        application_id: &str,
    ) -> Result<AiApplicationContext, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        ensure_active_application(&connection, application_id)?;
        connection
            .query_row(
                "SELECT a.id, c.name, p.title, p.department, p.location, a.current_stage,
                        p.jd_raw, c.notes, a.next_action,
                        rp.id,rp.name,rp.target_direction,rp.personal_info,rp.education_background,
                        rp.internship_experience,rp.project_experience,rp.professional_skills,
                        rp.academic_achievements,rp.skill_certificates
                 FROM applications a
                 JOIN positions p ON p.id = a.position_id
                 JOIN companies c ON c.id = p.company_id
                 LEFT JOIN resume_profiles rp ON rp.id=a.resume_profile_id AND rp.deleted_at IS NULL
                WHERE a.id = ?1 AND a.deleted_at IS NULL",
                [application_id],
                |row| {
                    let resume_id = row.get::<_, Option<String>>(9)?;
                    let resume = if let Some(resume_id) = resume_id {
                        Some(ResumeAiContext {
                            id: resume_id,
                            name: row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                            target_direction: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                            personal: parse_json_or_text(
                                &row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                            ),
                            education: parse_json_or_text(
                                &row.get::<_, Option<String>>(13)?.unwrap_or_default(),
                            ),
                            internships: parse_json_or_text(
                                &row.get::<_, Option<String>>(14)?.unwrap_or_default(),
                            ),
                            projects: parse_json_or_text(
                                &row.get::<_, Option<String>>(15)?.unwrap_or_default(),
                            ),
                            skills: row.get::<_, Option<String>>(16)?.unwrap_or_default(),
                            academics: parse_json_or_text(
                                &row.get::<_, Option<String>>(17)?.unwrap_or_default(),
                            ),
                            certificates: parse_json_or_text(
                                &row.get::<_, Option<String>>(18)?.unwrap_or_default(),
                            ),
                        })
                    } else {
                        None
                    };
                    Ok(AiApplicationContext {
                        application_id: row.get(0)?,
                        company_name: row.get(1)?,
                        position_title: row.get(2)?,
                        department: row.get(3)?,
                        location: row.get(4)?,
                        current_stage: row.get(5)?,
                        jd_raw: row.get(6)?,
                        company_notes: row.get(7)?,
                        next_action: row.get(8)?,
                        resume,
                    })
                },
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "投递不存在".to_string())
    }

    pub fn list_interview_experience_sources(
        &self,
        application_id: Option<&str>,
    ) -> Result<Vec<InterviewExperienceSource>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT id, application_id, source_type, url, title, status,
                        questions_json, error_message, created_at
                 FROM interview_experience_sources
                 WHERE (?1 IS NULL OR application_id = ?1)
                 ORDER BY created_at DESC, rowid DESC",
            )
            .map_err(db_error)?;
        let rows = statement
            .query_map([application_id], experience_source_row)
            .map_err(db_error)?;
        rows.map(|row| {
            row.map_err(db_error)
                .and_then(experience_source_from_values)
        })
        .collect()
    }

    pub fn get_interview_experience_source(
        &self,
        id: &str,
    ) -> Result<InterviewExperienceSource, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection
            .query_row(
                "SELECT id, application_id, source_type, url, title, status,
                        questions_json, error_message, created_at
                 FROM interview_experience_sources WHERE id = ?1",
                [id],
                experience_source_row,
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "面经来源不存在".to_string())
            .and_then(experience_source_from_values)
    }

    pub fn create_interview_experience_link(
        &self,
        application_id: &str,
        url: &str,
        title: &str,
    ) -> Result<InterviewExperienceSource, String> {
        let id = Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        ensure_active_application(&connection, application_id)?;
        connection
            .execute(
                "INSERT INTO interview_experience_sources
                    (id, application_id, source_type, url, title, status)
                 VALUES (?1, ?2, 'link', ?3, ?4, '待分析')",
                params![id, application_id, url, title],
            )
            .map_err(db_error)?;
        drop(connection);
        self.get_interview_experience_source(&id)
    }

    pub fn create_manual_interview_experience(
        &self,
        application_id: &str,
        title: &str,
        questions: &[String],
    ) -> Result<InterviewExperienceSource, String> {
        let id = Uuid::new_v4().to_string();
        let questions_json = serde_json::to_string(questions).map_err(|error| error.to_string())?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        ensure_active_application(&connection, application_id)?;
        connection
            .execute(
                "INSERT INTO interview_experience_sources
                    (id, application_id, source_type, title, status, questions_json)
                 VALUES (?1, ?2, 'manual', ?3, '已提取', ?4)",
                params![id, application_id, title, questions_json],
            )
            .map_err(db_error)?;
        drop(connection);
        self.get_interview_experience_source(&id)
    }

    pub fn update_interview_experience_analysis(
        &self,
        id: &str,
        title: Option<&str>,
        questions: &[String],
        error_message: Option<&str>,
    ) -> Result<InterviewExperienceSource, String> {
        let questions_json = serde_json::to_string(questions).map_err(|error| error.to_string())?;
        let status = if error_message.is_some() {
            "分析失败"
        } else {
            "已提取"
        };
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute(
                "UPDATE interview_experience_sources
                 SET title = COALESCE(?2, title), status = ?3, questions_json = ?4,
                     error_message = ?5, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ?1 AND source_type = 'link'",
                params![id, title, status, questions_json, error_message],
            )
            .map_err(db_error)?;
        if changed == 0 {
            return Err("待分析的网页面经不存在".into());
        }
        drop(connection);
        self.get_interview_experience_source(id)
    }

    pub fn delete_interview_experience_source(&self, id: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute(
                "DELETE FROM interview_experience_sources WHERE id = ?1",
                [id],
            )
            .map_err(db_error)?;
        if changed == 0 {
            return Err("面经来源不存在".into());
        }
        Ok(())
    }

    pub fn update_interview_experience_questions(
        &self,
        id: &str,
        questions: &[String],
    ) -> Result<InterviewExperienceSource, String> {
        let questions_json = serde_json::to_string(questions).map_err(|error| error.to_string())?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection
            .execute(
                "UPDATE interview_experience_sources
                 SET questions_json = ?2,
                     status = CASE WHEN ?3 > 0 THEN '已提取' ELSE status END,
                     error_message = CASE WHEN ?3 > 0 THEN NULL ELSE error_message END,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ?1",
                params![id, questions_json, questions.len()],
            )
            .map_err(db_error)?;
        if changed == 0 {
            return Err("面经来源不存在".into());
        }
        drop(connection);
        self.get_interview_experience_source(id)
    }

    pub fn start_ai_call(
        &self,
        application_id: Option<&str>,
        feature: &str,
        provider: &str,
        model: &str,
        sources_json: &str,
    ) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute(
            "INSERT INTO ai_calls(id, application_id, feature, provider, model, status, input_sources_json)
             VALUES (?1, ?2, ?3, ?4, ?5, 'running', ?6)",
            params![id, application_id, feature, provider, model, sources_json],
        ).map_err(db_error)?;
        Ok(id)
    }

    pub fn finish_ai_call(
        &self,
        id: &str,
        status: &str,
        attempts: i64,
        duration_ms: i64,
        response_json: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection
            .execute(
                "UPDATE ai_calls SET status = ?2, attempts = ?3, duration_ms = ?4,
                    response_json = ?5, error_message = ?6,
                    completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1",
                params![
                    id,
                    status,
                    attempts,
                    duration_ms,
                    response_json,
                    error_message
                ],
            )
            .map_err(db_error)?;
        Ok(())
    }

    pub fn save_interview_preparation(
        &self,
        application_id: &str,
        ai_call_id: &str,
        content_json: &str,
        source_json: &str,
        model: &str,
    ) -> Result<StoredInterviewPreparation, String> {
        let id = Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute(
            "INSERT INTO interview_preparations(id, application_id, ai_call_id, content_json, source_snapshot_json, model)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, application_id, ai_call_id, content_json, source_json, model],
        ).map_err(db_error)?;
        drop(connection);
        self.get_interview_preparation_by_id(&id)
    }

    fn get_interview_preparation_by_id(
        &self,
        id: &str,
    ) -> Result<StoredInterviewPreparation, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.query_row(
            "SELECT id, application_id, ai_call_id, content_json, source_snapshot_json, model, created_at
             FROM interview_preparations WHERE id = ?1",
            [id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?, row.get::<_, String>(6)?)),
        ).map_err(db_error).and_then(|(id, application_id, ai_call_id, content, sources, model, created_at)| Ok(StoredInterviewPreparation {
            id, application_id, ai_call_id,
            content: serde_json::from_str(&content).map_err(json_error)?,
            sources: serde_json::from_str(&sources).map_err(json_error)?,
            model, created_at,
        }))
    }

    pub fn latest_interview_preparation(
        &self,
        application_id: &str,
    ) -> Result<Option<StoredInterviewPreparation>, String> {
        let id = {
            let connection = self
                .connection
                .lock()
                .map_err(|_| "数据库连接锁已损坏".to_string())?;
            connection.query_row(
                "SELECT id FROM interview_preparations WHERE application_id = ?1 ORDER BY created_at DESC LIMIT 1",
                [application_id], |row| row.get::<_, String>(0),
            ).optional().map_err(db_error)?
        };
        id.map(|value| self.get_interview_preparation_by_id(&value))
            .transpose()
    }

    pub fn list_ai_calls(&self, application_id: &str) -> Result<Vec<AiCallSummary>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection.prepare(
            "SELECT id, feature, model, status, attempts, duration_ms, input_sources_json, error_message, created_at
             FROM ai_calls WHERE application_id = ?1 ORDER BY created_at DESC LIMIT 20",
        ).map_err(db_error)?;
        let rows = statement
            .query_map([application_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })
            .map_err(db_error)?;
        rows.map(|row| {
            let (
                id,
                feature,
                model,
                status,
                attempts,
                duration_ms,
                sources,
                error_message,
                created_at,
            ) = row.map_err(db_error)?;
            Ok(AiCallSummary {
                id,
                feature,
                model,
                status,
                attempts,
                duration_ms,
                input_sources: serde_json::from_str(&sources).map_err(json_error)?,
                error_message,
                created_at,
            })
        })
        .collect()
    }

    pub fn start_processing_job(
        &self,
        kind: &str,
        application_id: Option<&str>,
        source_path: &str,
    ) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute(
            "INSERT INTO processing_jobs(id, kind, application_id, source_path, status) VALUES (?1, ?2, ?3, ?4, 'running')",
            params![id, kind, application_id, source_path],
        ).map_err(db_error)?;
        Ok(id)
    }

    pub fn finish_processing_job(
        &self,
        id: &str,
        status: &str,
        result_json: Option<&str>,
        error_message: Option<&str>,
        duration_ms: i64,
    ) -> Result<ProcessingJobResult, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute(
            "UPDATE processing_jobs SET status=?2, result_json=?3, error_message=?4, duration_ms=?5,
                    completed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
            params![id, status, result_json, error_message, duration_ms],
        ).map_err(db_error)?;
        let kind = connection
            .query_row(
                "SELECT kind FROM processing_jobs WHERE id=?1",
                [id],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        Ok(ProcessingJobResult {
            id: id.to_string(),
            kind,
            status: status.to_string(),
            duration_ms: Some(duration_ms),
            result: result_json
                .map(serde_json::from_str)
                .transpose()
                .map_err(json_error)?,
        })
    }

    pub fn list_resume_profiles(&self) -> Result<Vec<ResumeProfile>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let mut statement = connection.prepare(
            "SELECT id,name,file_path,file_format,parsed_text,personal_info,education_background,
                    internship_experience,project_experience,professional_skills,academic_achievements,
                    skill_certificates,target_direction,notes,parent_profile_id,
                    (SELECT COUNT(*) FROM applications a WHERE a.resume_profile_id=resume_profiles.id AND a.deleted_at IS NULL),
                    (SELECT COUNT(*) FROM applications a WHERE a.resume_profile_id=resume_profiles.id AND a.deleted_at IS NULL AND ((a.current_stage LIKE '%测评%' OR a.current_stage LIKE '%笔试%') OR EXISTS (SELECT 1 FROM application_events e WHERE e.application_id=a.id AND e.reverted_at IS NULL AND (e.stage_after LIKE '%测评%' OR e.stage_after LIKE '%笔试%')))),
                    (SELECT COUNT(*) FROM applications a WHERE a.resume_profile_id=resume_profiles.id AND a.deleted_at IS NULL AND ((a.current_stage LIKE '%面试%' OR a.current_stage LIKE '%HR%') OR EXISTS (SELECT 1 FROM application_events e WHERE e.application_id=a.id AND e.reverted_at IS NULL AND (e.stage_after LIKE '%面试%' OR e.stage_after LIKE '%HR%')))),
                    (SELECT COUNT(*) FROM applications a WHERE a.resume_profile_id=resume_profiles.id AND a.deleted_at IS NULL AND (lower(a.current_stage) LIKE '%offer%' OR EXISTS (SELECT 1 FROM application_events e WHERE e.application_id=a.id AND e.reverted_at IS NULL AND lower(e.stage_after) LIKE '%offer%'))),
                    is_primary,archived_at,created_at,updated_at
             FROM resume_profiles WHERE deleted_at IS NULL
             ORDER BY archived_at IS NOT NULL, is_primary DESC, updated_at DESC",
        ).map_err(db_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(ResumeProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_path: row.get(2)?,
                    file_format: row.get(3)?,
                    parsed_text: row.get(4)?,
                    personal_info: row.get(5)?,
                    education_background: row.get(6)?,
                    internship_experience: row.get(7)?,
                    project_experience: row.get(8)?,
                    professional_skills: row.get(9)?,
                    academic_achievements: row.get(10)?,
                    skill_certificates: row.get(11)?,
                    target_direction: row.get(12)?,
                    notes: row.get(13)?,
                    parent_profile_id: row.get(14)?,
                    linked_application_count: row.get(15)?,
                    assessment_count: row.get(16)?,
                    interview_count: row.get(17)?,
                    offer_count: row.get(18)?,
                    is_primary: row.get::<_, i64>(19)? != 0,
                    archived_at: row.get(20)?,
                    created_at: row.get(21)?,
                    updated_at: row.get(22)?,
                })
            })
            .map_err(db_error)?;
        rows.map(|row| row.map_err(db_error)).collect()
    }

    pub fn create_resume_profile(
        &self,
        input: CreateResumeProfileInput,
    ) -> Result<ResumeProfile, String> {
        let name = required(input.name, "简历名称")?;
        let id = Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        if input.is_primary {
            connection
                .execute(
                    "UPDATE resume_profiles SET is_primary = 0 WHERE deleted_at IS NULL",
                    [],
                )
                .map_err(db_error)?;
        }
        connection.execute(
            "INSERT INTO resume_profiles(id,name,file_path,file_format,parsed_text,personal_info,education_background,
             internship_experience,project_experience,professional_skills,academic_achievements,skill_certificates,
             target_direction,notes,parent_profile_id,is_primary)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![id, name, input.file_path, input.file_format, input.parsed_text.unwrap_or_default(), input.personal_info.unwrap_or_default(), input.education_background.unwrap_or_default(), input.internship_experience.unwrap_or_default(), input.project_experience.unwrap_or_default(), input.professional_skills.unwrap_or_default(), input.academic_achievements.unwrap_or_default(), input.skill_certificates.unwrap_or_default(), input.target_direction.unwrap_or_default(), input.notes.unwrap_or_default(), input.parent_profile_id, input.is_primary as i64],
        ).map_err(db_error)?;
        drop(connection);
        self.get_resume_profile(&id)
    }

    pub fn get_resume_profile(&self, id: &str) -> Result<ResumeProfile, String> {
        self.list_resume_profiles()?
            .into_iter()
            .find(|profile| profile.id == id)
            .ok_or_else(|| "简历不存在".to_string())
    }

    pub fn update_resume_profile(
        &self,
        id: &str,
        input: UpdateResumeProfileInput,
    ) -> Result<ResumeProfile, String> {
        let name = required(input.name, "简历名称")?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection.execute(
            "UPDATE resume_profiles SET name=?2,personal_info=?3,education_background=?4,internship_experience=?5,
             project_experience=?6,professional_skills=?7,academic_achievements=?8,skill_certificates=?9,
             target_direction=?10,notes=?11,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id=?1 AND deleted_at IS NULL AND archived_at IS NULL",
            params![id, name, input.personal_info, input.education_background, input.internship_experience, input.project_experience, input.professional_skills, input.academic_achievements, input.skill_certificates, input.target_direction, input.notes],
        ).map_err(db_error)?;
        if changed == 0 {
            return Err("简历不存在或已归档".to_string());
        }
        drop(connection);
        self.get_resume_profile(id)
    }

    pub fn set_primary_resume_profile(&self, id: &str) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let exists: Option<String> = connection
            .query_row(
                "SELECT id FROM resume_profiles WHERE id=?1 AND deleted_at IS NULL AND archived_at IS NULL",
                [id],
                |row| row.get(0),
            )
            .optional()
            .map_err(db_error)?;
        if exists.is_none() {
            return Err("简历不存在".to_string());
        }
        connection.execute("UPDATE resume_profiles SET is_primary = CASE WHEN id=?1 THEN 1 ELSE 0 END WHERE deleted_at IS NULL", [id]).map_err(db_error)?;
        Ok(())
    }

    pub fn delete_resume_profile(&self, id: &str) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let transaction = connection.transaction().map_err(db_error)?;
        transaction.execute("UPDATE applications SET resume_profile_id=NULL,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE resume_profile_id=?1 AND deleted_at IS NULL", [id]).map_err(db_error)?;
        let changed = transaction.execute("UPDATE resume_profiles SET deleted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),is_primary=0 WHERE id=?1 AND deleted_at IS NULL", [id]).map_err(db_error)?;
        if changed == 0 {
            return Err("简历不存在".to_string());
        }
        transaction.execute(
            "UPDATE resume_profiles SET is_primary=1 WHERE id=(SELECT id FROM resume_profiles WHERE deleted_at IS NULL AND archived_at IS NULL ORDER BY updated_at DESC LIMIT 1) AND NOT EXISTS(SELECT 1 FROM resume_profiles WHERE deleted_at IS NULL AND archived_at IS NULL AND is_primary=1)",
            [],
        ).map_err(db_error)?;
        transaction.commit().map_err(db_error)?;
        Ok(())
    }

    pub fn duplicate_resume_profile(&self, id: &str) -> Result<ResumeProfile, String> {
        let new_id = Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection.execute(
            "INSERT INTO resume_profiles(id,name,file_path,file_format,parsed_text,personal_info,education_background,
             internship_experience,project_experience,professional_skills,academic_achievements,skill_certificates,
             target_direction,notes,parent_profile_id,is_primary)
             SELECT ?2,name || '（副本）',file_path,file_format,parsed_text,personal_info,education_background,
                    internship_experience,project_experience,professional_skills,academic_achievements,skill_certificates,
                    target_direction,notes,id,0
             FROM resume_profiles WHERE id=?1 AND deleted_at IS NULL",
            params![id, new_id],
        ).map_err(db_error)?;
        if changed == 0 {
            return Err("简历不存在".to_string());
        }
        drop(connection);
        self.get_resume_profile(&new_id)
    }

    pub fn set_resume_profile_archived(&self, id: &str, archived: bool) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let changed = connection.execute(
            "UPDATE resume_profiles SET archived_at=CASE WHEN ?2 THEN strftime('%Y-%m-%dT%H:%M:%fZ','now') ELSE NULL END,
                    is_primary=CASE WHEN ?2 THEN 0 ELSE is_primary END,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE id=?1 AND deleted_at IS NULL",
            params![id, archived],
        ).map_err(db_error)?;
        if changed == 0 {
            return Err("简历不存在".to_string());
        }
        if archived {
            connection.execute(
                "UPDATE resume_profiles SET is_primary=1 WHERE id=(SELECT id FROM resume_profiles WHERE deleted_at IS NULL AND archived_at IS NULL ORDER BY updated_at DESC LIMIT 1) AND NOT EXISTS(SELECT 1 FROM resume_profiles WHERE deleted_at IS NULL AND archived_at IS NULL AND is_primary=1)",
                [],
            ).map_err(db_error)?;
        }
        Ok(())
    }

    pub fn create_event(
        &self,
        application_id: &str,
        input: CreateEventInput,
    ) -> Result<ApplicationEvent, String> {
        let title = required(input.title, "事件标题")?;
        let happened_at = clean_timestamp(input.happened_at, "发生时间")?;
        let event_id = Uuid::new_v4().to_string();
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        connection.execute(
            "INSERT INTO application_events(id, application_id, event_type, title, content, source_type, happened_at)
             VALUES (?1, ?2, 'manual_note', ?3, ?4, 'manual', COALESCE(?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')))",
            params![event_id, application_id, title, clean(input.content), happened_at],
        ).map_err(db_error)?;
        query_events(&connection, application_id)?
            .into_iter()
            .find(|event| event.id == event_id)
            .ok_or_else(|| "创建事件后无法读取记录".to_string())
    }

    pub fn get_dashboard(
        &self,
        month_start: &str,
        month_end: &str,
        today_start: &str,
        today_end: &str,
    ) -> Result<DashboardData, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let summary = connection.query_row(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN current_stage NOT LIKE '%拒绝%' AND lower(current_stage) NOT LIKE '%offer%' AND current_stage NOT LIKE '%人才库%' AND current_stage NOT IN ('流程暂停', '流程结束', '主动放弃') THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN current_stage LIKE '%测评%' OR current_stage LIKE '%笔试%' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN current_stage LIKE '%面%' OR current_stage LIKE '%HR%' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN current_stage LIKE '%等待%' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN lower(current_stage) LIKE '%offer%' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN current_stage LIKE '%拒绝%' THEN 1 ELSE 0 END), 0)
             FROM applications WHERE deleted_at IS NULL AND archived_at IS NULL",
            [],
            |row| Ok(DashboardSummary {
                total: row.get(0)?, active: row.get(1)?, assessments: row.get(2)?, interviews: row.get(3)?,
                waiting: row.get(4)?, offers: row.get(5)?, rejected: row.get(6)?,
            }),
        ).map_err(db_error)?;

        let mut task_statement = connection.prepare(
            "WITH overdue AS (
                SELECT t.id, t.application_id, t.title, c.name AS company, p.title AS role, t.due_at, t.priority, t.status, t.application_stage, 0 AS bucket
                FROM tasks t JOIN applications a ON a.id = t.application_id JOIN positions p ON p.id = a.position_id JOIN companies c ON c.id = p.company_id
                WHERE t.deleted_at IS NULL AND a.deleted_at IS NULL AND a.archived_at IS NULL AND t.status IN ('todo', 'doing') AND t.due_at IS NOT NULL AND t.due_at < ?1
                  AND a.current_stage NOT LIKE '%拒绝%' AND lower(a.current_stage) NOT LIKE '%offer%' AND a.current_stage NOT LIKE '%人才库%' AND a.current_stage NOT IN ('流程暂停','流程结束','主动放弃')
                ORDER BY t.priority DESC, t.due_at LIMIT 6
             ), today AS (
                SELECT t.id, t.application_id, t.title, c.name AS company, p.title AS role, t.due_at, t.priority, t.status, t.application_stage, 1 AS bucket
                FROM tasks t JOIN applications a ON a.id = t.application_id JOIN positions p ON p.id = a.position_id JOIN companies c ON c.id = p.company_id
                WHERE t.deleted_at IS NULL AND a.deleted_at IS NULL AND a.archived_at IS NULL AND t.status IN ('todo', 'doing') AND t.due_at >= ?1 AND t.due_at < ?2
                  AND a.current_stage NOT LIKE '%拒绝%' AND lower(a.current_stage) NOT LIKE '%offer%' AND a.current_stage NOT LIKE '%人才库%' AND a.current_stage NOT IN ('流程暂停','流程结束','主动放弃')
             ), completed_today AS (
                SELECT t.id, t.application_id, t.title, c.name AS company, p.title AS role, t.due_at, t.priority, t.status, t.application_stage, 2 AS bucket
                FROM tasks t JOIN applications a ON a.id = t.application_id JOIN positions p ON p.id = a.position_id JOIN companies c ON c.id = p.company_id
                WHERE t.deleted_at IS NULL AND a.deleted_at IS NULL AND a.archived_at IS NULL AND t.status = 'done' AND t.due_at IS NOT NULL AND t.completed_at >= ?1 AND t.completed_at < ?2
                  AND a.current_stage NOT LIKE '%拒绝%' AND lower(a.current_stage) NOT LIKE '%offer%' AND a.current_stage NOT LIKE '%人才库%' AND a.current_stage NOT IN ('流程暂停','流程结束','主动放弃')
             )
             SELECT * FROM overdue UNION ALL SELECT * FROM today UNION ALL SELECT * FROM completed_today
             ORDER BY bucket, priority DESC, due_at",
        ).map_err(db_error)?;
        let task_rows = task_statement
            .query_map(params![today_start, today_end], |row| {
                let due_at: String = row.get(5)?;
                let stage: Option<String> = row.get(8)?;
                let status: String = row.get(7)?;
                Ok(DashboardTask {
                    id: row.get(0)?,
                    application_id: row.get(1)?,
                    title: row.get(2)?,
                    company: row.get(3)?,
                    role: row.get(4)?,
                    overdue: status != "done" && due_at.as_str() < today_start,
                    due_at,
                    priority: row.get(6)?,
                    status,
                    tone: schedule_tone(stage.as_deref().unwrap_or("")).to_string(),
                })
            })
            .map_err(db_error)?;
        let tasks = task_rows.collect::<Result<Vec<_>, _>>().map_err(db_error)?;

        let mut event_statement = connection.prepare(
            "SELECT 'task:' || t.id, a.id, t.title, c.name, p.title, t.due_at, 'task', COALESCE(t.application_stage, '')
             FROM tasks t JOIN applications a ON a.id = t.application_id JOIN positions p ON p.id = a.position_id JOIN companies c ON c.id = p.company_id
             WHERE t.deleted_at IS NULL AND a.deleted_at IS NULL AND a.archived_at IS NULL AND t.status NOT IN ('canceled') AND t.due_at >= ?1 AND t.due_at < ?2
               AND a.current_stage NOT LIKE '%拒绝%' AND lower(a.current_stage) NOT LIKE '%offer%' AND a.current_stage NOT LIKE '%人才库%' AND a.current_stage NOT IN ('流程暂停','流程结束','主动放弃')
             UNION ALL
             SELECT 'next:' || a.id, a.id, COALESCE(a.next_action, '下一步行动'), c.name, p.title, a.next_action_due_at, 'next_action', a.current_stage
             FROM applications a JOIN positions p ON p.id = a.position_id JOIN companies c ON c.id = p.company_id
             WHERE a.deleted_at IS NULL AND a.archived_at IS NULL AND a.next_action_due_at >= ?1 AND a.next_action_due_at < ?2
               AND a.current_stage NOT LIKE '%拒绝%' AND lower(a.current_stage) NOT LIKE '%offer%' AND a.current_stage NOT LIKE '%人才库%' AND a.current_stage NOT IN ('流程暂停','流程结束','主动放弃')
             UNION ALL
             SELECT 'milestone:' || e.id, a.id,
                    CASE WHEN e.event_type = 'application_created' THEN '新增投递'
                         WHEN e.stage_after IS NOT NULL THEN '进入' || e.stage_after
                         ELSE e.title END,
                    c.name, p.title, e.happened_at, 'milestone', COALESCE(e.stage_after, a.current_stage)
             FROM application_events e
             JOIN applications a ON a.id = e.application_id
             JOIN positions p ON p.id = a.position_id
             JOIN companies c ON c.id = p.company_id
             WHERE a.deleted_at IS NULL AND a.archived_at IS NULL
               AND e.reverted_at IS NULL AND e.event_type IN ('application_created', 'stage_changed', 'email_status')
               AND e.happened_at >= ?1 AND e.happened_at < ?2
             ORDER BY 6",
        ).map_err(db_error)?;
        let event_rows = event_statement
            .query_map(params![month_start, month_end], |row| {
                let stage: String = row.get(7)?;
                Ok(DashboardEvent {
                    id: row.get(0)?,
                    application_id: row.get(1)?,
                    title: row.get(2)?,
                    company: row.get(3)?,
                    role: row.get(4)?,
                    scheduled_at: row.get(5)?,
                    kind: row.get(6)?,
                    tone: schedule_tone(&stage).to_string(),
                })
            })
            .map_err(db_error)?;
        let events = event_rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        Ok(DashboardData {
            summary,
            tasks,
            events,
        })
    }

    pub fn get_analytics(&self) -> Result<AnalyticsData, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "数据库连接锁已损坏".to_string())?;
        let (total, this_month, previous_month, assessments, interviews, offers) = connection.query_row(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN date(COALESCE(applied_at,created_at),'localtime') >= date('now','localtime','start of month') THEN 1 ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN date(COALESCE(applied_at,created_at),'localtime') >= date('now','localtime','start of month','-1 month') AND date(COALESCE(applied_at,created_at),'localtime') < date('now','localtime','start of month') THEN 1 ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN (current_stage LIKE '%测评%' OR current_stage LIKE '%笔试%' OR EXISTS (SELECT 1 FROM application_events e WHERE e.application_id=applications.id AND e.reverted_at IS NULL AND (e.stage_after LIKE '%测评%' OR e.stage_after LIKE '%笔试%'))) THEN 1 ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN (current_stage LIKE '%面%' OR current_stage LIKE '%HR%' OR EXISTS (SELECT 1 FROM application_events e WHERE e.application_id=applications.id AND e.reverted_at IS NULL AND (e.stage_after LIKE '%面%' OR e.stage_after LIKE '%HR%'))) THEN 1 ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN (lower(current_stage) LIKE '%offer%' OR EXISTS (SELECT 1 FROM application_events e WHERE e.application_id=applications.id AND e.reverted_at IS NULL AND lower(e.stage_after) LIKE '%offer%')) THEN 1 ELSE 0 END),0)
             FROM applications WHERE deleted_at IS NULL AND archived_at IS NULL",
            [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        ).map_err(db_error)?;

        let average_feedback_days = connection.query_row(
            "SELECT AVG(julianday((SELECT MIN(e.happened_at) FROM application_events e WHERE e.application_id=a.id AND e.reverted_at IS NULL AND e.event_type <> 'application_created')) - julianday(COALESCE(a.applied_at,a.created_at)))
             FROM applications a WHERE a.deleted_at IS NULL AND a.archived_at IS NULL",
            [], |row| row.get::<_, Option<f64>>(0),
        ).map_err(db_error)?.map(|value| value.max(0.0));

        let periods = |sql: &str| -> Result<Vec<AnalyticsPeriod>, String> {
            let mut statement = connection.prepare(sql).map_err(db_error)?;
            let rows = statement
                .query_map([], |row| {
                    Ok(AnalyticsPeriod {
                        label: row.get(0)?,
                        applications: row.get(1)?,
                        interviews: row.get(2)?,
                    })
                })
                .map_err(db_error)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
        };
        let daily = periods(
            "WITH RECURSIVE periods(n,start_at) AS (SELECT 0,date('now','localtime','-6 days') UNION ALL SELECT n+1,date(start_at,'+1 day') FROM periods WHERE n<6)
             SELECT strftime('%m/%d',start_at),
                    (SELECT COUNT(*) FROM applications a WHERE a.deleted_at IS NULL AND a.archived_at IS NULL AND date(COALESCE(a.applied_at,a.created_at),'localtime')=periods.start_at),
                    (SELECT COUNT(DISTINCT e.application_id) FROM application_events e JOIN applications a ON a.id=e.application_id WHERE a.deleted_at IS NULL AND a.archived_at IS NULL AND e.reverted_at IS NULL AND (e.stage_after LIKE '%面%' OR e.stage_after LIKE '%HR%') AND date(e.happened_at,'localtime')=periods.start_at)
             FROM periods ORDER BY start_at"
        )?;
        let weekly = periods(
            "WITH RECURSIVE periods(n,start_at) AS (SELECT 0,date('now','localtime','-55 days') UNION ALL SELECT n+1,date(start_at,'+7 days') FROM periods WHERE n<7)
             SELECT strftime('%m/%d',start_at),
                    (SELECT COUNT(*) FROM applications a WHERE a.deleted_at IS NULL AND a.archived_at IS NULL AND date(COALESCE(a.applied_at,a.created_at),'localtime')>=periods.start_at AND date(COALESCE(a.applied_at,a.created_at),'localtime')<date(periods.start_at,'+7 days')),
                    (SELECT COUNT(DISTINCT e.application_id) FROM application_events e JOIN applications a ON a.id=e.application_id WHERE a.deleted_at IS NULL AND a.archived_at IS NULL AND e.reverted_at IS NULL AND (e.stage_after LIKE '%面%' OR e.stage_after LIKE '%HR%') AND date(e.happened_at,'localtime')>=periods.start_at AND date(e.happened_at,'localtime')<date(periods.start_at,'+7 days'))
             FROM periods ORDER BY start_at"
        )?;
        let mut direction_statement = connection.prepare(
            "SELECT COALESCE(NULLIF(TRIM(p.direction),''),p.title),COUNT(*) FROM applications a JOIN positions p ON p.id=a.position_id WHERE a.deleted_at IS NULL AND a.archived_at IS NULL GROUP BY 1 ORDER BY 2 DESC,1 LIMIT 6"
        ).map_err(db_error)?;
        let directions = direction_statement
            .query_map([], |row| {
                Ok(AnalyticsDirection {
                    name: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(db_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        Ok(AnalyticsData {
            total,
            this_month,
            previous_month,
            assessments,
            interviews,
            offers,
            average_feedback_days,
            daily,
            weekly,
            directions,
        })
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
        "SELECT id, event_type, title, content, source_type, stage_before, stage_after, happened_at, COALESCE(updated_at, created_at), reversible, reverted_at
         FROM application_events WHERE application_id = ?1 ORDER BY happened_at DESC, created_at DESC, rowid DESC",
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
                updated_at: row.get(8)?,
                reversible: row.get(9)?,
                reverted_at: row.get(10)?,
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

fn push_excel_row(target: &mut String, cells: &[&str], header: bool) {
    target.push_str("<Row>");
    for cell in cells {
        if header {
            target.push_str("<Cell ss:StyleID=\"Header\">");
        } else {
            target.push_str("<Cell>");
        }
        target.push_str("<Data ss:Type=\"String\">");
        for character in cell.chars() {
            match character {
                '&' => target.push_str("&amp;"),
                '<' => target.push_str("&lt;"),
                '>' => target.push_str("&gt;"),
                '\"' => target.push_str("&quot;"),
                '\'' => target.push_str("&apos;"),
                value if value.is_control() && !matches!(value, '\n' | '\r' | '\t') => {}
                value => target.push(value),
            }
        }
        target.push_str("</Data></Cell>");
    }
    target.push_str("</Row>\n");
}

fn parse_json_or_text(value: &str) -> serde_json::Value {
    serde_json::from_str(value).unwrap_or_else(|_| serde_json::Value::String(value.to_string()))
}
fn required(value: String, field: &str) -> Result<String, String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(format!("{field}不能为空"))
    } else if value.chars().count() > 500 {
        Err(format!("{field}不能超过 500 字"))
    } else {
        Ok(value)
    }
}
fn clean_date(value: Option<String>, field: &str) -> Result<Option<String>, String> {
    let Some(value) = clean(value) else {
        return Ok(None);
    };
    if chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d").is_ok() {
        return Ok(Some(value));
    }
    chrono::DateTime::parse_from_rfc3339(&value)
        .map(|date| Some(date.date_naive().to_string()))
        .map_err(|_| format!("{field}格式无效"))
}
fn clean_timestamp(value: Option<String>, field: &str) -> Result<Option<String>, String> {
    let Some(value) = clean(value) else {
        return Ok(None);
    };
    let parsed =
        chrono::DateTime::parse_from_rfc3339(&value).map_err(|_| format!("{field}格式无效"))?;
    Ok(Some(
        parsed
            .with_timezone(&chrono::Utc)
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    ))
}
fn validate_task_times(due_at: Option<&str>, remind_at: Option<&str>) -> Result<(), String> {
    match (due_at, remind_at) {
        (None, Some(_)) => Err("设置提醒前必须先设置任务截止时间".into()),
        (Some(due), Some(reminder)) if reminder > due => Err("任务提醒时间不能晚于截止时间".into()),
        _ => Ok(()),
    }
}
fn ensure_active_application(connection: &Connection, application_id: &str) -> Result<(), String> {
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM applications WHERE id=?1 AND deleted_at IS NULL AND archived_at IS NULL
               AND current_stage NOT LIKE '%拒绝%' AND lower(current_stage) NOT LIKE '%offer%'
               AND current_stage NOT LIKE '%人才库%' AND current_stage NOT IN ('流程暂停','流程结束','主动放弃'))",
            [application_id],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if exists {
        Ok(())
    } else {
        Err("只能为使用中的投递执行此操作".into())
    }
}

type ExperienceSourceRow = (
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    String,
    Option<String>,
    String,
);

fn experience_source_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExperienceSourceRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
    ))
}

fn experience_source_from_values(
    row: ExperienceSourceRow,
) -> Result<InterviewExperienceSource, String> {
    Ok(InterviewExperienceSource {
        id: row.0,
        application_id: row.1,
        source: row.2,
        url: row.3,
        title: row.4,
        status: row.5,
        questions: serde_json::from_str(&row.6).map_err(json_error)?,
        error_message: row.7,
        imported_at: row.8,
    })
}

fn db_error(error: rusqlite::Error) -> String {
    format!("数据库操作失败: {error}")
}
fn json_error(error: serde_json::Error) -> String {
    format!("设置数据格式错误: {error}")
}

fn remove_database_files(path: &Path) {
    for candidate in [
        path.to_path_buf(),
        PathBuf::from(format!("{}-wal", path.to_string_lossy())),
        PathBuf::from(format!("{}-shm", path.to_string_lossy())),
    ] {
        if candidate.exists() {
            let _ = fs::remove_file(candidate);
        }
    }
}

fn validate_database_file(path: &Path) -> Result<(), String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("无法打开数据库备份: {error}"))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(db_error)?;
    if integrity != "ok" {
        return Err(format!("备份完整性检查失败: {integrity}"));
    }
    let compatible: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_migrations')
                 AND EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='applications')",
            [],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if !compatible {
        return Err("所选文件不是有效的“投了吗”数据库备份".into());
    }
    Ok(())
}

fn is_allowed_provider_url(value: &str) -> bool {
    let Ok(parsed) = url::Url::parse(value) else {
        return false;
    };
    if !parsed.username().is_empty() || parsed.password().is_some() || parsed.host_str().is_none() {
        return false;
    }
    match parsed.scheme() {
        "https" => true,
        "http" => matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "::1")),
        _ => false,
    }
}
fn clean_external_url(value: Option<String>, field: &str) -> Result<Option<String>, String> {
    let Some(value) = clean(value) else {
        return Ok(None);
    };
    if value.chars().count() > 2048 {
        return Err(format!("{field}不能超过 2048 个字符"));
    }
    let parsed = url::Url::parse(&value).map_err(|_| format!("{field}不是有效网址"))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(format!("{field}必须是 HTTP 或 HTTPS 网址"));
    }
    Ok(Some(parsed.to_string()))
}
fn valid_network_host(host: &str) -> bool {
    if host.parse::<std::net::IpAddr>().is_ok() {
        return true;
    }
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
        })
}
fn is_local_network_host(host: &str) -> bool {
    matches!(
        host.to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1"
    )
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
        "teal"
    } else if stage.contains("面") || stage.contains("HR") {
        "purple"
    } else if stage.contains("测评")
        || stage.contains("笔试")
        || stage.contains("沟通")
        || stage.contains("谈薪")
    {
        "orange"
    } else if stage.contains("复盘") {
        "purple"
    } else if stage.contains("等待")
        || stage.contains("人才库")
        || stage.contains("暂停")
        || stage.contains("结束")
        || stage.contains("放弃")
    {
        "gray"
    } else {
        "blue"
    }
}
fn stage_progress(stage: &str) -> i64 {
    if stage.contains("拒绝")
        || stage.to_lowercase().contains("offer")
        || stage.contains("人才库")
        || stage.contains("暂停")
        || stage.contains("结束")
        || stage.contains("放弃")
    {
        5
    } else if stage.contains("等待") {
        4
    } else if stage.contains("面") || stage.contains("HR") {
        3
    } else if stage.contains("测评") || stage.contains("笔试") {
        2
    } else {
        1
    }
}

fn schedule_tone(value: &str) -> &'static str {
    if value.is_empty() {
        "gray"
    } else {
        stage_tone(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_location_persistence_keeps_the_original_database_active() {
        let root = std::env::temp_dir().join(format!("applied-yet-relocate-{}", Uuid::new_v4()));
        let original = root.join("original").join("applied-yet.sqlite3");
        let target_dir = root.join("target");
        let target = target_dir.join("applied-yet.sqlite3");
        let db = Database::open(&original).unwrap();
        let application = db.create_application(input()).unwrap();

        let result = db.relocate(&target_dir, |_| Err("pointer write failed".into()));

        assert!(result.is_err());
        assert_eq!(db.storage_path().unwrap(), original.to_string_lossy());
        assert!(!target.exists());
        assert_eq!(
            db.get_application_detail(&application.id).unwrap().id,
            application.id
        );
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn backup_and_restore_preserve_the_previous_database() {
        let root = std::env::temp_dir().join(format!("applied-yet-backup-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let source_path = root.join("source.sqlite3");
        let source = Database::open(&source_path).unwrap();
        let source_application = source.create_application(input()).unwrap();
        let backup_path = root.join("backup.sqlite3");
        source.backup_to(&backup_path).unwrap();

        let active_path = root.join("active.sqlite3");
        let active = Database::open(&active_path).unwrap();
        active.create_application(input()).unwrap();
        let restored_path = active.restore_from(&backup_path, |_| Ok(())).unwrap();
        assert_ne!(restored_path, active_path.to_string_lossy());
        assert!(active_path.exists());
        assert_eq!(
            active
                .get_application_detail(&source_application.id)
                .unwrap()
                .id,
            source_application.id
        );

        drop(active);
        drop(source);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_version_8_database_is_upgraded_with_recorded_at() {
        let mut connection = Connection::open_in_memory().unwrap();
        Database::configure(&connection).unwrap();
        connection.execute_batch(
            "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')));",
        ).unwrap();
        for (version, name, sql) in MIGRATIONS.iter().filter(|(version, _, _)| *version <= 7) {
            let transaction = connection.transaction().unwrap();
            transaction.execute_batch(sql).unwrap();
            transaction
                .execute(
                    "INSERT INTO schema_migrations(version,name) VALUES (?1,?2)",
                    params![version, name],
                )
                .unwrap();
            transaction.commit().unwrap();
        }
        connection.execute_batch(
            "ALTER TABLE application_events ADD COLUMN updated_at TEXT;
             UPDATE application_events SET updated_at=created_at WHERE updated_at IS NULL;
             CREATE TRIGGER application_events_fill_updated_at AFTER INSERT ON application_events
             WHEN NEW.updated_at IS NULL BEGIN UPDATE application_events SET updated_at=NEW.created_at WHERE id=NEW.id; END;
             INSERT INTO schema_migrations(version,name) VALUES (8,'008_application_event_times');",
        ).unwrap();
        let had_recorded_at: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('application_events') WHERE name='recorded_at')",
            [], |row| row.get(0),
        ).unwrap();
        assert!(!had_recorded_at);

        Database::migrate(&mut connection).unwrap();
        let has_recorded_at: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('application_events') WHERE name='recorded_at')",
            [], |row| row.get(0),
        ).unwrap();
        assert!(has_recorded_at);
        let db = Database {
            connection: Mutex::new(connection),
            path: Mutex::new(PathBuf::from(":memory:")),
        };
        let created = db.create_application(input()).unwrap();
        db.update_application_stage(&created.id, "等待结果")
            .unwrap();
        assert_eq!(
            db.get_application_detail(&created.id)
                .unwrap()
                .current_stage,
            "等待结果"
        );
    }

    #[test]
    fn version_12_repairs_sync_state_for_already_recorded_version_11() {
        let mut connection = Connection::open_in_memory().unwrap();
        Database::configure(&connection).unwrap();
        connection.execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')));").unwrap();
        for version in 1..=11 {
            connection
                .execute(
                    "INSERT INTO schema_migrations(version,name) VALUES (?1,?2)",
                    params![version, format!("legacy_{version}")],
                )
                .unwrap();
        }
        connection.execute_batch(
            "CREATE TABLE email_messages(id TEXT PRIMARY KEY);
             CREATE TABLE applications(id TEXT PRIMARY KEY,current_stage TEXT);
             CREATE TABLE tasks(id TEXT PRIMARY KEY,application_stage TEXT);
             CREATE TABLE application_events(id TEXT PRIMARY KEY,stage_before TEXT,stage_after TEXT);"
        ).unwrap();
        Database::migrate(&mut connection).unwrap();
        let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='email_sync_state')", [], |row| row.get(0)).unwrap();
        assert!(exists);
    }

    #[test]
    fn legacy_paused_stage_is_migrated_to_talent_pool() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        let connection = db.connection.lock().unwrap();
        connection
            .execute(
                "UPDATE applications SET current_stage='流程暂停' WHERE id=?1",
                [&created.id],
            )
            .unwrap();
        let migration = MIGRATIONS
            .iter()
            .find(|(version, _, _)| *version == 14)
            .unwrap()
            .2;
        connection.execute_batch(migration).unwrap();
        drop(connection);
        let item = db.list_applications().unwrap().remove(0);
        assert_eq!(item.stage, "进入人才库");
        assert_eq!(item.stage_tone, "gray");
        assert_eq!(item.progress, 5);
    }

    fn input() -> CreateApplicationInput {
        CreateApplicationInput {
            company_name: "测试公司".into(),
            company_short_name: None,
            industry: None,
            company_type: None,
            website: None,
            company_notes: None,
            position_title: "后端工程师".into(),
            department: None,
            location: Some("深圳".into()),
            recruitment_type: None,
            job_code: None,
            source_url: None,
            channel: Some("官网".into()),
            applied_at: Some("2026-07-14".into()),
            priority: None,
            jd_raw: None,
            resume_profile_id: None,
        }
    }

    #[test]
    fn interview_experience_sources_are_persisted_updated_and_deleted() {
        let db = Database::in_memory().unwrap();
        let application = db.create_application(input()).unwrap();
        let link = db
            .create_interview_experience_link(
                &application.id,
                "https://example.com/interview",
                "待分析帖子",
            )
            .unwrap();
        assert_eq!(link.status, "待分析");

        let analyzed = db
            .update_interview_experience_analysis(
                &link.id,
                Some("后端一面"),
                &["如何保证消息幂等？".into()],
                None,
            )
            .unwrap();
        assert_eq!(analyzed.title, "后端一面");
        assert_eq!(analyzed.questions, vec!["如何保证消息幂等？"]);

        let manual = db
            .create_manual_interview_experience(
                &application.id,
                "朋友分享",
                &["介绍一下项目。".into()],
            )
            .unwrap();
        assert_eq!(
            db.list_interview_experience_sources(Some(&application.id))
                .unwrap()
                .len(),
            2
        );
        db.delete_interview_experience_source(&manual.id).unwrap();
        assert_eq!(
            db.list_interview_experience_sources(Some(&application.id))
                .unwrap()
                .len(),
            1
        );
        let edited = db
            .update_interview_experience_questions(
                &link.id,
                &["编辑后的问题？".into(), "另一道问题？".into()],
            )
            .unwrap();
        assert_eq!(edited.questions.len(), 2);
        assert_eq!(edited.questions[0], "编辑后的问题？");
    }

    #[test]
    fn interview_sessions_resume_review_and_feed_question_bank() {
        let db = Database::in_memory().unwrap();
        let application = db.create_application(input()).unwrap();
        let session = db
            .create_interview_session(
                &application.id,
                "模拟面试",
                "技术综合模拟",
                "进行中",
                &[
                    CreateInterviewQuestion {
                        prompt: "如何保证消息幂等？".into(),
                        source: "面经".into(),
                        answer: String::new(),
                    },
                    CreateInterviewQuestion {
                        prompt: "介绍一下你的项目。".into(),
                        source: "AI 简历题".into(),
                        answer: String::new(),
                    },
                ],
            )
            .unwrap();
        db.update_interview_session_answer(
            &session.id,
            &session.questions[0].id,
            "使用业务唯一键和数据库唯一约束。",
        )
        .unwrap();
        db.update_interview_session_progress(&session.id, 1)
            .unwrap();
        let resumed = db.get_interview_session(&session.id).unwrap();
        assert_eq!(resumed.current_question_index, 1);
        assert!(resumed.questions[0].answer.contains("唯一键"));

        let completed = db.complete_interview_session(&session.id).unwrap();
        assert_eq!(completed.status, "待复盘");
        let reviewed = db
            .save_interview_session_review(
                &session.id,
                "幂等回答方向正确，项目题需要补充。",
                &[
                    InterviewQuestionReview {
                        question_id: session.questions[0].id.clone(),
                        score: 82,
                        evaluation: "方向正确，建议补充并发冲突处理。".into(),
                    },
                    InterviewQuestionReview {
                        question_id: session.questions[1].id.clone(),
                        score: 20,
                        evaluation: "本题未作答。".into(),
                    },
                ],
            )
            .unwrap();
        assert_eq!(reviewed.status, "复盘完成");
        let bank = db.list_question_bank_items().unwrap();
        assert_eq!(bank.len(), 2);
        assert!(bank.iter().any(|item| item.mastery == "熟悉"));
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
            resume_profile_id: None,
        }
    }

    #[test]
    fn migrates_and_creates_application() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        assert_eq!(created.company, "测试公司");
        assert_eq!(created.stage, "已投递");
        assert_eq!(db.list_applications().unwrap().len(), 1);
        let created_event = db
            .get_application_detail(&created.id)
            .unwrap()
            .events
            .into_iter()
            .find(|event| event.event_type == "application_created")
            .unwrap();
        assert_eq!(created_event.happened_at, "2026-07-14T00:00:00.000Z");
        assert!(!created_event.updated_at.is_empty());
    }

    #[test]
    fn stage_event_business_time_is_editable_without_breaking_sequence() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        db.update_application_stage(&created.id, "等待结果")
            .unwrap();
        let event = db
            .get_application_detail(&created.id)
            .unwrap()
            .events
            .into_iter()
            .find(|item| item.event_type == "stage_changed")
            .unwrap();

        let updated = db
            .update_application_event_time(&event.id, "2026-07-14T01:30:00.000Z")
            .unwrap();
        assert_eq!(
            updated
                .events
                .iter()
                .find(|item| item.id == event.id)
                .unwrap()
                .happened_at,
            "2026-07-14T01:30:00.000Z"
        );
        assert!(db
            .update_application_event_time(&event.id, "2026-07-13T23:59:00.000Z")
            .is_err());
        let created_event = updated
            .events
            .iter()
            .find(|item| item.event_type == "application_created")
            .unwrap();
        assert!(db
            .update_application_event_time(&created_event.id, "2026-07-14T02:00:00.000Z")
            .is_err());

        let calendar = db
            .get_dashboard(
                "2026-07-01T00:00:00.000Z",
                "2026-08-01T00:00:00.000Z",
                "2026-07-14T00:00:00.000Z",
                "2026-07-15T00:00:00.000Z",
            )
            .unwrap();
        assert!(calendar.events.iter().any(|item| {
            item.id == format!("milestone:{}", event.id)
                && item.scheduled_at == "2026-07-14T01:30:00.000Z"
        }));

        // Insertion order may differ from business chronology when users backfill events.
        // A later-inserted but earlier-dated stage must not be treated as the next boundary.
        let backfilled_id = Uuid::new_v4().to_string();
        let later_id = Uuid::new_v4().to_string();
        {
            let connection = db.connection.lock().unwrap();
            connection.execute(
                "INSERT INTO application_events(id,application_id,event_type,title,source_type,stage_before,stage_after,happened_at) VALUES (?1,?2,'stage_changed','后续节点','manual','等待结果','HR面试','2026-07-20T00:00:00.000Z')",
                params![later_id, created.id],
            ).unwrap();
            connection.execute(
                "INSERT INTO application_events(id,application_id,event_type,title,source_type,stage_before,stage_after,happened_at) VALUES (?1,?2,'stage_changed','补录节点','manual','已投递','在线测评','2026-07-10T00:00:00.000Z')",
                params![backfilled_id, created.id],
            ).unwrap();
        }
        assert!(db
            .update_application_event_time(&later_id, "2026-07-18T00:00:00.000Z")
            .is_ok());
    }

    #[test]
    fn stage_update_is_persisted() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        db.update_application_stage(&created.id, "面试中").unwrap();
        let item = db.list_applications().unwrap().remove(0);
        assert_eq!(item.stage, "面试中");
        assert_eq!(item.stage_tone, "purple");
        db.update_application_stage(&created.id, "终面").unwrap();
        let dashboard = db
            .get_dashboard(
                "2026-07-01T00:00:00.000Z",
                "2026-08-01T00:00:00.000Z",
                "2026-07-14T00:00:00.000Z",
                "2026-07-15T00:00:00.000Z",
            )
            .unwrap();
        assert_eq!(dashboard.summary.interviews, 1);
    }

    #[test]
    fn rapid_board_stage_changes_are_coalesced_but_manual_events_are_kept() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        db.update_application_stage(&created.id, "等待结果")
            .unwrap();
        let manual = db
            .create_event(
                &created.id,
                CreateEventInput {
                    title: "人工补记沟通".into(),
                    content: None,
                    happened_at: None,
                },
            )
            .unwrap();
        db.update_application_stage(&created.id, "HR面试").unwrap();

        let detail = db.get_application_detail(&created.id).unwrap();
        let stage_events: Vec<_> = detail
            .events
            .iter()
            .filter(|event| event.event_type == "stage_changed")
            .collect();
        assert_eq!(stage_events.len(), 1);
        assert_eq!(stage_events[0].stage_before.as_deref(), Some("已投递"));
        assert_eq!(stage_events[0].stage_after.as_deref(), Some("HR面试"));
        assert!(detail.events.iter().any(|event| event.id == manual.id));

        db.update_application_stage(&created.id, "已投递").unwrap();
        let returned = db.get_application_detail(&created.id).unwrap();
        let reverted_stage = returned
            .events
            .iter()
            .find(|event| event.event_type == "stage_changed")
            .expect("coalescing should retain an audit record");
        assert!(reverted_stage.reverted_at.is_some());
        assert_eq!(returned.current_stage, "已投递");
        assert!(returned.events.iter().any(|event| event.id == manual.id));
    }

    #[test]
    fn update_inputs_accept_missing_optional_fields() {
        let application: UpdateApplicationDetailInput = serde_json::from_value(serde_json::json!({
            "companyName":"测试公司","positionTitle":"后端工程师","priority":2,"currentStage":"已投递"
        })).unwrap();
        assert!(application.resume_profile_id.is_none());
        assert!(application.next_action_due_at.is_none());

        let task: UpdateTaskInput = serde_json::from_value(serde_json::json!({
            "title":"准备面试","priority":2
        }))
        .unwrap();
        assert!(task.due_at.is_none());
        assert!(task.remind_at.is_none());
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
                    remind_at: Some("2026-07-16T01:45:00.000Z".into()),
                    application_stage: Some("业务面试".into()),
                },
            )
            .unwrap();
        assert_eq!(task.status, "todo");
        assert_eq!(task.remind_at.as_deref(), Some("2026-07-16T01:45:00.000Z"));
        let dashboard = db
            .get_dashboard(
                "2026-07-01T00:00:00.000Z",
                "2026-08-01T00:00:00.000Z",
                "2026-07-16T00:00:00.000Z",
                "2026-07-17T00:00:00.000Z",
            )
            .unwrap();
        assert_eq!(dashboard.summary.total, 1);
        assert_eq!(dashboard.summary.interviews, 1);
        assert_eq!(dashboard.tasks.len(), 1);
        assert_eq!(dashboard.events.len(), 4);
        assert_eq!(
            dashboard
                .events
                .iter()
                .filter(|item| item.kind == "milestone")
                .count(),
            2
        );
        let completed = db.set_task_status(&task.id, "done").unwrap();
        assert_eq!(completed.status, "done");
        assert!(completed.completed_at.is_some());
        {
            let connection = db.connection.lock().unwrap();
            connection
                .execute(
                    "UPDATE tasks SET completed_at = '2026-07-16T03:00:00.000Z' WHERE id = ?1",
                    [&task.id],
                )
                .unwrap();
        }
        let completed_dashboard = db
            .get_dashboard(
                "2026-07-01T00:00:00.000Z",
                "2026-08-01T00:00:00.000Z",
                "2026-07-16T00:00:00.000Z",
                "2026-07-17T00:00:00.000Z",
            )
            .unwrap();
        assert!(completed_dashboard
            .tasks
            .iter()
            .any(|item| item.id == task.id && item.status == "done"));
        let restored = db.set_task_status(&task.id, "todo").unwrap();
        assert_eq!(restored.status, "todo");

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

    #[test]
    fn dashboard_keeps_today_tasks_visible_after_overdue_backlog() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        for index in 0..14 {
            db.create_task(
                &created.id,
                CreateTaskInput {
                    title: format!("逾期任务 {index}"),
                    description: None,
                    priority: 3,
                    due_at: Some("2026-07-13T02:00:00.000Z".into()),
                    remind_at: None,
                    application_stage: Some("在线测评".into()),
                },
            )
            .unwrap();
        }
        db.create_task(
            &created.id,
            CreateTaskInput {
                title: "今天必须完成".into(),
                description: None,
                priority: 1,
                due_at: Some("2026-07-14T08:00:00.000Z".into()),
                remind_at: None,
                application_stage: Some("HR面试".into()),
            },
        )
        .unwrap();

        let dashboard = db
            .get_dashboard(
                "2026-07-01T00:00:00.000Z",
                "2026-08-01T00:00:00.000Z",
                "2026-07-14T00:00:00.000Z",
                "2026-07-15T00:00:00.000Z",
            )
            .unwrap();
        assert_eq!(dashboard.tasks.len(), 7);
        assert!(dashboard
            .tasks
            .iter()
            .any(|task| task.title == "今天必须完成"));
        assert_eq!(schedule_tone("Offer沟通"), "teal");
        assert_eq!(schedule_tone("在线测评"), stage_tone("在线测评"));
        assert_eq!(schedule_tone("HR面试"), stage_tone("HR面试"));
    }

    #[test]
    fn terminal_stages_suppress_actions_tasks_and_reminders() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        db.create_task(
            &created.id,
            CreateTaskInput {
                title: "不应继续提醒".into(),
                description: None,
                priority: 2,
                due_at: Some("2026-07-20T02:00:00.000Z".into()),
                remind_at: Some("2026-07-20T01:00:00.000Z".into()),
                application_stage: Some("HR面试".into()),
            },
        )
        .unwrap();
        db.update_application_stage(&created.id, "主动放弃")
            .unwrap();

        assert!(db
            .list_due_task_reminders("2026-07-20T01:01:00.000Z")
            .unwrap()
            .is_empty());
        let dashboard = db
            .get_dashboard(
                "2026-07-01T00:00:00.000Z",
                "2026-08-01T00:00:00.000Z",
                "2026-07-20T00:00:00.000Z",
                "2026-07-21T00:00:00.000Z",
            )
            .unwrap();
        assert_eq!(dashboard.summary.active, 0);
        assert!(dashboard.tasks.is_empty());
        assert!(db.get_ai_application_context(&created.id).is_err());
        assert_eq!(stage_tone("主动放弃"), "gray");
        assert_eq!(stage_progress("主动放弃"), 5);
    }

    #[test]
    fn task_management_archiving_event_revert_and_reminder_dedup_work() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        db.update_application_stage(&created.id, "HR面试").unwrap();
        let changed_event = db
            .get_application_detail(&created.id)
            .unwrap()
            .events
            .into_iter()
            .find(|event| event.event_type == "stage_changed")
            .unwrap();
        let reverted = db.revert_application_event(&changed_event.id).unwrap();
        assert_eq!(reverted.current_stage, "已投递");
        assert!(reverted
            .events
            .iter()
            .any(|event| event.id == changed_event.id && event.reverted_at.is_some()));

        let task = db
            .create_task(
                &created.id,
                CreateTaskInput {
                    title: "准备面试".into(),
                    description: None,
                    priority: 2,
                    due_at: Some("2026-07-20T02:00:00.000Z".into()),
                    remind_at: Some("2026-07-20T01:00:00.000Z".into()),
                    application_stage: Some("HR面试".into()),
                },
            )
            .unwrap();
        assert_eq!(
            db.list_due_task_reminders("2026-07-20T01:01:00.000Z")
                .unwrap()
                .len(),
            1
        );
        db.mark_task_reminder_delivered(&task.id, "2026-07-20T01:01:00.000Z")
            .unwrap();
        assert!(db
            .list_due_task_reminders("2026-07-20T01:02:00.000Z")
            .unwrap()
            .is_empty());
        db.release_task_reminder_delivery(&task.id, "2026-07-20T01:01:00.000Z")
            .unwrap();
        assert_eq!(
            db.list_due_task_reminders("2026-07-20T01:02:00.000Z")
                .unwrap()
                .len(),
            1
        );
        db.mark_task_reminder_delivered(&task.id, "2026-07-20T01:02:00.000Z")
            .unwrap();

        let updated = db
            .update_task(
                &task.id,
                UpdateTaskInput {
                    title: "准备 HR 面试".into(),
                    description: Some("整理问题".into()),
                    priority: 3,
                    due_at: Some("2026-07-20T03:00:00.000Z".into()),
                    remind_at: Some("2026-07-20T02:00:00.000Z".into()),
                    application_stage: Some("HR面试".into()),
                },
            )
            .unwrap();
        assert_eq!(updated.title, "准备 HR 面试");
        assert_eq!(
            db.list_due_task_reminders("2026-07-20T02:01:00.000Z")
                .unwrap()
                .len(),
            1
        );
        db.delete_task(&task.id).unwrap();
        assert!(db
            .get_application_detail(&created.id)
            .unwrap()
            .tasks
            .is_empty());

        db.set_application_archived(&created.id, true).unwrap();
        assert!(
            db.list_applications()
                .unwrap()
                .into_iter()
                .find(|item| item.id == created.id)
                .unwrap()
                .archived
        );
        let dashboard = db
            .get_dashboard(
                "2026-07-01T00:00:00.000Z",
                "2026-08-01T00:00:00.000Z",
                "2026-07-20T00:00:00.000Z",
                "2026-07-21T00:00:00.000Z",
            )
            .unwrap();
        assert_eq!(dashboard.summary.total, 0);
        db.set_application_archived(&created.id, false).unwrap();
        assert!(db
            .get_application_detail(&created.id)
            .unwrap()
            .archived_at
            .is_none());
    }

    #[test]
    fn only_archived_applications_can_be_soft_deleted_with_an_event() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        assert!(db.delete_archived_application(&created.id).is_err());
        assert_eq!(db.list_applications().unwrap().len(), 1);

        db.set_application_archived(&created.id, true).unwrap();
        db.delete_archived_application(&created.id).unwrap();
        assert!(db.list_applications().unwrap().is_empty());
        assert!(db.get_application_detail(&created.id).is_err());

        let connection = db.connection.lock().unwrap();
        let (deleted, deletion_events): (bool, i64) = connection
            .query_row(
                "SELECT a.deleted_at IS NOT NULL,
                        (SELECT COUNT(*) FROM application_events e WHERE e.application_id=a.id AND e.event_type='application_deleted')
                 FROM applications a WHERE a.id=?1",
                [&created.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(deleted);
        assert_eq!(deletion_events, 1);
    }

    #[test]
    fn stage_events_can_be_reverted_out_of_order_and_excel_contains_history() {
        let db = Database::in_memory().unwrap();
        let created = db.create_application(input()).unwrap();
        db.update_application_stage(&created.id, "等待结果")
            .unwrap();
        {
            let connection = db.connection.lock().unwrap();
            connection.execute(
                "UPDATE application_events SET happened_at=strftime('%Y-%m-%dT%H:%M:%fZ','now','-10 minutes'), updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now','-10 minutes'), recorded_at=strftime('%Y-%m-%dT%H:%M:%fZ','now','-10 minutes') WHERE application_id=?1 AND event_type='stage_changed'",
                [&created.id],
            ).unwrap();
        }
        db.update_application_stage(&created.id, "HR面试").unwrap();
        let detail = db.get_application_detail(&created.id).unwrap();
        let waiting_event = detail
            .events
            .iter()
            .find(|event| event.stage_after.as_deref() == Some("等待结果"))
            .unwrap();
        let interview_event = detail
            .events
            .iter()
            .find(|event| event.stage_after.as_deref() == Some("HR面试"))
            .unwrap();

        let calendar_before = db
            .get_dashboard(
                "2000-01-01T00:00:00.000Z",
                "2100-01-01T00:00:00.000Z",
                "2026-07-14T00:00:00.000Z",
                "2026-07-15T00:00:00.000Z",
            )
            .unwrap();
        assert_eq!(
            calendar_before
                .events
                .iter()
                .filter(|item| item.kind == "milestone")
                .count(),
            3
        );

        let after_older_revert = db.revert_application_event(&waiting_event.id).unwrap();
        assert_eq!(after_older_revert.current_stage, "HR面试");
        let calendar_after_older_revert = db
            .get_dashboard(
                "2000-01-01T00:00:00.000Z",
                "2100-01-01T00:00:00.000Z",
                "2026-07-14T00:00:00.000Z",
                "2026-07-15T00:00:00.000Z",
            )
            .unwrap();
        assert_eq!(
            calendar_after_older_revert
                .events
                .iter()
                .filter(|item| item.kind == "milestone")
                .count(),
            2
        );
        let after_latest_revert = db.revert_application_event(&interview_event.id).unwrap();
        assert_eq!(after_latest_revert.current_stage, "已投递");
        let calendar_after_all_reverts = db
            .get_dashboard(
                "2000-01-01T00:00:00.000Z",
                "2100-01-01T00:00:00.000Z",
                "2026-07-14T00:00:00.000Z",
                "2026-07-15T00:00:00.000Z",
            )
            .unwrap();
        assert_eq!(
            calendar_after_all_reverts
                .events
                .iter()
                .filter(|item| item.kind == "milestone")
                .count(),
            1
        );

        let path = std::env::temp_dir().join(format!("applied-yet-export-{}.xls", Uuid::new_v4()));
        assert_eq!(
            db.export_applications_excel(path.to_str().unwrap())
                .unwrap(),
            1
        );
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("投递记录"));
        assert!(content.contains("流程与状态变更"));
        assert!(content.contains("事件时间线"));
        assert!(content.contains("创建投递"));
        assert!(content.contains("等待结果"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn provider_settings_use_defaults_validate_and_persist() {
        let db = Database::in_memory().unwrap();
        let defaults = db.get_provider_settings().unwrap();
        assert_eq!(defaults.ai.model, "gpt-4.1-mini");
        assert_eq!(defaults.asr.language, "zh");

        let ai = AiProviderSettings {
            provider: "兼容服务".into(),
            protocol: "responses".into(),
            base_url: "https://ai.example.com/v1".into(),
            model: "example-model".into(),
            fallback_model: Some("fallback-model".into()),
            max_output_tokens: 2048,
            timeout_seconds: 45,
            allow_resume: true,
            allow_transcript: true,
            prompt_before_send: true,
        };
        db.save_ai_settings(ai.clone()).unwrap();

        let asr = AsrProviderSettings {
            provider: "兼容服务".into(),
            base_url: "https://asr.example.com/v1".into(),
            model: "transcribe-model".into(),
            language: "auto".into(),
            speaker_diarization: true,
            segment_seconds: 600,
            file_limit_mb: 800,
            keep_original_audio: false,
            delete_temporary_files: true,
        };
        db.save_asr_settings(asr.clone()).unwrap();

        let saved = db.get_provider_settings().unwrap();
        assert_eq!(saved.ai.model, ai.model);
        assert_eq!(saved.ai.base_url, ai.base_url);
        assert!(saved.ai.allow_resume);
        assert!(saved.ai.prompt_before_send);
        assert_eq!(saved.asr.provider, asr.provider);
        assert_eq!(saved.asr.segment_seconds, 600);

        let mut invalid = ai;
        invalid.base_url = "http://remote.example.com/v1".into();
        assert!(db.save_ai_settings(invalid).is_err());
        let invalid_protocol = AiProviderSettings {
            protocol: "unsupported".into(),
            ..AiProviderSettings::default()
        };
        assert!(db.save_ai_settings(invalid_protocol).is_err());
        assert!(!is_allowed_provider_url("http://localhost.evil.example/v1"));
        assert!(!is_allowed_provider_url(
            "http://localhost:11434@evil.com/v1"
        ));
        assert!(is_allowed_provider_url("http://localhost:11434/v1"));
        assert!(clean_external_url(Some("javascript:alert(1)".into()), "链接").is_err());
        assert_eq!(
            clean_external_url(Some("https://example.com/jobs".into()), "链接").unwrap(),
            Some("https://example.com/jobs".into())
        );
        assert!(valid_network_host("imap.example.com"));
        assert!(valid_network_host("::1"));
        assert!(!valid_network_host("https://imap.example.com"));
        assert!(!valid_network_host("bad host"));
        assert!(db
            .save_email_settings(EmailSettings {
                imap_host: "imap.example.com".into(),
                use_tls: false,
                ..EmailSettings::default()
            })
            .is_err());
    }

    #[test]
    fn ai_audit_and_preparation_are_persisted_with_sources() {
        let db = Database::in_memory().unwrap();
        let application = db.create_application(input()).unwrap();
        let sources = r#"[{"type":"job_description","characters":120}]"#;
        let call_id = db
            .start_ai_call(
                Some(&application.id),
                "interview_preparation",
                "openai-compatible",
                "test-model",
                sources,
            )
            .unwrap();
        db.finish_ai_call(
            &call_id,
            "succeeded",
            2,
            345,
            Some(r#"{"summary":"准备建议"}"#),
            None,
        )
        .unwrap();
        let stored = db
            .save_interview_preparation(
                &application.id,
                &call_id,
                r#"{"summary":"准备建议"}"#,
                sources,
                "test-model",
            )
            .unwrap();
        assert_eq!(stored.ai_call_id, call_id);
        let latest = db
            .latest_interview_preparation(&application.id)
            .unwrap()
            .unwrap();
        assert_eq!(latest.content["summary"], "准备建议");
        let calls = db.list_ai_calls(&application.id).unwrap();
        assert_eq!(calls[0].attempts, 2);
        assert_eq!(calls[0].duration_ms, Some(345));
        assert_eq!(calls[0].input_sources[0]["type"], "job_description");
    }

    #[test]
    fn resume_profiles_support_multiple_versions_editing_and_primary() {
        let db = Database::in_memory().unwrap();
        let first = db
            .create_resume_profile(CreateResumeProfileInput {
                name: "后端简历".into(),
                file_path: None,
                file_format: Some("txt".into()),
                parsed_text: Some("原文".into()),
                personal_info: Some("张三".into()),
                education_background: Some("某大学".into()),
                internship_experience: None,
                project_experience: Some("订单项目".into()),
                professional_skills: Some("Rust".into()),
                academic_achievements: None,
                skill_certificates: Some("英语六级".into()),
                target_direction: Some("后端开发".into()),
                notes: None,
                parent_profile_id: None,
                is_primary: true,
            })
            .unwrap();
        let second = db
            .create_resume_profile(CreateResumeProfileInput {
                name: "产品简历".into(),
                file_path: None,
                file_format: None,
                parsed_text: None,
                personal_info: None,
                education_background: None,
                internship_experience: Some("实习".into()),
                project_experience: None,
                professional_skills: None,
                academic_achievements: None,
                skill_certificates: None,
                target_direction: None,
                notes: None,
                parent_profile_id: None,
                is_primary: false,
            })
            .unwrap();
        assert_eq!(db.list_resume_profiles().unwrap().len(), 2);
        db.set_primary_resume_profile(&second.id).unwrap();
        let edited = db
            .update_resume_profile(
                &second.id,
                UpdateResumeProfileInput {
                    name: "产品简历（更新）".into(),
                    personal_info: "李四".into(),
                    education_background: "大学".into(),
                    internship_experience: "实习".into(),
                    project_experience: "项目".into(),
                    professional_skills: "产品分析".into(),
                    academic_achievements: "论文".into(),
                    skill_certificates: "证书".into(),
                    target_direction: "产品经理".into(),
                    notes: "强调用户研究".into(),
                },
            )
            .unwrap();
        assert_eq!(edited.name, "产品简历（更新）");
        assert!(!db.get_resume_profile(&first.id).unwrap().is_primary);
        assert!(db.get_resume_profile(&second.id).unwrap().is_primary);
        db.delete_resume_profile(&first.id).unwrap();
        assert_eq!(db.list_resume_profiles().unwrap().len(), 1);
    }

    #[test]
    fn resume_stage_counts_include_historical_reach() {
        let db = Database::in_memory().unwrap();
        let resume = db
            .create_resume_profile(CreateResumeProfileInput {
                name: "历史阶段简历".into(),
                file_path: None,
                file_format: None,
                parsed_text: None,
                personal_info: None,
                education_background: None,
                internship_experience: None,
                project_experience: None,
                professional_skills: None,
                academic_achievements: None,
                skill_certificates: None,
                target_direction: None,
                notes: None,
                parent_profile_id: None,
                is_primary: true,
            })
            .unwrap();
        let mut application_input = input();
        application_input.resume_profile_id = Some(resume.id.clone());
        let application = db.create_application(application_input).unwrap();
        {
            let connection = db.connection.lock().unwrap();
            connection
                .execute(
                    "UPDATE applications SET current_stage='已获Offer' WHERE id=?1",
                    [&application.id],
                )
                .unwrap();
            connection.execute(
                "INSERT INTO application_events(id,application_id,event_type,title,source_type,stage_before,stage_after,happened_at) VALUES (?1,?2,'stage_changed','进入测评','manual','已投递','在线测评','2026-07-15T00:00:00Z')",
                params![Uuid::new_v4().to_string(), application.id],
            ).unwrap();
            connection.execute(
                "INSERT INTO application_events(id,application_id,event_type,title,source_type,stage_before,stage_after,happened_at) VALUES (?1,?2,'stage_changed','进入面试','manual','在线测评','HR面试','2026-07-16T00:00:00Z')",
                params![Uuid::new_v4().to_string(), application.id],
            ).unwrap();
        }
        let profile = db.get_resume_profile(&resume.id).unwrap();
        assert_eq!(profile.assessment_count, 1);
        assert_eq!(profile.interview_count, 1);
        assert_eq!(profile.offer_count, 1);
    }

    #[test]
    fn linked_resume_is_updated_in_place_and_delete_unlinks_applications() {
        let db = Database::in_memory().unwrap();
        let resume = db
            .create_resume_profile(CreateResumeProfileInput {
                name: "后端简历".into(),
                file_path: None,
                file_format: Some("pdf".into()),
                parsed_text: Some("简历原文".into()),
                personal_info: Some("{\"name\":\"张三\"}".into()),
                education_background: Some("[]".into()),
                internship_experience: Some("[]".into()),
                project_experience: Some("[{\"name\":\"订单系统\"}]".into()),
                professional_skills: Some("Rust".into()),
                academic_achievements: Some("[]".into()),
                skill_certificates: Some("[]".into()),
                target_direction: Some("后端开发".into()),
                notes: None,
                parent_profile_id: None,
                is_primary: true,
            })
            .unwrap();
        let mut application_input = input();
        application_input.resume_profile_id = Some(resume.id.clone());
        let application = db.create_application(application_input).unwrap();
        let detail = db.get_application_detail(&application.id).unwrap();
        assert_eq!(
            detail.resume_profile_id.as_deref(),
            Some(resume.id.as_str())
        );
        assert_eq!(
            db.get_ai_application_context(&application.id)
                .unwrap()
                .resume
                .unwrap()
                .name,
            "后端简历"
        );

        let updated = db
            .update_resume_profile(
                &resume.id,
                UpdateResumeProfileInput {
                    name: "后端简历 v2".into(),
                    personal_info: "{}".into(),
                    education_background: "[]".into(),
                    internship_experience: "[]".into(),
                    project_experience: "[]".into(),
                    professional_skills: "Rust, SQL".into(),
                    academic_achievements: "[]".into(),
                    skill_certificates: "[]".into(),
                    target_direction: "后端开发".into(),
                    notes: "针对新岗位优化".into(),
                },
            )
            .unwrap();
        assert_eq!(updated.id, resume.id);
        assert_eq!(updated.professional_skills, "Rust, SQL");
        assert_eq!(
            db.get_application_detail(&application.id)
                .unwrap()
                .resume_profile_id
                .as_deref(),
            Some(resume.id.as_str())
        );

        db.delete_resume_profile(&resume.id).unwrap();
        assert!(db.get_resume_profile(&resume.id).is_err());
        assert!(db
            .get_application_detail(&application.id)
            .unwrap()
            .resume_profile_id
            .is_none());
    }

    #[test]
    fn editing_a_linked_non_primary_resume_preserves_the_primary_resume() {
        let db = Database::in_memory().unwrap();
        let primary = db
            .create_resume_profile(CreateResumeProfileInput {
                name: "默认简历".into(),
                file_path: None,
                file_format: None,
                parsed_text: None,
                personal_info: None,
                education_background: None,
                internship_experience: None,
                project_experience: None,
                professional_skills: None,
                academic_achievements: None,
                skill_certificates: None,
                target_direction: None,
                notes: None,
                parent_profile_id: None,
                is_primary: true,
            })
            .unwrap();
        let secondary = db
            .create_resume_profile(CreateResumeProfileInput {
                name: "专项简历".into(),
                file_path: None,
                file_format: None,
                parsed_text: None,
                personal_info: None,
                education_background: None,
                internship_experience: None,
                project_experience: None,
                professional_skills: None,
                academic_achievements: None,
                skill_certificates: None,
                target_direction: None,
                notes: None,
                parent_profile_id: None,
                is_primary: false,
            })
            .unwrap();
        let mut application_input = input();
        application_input.resume_profile_id = Some(secondary.id.clone());
        db.create_application(application_input).unwrap();

        let fork = db
            .update_resume_profile(
                &secondary.id,
                UpdateResumeProfileInput {
                    name: "专项简历 v2".into(),
                    personal_info: String::new(),
                    education_background: String::new(),
                    internship_experience: String::new(),
                    project_experience: String::new(),
                    professional_skills: String::new(),
                    academic_achievements: String::new(),
                    skill_certificates: String::new(),
                    target_direction: String::new(),
                    notes: String::new(),
                },
            )
            .unwrap();

        assert!(db.get_resume_profile(&primary.id).unwrap().is_primary);
        assert!(!fork.is_primary);
    }

    #[test]
    fn ai_context_excludes_a_soft_deleted_resume() {
        let db = Database::in_memory().unwrap();
        let resume = db
            .create_resume_profile(CreateResumeProfileInput {
                name: "待删除简历".into(),
                file_path: None,
                file_format: None,
                parsed_text: None,
                personal_info: Some("敏感资料".into()),
                education_background: None,
                internship_experience: None,
                project_experience: None,
                professional_skills: None,
                academic_achievements: None,
                skill_certificates: None,
                target_direction: None,
                notes: None,
                parent_profile_id: None,
                is_primary: false,
            })
            .unwrap();
        let mut application_input = input();
        application_input.resume_profile_id = Some(resume.id.clone());
        let application = db.create_application(application_input).unwrap();
        {
            let connection = db.connection.lock().unwrap();
            connection
                .execute(
                    "UPDATE resume_profiles SET deleted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",
                    [&resume.id],
                )
                .unwrap();
        }

        assert!(db
            .get_ai_application_context(&application.id)
            .unwrap()
            .resume
            .is_none());
    }
}

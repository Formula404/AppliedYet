use crate::db::{
    AnalyticsData, ApplicationDetail, ApplicationEvent, ApplicationListItem, ApplicationTask,
    CreateApplicationInput, CreateEventInput, CreateTaskInput, DashboardData, Database,
    DueTaskReminder, UpdateApplicationDetailInput, UpdateTaskInput,
};

#[tauri::command]
pub(crate) fn list_applications(
    db: tauri::State<'_, Database>,
) -> Result<Vec<ApplicationListItem>, String> {
    db.list_applications()
}

#[tauri::command]
pub(crate) fn get_activity_summary(
    db: tauri::State<'_, Database>,
) -> Result<crate::db::ActivitySummary, String> {
    db.get_activity_summary()
}
#[tauri::command]
pub(crate) fn get_analytics(db: tauri::State<'_, Database>) -> Result<AnalyticsData, String> {
    db.get_analytics()
}
#[tauri::command]
pub(crate) fn export_applications_excel(
    db: tauri::State<'_, Database>,
    path: String,
) -> Result<usize, String> {
    db.export_applications_excel(&path)
}
#[tauri::command]
pub(crate) fn create_application(
    db: tauri::State<'_, Database>,
    input: CreateApplicationInput,
) -> Result<ApplicationListItem, String> {
    db.create_application(input)
}
#[tauri::command]
pub(crate) fn update_application_stage(
    db: tauri::State<'_, Database>,
    id: String,
    stage: String,
) -> Result<(), String> {
    db.update_application_stage(&id, &stage)
}
#[tauri::command]
pub(crate) fn get_application_detail(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<ApplicationDetail, String> {
    db.get_application_detail(&id)
}
#[tauri::command]
pub(crate) fn update_application_detail(
    db: tauri::State<'_, Database>,
    id: String,
    input: UpdateApplicationDetailInput,
) -> Result<ApplicationDetail, String> {
    db.update_application_detail(&id, input)
}
#[tauri::command]
pub(crate) fn create_application_task(
    db: tauri::State<'_, Database>,
    application_id: String,
    input: CreateTaskInput,
) -> Result<ApplicationTask, String> {
    db.create_task(&application_id, input)
}
#[tauri::command]
pub(crate) fn set_application_task_status(
    db: tauri::State<'_, Database>,
    task_id: String,
    status: String,
) -> Result<ApplicationTask, String> {
    db.set_task_status(&task_id, &status)
}
#[tauri::command]
pub(crate) fn update_application_task(
    db: tauri::State<'_, Database>,
    task_id: String,
    input: UpdateTaskInput,
) -> Result<ApplicationTask, String> {
    db.update_task(&task_id, input)
}
#[tauri::command]
pub(crate) fn delete_application_task(
    db: tauri::State<'_, Database>,
    task_id: String,
) -> Result<(), String> {
    db.delete_task(&task_id)
}
#[tauri::command]
pub(crate) fn create_application_event(
    db: tauri::State<'_, Database>,
    application_id: String,
    input: CreateEventInput,
) -> Result<ApplicationEvent, String> {
    db.create_event(&application_id, input)
}
#[tauri::command]
pub(crate) fn revert_application_event(
    db: tauri::State<'_, Database>,
    event_id: String,
) -> Result<ApplicationDetail, String> {
    db.revert_application_event(&event_id)
}
#[tauri::command]
pub(crate) fn update_application_event_time(
    db: tauri::State<'_, Database>,
    event_id: String,
    happened_at: String,
) -> Result<ApplicationDetail, String> {
    db.update_application_event_time(&event_id, &happened_at)
}
#[tauri::command]
pub(crate) fn set_application_archived(
    db: tauri::State<'_, Database>,
    id: String,
    archived: bool,
) -> Result<(), String> {
    db.set_application_archived(&id, archived)
}
#[tauri::command]
pub(crate) fn delete_archived_application(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_archived_application(&id)
}
#[tauri::command]
pub(crate) fn get_dashboard(
    db: tauri::State<'_, Database>,
    month_start: String,
    month_end: String,
    today_start: String,
    today_end: String,
) -> Result<DashboardData, String> {
    db.get_dashboard(&month_start, &month_end, &today_start, &today_end)
}
#[tauri::command]
pub(crate) fn list_due_task_reminders(
    db: tauri::State<'_, Database>,
    now: String,
) -> Result<Vec<DueTaskReminder>, String> {
    db.list_due_task_reminders(&now)
}
#[tauri::command]
pub(crate) fn mark_task_reminder_delivered(
    db: tauri::State<'_, Database>,
    task_id: String,
    notified_at: String,
) -> Result<(), String> {
    db.mark_task_reminder_delivered(&task_id, &notified_at)
}
#[tauri::command]
pub(crate) fn release_task_reminder_delivery(
    db: tauri::State<'_, Database>,
    task_id: String,
    notified_at: String,
) -> Result<(), String> {
    db.release_task_reminder_delivery(&task_id, &notified_at)
}

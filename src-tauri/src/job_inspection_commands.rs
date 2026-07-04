use crate::job_inspector;
use crate::models::{MetricCard, MySqlSettings};

#[tauri::command]
pub fn etl_get_recent_steps(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    job_inspector::recent_steps(&settings, &import_batch_id, 30)
}

#[tauri::command]
pub fn etl_get_failed_steps(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    job_inspector::failed_steps(&settings, &import_batch_id)
}

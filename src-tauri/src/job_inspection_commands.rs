use crate::job_inspector;
use crate::models::{EtlJobStepRow, EtlJobStepsRequest, MetricCard, MySqlSettings};

#[tauri::command]
pub fn etl_get_recent_steps(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    job_inspector::recent_steps(&settings, &import_batch_id, 30)
}

#[tauri::command]
pub fn etl_get_failed_steps(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    job_inspector::failed_steps(&settings, &import_batch_id)
}

#[tauri::command]
pub fn etl_get_job_steps(req: EtlJobStepsRequest) -> Result<Vec<EtlJobStepRow>, String> {
    job_inspector::job_step_rows(&req)
}

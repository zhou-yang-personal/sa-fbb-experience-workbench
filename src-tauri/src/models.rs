use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MySqlSettings {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub secret: String,
    pub local_infile: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandAck {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CsvProbeResult {
    pub path: String,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub sha256: String,
    pub delimiter: String,
    pub headers: Vec<String>,
    pub preview_rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportBatchResult {
    pub import_batch_id: String,
    pub batch_display_name: Option<String>,
    pub data_type: String,
    pub source_file_name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchListItem {
    pub import_batch_id: String,
    pub batch_display_name: Option<String>,
    pub data_type: String,
    pub source_file_name: String,
    pub status: String,
    pub total_rows: Option<i64>,
    pub imported_rows: Option<i64>,
    pub analysis_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchTableRegistryRow {
    pub import_batch_id: String,
    pub layer: String,
    pub data_type: String,
    pub logical_table_name: String,
    pub base_table_name: String,
    pub physical_table_name: String,
    pub row_count: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleStatusRow {
    pub import_batch_id: String,
    pub analysis_run_id: Option<String>,
    pub module_id: String,
    pub module_name: String,
    pub enabled: bool,
    pub data_type: Option<String>,
    pub missing_required_fields: Option<String>,
    pub missing_tables: Option<String>,
    pub row_count: i64,
    pub status_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBatchRequest {
    pub settings: MySqlSettings,
    pub data_type: String,
    pub file_path: String,
    pub batch_display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawLoadRequest {
    pub settings: MySqlSettings,
    pub import_batch_id: String,
    pub data_type: String,
    pub file_path: String,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EtlRequest {
    pub settings: MySqlSettings,
    pub import_batch_id: String,
    pub analysis_run_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EtlJobStepsRequest {
    pub settings: MySqlSettings,
    pub import_batch_id: String,
    pub job_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DashboardRequest {
    pub settings: MySqlSettings,
    pub import_batch_id: String,
    pub analysis_run_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleStatusRequest {
    pub settings: MySqlSettings,
    pub import_batch_id: String,
    pub data_type: Option<String>,
    pub analysis_run_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportModuleRequest {
    pub settings: MySqlSettings,
    pub import_batch_id: String,
    pub analysis_run_id: Option<String>,
    pub module_id: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LeadsQueryRequest {
    pub settings: MySqlSettings,
    pub analysis_run_id: String,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub lead_type: Option<String>,
    pub final_action: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportLeadsRequest {
    pub settings: MySqlSettings,
    pub analysis_run_id: String,
    pub output_path: String,
    pub final_actions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardOverview {
    pub metrics: Vec<MetricCard>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EtlJobStepRow {
    pub job_id: String,
    pub job_type: String,
    pub step_name: String,
    pub target_table: Option<String>,
    pub status: String,
    pub affected_rows: Option<i64>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeadUserRow {
    pub user_key: String,
    pub user_type: Option<String>,
    pub lead_type: String,
    pub demand_score: i32,
    pub migration_motive_score: i32,
    pub recommended_offer: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinalLeadUserRow {
    pub user_key: String,
    pub crm_user_id: Option<String>,
    pub lead_type: String,
    pub demand_score: i32,
    pub migration_motive_score: i32,
    pub current_plan_name: Option<String>,
    pub current_arpu: Option<f64>,
    pub ftth_available_flag: Option<String>,
    pub reachable_flag: Option<String>,
    pub final_action: Option<String>,
    pub recommended_offer: Option<String>,
}

pub fn ack(message: impl Into<String>) -> CommandAck {
    CommandAck {
        status: "ok".to_string(),
        message: message.into(),
    }
}

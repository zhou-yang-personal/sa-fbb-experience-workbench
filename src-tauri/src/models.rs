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
pub struct ImportCurrentFileRequest {
    pub settings: MySqlSettings,
    pub data_type: String,
    pub file_path: String,
    pub batch_display_name: String,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportCurrentFileResult {
    pub batch: ImportBatchResult,
    pub mapping_summary: Vec<MetricCard>,
    pub mapping_results: Vec<MetricCard>,
    pub raw_status: Vec<MetricCard>,
    pub profile: Vec<MetricCard>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportPipelineStartRequest {
    pub settings: MySqlSettings,
    pub data_type: String,
    pub file_path: String,
    pub batch_display_name: String,
    pub import_mode: Option<String>,
    pub analysis_run_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportPipelineStatusRequest {
    pub settings: MySqlSettings,
    pub pipeline_run_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportPipelineLogsRequest {
    pub settings: MySqlSettings,
    pub pipeline_run_id: String,
    pub after_sequence: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPipelineStartResult {
    pub pipeline_run_id: String,
    pub import_batch_id: Option<String>,
    pub analysis_run_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPipelineStepRow {
    pub step_index: i32,
    pub step_name: String,
    pub step_label: String,
    pub status: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub elapsed_ms: i64,
    pub message: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPipelineStatus {
    pub pipeline_run_id: String,
    pub status: String,
    pub current_step: Option<String>,
    pub percent: f64,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub elapsed_ms: i64,
    pub import_batch_id: Option<String>,
    pub analysis_run_id: Option<String>,
    pub failed_step: Option<String>,
    pub error_message: Option<String>,
    pub final_fusion_status: Option<String>,
    pub message: Option<String>,
    pub steps: Vec<ImportPipelineStepRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPipelineLogRow {
    pub sequence: i64,
    pub timestamp: String,
    pub level: String,
    pub step_name: Option<String>,
    pub message: String,
    pub elapsed_ms: i64,
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
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub keyword: Option<String>,
    pub sort_by: Option<String>,
    pub min_value: Option<f64>,
}

impl DashboardRequest {
    pub fn run_id(&self) -> String {
        self.analysis_run_id
            .clone()
            .unwrap_or_else(|| "RUN_DEFAULT".to_string())
    }

    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self, default_size: u64, max_size: u64) -> u64 {
        self.page_size.unwrap_or(default_size).max(1).min(max_size)
    }

    pub fn offset(&self, default_size: u64, max_size: u64) -> u64 {
        (self.page().saturating_sub(1)).saturating_mul(self.page_size(default_size, max_size))
    }

    pub fn keyword_like(&self) -> Option<String> {
        let keyword = self.keyword.as_deref()?.trim();
        if keyword.is_empty() {
            None
        } else {
            Some(format!("%{}%", keyword.replace('%', "\\%").replace('_', "\\_")))
        }
    }

    pub fn min_value(&self) -> f64 {
        self.min_value.unwrap_or(0.0).max(0.0)
    }

    pub fn sort_by(&self) -> String {
        self.sort_by
            .as_deref()
            .unwrap_or("default")
            .trim()
            .to_lowercase()
    }
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

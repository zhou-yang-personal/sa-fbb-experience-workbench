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
    pub data_type: String,
    pub source_file_name: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBatchRequest {
    pub settings: MySqlSettings,
    pub data_type: String,
    pub file_path: String,
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
pub struct DashboardRequest {
    pub settings: MySqlSettings,
    pub import_batch_id: String,
    pub analysis_run_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LeadsQueryRequest {
    pub settings: MySqlSettings,
    pub analysis_run_id: String,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportLeadsRequest {
    pub settings: MySqlSettings,
    pub analysis_run_id: String,
    pub output_path: String,
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

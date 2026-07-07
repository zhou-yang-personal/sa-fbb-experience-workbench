use crate::batch_tables;
use crate::models::{ack, CommandAck, EtlRequest};
use crate::sql_runner;

const SQL: &str = include_str!("../../database/sql/dws_to_ads/003b_analytics_app_rank.sql");

#[tauri::command]
pub fn analytics_materialize_app_rank(req: EtlRequest) -> Result<CommandAck, String> {
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let run_id = req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let bound = sql_runner::bind_batch_params(SQL, &req.import_batch_id, Some(&run_id));
    let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
    let rows = sql_runner::execute_script(&req.settings, &sql)?;
    Ok(ack(format!("analytics app rank materialized: analysis_run_id={run_id}; affected_rows={rows}")))
}

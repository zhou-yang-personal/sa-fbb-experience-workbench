use crate::{batch_tables, sql_runner};
use crate::models::{ack, CommandAck, EtlRequest};

const S: &str = include_str!("../../database/sql/dws_to_ads/003d_analytics_user_profile.sql");

#[tauri::command]
pub fn ads_user(req: EtlRequest) -> Result<CommandAck, String> {
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let run = req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let b = sql_runner::bind_batch_params(S, &req.import_batch_id, Some(&run));
    let s = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &b)?;
    let n = sql_runner::execute_script(&req.settings, &s)?;
    Ok(ack(format!("ads_user ok: analysis_run_id={run}; affected_rows={n}")))
}

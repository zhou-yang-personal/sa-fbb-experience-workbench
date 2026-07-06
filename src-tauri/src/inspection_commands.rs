use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard, MySqlSettings};

#[tauri::command]
pub fn quality_get_gate_results(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT check_name, check_status, COALESCE(check_message,'') FROM meta_quality_check_result WHERE import_batch_id=? ORDER BY check_status, check_name",
        (&import_batch_id,),
        |(check_name, check_status, check_message): (String, String, String)| MetricCard {
            label: check_name,
            value: check_status,
            hint: check_message,
        },
    ).map_err(|err| format!("failed to query quality gate results: {err}"))
}

#[tauri::command]
pub fn etl_get_recent_steps(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT CONCAT(j.job_type, ':', s.step_name), s.status, CONCAT('target=', COALESCE(s.target_table,''), ', affected_rows=', COALESCE(s.affected_rows,0), ', message=', COALESCE(s.message,'')) FROM meta_etl_job j JOIN meta_etl_job_step s ON s.job_id=j.job_id WHERE j.import_batch_id=? ORDER BY s.started_at DESC LIMIT 30",
        (&import_batch_id,),
        |(label, value, hint): (String, String, String)| MetricCard { label, value, hint },
    ).map_err(|err| format!("failed to query ETL job steps: {err}"))
}

#[tauri::command]
pub fn final_leads_get_action_mix(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.analysis_run_id.unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_final_marketing_lead_user")?;
    conn.exec_map(
        format!("SELECT COALESCE(final_action,'UNKNOWN'), COUNT(*), ROUND(AVG(demand_score),2) FROM `{table}` WHERE analysis_run_id=? GROUP BY COALESCE(final_action,'UNKNOWN') ORDER BY COUNT(*) DESC"),
        (&run_id,),
        |(action, count, avg_score): (String, u64, Option<f64>)| MetricCard {
            label: action,
            value: count.to_string(),
            hint: format!("avg_demand_score={:.2}", avg_score.unwrap_or(0.0)),
        },
    ).map_err(|err| format!("failed to query final lead action mix: {err}"))
}

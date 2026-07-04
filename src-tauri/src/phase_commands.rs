use mysql::prelude::*;

use crate::db;
use crate::final_fusion;
use crate::job_runner::{self, JobStep};
use crate::models::{ack, CommandAck, DashboardRequest, EtlRequest, MetricCard};
use crate::sql_runner;

const QUALITY_SQL: &str = include_str!("../../database/sql/quality/001_raw_quality_gate.sql");
const COMPLETE_DWS_SQL: &str = include_str!("../../database/sql/clean_to_dws/002_complete_aggregates.sql");
const COMPLETE_DASHBOARD_SQL: &str = include_str!("../../database/sql/dws_to_ads/002_complete_dashboards.sql");

#[tauri::command]
pub fn quality_run_gate(req: EtlRequest) -> Result<CommandAck, String> {
    let sql = sql_runner::bind_batch_params(QUALITY_SQL, &req.import_batch_id, None);
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "quality_gate", vec![
        JobStep { step_name: "raw_quality_gate", source_table: "raw_tcp_detail_import,raw_game_detail_import", target_table: "meta_quality_check_result", sql_template: "001_raw_quality_gate.sql", sql },
    ])?;
    Ok(ack(message))
}

#[tauri::command]
pub fn etl_run_complete_aggregates(req: EtlRequest) -> Result<CommandAck, String> {
    let sql = sql_runner::bind_batch_params(COMPLETE_DWS_SQL, &req.import_batch_id, None);
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "complete_dws_aggregate", vec![
        JobStep { step_name: "complete_dws_aggregates", source_table: "dwd_tcp_detail_clean,dwd_game_detail_clean,dws_user_daily_profile", target_table: "dws_app_category_daily,dws_access_type_hourly_compare,dws_user_experience_bottleneck", sql_template: "002_complete_aggregates.sql", sql },
    ])?;
    Ok(ack(message))
}

#[tauri::command]
pub fn ads_run_complete_dashboards(req: EtlRequest) -> Result<CommandAck, String> {
    let run_id = req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let sql = sql_runner::bind_batch_params(COMPLETE_DASHBOARD_SQL, &req.import_batch_id, Some(&run_id));
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "complete_ads_dashboard", vec![
        JobStep { step_name: "complete_ads_dashboards", source_table: "dws_user_daily_profile,dws_app_category_daily,dws_access_type_hourly_compare", target_table: "ads_dashboard_overview,ads_app_category_detail,ads_experience_quality_summary,ads_cable_fiber_compare", sql_template: "002_complete_dashboards.sql", sql },
    ])?;
    Ok(ack(format!("analysis_run_id={run_id}; {message}")))
}

#[tauri::command]
pub fn leads_run_final_fusion(req: EtlRequest) -> Result<CommandAck, String> {
    let run_id = req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let step = final_fusion::build_final_fusion_step(&req.settings, &req.import_batch_id, &run_id)?;
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "final_lead_fusion", vec![step])?;
    Ok(ack(format!("analysis_run_id={run_id}; {message}")))
}

#[tauri::command]
pub fn dashboard_get_app_category(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.analysis_run_id.unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    conn.exec_map(
        "SELECT app_category, CAST(SUM(active_users) AS SIGNED), CAST(ROUND(SUM(total_download_gb),2) AS DOUBLE) FROM ads_app_category_detail WHERE analysis_run_id=? GROUP BY app_category ORDER BY SUM(active_users) DESC LIMIT 20",
        (&run_id,),
        |(category, users, gb): (String, i64, f64)| MetricCard { label: category, value: users.to_string(), hint: format!("download_gb={gb}") }
    ).map_err(|err| format!("failed to query app category detail: {err}"))
}

#[tauri::command]
pub fn dashboard_get_experience_quality(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.analysis_run_id.unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    conn.exec_map(
        "SELECT quality_dimension, COALESCE(user_type,'ALL'), CAST(ROUND(COALESCE(avg_value,0),2) AS DOUBLE), severity FROM ads_experience_quality_summary WHERE analysis_run_id=? ORDER BY quality_dimension, user_type",
        (&run_id,),
        |(dimension, user_type, avg_value, severity): (String, String, f64, Option<String>)| MetricCard { label: format!("{dimension} {user_type}"), value: format!("{avg_value}"), hint: severity.unwrap_or_default() }
    ).map_err(|err| format!("failed to query experience quality: {err}"))
}

#[tauri::command]
pub fn dashboard_get_cable_fiber_compare(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.analysis_run_id.unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    conn.exec_map(
        "SELECT metric_key, CAST(ROUND(AVG(cable_value),2) AS DOUBLE), CAST(ROUND(AVG(ftth_value),2) AS DOUBLE), CAST(ROUND(AVG(diff_value),2) AS DOUBLE) FROM ads_cable_fiber_compare WHERE analysis_run_id=? GROUP BY metric_key ORDER BY metric_key",
        (&run_id,),
        |(metric, cable, ftth, diff): (String, Option<f64>, Option<f64>, Option<f64>)| MetricCard { label: metric, value: format!("diff={:.2}", diff.unwrap_or(0.0)), hint: format!("cable={:.2}, ftth={:.2}", cable.unwrap_or(0.0), ftth.unwrap_or(0.0)) }
    ).map_err(|err| format!("failed to query cable fiber compare: {err}"))
}

#[tauri::command]
pub fn leads_get_final_summary(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.analysis_run_id.unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    conn.exec_map(
        "SELECT COALESCE(final_action,'UNKNOWN'), CAST(COUNT(*) AS SIGNED) FROM ads_final_marketing_lead_user WHERE analysis_run_id=? GROUP BY COALESCE(final_action,'UNKNOWN') ORDER BY COUNT(*) DESC",
        (&run_id,),
        |(action, count): (String, i64)| MetricCard { label: action, value: count.to_string(), hint: "final marketing lead action".to_string() }
    ).map_err(|err| format!("failed to query final lead summary: {err}"))
}

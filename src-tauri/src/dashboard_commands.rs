use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardOverview, DashboardRequest, MetricCard, MySqlSettings};

#[tauri::command]
pub fn db_test_connection(settings: MySqlSettings) -> Result<crate::models::CommandAck, String> {
    db::ping(&settings).map(crate::models::ack)
}

#[tauri::command]
pub fn db_initialize(settings: MySqlSettings) -> Result<crate::models::CommandAck, String> {
    crate::migrations::init_database(&settings).map(crate::models::ack)
}

#[tauri::command]
pub fn quality_get_batch_report(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    let raw_tcp = batch_tables::resolve_table(&settings, &import_batch_id, "raw_tcp_detail_import")?;
    let raw_game = batch_tables::resolve_table(&settings, &import_batch_id, "raw_game_detail_import")?;
    let clean_tcp = batch_tables::resolve_table(&settings, &import_batch_id, "dwd_tcp_detail_clean")?;
    let clean_game = batch_tables::resolve_table(&settings, &import_batch_id, "dwd_game_detail_clean")?;
    let tcp_rows: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{raw_tcp}`"), ()).map_err(|err| err.to_string())?;
    let game_rows: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{raw_game}`"), ()).map_err(|err| err.to_string())?;
    let clean_video_rows: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{clean_tcp}`"), ()).map_err(|err| err.to_string())?;
    let clean_game_rows: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{clean_game}`"), ()).map_err(|err| err.to_string())?;
    Ok(vec![
        MetricCard { label: "RAW TCP rows".to_string(), value: tcp_rows.unwrap_or(0).to_string(), hint: "raw_tcp_detail_import".to_string() },
        MetricCard { label: "RAW Game rows".to_string(), value: game_rows.unwrap_or(0).to_string(), hint: "raw_game_detail_import".to_string() },
        MetricCard { label: "Clean TCP rows".to_string(), value: clean_video_rows.unwrap_or(0).to_string(), hint: "dwd_tcp_detail_clean".to_string() },
        MetricCard { label: "Clean Game rows".to_string(), value: clean_game_rows.unwrap_or(0).to_string(), hint: "dwd_game_detail_clean".to_string() },
    ])
}

#[tauri::command]
pub fn dashboard_get_overview(req: DashboardRequest) -> Result<DashboardOverview, String> {
    let mut conn = db::conn(&req.settings)?;
    let dws_user = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?;
    let ads_lead = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_migration_lead_user")?;
    let users: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(DISTINCT user_key) AS SIGNED) FROM `{dws_user}` WHERE import_batch_id=?"), (&req.import_batch_id,)).map_err(|err| err.to_string())?;
    let download_gb: Option<f64> = conn.exec_first(format!("SELECT CAST(COALESCE(SUM(total_download_gb),0) AS DOUBLE) FROM `{dws_user}` WHERE import_batch_id=?"), (&req.import_batch_id,)).map_err(|err| err.to_string())?;
    let game_hours: Option<f64> = conn.exec_first(format!("SELECT CAST(COALESCE(SUM(total_game_hours),0) AS DOUBLE) FROM `{dws_user}` WHERE import_batch_id=?"), (&req.import_batch_id,)).map_err(|err| err.to_string())?;
    let a1_users: Option<i64> = if let Some(run_id) = &req.analysis_run_id { conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{ads_lead}` WHERE analysis_run_id=? AND lead_type LIKE 'A1_%'"), (run_id,)).map_err(|err| err.to_string())? } else { Some(0) };
    Ok(DashboardOverview { metrics: vec![
        MetricCard { label: "Clean users".to_string(), value: users.unwrap_or(0).to_string(), hint: "DWS distinct users".to_string() },
        MetricCard { label: "Video GB".to_string(), value: format!("{:.2}", download_gb.unwrap_or(0.0)), hint: "sum total_download_gb".to_string() },
        MetricCard { label: "Game hours".to_string(), value: format!("{:.2}", game_hours.unwrap_or(0.0)), hint: "sum total_game_hours".to_string() },
        MetricCard { label: "A1 leads".to_string(), value: a1_users.unwrap_or(0).to_string(), hint: "priority marketing users".to_string() },
    ]})
}

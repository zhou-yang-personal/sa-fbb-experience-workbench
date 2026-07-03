#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod migrations;
mod models;
mod phase_commands;
mod probe;
mod sql_runner;

use mysql::prelude::*;
use uuid::Uuid;

use models::{
    ack, CommandAck, CreateBatchRequest, DashboardOverview, DashboardRequest, EtlRequest,
    ExportLeadsRequest, ImportBatchResult, LeadUserRow, LeadsQueryRequest, MetricCard,
    MySqlSettings, RawLoadRequest,
};

const TCP_CLEAN_SQL: &str = include_str!("../../database/sql/raw_to_clean/001_tcp_raw_to_clean.sql");
const GAME_CLEAN_SQL: &str = include_str!("../../database/sql/raw_to_clean/002_game_raw_to_clean.sql");
const USER_DAILY_SQL: &str = include_str!("../../database/sql/clean_to_dws/001_user_daily_profile.sql");
const LEADS_SQL: &str = include_str!("../../database/sql/dws_to_ads/001_migration_leads.sql");

#[tauri::command]
fn db_test_connection(settings: MySqlSettings) -> Result<CommandAck, String> { db::ping(&settings).map(ack) }

#[tauri::command]
fn db_initialize(settings: MySqlSettings) -> Result<CommandAck, String> { migrations::init_database(&settings).map(ack) }

#[tauri::command]
fn import_probe_csv(path: String) -> Result<models::CsvProbeResult, String> { probe::probe_file(path) }

#[tauri::command]
fn import_create_batch(req: CreateBatchRequest) -> Result<ImportBatchResult, String> {
    let import_batch_id = format!("BATCH_{}", Uuid::new_v4().simple());
    let source_file_name = req.file_path.clone();
    let mut conn = db::conn(&req.settings)?;
    conn.exec_drop(
        "INSERT INTO meta_import_batch (import_batch_id, data_type, source_file_name, source_file_path, status) VALUES (?, ?, ?, ?, 'pending')",
        (&import_batch_id, &req.data_type, &source_file_name, &req.file_path),
    ).map_err(|err| format!("failed to create import batch: {err}"))?;
    Ok(ImportBatchResult { import_batch_id, data_type: req.data_type, source_file_name, status: "pending".to_string() })
}

#[tauri::command]
fn import_start_raw_load(req: RawLoadRequest) -> Result<CommandAck, String> {
    let table = match req.data_type.as_str() { "tcp" => "raw_tcp_detail_import", "game" => "raw_game_detail_import", other => return Err(format!("unsupported data type: {other}")) };
    let mut conn = db::conn(&req.settings)?;
    conn.exec_drop("UPDATE meta_import_batch SET status='running', started_at=NOW(), message='raw load started' WHERE import_batch_id=?", (&req.import_batch_id,)).map_err(|err| format!("failed to update import batch status: {err}"))?;
    let path = sql_runner::escape_sql_literal(&req.file_path.replace('\\', "/"));
    let batch_id = sql_runner::escape_sql_literal(&req.import_batch_id);
    let load_keyword = "LOAD";
    let sql = format!("{load_keyword} DATA LOCAL INFILE '{path}' INTO TABLE {table} CHARACTER SET utf8mb4 FIELDS TERMINATED BY ',' ENCLOSED BY '\"' LINES TERMINATED BY '\n' IGNORE 1 LINES SET import_batch_id='{batch_id}', source_file_name='{path}', source_line_no=NULL");
    conn.query_drop(sql).map_err(|err| format!("raw load failed; check MySQL local_infile and CSV column order: {err}"))?;
    let rows = conn.affected_rows();
    conn.exec_drop("UPDATE meta_import_batch SET status='success', imported_rows=?, finished_at=NOW(), message='raw load finished' WHERE import_batch_id=?", (rows, &req.import_batch_id)).map_err(|err| format!("failed to finalize import batch: {err}"))?;
    Ok(ack(format!("raw load finished: table={table}, rows={rows}")))
}

#[tauri::command]
fn quality_get_batch_report(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    let tcp_rows: Option<u64> = conn.exec_first("SELECT COUNT(*) FROM raw_tcp_detail_import WHERE import_batch_id=?", (&import_batch_id,)).map_err(|err| err.to_string())?;
    let game_rows: Option<u64> = conn.exec_first("SELECT COUNT(*) FROM raw_game_detail_import WHERE import_batch_id=?", (&import_batch_id,)).map_err(|err| err.to_string())?;
    let clean_video_rows: Option<u64> = conn.exec_first("SELECT COUNT(*) FROM dwd_tcp_detail_clean WHERE import_batch_id=?", (&import_batch_id,)).map_err(|err| err.to_string())?;
    let clean_game_rows: Option<u64> = conn.exec_first("SELECT COUNT(*) FROM dwd_game_detail_clean WHERE import_batch_id=?", (&import_batch_id,)).map_err(|err| err.to_string())?;
    Ok(vec![
        MetricCard { label: "RAW TCP rows".to_string(), value: tcp_rows.unwrap_or(0).to_string(), hint: "raw_tcp_detail_import".to_string() },
        MetricCard { label: "RAW Game rows".to_string(), value: game_rows.unwrap_or(0).to_string(), hint: "raw_game_detail_import".to_string() },
        MetricCard { label: "Clean TCP rows".to_string(), value: clean_video_rows.unwrap_or(0).to_string(), hint: "dwd_tcp_detail_clean".to_string() },
        MetricCard { label: "Clean Game rows".to_string(), value: clean_game_rows.unwrap_or(0).to_string(), hint: "dwd_game_detail_clean".to_string() },
    ])
}

#[tauri::command]
fn etl_start_clean_job(req: EtlRequest) -> Result<CommandAck, String> {
    let tcp_sql = sql_runner::bind_batch_params(TCP_CLEAN_SQL, &req.import_batch_id, None);
    let game_sql = sql_runner::bind_batch_params(GAME_CLEAN_SQL, &req.import_batch_id, None);
    let tcp_rows = sql_runner::execute_script(&req.settings, &tcp_sql)?;
    let game_rows = sql_runner::execute_script(&req.settings, &game_sql)?;
    Ok(ack(format!("RAW to CLEAN finished: tcp affected={tcp_rows}, game affected={game_rows}")))
}

#[tauri::command]
fn etl_start_aggregate_job(req: EtlRequest) -> Result<CommandAck, String> {
    let analysis_run_id = req.analysis_run_id.unwrap_or_else(|| format!("RUN_{}", Uuid::new_v4().simple()));
    let dws_sql = sql_runner::bind_batch_params(USER_DAILY_SQL, &req.import_batch_id, None);
    let ads_sql = sql_runner::bind_batch_params(LEADS_SQL, &req.import_batch_id, Some(&analysis_run_id));
    let dws_rows = sql_runner::execute_script(&req.settings, &dws_sql)?;
    let ads_rows = sql_runner::execute_script(&req.settings, &ads_sql)?;
    Ok(ack(format!("aggregate finished: analysis_run_id={analysis_run_id}, dws affected={dws_rows}, ads affected={ads_rows}")))
}

#[tauri::command]
fn dashboard_get_overview(req: DashboardRequest) -> Result<DashboardOverview, String> {
    let mut conn = db::conn(&req.settings)?;
    let users: Option<u64> = conn.exec_first("SELECT COUNT(DISTINCT user_key) FROM dws_user_daily_profile WHERE import_batch_id=?", (&req.import_batch_id,)).map_err(|err| err.to_string())?;
    let download_gb: Option<f64> = conn.exec_first("SELECT COALESCE(SUM(total_download_gb),0) FROM dws_user_daily_profile WHERE import_batch_id=?", (&req.import_batch_id,)).map_err(|err| err.to_string())?;
    let game_hours: Option<f64> = conn.exec_first("SELECT COALESCE(SUM(total_game_hours),0) FROM dws_user_daily_profile WHERE import_batch_id=?", (&req.import_batch_id,)).map_err(|err| err.to_string())?;
    let a1_users: Option<u64> = if let Some(run_id) = &req.analysis_run_id { conn.exec_first("SELECT COUNT(*) FROM ads_migration_lead_user WHERE analysis_run_id=? AND lead_type LIKE 'A1_%'", (run_id,)).map_err(|err| err.to_string())? } else { Some(0) };
    Ok(DashboardOverview { metrics: vec![
        MetricCard { label: "Clean users".to_string(), value: users.unwrap_or(0).to_string(), hint: "DWS distinct users".to_string() },
        MetricCard { label: "Video GB".to_string(), value: format!("{:.2}", download_gb.unwrap_or(0.0)), hint: "sum total_download_gb".to_string() },
        MetricCard { label: "Game hours".to_string(), value: format!("{:.2}", game_hours.unwrap_or(0.0)), hint: "sum total_game_hours".to_string() },
        MetricCard { label: "A1 leads".to_string(), value: a1_users.unwrap_or(0).to_string(), hint: "priority marketing users".to_string() },
    ]})
}

#[tauri::command]
fn leads_query_users(req: LeadsQueryRequest) -> Result<Vec<LeadUserRow>, String> {
    let mut conn = db::conn(&req.settings)?;
    let page = req.page.unwrap_or(1).max(1);
    let page_size = req.page_size.unwrap_or(100).clamp(1, 1000);
    let offset = (page - 1) * page_size;
    conn.exec_map("SELECT user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer FROM ads_migration_lead_user WHERE analysis_run_id=? ORDER BY demand_score DESC, migration_motive_score DESC LIMIT ? OFFSET ?", (&req.analysis_run_id, page_size, offset), |(user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer)| LeadUserRow { user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer }).map_err(|err| format!("failed to query leads: {err}"))
}

#[tauri::command]
fn export_leads_csv(req: ExportLeadsRequest) -> Result<CommandAck, String> {
    let rows = leads_query_users(LeadsQueryRequest { settings: req.settings, analysis_run_id: req.analysis_run_id, page: Some(1), page_size: Some(1000) })?;
    let mut writer = csv::Writer::from_path(&req.output_path).map_err(|err| format!("failed to create export file: {err}"))?;
    writer.write_record(["user_key", "user_type", "lead_type", "demand_score", "migration_motive_score", "recommended_offer"]).map_err(|err| err.to_string())?;
    for row in rows { writer.write_record([row.user_key, row.user_type.unwrap_or_default(), row.lead_type, row.demand_score.to_string(), row.migration_motive_score.to_string(), row.recommended_offer.unwrap_or_default()]).map_err(|err| err.to_string())?; }
    writer.flush().map_err(|err| err.to_string())?;
    Ok(ack(format!("leads exported to {}", req.output_path)))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            db_test_connection,
            db_initialize,
            import_probe_csv,
            import_create_batch,
            import_start_raw_load,
            quality_get_batch_report,
            etl_start_clean_job,
            etl_start_aggregate_job,
            dashboard_get_overview,
            leads_query_users,
            export_leads_csv,
            phase_commands::quality_run_gate,
            phase_commands::etl_run_complete_aggregates,
            phase_commands::ads_run_complete_dashboards,
            phase_commands::leads_run_final_fusion,
            phase_commands::dashboard_get_app_category,
            phase_commands::dashboard_get_experience_quality,
            phase_commands::dashboard_get_cable_fiber_compare,
            phase_commands::leads_get_final_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running SA FBB Experience Workbench");
}

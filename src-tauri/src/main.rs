#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod job_runner;
mod migrations;
mod models;
mod phase_commands;
mod probe;
mod raw_import;
mod sql_runner;

use mysql::prelude::*;
use uuid::Uuid;

use job_runner::JobStep;
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
    let source_file_name = std::path::Path::new(&req.file_path).file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| req.file_path.clone());
    let file_size = std::fs::metadata(&req.file_path).map(|m| m.len()).ok();
    let mut conn = db::conn(&req.settings)?;
    conn.exec_drop(
        "INSERT INTO meta_import_batch (import_batch_id, data_type, source_file_name, source_file_path, source_file_size_bytes, status) VALUES (?, ?, ?, ?, ?, 'pending')",
        (&import_batch_id, &req.data_type, &source_file_name, &req.file_path, file_size),
    ).map_err(|err| format!("failed to create import batch: {err}"))?;
    Ok(ImportBatchResult { import_batch_id, data_type: req.data_type, source_file_name, status: "pending".to_string() })
}

#[tauri::command]
fn import_start_raw_load(req: RawLoadRequest) -> Result<CommandAck, String> { raw_import::start_raw_load(req).map(ack) }

#[tauri::command]
fn import_get_batch_status(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    let row: Option<(String, u64, u64, String)> = conn.exec_first(
        "SELECT status, COALESCE(imported_rows,0), COALESCE(total_rows,0), COALESCE(message,'') FROM meta_import_batch WHERE import_batch_id=?",
        (&import_batch_id,),
    ).map_err(|err| format!("failed to query import batch status: {err}"))?;
    let Some((status, imported_rows, total_rows, message)) = row else {
        return Ok(vec![MetricCard { label: "Import batch".to_string(), value: "not_found".to_string(), hint: import_batch_id }]);
    };
    let progress = if total_rows > 0 { imported_rows as f64 / total_rows as f64 * 100.0 } else { 0.0 };
    Ok(vec![
        MetricCard { label: "Import status".to_string(), value: status, hint: message },
        MetricCard { label: "Imported rows".to_string(), value: imported_rows.to_string(), hint: "meta_import_batch.imported_rows".to_string() },
        MetricCard { label: "Total rows".to_string(), value: total_rows.to_string(), hint: "meta_import_batch.total_rows".to_string() },
        MetricCard { label: "Progress".to_string(), value: format!("{progress:.2}%"), hint: "imported_rows / total_rows".to_string() },
    ])
}

#[tauri::command]
fn etl_get_recent_jobs(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT job_type, status, COALESCE(current_step,'-'), COALESCE(affected_rows,0) FROM meta_etl_job WHERE import_batch_id=? ORDER BY started_at DESC LIMIT 12",
        (&import_batch_id,),
        |(job_type, status, current_step, affected_rows): (String, String, String, u64)| MetricCard {
            label: job_type,
            value: status,
            hint: format!("step={current_step}, affected_rows={affected_rows}"),
        },
    ).map_err(|err| format!("failed to query ETL jobs: {err}"))
}

fn row_consistency_card(imported_rows: u64, total_rows: u64, raw_rows: u64) -> MetricCard {
    let expected_rows = if total_rows > 0 { total_rows } else { imported_rows };
    let diff = raw_rows as i128 - expected_rows as i128;
    let value = if expected_rows == 0 {
        "unknown".to_string()
    } else if diff == 0 {
        "ok".to_string()
    } else {
        "mismatch".to_string()
    };
    MetricCard {
        label: "CSV vs RAW rows".to_string(),
        value,
        hint: format!("raw_rows={raw_rows}, expected_rows={expected_rows}, diff={diff}, rule=total_rows_or_imported_rows"),
    }
}

fn typed_raw_distribution_card(tcp_rows: u64, game_rows: u64) -> MetricCard {
    let loaded_types = [tcp_rows > 0, game_rows > 0].into_iter().filter(|loaded| *loaded).count();
    let value = match loaded_types {
        0 => "empty".to_string(),
        1 => "single_type".to_string(),
        _ => "mixed_type".to_string(),
    };
    MetricCard {
        label: "RAW type distribution".to_string(),
        value,
        hint: format!("tcp_rows={tcp_rows}, game_rows={game_rows}; mixed_type is expected only when one batch intentionally loads multiple source types"),
    }
}

#[tauri::command]
fn quality_get_batch_report(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    let tcp_rows: Option<u64> = conn.exec_first("SELECT COUNT(*) FROM raw_tcp_detail_import WHERE import_batch_id=?", (&import_batch_id,)).map_err(|err| err.to_string())?;
    let game_rows: Option<u64> = conn.exec_first("SELECT COUNT(*) FROM raw_game_detail_import WHERE import_batch_id=?", (&import_batch_id,)).map_err(|err| err.to_string())?;
    let clean_video_rows: Option<u64> = conn.exec_first("SELECT COUNT(*) FROM dwd_tcp_detail_clean WHERE import_batch_id=?", (&import_batch_id,)).map_err(|err| err.to_string())?;
    let clean_game_rows: Option<u64> = conn.exec_first("SELECT COUNT(*) FROM dwd_game_detail_clean WHERE import_batch_id=?", (&import_batch_id,)).map_err(|err| err.to_string())?;
    let import_meta: Option<(u64, u64)> = conn.exec_first(
        "SELECT COALESCE(imported_rows,0), COALESCE(total_rows,0) FROM meta_import_batch WHERE import_batch_id=?",
        (&import_batch_id,),
    ).map_err(|err| err.to_string())?;
    let tcp_rows = tcp_rows.unwrap_or(0);
    let game_rows = game_rows.unwrap_or(0);
    let (imported_rows, total_rows) = import_meta.unwrap_or((0, 0));
    Ok(vec![
        MetricCard { label: "RAW TCP rows".to_string(), value: tcp_rows.to_string(), hint: "raw_tcp_detail_import".to_string() },
        MetricCard { label: "RAW Game rows".to_string(), value: game_rows.to_string(), hint: "raw_game_detail_import".to_string() },
        MetricCard { label: "Clean TCP rows".to_string(), value: clean_video_rows.unwrap_or(0).to_string(), hint: "dwd_tcp_detail_clean".to_string() },
        MetricCard { label: "Clean Game rows".to_string(), value: clean_game_rows.unwrap_or(0).to_string(), hint: "dwd_game_detail_clean".to_string() },
        row_consistency_card(imported_rows, total_rows, tcp_rows + game_rows),
        typed_raw_distribution_card(tcp_rows, game_rows),
    ])
}

#[tauri::command]
fn etl_start_clean_job(req: EtlRequest) -> Result<CommandAck, String> {
    let tcp_sql = sql_runner::bind_batch_params(TCP_CLEAN_SQL, &req.import_batch_id, None);
    let game_sql = sql_runner::bind_batch_params(GAME_CLEAN_SQL, &req.import_batch_id, None);
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "raw_to_clean", vec![
        JobStep { step_name: "tcp_raw_to_clean", source_table: "raw_tcp_detail_import", target_table: "dwd_tcp_detail_clean", sql_template: "001_tcp_raw_to_clean.sql", sql: tcp_sql },
        JobStep { step_name: "game_raw_to_clean", source_table: "raw_game_detail_import", target_table: "dwd_game_detail_clean", sql_template: "002_game_raw_to_clean.sql", sql: game_sql },
    ])?;
    Ok(ack(message))
}

#[tauri::command]
fn etl_start_aggregate_job(req: EtlRequest) -> Result<CommandAck, String> {
    let analysis_run_id = req.analysis_run_id.unwrap_or_else(|| format!("RUN_{}", Uuid::new_v4().simple()));
    let dws_sql = sql_runner::bind_batch_params(USER_DAILY_SQL, &req.import_batch_id, None);
    let ads_sql = sql_runner::bind_batch_params(LEADS_SQL, &req.import_batch_id, Some(&analysis_run_id));
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "base_aggregate", vec![
        JobStep { step_name: "user_daily_profile", source_table: "dwd_tcp_detail_clean,dwd_game_detail_clean", target_table: "dws_user_daily_profile", sql_template: "001_user_daily_profile.sql", sql: dws_sql },
        JobStep { step_name: "migration_leads", source_table: "dws_user_daily_profile", target_table: "ads_migration_lead_user", sql_template: "001_migration_leads.sql", sql: ads_sql },
    ])?;
    Ok(ack(format!("analysis_run_id={analysis_run_id}; {message}")))
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
            db_test_connection, db_initialize, import_probe_csv, import_create_batch, import_start_raw_load,
            import_get_batch_status, etl_get_recent_jobs, quality_get_batch_report, etl_start_clean_job,
            etl_start_aggregate_job, dashboard_get_overview, leads_query_users, export_leads_csv,
            phase_commands::quality_run_gate, phase_commands::etl_run_complete_aggregates,
            phase_commands::ads_run_complete_dashboards, phase_commands::leads_run_final_fusion,
            phase_commands::dashboard_get_app_category, phase_commands::dashboard_get_experience_quality,
            phase_commands::dashboard_get_cable_fiber_compare, phase_commands::leads_get_final_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running SA FBB Experience Workbench");
}

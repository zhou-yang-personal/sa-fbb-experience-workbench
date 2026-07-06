use mysql::prelude::*;
use uuid::Uuid;

use crate::batch_tables;
use crate::db;
use crate::job_runner::{self, JobStep};
use crate::models::{ack, CommandAck, EtlRequest, MetricCard, MySqlSettings};
use crate::sql_runner;

const TCP_CLEAN_SQL: &str = include_str!("../../database/sql/raw_to_clean/001_tcp_raw_to_clean.sql");
const GAME_CLEAN_SQL: &str = include_str!("../../database/sql/raw_to_clean/002_game_raw_to_clean.sql");
const USER_DAILY_SQL: &str = include_str!("../../database/sql/clean_to_dws/001_user_daily_profile.sql");
const LEADS_SQL: &str = include_str!("../../database/sql/dws_to_ads/001_migration_leads.sql");

#[tauri::command]
pub fn etl_get_recent_jobs(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
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

#[tauri::command]
pub fn etl_start_clean_job(req: EtlRequest) -> Result<CommandAck, String> {
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let tcp_bound = sql_runner::bind_batch_params(TCP_CLEAN_SQL, &req.import_batch_id, None);
    let game_bound = sql_runner::bind_batch_params(GAME_CLEAN_SQL, &req.import_batch_id, None);
    let tcp_sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &tcp_bound)?;
    let game_sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &game_bound)?;
    let raw_tcp = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "raw_tcp_detail_import")?;
    let raw_game = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "raw_game_detail_import")?;
    let dwd_tcp = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;
    let dwd_game = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_game_detail_clean")?;
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "raw_to_clean", vec![
        JobStep { step_name: "tcp_raw_to_clean", source_table: Box::leak(raw_tcp.into_boxed_str()), target_table: Box::leak(dwd_tcp.into_boxed_str()), sql_template: "001_tcp_raw_to_clean.sql", sql: tcp_sql },
        JobStep { step_name: "game_raw_to_clean", source_table: Box::leak(raw_game.into_boxed_str()), target_table: Box::leak(dwd_game.into_boxed_str()), sql_template: "002_game_raw_to_clean.sql", sql: game_sql },
    ])?;
    let _ = batch_tables::refresh_registry_counts(&req.settings, &req.import_batch_id);
    Ok(ack(message))
}

#[tauri::command]
pub fn etl_start_aggregate_job(req: EtlRequest) -> Result<CommandAck, String> {
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let analysis_run_id = req.analysis_run_id.unwrap_or_else(|| format!("RUN_{}", Uuid::new_v4().simple()));
    let dws_bound = sql_runner::bind_batch_params(USER_DAILY_SQL, &req.import_batch_id, None);
    let ads_bound = sql_runner::bind_batch_params(LEADS_SQL, &req.import_batch_id, Some(&analysis_run_id));
    let dws_sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &dws_bound)?;
    let ads_sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &ads_bound)?;
    let dwd_tcp = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;
    let dwd_game = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_game_detail_clean")?;
    let dws_user = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?;
    let ads_lead = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_migration_lead_user")?;
    let mut conn = db::conn(&req.settings)?;
    let _ = conn.exec_drop(
        "REPLACE INTO meta_analysis_run (analysis_run_id, import_batch_id, run_type, status, started_at, message) VALUES (?, ?, 'base_aggregate', 'running', NOW(), 'aggregate started')",
        (&analysis_run_id, &req.import_batch_id),
    );
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "base_aggregate", vec![
        JobStep { step_name: "user_daily_profile", source_table: Box::leak(format!("{dwd_tcp},{dwd_game}").into_boxed_str()), target_table: Box::leak(dws_user.into_boxed_str()), sql_template: "001_user_daily_profile.sql", sql: dws_sql },
        JobStep { step_name: "migration_leads", source_table: Box::leak(batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?.into_boxed_str()), target_table: Box::leak(ads_lead.into_boxed_str()), sql_template: "001_migration_leads.sql", sql: ads_sql },
    ])?;
    let _ = conn.exec_drop(
        "UPDATE meta_analysis_run SET status='success', finished_at=NOW(), message=? WHERE analysis_run_id=?",
        (&message, &analysis_run_id),
    );
    let _ = batch_tables::refresh_registry_counts(&req.settings, &req.import_batch_id);
    Ok(ack(format!("analysis_run_id={analysis_run_id}; {message}")))
}

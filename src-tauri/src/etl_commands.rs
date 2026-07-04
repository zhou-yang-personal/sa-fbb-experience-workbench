use mysql::prelude::*;
use uuid::Uuid;

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
    let tcp_sql = sql_runner::bind_batch_params(TCP_CLEAN_SQL, &req.import_batch_id, None);
    let game_sql = sql_runner::bind_batch_params(GAME_CLEAN_SQL, &req.import_batch_id, None);
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "raw_to_clean", vec![
        JobStep { step_name: "tcp_raw_to_clean", source_table: "raw_tcp_detail_import", target_table: "dwd_tcp_detail_clean", sql_template: "001_tcp_raw_to_clean.sql", sql: tcp_sql },
        JobStep { step_name: "game_raw_to_clean", source_table: "raw_game_detail_import", target_table: "dwd_game_detail_clean", sql_template: "002_game_raw_to_clean.sql", sql: game_sql },
    ])?;
    Ok(ack(message))
}

#[tauri::command]
pub fn etl_start_aggregate_job(req: EtlRequest) -> Result<CommandAck, String> {
    let analysis_run_id = req.analysis_run_id.unwrap_or_else(|| format!("RUN_{}", Uuid::new_v4().simple()));
    let dws_sql = sql_runner::bind_batch_params(USER_DAILY_SQL, &req.import_batch_id, None);
    let ads_sql = sql_runner::bind_batch_params(LEADS_SQL, &req.import_batch_id, Some(&analysis_run_id));
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "base_aggregate", vec![
        JobStep { step_name: "user_daily_profile", source_table: "dwd_tcp_detail_clean,dwd_game_detail_clean", target_table: "dws_user_daily_profile", sql_template: "001_user_daily_profile.sql", sql: dws_sql },
        JobStep { step_name: "migration_leads", source_table: "dws_user_daily_profile", target_table: "ads_migration_lead_user", sql_template: "001_migration_leads.sql", sql: ads_sql },
    ])?;
    Ok(ack(format!("analysis_run_id={analysis_run_id}; {message}")))
}

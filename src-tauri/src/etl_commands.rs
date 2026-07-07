use mysql::prelude::*;
use uuid::Uuid;

use crate::batch_tables;
use crate::db;
use crate::job_runner::{self, JobStep};
use crate::models::{ack, CommandAck, EtlRequest, MetricCard, MySqlSettings};
use crate::sql_runner;

const TCP_CLEAN_SQL: &str =
    include_str!("../../database/sql/raw_to_clean/001_tcp_raw_to_clean.sql");
const GAME_CLEAN_SQL: &str =
    include_str!("../../database/sql/raw_to_clean/002_game_raw_to_clean.sql");
const USER_DAILY_SQL: &str =
    include_str!("../../database/sql/clean_to_dws/001_user_daily_profile.sql");
const LEADS_SQL: &str = include_str!("../../database/sql/dws_to_ads/001_migration_leads.sql");

#[tauri::command]
pub fn etl_get_recent_jobs(
    settings: MySqlSettings,
    import_batch_id: String,
) -> Result<Vec<MetricCard>, String> {
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
    let data_type = fetch_batch_data_type(&req.settings, &req.import_batch_id)?;
    let clean_plan = clean_steps_for_data_type(&data_type);
    if clean_plan.is_empty() {
        let message = format!(
            "raw_to_clean skipped: data_type={data_type} is auxiliary and has no TCP/Game clean step"
        );
        record_skipped_clean_job(&req.settings, &req.import_batch_id, &data_type, &message)?;
        return Ok(ack(message));
    }
    let mut steps = Vec::new();
    for step_name in clean_plan {
        if step_name == "tcp_raw_to_clean" {
            let bound = sql_runner::bind_batch_params(TCP_CLEAN_SQL, &req.import_batch_id, None);
            let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
            let raw = batch_tables::resolve_table(
                &req.settings,
                &req.import_batch_id,
                "raw_tcp_detail_import",
            )?;
            let dwd = batch_tables::resolve_table(
                &req.settings,
                &req.import_batch_id,
                "dwd_tcp_detail_clean",
            )?;
            steps.push(JobStep {
                step_name: "tcp_raw_to_clean",
                source_table: Box::leak(raw.into_boxed_str()),
                target_table: Box::leak(dwd.into_boxed_str()),
                sql_template: "001_tcp_raw_to_clean.sql",
                sql,
            });
        } else if step_name == "game_raw_to_clean" {
            let bound = sql_runner::bind_batch_params(GAME_CLEAN_SQL, &req.import_batch_id, None);
            let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
            let raw = batch_tables::resolve_table(
                &req.settings,
                &req.import_batch_id,
                "raw_game_detail_import",
            )?;
            let dwd = batch_tables::resolve_table(
                &req.settings,
                &req.import_batch_id,
                "dwd_game_detail_clean",
            )?;
            steps.push(JobStep {
                step_name: "game_raw_to_clean",
                source_table: Box::leak(raw.into_boxed_str()),
                target_table: Box::leak(dwd.into_boxed_str()),
                sql_template: "002_game_raw_to_clean.sql",
                sql,
            });
        }
    }
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "raw_to_clean", steps)?;
    let _ = batch_tables::refresh_registry_counts(&req.settings, &req.import_batch_id);
    Ok(ack(message))
}

fn fetch_batch_data_type(
    settings: &MySqlSettings,
    import_batch_id: &str,
) -> Result<String, String> {
    let mut conn = db::conn(settings)?;
    let data_type: Option<String> = conn
        .exec_first(
            "SELECT data_type FROM meta_import_batch WHERE import_batch_id=? LIMIT 1",
            (import_batch_id,),
        )
        .map_err(|err| format!("failed to query batch data_type for clean job: {err}"))?;
    Ok(data_type
        .unwrap_or_else(|| "mixed".to_string())
        .to_lowercase())
}

fn clean_steps_for_data_type(data_type: &str) -> Vec<&'static str> {
    match data_type.to_lowercase().as_str() {
        "tcp" => vec!["tcp_raw_to_clean"],
        "game" => vec!["game_raw_to_clean"],
        "mixed" => vec!["tcp_raw_to_clean", "game_raw_to_clean"],
        _ => Vec::new(),
    }
}

fn record_skipped_clean_job(
    settings: &MySqlSettings,
    import_batch_id: &str,
    data_type: &str,
    message: &str,
) -> Result<(), String> {
    let job_id = format!("JOB_{}", Uuid::new_v4().simple());
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "INSERT INTO meta_etl_job (job_id, import_batch_id, job_type, status, current_step, started_at, finished_at, affected_rows) VALUES (?, ?, 'raw_to_clean', 'success', 'skipped_not_applicable', NOW(), NOW(), 0)",
        (&job_id, import_batch_id),
    )
    .map_err(|err| format!("failed to record skipped clean job: {err}"))?;
    conn.exec_drop(
        "INSERT INTO meta_etl_job_step (job_id, step_name, source_table, target_table, sql_template, status, started_at, finished_at, affected_rows, message) VALUES (?, 'skipped_not_applicable', ?, NULL, NULL, 'skipped', NOW(), NOW(), 0, ?)",
        (&job_id, data_type, message),
    )
    .map_err(|err| format!("failed to record skipped clean step: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{clean_steps_for_data_type, GAME_CLEAN_SQL, TCP_CLEAN_SQL};

    fn normalize_stat_time_text(value: &str) -> String {
        value
            .chars()
            .map(|ch| match ch {
                '\t' | '\n' | '\r' | '\u{00a0}' => ' ',
                other => other,
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn supported_stat_time_pattern(value: &str) -> bool {
        let Some((date, time)) = value.split_once(' ') else {
            return false;
        };
        let time_parts = time.split(':').collect::<Vec<_>>();
        let valid_time = (time_parts.len() == 2 || time_parts.len() == 3)
            && time_parts
                .iter()
                .all(|part| part.len() == 2 && part.chars().all(|ch| ch.is_ascii_digit()));
        if !valid_time {
            return false;
        }
        if date.contains('/') {
            let parts = date.split('/').collect::<Vec<_>>();
            return parts.len() == 3
                && (1..=2).contains(&parts[0].len())
                && (1..=2).contains(&parts[1].len())
                && parts[2].len() == 4
                && parts
                    .iter()
                    .all(|part| part.chars().all(|ch| ch.is_ascii_digit()));
        }
        let parts = date.split('-').collect::<Vec<_>>();
        parts.len() == 3
            && parts[0].len() == 4
            && parts[1].len() == 2
            && parts[2].len() == 2
            && parts
                .iter()
                .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
    }

    #[test]
    fn tcp_batch_clean_job_only_contains_tcp_step() {
        assert_eq!(clean_steps_for_data_type("tcp"), vec!["tcp_raw_to_clean"]);
    }

    #[test]
    fn game_batch_clean_job_only_contains_game_step() {
        assert_eq!(clean_steps_for_data_type("game"), vec!["game_raw_to_clean"]);
    }

    #[test]
    fn auxiliary_batch_clean_job_is_not_applicable() {
        assert!(clean_steps_for_data_type("crm").is_empty());
        assert!(clean_steps_for_data_type("coverage").is_empty());
        assert!(clean_steps_for_data_type("reachability").is_empty());
    }

    #[test]
    fn clean_sql_normalizes_invisible_stat_time_characters() {
        for sql in [TCP_CLEAN_SQL, GAME_CLEAN_SQL] {
            assert!(sql.contains("CHAR(9)"));
            assert!(sql.contains("CHAR(10)"));
            assert!(sql.contains("CHAR(13)"));
            assert!(sql.contains("CONVERT(0xC2A0 USING utf8mb4)"));
            assert!(!sql.contains("CHAR(160)"));
            assert!(sql.contains("stat_time_text"));
            assert!(sql.contains("WARN_INVALID_STAT_TIME"));
            assert!(sql.contains("[0-9]{1,2}/[0-9]{1,2}/[0-9]{4}"));
            assert!(!sql.contains("STR_TO_DATE(NULLIF(TRIM(r.statistics_duration)"));
            assert!(!sql.contains("STR_TO_DATE(NULLIF(TRIM(r.statistical_time)"));
            assert!(!sql.contains("CHAR(9), '')"));
            assert!(!sql.contains("CHAR(10), '')"));
            assert!(!sql.contains("CHAR(13), '')"));
        }
    }

    #[test]
    fn stat_time_text_normalization_preserves_middle_separator() {
        assert_eq!(
            normalize_stat_time_text("2026-06-28 15:35:00\t"),
            "2026-06-28 15:35:00"
        );
        assert_eq!(
            normalize_stat_time_text("2026-06-28\t15:35:00"),
            "2026-06-28 15:35:00"
        );
        assert_eq!(
            normalize_stat_time_text("2026-06-28\r\n15:35:00\u{00a0}"),
            "2026-06-28 15:35:00"
        );
    }

    #[test]
    fn stat_time_pattern_accepts_single_digit_day_or_month() {
        assert!(supported_stat_time_pattern("20/9/2025 23:58:06"));
        assert!(supported_stat_time_pattern("1/9/2025 03:05:00"));
        assert!(supported_stat_time_pattern("26/10/2025 11:20:00"));
        assert!(supported_stat_time_pattern("2026-06-28 15:35:00"));
        assert!(!supported_stat_time_pattern("bad_time_value"));
    }
}

#[tauri::command]
pub fn etl_start_aggregate_job(req: EtlRequest) -> Result<CommandAck, String> {
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let analysis_run_id = req
        .analysis_run_id
        .unwrap_or_else(|| format!("RUN_{}", Uuid::new_v4().simple()));
    let dws_bound = sql_runner::bind_batch_params(USER_DAILY_SQL, &req.import_batch_id, None);
    let ads_bound =
        sql_runner::bind_batch_params(LEADS_SQL, &req.import_batch_id, Some(&analysis_run_id));
    let dws_sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &dws_bound)?;
    let ads_sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &ads_bound)?;
    let dwd_tcp =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;
    let dwd_game =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_game_detail_clean")?;
    let dws_user = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_daily_profile",
    )?;
    let ads_lead = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_migration_lead_user",
    )?;
    let mut conn = db::conn(&req.settings)?;
    let _ = conn.exec_drop(
        "REPLACE INTO meta_analysis_run (analysis_run_id, import_batch_id, run_type, status, started_at, message) VALUES (?, ?, 'base_aggregate', 'running', NOW(), 'aggregate started')",
        (&analysis_run_id, &req.import_batch_id),
    );
    let message = job_runner::run_job(
        &req.settings,
        &req.import_batch_id,
        "base_aggregate",
        vec![
            JobStep {
                step_name: "user_daily_profile",
                source_table: Box::leak(format!("{dwd_tcp},{dwd_game}").into_boxed_str()),
                target_table: Box::leak(dws_user.into_boxed_str()),
                sql_template: "001_user_daily_profile.sql",
                sql: dws_sql,
            },
            JobStep {
                step_name: "migration_leads",
                source_table: Box::leak(
                    batch_tables::resolve_table(
                        &req.settings,
                        &req.import_batch_id,
                        "dws_user_daily_profile",
                    )?
                    .into_boxed_str(),
                ),
                target_table: Box::leak(ads_lead.into_boxed_str()),
                sql_template: "001_migration_leads.sql",
                sql: ads_sql,
            },
        ],
    )?;
    let _ = conn.exec_drop(
        "UPDATE meta_analysis_run SET status='success', finished_at=NOW(), message=? WHERE analysis_run_id=?",
        (&message, &analysis_run_id),
    );
    let _ = batch_tables::refresh_registry_counts(&req.settings, &req.import_batch_id);
    Ok(ack(format!("analysis_run_id={analysis_run_id}; {message}")))
}

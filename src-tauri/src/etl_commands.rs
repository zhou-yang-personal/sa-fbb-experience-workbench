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
const CLEAN_CHUNK_ROWS: u64 = 500_000;
const CLEAN_SECONDARY_INDEXES: &[(&str, &str)] = &[
    (
        "ix_batch_user_time",
        "(import_batch_id,user_key,stat_time)",
    ),
    (
        "ix_batch_category",
        "(import_batch_id,app_category,user_type,stat_date)",
    ),
    (
        "ix_batch_date_hour",
        "(import_batch_id,stat_date,hour_of_day)",
    ),
];

fn raw_id_partitions(
    settings: &MySqlSettings,
    raw_table: &str,
) -> Result<Vec<(u64, u64)>, String> {
    let safe = batch_tables::sanitize_identifier(raw_table)?;
    let mut conn = db::conn(settings)?;
    let bounds: Option<(Option<u64>, Option<u64>)> = conn
        .query_first(format!("SELECT MIN(id),MAX(id) FROM `{safe}`"))
        .map_err(|err| format!("failed to inspect RAW id range for {safe}: {err}"))?;
    let Some((Some(min_id), Some(max_id))) = bounds else {
        return Ok(Vec::new());
    };
    let mut partitions = Vec::new();
    let mut start = min_id;
    loop {
        let end = start
            .saturating_add(CLEAN_CHUNK_ROWS.saturating_sub(1))
            .min(max_id);
        partitions.push((start, end));
        if end >= max_id {
            break;
        }
        start = end.saturating_add(1);
    }
    Ok(partitions)
}

fn prepare_clean_target(settings: &MySqlSettings, table: &str) -> Result<(), String> {
    let safe = batch_tables::sanitize_identifier(table)?;
    sql_runner::execute_script(settings, &format!("TRUNCATE TABLE `{safe}`;"))?;
    let mut conn = db::conn(settings)?;
    let existing: Vec<String> = conn
        .exec_map(
            "SELECT DISTINCT index_name FROM information_schema.statistics WHERE table_schema=DATABASE() AND table_name=? AND index_name IN ('ix_batch_user_time','ix_batch_category','ix_batch_date_hour')",
            (&safe,),
            |name: String| name,
        )
        .map_err(|err| format!("failed to inspect CLEAN indexes on {safe}: {err}"))?;
    if existing.is_empty() {
        return Ok(());
    }
    let drops = existing
        .iter()
        .map(|name| format!("DROP INDEX `{name}`"))
        .collect::<Vec<_>>()
        .join(",");
    sql_runner::execute_script(
        settings,
        &format!("ALTER TABLE `{safe}` {drops}, ALGORITHM=INPLACE, LOCK=NONE;"),
    )?;
    Ok(())
}

fn restore_clean_indexes(settings: &MySqlSettings, table: &str) -> Result<(), String> {
    let safe = batch_tables::sanitize_identifier(table)?;
    let mut conn = db::conn(settings)?;
    let existing: Vec<String> = conn
        .exec_map(
            "SELECT DISTINCT index_name FROM information_schema.statistics WHERE table_schema=DATABASE() AND table_name=?",
            (&safe,),
            |name: String| name,
        )
        .map_err(|err| format!("failed to inspect CLEAN indexes on {safe}: {err}"))?;
    let additions = CLEAN_SECONDARY_INDEXES
        .iter()
        .filter(|(name, _)| !existing.iter().any(|value| value == name))
        .map(|(name, definition)| format!("ADD INDEX `{name}` {definition}"))
        .collect::<Vec<_>>();
    if additions.is_empty() {
        return Ok(());
    }
    sql_runner::execute_script(
        settings,
        &format!(
            "ALTER TABLE `{safe}` {}, ALGORITHM=INPLACE, LOCK=NONE;",
            additions.join(",")
        ),
    )?;
    Ok(())
}

fn bind_clean_partition(sql: &str, start_id: u64, end_id: u64) -> String {
    format!(
        "/* clean_partition raw_id={start_id}..{end_id} */\n{}",
        sql.replace(":source_id_start", &start_id.to_string())
            .replace(":source_id_end", &end_id.to_string())
    )
}

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
    let _clean_lock = db::acquire_named_lock(&req.settings, db::CLEAN_LOCK_NAME)?;
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
    let mut clean_targets: Vec<String> = Vec::new();
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
            let partitions = match raw_id_partitions(&req.settings, &raw) {
                Ok(value) => value,
                Err(err) => {
                    for target in &clean_targets {
                        let _ = restore_clean_indexes(&req.settings, target);
                    }
                    return Err(err);
                }
            };
            if partitions.is_empty() {
                for target in &clean_targets {
                    let _ = restore_clean_indexes(&req.settings, target);
                }
                return Err(format!("RAW TCP table contains no rows: {raw}"));
            }
            if let Err(err) = prepare_clean_target(&req.settings, &dwd) {
                for target in &clean_targets {
                    let _ = restore_clean_indexes(&req.settings, target);
                }
                return Err(err);
            }
            clean_targets.push(dwd.clone());
            let partition_count = partitions.len();
            for (index, (start_id, end_id)) in partitions.into_iter().enumerate() {
                steps.push(JobStep {
                    step_name: Box::leak(
                        format!(
                            "tcp_raw_to_clean_{:03}_of_{partition_count:03}_id_{start_id}_{end_id}",
                            index + 1
                        )
                        .into_boxed_str(),
                    ),
                    source_table: Box::leak(raw.clone().into_boxed_str()),
                    target_table: Box::leak(dwd.clone().into_boxed_str()),
                    sql_template: "001_tcp_raw_to_clean.sql",
                    sql: bind_clean_partition(&sql, start_id, end_id),
                });
            }
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
            let partitions = match raw_id_partitions(&req.settings, &raw) {
                Ok(value) => value,
                Err(err) => {
                    for target in &clean_targets {
                        let _ = restore_clean_indexes(&req.settings, target);
                    }
                    return Err(err);
                }
            };
            if partitions.is_empty() {
                for target in &clean_targets {
                    let _ = restore_clean_indexes(&req.settings, target);
                }
                return Err(format!("RAW Game table contains no rows: {raw}"));
            }
            if let Err(err) = prepare_clean_target(&req.settings, &dwd) {
                for target in &clean_targets {
                    let _ = restore_clean_indexes(&req.settings, target);
                }
                return Err(err);
            }
            clean_targets.push(dwd.clone());
            let partition_count = partitions.len();
            for (index, (start_id, end_id)) in partitions.into_iter().enumerate() {
                steps.push(JobStep {
                    step_name: Box::leak(
                        format!(
                            "game_raw_to_clean_{:03}_of_{partition_count:03}_id_{start_id}_{end_id}",
                            index + 1
                        )
                        .into_boxed_str(),
                    ),
                    source_table: Box::leak(raw.clone().into_boxed_str()),
                    target_table: Box::leak(dwd.clone().into_boxed_str()),
                    sql_template: "002_game_raw_to_clean.sql",
                    sql: bind_clean_partition(&sql, start_id, end_id),
                });
            }
        }
    }
    let job_result = job_runner::run_job(&req.settings, &req.import_batch_id, "raw_to_clean", steps);
    let mut index_errors = Vec::new();
    for target in &clean_targets {
        if let Err(err) = restore_clean_indexes(&req.settings, target) {
            index_errors.push(err);
        }
    }
    let message = match (job_result, index_errors.is_empty()) {
        (Ok(message), true) => message,
        (Ok(_), false) => {
            return Err(format!(
                "CLEAN rows loaded but secondary index rebuild failed: {}",
                index_errors.join(" | ")
            ))
        }
        (Err(err), true) => return Err(err),
        (Err(err), false) => {
            return Err(format!(
                "{err}; additionally failed to restore CLEAN indexes: {}",
                index_errors.join(" | ")
            ))
        }
    };
    let _ = batch_tables::refresh_registry_estimates(&req.settings, &req.import_batch_id);
    Ok(ack(format!(
        "{message}; partition_rows={CLEAN_CHUNK_ROWS}; secondary indexes rebuilt after bulk load"
    )))
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
    use super::{
        bind_clean_partition, clean_steps_for_data_type, GAME_CLEAN_SQL, TCP_CLEAN_SQL,
    };

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
            assert!(!sql.contains("REGEXP_REPLACE"));
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
    fn clean_sql_validates_ipv4_before_inet_aton() {
        for sql in [TCP_CLEAN_SQL, GAME_CLEAN_SQL] {
            assert!(sql.contains("IS_IPV4(r.account_key) = 1"));
            assert!(sql.contains("IS_IPV4(r.ip_key) = 1"));
            assert!(sql.contains("INET_ATON(CASE WHEN IS_IPV4(r.account_key) = 1"));
            assert!(sql.contains("INET_ATON(CASE WHEN IS_IPV4(r.ip_key) = 1"));
            assert!(!sql.contains("WHEN INET_ATON(r.account_key) IS NOT NULL"));
            assert!(!sql.contains("WHEN INET_ATON(r.ip_key) IS NOT NULL"));
        }
    }

    #[test]
    fn clean_sql_is_bounded_by_raw_primary_key_partition() {
        for sql in [TCP_CLEAN_SQL, GAME_CLEAN_SQL] {
            assert!(sql.contains("r.id BETWEEN :source_id_start AND :source_id_end"));
            assert!(sql.contains("FORCE INDEX (PRIMARY)"));
            assert!(!sql.contains("DELETE FROM :dwd_"));
            let bound = bind_clean_partition(sql, 500_001, 1_000_000);
            assert!(bound.starts_with("/* clean_partition raw_id=500001..1000000 */"));
            assert!(bound.contains("r.id BETWEEN 500001 AND 1000000"));
            assert!(!bound.contains(":source_id_start"));
            assert!(!bound.contains(":source_id_end"));
        }
    }

    #[test]
    fn tcp_clean_keeps_duration_rate_and_connection_evidence() {
        for field in [
            "effective_duration_hours",
            "video_duration_hours",
            "avg_download_mbps",
            "throughput_mbps",
            "connection_success_pct",
            "connection_delay_ms",
            "download_fluency",
        ] {
            assert!(TCP_CLEAN_SQL.contains(field), "missing {field}");
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
    crate::migrations::ensure_access_schema(&req.settings)?;
    crate::migrations::ensure_experience_policy_schema(&req.settings)?;
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let analysis_run_id = req
        .analysis_run_id
        .unwrap_or_else(|| format!("RUN_{}", Uuid::new_v4().simple()));
    let dws_bound = sql_runner::bind_batch_params(USER_DAILY_SQL, &req.import_batch_id, None);
    let dws_sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &dws_bound)?;
    let dwd_tcp =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;
    let dwd_game =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_game_detail_clean")?;
    let dws_user = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_daily_profile",
    )?;
    let mut conn = db::conn(&req.settings)?;
    let (access_rule_set_id, access_rule_set_version, others_access_type): (
        Option<String>,
        Option<i64>,
        Option<String>,
    ) = conn
        .exec_first(
            "SELECT b.access_rule_set_id, b.access_rule_set_version, s.default_access_type FROM meta_import_batch b LEFT JOIN meta_access_rule_set s ON s.rule_set_id=b.access_rule_set_id WHERE b.import_batch_id=?",
            (&req.import_batch_id,),
        )
        .map_err(|err| format!("failed to snapshot access rules for analysis run: {err}"))?
        .ok_or_else(|| format!("import batch not found: {}", req.import_batch_id))?;
    let access_rule_set_id = access_rule_set_id.ok_or_else(|| {
        "analysis requires a published IP rule version with an explicit Others mapping".to_string()
    })?;
    let access_rule_set_version = access_rule_set_version.ok_or_else(|| {
        "analysis batch is missing its access rule version; bind a published rule first".to_string()
    })?;
    let others_access_type = others_access_type
        .as_deref()
        .map(crate::access_rule_commands::normalize_others_access_type)
        .transpose()?
        .ok_or_else(|| {
            "analysis batch rule has no explicit Others mapping; publish a corrected rule version"
                .to_string()
        })?;
    let (experience_policy_id, experience_policy_version): (String, i64) = conn
        .exec_first(
            "SELECT policy_id, CAST(version AS SIGNED) FROM meta_experience_analysis_policy WHERE status='published' ORDER BY version DESC LIMIT 1",
            (),
        )
        .map_err(|err| format!("failed to resolve published experience policy: {err}"))?
        .ok_or_else(|| "no published experience analysis policy is available".to_string())?;
    conn.exec_drop(
        "REPLACE INTO meta_analysis_run (analysis_run_id, import_batch_id, run_type, access_rule_set_id, access_rule_set_version, others_access_type, experience_policy_id, experience_policy_version, status, started_at, message) VALUES (?, ?, 'base_aggregate', ?, ?, ?, ?, ?, 'running', NOW(), 'aggregate started')",
        (&analysis_run_id, &req.import_batch_id, &access_rule_set_id, access_rule_set_version, &others_access_type, &experience_policy_id, experience_policy_version),
    )
    .map_err(|err| format!("failed to create traceable analysis run: {err}"))?;
    conn.exec_drop(
        "REPLACE INTO meta_analysis_run_policy_binding (analysis_run_id, import_batch_id, access_rule_set_id, access_rule_set_version, others_access_type, app_mapping_version, experience_policy_id, experience_policy_version, policy_snapshot) VALUES (?, ?, ?, ?, ?, NULL, ?, ?, JSON_OBJECT('access_rule_set_id', ?, 'access_rule_set_version', ?, 'others_access_type', ?, 'experience_policy_id', ?, 'experience_policy_version', ?))",
        (
            &analysis_run_id,
            &req.import_batch_id,
            &access_rule_set_id,
            access_rule_set_version,
            &others_access_type,
            &experience_policy_id,
            experience_policy_version,
            &access_rule_set_id,
            access_rule_set_version,
            &others_access_type,
            &experience_policy_id,
            experience_policy_version,
        ),
    )
    .map_err(|err| format!("failed to bind analysis policy snapshot: {err}"))?;
    let message = match job_runner::run_job(
        &req.settings,
        &req.import_batch_id,
        "base_aggregate",
        vec![JobStep {
            step_name: "user_daily_profile",
            source_table: Box::leak(format!("{dwd_tcp},{dwd_game}").into_boxed_str()),
            target_table: Box::leak(dws_user.into_boxed_str()),
            sql_template: "001_user_daily_profile.sql",
            sql: dws_sql,
        }],
    ) {
        Ok(message) => message,
        Err(err) => {
            let _ = mark_analysis_run_status(
                &req.settings,
                &analysis_run_id,
                "failed",
                &format!("base aggregate failed: {err}"),
            );
            return Err(err);
        }
    };
    let _ = conn.exec_drop(
        "UPDATE meta_analysis_run SET status='running', finished_at=NULL, message=? WHERE analysis_run_id=?",
        (
            format!("base aggregate ready; complete DWS/ADS pending; {message}"),
            &analysis_run_id,
        ),
    );
    let _ = batch_tables::refresh_registry_estimates(&req.settings, &req.import_batch_id);
    Ok(ack(format!("analysis_run_id={analysis_run_id}; {message}")))
}

pub fn mark_analysis_run_status(
    settings: &MySqlSettings,
    analysis_run_id: &str,
    status: &str,
    message: &str,
) -> Result<(), String> {
    if !matches!(
        status,
        "running" | "success" | "failed" | "degraded" | "interrupted"
    ) {
        return Err(format!("unsupported analysis run status: {status}"));
    }
    let mut conn = db::conn(settings)?;
    let finished_at = if matches!(status, "success" | "failed" | "degraded" | "interrupted") {
        "UTC_TIMESTAMP()"
    } else {
        "NULL"
    };
    conn.exec_drop(
        format!(
            "UPDATE meta_analysis_run SET status=?, finished_at={finished_at}, message=? WHERE analysis_run_id=?"
        ),
        (status, message, analysis_run_id),
    )
    .map_err(|err| format!("failed to update analysis run status: {err}"))
}

pub fn refresh_migration_leads(
    settings: &MySqlSettings,
    import_batch_id: &str,
    analysis_run_id: &str,
) -> Result<String, String> {
    let bound = sql_runner::bind_batch_params(LEADS_SQL, import_batch_id, Some(analysis_run_id));
    let sql = batch_tables::bind_batch_tables(settings, import_batch_id, &bound)?;
    let dws_user =
        batch_tables::resolve_table(settings, import_batch_id, "dws_user_daily_profile")?;
    let dws_bottleneck =
        batch_tables::resolve_table(settings, import_batch_id, "dws_user_experience_bottleneck")?;
    let ads_lead =
        batch_tables::resolve_table(settings, import_batch_id, "ads_migration_lead_user")?;
    job_runner::run_job(
        settings,
        import_batch_id,
        "migration_lead_scoring",
        vec![JobStep {
            step_name: "migration_leads_after_bottleneck",
            source_table: Box::leak(format!("{dws_user},{dws_bottleneck}").into_boxed_str()),
            target_table: Box::leak(ads_lead.into_boxed_str()),
            sql_template: "001_migration_leads.sql",
            sql,
        }],
    )
}

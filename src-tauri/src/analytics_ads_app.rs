use mysql::prelude::*;
use uuid::Uuid;

use crate::batch_tables;
use crate::models::{ack, CommandAck, EtlRequest};
use crate::sql_runner;

const SQL: &str = include_str!("../../database/sql/dws_to_ads/003b_analytics_app_rank.sql");
const EXPERIENCE_DWS_V2_SQL: &str =
    include_str!("../../database/sql/clean_to_dws/003_experience_metrics_v2.sql");
const EXPERIENCE_ADS_V2_SQL: &str =
    include_str!("../../database/sql/dws_to_ads/004_experience_app_v2.sql");
const EXPERIENCE_HOURLY_DWS_V2_SQL: &str =
    include_str!("../../database/sql/clean_to_dws/004_experience_hourly_v2.sql");
const EXPERIENCE_HOURLY_ADS_V2_SQL: &str =
    include_str!("../../database/sql/dws_to_ads/005_experience_hourly_v2.sql");
const HOURLY_STAGE: &str = "dws_ads_aggregate";
const HOURLY_SUBTASK: &str = "hourly_v2";

#[derive(Clone, Debug, PartialEq, Eq)]
struct HourPartition {
    stat_date: String,
    hour_of_day: u8,
}

#[derive(Clone, Debug, Default)]
struct PartitionSummary {
    total: usize,
    completed: usize,
    skipped: usize,
    affected_rows: u64,
}

fn bind_partition_params(sql: &str, partition: &HourPartition) -> String {
    sql.replace(
        ":partition_date",
        &format!("'{}'", sql_runner::escape_sql_literal(&partition.stat_date)),
    )
    .replace(":partition_hour", &partition.hour_of_day.to_string())
}

fn discover_partitions(req: &EtlRequest) -> Result<Vec<HourPartition>, String> {
    let template = "SELECT DATE_FORMAT(stat_date,'%Y-%m-%d'),CAST(hour_of_day AS SIGNED) FROM :dwd_tcp_detail_clean WHERE import_batch_id=:import_batch_id AND stat_date IS NOT NULL AND hour_of_day BETWEEN 0 AND 23 UNION SELECT DATE_FORMAT(stat_date,'%Y-%m-%d'),CAST(hour_of_day AS SIGNED) FROM :dwd_game_detail_clean WHERE import_batch_id=:import_batch_id AND stat_date IS NOT NULL AND hour_of_day BETWEEN 0 AND 23 ORDER BY 1,2";
    let bound = sql_runner::bind_batch_params(template, &req.import_batch_id, None);
    let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
    let mut conn = crate::db::conn(&req.settings)?;
    let rows: Vec<(String, i64)> = conn
        .query(sql)
        .map_err(|err| format!("failed to discover hourly V2 partitions: {err}"))?;
    rows.into_iter()
        .map(|(stat_date, hour)| {
            let hour_of_day = u8::try_from(hour)
                .ok()
                .filter(|value| *value <= 23)
                .ok_or_else(|| format!("invalid hourly partition {stat_date}/{hour}"))?;
            Ok(HourPartition {
                stat_date,
                hour_of_day,
            })
        })
        .collect()
}

fn ensure_hourly_partition_index(
    req: &EtlRequest,
    pipeline_run_id: &str,
    logical_table: &str,
) -> Result<(), String> {
    let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, logical_table)?;
    let mut conn = crate::db::conn(&req.settings)?;
    let exists: Option<i8> = conn
        .exec_first(
            "SELECT EXISTS(SELECT 1 FROM information_schema.statistics WHERE table_schema=DATABASE() AND table_name=? AND index_name='ix_batch_date_hour')",
            (&table,),
        )
        .map_err(|err| format!("failed to inspect hourly partition index on {table}: {err}"))?;
    if exists.unwrap_or(0) != 0 {
        return Ok(());
    }
    crate::import_pipeline_commands::record_aggregation_partition_progress(
        &req.settings,
        pipeline_run_id,
        "info",
        &format!(
            "小时分片索引开始：table={table}；index=ix_batch_date_hour；首次执行可能需要较长时间，但不会产生整批聚合的大事务"
        ),
    );
    let started = std::time::Instant::now();
    sql_runner::execute_script(
        &req.settings,
        &format!(
            "ALTER TABLE `{table}` ADD INDEX ix_batch_date_hour (import_batch_id,stat_date,hour_of_day), ALGORITHM=INPLACE, LOCK=NONE"
        ),
    )?;
    crate::import_pipeline_commands::record_aggregation_partition_progress(
        &req.settings,
        pipeline_run_id,
        "info",
        &format!(
            "小时分片索引完成：table={table}；index=ix_batch_date_hour；duration_ms={}",
            started.elapsed().as_millis().min(i64::MAX as u128)
        ),
    );
    Ok(())
}

fn run_has_rows(req: &EtlRequest, logical_table: &str, run_id: &str) -> Result<bool, String> {
    let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, logical_table)?;
    let mut conn = crate::db::conn(&req.settings)?;
    let exists: Option<i8> = conn
        .exec_first(
            format!("SELECT EXISTS(SELECT 1 FROM `{table}` WHERE analysis_run_id=? LIMIT 1)"),
            (run_id,),
        )
        .map_err(|err| format!("failed to inspect reusable {logical_table} result: {err}"))?;
    Ok(exists.unwrap_or(0) != 0)
}

fn record_reused_result(req: &EtlRequest, pipeline_run_id: &str, run_id: &str, label: &str) {
    crate::import_pipeline_commands::record_aggregation_partition_progress(
        &req.settings,
        pipeline_run_id,
        "info",
        &format!("复用已提交结果：{label}；analysis_run_id={run_id}"),
    );
}

fn prepare_checkpoints(
    req: &EtlRequest,
    pipeline_run_id: &str,
    run_id: &str,
    partitions: &[HourPartition],
) -> Result<(), String> {
    let mut conn = crate::db::conn(&req.settings)?;
    conn.exec_drop(
        "UPDATE meta_aggregation_partition_checkpoint SET status='interrupted',finished_at=UTC_TIMESTAMP(),error_summary=COALESCE(error_summary,'previous application or MySQL process ended before partition completion'),updated_at=UTC_TIMESTAMP() WHERE analysis_run_id=? AND stage_name=? AND subtask_name=? AND status='running'",
        (run_id, HOURLY_STAGE, HOURLY_SUBTASK),
    )
    .map_err(|err| format!("failed to recover interrupted hourly checkpoints: {err}"))?;
    for partition in partitions {
        conn.exec_drop(
            "INSERT INTO meta_aggregation_partition_checkpoint (pipeline_run_id,import_batch_id,analysis_run_id,stage_name,subtask_name,partition_date,partition_hour,status) VALUES (?,?,?,?,?,?,?,'pending') ON DUPLICATE KEY UPDATE pipeline_run_id=VALUES(pipeline_run_id),import_batch_id=VALUES(import_batch_id),updated_at=UTC_TIMESTAMP()",
            (
                pipeline_run_id,
                &req.import_batch_id,
                run_id,
                HOURLY_STAGE,
                HOURLY_SUBTASK,
                &partition.stat_date,
                partition.hour_of_day,
            ),
        )
        .map_err(|err| format!("failed to initialize hourly checkpoint: {err}"))?;
    }
    Ok(())
}

fn checkpoint_status(
    req: &EtlRequest,
    run_id: &str,
    partition: &HourPartition,
) -> Result<String, String> {
    let mut conn = crate::db::conn(&req.settings)?;
    conn.exec_first(
        "SELECT status FROM meta_aggregation_partition_checkpoint WHERE analysis_run_id=? AND stage_name=? AND subtask_name=? AND partition_date=? AND partition_hour=?",
        (
            run_id,
            HOURLY_STAGE,
            HOURLY_SUBTASK,
            &partition.stat_date,
            partition.hour_of_day,
        ),
    )
    .map_err(|err| format!("failed to read hourly checkpoint: {err}"))?
    .ok_or_else(|| "hourly checkpoint disappeared after initialization".to_string())
}

fn mark_checkpoint_running(
    req: &EtlRequest,
    pipeline_run_id: &str,
    run_id: &str,
    partition: &HourPartition,
    connection_id: u64,
) -> Result<(), String> {
    let mut conn = crate::db::conn(&req.settings)?;
    conn.exec_drop(
        "UPDATE meta_aggregation_partition_checkpoint SET pipeline_run_id=?,status='running',attempt_count=attempt_count+1,connection_id=?,started_at=UTC_TIMESTAMP(),finished_at=NULL,duration_ms=0,affected_rows=NULL,error_summary=NULL,updated_at=UTC_TIMESTAMP() WHERE analysis_run_id=? AND stage_name=? AND subtask_name=? AND partition_date=? AND partition_hour=?",
        (
            pipeline_run_id,
            connection_id,
            run_id,
            HOURLY_STAGE,
            HOURLY_SUBTASK,
            &partition.stat_date,
            partition.hour_of_day,
        ),
    )
    .map_err(|err| format!("failed to mark hourly checkpoint running: {err}"))
}

fn mark_checkpoint_finished(
    req: &EtlRequest,
    run_id: &str,
    partition: &HourPartition,
    status: &str,
    duration_ms: i64,
    affected_rows: Option<u64>,
    error_summary: Option<&str>,
) -> Result<(), String> {
    let mut conn = crate::db::conn(&req.settings)?;
    conn.exec_drop(
        "UPDATE meta_aggregation_partition_checkpoint SET status=?,finished_at=UTC_TIMESTAMP(),duration_ms=?,affected_rows=?,error_summary=?,updated_at=UTC_TIMESTAMP() WHERE analysis_run_id=? AND stage_name=? AND subtask_name=? AND partition_date=? AND partition_hour=?",
        (
            status,
            duration_ms,
            affected_rows,
            error_summary,
            run_id,
            HOURLY_STAGE,
            HOURLY_SUBTASK,
            &partition.stat_date,
            partition.hour_of_day,
        ),
    )
    .map_err(|err| format!("failed to finalize hourly checkpoint: {err}"))
}

fn materialize_hourly_partitions(
    req: &EtlRequest,
    pipeline_run_id: &str,
    run_id: &str,
) -> Result<PartitionSummary, String> {
    crate::migrations::ensure_aggregation_checkpoint_schema(&req.settings)?;
    ensure_hourly_partition_index(req, pipeline_run_id, "dwd_tcp_detail_clean")?;
    ensure_hourly_partition_index(req, pipeline_run_id, "dwd_game_detail_clean")?;
    let partitions = discover_partitions(req)?;
    prepare_checkpoints(req, pipeline_run_id, run_id, &partitions)?;
    let mut summary = PartitionSummary {
        total: partitions.len(),
        ..PartitionSummary::default()
    };
    for (index, partition) in partitions.iter().enumerate() {
        if checkpoint_status(req, run_id, partition)? == "success" {
            summary.skipped += 1;
            crate::import_pipeline_commands::record_aggregation_partition_progress(
                &req.settings,
                pipeline_run_id,
                "info",
                &format!(
                "小时分片已完成，跳过重算：partition={}T{:02}:00；progress={}/{}；completed={}；skipped={}",
                partition.stat_date,
                partition.hour_of_day,
                index + 1,
                summary.total,
                summary.completed,
                summary.skipped
            ));
            continue;
        }
        crate::import_pipeline_commands::record_aggregation_partition_progress(
            &req.settings,
            pipeline_run_id,
            "info",
            &format!(
                "小时分片开始：partition={}T{:02}:00；progress={}/{}；analysis_run_id={run_id}",
                partition.stat_date,
                partition.hour_of_day,
                index + 1,
                summary.total
            ),
        );
        let bound = sql_runner::bind_batch_params(
            EXPERIENCE_HOURLY_DWS_V2_SQL,
            &req.import_batch_id,
            Some(run_id),
        );
        let bound = bind_partition_params(&bound, partition);
        let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
        let started = std::time::Instant::now();
        let result =
            sql_runner::execute_script_transactional(&req.settings, &sql, |connection_id| {
                mark_checkpoint_running(req, pipeline_run_id, run_id, partition, connection_id)
            });
        let duration_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
        match result {
            Ok(report) => {
                mark_checkpoint_finished(
                    req,
                    run_id,
                    partition,
                    "success",
                    duration_ms,
                    Some(report.affected_rows),
                    None,
                )?;
                summary.completed += 1;
                summary.affected_rows = summary.affected_rows.saturating_add(report.affected_rows);
                crate::import_pipeline_commands::record_aggregation_partition_progress(
                    &req.settings,
                    pipeline_run_id,
                    "info",
                    &format!(
                    "小时分片完成：partition={}T{:02}:00；progress={}/{}；connection_id={}；duration_ms={duration_ms}；affected_rows={}",
                    partition.stat_date,
                    partition.hour_of_day,
                    index + 1,
                    summary.total,
                    report.connection_id,
                    report.affected_rows
                ));
            }
            Err(err) => {
                let error_summary = err.chars().take(1000).collect::<String>();
                let _ = mark_checkpoint_finished(
                    req,
                    run_id,
                    partition,
                    "failed",
                    duration_ms,
                    None,
                    Some(&error_summary),
                );
                crate::import_pipeline_commands::record_aggregation_partition_progress(
                    &req.settings,
                    pipeline_run_id,
                    "error",
                    &format!(
                    "小时分片失败：partition={}T{:02}:00；progress={}/{}；duration_ms={duration_ms}；error={error_summary}",
                    partition.stat_date,
                    partition.hour_of_day,
                    index + 1,
                    summary.total
                ));
                return Err(format!(
                    "hourly V2 partition failed at {} {:02}:00 ({}/{}): {err}",
                    partition.stat_date,
                    partition.hour_of_day,
                    index + 1,
                    summary.total
                ));
            }
        }
    }
    Ok(summary)
}

pub(crate) fn analytics_materialize_app_rank_for_pipeline(
    req: EtlRequest,
    pipeline_run_id: &str,
) -> Result<CommandAck, String> {
    crate::migrations::ensure_experience_policy_schema(&req.settings)?;
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let run_id = req
        .analysis_run_id
        .clone()
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let rows = if run_has_rows(&req, "ads_app_experience_rank", &run_id)? {
        record_reused_result(&req, pipeline_run_id, &run_id, "legacy App Rank");
        0
    } else {
        let bound = sql_runner::bind_batch_params(SQL, &req.import_batch_id, Some(&run_id));
        let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
        sql_runner::execute_script(&req.settings, &sql)?
    };
    let period_ready = run_has_rows(&req, "dws_user_app_period_experience_v2", &run_id)?
        && run_has_rows(&req, "dws_app_access_period_experience_v2", &run_id)?;
    let dws_v2_rows = if period_ready {
        record_reused_result(&req, pipeline_run_id, &run_id, "Period V2 DWS");
        0
    } else {
        let dws_v2_bound = sql_runner::bind_batch_params(
            EXPERIENCE_DWS_V2_SQL,
            &req.import_batch_id,
            Some(&run_id),
        );
        let dws_v2_sql =
            batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &dws_v2_bound)?;
        sql_runner::execute_script(&req.settings, &dws_v2_sql)?
    };
    let ads_v2_rows = if run_has_rows(&req, "ads_app_experience_v2", &run_id)? {
        record_reused_result(&req, pipeline_run_id, &run_id, "App ADS V2");
        0
    } else {
        let ads_v2_bound = sql_runner::bind_batch_params(
            EXPERIENCE_ADS_V2_SQL,
            &req.import_batch_id,
            Some(&run_id),
        );
        let ads_v2_sql =
            batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &ads_v2_bound)?;
        sql_runner::execute_script(&req.settings, &ads_v2_sql)?
    };
    let hourly = materialize_hourly_partitions(&req, pipeline_run_id, &run_id)?;
    let hourly_ads_rows = if hourly.completed == 0
        && hourly.skipped == hourly.total
        && run_has_rows(&req, "ads_app_hourly_experience_v2", &run_id)?
    {
        record_reused_result(&req, pipeline_run_id, &run_id, "Hourly ADS V2");
        0
    } else {
        let hourly_ads_bound = sql_runner::bind_batch_params(
            EXPERIENCE_HOURLY_ADS_V2_SQL,
            &req.import_batch_id,
            Some(&run_id),
        );
        let hourly_ads_sql = batch_tables::bind_batch_tables(
            &req.settings,
            &req.import_batch_id,
            &hourly_ads_bound,
        )?;
        sql_runner::execute_script(&req.settings, &hourly_ads_sql)?
    };
    Ok(ack(format!(
        "analytics app rank materialized: analysis_run_id={run_id}; legacy_rows={rows}; experience_v2_dws_rows={dws_v2_rows}; experience_v2_ads_rows={ads_v2_rows}; hourly_partitions={}; hourly_completed={}; hourly_skipped={}; hourly_dws_rows={}; hourly_ads_rows={hourly_ads_rows}",
        hourly.total,
        hourly.completed,
        hourly.skipped,
        hourly.affected_rows
    )))
}

#[tauri::command]
pub fn analytics_materialize_app_rank(req: EtlRequest) -> Result<CommandAck, String> {
    let _lock = crate::db::acquire_named_lock(&req.settings, crate::db::AGGREGATION_LOCK_NAME)?;
    let pipeline_run_id = format!("PIPE_MANUAL_{}", Uuid::new_v4().simple());
    analytics_materialize_app_rank_for_pipeline(req, &pipeline_run_id)
}

#[cfg(test)]
mod tests {
    use super::{
        bind_partition_params, HourPartition, EXPERIENCE_DWS_V2_SQL,
        EXPERIENCE_HOURLY_DWS_V2_SQL,
    };

    #[test]
    fn period_v2_keeps_reusable_duration_and_rate_evidence() {
        for field in [
            "total_effective_duration_hours",
            "total_video_duration_hours",
            "active_days",
            "avg_download_mbps",
            "avg_throughput_mbps",
            "avg_connection_success_pct",
        ] {
            assert!(EXPERIENCE_DWS_V2_SQL.contains(field), "missing {field}");
        }
    }

    #[test]
    fn hourly_v2_template_is_bounded_by_date_and_hour() {
        assert!(
            EXPERIENCE_HOURLY_DWS_V2_SQL
                .matches("d.stat_date=:partition_date")
                .count()
                >= 2
        );
        assert!(
            EXPERIENCE_HOURLY_DWS_V2_SQL
                .matches("d.hour_of_day=:partition_hour")
                .count()
                >= 2
        );
        assert!(EXPERIENCE_HOURLY_DWS_V2_SQL.contains("stat_date=:partition_date"));
        assert!(EXPERIENCE_HOURLY_DWS_V2_SQL.contains("hour_of_day=:partition_hour"));
        assert!(EXPERIENCE_HOURLY_DWS_V2_SQL.contains("total_effective_duration_hours"));
        assert!(EXPERIENCE_HOURLY_DWS_V2_SQL.contains("observation_rows"));
    }

    #[test]
    fn partition_binding_removes_all_partition_placeholders() {
        let partition = HourPartition {
            stat_date: "2026-08-20".to_string(),
            hour_of_day: 13,
        };
        let bound = bind_partition_params(EXPERIENCE_HOURLY_DWS_V2_SQL, &partition);
        assert!(!bound.contains(":partition_date"));
        assert!(!bound.contains(":partition_hour"));
        assert!(bound.contains("'2026-08-20'"));
        assert!(bound.contains("hour_of_day=13"));
    }
}

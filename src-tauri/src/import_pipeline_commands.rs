use mysql::prelude::*;
use uuid::Uuid;

use crate::db;
use crate::models::{
    EtlRequest, ImportCurrentFileRequest, ImportPipelineLogRow, ImportPipelineLogsRequest,
    ImportPipelineRebuildRequest, ImportPipelineResumeRequest, ImportPipelineStartRequest,
    ImportPipelineStartResult, ImportPipelineStatus, ImportPipelineStatusRequest,
    ImportPipelineStepRow, MySqlSettings,
};
use crate::sql_runner;

const PIPELINE_SCHEMA: &str = include_str!("../../database/migrations/005_pipeline_schema.sql");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PipelineOutcome {
    Failed,
    Degraded,
}

#[derive(Clone, Copy)]
struct PipelineStepDef {
    name: &'static str,
    label: &'static str,
}

const PIPELINE_STEPS: &[PipelineStepDef] = &[
    PipelineStepDef {
        name: "prepare_environment",
        label: "导入准备",
    },
    PipelineStepDef {
        name: "probe_csv",
        label: "CSV 探测",
    },
    PipelineStepDef {
        name: "import_current_file_atomic",
        label: "字段映射与 RAW 入库",
    },
    PipelineStepDef {
        name: "raw_to_clean",
        label: "CLEAN/DWD 生成",
    },
    PipelineStepDef {
        name: "raw_quality_gate",
        label: "CLEAN 质量验证",
    },
    PipelineStepDef {
        name: "dws_ads_aggregate",
        label: "DWS/ADS 聚合",
    },
    PipelineStepDef {
        name: "final_fusion_optional",
        label: "Final Lead 融合（可降级）",
    },
    PipelineStepDef {
        name: "module_ready",
        label: "Module Ready",
    },
    PipelineStepDef {
        name: "finish",
        label: "完成",
    },
];

const RESUME_PIPELINE_STEPS: &[PipelineStepDef] = &[
    PipelineStepDef {
        name: "prepare_resume",
        label: "复用批次检查",
    },
    PipelineStepDef {
        name: "dws_ads_aggregate",
        label: "完整 DWS/ADS 聚合",
    },
    PipelineStepDef {
        name: "final_fusion_optional",
        label: "Final Lead 融合（可降级）",
    },
    PipelineStepDef {
        name: "module_ready",
        label: "Module Ready",
    },
    PipelineStepDef {
        name: "finish",
        label: "完成",
    },
];

const REBUILD_PIPELINE_STEPS: &[PipelineStepDef] = &[
    PipelineStepDef {
        name: "prepare_rebuild",
        label: "RAW 重建检查",
    },
    PipelineStepDef {
        name: "raw_to_clean",
        label: "CLEAN/DWD 重建",
    },
    PipelineStepDef {
        name: "raw_quality_gate",
        label: "CLEAN 质量验证",
    },
    PipelineStepDef {
        name: "dws_ads_aggregate",
        label: "DWS/ADS/V2 重建",
    },
    PipelineStepDef {
        name: "final_fusion_optional",
        label: "Final Lead 融合（可降级）",
    },
    PipelineStepDef {
        name: "module_ready",
        label: "Module Ready",
    },
    PipelineStepDef {
        name: "finish",
        label: "完成",
    },
];

const AGGREGATE_SUBTASKS: &[&str] = &[
    "base_user_daily",
    "complete_dws",
    "base_dashboards",
    "app_rank",
    "hourly_trend",
    "network_hotspot",
    "user_profile",
    "decision_opportunities",
    "lead_evidence",
];

fn now_elapsed_ms(started: std::time::Instant) -> i64 {
    started.elapsed().as_millis().min(i64::MAX as u128) as i64
}

fn source_file_name(file_path: &str) -> String {
    std::path::Path::new(file_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string())
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * KIB;
    const GIB: f64 = 1024.0 * MIB;
    let value = bytes as f64;
    if value >= GIB {
        format!("{:.2} GiB", value / GIB)
    } else if value >= MIB {
        format!("{:.1} MiB", value / MIB)
    } else if value >= KIB {
        format!("{:.1} KiB", value / KIB)
    } else {
        format!("{bytes} B")
    }
}

fn raw_import_heartbeat_message(mode: &str, transferred_bytes: u64, total_bytes: u64) -> String {
    let streaming_insert = matches!(mode, "streaming_insert" | "insert" | "fallback");
    let action = if streaming_insert {
        "Streaming INSERT 正在读取源文件"
    } else {
        "LOAD DATA 正在传输客户端文件"
    };
    if total_bytes == 0 {
        return format!(
            "{action}：已处理 {}；源文件大小未知",
            format_bytes(transferred_bytes)
        );
    }
    let percent = (transferred_bytes as f64 / total_bytes as f64 * 100.0).min(100.0);
    if transferred_bytes >= total_bytes {
        if streaming_insert {
            format!(
                "源文件已读取完成：{} / {}（100%）；正在等待剩余批次写入与 MySQL 提交",
                format_bytes(transferred_bytes),
                format_bytes(total_bytes)
            )
        } else {
            format!(
                "客户端文件已传输完成：{} / {}（100%）；正在等待 MySQL 解析、索引更新与提交",
                format_bytes(transferred_bytes),
                format_bytes(total_bytes)
            )
        }
    } else {
        format!(
            "{action}：{} / {}（{percent:.1}%）",
            format_bytes(transferred_bytes),
            format_bytes(total_bytes)
        )
    }
}

fn raw_import_stall_hint(transferred_bytes: u64, total_bytes: u64) -> &'static str {
    if transferred_bytes == 0 {
        "客户端字节连续 30 秒未变化；请检查文件是否仍可读、MySQL LOCAL INFILE 请求或磁盘状态"
    } else if total_bytes > 0 && transferred_bytes >= total_bytes {
        "文件已传完但 MySQL 连续 30 秒未返回；可能仍在解析、更新索引或提交，可用 SHOW PROCESSLIST 确认"
    } else {
        "客户端传输字节连续 30 秒未增长；请检查磁盘读取、MySQL 连接和安全软件"
    }
}

fn format_elapsed(elapsed_ms: i64) -> String {
    let seconds = elapsed_ms.max(0) / 1_000;
    let minutes = seconds / 60;
    let rest = seconds % 60;
    if minutes > 0 {
        format!("{minutes} 分 {rest} 秒")
    } else {
        format!("{rest} 秒")
    }
}

fn step_heartbeat_message(step: PipelineStepDef, elapsed_ms: i64) -> String {
    let phase = match step.name {
        "raw_quality_gate" => "MySQL 正在复用 CLEAN/DWD 字段计算完整性、身份、应用和拓扑质量指标",
        "raw_to_clean" => "MySQL 正在按 RAW 主键分块生成 CLEAN/DWD；完成后将一次性重建查询索引",
        "dws_ads_aggregate" => {
            "应用工作线程正在协调 DWS/ADS；具体 SQL、连接和小时分片状态请查看执行日志"
        }
        "final_fusion_optional" => "MySQL 正在融合 CRM、覆盖与可触达资格数据",
        "module_ready" => "系统正在检查结果表和模块可用性",
        "prepare_environment" => "系统正在初始化数据库结构和字段映射目录",
        _ => "任务仍在后端运行",
    };
    format!(
        "{}状态心跳：{}；本步骤已持续 {}；该心跳仅表示应用线程可写日志，不等同于当前 SQL 一定存活",
        step.label,
        phase,
        format_elapsed(elapsed_ms)
    )
}

fn spawn_raw_import_reporter(
    settings: MySqlSettings,
    pipeline_run_id: String,
    step_index: i32,
    progress: crate::raw_import_v2::RawLoadProgress,
    mode: String,
    total_bytes: u64,
    total_started: std::time::Instant,
) -> (std::sync::mpsc::Sender<()>, std::thread::JoinHandle<()>) {
    let (stop_tx, stop_rx) = std::sync::mpsc::channel();
    let reporter = std::thread::spawn(move || {
        let mut last_bytes = progress.transferred_bytes();
        let mut stagnant_intervals = 0_u8;
        loop {
            match stop_rx.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let transferred_bytes = progress.transferred_bytes();
                    if transferred_bytes == last_bytes {
                        stagnant_intervals = stagnant_intervals.saturating_add(1);
                    } else {
                        stagnant_intervals = 0;
                        last_bytes = transferred_bytes;
                    }
                    let stalled = stagnant_intervals >= 6;
                    let mut message =
                        raw_import_heartbeat_message(&mode, transferred_bytes, total_bytes);
                    if stalled {
                        message.push_str("；");
                        message.push_str(raw_import_stall_hint(transferred_bytes, total_bytes));
                        stagnant_intervals = 0;
                    }
                    let elapsed_ms = now_elapsed_ms(total_started);
                    let _ = update_running_step_heartbeat(
                        &settings,
                        &pipeline_run_id,
                        step_index,
                        &message,
                        elapsed_ms,
                        elapsed_ms,
                    );
                    let _ = append_log(
                        &settings,
                        &pipeline_run_id,
                        if stalled { "warning" } else { "info" },
                        Some("import_current_file_atomic"),
                        &message,
                        elapsed_ms,
                    );
                }
            }
        }
    });
    (stop_tx, reporter)
}

fn spawn_step_reporter(
    settings: MySqlSettings,
    pipeline_run_id: String,
    step_index: i32,
    step: PipelineStepDef,
    step_started: std::time::Instant,
    total_started: std::time::Instant,
) -> (std::sync::mpsc::Sender<()>, std::thread::JoinHandle<()>) {
    let (stop_tx, stop_rx) = std::sync::mpsc::channel();
    let reporter = std::thread::spawn(move || loop {
        match stop_rx.recv_timeout(std::time::Duration::from_secs(15)) {
            Ok(_) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let step_elapsed_ms = now_elapsed_ms(step_started);
                let total_elapsed_ms = now_elapsed_ms(total_started);
                let message = step_heartbeat_message(step, step_elapsed_ms);
                let _ = update_running_step_heartbeat(
                    &settings,
                    &pipeline_run_id,
                    step_index,
                    &message,
                    step_elapsed_ms,
                    total_elapsed_ms,
                );
                let _ = append_log(
                    &settings,
                    &pipeline_run_id,
                    "info",
                    Some(step.name),
                    &message,
                    total_elapsed_ms,
                );
            }
        }
    });
    (stop_tx, reporter)
}

fn ensure_pipeline_schema(settings: &MySqlSettings) -> Result<(), String> {
    sql_runner::execute_script(settings, PIPELINE_SCHEMA)?;
    crate::migrations::ensure_aggregation_checkpoint_schema(settings)
}

#[cfg(test)]
fn pipeline_plan() -> &'static [PipelineStepDef] {
    PIPELINE_STEPS
}

fn final_status_for_step_failure(step_name: &str) -> PipelineOutcome {
    if step_name == "final_fusion_optional" {
        PipelineOutcome::Degraded
    } else {
        PipelineOutcome::Failed
    }
}

fn insert_pipeline_run(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    req: &ImportPipelineStartRequest,
    analysis_run_id: &str,
) -> Result<(), String> {
    ensure_pipeline_schema(settings)?;
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "INSERT INTO meta_pipeline_run (pipeline_run_id, analysis_run_id, data_type, source_file_name, batch_display_name, status, total_steps, completed_steps, percent, message, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'pending', ?, 0, 0, 'pipeline queued', UTC_TIMESTAMP(), UTC_TIMESTAMP())",
        (
            pipeline_run_id,
            analysis_run_id,
            &req.data_type,
            source_file_name(&req.file_path),
            &req.batch_display_name,
            PIPELINE_STEPS.len() as i32,
        ),
    )
    .map_err(|err| format!("failed to create pipeline run: {err}"))?;
    for (index, step) in PIPELINE_STEPS.iter().enumerate() {
        conn.exec_drop(
            "INSERT INTO meta_pipeline_step (pipeline_run_id, step_index, step_name, step_label, status, message) VALUES (?, ?, ?, ?, 'pending', 'waiting')",
            (pipeline_run_id, (index + 1) as i32, step.name, step.label),
        )
        .map_err(|err| format!("failed to create pipeline step {}: {err}", step.name))?;
    }
    append_log(
        settings,
        pipeline_run_id,
        "info",
        Some("start"),
        "pipeline execution plan created",
        0,
    )?;
    Ok(())
}

fn append_log(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    level: &str,
    step_name: Option<&str>,
    message: &str,
    elapsed_ms: i64,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    let mut last_duplicate = None;
    for _ in 0..5 {
        let seq: Option<i64> = conn
            .exec_first(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM meta_pipeline_log WHERE pipeline_run_id=?",
                (pipeline_run_id,),
            )
            .map_err(|err| format!("failed to read pipeline log seq: {err}"))?;
        match conn.exec_drop(
            "INSERT INTO meta_pipeline_log (pipeline_run_id, seq, ts, level, step_name, message, elapsed_ms) VALUES (?, ?, UTC_TIMESTAMP(), ?, ?, ?, ?)",
            (pipeline_run_id, seq.unwrap_or(1), level, step_name, message, elapsed_ms),
        ) {
            Ok(()) => return Ok(()),
            Err(err) => {
                let text = err.to_string();
                if text.contains("Duplicate entry") || text.contains("1062") {
                    last_duplicate = Some(text);
                    std::thread::yield_now();
                    continue;
                }
                return Err(format!("failed to append pipeline log: {err}"));
            }
        }
    }
    Err(format!(
        "failed to append pipeline log after sequence retries: {}",
        last_duplicate.unwrap_or_else(|| "unknown duplicate sequence error".to_string())
    ))
}

pub(crate) fn record_aggregation_partition_progress(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    level: &str,
    message: &str,
) {
    crate::append_runtime_log(&format!(
        "aggregation_partition pipeline_run_id={pipeline_run_id} {message}"
    ));
    let _ = append_log(
        settings,
        pipeline_run_id,
        level,
        Some("dws_ads_aggregate"),
        message,
        0,
    );
    if let Ok(mut conn) = db::conn(settings) {
        let _ = conn.exec_drop(
            "UPDATE meta_pipeline_run SET message=?,updated_at=UTC_TIMESTAMP() WHERE pipeline_run_id=? AND status='running'",
            (message, pipeline_run_id),
        );
        let _ = conn.exec_drop(
            "UPDATE meta_pipeline_step SET message=? WHERE pipeline_run_id=? AND step_name='dws_ads_aggregate' AND status='running'",
            (message, pipeline_run_id),
        );
    }
}

fn run_with_sql_logging<T, F>(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    stage: &'static str,
    total_started: std::time::Instant,
    action: F,
) -> T
where
    F: FnOnce() -> T,
{
    let observed_settings = settings.clone();
    let observed_pipeline_run_id = pipeline_run_id.to_string();
    sql_runner::with_sql_execution_observer(
        move |event| {
            let level = if event.status == "failed" {
                "error"
            } else {
                "info"
            };
            let status = event.status.to_ascii_uppercase();
            let affected = event
                .affected_rows
                .map(|rows| rows.to_string())
                .unwrap_or_else(|| "-".to_string());
            let error = event
                .error
                .as_deref()
                .map(|value| format!("；error={value}"))
                .unwrap_or_default();
            let message = format!(
                "SQL {}/{} {status}；stage={stage}；connection_id={}；duration_ms={}；affected_rows={affected}；statement={}{}",
                event.statement_index,
                event.statement_count,
                event.connection_id.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string()),
                event.duration_ms,
                event.statement_preview,
                error,
            );
            crate::append_runtime_log(&format!(
                "pipeline_sql pipeline_run_id={} {}",
                observed_pipeline_run_id, message
            ));
            let _ = append_log(
                &observed_settings,
                &observed_pipeline_run_id,
                level,
                Some(stage),
                &message,
                now_elapsed_ms(total_started),
            );
        },
        action,
    )
}

fn update_run(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    status: &str,
    current_step: Option<&str>,
    message: Option<&str>,
    error_message: Option<&str>,
    final_fusion_status: Option<&str>,
    elapsed_ms: i64,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    let progress: Option<(i64, i64)> = conn
        .exec_first(
            "SELECT CAST(COUNT(*) AS SIGNED), CAST(COALESCE(MAX(r.total_steps), 0) AS SIGNED) FROM meta_pipeline_step s JOIN meta_pipeline_run r ON r.pipeline_run_id=s.pipeline_run_id WHERE s.pipeline_run_id=? AND s.status IN ('success','skipped','degraded')",
            (pipeline_run_id,),
        )
        .map_err(|err| format!("failed to count completed pipeline steps: {err}"))?;
    let (completed, total_steps) = progress.unwrap_or((0, 0));
    let percent = if total_steps <= 0 {
        0.0
    } else {
        completed as f64 / total_steps as f64 * 100.0
    };
    let finished_expr = if matches!(status, "success" | "failed" | "degraded" | "canceled") {
        ", finished_at=COALESCE(finished_at, UTC_TIMESTAMP())"
    } else {
        ""
    };
    conn.exec_drop(
        format!(
            "UPDATE meta_pipeline_run SET status=?, current_step=?, completed_steps=?, percent=?, elapsed_ms=?, message=COALESCE(?, message), error_message=COALESCE(?, error_message), final_fusion_status=COALESCE(?, final_fusion_status), started_at=COALESCE(started_at, UTC_TIMESTAMP()), updated_at=UTC_TIMESTAMP(){finished_expr} WHERE pipeline_run_id=?"
        ),
        (
            status,
            current_step,
            completed,
            percent,
            elapsed_ms,
            message,
            error_message,
            final_fusion_status,
            pipeline_run_id,
        ),
    )
    .map_err(|err| format!("failed to update pipeline run: {err}"))
}

fn update_running_step_heartbeat(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    step_index: i32,
    message: &str,
    step_elapsed_ms: i64,
    total_elapsed_ms: i64,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "UPDATE meta_pipeline_step SET elapsed_ms=?, message=? WHERE pipeline_run_id=? AND step_index=? AND status='running'",
        (step_elapsed_ms, message, pipeline_run_id, step_index),
    )
    .map_err(|err| format!("failed to update pipeline step heartbeat: {err}"))?;
    conn.exec_drop(
        "UPDATE meta_pipeline_run SET elapsed_ms=?, message=?, updated_at=UTC_TIMESTAMP() WHERE pipeline_run_id=? AND status='running'",
        (total_elapsed_ms, message, pipeline_run_id),
    )
    .map_err(|err| format!("failed to update pipeline run heartbeat: {err}"))
}

fn update_batch_id(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    import_batch_id: &str,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "UPDATE meta_pipeline_run SET import_batch_id=?, updated_at=UTC_TIMESTAMP() WHERE pipeline_run_id=?",
        (import_batch_id, pipeline_run_id),
    )
    .map_err(|err| format!("failed to update pipeline batch id: {err}"))
}

fn start_step(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    step_index: i32,
    step_name: &str,
    message: &str,
    elapsed_ms: i64,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "UPDATE meta_pipeline_step SET status='running', started_at=COALESCE(started_at, UTC_TIMESTAMP()), finished_at=NULL, elapsed_ms=0, message=?, error_message=NULL WHERE pipeline_run_id=? AND step_index=?",
        (message, pipeline_run_id, step_index),
    )
    .map_err(|err| format!("failed to start pipeline step {step_name}: {err}"))?;
    update_run(
        settings,
        pipeline_run_id,
        "running",
        Some(step_name),
        Some(message),
        None,
        None,
        elapsed_ms,
    )?;
    append_log(
        settings,
        pipeline_run_id,
        "info",
        Some(step_name),
        message,
        elapsed_ms,
    )
}

fn finish_step(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    step_index: i32,
    step_name: &str,
    status: &str,
    message: &str,
    error_message: Option<&str>,
    step_elapsed_ms: i64,
    total_elapsed_ms: i64,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "UPDATE meta_pipeline_step SET status=?, finished_at=UTC_TIMESTAMP(), elapsed_ms=?, message=?, error_message=? WHERE pipeline_run_id=? AND step_index=?",
        (status, step_elapsed_ms, message, error_message, pipeline_run_id, step_index),
    )
    .map_err(|err| format!("failed to finish pipeline step {step_name}: {err}"))?;
    let level = if status == "failed" {
        "error"
    } else if status == "degraded" {
        "warning"
    } else {
        "info"
    };
    append_log(
        settings,
        pipeline_run_id,
        level,
        Some(step_name),
        message,
        total_elapsed_ms,
    )
}

fn fail_remaining_steps(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    after_step_index: i32,
    message: &str,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "UPDATE meta_pipeline_step SET status='skipped', finished_at=UTC_TIMESTAMP(), elapsed_ms=0, message=? WHERE pipeline_run_id=? AND step_index>? AND status='pending'",
        (message, pipeline_run_id, after_step_index),
    )
    .map_err(|err| format!("failed to skip remaining pipeline steps: {err}"))
}

fn run_observed_step<F>(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    step_index: i32,
    step: PipelineStepDef,
    total_started: std::time::Instant,
    action: F,
) -> Result<Option<String>, (PipelineOutcome, String)>
where
    F: FnOnce() -> Result<Option<String>, String>,
{
    let step_started = std::time::Instant::now();
    if let Err(err) = start_step(
        settings,
        pipeline_run_id,
        step_index,
        step.name,
        step.label,
        now_elapsed_ms(total_started),
    ) {
        return Err((PipelineOutcome::Failed, err));
    }
    let heartbeat = if step.name == "import_current_file_atomic" || step.name == "finish" {
        None
    } else {
        Some(spawn_step_reporter(
            settings.clone(),
            pipeline_run_id.to_string(),
            step_index,
            step,
            step_started,
            total_started,
        ))
    };
    let action_result = action();
    if let Some((stop_reporter, reporter)) = heartbeat {
        let _ = stop_reporter.send(());
        let _ = reporter.join();
    }
    match action_result {
        Ok(message) => {
            let text = message.unwrap_or_else(|| format!("{} completed", step.label));
            if let Err(err) = finish_step(
                settings,
                pipeline_run_id,
                step_index,
                step.name,
                "success",
                &text,
                None,
                now_elapsed_ms(step_started),
                now_elapsed_ms(total_started),
            ) {
                return Err((PipelineOutcome::Failed, err));
            }
            Ok(Some(text))
        }
        Err(err) => {
            let outcome = final_status_for_step_failure(step.name);
            let status = if outcome == PipelineOutcome::Degraded {
                "degraded"
            } else {
                "failed"
            };
            let message = if outcome == PipelineOutcome::Degraded {
                format!("{} degraded: {err}", step.label)
            } else {
                err.clone()
            };
            let _ = finish_step(
                settings,
                pipeline_run_id,
                step_index,
                step.name,
                status,
                &message,
                Some(&err),
                now_elapsed_ms(step_started),
                now_elapsed_ms(total_started),
            );
            Err((outcome, err))
        }
    }
}

fn run_logged_subtask<F>(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    import_batch_id: &str,
    analysis_run_id: &str,
    subtask: &str,
    label: &str,
    total_started: std::time::Instant,
    action: F,
) -> Result<(), String>
where
    F: FnOnce() -> Result<crate::models::CommandAck, String>,
{
    let mut checkpoint_conn = db::conn(settings)?;
    let checkpoint_status: Option<String> = checkpoint_conn.exec_first(
        "SELECT status FROM meta_aggregation_subtask_checkpoint WHERE analysis_run_id=? AND stage_name='dws_ads_aggregate' AND subtask_name=?",
        (analysis_run_id, subtask),
    ).map_err(|err| format!("failed to inspect aggregation subtask checkpoint: {err}"))?;
    if checkpoint_status.as_deref() == Some("success") {
        append_log(settings, pipeline_run_id, "info", Some("dws_ads_aggregate"), &format!("聚合子阶段已完成，断点复用：{label} [{subtask}]"), now_elapsed_ms(total_started))?;
        return Ok(());
    }
    checkpoint_conn.exec_drop(
        "INSERT INTO meta_aggregation_subtask_checkpoint (pipeline_run_id,import_batch_id,analysis_run_id,stage_name,subtask_name,status,attempt_count,started_at,finished_at,duration_ms,message) VALUES (?,?,?,'dws_ads_aggregate',?,'running',1,UTC_TIMESTAMP(),NULL,0,NULL) ON DUPLICATE KEY UPDATE pipeline_run_id=VALUES(pipeline_run_id),import_batch_id=VALUES(import_batch_id),status='running',attempt_count=attempt_count+1,started_at=UTC_TIMESTAMP(),finished_at=NULL,duration_ms=0,message=NULL",
        (pipeline_run_id, import_batch_id, analysis_run_id, subtask),
    ).map_err(|err| format!("failed to start aggregation subtask checkpoint: {err}"))?;
    let started = std::time::Instant::now();
    append_log(
        settings,
        pipeline_run_id,
        "info",
        Some("dws_ads_aggregate"),
        &format!("聚合子阶段开始：{label} [{subtask}]"),
        now_elapsed_ms(total_started),
    )?;
    match action() {
        Ok(result) => {
            checkpoint_conn.exec_drop("UPDATE meta_aggregation_subtask_checkpoint SET status='success',finished_at=UTC_TIMESTAMP(),duration_ms=?,message=? WHERE analysis_run_id=? AND stage_name='dws_ads_aggregate' AND subtask_name=?", (now_elapsed_ms(started), &result.message, analysis_run_id, subtask)).map_err(|err| format!("failed to complete aggregation subtask checkpoint: {err}"))?;
            append_log(
                settings,
                pipeline_run_id,
                "info",
                Some("dws_ads_aggregate"),
                &format!(
                    "聚合子阶段完成：{label} [{subtask}]；耗时={} ms；{}",
                    now_elapsed_ms(started),
                    result.message
                ),
                now_elapsed_ms(total_started),
            )?;
            Ok(())
        }
        Err(err) => {
            let _ = checkpoint_conn.exec_drop("UPDATE meta_aggregation_subtask_checkpoint SET status='failed',finished_at=UTC_TIMESTAMP(),duration_ms=?,message=? WHERE analysis_run_id=? AND stage_name='dws_ads_aggregate' AND subtask_name=?", (now_elapsed_ms(started), err.chars().take(2000).collect::<String>(), analysis_run_id, subtask));
            let _ = append_log(
                settings,
                pipeline_run_id,
                "error",
                Some("dws_ads_aggregate"),
                &format!("聚合子阶段失败：{label} [{subtask}]；{err}"),
                now_elapsed_ms(total_started),
            );
            Err(format!("aggregate subtask {subtask} failed: {err}"))
        }
    }
}

fn run_dws_ads_stage(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    import_batch_id: &str,
    analysis_run_id: &str,
    total_started: std::time::Instant,
) -> Result<String, String> {
    let _aggregation_lock = db::acquire_named_lock(settings, db::AGGREGATION_LOCK_NAME)?;
    crate::migrations::ensure_decision_workspace_schema(settings)?;
    let request = || EtlRequest {
        settings: settings.clone(),
        import_batch_id: import_batch_id.to_string(),
        analysis_run_id: Some(analysis_run_id.to_string()),
    };
    let result = (|| {
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[0],
            "用户日聚合",
            total_started,
            || crate::etl_commands::etl_start_aggregate_job(request()),
        )?;
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[1],
            "完整 DWS 聚合",
            total_started,
            || crate::phase_commands::etl_run_complete_aggregates(request()),
        )?;
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[2],
            "基础看板 ADS",
            total_started,
            || crate::phase_commands::ads_run_complete_dashboards(request()),
        )?;
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[3],
            "App Rank",
            total_started,
            || {
                crate::analytics_ads_app::analytics_materialize_app_rank_for_pipeline(
                    request(),
                    pipeline_run_id,
                )
            },
        )?;
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[4],
            "小时趋势",
            total_started,
            || crate::ads_hour::ads_hour(request()),
        )?;
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[5],
            "网络热点",
            total_started,
            || crate::ads_net::ads_net(request()),
        )?;
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[6],
            "用户画像",
            total_started,
            || crate::ads_user::ads_user(request()),
        )?;
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[7],
            "四类潜客机会",
            total_started,
            || crate::decision_workspace_commands::materialize_opportunities(request()),
        )?;
        run_logged_subtask(
            settings,
            pipeline_run_id,
            import_batch_id,
            analysis_run_id,
            AGGREGATE_SUBTASKS[8],
            "Lead Evidence",
            total_started,
            || crate::ads_lead::ads_lead(request()),
        )?;
        Ok(format!(
            "DWS/ADS ready: analysis_run_id={analysis_run_id}; subtasks={}",
            AGGREGATE_SUBTASKS.len()
        ))
    })();
    match result {
        Ok(message) => {
            crate::etl_commands::mark_analysis_run_status(
                settings,
                analysis_run_id,
                "success",
                &message,
            )?;
            Ok(message)
        }
        Err(err) => {
            let _ = crate::etl_commands::mark_analysis_run_status(
                settings,
                analysis_run_id,
                "failed",
                &format!("complete DWS/ADS failed: {err}"),
            );
            Err(err)
        }
    }
}

fn run_final_fusion_stage(
    settings: &MySqlSettings,
    import_batch_id: &str,
    analysis_run_id: &str,
) -> Result<String, String> {
    let request = || EtlRequest {
        settings: settings.clone(),
        import_batch_id: import_batch_id.to_string(),
        analysis_run_id: Some(analysis_run_id.to_string()),
    };
    crate::phase_commands::leads_run_final_fusion(request())?;
    crate::ads_lead::ads_lead(request())?;
    let final_table = crate::batch_tables::resolve_table(
        settings,
        import_batch_id,
        "ads_final_marketing_lead_user",
    )?;
    let mut conn = db::conn(settings)?;
    if !crate::batch_tables::table_has_analysis_run(&mut conn, &final_table, analysis_run_id)? {
        return Err("Final Lead not generated; likely missing CRM/coverage/reachability, SA Lead remains available".to_string());
    }
    Ok("Final Lead fusion ready".to_string())
}

fn run_module_ready_stage(
    settings: &MySqlSettings,
    import_batch_id: &str,
    analysis_run_id: &str,
) -> Result<String, String> {
    crate::analysis_commands::analysis_prepare_batch_tables(
        settings.clone(),
        import_batch_id.to_string(),
    )?;
    let status = crate::analysis_commands::analysis_get_module_status(
        settings.clone(),
        import_batch_id.to_string(),
        Some(analysis_run_id.to_string()),
    )?;
    Ok(format!(
        "module ready refreshed without full-table counts: enabled={}",
        status.iter().filter(|item| item.enabled).count()
    ))
}

fn run_pipeline_job(
    req: ImportPipelineStartRequest,
    pipeline_run_id: String,
    analysis_run_id: String,
) {
    let total_started = std::time::Instant::now();
    let settings = req.settings.clone();
    let mut import_batch_id: Option<String> = None;
    let mut degraded = false;
    let mut final_fusion_status = "pending".to_string();

    let _ = update_run(
        &settings,
        &pipeline_run_id,
        "running",
        Some("prepare_environment"),
        Some("pipeline running"),
        None,
        None,
        0,
    );

    for (idx, step) in PIPELINE_STEPS.iter().copied().enumerate() {
        let step_index = (idx + 1) as i32;
        let result = match step.name {
            "prepare_environment" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    db::ping(&settings)?;
                    crate::migrations::init_database(&settings)?;
                    crate::mapping_catalog::ensure_import_mapping_catalog(&settings)?;
                    let health = crate::mapping_catalog::check_import_mapping_catalog(&settings)?;
                    Ok(Some(format!(
                        "environment ready; catalog_metrics={}",
                        health.len()
                    )))
                },
            ),
            "probe_csv" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let probe = crate::probe::probe_file(req.file_path.clone())?;
                    Ok(Some(format!(
                        "probe ok: {}, headers={}",
                        probe.file_name,
                        probe.headers.len()
                    )))
                },
            ),
            "import_current_file_atomic" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let total_bytes = std::fs::metadata(&req.file_path)
                        .map(|metadata| metadata.len())
                        .unwrap_or(0);
                    let progress = crate::raw_import_v2::RawLoadProgress::default();
                    let import_mode = req
                        .import_mode
                        .clone()
                        .unwrap_or_else(|| "load_data".to_string())
                        .to_lowercase();
                    let monitor_message = format!(
                        "RAW 导入监控已启动：源文件大小={}；每 5 秒报告客户端传输进度",
                        format_bytes(total_bytes)
                    );
                    append_log(
                        &settings,
                        &pipeline_run_id,
                        "info",
                        Some(step.name),
                        &monitor_message,
                        now_elapsed_ms(total_started),
                    )?;
                    let (stop_reporter, reporter) = spawn_raw_import_reporter(
                        settings.clone(),
                        pipeline_run_id.clone(),
                        step_index,
                        progress.clone(),
                        import_mode,
                        total_bytes,
                        total_started,
                    );
                    let import_result = crate::import_commands::import_current_file_atomic_observed(
                        ImportCurrentFileRequest {
                            settings: settings.clone(),
                            data_type: req.data_type.clone(),
                            file_path: req.file_path.clone(),
                            batch_display_name: req.batch_display_name.clone(),
                            mode: req.import_mode.clone(),
                            access_rule_set_id: req.access_rule_set_id.clone(),
                        },
                        progress,
                        |batch| {
                            update_batch_id(&settings, &pipeline_run_id, &batch.import_batch_id)
                        },
                    );
                    let _ = stop_reporter.send(());
                    let _ = reporter.join();
                    let result = import_result?;
                    import_batch_id = Some(result.batch.import_batch_id.clone());
                    Ok(Some(format!(
                        "RAW import finished: batch={}, mapping_rows={}",
                        result.batch.import_batch_id,
                        result.mapping_results.len()
                    )))
                },
            ),
            "raw_quality_gate" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let batch = import_batch_id
                        .as_ref()
                        .ok_or_else(|| "missing import_batch_id before quality gate".to_string())?;
                    crate::phase_commands::quality_run_gate(EtlRequest {
                        settings: settings.clone(),
                        import_batch_id: batch.clone(),
                        analysis_run_id: None,
                    })?;
                    Ok(Some("Quality Gate finished".to_string()))
                },
            ),
            "raw_to_clean" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let batch = import_batch_id
                        .as_ref()
                        .ok_or_else(|| "missing import_batch_id before clean".to_string())?;
                    crate::etl_commands::etl_start_clean_job(EtlRequest {
                        settings: settings.clone(),
                        import_batch_id: batch.clone(),
                        analysis_run_id: None,
                    })?;
                    Ok(Some("RAW to CLEAN finished".to_string()))
                },
            ),
            "dws_ads_aggregate" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let batch = import_batch_id
                        .as_ref()
                        .ok_or_else(|| "missing import_batch_id before aggregate".to_string())?;
                    run_with_sql_logging(
                        &settings,
                        &pipeline_run_id,
                        step.name,
                        total_started,
                        || {
                            run_dws_ads_stage(
                                &settings,
                                &pipeline_run_id,
                                batch,
                                &analysis_run_id,
                                total_started,
                            )
                        },
                    )
                    .map(Some)
                },
            ),
            "final_fusion_optional" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let batch = import_batch_id
                        .as_ref()
                        .ok_or_else(|| "missing import_batch_id before final fusion".to_string())?;
                    let message = run_final_fusion_stage(&settings, batch, &analysis_run_id)?;
                    final_fusion_status = "success".to_string();
                    Ok(Some(message))
                },
            ),
            "module_ready" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let batch = import_batch_id
                        .as_ref()
                        .ok_or_else(|| "missing import_batch_id before module ready".to_string())?;
                    run_module_ready_stage(&settings, batch, &analysis_run_id).map(Some)
                },
            ),
            "finish" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || Ok(Some("pipeline finished".to_string())),
            ),
            _ => Ok(Some("unknown step skipped".to_string())),
        };

        match result {
            Ok(_) => {}
            Err((PipelineOutcome::Degraded, err)) => {
                degraded = true;
                final_fusion_status = "degraded".to_string();
                let _ = append_log(
                    &settings,
                    &pipeline_run_id,
                    "warning",
                    Some(step.name),
                    &format!("optional step degraded and pipeline continues: {err}"),
                    now_elapsed_ms(total_started),
                );
                continue;
            }
            Err((PipelineOutcome::Failed, err)) => {
                let _ = fail_remaining_steps(
                    &settings,
                    &pipeline_run_id,
                    step_index,
                    "前序步骤失败，后续步骤已跳过。",
                );
                let _ = update_run(
                    &settings,
                    &pipeline_run_id,
                    "failed",
                    Some(step.name),
                    Some("pipeline failed"),
                    Some(&err),
                    Some(&final_fusion_status),
                    now_elapsed_ms(total_started),
                );
                let _ = append_log(
                    &settings,
                    &pipeline_run_id,
                    "error",
                    Some(step.name),
                    &format!("pipeline failed: {err}"),
                    now_elapsed_ms(total_started),
                );
                return;
            }
        }
    }

    let final_status = if degraded { "degraded" } else { "success" };
    let final_message = if degraded {
        "pipeline finished with Final Lead degraded; base DWS/ADS and SA Lead are available"
    } else {
        "pipeline finished successfully"
    };
    let _ = update_run(
        &settings,
        &pipeline_run_id,
        final_status,
        Some("finish"),
        Some(final_message),
        None,
        Some(&final_fusion_status),
        now_elapsed_ms(total_started),
    );
    let _ = append_log(
        &settings,
        &pipeline_run_id,
        if degraded { "warning" } else { "info" },
        Some("finish"),
        final_message,
        now_elapsed_ms(total_started),
    );
}

fn active_batch_statements(
    settings: &MySqlSettings,
    import_batch_id: &str,
) -> Result<Vec<String>, String> {
    let mut conn = db::conn(settings)?;
    let mut needles: Vec<String> = conn
        .exec_map(
            "SELECT physical_table_name FROM meta_batch_table_registry WHERE import_batch_id=?",
            (import_batch_id,),
            |name: String| name.to_ascii_lowercase(),
        )
        .unwrap_or_default();
    needles.push(crate::batch_tables::batch_short_id(import_batch_id));
    let rows: Vec<(u64, u64, Option<String>, Option<String>)> = conn
        .query(
            "SELECT ID, TIME, STATE, INFO FROM information_schema.PROCESSLIST WHERE DB=DATABASE() AND COMMAND<>'Sleep' AND ID<>CONNECTION_ID()",
        )
        .map_err(|err| format!("failed to inspect active MySQL statements: {err}"))?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, seconds, state, info)| {
            let sql = info.unwrap_or_default();
            let normalized = sql.to_ascii_lowercase();
            needles
                .iter()
                .any(|needle| normalized.contains(needle))
                .then(|| {
                    format!(
                        "connection={id}, seconds={seconds}, state={}, sql={}",
                        state.unwrap_or_else(|| "-".to_string()),
                        sql.chars().take(240).collect::<String>()
                    )
                })
        })
        .collect())
}

fn latest_job_status(
    conn: &mut mysql::PooledConn,
    import_batch_id: &str,
    job_type: &str,
) -> Result<Option<String>, String> {
    conn.exec_first(
        "SELECT status FROM meta_etl_job WHERE import_batch_id=? AND job_type=? ORDER BY started_at DESC LIMIT 1",
        (import_batch_id, job_type),
    )
    .map_err(|err| format!("failed to inspect {job_type} readiness: {err}"))
}

fn insert_rebuild_pipeline_run(
    req: &ImportPipelineRebuildRequest,
    pipeline_run_id: &str,
    analysis_run_id: &str,
) -> Result<(), String> {
    ensure_pipeline_schema(&req.settings)?;
    let mut conn = db::conn(&req.settings)?;
    let batch: Option<(String, String, String, String, i64)> = conn
        .exec_first(
            "SELECT data_type, source_file_name, COALESCE(NULLIF(batch_display_name,''), source_file_name), status, CAST(COALESCE(imported_rows,0) AS SIGNED) FROM meta_import_batch WHERE import_batch_id=?",
            (&req.import_batch_id,),
        )
        .map_err(|err| format!("failed to inspect RAW rebuild batch: {err}"))?;
    let Some((data_type, source_file_name, batch_display_name, raw_status, imported_rows)) = batch
    else {
        return Err(format!("import batch not found: {}", req.import_batch_id));
    };
    if !matches!(
        data_type.to_ascii_lowercase().as_str(),
        "tcp" | "game" | "mixed"
    ) {
        return Err(format!(
            "RAW rebuild supports TCP/Game analysis batches only; data_type={data_type}"
        ));
    }
    if raw_status.to_ascii_lowercase() != "success" || imported_rows <= 0 {
        return Err(format!(
            "batch RAW is not reusable: raw_status={raw_status}, imported_rows={imported_rows}"
        ));
    }
    let active_pipeline: Option<String> = conn
        .exec_first(
            "SELECT pipeline_run_id FROM meta_pipeline_run WHERE import_batch_id=? AND status IN ('pending','running') ORDER BY updated_at DESC LIMIT 1",
            (&req.import_batch_id,),
        )
        .map_err(|err| format!("failed to inspect active pipeline: {err}"))?;
    if active_pipeline.is_some() && !req.confirm_original_process_stopped.unwrap_or(false) {
        return Err(format!(
            "batch still has active pipeline {}; wait for completion, or confirm stale takeover after the original EXE exits",
            active_pipeline.as_deref().unwrap_or("unknown")
        ));
    }
    let active_sql = active_batch_statements(&req.settings, &req.import_batch_id)?;
    if !active_sql.is_empty() {
        return Err(format!(
            "RAW rebuild rejected because MySQL is still executing this batch: {}",
            active_sql.join(" | ")
        ));
    }
    if let Some(active_pipeline) = active_pipeline {
        conn.exec_drop(
            "UPDATE meta_pipeline_step SET error_message=CASE WHEN status='running' THEN 'stale pipeline superseded by explicit RAW rebuild' ELSE error_message END, status=CASE WHEN status='running' THEN 'interrupted' ELSE 'skipped' END, finished_at=UTC_TIMESTAMP(), message='原应用进程已退出；用户确认无活动 SQL 后由 RAW 重建任务接管。' WHERE pipeline_run_id=? AND status IN ('pending','running')",
            (&active_pipeline,),
        )
        .map_err(|err| format!("failed to close stale pipeline steps: {err}"))?;
        conn.exec_drop(
            "UPDATE meta_pipeline_run SET status='interrupted', finished_at=UTC_TIMESTAMP(), updated_at=UTC_TIMESTAMP(), message='stale pipeline superseded by explicit RAW rebuild', error_message='original process confirmed stopped; no active batch SQL found' WHERE pipeline_run_id=? AND status IN ('pending','running')",
            (&active_pipeline,),
        )
        .map_err(|err| format!("failed to close stale pipeline: {err}"))?;
    }
    conn.exec_drop(
        "INSERT INTO meta_pipeline_run (pipeline_run_id, import_batch_id, analysis_run_id, data_type, source_file_name, batch_display_name, status, total_steps, completed_steps, percent, message, created_at, updated_at) SELECT ?, ?, ?, ?, ?, ?, 'pending', ?, 0, 0, 'RAW rebuild queued; CSV and RAW import are preserved', UTC_TIMESTAMP(), UTC_TIMESTAMP() WHERE NOT EXISTS (SELECT 1 FROM meta_pipeline_run WHERE import_batch_id=? AND status IN ('pending','running'))",
        (
            pipeline_run_id,
            &req.import_batch_id,
            analysis_run_id,
            data_type,
            source_file_name,
            batch_display_name,
            REBUILD_PIPELINE_STEPS.len() as i32,
            &req.import_batch_id,
        ),
    )
    .map_err(|err| format!("failed to create RAW rebuild pipeline: {err}"))?;
    if conn.affected_rows() == 0 {
        return Err(
            "RAW rebuild rejected because another pipeline became active for this batch"
                .to_string(),
        );
    }
    for (index, step) in REBUILD_PIPELINE_STEPS.iter().enumerate() {
        conn.exec_drop(
            "INSERT INTO meta_pipeline_step (pipeline_run_id, step_index, step_name, step_label, status, message) VALUES (?, ?, ?, ?, 'pending', 'waiting')",
            (pipeline_run_id, (index + 1) as i32, step.name, step.label),
        )
        .map_err(|err| format!("failed to create RAW rebuild step {}: {err}", step.name))?;
    }
    append_log(
        &req.settings,
        pipeline_run_id,
        "info",
        Some("start"),
        &format!(
            "RAW rebuild plan created; import_batch_id={}; analysis_run_id={analysis_run_id}; CSV and RAW import are preserved; CLEAN/DWS/ADS will be regenerated",
            req.import_batch_id
        ),
        0,
    )
}

fn insert_resume_pipeline_run(
    req: &ImportPipelineResumeRequest,
    pipeline_run_id: &str,
    analysis_run_id: &str,
) -> Result<(), String> {
    ensure_pipeline_schema(&req.settings)?;
    let mut conn = db::conn(&req.settings)?;
    let batch: Option<(String, String, String, String, i64)> = conn
        .exec_first(
            "SELECT data_type, source_file_name, COALESCE(NULLIF(batch_display_name,''), source_file_name), status, CAST(COALESCE(imported_rows,0) AS SIGNED) FROM meta_import_batch WHERE import_batch_id=?",
            (&req.import_batch_id,),
        )
        .map_err(|err| format!("failed to inspect reusable batch: {err}"))?;
    let Some((data_type, source_file_name, batch_display_name, raw_status, imported_rows)) = batch
    else {
        return Err(format!("import batch not found: {}", req.import_batch_id));
    };
    if raw_status.to_ascii_lowercase() != "success" || imported_rows <= 0 {
        return Err(format!(
            "batch is not reusable: raw_status={raw_status}, imported_rows={imported_rows}"
        ));
    }
    for job_type in ["quality_gate", "raw_to_clean"] {
        let status = latest_job_status(&mut conn, &req.import_batch_id, job_type)?;
        if !matches!(status.as_deref(), Some("success")) {
            return Err(format!(
                "batch is not ready to resume from DWS/ADS: latest {job_type} status={}",
                status.unwrap_or_else(|| "missing".to_string())
            ));
        }
    }
    let active_pipeline: Option<String> = conn
        .exec_first(
            "SELECT pipeline_run_id FROM meta_pipeline_run WHERE import_batch_id=? AND status IN ('pending','running') ORDER BY updated_at DESC LIMIT 1",
            (&req.import_batch_id,),
        )
        .map_err(|err| format!("failed to inspect active pipeline: {err}"))?;
    if active_pipeline.is_some() && !req.confirm_original_process_stopped.unwrap_or(false) {
        return Err(format!(
            "batch still has active pipeline {}; wait for completion, or close the original EXE and explicitly confirm stale takeover",
            active_pipeline.as_deref().unwrap_or("unknown")
        ));
    }
    let active_sql = active_batch_statements(&req.settings, &req.import_batch_id)?;
    if !active_sql.is_empty() {
        return Err(format!(
            "resume rejected because MySQL is still executing this batch: {}",
            active_sql.join(" | ")
        ));
    }
    if let Some(active_pipeline) = active_pipeline {
        conn.exec_drop(
            "UPDATE meta_pipeline_step SET error_message=CASE WHEN status='running' THEN 'stale pipeline superseded by explicit batch resume' ELSE error_message END, status=CASE WHEN status='running' THEN 'interrupted' ELSE 'skipped' END, finished_at=UTC_TIMESTAMP(), message='原应用进程已退出；用户确认无活动 SQL 后由新续跑任务接管。' WHERE pipeline_run_id=? AND status IN ('pending','running')",
            (&active_pipeline,),
        )
        .map_err(|err| format!("failed to close stale pipeline steps: {err}"))?;
        conn.exec_drop(
            "UPDATE meta_pipeline_run SET status='interrupted', finished_at=UTC_TIMESTAMP(), updated_at=UTC_TIMESTAMP(), message='stale pipeline superseded by explicit batch resume', error_message='original process confirmed stopped; no active batch SQL found' WHERE pipeline_run_id=? AND status IN ('pending','running')",
            (&active_pipeline,),
        )
        .map_err(|err| format!("failed to close stale pipeline: {err}"))?;
    }
    conn.exec_drop(
        "INSERT INTO meta_pipeline_run (pipeline_run_id, import_batch_id, analysis_run_id, data_type, source_file_name, batch_display_name, status, total_steps, completed_steps, percent, message, created_at, updated_at) SELECT ?, ?, ?, ?, ?, ?, 'pending', ?, 0, 0, 'existing batch resume queued', UTC_TIMESTAMP(), UTC_TIMESTAMP() WHERE NOT EXISTS (SELECT 1 FROM meta_pipeline_run WHERE import_batch_id=? AND status IN ('pending','running'))",
        (
            pipeline_run_id,
            &req.import_batch_id,
            analysis_run_id,
            data_type,
            source_file_name,
            batch_display_name,
            RESUME_PIPELINE_STEPS.len() as i32,
            &req.import_batch_id,
        ),
    )
    .map_err(|err| format!("failed to create resume pipeline: {err}"))?;
    if conn.affected_rows() == 0 {
        return Err(
            "resume rejected because another pipeline became active for this batch".to_string(),
        );
    }
    for (index, step) in RESUME_PIPELINE_STEPS.iter().enumerate() {
        conn.exec_drop(
            "INSERT INTO meta_pipeline_step (pipeline_run_id, step_index, step_name, step_label, status, message) VALUES (?, ?, ?, ?, 'pending', 'waiting')",
            (pipeline_run_id, (index + 1) as i32, step.name, step.label),
        )
        .map_err(|err| format!("failed to create resume step {}: {err}", step.name))?;
    }
    append_log(
        &req.settings,
        pipeline_run_id,
        "info",
        Some("start"),
        &format!(
            "existing batch resume plan created; import_batch_id={}; RAW import will be skipped",
            req.import_batch_id
        ),
        0,
    )
}

fn run_resume_pipeline_job(
    req: ImportPipelineResumeRequest,
    pipeline_run_id: String,
    analysis_run_id: String,
) {
    let total_started = std::time::Instant::now();
    let settings = req.settings.clone();
    let batch = req.import_batch_id.clone();
    let mut degraded = false;
    let mut final_fusion_status = "pending".to_string();
    let _ = update_run(
        &settings,
        &pipeline_run_id,
        "running",
        Some("prepare_resume"),
        Some("existing batch resume running; CSV and RAW import are skipped"),
        None,
        None,
        0,
    );
    for (idx, step) in RESUME_PIPELINE_STEPS.iter().copied().enumerate() {
        let step_index = (idx + 1) as i32;
        let result = match step.name {
            "prepare_resume" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    db::ping(&settings)?;
                    crate::migrations::ensure_experience_policy_schema(&settings)?;
                    crate::batch_tables::ensure_batch_tables(&settings, &batch)?;
                    Ok(Some(format!(
                        "reusing batch={batch}; analysis_run_id={analysis_run_id}; RAW import skipped"
                    )))
                },
            ),
            "dws_ads_aggregate" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    run_with_sql_logging(
                        &settings,
                        &pipeline_run_id,
                        step.name,
                        total_started,
                        || {
                            run_dws_ads_stage(
                                &settings,
                                &pipeline_run_id,
                                &batch,
                                &analysis_run_id,
                                total_started,
                            )
                        },
                    )
                    .map(Some)
                },
            ),
            "final_fusion_optional" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let message = run_final_fusion_stage(&settings, &batch, &analysis_run_id)?;
                    final_fusion_status = "success".to_string();
                    Ok(Some(message))
                },
            ),
            "module_ready" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || run_module_ready_stage(&settings, &batch, &analysis_run_id).map(Some),
            ),
            "finish" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || Ok(Some("existing batch resume finished".to_string())),
            ),
            _ => Ok(Some("unknown resume step skipped".to_string())),
        };
        match result {
            Ok(_) => {}
            Err((PipelineOutcome::Degraded, err)) => {
                degraded = true;
                final_fusion_status = "degraded".to_string();
                let _ = append_log(
                    &settings,
                    &pipeline_run_id,
                    "warning",
                    Some(step.name),
                    &format!("optional step degraded and resume continues: {err}"),
                    now_elapsed_ms(total_started),
                );
            }
            Err((PipelineOutcome::Failed, err)) => {
                let _ = fail_remaining_steps(
                    &settings,
                    &pipeline_run_id,
                    step_index,
                    "前序续跑步骤失败，后续步骤已跳过。",
                );
                let _ = update_run(
                    &settings,
                    &pipeline_run_id,
                    "failed",
                    Some(step.name),
                    Some("existing batch resume failed"),
                    Some(&err),
                    Some(&final_fusion_status),
                    now_elapsed_ms(total_started),
                );
                let _ = append_log(
                    &settings,
                    &pipeline_run_id,
                    "error",
                    Some(step.name),
                    &format!("existing batch resume failed: {err}"),
                    now_elapsed_ms(total_started),
                );
                return;
            }
        }
    }
    let final_status = if degraded { "degraded" } else { "success" };
    let final_message = if degraded {
        "existing batch resume finished with Final Lead degraded; base and structured ADS are available"
    } else {
        "existing batch resume finished successfully"
    };
    let _ = update_run(
        &settings,
        &pipeline_run_id,
        final_status,
        Some("finish"),
        Some(final_message),
        None,
        Some(&final_fusion_status),
        now_elapsed_ms(total_started),
    );
    let _ = append_log(
        &settings,
        &pipeline_run_id,
        if degraded { "warning" } else { "info" },
        Some("finish"),
        final_message,
        now_elapsed_ms(total_started),
    );
}

fn run_rebuild_pipeline_job(
    req: ImportPipelineRebuildRequest,
    pipeline_run_id: String,
    analysis_run_id: String,
) {
    let total_started = std::time::Instant::now();
    let settings = req.settings.clone();
    let batch = req.import_batch_id.clone();
    let mut degraded = false;
    let mut final_fusion_status = "pending".to_string();
    let _ = update_run(
        &settings,
        &pipeline_run_id,
        "running",
        Some("prepare_rebuild"),
        Some("RAW rebuild running; CSV and RAW import are preserved"),
        None,
        None,
        0,
    );
    for (idx, step) in REBUILD_PIPELINE_STEPS.iter().copied().enumerate() {
        let step_index = (idx + 1) as i32;
        let result = match step.name {
            "prepare_rebuild" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    db::ping(&settings)?;
                    crate::migrations::ensure_experience_policy_schema(&settings)?;
                    crate::batch_tables::ensure_batch_tables(&settings, &batch)?;
                    let mut conn = db::conn(&settings)?;
                    let data_type: String = conn
                        .exec_first(
                            "SELECT data_type FROM meta_import_batch WHERE import_batch_id=? LIMIT 1",
                            (&batch,),
                        )
                        .map_err(|err| format!("failed to inspect RAW rebuild data type: {err}"))?
                        .ok_or_else(|| format!("import batch not found: {batch}"))?;
                    let raw_bases: &[&str] = match data_type.to_ascii_lowercase().as_str() {
                        "tcp" => &["raw_tcp_detail_import"],
                        "game" => &["raw_game_detail_import"],
                        "mixed" => &["raw_tcp_detail_import", "raw_game_detail_import"],
                        _ => &[],
                    };
                    let mut ready = 0;
                    for raw_base in raw_bases {
                        let table =
                            crate::batch_tables::resolve_table(&settings, &batch, raw_base)?;
                        if crate::batch_tables::table_has_rows(&mut conn, &table)? {
                            ready += 1;
                        }
                    }
                    if ready == 0 {
                        return Err(
                            "RAW rebuild stopped because no batch RAW table contains rows"
                                .to_string(),
                        );
                    }
                    Ok(Some(format!(
                        "RAW source verified; batch={batch}; analysis_run_id={analysis_run_id}; ready_raw_tables={ready}; CSV import skipped"
                    )))
                },
            ),
            "raw_quality_gate" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    run_with_sql_logging(
                        &settings,
                        &pipeline_run_id,
                        step.name,
                        total_started,
                        || {
                            crate::phase_commands::quality_run_gate(EtlRequest {
                                settings: settings.clone(),
                                import_batch_id: batch.clone(),
                                analysis_run_id: None,
                            })
                        },
                    )?;
                    Ok(Some("RAW Quality Gate regenerated".to_string()))
                },
            ),
            "raw_to_clean" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    run_with_sql_logging(
                        &settings,
                        &pipeline_run_id,
                        step.name,
                        total_started,
                        || {
                            crate::etl_commands::etl_start_clean_job(EtlRequest {
                                settings: settings.clone(),
                                import_batch_id: batch.clone(),
                                analysis_run_id: None,
                            })
                        },
                    )?;
                    Ok(Some("CLEAN/DWD regenerated from existing RAW".to_string()))
                },
            ),
            "dws_ads_aggregate" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    run_with_sql_logging(
                        &settings,
                        &pipeline_run_id,
                        step.name,
                        total_started,
                        || {
                            run_dws_ads_stage(
                                &settings,
                                &pipeline_run_id,
                                &batch,
                                &analysis_run_id,
                                total_started,
                            )
                        },
                    )
                    .map(Some)
                },
            ),
            "final_fusion_optional" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || {
                    let message = run_with_sql_logging(
                        &settings,
                        &pipeline_run_id,
                        step.name,
                        total_started,
                        || run_final_fusion_stage(&settings, &batch, &analysis_run_id),
                    )?;
                    final_fusion_status = "success".to_string();
                    Ok(Some(message))
                },
            ),
            "module_ready" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || run_module_ready_stage(&settings, &batch, &analysis_run_id).map(Some),
            ),
            "finish" => run_observed_step(
                &settings,
                &pipeline_run_id,
                step_index,
                step,
                total_started,
                || Ok(Some("RAW rebuild finished".to_string())),
            ),
            _ => Ok(Some("unknown RAW rebuild step skipped".to_string())),
        };
        match result {
            Ok(_) => {}
            Err((PipelineOutcome::Degraded, err)) => {
                degraded = true;
                final_fusion_status = "degraded".to_string();
                let _ = append_log(
                    &settings,
                    &pipeline_run_id,
                    "warning",
                    Some(step.name),
                    &format!("optional step degraded and RAW rebuild continues: {err}"),
                    now_elapsed_ms(total_started),
                );
            }
            Err((PipelineOutcome::Failed, err)) => {
                let _ = fail_remaining_steps(
                    &settings,
                    &pipeline_run_id,
                    step_index,
                    "前序 RAW 重建步骤失败，后续步骤已跳过。",
                );
                let _ = crate::etl_commands::mark_analysis_run_status(
                    &settings,
                    &analysis_run_id,
                    "failed",
                    &format!("RAW rebuild failed: {err}"),
                );
                let _ = update_run(
                    &settings,
                    &pipeline_run_id,
                    "failed",
                    Some(step.name),
                    Some("RAW rebuild failed"),
                    Some(&err),
                    Some(&final_fusion_status),
                    now_elapsed_ms(total_started),
                );
                let _ = append_log(
                    &settings,
                    &pipeline_run_id,
                    "error",
                    Some(step.name),
                    &format!("RAW rebuild failed: {err}"),
                    now_elapsed_ms(total_started),
                );
                return;
            }
        }
    }
    let final_status = if degraded { "degraded" } else { "success" };
    let final_message = if degraded {
        "RAW rebuild finished with Final Lead degraded; CLEAN/DWS/ADS/V2 are available"
    } else {
        "RAW rebuild finished successfully; CLEAN/DWS/ADS/V2 are available"
    };
    let _ = update_run(
        &settings,
        &pipeline_run_id,
        final_status,
        Some("finish"),
        Some(final_message),
        None,
        Some(&final_fusion_status),
        now_elapsed_ms(total_started),
    );
    let _ = append_log(
        &settings,
        &pipeline_run_id,
        if degraded { "warning" } else { "info" },
        Some("finish"),
        final_message,
        now_elapsed_ms(total_started),
    );
}

#[tauri::command]
pub fn import_pipeline_rebuild_batch_from_raw(
    req: ImportPipelineRebuildRequest,
) -> Result<ImportPipelineStartResult, String> {
    if req.import_batch_id.trim().is_empty() {
        return Err("import_batch_id is required".to_string());
    }
    let analysis_run_id = format!("RUN_REBUILD_{}", Uuid::new_v4().simple());
    let pipeline_run_id = format!("PIPE_{}", Uuid::new_v4().simple());
    insert_rebuild_pipeline_run(&req, &pipeline_run_id, &analysis_run_id)?;
    let task_req = req.clone();
    let task_pipeline_run_id = pipeline_run_id.clone();
    let task_analysis_run_id = analysis_run_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_rebuild_pipeline_job(task_req, task_pipeline_run_id, task_analysis_run_id);
    });
    Ok(ImportPipelineStartResult {
        pipeline_run_id,
        import_batch_id: Some(req.import_batch_id),
        analysis_run_id,
        status: "running".to_string(),
    })
}

#[tauri::command]
pub fn import_pipeline_resume_batch(
    req: ImportPipelineResumeRequest,
) -> Result<ImportPipelineStartResult, String> {
    if req.import_batch_id.trim().is_empty() {
        return Err("import_batch_id is required".to_string());
    }
    let analysis_run_id = if let Some(value) = req
        .analysis_run_id
        .clone()
        .filter(|value| !value.trim().is_empty())
    {
        value
    } else {
        let mut conn = db::conn(&req.settings)?;
        let pipeline_analysis_run = conn
            .exec_first::<String, _, _>(
                "SELECT analysis_run_id FROM meta_pipeline_run WHERE import_batch_id=? AND analysis_run_id IS NOT NULL AND analysis_run_id<>'' ORDER BY updated_at DESC, created_at DESC LIMIT 1",
                (&req.import_batch_id,),
            )
            .map_err(|err| format!("failed to resolve reusable pipeline analysis run: {err}"))?;
        let fallback_analysis_run = if pipeline_analysis_run.is_none() {
            conn.exec_first::<String, _, _>(
                "SELECT analysis_run_id FROM meta_analysis_run WHERE import_batch_id=? ORDER BY started_at DESC LIMIT 1",
                (&req.import_batch_id,),
            )
            .map_err(|err| format!("failed to resolve reusable analysis run: {err}"))?
        } else {
            None
        };
        pipeline_analysis_run
            .or(fallback_analysis_run)
            .unwrap_or_else(|| format!("RUN_{}", Uuid::new_v4().simple()))
    };
    let pipeline_run_id = format!("PIPE_{}", Uuid::new_v4().simple());
    insert_resume_pipeline_run(&req, &pipeline_run_id, &analysis_run_id)?;
    let task_req = req.clone();
    let task_pipeline_run_id = pipeline_run_id.clone();
    let task_analysis_run_id = analysis_run_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_resume_pipeline_job(task_req, task_pipeline_run_id, task_analysis_run_id);
    });
    Ok(ImportPipelineStartResult {
        pipeline_run_id,
        import_batch_id: Some(req.import_batch_id),
        analysis_run_id,
        status: "running".to_string(),
    })
}

#[tauri::command]
pub fn import_pipeline_start(
    req: ImportPipelineStartRequest,
) -> Result<ImportPipelineStartResult, String> {
    if req.file_path.trim().is_empty() {
        return Err("CSV file path is required".to_string());
    }
    if req.batch_display_name.trim().is_empty() {
        return Err("batch_display_name is required".to_string());
    }
    let pipeline_run_id = format!("PIPE_{}", Uuid::new_v4().simple());
    let analysis_run_id = req
        .analysis_run_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("RUN_{}", Uuid::new_v4().simple()));
    insert_pipeline_run(&req.settings, &pipeline_run_id, &req, &analysis_run_id)?;
    let task_req = req.clone();
    let task_pipeline_run_id = pipeline_run_id.clone();
    let task_analysis_run_id = analysis_run_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_pipeline_job(task_req, task_pipeline_run_id, task_analysis_run_id);
    });
    Ok(ImportPipelineStartResult {
        pipeline_run_id,
        import_batch_id: None,
        analysis_run_id,
        status: "running".to_string(),
    })
}

fn reconcile_pipeline_after_mysql_restart(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    let uptime_seconds: u64 = conn
        .exec_first(
            "SELECT CAST(VARIABLE_VALUE AS UNSIGNED) FROM performance_schema.global_status WHERE VARIABLE_NAME='Uptime'",
            (),
        )
        .map_err(|err| format!("failed to read MySQL uptime: {err}"))?
        .unwrap_or(0);
    let row: Option<(String, Option<String>, Option<String>, i64)> = conn
        .exec_first(
            "SELECT status,import_batch_id,analysis_run_id,TIMESTAMPDIFF(SECOND,updated_at,UTC_TIMESTAMP()) FROM meta_pipeline_run WHERE pipeline_run_id=?",
            (pipeline_run_id,),
        )
        .map_err(|err| format!("failed to inspect pipeline restart boundary: {err}"))?;
    let Some((status, import_batch_id, analysis_run_id, silent_seconds)) = row else {
        return Ok(());
    };
    if !matches!(status.as_str(), "pending" | "running")
        || silent_seconds.max(0) as u64 <= uptime_seconds
    {
        return Ok(());
    }
    if let Some(batch) = import_batch_id.as_deref() {
        if !active_batch_statements(settings, batch)?.is_empty() {
            return Ok(());
        }
    }
    conn.exec_drop(
        "UPDATE meta_pipeline_step SET status=CASE WHEN status='running' THEN 'interrupted' ELSE 'skipped' END,finished_at=UTC_TIMESTAMP(),message=CASE WHEN status='running' THEN 'MySQL restarted after the last heartbeat; no active batch SQL remains.' ELSE message END,error_message=CASE WHEN status='running' THEN 'execution interrupted by MySQL restart' ELSE error_message END WHERE pipeline_run_id=? AND status IN ('pending','running')",
        (pipeline_run_id,),
    )
    .map_err(|err| format!("failed to reconcile interrupted pipeline steps: {err}"))?;
    conn.exec_drop(
        "UPDATE meta_pipeline_run SET status='interrupted',finished_at=UTC_TIMESTAMP(),updated_at=UTC_TIMESTAMP(),message='MySQL restarted after the last heartbeat; task can resume from completed hourly checkpoints.',error_message='execution interrupted by MySQL restart; no active batch SQL remains' WHERE pipeline_run_id=? AND status IN ('pending','running')",
        (pipeline_run_id,),
    )
    .map_err(|err| format!("failed to reconcile interrupted pipeline: {err}"))?;
    if let Some(run_id) = analysis_run_id.as_deref() {
        let _ = conn.exec_drop(
            "UPDATE meta_analysis_run SET status='interrupted',finished_at=UTC_TIMESTAMP(),message='analysis interrupted by MySQL restart; completed hourly checkpoints are reusable' WHERE analysis_run_id=? AND status='running'",
            (run_id,),
        );
        if crate::batch_tables::table_exists(&mut conn, "meta_aggregation_partition_checkpoint")
            .unwrap_or(false)
        {
            let _ = conn.exec_drop(
                "UPDATE meta_aggregation_partition_checkpoint SET status='interrupted',finished_at=UTC_TIMESTAMP(),error_summary=COALESCE(error_summary,'MySQL restarted before partition completion'),updated_at=UTC_TIMESTAMP() WHERE analysis_run_id=? AND status='running'",
                (run_id,),
            );
        }
    }
    crate::append_runtime_log(&format!(
        "pipeline_interrupted_after_mysql_restart pipeline_run_id={pipeline_run_id} mysql_uptime_seconds={uptime_seconds} silent_seconds={silent_seconds}"
    ));
    let _ = append_log(
        settings,
        pipeline_run_id,
        "warning",
        Some("interrupted"),
        "MySQL restarted after the last heartbeat; no active batch SQL remains. Resume will reuse successful hourly checkpoints.",
        0,
    );
    Ok(())
}

#[tauri::command]
pub fn import_pipeline_get_status(
    req: ImportPipelineStatusRequest,
) -> Result<ImportPipelineStatus, String> {
    if let Err(err) = reconcile_pipeline_after_mysql_restart(&req.settings, &req.pipeline_run_id) {
        crate::append_runtime_log(&format!(
            "pipeline_restart_reconciliation_skipped pipeline_run_id={} error={err}",
            req.pipeline_run_id
        ));
    }
    let mut conn = db::conn(&req.settings)?;
    let row: Option<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        f64,
        Option<String>,
        Option<String>,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = conn
        .exec_first(
            "SELECT status, current_step, DATE_FORMAT(started_at, '%Y-%m-%dT%H:%i:%sZ'), DATE_FORMAT(finished_at, '%Y-%m-%dT%H:%i:%sZ'), CAST(percent AS DOUBLE), import_batch_id, analysis_run_id, elapsed_ms, error_message, final_fusion_status, message, (SELECT step_name FROM meta_pipeline_step s WHERE s.pipeline_run_id=meta_pipeline_run.pipeline_run_id AND s.status IN ('failed','interrupted') ORDER BY step_index LIMIT 1) FROM meta_pipeline_run WHERE pipeline_run_id=?",
            (&req.pipeline_run_id,),
        )
        .map_err(|err| format!("failed to read pipeline status: {err}"))?;
    let Some((
        status,
        current_step,
        started_at,
        finished_at,
        percent,
        import_batch_id,
        analysis_run_id,
        elapsed_ms,
        error_message,
        final_fusion_status,
        message,
        failed_step,
    )) = row
    else {
        return Err(format!("pipeline not found: {}", req.pipeline_run_id));
    };
    let steps = conn
        .exec_map(
            "SELECT step_index, step_name, step_label, status, DATE_FORMAT(started_at, '%Y-%m-%dT%H:%i:%sZ'), DATE_FORMAT(finished_at, '%Y-%m-%dT%H:%i:%sZ'), elapsed_ms, message, error_message FROM meta_pipeline_step WHERE pipeline_run_id=? ORDER BY step_index",
            (&req.pipeline_run_id,),
            |(step_index, step_name, step_label, status, started_at, finished_at, elapsed_ms, message, error_message)| ImportPipelineStepRow {
                step_index,
                step_name,
                step_label,
                status,
                started_at,
                finished_at,
                elapsed_ms,
                message,
                error_message,
            },
        )
        .map_err(|err| format!("failed to read pipeline steps: {err}"))?;
    Ok(ImportPipelineStatus {
        pipeline_run_id: req.pipeline_run_id,
        status,
        current_step,
        percent,
        started_at,
        finished_at,
        elapsed_ms,
        import_batch_id,
        analysis_run_id,
        failed_step,
        error_message,
        final_fusion_status,
        message,
        steps,
    })
}

#[tauri::command]
pub fn import_pipeline_get_logs(
    req: ImportPipelineLogsRequest,
) -> Result<Vec<ImportPipelineLogRow>, String> {
    let after = req.after_sequence.unwrap_or(0);
    let mut conn = db::conn(&req.settings)?;
    conn.exec_map(
        "SELECT seq, DATE_FORMAT(ts, '%Y-%m-%dT%H:%i:%sZ'), level, step_name, message, elapsed_ms FROM meta_pipeline_log WHERE pipeline_run_id=? AND seq>? ORDER BY seq LIMIT 100",
        (&req.pipeline_run_id, after),
        |(sequence, timestamp, level, step_name, message, elapsed_ms)| ImportPipelineLogRow {
            sequence,
            timestamp,
            level,
            step_name,
            message,
            elapsed_ms,
        },
    )
    .map_err(|err| format!("failed to read pipeline logs: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{
        final_status_for_step_failure, pipeline_plan, raw_import_heartbeat_message,
        raw_import_stall_hint, step_heartbeat_message, PipelineOutcome, AGGREGATE_SUBTASKS,
        REBUILD_PIPELINE_STEPS, RESUME_PIPELINE_STEPS,
    };

    #[test]
    fn pipeline_step_order_is_fixed() {
        let names = pipeline_plan()
            .iter()
            .map(|step| step.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "prepare_environment",
                "probe_csv",
                "import_current_file_atomic",
                "raw_to_clean",
                "raw_quality_gate",
                "dws_ads_aggregate",
                "final_fusion_optional",
                "module_ready",
                "finish",
            ]
        );
    }

    #[test]
    fn resume_plan_skips_raw_and_keeps_complete_analysis_tail() {
        let names = RESUME_PIPELINE_STEPS
            .iter()
            .map(|step| step.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "prepare_resume",
                "dws_ads_aggregate",
                "final_fusion_optional",
                "module_ready",
                "finish",
            ]
        );
        assert!(!names.contains(&"import_current_file_atomic"));
        assert!(!names.contains(&"raw_to_clean"));
    }

    #[test]
    fn rebuild_plan_preserves_raw_and_regenerates_clean_and_aggregates() {
        let names = REBUILD_PIPELINE_STEPS
            .iter()
            .map(|step| step.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "prepare_rebuild",
                "raw_to_clean",
                "raw_quality_gate",
                "dws_ads_aggregate",
                "final_fusion_optional",
                "module_ready",
                "finish",
            ]
        );
        assert!(!names.contains(&"probe_csv"));
        assert!(!names.contains(&"import_current_file_atomic"));
    }

    #[test]
    fn aggregate_stage_materializes_all_structured_dashboards() {
        assert_eq!(
            AGGREGATE_SUBTASKS,
            [
                "base_user_daily",
                "complete_dws",
                "base_dashboards",
                "app_rank",
                "hourly_trend",
                "network_hotspot",
                "user_profile",
                "decision_opportunities",
                "lead_evidence",
            ]
        );
    }

    #[test]
    fn final_fusion_failure_degrades_pipeline() {
        assert_eq!(
            final_status_for_step_failure("final_fusion_optional"),
            PipelineOutcome::Degraded
        );
    }

    #[test]
    fn raw_quality_gate_failure_fails_pipeline() {
        assert_eq!(
            final_status_for_step_failure("raw_quality_gate"),
            PipelineOutcome::Failed
        );
    }

    #[test]
    fn raw_import_heartbeat_distinguishes_transfer_from_mysql_commit() {
        assert!(raw_import_heartbeat_message("load_data", 25, 100).contains("25.0%"));
        let committed = raw_import_heartbeat_message("load_data", 100, 100);
        assert!(committed.contains("100%"));
        assert!(committed.contains("等待 MySQL"));
        assert!(raw_import_stall_hint(0, 100).contains("LOCAL INFILE"));
        assert!(raw_import_stall_hint(100, 100).contains("SHOW PROCESSLIST"));
    }

    #[test]
    fn long_step_heartbeat_explains_quality_scan_without_claiming_sql_liveness() {
        let quality = pipeline_plan()
            .iter()
            .copied()
            .find(|step| step.name == "raw_quality_gate")
            .expect("quality step");
        let message = step_heartbeat_message(quality, 135_000);
        assert!(message.contains("复用 CLEAN/DWD 字段"));
        assert!(message.contains("2 分 15 秒"));
        assert!(message.contains("仅表示应用线程可写日志"));
        assert!(message.contains("不等同于当前 SQL 一定存活"));
    }
}

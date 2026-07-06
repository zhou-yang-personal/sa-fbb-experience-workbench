use mysql::prelude::*;
use uuid::Uuid;

use crate::db;
use crate::models::{
    EtlRequest, ImportCurrentFileRequest, ImportPipelineLogRow, ImportPipelineLogsRequest,
    ImportPipelineStartRequest, ImportPipelineStartResult, ImportPipelineStatus,
    ImportPipelineStatusRequest, ImportPipelineStepRow, MySqlSettings,
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
        name: "raw_quality_gate",
        label: "RAW 质量检查",
    },
    PipelineStepDef {
        name: "raw_to_clean",
        label: "CLEAN/DWD 生成",
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

fn now_elapsed_ms(started: std::time::Instant) -> i64 {
    started.elapsed().as_millis().min(i64::MAX as u128) as i64
}

fn source_file_name(file_path: &str) -> String {
    std::path::Path::new(file_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string())
}

fn ensure_pipeline_schema(settings: &MySqlSettings) -> Result<(), String> {
    sql_runner::execute_script(settings, PIPELINE_SCHEMA).map(|_| ())
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
        "INSERT INTO meta_pipeline_run (pipeline_run_id, analysis_run_id, data_type, source_file_name, batch_display_name, status, total_steps, completed_steps, percent, message, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'pending', ?, 0, 0, 'pipeline queued', NOW(), NOW())",
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
    let seq: Option<i64> = conn
        .exec_first(
            "SELECT COALESCE(MAX(seq), 0) + 1 FROM meta_pipeline_log WHERE pipeline_run_id=?",
            (pipeline_run_id,),
        )
        .map_err(|err| format!("failed to read pipeline log seq: {err}"))?;
    conn.exec_drop(
        "INSERT INTO meta_pipeline_log (pipeline_run_id, seq, ts, level, step_name, message, elapsed_ms) VALUES (?, ?, NOW(), ?, ?, ?, ?)",
        (pipeline_run_id, seq.unwrap_or(1), level, step_name, message, elapsed_ms),
    )
    .map_err(|err| format!("failed to append pipeline log: {err}"))
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
    let completed: Option<i64> = conn
        .exec_first(
            "SELECT CAST(COUNT(*) AS SIGNED) FROM meta_pipeline_step WHERE pipeline_run_id=? AND status IN ('success','skipped','degraded')",
            (pipeline_run_id,),
        )
        .map_err(|err| format!("failed to count completed pipeline steps: {err}"))?;
    let completed = completed.unwrap_or(0);
    let percent = if PIPELINE_STEPS.is_empty() {
        0.0
    } else {
        completed as f64 / PIPELINE_STEPS.len() as f64 * 100.0
    };
    let finished_expr = if matches!(status, "success" | "failed" | "degraded" | "canceled") {
        ", finished_at=COALESCE(finished_at, NOW())"
    } else {
        ""
    };
    conn.exec_drop(
        format!(
            "UPDATE meta_pipeline_run SET status=?, current_step=?, completed_steps=?, percent=?, elapsed_ms=?, message=COALESCE(?, message), error_message=COALESCE(?, error_message), final_fusion_status=COALESCE(?, final_fusion_status), started_at=COALESCE(started_at, NOW()), updated_at=NOW(){finished_expr} WHERE pipeline_run_id=?"
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

fn update_batch_id(
    settings: &MySqlSettings,
    pipeline_run_id: &str,
    import_batch_id: &str,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "UPDATE meta_pipeline_run SET import_batch_id=?, updated_at=NOW() WHERE pipeline_run_id=?",
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
        "UPDATE meta_pipeline_step SET status='running', started_at=COALESCE(started_at, NOW()), finished_at=NULL, elapsed_ms=0, message=?, error_message=NULL WHERE pipeline_run_id=? AND step_index=?",
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
        "UPDATE meta_pipeline_step SET status=?, finished_at=NOW(), elapsed_ms=?, message=?, error_message=? WHERE pipeline_run_id=? AND step_index=?",
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
        "UPDATE meta_pipeline_step SET status='skipped', finished_at=NOW(), elapsed_ms=0, message=? WHERE pipeline_run_id=? AND step_index>? AND status='pending'",
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
    match action() {
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
                    let result = crate::import_commands::import_current_file_atomic(
                        ImportCurrentFileRequest {
                            settings: settings.clone(),
                            data_type: req.data_type.clone(),
                            file_path: req.file_path.clone(),
                            batch_display_name: req.batch_display_name.clone(),
                            mode: req.import_mode.clone(),
                        },
                    )?;
                    import_batch_id = Some(result.batch.import_batch_id.clone());
                    update_batch_id(&settings, &pipeline_run_id, &result.batch.import_batch_id)?;
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
                    let req = EtlRequest {
                        settings: settings.clone(),
                        import_batch_id: batch.clone(),
                        analysis_run_id: Some(analysis_run_id.clone()),
                    };
                    crate::etl_commands::etl_start_aggregate_job(req.clone())?;
                    crate::phase_commands::etl_run_complete_aggregates(req.clone())?;
                    crate::phase_commands::ads_run_complete_dashboards(req)?;
                    Ok(Some(format!(
                        "DWS/ADS ready: analysis_run_id={analysis_run_id}"
                    )))
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
                    crate::phase_commands::leads_run_final_fusion(EtlRequest {
                        settings: settings.clone(),
                        import_batch_id: batch.clone(),
                        analysis_run_id: Some(analysis_run_id.clone()),
                    })?;
                    let final_table = crate::batch_tables::resolve_table(
                        &settings,
                        batch,
                        "ads_final_marketing_lead_user",
                    )?;
                    let final_table = crate::batch_tables::sanitize_identifier(&final_table)?;
                    let mut conn = db::conn(&settings)?;
                    let final_rows: Option<i64> = conn
                        .exec_first(
                            format!(
                                "SELECT CAST(COUNT(*) AS SIGNED) FROM `{final_table}` WHERE analysis_run_id=?"
                            ),
                            (&analysis_run_id,),
                        )
                        .map_err(|err| format!("failed to inspect Final Lead rows: {err}"))?;
                    if final_rows.unwrap_or(0) <= 0 {
                        return Err("Final Lead not generated; likely missing CRM/coverage/reachability, SA Lead remains available".to_string());
                    }
                    final_fusion_status = "success".to_string();
                    Ok(Some("Final Lead fusion ready".to_string()))
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
                    crate::analysis_commands::analysis_prepare_batch_tables(
                        settings.clone(),
                        batch.clone(),
                    )?;
                    crate::analysis_commands::batch_get_table_registry(
                        settings.clone(),
                        batch.clone(),
                    )?;
                    let status = crate::analysis_commands::analysis_get_module_status(
                        settings.clone(),
                        batch.clone(),
                        Some(analysis_run_id.clone()),
                    )?;
                    Ok(Some(format!(
                        "module ready refreshed: enabled={}",
                        status.iter().filter(|item| item.enabled).count()
                    )))
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
    tauri::async_runtime::spawn(async move {
        run_pipeline_job(task_req, task_pipeline_run_id, task_analysis_run_id);
    });
    Ok(ImportPipelineStartResult {
        pipeline_run_id,
        import_batch_id: None,
        analysis_run_id,
        status: "running".to_string(),
    })
}

#[tauri::command]
pub fn import_pipeline_get_status(
    req: ImportPipelineStatusRequest,
) -> Result<ImportPipelineStatus, String> {
    ensure_pipeline_schema(&req.settings)?;
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
            "SELECT status, current_step, DATE_FORMAT(started_at, '%Y-%m-%d %H:%i:%s'), DATE_FORMAT(finished_at, '%Y-%m-%d %H:%i:%s'), CAST(percent AS DOUBLE), import_batch_id, analysis_run_id, elapsed_ms, error_message, final_fusion_status, message, (SELECT step_name FROM meta_pipeline_step s WHERE s.pipeline_run_id=meta_pipeline_run.pipeline_run_id AND s.status='failed' ORDER BY step_index LIMIT 1) FROM meta_pipeline_run WHERE pipeline_run_id=?",
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
            "SELECT step_index, step_name, step_label, status, DATE_FORMAT(started_at, '%Y-%m-%d %H:%i:%s'), DATE_FORMAT(finished_at, '%Y-%m-%d %H:%i:%s'), elapsed_ms, message, error_message FROM meta_pipeline_step WHERE pipeline_run_id=? ORDER BY step_index",
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
    ensure_pipeline_schema(&req.settings)?;
    let after = req.after_sequence.unwrap_or(0);
    let mut conn = db::conn(&req.settings)?;
    conn.exec_map(
        "SELECT seq, DATE_FORMAT(ts, '%Y-%m-%d %H:%i:%s'), level, step_name, message, elapsed_ms FROM meta_pipeline_log WHERE pipeline_run_id=? AND seq>? ORDER BY seq LIMIT 100",
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
    use super::{final_status_for_step_failure, pipeline_plan, PipelineOutcome};

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
                "raw_quality_gate",
                "raw_to_clean",
                "dws_ads_aggregate",
                "final_fusion_optional",
                "module_ready",
                "finish",
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
}

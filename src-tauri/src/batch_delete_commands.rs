use std::collections::HashSet;

use mysql::prelude::*;
use mysql::TxOpts;

use crate::batch_tables;
use crate::db;
use crate::models::{ack, CommandAck, DeleteBatchRequest};

const BATCH_DATA_TABLES: &[&str] = &[
    "raw_tcp_detail_import",
    "raw_game_detail_import",
    "raw_crm_user_import",
    "raw_ftth_coverage_import",
    "raw_reachability_import",
    "dwd_tcp_detail_clean",
    "dwd_game_detail_clean",
    "dws_user_daily_profile",
    "dws_app_category_daily",
    "dws_app_daily",
    "dws_app_user_summary",
    "dws_access_type_hourly_compare",
    "dws_user_experience_bottleneck",
    "ads_migration_lead_user",
    "ads_dashboard_kpi_summary",
    "ads_app_experience_rank",
    "ads_hourly_experience_trend",
    "ads_network_hotspot_rank",
    "ads_user_experience_profile",
    "ads_lead_evidence_detail",
];

const ANALYSIS_DATA_TABLES: &[&str] = &[
    "ads_dashboard_overview",
    "ads_app_category_detail",
    "ads_experience_quality_summary",
    "ads_cable_fiber_compare",
    "ads_migration_lead_user",
    "ads_final_marketing_lead_user",
    "ads_dashboard_kpi_summary",
    "ads_app_experience_rank",
    "ads_hourly_experience_trend",
    "ads_network_hotspot_rank",
    "ads_user_experience_profile",
    "ads_lead_evidence_detail",
];

const BATCH_METADATA_TABLES: &[&str] = &[
    "meta_quality_check_result",
    "meta_import_row_error",
    "meta_mapping_validation_result",
    "meta_dataset_profile",
    "meta_batch_module_status",
    "meta_batch_table_registry",
];

fn is_active_status(status: &str) -> bool {
    status.trim().eq_ignore_ascii_case("running")
}

fn table_exists(conn: &mut mysql::PooledConn, table: &str) -> Result<bool, String> {
    let safe = batch_tables::sanitize_identifier(table)?;
    let count: Option<i64> = conn
        .exec_first(
            "SELECT CAST(COUNT(*) AS SIGNED) FROM information_schema.tables WHERE table_schema=DATABASE() AND table_name=?",
            (&safe,),
        )
        .map_err(|err| format!("failed to inspect table {safe}: {err}"))?;
    Ok(count.unwrap_or(0) > 0)
}

fn present_tables(
    conn: &mut mysql::PooledConn,
    candidates: &[&str],
) -> Result<Vec<String>, String> {
    let mut tables = Vec::new();
    for table in candidates {
        if table_exists(conn, table)? {
            tables.push((*table).to_string());
        }
    }
    Ok(tables)
}

fn expected_physical_tables(import_batch_id: &str) -> Vec<String> {
    batch_tables::TABLE_DEFS
        .iter()
        .map(|def| batch_tables::physical_for_base(import_batch_id, def.base))
        .collect()
}

#[tauri::command]
pub fn import_delete_batch(req: DeleteBatchRequest) -> Result<CommandAck, String> {
    let batch_id = req.import_batch_id.trim();
    if batch_id.is_empty() {
        return Err("import_batch_id is required".to_string());
    }

    delete_batch_internal(&req.settings, batch_id)
}

fn delete_batch_internal(
    settings: &crate::models::MySqlSettings,
    batch_id: &str,
) -> Result<CommandAck, String> {
    let mut conn = db::conn(settings)?;
    batch_tables::ensure_registry_tables(&mut conn)?;

    let status: Option<String> = conn
        .exec_first(
            "SELECT status FROM meta_import_batch WHERE import_batch_id=?",
            (batch_id,),
        )
        .map_err(|err| format!("failed to inspect import batch: {err}"))?;
    let status = status.ok_or_else(|| format!("import batch not found: {batch_id}"))?;
    if is_active_status(&status) {
        return Err(format!(
            "cannot delete batch {batch_id} while its raw import is running"
        ));
    }

    for (table, label) in [
        ("meta_etl_job", "ETL job"),
        ("meta_pipeline_run", "pipeline"),
    ] {
        if !table_exists(&mut conn, table)? {
            continue;
        }
        let active: Option<i64> = conn
            .exec_first(
                format!(
                    "SELECT CAST(COUNT(*) AS SIGNED) FROM `{table}` WHERE import_batch_id=? AND LOWER(status) IN ('pending','running')"
                ),
                (batch_id,),
            )
            .map_err(|err| format!("failed to inspect active {label}s: {err}"))?;
        if active.unwrap_or(0) > 0 {
            return Err(format!(
                "cannot delete batch {batch_id}: {label} is still pending or running"
            ));
        }
    }

    let expected_tables = expected_physical_tables(batch_id);
    let expected_set: HashSet<&str> = expected_tables.iter().map(String::as_str).collect();
    let registered_tables: Vec<String> = conn
        .exec_map(
            "SELECT physical_table_name FROM meta_batch_table_registry WHERE import_batch_id=?",
            (batch_id,),
            |table: String| table,
        )
        .map_err(|err| format!("failed to inspect batch table registry: {err}"))?;
    for table in &registered_tables {
        batch_tables::sanitize_identifier(table)?;
        if !expected_set.contains(table.as_str()) {
            return Err(format!(
                "refusing to delete unexpected registered table for {batch_id}: {table}"
            ));
        }
    }
    for table in &expected_tables {
        let other_owner: Option<i64> = conn
            .exec_first(
                "SELECT CAST(COUNT(*) AS SIGNED) FROM meta_batch_table_registry WHERE physical_table_name=? AND import_batch_id<>?",
                (table, batch_id),
            )
            .map_err(|err| format!("failed to verify table ownership for {table}: {err}"))?;
        if other_owner.unwrap_or(0) > 0 {
            return Err(format!(
                "refusing to delete shared physical table {table}; another batch references it"
            ));
        }
    }

    let has_analysis_runs = table_exists(&mut conn, "meta_analysis_run")?;
    let has_pipeline_runs = table_exists(&mut conn, "meta_pipeline_run")?;
    let has_pipeline_steps = table_exists(&mut conn, "meta_pipeline_step")?;
    let has_pipeline_logs = table_exists(&mut conn, "meta_pipeline_log")?;
    let has_etl_jobs = table_exists(&mut conn, "meta_etl_job")?;
    let has_etl_job_steps = table_exists(&mut conn, "meta_etl_job_step")?;
    let has_export_jobs = table_exists(&mut conn, "meta_export_job")?;
    let mut analysis_run_ids = HashSet::new();
    if has_analysis_runs {
        let ids: Vec<String> = conn
            .exec_map(
                "SELECT analysis_run_id FROM meta_analysis_run WHERE import_batch_id=?",
                (batch_id,),
                |run_id: String| run_id,
            )
            .map_err(|err| format!("failed to resolve analysis runs for batch deletion: {err}"))?;
        analysis_run_ids.extend(ids);
    }
    if has_pipeline_runs {
        let ids: Vec<String> = conn
            .exec_map(
                "SELECT analysis_run_id FROM meta_pipeline_run WHERE import_batch_id=? AND analysis_run_id IS NOT NULL",
                (batch_id,),
                |run_id: String| run_id,
            )
            .map_err(|err| format!("failed to resolve pipeline analysis runs: {err}"))?;
        analysis_run_ids.extend(ids);
    }
    let mut analysis_run_ids: Vec<String> = analysis_run_ids.into_iter().collect();
    analysis_run_ids.sort();

    let batch_data_tables = present_tables(&mut conn, BATCH_DATA_TABLES)?;
    let analysis_data_tables = present_tables(&mut conn, ANALYSIS_DATA_TABLES)?;
    let metadata_tables = present_tables(&mut conn, BATCH_METADATA_TABLES)?;

    conn.exec_drop(
        "UPDATE meta_import_batch SET status='deleting', message='batch deletion started' WHERE import_batch_id=?",
        (batch_id,),
    )
    .map_err(|err| format!("failed to mark batch deleting: {err}"))?;

    if !expected_tables.is_empty() {
        let identifiers = expected_tables
            .iter()
            .map(|table| batch_tables::sanitize_identifier(table).map(|safe| format!("`{safe}`")))
            .collect::<Result<Vec<_>, _>>()?;
        conn.query_drop(format!("DROP TABLE IF EXISTS {}", identifiers.join(", ")))
            .map_err(|err| format!("failed to drop per-batch physical tables: {err}"))?;
    }

    let mut tx = conn
        .start_transaction(TxOpts::default())
        .map_err(|err| format!("failed to start batch deletion transaction: {err}"))?;

    for table in &batch_data_tables {
        tx.exec_drop(
            format!("DELETE FROM `{table}` WHERE import_batch_id=?"),
            (batch_id,),
        )
        .map_err(|err| format!("failed to delete batch rows from {table}: {err}"))?;
    }
    for run_id in &analysis_run_ids {
        for table in &analysis_data_tables {
            tx.exec_drop(
                format!("DELETE FROM `{table}` WHERE analysis_run_id=?"),
                (run_id,),
            )
            .map_err(|err| format!("failed to delete analysis rows from {table}: {err}"))?;
        }
        if has_export_jobs {
            tx.exec_drop(
                "DELETE FROM meta_export_job WHERE analysis_run_id=?",
                (run_id,),
            )
            .map_err(|err| format!("failed to delete export jobs: {err}"))?;
        }
    }

    if has_pipeline_runs {
        if has_pipeline_logs {
            tx.exec_drop(
                "DELETE l FROM meta_pipeline_log l JOIN meta_pipeline_run r ON r.pipeline_run_id=l.pipeline_run_id WHERE r.import_batch_id=?",
                (batch_id,),
            )
            .map_err(|err| format!("failed to delete pipeline logs: {err}"))?;
        }
        if has_pipeline_steps {
            tx.exec_drop(
                "DELETE s FROM meta_pipeline_step s JOIN meta_pipeline_run r ON r.pipeline_run_id=s.pipeline_run_id WHERE r.import_batch_id=?",
                (batch_id,),
            )
            .map_err(|err| format!("failed to delete pipeline steps: {err}"))?;
        }
        tx.exec_drop(
            "DELETE FROM meta_pipeline_run WHERE import_batch_id=?",
            (batch_id,),
        )
        .map_err(|err| format!("failed to delete pipeline runs: {err}"))?;
    }
    if has_etl_jobs {
        if has_etl_job_steps {
            tx.exec_drop(
                "DELETE s FROM meta_etl_job_step s JOIN meta_etl_job j ON j.job_id=s.job_id WHERE j.import_batch_id=?",
                (batch_id,),
            )
            .map_err(|err| format!("failed to delete ETL job steps: {err}"))?;
        }
        tx.exec_drop(
            "DELETE FROM meta_etl_job WHERE import_batch_id=?",
            (batch_id,),
        )
        .map_err(|err| format!("failed to delete ETL jobs: {err}"))?;
    }
    for table in &metadata_tables {
        tx.exec_drop(
            format!("DELETE FROM `{table}` WHERE import_batch_id=?"),
            (batch_id,),
        )
        .map_err(|err| format!("failed to delete metadata from {table}: {err}"))?;
    }
    if has_analysis_runs {
        tx.exec_drop(
            "DELETE FROM meta_analysis_run WHERE import_batch_id=?",
            (batch_id,),
        )
        .map_err(|err| format!("failed to delete analysis runs: {err}"))?;
    }
    tx.exec_drop(
        "DELETE FROM meta_import_batch WHERE import_batch_id=?",
        (batch_id,),
    )
    .map_err(|err| format!("failed to delete import batch: {err}"))?;
    tx.commit()
        .map_err(|err| format!("failed to commit batch deletion: {err}"))?;

    Ok(ack(format!(
        "deleted import batch {batch_id}, {} per-batch tables and {} analysis runs",
        expected_tables.len(),
        analysis_run_ids.len()
    )))
}

#[cfg(test)]
mod tests {
    use super::{expected_physical_tables, is_active_status};
    use crate::batch_tables;

    #[test]
    fn active_statuses_are_protected() {
        assert!(!is_active_status("pending"));
        assert!(is_active_status("RUNNING"));
        assert!(!is_active_status("success"));
        assert!(!is_active_status("failed"));
    }

    #[test]
    fn deletion_targets_only_current_batch_physical_tables() {
        let batch_id = "BATCH_1234567890abcdef";
        let tables = expected_physical_tables(batch_id);
        assert_eq!(tables.len(), batch_tables::TABLE_DEFS.len());
        assert!(tables
            .iter()
            .all(|table| table.ends_with("234567890abcdef")));
    }
}

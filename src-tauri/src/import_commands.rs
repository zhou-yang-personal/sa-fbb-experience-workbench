use mysql::prelude::*;
use uuid::Uuid;

use crate::db;
use crate::models::{
    ack, CommandAck, CreateBatchRequest, ImportBatchResult, ImportCurrentFileRequest,
    ImportCurrentFileResult, MetricCard, MySqlSettings, RawLoadRequest,
};
use crate::probe;
use crate::raw_import;

fn ensure_batch_display_name_column(
    conn: &mut mysql::PooledConn,
    database: &str,
) -> Result<(), String> {
    let exists: Option<u8> = conn.exec_first(
        "SELECT 1 FROM information_schema.columns WHERE table_schema=? AND table_name='meta_import_batch' AND column_name='batch_display_name' LIMIT 1",
        (database,),
    ).map_err(|err| format!("failed to inspect meta_import_batch columns: {err}"))?;
    if exists.is_none() {
        conn.query_drop("ALTER TABLE meta_import_batch ADD COLUMN batch_display_name VARCHAR(255) NULL AFTER import_batch_id")
            .map_err(|err| format!("failed to add meta_import_batch.batch_display_name: {err}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn import_probe_csv(path: String) -> Result<crate::models::CsvProbeResult, String> {
    probe::probe_file(path)
}

#[tauri::command]
pub fn import_create_batch(req: CreateBatchRequest) -> Result<ImportBatchResult, String> {
    create_batch_internal(
        &req.settings,
        &req.data_type,
        &req.file_path,
        req.batch_display_name.as_deref(),
    )
}

pub fn create_batch_internal(
    settings: &MySqlSettings,
    data_type: &str,
    file_path: &str,
    batch_display_name: Option<&str>,
) -> Result<ImportBatchResult, String> {
    let import_batch_id = format!("BATCH_{}", Uuid::new_v4().simple());
    let source_file_name = std::path::Path::new(file_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());
    let batch_display_name = batch_display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| source_file_name.clone());
    let file_size = std::fs::metadata(file_path).map(|m| m.len()).ok();
    let mut conn = db::conn(settings)?;
    ensure_batch_display_name_column(&mut conn, &settings.database)?;
    conn.exec_drop(
        "INSERT INTO meta_import_batch (import_batch_id, batch_display_name, data_type, source_file_name, source_file_path, source_file_size_bytes, status) VALUES (?, ?, ?, ?, ?, ?, 'pending')",
        (&import_batch_id, &batch_display_name, data_type, &source_file_name, file_path, file_size),
    ).map_err(|err| format!("failed to create import batch: {err}"))?;
    Ok(ImportBatchResult {
        import_batch_id,
        batch_display_name: Some(batch_display_name),
        data_type: data_type.to_string(),
        source_file_name,
        status: "pending".to_string(),
    })
}

pub fn import_start_raw_load(req: RawLoadRequest) -> Result<CommandAck, String> {
    raw_import::start_raw_load(req).map(ack)
}

#[tauri::command]
pub fn import_get_batch_status(
    settings: MySqlSettings,
    import_batch_id: String,
) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    ensure_batch_display_name_column(&mut conn, &settings.database)?;
    let row: Option<(String, String, i64, i64, String)> = conn.exec_first(
        "SELECT COALESCE(batch_display_name,''), status, CAST(COALESCE(imported_rows,0) AS SIGNED), CAST(COALESCE(total_rows,0) AS SIGNED), COALESCE(message,'') FROM meta_import_batch WHERE import_batch_id=?",
        (&import_batch_id,),
    ).map_err(|err| format!("failed to query import batch status: {err}"))?;
    let Some((batch_display_name, status, imported_rows, total_rows, message)) = row else {
        return Ok(vec![MetricCard {
            label: "Import batch".to_string(),
            value: "not_found".to_string(),
            hint: import_batch_id,
        }]);
    };
    let progress = if total_rows > 0 {
        imported_rows as f64 / total_rows as f64 * 100.0
    } else {
        0.0
    };
    Ok(vec![
        MetricCard {
            label: "Batch name".to_string(),
            value: batch_display_name,
            hint: "meta_import_batch.batch_display_name".to_string(),
        },
        MetricCard {
            label: "Import status".to_string(),
            value: status,
            hint: message,
        },
        MetricCard {
            label: "Imported rows".to_string(),
            value: imported_rows.to_string(),
            hint: "meta_import_batch.imported_rows".to_string(),
        },
        MetricCard {
            label: "Total rows".to_string(),
            value: total_rows.to_string(),
            hint: "meta_import_batch.total_rows".to_string(),
        },
        MetricCard {
            label: "Progress".to_string(),
            value: format!("{progress:.2}%"),
            hint: "imported_rows / total_rows".to_string(),
        },
    ])
}

#[tauri::command]
pub fn import_current_file_atomic(
    req: ImportCurrentFileRequest,
) -> Result<ImportCurrentFileResult, String> {
    if req.file_path.trim().is_empty() {
        return Err("CSV file path is required".to_string());
    }
    if req.batch_display_name.trim().is_empty() {
        return Err("batch_display_name is required".to_string());
    }
    probe::probe_file(req.file_path.clone())?;
    crate::mapping_catalog::ensure_import_mapping_catalog(&req.settings)?;
    let batch = create_batch_internal(
        &req.settings,
        &req.data_type,
        &req.file_path,
        Some(&req.batch_display_name),
    )?;
    let validation_rows = crate::mapping_validation_commands::validate_mapping_to_db(
        &req.settings,
        &batch.import_batch_id,
        &req.data_type,
        &req.file_path,
    )?;
    if let Some(message) =
        crate::mapping_validation_commands::missing_required_message(&validation_rows)
    {
        mark_batch_failed(&req.settings, &batch.import_batch_id, &message)?;
        return Err(format!(
            "{message}; import_batch_id={}",
            batch.import_batch_id
        ));
    }
    let raw_req = RawLoadRequest {
        settings: req.settings.clone(),
        import_batch_id: batch.import_batch_id.clone(),
        data_type: req.data_type.clone(),
        file_path: req.file_path.clone(),
        mode: req.mode.clone(),
    };
    if let Err(err) = crate::raw_import_v2::start_raw_load(raw_req) {
        mark_batch_failed(&req.settings, &batch.import_batch_id, &err)?;
        return Err(format!("{err}; import_batch_id={}", batch.import_batch_id));
    }
    let _ = crate::dataset_profile_commands::dataset_profile_refresh(
        req.settings.clone(),
        batch.import_batch_id.clone(),
        req.data_type.clone(),
    )?;
    let mapping_summary = crate::mapping_result_commands::import_get_mapping_summary(
        req.settings.clone(),
        batch.import_batch_id.clone(),
        req.data_type.clone(),
    )?;
    let mapping_results = crate::mapping_result_commands::import_get_mapping_results(
        req.settings.clone(),
        batch.import_batch_id.clone(),
        req.data_type.clone(),
    )?;
    let raw_status = import_get_batch_status(req.settings.clone(), batch.import_batch_id.clone())?;
    let profile = crate::dataset_profile_commands::dataset_profile_get(
        req.settings,
        batch.import_batch_id.clone(),
        req.data_type,
    )?;
    Ok(ImportCurrentFileResult {
        batch,
        mapping_summary,
        mapping_results,
        raw_status,
        profile,
        message: "atomic import finished".to_string(),
    })
}

fn mark_batch_failed(
    settings: &MySqlSettings,
    batch_id: &str,
    message: &str,
) -> Result<(), String> {
    let mut conn = db::conn(settings)?;
    ensure_batch_display_name_column(&mut conn, &settings.database)?;
    conn.exec_drop(
        "UPDATE meta_import_batch SET status='failed', finished_at=NOW(), message=? WHERE import_batch_id=?",
        (message, batch_id),
    )
    .map_err(|err| format!("failed to mark batch failed: {err}"))
}

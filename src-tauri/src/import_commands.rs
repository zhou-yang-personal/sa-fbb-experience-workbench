use mysql::prelude::*;
use uuid::Uuid;

use crate::db;
use crate::models::{ack, CommandAck, CreateBatchRequest, ImportBatchResult, MetricCard, MySqlSettings, RawLoadRequest};
use crate::probe;
use crate::raw_import;

#[tauri::command]
pub fn import_probe_csv(path: String) -> Result<crate::models::CsvProbeResult, String> {
    probe::probe_file(path)
}

#[tauri::command]
pub fn import_create_batch(req: CreateBatchRequest) -> Result<ImportBatchResult, String> {
    let import_batch_id = format!("BATCH_{}", Uuid::new_v4().simple());
    let source_file_name = std::path::Path::new(&req.file_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| req.file_path.clone());
    let file_size = std::fs::metadata(&req.file_path).map(|m| m.len()).ok();
    let mut conn = db::conn(&req.settings)?;
    conn.exec_drop(
        "INSERT INTO meta_import_batch (import_batch_id, data_type, source_file_name, source_file_path, source_file_size_bytes, status) VALUES (?, ?, ?, ?, ?, 'pending')",
        (&import_batch_id, &req.data_type, &source_file_name, &req.file_path, file_size),
    ).map_err(|err| format!("failed to create import batch: {err}"))?;
    Ok(ImportBatchResult { import_batch_id, data_type: req.data_type, source_file_name, status: "pending".to_string() })
}

#[tauri::command]
pub fn import_start_raw_load(req: RawLoadRequest) -> Result<CommandAck, String> {
    raw_import::start_raw_load(req).map(ack)
}

#[tauri::command]
pub fn import_get_batch_status(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    let row: Option<(String, i64, i64, String)> = conn.exec_first(
        "SELECT status, CAST(COALESCE(imported_rows,0) AS SIGNED), CAST(COALESCE(total_rows,0) AS SIGNED), COALESCE(message,'') FROM meta_import_batch WHERE import_batch_id=?",
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

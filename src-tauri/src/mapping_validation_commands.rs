use std::collections::HashSet;

use csv::StringRecord;
use mysql::prelude::*;

use crate::db;
use crate::models::{ack, CommandAck, MySqlSettings};

#[tauri::command]
pub fn import_validate_mapping(settings: MySqlSettings, import_batch_id: String, data_type: String, file_path: String) -> Result<CommandAck, String> {
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_path(&file_path).map_err(|err| format!("failed to open CSV for mapping validation: {err}"))?;
    let headers = reader.headers().map_err(|err| format!("failed to read CSV headers: {err}"))?.clone();
    let normalized_headers = normalized_header_set(&headers);
    let mut conn = db::conn(&settings)?;
    conn.exec_drop("DELETE FROM meta_mapping_validation_result WHERE import_batch_id=? AND data_type=?", (&import_batch_id, &data_type)).map_err(|err| format!("failed to clear previous mapping validation: {err}"))?;
    let rows: Vec<(String, String, u8)> = conn.exec(
        "SELECT target_column, source_header, required_flag FROM cfg_import_field_mapping WHERE data_type=? AND active_flag=1 ORDER BY target_column, priority",
        (&data_type,),
    ).map_err(|err| format!("failed to query import mappings: {err}"))?;
    let mut checked = 0_u64;
    for (target_column, source_header, required_flag) in rows {
        let matched = normalized_headers.contains(&normalize(&source_header));
        let status = if matched { "matched" } else if required_flag == 1 { "missing_required" } else { "missing_optional" };
        let matched_header = if matched { Some(source_header.clone()) } else { None };
        conn.exec_drop(
            "INSERT INTO meta_mapping_validation_result (import_batch_id, data_type, target_column, matched_source_header, required_flag, match_status) VALUES (?, ?, ?, ?, ?, ?)",
            (&import_batch_id, &data_type, &target_column, matched_header, required_flag, status),
        ).map_err(|err| format!("failed to write mapping validation: {err}"))?;
        checked += 1;
    }
    Ok(ack(format!("mapping validation finished: checked={checked}")))
}

fn normalized_header_set(headers: &StringRecord) -> HashSet<String> {
    headers.iter().map(normalize).collect()
}

fn normalize(value: &str) -> String {
    value.trim().trim_start_matches('\u{feff}').to_lowercase().replace(' ', "_").replace('-', "_")
}

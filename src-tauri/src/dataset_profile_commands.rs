use mysql::prelude::*;

use crate::db;
use crate::models::{ack, CommandAck, MySqlSettings};

#[tauri::command]
pub fn dataset_profile_refresh(settings: MySqlSettings, import_batch_id: String, data_type: String) -> Result<CommandAck, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_drop("DELETE FROM meta_dataset_profile WHERE import_batch_id=? AND data_type=?", (&import_batch_id, &data_type)).map_err(|err| format!("failed to clear dataset profile: {err}"))?;
    let table = match data_type.as_str() {
        "tcp" => "raw_tcp_detail_import",
        "game" => "raw_game_detail_import",
        "crm" => "raw_crm_user_import",
        "coverage" => "raw_ftth_coverage_import",
        "reachability" => "raw_reachability_import",
        other => return Err(format!("unsupported data type for profile: {other}")),
    };
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE import_batch_id=?");
    let rows: Option<u64> = conn.exec_first(sql, (&import_batch_id,)).map_err(|err| format!("failed to count rows: {err}"))?;
    conn.exec_drop(
        "INSERT INTO meta_dataset_profile (import_batch_id, data_type, profile_key, profile_value, profile_number) VALUES (?, ?, 'row_count', ?, ?)",
        (&import_batch_id, &data_type, rows.unwrap_or(0).to_string(), rows.unwrap_or(0)),
    ).map_err(|err| format!("failed to write dataset profile: {err}"))?;
    Ok(ack(format!("dataset profile refreshed: data_type={data_type}, rows={}", rows.unwrap_or(0))))
}

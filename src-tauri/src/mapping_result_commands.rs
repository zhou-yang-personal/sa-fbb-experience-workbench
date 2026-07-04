use mysql::prelude::*;

use crate::db;
use crate::models::{MetricCard, MySqlSettings};

#[tauri::command]
pub fn import_get_mapping_results(settings: MySqlSettings, import_batch_id: String, data_type: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT target_column, match_status, CONCAT('source=', COALESCE(matched_source_header,''), ', required=', required_flag) FROM meta_mapping_validation_result WHERE import_batch_id=? AND data_type=? ORDER BY required_flag DESC, match_status, target_column",
        (&import_batch_id, &data_type),
        |(label, value, hint): (String, String, String)| MetricCard { label, value, hint },
    ).map_err(|err| format!("failed to query mapping validation results: {err}"))
}

#[tauri::command]
pub fn import_get_mapping_summary(settings: MySqlSettings, import_batch_id: String, data_type: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT match_status, COUNT(*), CONCAT('data_type=', data_type) FROM meta_mapping_validation_result WHERE import_batch_id=? AND data_type=? GROUP BY match_status, data_type ORDER BY match_status",
        (&import_batch_id, &data_type),
        |(label, value, hint): (String, u64, String)| MetricCard { label, value: value.to_string(), hint },
    ).map_err(|err| format!("failed to query mapping validation summary: {err}"))
}

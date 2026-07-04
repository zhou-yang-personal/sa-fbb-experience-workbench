use mysql::prelude::*;

use crate::db;
use crate::models::{MetricCard, MySqlSettings};

#[tauri::command]
pub fn quality_get_gate_results(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT check_item, severity, COALESCE(metric_text, '') FROM meta_quality_check_result WHERE import_batch_id=? ORDER BY passed, severity, check_section, check_item",
        (&import_batch_id,),
        |(label, value, hint): (String, String, String)| MetricCard { label, value, hint },
    ).map_err(|err| format!("failed to query quality gate results: {err}"))
}

#[tauri::command]
pub fn quality_get_failed_results(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT check_item, severity, COALESCE(metric_text, '') FROM meta_quality_check_result WHERE import_batch_id=? AND passed=0 ORDER BY severity, check_section, check_item",
        (&import_batch_id,),
        |(label, value, hint): (String, String, String)| MetricCard { label, value, hint },
    ).map_err(|err| format!("failed to query failed quality gate results: {err}"))
}

use mysql::prelude::*;

use crate::db;
use crate::models::{MetricCard, MySqlSettings};

#[tauri::command]
pub fn import_get_mapping_results(
    settings: MySqlSettings,
    import_batch_id: String,
    data_type: String,
) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    ensure_mapping_validation_columns(&mut conn)?;
    conn.exec_map(
        "SELECT target_column, match_status, CONCAT('source=', COALESCE(matched_source_header,''), ' | required=', required_flag, ' | alias_candidates=', COALESCE(alias_candidates,''), ' | normalized_aliases=', COALESCE(normalized_aliases,''), ' | normalized_csv_headers=', COALESCE(normalized_csv_headers,'')) FROM meta_mapping_validation_result WHERE import_batch_id=? AND data_type=? ORDER BY required_flag DESC, match_status, target_column",
        (&import_batch_id, &data_type),
        |(label, value, hint): (String, String, String)| MetricCard { label, value, hint },
    ).map_err(|err| format!("failed to query mapping validation results: {err}"))
}

#[tauri::command]
pub fn import_get_mapping_summary(
    settings: MySqlSettings,
    import_batch_id: String,
    data_type: String,
) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT match_status, CAST(COUNT(*) AS SIGNED), CONCAT('data_type=', data_type) FROM meta_mapping_validation_result WHERE import_batch_id=? AND data_type=? GROUP BY match_status, data_type ORDER BY match_status",
        (&import_batch_id, &data_type),
        |(label, value, hint): (String, i64, String)| MetricCard { label, value: value.to_string(), hint },
    ).map_err(|err| format!("failed to query mapping validation summary: {err}"))
}

fn ensure_mapping_validation_columns(conn: &mut mysql::PooledConn) -> Result<(), String> {
    ensure_column(conn, "alias_candidates", "TEXT NULL")?;
    ensure_column(conn, "normalized_aliases", "TEXT NULL")?;
    ensure_column(conn, "normalized_csv_headers", "TEXT NULL")?;
    Ok(())
}

fn ensure_column(conn: &mut mysql::PooledConn, column: &str, ddl: &str) -> Result<(), String> {
    let found: Option<u8> = conn.exec_first(
        "SELECT 1 FROM information_schema.columns WHERE table_schema=DATABASE() AND table_name='meta_mapping_validation_result' AND column_name=? LIMIT 1",
        (column,),
    ).map_err(|err| format!("failed to inspect meta_mapping_validation_result.{column}: {err}"))?;
    if found.is_none() {
        conn.query_drop(format!(
            "ALTER TABLE meta_mapping_validation_result ADD COLUMN {column} {ddl}"
        ))
        .map_err(|err| format!("failed to add meta_mapping_validation_result.{column}: {err}"))?;
    }
    Ok(())
}

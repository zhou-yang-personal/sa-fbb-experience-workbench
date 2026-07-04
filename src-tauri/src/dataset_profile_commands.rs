use mysql::prelude::*;

use crate::db;
use crate::models::{ack, CommandAck, MetricCard, MySqlSettings};

#[derive(Clone, Copy)]
struct ProfileSpec {
    table: &'static str,
    account_col: Option<&'static str>,
    mac_col: Option<&'static str>,
    dimension_col: Option<&'static str>,
}

#[tauri::command]
pub fn dataset_profile_refresh(settings: MySqlSettings, import_batch_id: String, data_type: String) -> Result<CommandAck, String> {
    let spec = profile_spec(&data_type)?;
    let mut conn = db::conn(&settings)?;
    conn.exec_drop("DELETE FROM meta_dataset_profile WHERE import_batch_id=? AND data_type=?", (&import_batch_id, &data_type)).map_err(|err| format!("failed to clear dataset profile: {err}"))?;

    let row_count = count_metric(&mut conn, spec.table, &import_batch_id, "COUNT(*)")?;
    write_profile(&mut conn, &import_batch_id, &data_type, "row_count", row_count.to_string(), Some(row_count))?;

    let source_lines = text_metric(&mut conn, spec.table, &import_batch_id, "CONCAT(COALESCE(MIN(source_line_no),0), '-', COALESCE(MAX(source_line_no),0))")?;
    write_profile(&mut conn, &import_batch_id, &data_type, "source_line_range", source_lines, None)?;

    if let Some(col) = spec.account_col {
        let value = count_metric(&mut conn, spec.table, &import_batch_id, &format!("COUNT(DISTINCT NULLIF(TRIM({col}), ''))"))?;
        write_profile(&mut conn, &import_batch_id, &data_type, "distinct_accounts", value.to_string(), Some(value))?;
    }
    if let Some(col) = spec.mac_col {
        let value = count_metric(&mut conn, spec.table, &import_batch_id, &format!("COUNT(DISTINCT NULLIF(TRIM({col}), ''))"))?;
        write_profile(&mut conn, &import_batch_id, &data_type, "distinct_macs", value.to_string(), Some(value))?;
    }
    if let Some(col) = spec.dimension_col {
        let value = count_metric(&mut conn, spec.table, &import_batch_id, &format!("COUNT(DISTINCT NULLIF(TRIM({col}), ''))"))?;
        write_profile(&mut conn, &import_batch_id, &data_type, &format!("distinct_{col}"), value.to_string(), Some(value))?;
    }

    Ok(ack(format!("dataset profile refreshed: data_type={data_type}, rows={row_count}")))
}

#[tauri::command]
pub fn dataset_profile_get(settings: MySqlSettings, import_batch_id: String, data_type: String) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT profile_key, COALESCE(profile_value, ''), COALESCE(CAST(profile_number AS CHAR), '') FROM meta_dataset_profile WHERE import_batch_id=? AND data_type=? ORDER BY profile_key",
        (&import_batch_id, &data_type),
        |(label, value, hint): (String, String, String)| MetricCard { label, value, hint },
    ).map_err(|err| format!("failed to query dataset profile: {err}"))
}

fn profile_spec(data_type: &str) -> Result<ProfileSpec, String> {
    match data_type {
        "tcp" => Ok(ProfileSpec { table: "raw_tcp_detail_import", account_col: Some("user_account"), mac_col: Some("user_mac"), dimension_col: Some("user_type") }),
        "game" => Ok(ProfileSpec { table: "raw_game_detail_import", account_col: Some("user_account"), mac_col: Some("user_mac"), dimension_col: Some("user_type") }),
        "crm" => Ok(ProfileSpec { table: "raw_crm_user_import", account_col: Some("user_account"), mac_col: Some("user_mac"), dimension_col: Some("contract_status") }),
        "coverage" => Ok(ProfileSpec { table: "raw_ftth_coverage_import", account_col: None, mac_col: None, dimension_col: Some("city") }),
        "reachability" => Ok(ProfileSpec { table: "raw_reachability_import", account_col: Some("user_account"), mac_col: None, dimension_col: Some("last_contact_result") }),
        other => Err(format!("unsupported data type for profile: {other}")),
    }
}

fn count_metric(conn: &mut mysql::PooledConn, table: &str, batch_id: &str, expr: &str) -> Result<i64, String> {
    let sql = format!("SELECT CAST(COALESCE({expr},0) AS SIGNED) FROM {table} WHERE import_batch_id=?");
    let value: Option<i64> = conn.exec_first(sql, (batch_id,)).map_err(|err| format!("failed to calculate dataset profile metric: {err}"))?;
    Ok(value.unwrap_or(0))
}

fn text_metric(conn: &mut mysql::PooledConn, table: &str, batch_id: &str, expr: &str) -> Result<String, String> {
    let sql = format!("SELECT {expr} FROM {table} WHERE import_batch_id=?");
    let value: Option<String> = conn.exec_first(sql, (batch_id,)).map_err(|err| format!("failed to calculate dataset profile text metric: {err}"))?;
    Ok(value.unwrap_or_default())
}

fn write_profile(conn: &mut mysql::PooledConn, batch_id: &str, data_type: &str, key: &str, value: String, number: Option<i64>) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO meta_dataset_profile (import_batch_id, data_type, profile_key, profile_value, profile_number) VALUES (?, ?, ?, ?, ?)",
        (batch_id, data_type, key, value, number),
    ).map_err(|err| format!("failed to write dataset profile: {err}"))
}

use mysql::prelude::*;

use crate::db;
use crate::models::{MetricCard, MySqlSettings};
use crate::sql_runner::escape_sql_literal;

#[derive(Clone, Copy)]
pub struct TableDef {
    pub logical: &'static str,
    pub base: &'static str,
    pub layer: &'static str,
    pub data_type: &'static str,
}

pub const TABLE_DEFS: &[TableDef] = &[
    TableDef { logical: "raw_tcp", base: "raw_tcp_detail_import", layer: "raw", data_type: "tcp" },
    TableDef { logical: "raw_game", base: "raw_game_detail_import", layer: "raw", data_type: "game" },
    TableDef { logical: "raw_crm", base: "raw_crm_user_import", layer: "raw", data_type: "crm" },
    TableDef { logical: "raw_coverage", base: "raw_ftth_coverage_import", layer: "raw", data_type: "coverage" },
    TableDef { logical: "raw_reachability", base: "raw_reachability_import", layer: "raw", data_type: "reachability" },
    TableDef { logical: "dwd_tcp", base: "dwd_tcp_detail_clean", layer: "clean", data_type: "tcp" },
    TableDef { logical: "dwd_game", base: "dwd_game_detail_clean", layer: "clean", data_type: "game" },
    TableDef { logical: "dws_user_daily", base: "dws_user_daily_profile", layer: "dws", data_type: "mixed" },
    TableDef { logical: "dws_app_category", base: "dws_app_category_daily", layer: "dws", data_type: "mixed" },
    TableDef { logical: "dws_access_hourly", base: "dws_access_type_hourly_compare", layer: "dws", data_type: "mixed" },
    TableDef { logical: "dws_bottleneck", base: "dws_user_experience_bottleneck", layer: "dws", data_type: "mixed" },
    TableDef { logical: "ads_overview", base: "ads_dashboard_overview", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_app_category", base: "ads_app_category_detail", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_quality", base: "ads_experience_quality_summary", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_cable_fiber", base: "ads_cable_fiber_compare", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_migration_lead", base: "ads_migration_lead_user", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_final_lead", base: "ads_final_marketing_lead_user", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_dashboard_kpi_summary", base: "ads_dashboard_kpi_summary", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_app_experience_rank", base: "ads_app_experience_rank", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_hourly_experience_trend", base: "ads_hourly_experience_trend", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_network_hotspot_rank", base: "ads_network_hotspot_rank", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_user_experience_profile", base: "ads_user_experience_profile", layer: "ads", data_type: "mixed" },
    TableDef { logical: "ads_lead_evidence_detail", base: "ads_lead_evidence_detail", layer: "ads", data_type: "mixed" },
];

pub fn ensure_registry_tables(conn: &mut mysql::PooledConn) -> Result<(), String> {
    conn.query_drop("CREATE TABLE IF NOT EXISTS meta_batch_table_registry (id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY, import_batch_id VARCHAR(64) NOT NULL, layer VARCHAR(32) NOT NULL, data_type VARCHAR(32) NOT NULL, logical_table_name VARCHAR(128) NOT NULL, base_table_name VARCHAR(128) NOT NULL, physical_table_name VARCHAR(160) NOT NULL, row_count BIGINT NOT NULL DEFAULT 0, status VARCHAR(32) NOT NULL DEFAULT 'created', created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP, UNIQUE KEY uk_batch_logical (import_batch_id, logical_table_name), INDEX ix_batch_layer (import_batch_id, layer), INDEX ix_physical_table (physical_table_name)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4")
        .map_err(|err| format!("failed to create meta_batch_table_registry: {err}"))?;
    conn.query_drop("CREATE TABLE IF NOT EXISTS meta_batch_module_status (id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY, import_batch_id VARCHAR(64) NOT NULL, analysis_run_id VARCHAR(64) NULL, module_id VARCHAR(64) NOT NULL, module_name VARCHAR(128) NOT NULL, enabled TINYINT(1) NOT NULL DEFAULT 0, data_type VARCHAR(32) NULL, missing_required_fields TEXT NULL, missing_tables TEXT NULL, row_count BIGINT NOT NULL DEFAULT 0, status_text TEXT NULL, updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP, UNIQUE KEY uk_batch_module (import_batch_id, module_id), INDEX ix_batch_enabled (import_batch_id, enabled)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4")
        .map_err(|err| format!("failed to create meta_batch_module_status: {err}"))?;
    Ok(())
}

pub fn batch_short_id(import_batch_id: &str) -> String {
    let normalized: String = import_batch_id.chars().filter(|ch| ch.is_ascii_alphanumeric()).map(|ch| ch.to_ascii_lowercase()).collect();
    let tail = if normalized.len() > 16 { &normalized[normalized.len() - 16..] } else { normalized.as_str() };
    if tail.is_empty() { "manualbatch".to_string() } else { tail.to_string() }
}

pub fn physical_name(base_table: &str, import_batch_id: &str) -> String {
    format!("{}__{}", base_table, batch_short_id(import_batch_id))
}

pub fn def_by_base(base_table: &str) -> Option<TableDef> {
    TABLE_DEFS.iter().copied().find(|item| item.base == base_table)
}

pub fn physical_for_base(import_batch_id: &str, base_table: &str) -> String {
    physical_name(base_table, import_batch_id)
}

pub fn raw_base_for_data_type(data_type: &str) -> Option<&'static str> {
    match data_type.to_lowercase().as_str() {
        "tcp" => Some("raw_tcp_detail_import"),
        "game" => Some("raw_game_detail_import"),
        "crm" => Some("raw_crm_user_import"),
        "coverage" => Some("raw_ftth_coverage_import"),
        "reachability" => Some("raw_reachability_import"),
        _ => None,
    }
}

pub fn ensure_batch_tables(settings: &MySqlSettings, import_batch_id: &str) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(settings)?;
    ensure_registry_tables(&mut conn)?;
    let mut metrics = Vec::new();
    for def in TABLE_DEFS {
        let physical = physical_name(def.base, import_batch_id);
        ensure_one_table(&mut conn, def, &physical)?;
        let rows = count_table_rows(&mut conn, &physical).unwrap_or(0);
        upsert_registry(&mut conn, def, import_batch_id, &physical, rows, "created")?;
        metrics.push(MetricCard { label: def.logical.to_string(), value: physical, hint: format!("layer={}, base={}, rows={rows}", def.layer, def.base) });
    }
    Ok(metrics)
}

pub fn ensure_raw_table(settings: &MySqlSettings, import_batch_id: &str, data_type: &str) -> Result<String, String> {
    let Some(base) = raw_base_for_data_type(data_type) else { return Err(format!("unsupported raw data type: {data_type}")); };
    let Some(def) = def_by_base(base) else { return Err(format!("raw table definition not found for {base}")); };
    let mut conn = db::conn(settings)?;
    ensure_registry_tables(&mut conn)?;
    let physical = physical_name(base, import_batch_id);
    ensure_one_table(&mut conn, &def, &physical)?;
    upsert_registry(&mut conn, &def, import_batch_id, &physical, 0, "created")?;
    Ok(physical)
}

fn ensure_one_table(conn: &mut mysql::PooledConn, def: &TableDef, physical: &str) -> Result<(), String> {
    let base = sanitize_identifier(def.base)?;
    let table = sanitize_identifier(physical)?;
    conn.query_drop(format!("CREATE TABLE IF NOT EXISTS `{table}` LIKE `{base}`"))
        .map_err(|err| format!("failed to create per-batch table {table} from {base}: {err}"))
}

fn upsert_registry(conn: &mut mysql::PooledConn, def: &TableDef, import_batch_id: &str, physical: &str, rows: u64, status: &str) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO meta_batch_table_registry (import_batch_id, layer, data_type, logical_table_name, base_table_name, physical_table_name, row_count, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE physical_table_name=VALUES(physical_table_name), row_count=VALUES(row_count), status=VALUES(status), updated_at=NOW()",
        (import_batch_id, def.layer, def.data_type, def.logical, def.base, physical, rows, status),
    ).map_err(|err| format!("failed to upsert batch table registry: {err}"))
}

pub fn refresh_registry_counts(settings: &MySqlSettings, import_batch_id: &str) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(settings)?;
    ensure_registry_tables(&mut conn)?;
    let rows: Vec<(String, String, String, String)> = conn.exec("SELECT logical_table_name, layer, base_table_name, physical_table_name FROM meta_batch_table_registry WHERE import_batch_id=? ORDER BY layer, logical_table_name", (import_batch_id,))
        .map_err(|err| format!("failed to read batch table registry: {err}"))?;
    let mut metrics = Vec::new();
    for (logical, layer, base, physical) in rows {
        let count = count_table_rows(&mut conn, &physical).unwrap_or(0);
        let _ = conn.exec_drop("UPDATE meta_batch_table_registry SET row_count=?, updated_at=NOW() WHERE import_batch_id=? AND logical_table_name=?", (count, import_batch_id, &logical));
        metrics.push(MetricCard { label: logical, value: count.to_string(), hint: format!("layer={layer}, base={base}, physical={physical}") });
    }
    Ok(metrics)
}

pub fn analysis_run_batch(settings: &MySqlSettings, analysis_run_id: &str) -> Result<Option<String>, String> {
    let run_id = analysis_run_id.trim();
    if run_id.is_empty() { return Ok(None); }
    let mut conn = db::conn(settings)?;
    let from_pipeline: Option<String> = conn.exec_first(
        "SELECT import_batch_id FROM meta_pipeline_run WHERE analysis_run_id=? AND import_batch_id IS NOT NULL ORDER BY updated_at DESC LIMIT 1",
        (run_id,),
    ).map_err(|err| format!("failed to resolve analysis_run_id from meta_pipeline_run: {err}"))?;
    if from_pipeline.is_some() { return Ok(from_pipeline); }
    ensure_registry_tables(&mut conn)?;
    conn.exec_first(
        "SELECT import_batch_id FROM meta_batch_module_status WHERE analysis_run_id=? ORDER BY updated_at DESC LIMIT 1",
        (run_id,),
    ).map_err(|err| format!("failed to resolve analysis_run_id from module status: {err}"))
}

pub fn resolve_table(settings: &MySqlSettings, import_batch_id: &str, base_table: &str) -> Result<String, String> {
    let mut conn = db::conn(settings)?;
    ensure_registry_tables(&mut conn)?;
    let existing: Option<String> = conn.exec_first(
        "SELECT physical_table_name FROM meta_batch_table_registry WHERE import_batch_id=? AND base_table_name=? LIMIT 1",
        (import_batch_id, base_table),
    ).map_err(|err| format!("failed to resolve physical table for {base_table}: {err}"))?;
    if let Some(table) = existing { return Ok(table); }
    let Some(def) = def_by_base(base_table) else { return Err(format!("unknown batch table base: {base_table}")); };
    let physical = physical_name(base_table, import_batch_id);
    ensure_one_table(&mut conn, &def, &physical)?;
    upsert_registry(&mut conn, &def, import_batch_id, &physical, 0, "created")?;
    Ok(physical)
}

pub fn sanitize_identifier(identifier: &str) -> Result<String, String> {
    if identifier.is_empty() || !identifier.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err(format!("unsafe table identifier: {identifier}"));
    }
    Ok(identifier.to_string())
}

pub fn table_exists(conn: &mut mysql::PooledConn, table: &str) -> Result<bool, String> {
    let safe = sanitize_identifier(table)?;
    let count: Option<i64> = conn.exec_first(
        "SELECT CAST(COUNT(*) AS SIGNED) FROM information_schema.tables WHERE table_schema=DATABASE() AND table_name=?",
        (&safe,),
    ).map_err(|err| format!("failed to inspect table {safe}: {err}"))?;
    Ok(count.unwrap_or(0) > 0)
}

pub fn table_columns(conn: &mut mysql::PooledConn, table: &str) -> Result<Vec<String>, String> {
    let safe = sanitize_identifier(table)?;
    conn.exec_map(
        "SELECT column_name FROM information_schema.columns WHERE table_schema=DATABASE() AND table_name=? ORDER BY ordinal_position",
        (&safe,),
        |name: String| name,
    ).map_err(|err| format!("failed to inspect columns for {safe}: {err}"))
}

pub fn bind_batch_tables(settings: &MySqlSettings, import_batch_id: &str, sql: &str) -> Result<String, String> {
    let mut result = sql.to_string();
    for def in TABLE_DEFS {
        let physical = resolve_table(settings, import_batch_id, def.base)?;
        result = result.replace(&format!(":{}", def.base), &format!("`{physical}`"));
    }
    result = result.replace(":import_batch_id", &format!("'{}'", escape_sql_literal(import_batch_id)));
    Ok(result)
}

fn count_table_rows(conn: &mut mysql::PooledConn, table: &str) -> Result<u64, String> {
    let safe = sanitize_identifier(table)?;
    conn.query_first::<u64, _>(format!("SELECT COUNT(*) FROM `{safe}`"))
        .map_err(|err| format!("failed to count rows for {safe}: {err}"))?
        .ok_or_else(|| format!("count query returned no row for {safe}"))
}

#[cfg(test)]
mod tests {
    use super::{batch_short_id, physical_for_base};

    #[test]
    fn batch_short_id_keeps_tail_and_strips_symbols() {
        assert_eq!(batch_short_id("BATCH_20260705_VIDEO_PEAK"), "260705videopeak");
        assert_eq!(physical_for_base("BATCH_20260705_VIDEO_PEAK", "raw_tcp_detail_import"), "raw_tcp_detail_import__260705videopeak");
    }
}

use std::collections::HashMap;

use csv::Writer;
use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{ack, BatchListItem, BatchTableRegistryRow, CommandAck, MetricCard, ModuleStatusRow, MySqlSettings};

#[derive(Clone, Copy)]
struct ModuleSpec {
    module_id: &'static str,
    module_name: &'static str,
    data_types: &'static [&'static str],
    required_fields: &'static [&'static str],
    required_tables: &'static [&'static str],
}

const MODULE_SPECS: &[ModuleSpec] = &[
    ModuleSpec {
        module_id: "overview",
        module_name: "总览看板",
        data_types: &["tcp", "game", "mixed"],
        required_fields: &["import_batch_id", "analysis_run_id"],
        required_tables: &["dws_user_daily_profile", "ads_dashboard_overview"],
    },
    ModuleSpec {
        module_id: "app_usage",
        module_name: "应用使用分析",
        data_types: &["tcp", "game", "mixed"],
        required_fields: &["app_name", "app_category", "statistics_duration"],
        required_tables: &["dws_app_category_daily", "ads_app_category_detail"],
    },
    ModuleSpec {
        module_id: "video_experience",
        module_name: "视频体验分析",
        data_types: &["tcp", "mixed"],
        required_fields: &["universal_video_applications", "vmos", "effective_download_duration_s"],
        required_tables: &["dwd_tcp_detail_clean", "dws_user_daily_profile", "ads_experience_quality_summary"],
    },
    ModuleSpec {
        module_id: "game_experience",
        module_name: "游戏体验分析",
        data_types: &["game", "mixed"],
        required_fields: &["application_protocol", "mos", "game_duration_s"],
        required_tables: &["dwd_game_detail_clean", "dws_user_daily_profile"],
    },
    ModuleSpec {
        module_id: "network_quality",
        module_name: "网络质量分析",
        data_types: &["tcp", "game", "mixed"],
        required_fields: &["subscriber_side_rtt_ms", "network_side_rtt_ms", "packet_loss"],
        required_tables: &["ads_experience_quality_summary", "dws_access_type_hourly_compare"],
    },
    ModuleSpec {
        module_id: "cable_fiber_compare",
        module_name: "Cable / FTTH 对比",
        data_types: &["tcp", "game", "mixed"],
        required_fields: &["wan_type", "hour_of_day"],
        required_tables: &["dws_access_type_hourly_compare", "ads_cable_fiber_compare"],
    },
    ModuleSpec {
        module_id: "migration_lead",
        module_name: "迁转升套机会",
        data_types: &["tcp", "game", "mixed", "crm", "coverage", "reachability"],
        required_fields: &["lead_type", "final_action"],
        required_tables: &["ads_migration_lead_user", "ads_final_marketing_lead_user"],
    },
    ModuleSpec {
        module_id: "user_profile",
        module_name: "用户画像",
        data_types: &["tcp", "game", "mixed"],
        required_fields: &["user_key"],
        required_tables: &["dws_user_daily_profile"],
    },
];

fn module_spec(module_id: &str) -> Option<&'static ModuleSpec> {
    MODULE_SPECS.iter().find(|spec| spec.module_id == module_id)
}

fn is_supported(spec: &ModuleSpec, data_type: &str) -> bool {
    spec.data_types.iter().any(|item| item.eq_ignore_ascii_case(data_type))
}

fn normalize_list(items: impl IntoIterator<Item = String>) -> String {
    items.into_iter().filter(|item| !item.trim().is_empty()).collect::<Vec<_>>().join(", ")
}

fn fetch_batch_data_type(settings: &MySqlSettings, import_batch_id: &str) -> Result<Option<String>, String> {
    let mut conn = db::conn(settings)?;
    conn.exec_first(
        "SELECT data_type FROM meta_import_batch WHERE import_batch_id=? LIMIT 1",
        (import_batch_id,),
    )
    .map_err(|err| format!("failed to query batch data_type: {err}"))
}

#[tauri::command]
pub fn import_list_batches(settings: MySqlSettings, data_type: Option<String>) -> Result<Vec<BatchListItem>, String> {
    let mut conn = db::conn(&settings)?;
    let sql = if data_type.is_some() {
        "SELECT b.import_batch_id, COALESCE(b.batch_display_name, ''), b.data_type, b.source_file_name, b.status, CAST(COALESCE(b.total_rows, 0) AS SIGNED), CAST(COALESCE(b.imported_rows, 0) AS SIGNED), (SELECT analysis_run_id FROM meta_analysis_run ar WHERE ar.import_batch_id = b.import_batch_id ORDER BY ar.started_at DESC LIMIT 1) FROM meta_import_batch b WHERE b.data_type = ? ORDER BY b.created_at DESC, b.import_batch_id DESC LIMIT 500"
    } else {
        "SELECT b.import_batch_id, COALESCE(b.batch_display_name, ''), b.data_type, b.source_file_name, b.status, CAST(COALESCE(b.total_rows, 0) AS SIGNED), CAST(COALESCE(b.imported_rows, 0) AS SIGNED), (SELECT analysis_run_id FROM meta_analysis_run ar WHERE ar.import_batch_id = b.import_batch_id ORDER BY ar.started_at DESC LIMIT 1) FROM meta_import_batch b ORDER BY b.created_at DESC, b.import_batch_id DESC LIMIT 500"
    };
    let rows = if let Some(data_type) = data_type {
        conn.exec_map(
            sql,
            (&data_type,),
            |(import_batch_id, batch_display_name, data_type, source_file_name, status, total_rows, imported_rows, analysis_run_id): (String, String, String, String, String, i64, i64, Option<String>)| BatchListItem {
                import_batch_id,
                batch_display_name: if batch_display_name.trim().is_empty() { None } else { Some(batch_display_name) },
                data_type,
                source_file_name,
                status,
                total_rows: Some(total_rows),
                imported_rows: Some(imported_rows),
                analysis_run_id,
            },
        )
    } else {
        conn.exec_map(
            sql,
            (),
            |(import_batch_id, batch_display_name, data_type, source_file_name, status, total_rows, imported_rows, analysis_run_id): (String, String, String, String, String, i64, i64, Option<String>)| BatchListItem {
                import_batch_id,
                batch_display_name: if batch_display_name.trim().is_empty() { None } else { Some(batch_display_name) },
                data_type,
                source_file_name,
                status,
                total_rows: Some(total_rows),
                imported_rows: Some(imported_rows),
                analysis_run_id,
            },
        )
    }.map_err(|err| format!("failed to list import batches: {err}"))?;
    Ok(rows)
}

#[tauri::command]
pub fn analysis_prepare_batch_tables(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<MetricCard>, String> {
    let metrics = batch_tables::ensure_batch_tables(&settings, &import_batch_id)?;
    let mut registry_metrics = batch_tables::refresh_registry_counts(&settings, &import_batch_id)?;
    let mut combined = Vec::with_capacity(metrics.len() + registry_metrics.len());
    combined.extend(metrics);
    combined.append(&mut registry_metrics);
    Ok(combined)
}

#[tauri::command]
pub fn batch_get_table_registry(settings: MySqlSettings, import_batch_id: String) -> Result<Vec<BatchTableRegistryRow>, String> {
    let mut conn = db::conn(&settings)?;
    batch_tables::ensure_batch_tables(&settings, &import_batch_id)?;
    let _ = batch_tables::refresh_registry_counts(&settings, &import_batch_id);
    conn.exec_map(
        "SELECT import_batch_id, layer, data_type, logical_table_name, base_table_name, physical_table_name, CAST(row_count AS SIGNED), status FROM meta_batch_table_registry WHERE import_batch_id=? ORDER BY layer, logical_table_name",
        (&import_batch_id,),
        |(import_batch_id, layer, data_type, logical_table_name, base_table_name, physical_table_name, row_count, status): (String, String, String, String, String, String, i64, String)| BatchTableRegistryRow {
            import_batch_id,
            layer,
            data_type,
            logical_table_name,
            base_table_name,
            physical_table_name,
            row_count,
            status,
        },
    )
    .map_err(|err| format!("failed to query batch table registry: {err}"))
}

#[tauri::command]
pub fn analysis_get_module_status(settings: MySqlSettings, import_batch_id: String, analysis_run_id: Option<String>) -> Result<Vec<ModuleStatusRow>, String> {
    let batch_data_type = fetch_batch_data_type(&settings, &import_batch_id)?.unwrap_or_else(|| "mixed".to_string());
    batch_tables::ensure_batch_tables(&settings, &import_batch_id)?;
    let registry_rows = batch_get_table_registry(settings.clone(), import_batch_id.clone())?;
    let registry_map: HashMap<String, i64> = registry_rows.iter().map(|row| (row.base_table_name.clone(), row.row_count)).collect();
    let mut conn = db::conn(&settings)?;
    let mut out = Vec::new();
    for spec in MODULE_SPECS {
        let table_names: Vec<String> = spec.required_tables.iter().map(|base| batch_tables::resolve_table(&settings, &import_batch_id, base).unwrap_or_else(|_| (*base).to_string())).collect();
        let missing_tables: Vec<String> = spec.required_tables.iter().zip(table_names.iter()).filter_map(|(base, physical)| {
            if registry_map.get(*base).copied().unwrap_or(0) == 0 { Some(format!("{base}->{physical}")) } else { None }
        }).collect();
        let row_count = spec.required_tables.iter().map(|base| registry_map.get(*base).copied().unwrap_or(0)).sum::<i64>();
        let data_type_ok = is_supported(spec, &batch_data_type);
        let enabled = data_type_ok && missing_tables.is_empty();
        let missing_required_fields = if enabled { None } else { Some(spec.required_fields.join(", ")) };
        let status_text = if enabled {
            Some(format!("enabled: data_type={batch_data_type}, tables={}, rows={row_count}", normalize_list(table_names.into_iter())))
        } else {
            let mut reasons = Vec::new();
            if !data_type_ok {
                reasons.push(format!("data_type {} not supported", batch_data_type));
            }
            if !missing_tables.is_empty() {
                reasons.push(format!("missing tables: {}", missing_tables.join(", ")));
            }
            Some(reasons.join("; "))
        };
        let row = ModuleStatusRow {
            import_batch_id: import_batch_id.clone(),
            analysis_run_id: analysis_run_id.clone(),
            module_id: spec.module_id.to_string(),
            module_name: spec.module_name.to_string(),
            enabled,
            data_type: Some(batch_data_type.clone()),
            missing_required_fields,
            missing_tables: if missing_tables.is_empty() { None } else { Some(missing_tables.join(", ")) },
            row_count,
            status_text,
        };
        let _ = conn.exec_drop(
            "INSERT INTO meta_batch_module_status (import_batch_id, analysis_run_id, module_id, module_name, enabled, data_type, missing_required_fields, missing_tables, row_count, status_text) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE analysis_run_id=VALUES(analysis_run_id), module_name=VALUES(module_name), enabled=VALUES(enabled), data_type=VALUES(data_type), missing_required_fields=VALUES(missing_required_fields), missing_tables=VALUES(missing_tables), row_count=VALUES(row_count), status_text=VALUES(status_text), updated_at=NOW()",
            (&row.import_batch_id, &row.analysis_run_id, &row.module_id, &row.module_name, row.enabled, &row.data_type, &row.missing_required_fields, &row.missing_tables, row.row_count, &row.status_text),
        );
        out.push(row);
    }
    Ok(out)
}

#[tauri::command]
pub fn analysis_get_module_metrics(settings: MySqlSettings, import_batch_id: String, analysis_run_id: Option<String>) -> Result<Vec<MetricCard>, String> {
    let statuses = analysis_get_module_status(settings.clone(), import_batch_id.clone(), analysis_run_id)?;
    let registry = batch_get_table_registry(settings, import_batch_id.clone())?;
    let enabled = statuses.iter().filter(|row| row.enabled).count();
    let disabled = statuses.len().saturating_sub(enabled);
    let mut metrics = vec![
        MetricCard { label: "analysis modules".to_string(), value: statuses.len().to_string(), hint: format!("enabled={enabled}, disabled={disabled}") },
        MetricCard { label: "batch tables".to_string(), value: registry.len().to_string(), hint: "meta_batch_table_registry".to_string() },
    ];
    metrics.extend(statuses.into_iter().map(|row| MetricCard {
        label: row.module_name,
        value: if row.enabled { "enabled".to_string() } else { "disabled".to_string() },
        hint: row.status_text.unwrap_or_else(|| row.module_id),
    }));
    Ok(metrics)
}

#[tauri::command]
pub fn analysis_export_module_csv(settings: MySqlSettings, import_batch_id: String, analysis_run_id: Option<String>, module_id: String, output_path: String) -> Result<CommandAck, String> {
    let statuses = analysis_get_module_status(settings.clone(), import_batch_id.clone(), analysis_run_id.clone())?;
    let Some(spec) = module_spec(&module_id) else {
        return Err(format!("unknown analysis module: {module_id}"));
    };
    let maybe_status = statuses.iter().find(|row| row.module_id == module_id);
    let mut writer = Writer::from_path(&output_path).map_err(|err| format!("failed to create export file: {err}"))?;
    writer.write_record(["module_id", "module_name", "enabled", "label", "value", "hint"]).map_err(|err| err.to_string())?;
    if let Some(status) = maybe_status {
        let row_count = status.row_count.to_string();
        let enabled_text = if status.enabled { "true".to_string() } else { "false".to_string() };
        let state_text = if status.enabled { "enabled".to_string() } else { "disabled".to_string() };
        writer.write_record([
            status.module_id.as_str(),
            status.module_name.as_str(),
            enabled_text.as_str(),
            "status",
            state_text.as_str(),
            status.status_text.as_deref().unwrap_or(""),
        ]).map_err(|err| err.to_string())?;
        writer.write_record([
            status.module_id.as_str(),
            status.module_name.as_str(),
            enabled_text.as_str(),
            "row_count",
            row_count.as_str(),
            status.missing_tables.as_deref().unwrap_or(""),
        ]).map_err(|err| err.to_string())?;
        if let Some(fields) = &status.missing_required_fields {
            writer.write_record([status.module_id.as_str(), status.module_name.as_str(), "false", "missing_required_fields", fields.as_str(), ""]).map_err(|err| err.to_string())?;
        }
    }
    let registry = batch_get_table_registry(settings, import_batch_id)?;
    for base in spec.required_tables {
        if let Some(row) = registry.iter().find(|item| item.base_table_name == *base) {
            let row_count = row.row_count.to_string();
            writer.write_record([
                spec.module_id,
                spec.module_name,
                "true",
                row.logical_table_name.as_str(),
                row_count.as_str(),
                row.physical_table_name.as_str(),
            ]).map_err(|err| err.to_string())?;
        }
    }
    writer.flush().map_err(|err| err.to_string())?;
    Ok(ack(format!("module {} exported to {}", module_id, output_path)))
}

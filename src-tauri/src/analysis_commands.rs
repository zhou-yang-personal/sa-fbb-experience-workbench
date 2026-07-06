use std::collections::HashMap;

use csv::Writer;
use mysql::prelude::*;
use mysql::Value;

use crate::batch_tables;
use crate::db;
use crate::models::{
    ack, BatchListItem, BatchTableRegistryRow, CommandAck, MetricCard, ModuleStatusRow,
    MySqlSettings,
};
use crate::sql_runner::escape_sql_literal;

#[derive(Clone, Copy)]
struct ModuleSpec {
    module_id: &'static str,
    module_name: &'static str,
    data_types: &'static [&'static str],
    required_tables: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct FieldTableCheck {
    table: &'static str,
    columns: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct RequiredFieldCheck {
    field: &'static str,
    logical: bool,
    checks: &'static [FieldTableCheck],
}

const MODULE_SPECS: &[ModuleSpec] = &[
    ModuleSpec {
        module_id: "overview",
        module_name: "总览看板",
        data_types: &["tcp", "game", "mixed"],
        required_tables: &["dws_user_daily_profile", "ads_dashboard_overview"],
    },
    ModuleSpec {
        module_id: "app_usage",
        module_name: "应用使用分析",
        data_types: &["tcp", "game", "mixed"],
        required_tables: &["dws_app_category_daily", "ads_app_category_detail"],
    },
    ModuleSpec {
        module_id: "video_experience",
        module_name: "视频体验分析",
        data_types: &["tcp", "mixed"],
        required_tables: &[
            "dwd_tcp_detail_clean",
            "dws_user_daily_profile",
            "ads_experience_quality_summary",
        ],
    },
    ModuleSpec {
        module_id: "game_experience",
        module_name: "游戏体验分析",
        data_types: &["game", "mixed"],
        required_tables: &["dwd_game_detail_clean", "dws_user_daily_profile"],
    },
    ModuleSpec {
        module_id: "network_quality",
        module_name: "网络质量分析",
        data_types: &["tcp", "game", "mixed"],
        required_tables: &[
            "ads_experience_quality_summary",
            "dws_access_type_hourly_compare",
        ],
    },
    ModuleSpec {
        module_id: "cable_fiber_compare",
        module_name: "Cable / FTTH 对比",
        data_types: &["tcp", "game", "mixed"],
        required_tables: &["dws_access_type_hourly_compare", "ads_cable_fiber_compare"],
    },
    ModuleSpec {
        module_id: "migration_lead",
        module_name: "迁转升套机会",
        data_types: &["tcp", "game", "mixed", "crm", "coverage", "reachability"],
        required_tables: &["ads_migration_lead_user"],
    },
    ModuleSpec {
        module_id: "user_profile",
        module_name: "用户画像",
        data_types: &["tcp", "game", "mixed"],
        required_tables: &["dws_user_daily_profile"],
    },
];

fn module_spec(module_id: &str) -> Option<&'static ModuleSpec> {
    MODULE_SPECS.iter().find(|spec| spec.module_id == module_id)
}

fn is_supported(spec: &ModuleSpec, data_type: &str) -> bool {
    spec.data_types
        .iter()
        .any(|item| item.eq_ignore_ascii_case(data_type))
}

fn normalize_list(items: impl IntoIterator<Item = String>) -> String {
    items
        .into_iter()
        .filter(|item| !item.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

fn field_checks(module_id: &str) -> &'static [RequiredFieldCheck] {
    const OVERVIEW: &[RequiredFieldCheck] = &[
        RequiredFieldCheck {
            field: "import_batch_id",
            logical: false,
            checks: &[FieldTableCheck {
                table: "dws_user_daily_profile",
                columns: &["import_batch_id"],
            }],
        },
        RequiredFieldCheck {
            field: "analysis_run_id",
            logical: false,
            checks: &[FieldTableCheck {
                table: "ads_dashboard_overview",
                columns: &["analysis_run_id"],
            }],
        },
    ];
    const APP_USAGE: &[RequiredFieldCheck] = &[
        RequiredFieldCheck {
            field: "app_name",
            logical: true,
            checks: &[],
        },
        RequiredFieldCheck {
            field: "app_category",
            logical: false,
            checks: &[
                FieldTableCheck {
                    table: "ads_app_category_detail",
                    columns: &["app_category"],
                },
                FieldTableCheck {
                    table: "dws_app_category_daily",
                    columns: &["app_category"],
                },
            ],
        },
        RequiredFieldCheck {
            field: "statistics_duration",
            logical: true,
            checks: &[],
        },
    ];
    const VIDEO: &[RequiredFieldCheck] = &[
        RequiredFieldCheck {
            field: "universal_video_applications",
            logical: true,
            checks: &[],
        },
        RequiredFieldCheck {
            field: "vmos",
            logical: false,
            checks: &[
                FieldTableCheck {
                    table: "dwd_tcp_detail_clean",
                    columns: &["vmos"],
                },
                FieldTableCheck {
                    table: "dws_user_daily_profile",
                    columns: &["avg_vmos"],
                },
            ],
        },
        RequiredFieldCheck {
            field: "effective_download_duration_s",
            logical: true,
            checks: &[],
        },
    ];
    const GAME: &[RequiredFieldCheck] = &[
        RequiredFieldCheck {
            field: "application_protocol",
            logical: true,
            checks: &[],
        },
        RequiredFieldCheck {
            field: "mos",
            logical: false,
            checks: &[
                FieldTableCheck {
                    table: "dwd_game_detail_clean",
                    columns: &["mos"],
                },
                FieldTableCheck {
                    table: "dws_user_daily_profile",
                    columns: &["avg_mos"],
                },
            ],
        },
        RequiredFieldCheck {
            field: "game_duration_s",
            logical: true,
            checks: &[],
        },
    ];
    const NETWORK: &[RequiredFieldCheck] = &[
        RequiredFieldCheck {
            field: "subscriber_side_rtt_ms",
            logical: false,
            checks: &[
                FieldTableCheck {
                    table: "dwd_tcp_detail_clean",
                    columns: &["subscriber_side_rtt_ms"],
                },
                FieldTableCheck {
                    table: "dws_access_type_hourly_compare",
                    columns: &["avg_subscriber_rtt_ms"],
                },
            ],
        },
        RequiredFieldCheck {
            field: "network_side_rtt_ms",
            logical: false,
            checks: &[
                FieldTableCheck {
                    table: "dwd_tcp_detail_clean",
                    columns: &["network_side_rtt_ms"],
                },
                FieldTableCheck {
                    table: "dws_access_type_hourly_compare",
                    columns: &["avg_network_rtt_ms"],
                },
            ],
        },
        RequiredFieldCheck {
            field: "packet_loss",
            logical: false,
            checks: &[
                FieldTableCheck {
                    table: "dwd_tcp_detail_clean",
                    columns: &["user_down_loss", "network_down_loss"],
                },
                FieldTableCheck {
                    table: "dws_access_type_hourly_compare",
                    columns: &["avg_user_down_loss", "avg_network_down_loss"],
                },
            ],
        },
    ];
    const CABLE: &[RequiredFieldCheck] = &[
        RequiredFieldCheck {
            field: "wan_type",
            logical: true,
            checks: &[],
        },
        RequiredFieldCheck {
            field: "hour_of_day",
            logical: false,
            checks: &[
                FieldTableCheck {
                    table: "dws_access_type_hourly_compare",
                    columns: &["hour_of_day"],
                },
                FieldTableCheck {
                    table: "ads_cable_fiber_compare",
                    columns: &["hour_of_day"],
                },
            ],
        },
    ];
    const MIGRATION: &[RequiredFieldCheck] = &[RequiredFieldCheck {
        field: "lead_type",
        logical: false,
        checks: &[FieldTableCheck {
            table: "ads_migration_lead_user",
            columns: &["lead_type"],
        }],
    }];
    const USER: &[RequiredFieldCheck] = &[RequiredFieldCheck {
        field: "user_key",
        logical: false,
        checks: &[FieldTableCheck {
            table: "dws_user_daily_profile",
            columns: &["user_key"],
        }],
    }];
    match module_id {
        "overview" => OVERVIEW,
        "app_usage" => APP_USAGE,
        "video_experience" => VIDEO,
        "game_experience" => GAME,
        "network_quality" => NETWORK,
        "cable_fiber_compare" => CABLE,
        "migration_lead" => MIGRATION,
        "user_profile" => USER,
        _ => &[],
    }
}

fn fetch_batch_data_type(
    settings: &MySqlSettings,
    import_batch_id: &str,
) -> Result<Option<String>, String> {
    let mut conn = db::conn(settings)?;
    conn.exec_first(
        "SELECT data_type FROM meta_import_batch WHERE import_batch_id=? LIMIT 1",
        (import_batch_id,),
    )
    .map_err(|err| format!("failed to query batch data_type: {err}"))
}

fn final_lead_readiness_text(
    settings: &MySqlSettings,
    conn: &mut mysql::PooledConn,
    import_batch_id: &str,
    analysis_run_id: Option<&String>,
    registry_map: &HashMap<String, (String, i64)>,
) -> Result<String, String> {
    let base = "ads_final_marketing_lead_user";
    let physical = batch_tables::resolve_table(settings, import_batch_id, base)?;
    if !batch_tables::table_exists(conn, &physical)? {
        return Ok(
            "Final Lead not ready / degraded due to missing CRM/coverage/reachability".to_string(),
        );
    }
    let registry_rows = registry_map.get(base).map(|(_, rows)| *rows).unwrap_or(0);
    if registry_rows <= 0 {
        return Ok(
            "Final Lead not generated/degraded due to missing CRM/coverage/reachability"
                .to_string(),
        );
    }
    if let Some(run_id) = analysis_run_id.filter(|value| !value.trim().is_empty()) {
        let table = batch_tables::sanitize_identifier(&physical)?;
        let count: Option<i64> = conn
            .exec_first(
                format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{table}` WHERE analysis_run_id=?"),
                (run_id,),
            )
            .map_err(|err| {
                format!("failed to check Final Lead analysis_run_id for {physical}: {err}")
            })?;
        if count.unwrap_or(0) <= 0 {
            return Ok(format!(
                "Final Lead not generated for current analysis_run_id={run_id}; SA Lead remains available"
            ));
        }
    }
    Ok("Final Lead ready".to_string())
}

#[tauri::command]
pub fn import_list_batches(
    settings: MySqlSettings,
    data_type: Option<String>,
) -> Result<Vec<BatchListItem>, String> {
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
            |(
                import_batch_id,
                batch_display_name,
                data_type,
                source_file_name,
                status,
                total_rows,
                imported_rows,
                analysis_run_id,
            ): (
                String,
                String,
                String,
                String,
                String,
                i64,
                i64,
                Option<String>,
            )| BatchListItem {
                import_batch_id,
                batch_display_name: if batch_display_name.trim().is_empty() {
                    None
                } else {
                    Some(batch_display_name)
                },
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
            |(
                import_batch_id,
                batch_display_name,
                data_type,
                source_file_name,
                status,
                total_rows,
                imported_rows,
                analysis_run_id,
            ): (
                String,
                String,
                String,
                String,
                String,
                i64,
                i64,
                Option<String>,
            )| BatchListItem {
                import_batch_id,
                batch_display_name: if batch_display_name.trim().is_empty() {
                    None
                } else {
                    Some(batch_display_name)
                },
                data_type,
                source_file_name,
                status,
                total_rows: Some(total_rows),
                imported_rows: Some(imported_rows),
                analysis_run_id,
            },
        )
    }
    .map_err(|err| format!("failed to list import batches: {err}"))?;
    Ok(rows)
}

#[tauri::command]
pub fn analysis_prepare_batch_tables(
    settings: MySqlSettings,
    import_batch_id: String,
) -> Result<Vec<MetricCard>, String> {
    let metrics = batch_tables::ensure_batch_tables(&settings, &import_batch_id)?;
    let mut registry_metrics = batch_tables::refresh_registry_counts(&settings, &import_batch_id)?;
    let mut combined = Vec::with_capacity(metrics.len() + registry_metrics.len());
    combined.extend(metrics);
    combined.append(&mut registry_metrics);
    Ok(combined)
}

#[tauri::command]
pub fn batch_get_table_registry(
    settings: MySqlSettings,
    import_batch_id: String,
) -> Result<Vec<BatchTableRegistryRow>, String> {
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
pub fn analysis_get_module_status(
    settings: MySqlSettings,
    import_batch_id: String,
    analysis_run_id: Option<String>,
) -> Result<Vec<ModuleStatusRow>, String> {
    let batch_data_type =
        fetch_batch_data_type(&settings, &import_batch_id)?.unwrap_or_else(|| "mixed".to_string());
    batch_tables::ensure_batch_tables(&settings, &import_batch_id)?;
    let registry_rows = batch_get_table_registry(settings.clone(), import_batch_id.clone())?;
    let registry_map: HashMap<String, (String, i64)> = registry_rows
        .iter()
        .map(|row| {
            (
                row.base_table_name.clone(),
                (row.physical_table_name.clone(), row.row_count),
            )
        })
        .collect();
    let mut conn = db::conn(&settings)?;
    let mut column_cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut out = Vec::new();
    for spec in MODULE_SPECS {
        let table_names: Vec<String> = spec
            .required_tables
            .iter()
            .map(|base| {
                batch_tables::resolve_table(&settings, &import_batch_id, base)
                    .unwrap_or_else(|_| (*base).to_string())
            })
            .collect();
        let mut missing_tables: Vec<String> = Vec::new();
        let mut empty_tables: Vec<String> = Vec::new();
        let mut empty_run_tables: Vec<String> = Vec::new();
        for (base, physical) in spec.required_tables.iter().zip(table_names.iter()) {
            let exists = batch_tables::table_exists(&mut conn, physical)?;
            let rows = registry_map.get(*base).map(|(_, rows)| *rows).unwrap_or(0);
            if !exists {
                missing_tables.push(format!("{base}->{physical}"));
            } else if rows <= 0 {
                empty_tables.push(format!("{base}->{physical}"));
            }
        }
        if let Some(run_id) = analysis_run_id
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            for base in spec
                .required_tables
                .iter()
                .filter(|table| table.starts_with("ads_"))
            {
                let physical = batch_tables::resolve_table(&settings, &import_batch_id, base)?;
                if batch_tables::table_exists(&mut conn, &physical)? {
                    let table = batch_tables::sanitize_identifier(&physical)?;
                    let count: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{table}` WHERE analysis_run_id=?"), (run_id,))
                        .map_err(|err| format!("failed to check analysis_run_id for {physical}: {err}"))?;
                    if count.unwrap_or(0) <= 0 {
                        empty_run_tables.push(format!("{base}->{physical}"));
                    }
                }
            }
        }
        let mut missing_fields: Vec<String> = Vec::new();
        for field in field_checks(spec.module_id) {
            if field.logical {
                continue;
            }
            let mut found = false;
            for check in field.checks {
                let physical =
                    batch_tables::resolve_table(&settings, &import_batch_id, check.table)?;
                if !column_cache.contains_key(&physical) {
                    column_cache.insert(
                        physical.clone(),
                        batch_tables::table_columns(&mut conn, &physical)?,
                    );
                }
                if let Some(columns) = column_cache.get(&physical) {
                    if check.columns.iter().any(|name| {
                        columns
                            .iter()
                            .any(|column| column.eq_ignore_ascii_case(name))
                    }) {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                missing_fields.push(field.field.to_string());
            }
        }
        let row_count = spec
            .required_tables
            .iter()
            .map(|base| registry_map.get(*base).map(|(_, rows)| *rows).unwrap_or(0))
            .sum::<i64>();
        let data_type_ok = is_supported(spec, &batch_data_type);
        let enabled = data_type_ok
            && missing_tables.is_empty()
            && empty_tables.is_empty()
            && empty_run_tables.is_empty()
            && missing_fields.is_empty();
        let final_lead_note = if spec.module_id == "migration_lead" {
            Some(final_lead_readiness_text(
                &settings,
                &mut conn,
                &import_batch_id,
                analysis_run_id.as_ref(),
                &registry_map,
            )?)
        } else {
            None
        };
        let missing_required_fields = if missing_fields.is_empty() {
            None
        } else {
            Some(missing_fields.join(", "))
        };
        let status_text = if enabled {
            if spec.module_id == "migration_lead" {
                Some(format!(
                    "SA Lead available; {}; data_type={batch_data_type}, tables={}, rows={row_count}",
                    final_lead_note.unwrap_or_else(|| "Final Lead not checked".to_string()),
                    normalize_list(table_names.into_iter())
                ))
            } else {
                Some(format!(
                    "enabled: data_type={batch_data_type}, tables={}, rows={row_count}",
                    normalize_list(table_names.into_iter())
                ))
            }
        } else {
            let mut reasons = Vec::new();
            if !data_type_ok {
                reasons.push(format!("当前模块不适用于 {batch_data_type} 批次"));
            }
            if !missing_tables.is_empty() {
                reasons.push(format!("missing table: {}", missing_tables.join(", ")));
            }
            if !empty_tables.is_empty() {
                reasons.push(format!(
                    "当前批次尚未生成分析结果，请回到数据导入完成 CLEAN/DWS/ADS：{}",
                    empty_tables.join(", ")
                ));
            }
            if !empty_run_tables.is_empty() {
                let run_id = analysis_run_id
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("current");
                reasons.push(format!(
                    "当前 analysis_run_id 暂无结果，请确认是否使用正确 run id：{run_id}; {}",
                    empty_run_tables.join(", ")
                ));
            }
            if !missing_fields.is_empty() {
                reasons.push(format!("当前模块缺少字段：{}", missing_fields.join(", ")));
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
            missing_tables: if missing_tables.is_empty() {
                None
            } else {
                Some(missing_tables.join(", "))
            },
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
pub fn analysis_get_module_metrics(
    settings: MySqlSettings,
    import_batch_id: String,
    analysis_run_id: Option<String>,
) -> Result<Vec<MetricCard>, String> {
    let statuses =
        analysis_get_module_status(settings.clone(), import_batch_id.clone(), analysis_run_id)?;
    let registry = batch_get_table_registry(settings, import_batch_id.clone())?;
    let enabled = statuses.iter().filter(|row| row.enabled).count();
    let disabled = statuses.len().saturating_sub(enabled);
    let mut metrics = vec![
        MetricCard {
            label: "analysis modules".to_string(),
            value: statuses.len().to_string(),
            hint: format!("enabled={enabled}, disabled={disabled}"),
        },
        MetricCard {
            label: "batch tables".to_string(),
            value: registry.len().to_string(),
            hint: "meta_batch_table_registry".to_string(),
        },
    ];
    metrics.extend(statuses.into_iter().map(|row| MetricCard {
        label: row.module_name,
        value: if row.enabled {
            "enabled".to_string()
        } else {
            "disabled".to_string()
        },
        hint: row.status_text.unwrap_or_else(|| row.module_id),
    }));
    Ok(metrics)
}

#[tauri::command]
pub fn analysis_export_module_csv(
    settings: MySqlSettings,
    import_batch_id: String,
    analysis_run_id: Option<String>,
    module_id: String,
    output_path: String,
) -> Result<CommandAck, String> {
    let statuses = analysis_get_module_status(
        settings.clone(),
        import_batch_id.clone(),
        analysis_run_id.clone(),
    )?;
    let Some(spec) = module_spec(&module_id) else {
        return Err(format!("unknown analysis module: {module_id}"));
    };
    if let Some(status) = statuses.iter().find(|row| row.module_id == module_id) {
        if !status.enabled {
            return Err(format!(
                "module {} is not ready: {}",
                spec.module_id,
                status.status_text.as_deref().unwrap_or("unknown reason")
            ));
        }
    }
    let (headers, sql) = export_query(
        &settings,
        &import_batch_id,
        analysis_run_id.as_deref(),
        spec,
    )?;
    let rows = write_query_csv(&settings, &output_path, &headers, &sql)?;
    Ok(ack(format!(
        "module {} business export wrote {} rows to {}",
        module_id, rows, output_path
    )))
}

fn export_query(
    settings: &MySqlSettings,
    import_batch_id: &str,
    analysis_run_id: Option<&str>,
    spec: &ModuleSpec,
) -> Result<(Vec<&'static str>, String), String> {
    let run_id = escape_sql_literal(analysis_run_id.unwrap_or("RUN_DEFAULT"));
    let batch_id = escape_sql_literal(import_batch_id);
    let table = |base: &str| -> Result<String, String> {
        batch_tables::resolve_table(settings, import_batch_id, base)
            .and_then(|name| batch_tables::sanitize_identifier(&name))
    };
    match spec.module_id {
        "overview" => {
            let t = table("ads_dashboard_overview")?;
            Ok((vec!["analysis_run_id", "metric_key", "metric_label", "metric_value", "metric_hint"], format!("SELECT analysis_run_id, metric_key, metric_label, metric_value, metric_hint FROM `{t}` WHERE analysis_run_id='{run_id}' ORDER BY metric_key")))
        }
        "app_usage" => {
            let t = table("ads_app_category_detail")?;
            Ok((vec!["analysis_run_id", "app_category", "user_type", "active_users", "total_download_gb", "total_game_hours", "avg_experience_score"], format!("SELECT analysis_run_id, app_category, user_type, active_users, total_download_gb, total_game_hours, avg_experience_score FROM `{t}` WHERE analysis_run_id='{run_id}' ORDER BY active_users DESC, app_category, user_type")))
        }
        "video_experience" => {
            let t = table("dwd_tcp_detail_clean")?;
            Ok((vec!["import_batch_id", "user_key", "user_type", "video_rows", "downloaded_gb", "avg_vmos", "avg_effective_download_mbps", "avg_subscriber_rtt_ms", "avg_network_rtt_ms", "avg_user_down_loss"], format!("SELECT import_batch_id, user_key, COALESCE(user_type,'UNKNOWN') AS user_type, COUNT(*) AS video_rows, ROUND(SUM(COALESCE(downloaded_gb,0)), 6) AS downloaded_gb, ROUND(AVG(vmos), 6) AS avg_vmos, ROUND(AVG(effective_download_mbps), 6) AS avg_effective_download_mbps, ROUND(AVG(subscriber_side_rtt_ms), 6) AS avg_subscriber_rtt_ms, ROUND(AVG(network_side_rtt_ms), 6) AS avg_network_rtt_ms, ROUND(AVG(user_down_loss), 6) AS avg_user_down_loss FROM `{t}` WHERE import_batch_id='{batch_id}' GROUP BY import_batch_id, user_key, COALESCE(user_type,'UNKNOWN') ORDER BY avg_vmos IS NULL, avg_vmos ASC, downloaded_gb DESC")))
        }
        "game_experience" => {
            let t = table("dwd_game_detail_clean")?;
            Ok((vec!["import_batch_id", "user_key", "user_type", "game_rows", "game_hours", "avg_mos", "avg_latency_ms", "avg_loss", "avg_jitter_ms", "avg_wifi_delay_ms"], format!("SELECT import_batch_id, user_key, COALESCE(user_type,'UNKNOWN') AS user_type, COUNT(*) AS game_rows, ROUND(SUM(COALESCE(game_hours,0)), 6) AS game_hours, ROUND(AVG(mos), 6) AS avg_mos, ROUND(AVG(worst_latency_ms), 6) AS avg_latency_ms, ROUND(AVG(worst_loss), 6) AS avg_loss, ROUND(AVG(worst_jitter_ms), 6) AS avg_jitter_ms, ROUND(AVG(wifi_delay_ms), 6) AS avg_wifi_delay_ms FROM `{t}` WHERE import_batch_id='{batch_id}' GROUP BY import_batch_id, user_key, COALESCE(user_type,'UNKNOWN') ORDER BY avg_mos IS NULL, avg_mos ASC, game_hours DESC")))
        }
        "network_quality" => {
            let t = table("ads_experience_quality_summary")?;
            Ok((vec!["analysis_run_id", "quality_dimension", "user_type", "affected_users", "avg_value", "p90_value", "severity"], format!("SELECT analysis_run_id, quality_dimension, user_type, affected_users, avg_value, p90_value, severity FROM `{t}` WHERE analysis_run_id='{run_id}' ORDER BY severity DESC, quality_dimension, user_type")))
        }
        "cable_fiber_compare" => {
            let t = table("ads_cable_fiber_compare")?;
            Ok((vec!["analysis_run_id", "stat_date", "hour_of_day", "metric_key", "cable_value", "ftth_value", "diff_value"], format!("SELECT analysis_run_id, stat_date, hour_of_day, metric_key, cable_value, ftth_value, diff_value FROM `{t}` WHERE analysis_run_id='{run_id}' ORDER BY stat_date, hour_of_day, metric_key")))
        }
        "migration_lead" => {
            let t = table("ads_migration_lead_user")?;
            Ok((vec!["analysis_run_id", "lead_type", "users", "avg_demand_score", "avg_migration_motive_score"], format!("SELECT analysis_run_id, lead_type, COUNT(*) AS users, ROUND(AVG(demand_score),2) AS avg_demand_score, ROUND(AVG(migration_motive_score),2) AS avg_migration_motive_score FROM `{t}` WHERE analysis_run_id='{run_id}' GROUP BY analysis_run_id, lead_type ORDER BY users DESC, lead_type")))
        }
        "user_profile" => {
            let t = table("dws_user_daily_profile")?;
            Ok((vec!["import_batch_id", "user_key", "user_type", "stat_date", "video_rows", "game_rows", "total_download_gb", "total_game_hours", "avg_vmos", "avg_mos", "avg_subscriber_rtt_ms", "avg_network_rtt_ms", "avg_user_down_loss", "avg_network_down_loss", "peak_row_pct"], format!("SELECT import_batch_id, user_key, user_type, stat_date, video_rows, game_rows, total_download_gb, total_game_hours, avg_vmos, avg_mos, avg_subscriber_rtt_ms, avg_network_rtt_ms, avg_user_down_loss, avg_network_down_loss, peak_row_pct FROM `{t}` WHERE import_batch_id='{batch_id}' ORDER BY stat_date DESC, user_key")))
        }
        _ => Err(format!(
            "module export is not implemented for {}",
            spec.module_id
        )),
    }
}

fn write_query_csv(
    settings: &MySqlSettings,
    output_path: &str,
    headers: &[&str],
    sql: &str,
) -> Result<u64, String> {
    let mut conn = db::conn(settings)?;
    let mut writer = Writer::from_path(output_path)
        .map_err(|err| format!("failed to create export file: {err}"))?;
    writer
        .write_record(headers)
        .map_err(|err| err.to_string())?;
    let result = conn
        .query_iter(sql)
        .map_err(|err| format!("failed to query business export: {err}"))?;
    let mut rows = 0_u64;
    for row in result {
        let values = row.map_err(|err| err.to_string())?.unwrap();
        let record: Vec<String> = values.iter().map(value_to_csv).collect();
        writer.write_record(record).map_err(|err| err.to_string())?;
        rows += 1;
    }
    writer.flush().map_err(|err| err.to_string())?;
    Ok(rows)
}

fn value_to_csv(value: &Value) -> String {
    match value {
        Value::NULL => String::new(),
        Value::Bytes(bytes) => String::from_utf8_lossy(bytes).to_string(),
        Value::Int(value) => value.to_string(),
        Value::UInt(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::Double(value) => value.to_string(),
        Value::Date(year, month, day, hour, minute, second, micros) => {
            if *hour == 0 && *minute == 0 && *second == 0 && *micros == 0 {
                format!("{year:04}-{month:02}-{day:02}")
            } else {
                format!(
                    "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{:06}",
                    micros
                )
            }
        }
        Value::Time(is_neg, days, hours, minutes, seconds, micros) => {
            let sign = if *is_neg { "-" } else { "" };
            format!(
                "{sign}{days} {hours:02}:{minutes:02}:{seconds:02}.{:06}",
                micros
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{field_checks, module_spec};

    #[test]
    fn migration_lead_requires_sa_lead_table_only() {
        let spec = module_spec("migration_lead").expect("migration_lead spec");

        assert_eq!(spec.required_tables, &["ads_migration_lead_user"]);
        assert!(!spec
            .required_tables
            .contains(&"ads_final_marketing_lead_user"));
    }

    #[test]
    fn migration_lead_required_fields_do_not_depend_on_final_lead() {
        let checks = field_checks("migration_lead");
        let fields: Vec<&str> = checks.iter().map(|check| check.field).collect();
        let tables: Vec<&str> = checks
            .iter()
            .flat_map(|check| check.checks.iter().map(|table| table.table))
            .collect();

        assert_eq!(fields, vec!["lead_type"]);
        assert!(tables.contains(&"ads_migration_lead_user"));
        assert!(!tables.contains(&"ads_final_marketing_lead_user"));
    }
}

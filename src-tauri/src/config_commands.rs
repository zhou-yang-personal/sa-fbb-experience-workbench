use mysql::prelude::*;

use crate::db;
use crate::models::{ack, CommandAck, MetricCard, MySqlSettings};
#[tauri::command]
pub fn config_seed_defaults(settings: MySqlSettings) -> Result<CommandAck, String> {
    let rows = crate::mapping_catalog::ensure_import_mapping_catalog(&settings)?;
    Ok(ack(format!(
        "default import mappings and join rules seeded: affected_rows={rows}"
    )))
}

#[tauri::command]
pub fn config_check_import_catalog(settings: MySqlSettings) -> Result<Vec<MetricCard>, String> {
    crate::mapping_catalog::check_import_mapping_catalog(&settings)
}

#[tauri::command]
pub fn config_get_import_mappings(
    settings: MySqlSettings,
    data_type: String,
) -> Result<Vec<MetricCard>, String> {
    crate::mapping_catalog::ensure_import_mapping_catalog(&settings)?;
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT target_column, source_header, required_flag, priority FROM cfg_import_field_mapping WHERE data_type=? AND active_flag=1 ORDER BY target_column, priority, source_header",
        (&data_type,),
        |(target_column, source_header, required_flag, priority): (String, String, u8, i32)| {
            let effective_required = if target_column.eq_ignore_ascii_case("user_type") && (data_type.eq_ignore_ascii_case("tcp") || data_type.eq_ignore_ascii_case("game")) { 0 } else { required_flag };
            MetricCard {
            label: target_column,
            value: source_header,
            hint: format!("required={}, priority={}", effective_required, priority),
        }},
    ).map_err(|err| format!("failed to query import mappings: {err}"))
}

#[tauri::command]
pub fn config_get_join_rules(settings: MySqlSettings) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_map(
        "SELECT rule_scope, left_alias, left_column, right_alias, right_column, priority FROM cfg_final_join_rule WHERE active_flag=1 ORDER BY rule_scope, priority",
        (),
        |(scope, left_alias, left_column, right_alias, right_column, priority): (String, String, String, String, String, i32)| MetricCard {
            label: scope,
            value: format!("{left_alias}.{left_column} = {right_alias}.{right_column}"),
            hint: format!("priority={priority}"),
        },
    ).map_err(|err| format!("failed to query final join rules: {err}"))
}

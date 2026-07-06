use csv::StringRecord;
use mysql::prelude::*;

use crate::db;
use crate::header_normalizer::{normalize_header, normalized_header_list, normalized_header_map};
use crate::models::{ack, CommandAck, MySqlSettings};

#[derive(Debug, Clone)]
struct MappingRule {
    target_column: String,
    source_header: String,
    required_flag: u8,
}

#[derive(Debug, Clone)]
pub struct MappingValidationRow {
    pub target_column: String,
    pub matched_source_header: Option<String>,
    pub required_flag: u8,
    pub match_status: String,
    pub alias_candidates: String,
    pub normalized_aliases: String,
    pub normalized_csv_headers: String,
}

#[tauri::command]
pub fn import_validate_mapping(
    settings: MySqlSettings,
    import_batch_id: String,
    data_type: String,
    file_path: String,
) -> Result<CommandAck, String> {
    let checked =
        validate_mapping_to_db(&settings, &import_batch_id, &data_type, &file_path)?.len();
    Ok(ack(format!(
        "mapping validation finished: checked={checked}"
    )))
}

pub fn validate_mapping_to_db(
    settings: &MySqlSettings,
    import_batch_id: &str,
    data_type: &str,
    file_path: &str,
) -> Result<Vec<MappingValidationRow>, String> {
    crate::mapping_catalog::ensure_import_mapping_catalog(settings)?;
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(file_path)
        .map_err(|err| format!("failed to open CSV for mapping validation: {err}"))?;
    let headers = reader
        .headers()
        .map_err(|err| format!("failed to read CSV headers: {err}"))?
        .clone();
    let mut conn = db::conn(settings)?;
    ensure_mapping_validation_columns(&mut conn)?;
    conn.exec_drop(
        "DELETE FROM meta_mapping_validation_result WHERE import_batch_id=? AND data_type=?",
        (import_batch_id, data_type),
    )
    .map_err(|err| format!("failed to clear previous mapping validation: {err}"))?;
    let rows: Vec<MappingRule> = conn.exec_map(
        "SELECT target_column, source_header, required_flag FROM cfg_import_field_mapping WHERE data_type=? AND active_flag=1 ORDER BY target_column, priority",
        (data_type,),
        |(target_column, source_header, required_flag): (String, String, u8)| MappingRule {
            required_flag: effective_required_flag(data_type, &target_column, required_flag),
            target_column,
            source_header,
        },
    ).map_err(|err| format!("failed to query import mappings: {err}"))?;
    let validation_rows = evaluate_mapping_groups(&headers, rows);
    for row in &validation_rows {
        conn.exec_drop(
            "INSERT INTO meta_mapping_validation_result (import_batch_id, data_type, target_column, matched_source_header, required_flag, match_status, alias_candidates, normalized_aliases, normalized_csv_headers) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (import_batch_id, data_type, &row.target_column, &row.matched_source_header, row.required_flag, &row.match_status, &row.alias_candidates, &row.normalized_aliases, &row.normalized_csv_headers),
        ).map_err(|err| format!("failed to write mapping validation: {err}"))?;
    }
    Ok(validation_rows)
}

pub fn missing_required_message(rows: &[MappingValidationRow]) -> Option<String> {
    let missing = rows
        .iter()
        .filter(|row| row.match_status == "missing_required")
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return None;
    }
    let details = missing
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let top_headers = row
                .normalized_csv_headers
                .split('|')
                .take(20)
                .collect::<Vec<_>>()
                .join("|");
            format!(
                "{}. target={}, candidates=[{}], normalized_candidates=[{}], matched=false, top_normalized_csv_headers=[{}]",
                index + 1,
                row.target_column,
                row.alias_candidates,
                row.normalized_aliases,
                top_headers
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    Some(format!(
        "mapping validation failed: missing_required_count={}, {details}. Full normalized headers are available in mapping results.",
        missing.len()
    ))
}

fn effective_required_flag(data_type: &str, target_column: &str, required_flag: u8) -> u8 {
    if target_column.eq_ignore_ascii_case("user_type")
        && (data_type.eq_ignore_ascii_case("tcp") || data_type.eq_ignore_ascii_case("game"))
    {
        0
    } else {
        required_flag
    }
}

fn evaluate_mapping_groups(
    headers: &StringRecord,
    rows: Vec<MappingRule>,
) -> Vec<MappingValidationRow> {
    let normalized_headers = normalized_header_map(headers);
    let normalized_csv_headers = normalized_header_list(headers).join("|");
    let mut out = Vec::new();
    let mut cursor: Option<(String, Vec<MappingRule>)> = None;
    for row in rows {
        match &mut cursor {
            Some((current_target, items)) if current_target == &row.target_column => {
                items.push(row)
            }
            Some((current_target, items)) => {
                let previous_target = std::mem::take(current_target);
                let previous_items = std::mem::take(items);
                out.push(evaluate_group(
                    previous_target,
                    previous_items,
                    &normalized_headers,
                    &normalized_csv_headers,
                ));
                *current_target = row.target_column.clone();
                items.push(row);
            }
            None => cursor = Some((row.target_column.clone(), vec![row])),
        }
    }
    if let Some((target_column, items)) = cursor {
        out.push(evaluate_group(
            target_column,
            items,
            &normalized_headers,
            &normalized_csv_headers,
        ));
    }
    out
}

fn evaluate_group(
    target_column: String,
    items: Vec<MappingRule>,
    normalized_headers: &std::collections::HashMap<String, String>,
    normalized_csv_headers: &str,
) -> MappingValidationRow {
    let required_flag = items
        .iter()
        .map(|item| item.required_flag)
        .max()
        .unwrap_or(0);
    let alias_candidates = items
        .iter()
        .map(|item| item.source_header.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let normalized_aliases = items
        .iter()
        .map(|item| normalize_header(&item.source_header))
        .collect::<Vec<_>>()
        .join("|");
    let matched_source_header = items.iter().find_map(|item| {
        normalized_headers
            .get(&normalize_header(&item.source_header))
            .cloned()
    });
    let match_status = if matched_source_header.is_some() {
        "matched"
    } else if required_flag == 1 {
        "missing_required"
    } else {
        "missing_optional"
    }
    .to_string();
    MappingValidationRow {
        target_column,
        matched_source_header,
        required_flag,
        match_status,
        alias_candidates,
        normalized_aliases,
        normalized_csv_headers: normalized_csv_headers.to_string(),
    }
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

#[cfg(test)]
mod tests {
    use super::{effective_required_flag, evaluate_mapping_groups, MappingRule};
    use csv::StringRecord;

    fn rule(target_column: &str, source_header: &str, required_flag: u8) -> MappingRule {
        MappingRule {
            target_column: target_column.to_string(),
            source_header: source_header.to_string(),
            required_flag,
        }
    }

    #[test]
    fn universal_video_aliases_match_normalized_headers() {
        let headers = StringRecord::from(vec![
            "Subscriber\u{00a0}Account",
            "Downloaded Data Volume (KB)",
            "Effective Download Duration (s)",
        ]);
        let rows = vec![
            rule("user_account", "Subscriber Account", 1),
            rule("downloaded_data_volume_kb", "Downloaded Data Volume KB", 1),
            rule(
                "effective_download_duration_s",
                "Effective Download Duration (s)",
                1,
            ),
            rule("user_type", "user_type", 0),
        ];
        let results = evaluate_mapping_groups(&headers, rows);
        assert_eq!(
            results
                .iter()
                .find(|row| row.target_column == "user_account")
                .unwrap()
                .match_status,
            "matched"
        );
        assert_eq!(
            results
                .iter()
                .find(|row| row.target_column == "downloaded_data_volume_kb")
                .unwrap()
                .match_status,
            "matched"
        );
        assert_eq!(
            results
                .iter()
                .find(|row| row.target_column == "effective_download_duration_s")
                .unwrap()
                .match_status,
            "matched"
        );
        assert_eq!(
            results
                .iter()
                .find(|row| row.target_column == "user_type")
                .unwrap()
                .match_status,
            "missing_optional"
        );
        assert!(results
            .iter()
            .all(|row| row.match_status != "missing_required"));
    }

    #[test]
    fn universal_video_headers_with_id_first_pass_mapping_validation() {
        let headers = StringRecord::from(vec![
            "ID",
            "Subscriber Account",
            "User Mac",
            "Local IP Address",
            "Universal Video Applications",
            "Statistics Duration",
            "Downloaded Data Volume (KB)",
            "Effective Download Duration (s)",
        ]);
        let rows = vec![
            rule("user_account", "Subscriber Account", 1),
            rule("user_mac", "User Mac", 0),
            rule("local_ip_address", "Local IP Address", 0),
            rule(
                "universal_video_applications",
                "Universal Video Applications",
                1,
            ),
            rule("statistics_duration", "Statistics Duration", 1),
            rule(
                "downloaded_data_volume_kb",
                "Downloaded Data Volume (KB)",
                1,
            ),
            rule(
                "effective_download_duration_s",
                "Effective Download Duration (s)",
                1,
            ),
            rule("user_type", "user_type", 0),
        ];
        let results = evaluate_mapping_groups(&headers, rows);
        assert!(results
            .iter()
            .all(|row| row.match_status != "missing_required"));
        assert_eq!(
            results
                .iter()
                .find(|row| row.target_column == "user_type")
                .unwrap()
                .match_status,
            "missing_optional"
        );
    }

    #[test]
    fn user_type_is_optional_for_sa_detail_imports() {
        assert_eq!(effective_required_flag("tcp", "user_type", 1), 0);
        assert_eq!(effective_required_flag("game", "user_type", 1), 0);
        assert_eq!(effective_required_flag("crm", "user_type", 1), 1);
    }
}

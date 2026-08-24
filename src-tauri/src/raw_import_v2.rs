use std::collections::HashMap;
use std::path::Path;

use csv::StringRecord;
use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::header_normalizer::normalize_header;
use crate::models::RawLoadRequest;
use crate::sql_runner::escape_sql_literal;

#[derive(Clone)]
struct RawSpec {
    table: String,
    columns: &'static [&'static str],
}

type HeaderAliases = HashMap<String, Vec<String>>;

const TCP_COLUMNS: &[&str] = &[
    "user_account",
    "user_mac",
    "user_type",
    "universal_video_applications",
    "statistics_duration",
    "local_ip_address",
    "server_ip",
    "device_characteristic",
    "device_description",
    "bras",
    "olt",
    "pon",
    "wan_type",
    "vmos",
    "connection_establishment_success_rate",
    "connection_establishment_delay_ms",
    "upstream_data_transmission_rtt_ms",
    "downstream_data_transmission_rtt_ms",
    "network_side_rtt_ms",
    "subscriber_side_rtt_ms",
    "user_avg_download_rate_kbps",
    "max_single_flow_rate_kbps",
    "network_side_downstream_packet_loss_rate",
    "user_side_downstream_packet_loss_rate",
    "network_side_upstream_packet_loss_rate",
    "user_side_upstream_packet_loss_rate",
    "throughput_avg_bandwidth_kbps",
    "user_avg_effective_download_rate_kbps",
    "download_fluency",
    "downloaded_data_volume_kb",
    "effective_download_duration_s",
    "video_download_duration_s",
    "wifi_delay_ms",
];

const GAME_COLUMNS: &[&str] = &[
    "user_account",
    "user_mac",
    "user_type",
    "application_protocol",
    "statistical_time",
    "local_ip_address",
    "server_ip",
    "device_characteristic",
    "device_description",
    "bras",
    "olt",
    "pon",
    "wan_type",
    "mos",
    "connection_establishment_success_rate",
    "connection_establishment_delay_ms",
    "upstream_data_transmission_rtt_ms",
    "downstream_data_transmission_rtt_ms",
    "network_side_rtt_ms",
    "subscriber_side_rtt_ms",
    "network_side_downstream_packet_loss_rate",
    "user_side_downstream_packet_loss_rate",
    "upstream_rtt_jitter_ms",
    "downstream_rtt_jitter_ms",
    "game_duration_s",
    "single_flow_rate_kbps",
    "wifi_delay_ms",
];

const CRM_COLUMNS: &[&str] = &[
    "crm_user_id",
    "user_account",
    "user_mac",
    "current_plan_name",
    "current_plan_speed_mbps",
    "current_arpu",
    "contract_status",
    "arrears_flag",
    "blacklist_flag",
];

const COVERAGE_COLUMNS: &[&str] = &[
    "area_key",
    "city",
    "neighborhood",
    "hp",
    "hc",
    "ftth_available_flag",
    "build_priority",
];

const REACHABILITY_COLUMNS: &[&str] = &[
    "crm_user_id",
    "user_account",
    "phone_available_flag",
    "sms_available_flag",
    "app_push_available_flag",
    "last_contact_result",
];

pub fn start_raw_load(req: RawLoadRequest) -> Result<String, String> {
    preflight_required_mapping(&req)?;
    let mode = req
        .mode
        .clone()
        .unwrap_or_else(|| {
            if req.settings.local_infile.unwrap_or(true) {
                "load_data".to_string()
            } else {
                "streaming_insert".to_string()
            }
        })
        .to_lowercase();
    if mode == "streaming_insert" || mode == "insert" || mode == "fallback" {
        streaming_insert(req)
    } else {
        mapped_load_data(req)
    }
}

fn preflight_required_mapping(req: &RawLoadRequest) -> Result<(), String> {
    let rows = crate::mapping_validation_commands::validate_mapping_to_db(
        &req.settings,
        &req.import_batch_id,
        &req.data_type,
        &req.file_path,
    )?;
    if let Some(message) = crate::mapping_validation_commands::missing_required_message(&rows) {
        let mut conn = db::conn(&req.settings)?;
        mark_failed(&mut conn, &req.import_batch_id, &message);
        return Err(message);
    }
    Ok(())
}

fn raw_spec(req: &RawLoadRequest) -> Result<RawSpec, String> {
    let table =
        batch_tables::ensure_raw_table(&req.settings, &req.import_batch_id, &req.data_type)?;
    match req.data_type.to_lowercase().as_str() {
        "tcp" => Ok(RawSpec {
            table,
            columns: TCP_COLUMNS,
        }),
        "game" => Ok(RawSpec {
            table,
            columns: GAME_COLUMNS,
        }),
        "crm" => Ok(RawSpec {
            table,
            columns: CRM_COLUMNS,
        }),
        "coverage" => Ok(RawSpec {
            table,
            columns: COVERAGE_COLUMNS,
        }),
        "reachability" => Ok(RawSpec {
            table,
            columns: REACHABILITY_COLUMNS,
        }),
        other => Err(format!("unsupported raw data type: {other}")),
    }
}

fn mapped_load_data(req: RawLoadRequest) -> Result<String, String> {
    let spec = raw_spec(&req)?;
    let mut conn = db::conn(&req.settings)?;
    let aliases = load_header_aliases(&mut conn, &req.data_type);
    drop(conn);
    let delimiter = crate::probe::detect_delimiter(&req.file_path)?;
    let headers = read_headers(&req.file_path, delimiter)?;
    load_data(req, spec, &headers, &aliases, delimiter)
}

fn load_data(
    req: RawLoadRequest,
    spec: RawSpec,
    headers: &StringRecord,
    aliases: &HeaderAliases,
    delimiter: u8,
) -> Result<String, String> {
    let mut conn = db::conn(&req.settings)?;
    let file_name = source_file_name(&req.file_path);
    let path = escape_sql_literal(&req.file_path.replace('\\', "/"));
    let batch_id = escape_sql_literal(&req.import_batch_id);
    let source_name = escape_sql_literal(&file_name);
    let header_index = header_index(headers);
    let input_variables = (0..headers.len())
        .map(|index| format!("@csv_{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let assignments = spec
        .columns
        .iter()
        .filter_map(|column| {
            source_index_for_column(column, &header_index, aliases)
                .map(|index| format!("`{column}`=NULLIF(TRIM(@csv_{index}), '')"))
        })
        .collect::<Vec<_>>();
    if assignments.is_empty() {
        return Err("no CSV columns can be mapped to the selected RAW schema".to_string());
    }
    let local_infile_handler = db::local_infile_handler_for_path(&req.file_path)?;
    conn.exec_drop("UPDATE meta_import_batch SET status='running', started_at=NOW(), total_rows=NULL, imported_rows=0, message='raw mapped load_data started' WHERE import_batch_id=?", (&req.import_batch_id,)).map_err(|err| format!("failed to mark batch running: {err}"))?;
    conn.query_drop(format!("TRUNCATE TABLE `{}`", spec.table))
        .map_err(|err| format!("failed to reset RAW batch table before LOAD DATA: {err}"))?;
    let field_terminator = load_data_field_terminator(delimiter)?;
    let sql = format!(
        "LOAD DATA LOCAL INFILE '{path}' INTO TABLE `{}` CHARACTER SET utf8mb4 FIELDS TERMINATED BY '{field_terminator}' ENCLOSED BY '\"' LINES TERMINATED BY '\n' IGNORE 1 LINES ({input_variables}) SET import_batch_id='{batch_id}', source_file_name='{source_name}', source_line_no=NULL, {}",
        spec.table,
        assignments.join(", ")
    );
    conn.set_local_infile_handler(Some(local_infile_handler));
    let load_result = conn.query_drop(sql);
    conn.set_local_infile_handler(None);
    match load_result {
        Ok(_) => {
            let reported_rows = conn.affected_rows();
            let warnings = load_data_warnings(&mut conn);
            let rows =
                verify_imported_rows(&mut conn, &spec.table, &req.import_batch_id, reported_rows)?;
            if rows == 0 {
                let warning_text = if warnings.is_empty() {
                    "MySQL returned no LOAD DATA warnings".to_string()
                } else {
                    format!("MySQL warnings: {}", warnings.join(" | "))
                };
                let msg = format!(
                    "RAW import produced zero rows: table={}, delimiter={}, file={}; {}. Check encoding/line endings and retry with Streaming INSERT only for diagnosis",
                    spec.table,
                    delimiter_label(delimiter),
                    req.file_path,
                    warning_text
                );
                mark_registry_failed(&mut conn, &req.import_batch_id, &spec.table, 0);
                mark_failed(&mut conn, &req.import_batch_id, &msg);
                return Err(msg);
            }
            finalize_batch(
                &mut conn,
                &req.import_batch_id,
                rows,
                rows,
                &spec.table,
                "raw mapped load_data finished",
            )?;
            Ok(format!(
                "raw mapped load_data finished: table={}, rows={rows}, mysql_reported_rows={reported_rows}, delimiter={}, local_infile_stream=enabled",
                spec.table,
                delimiter_label(delimiter)
            ))
        }
        Err(err) => {
            let msg = format!("raw mapped load_data failed: {err}");
            mark_failed(&mut conn, &req.import_batch_id, &msg);
            Err(msg)
        }
    }
}

fn streaming_insert(req: RawLoadRequest) -> Result<String, String> {
    let spec = raw_spec(&req)?;
    let delimiter = crate::probe::detect_delimiter(&req.file_path)?;
    let mut conn = db::conn(&req.settings)?;
    let aliases = load_header_aliases(&mut conn, &req.data_type);
    let file_name = source_file_name(&req.file_path);
    conn.exec_drop("UPDATE meta_import_batch SET status='running', started_at=NOW(), total_rows=NULL, imported_rows=0, message='mapped streaming insert started' WHERE import_batch_id=?", (&req.import_batch_id,)).map_err(|err| format!("failed to mark batch running: {err}"))?;
    conn.query_drop(format!("TRUNCATE TABLE `{}`", spec.table))
        .map_err(|err| format!("failed to reset RAW batch table before streaming insert: {err}"))?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_path(&req.file_path)
        .map_err(|err| format!("failed to open CSV for mapped streaming insert: {err}"))?;
    let headers = reader
        .headers()
        .map_err(|err| format!("failed to read CSV headers: {err}"))?
        .clone();
    let header_index = header_index(&headers);
    let mut rows = Vec::with_capacity(500);
    let mut source_line_no = 1_u64;
    let mut total_rows = 0_u64;

    for row in reader.records() {
        source_line_no += 1;
        let row = row.map_err(|err| format!("failed to read CSV row {source_line_no}: {err}"))?;
        rows.push(row_to_values(
            &req.import_batch_id,
            &file_name,
            source_line_no,
            spec.columns,
            &header_index,
            &aliases,
            &row,
        ));
        if rows.len() >= 500 {
            flush_rows(&mut conn, &spec, &rows)?;
            total_rows += rows.len() as u64;
            update_progress(
                &mut conn,
                &req.import_batch_id,
                total_rows,
                "mapped streaming insert running",
            )?;
            rows.clear();
        }
    }
    if !rows.is_empty() {
        flush_rows(&mut conn, &spec, &rows)?;
        total_rows += rows.len() as u64;
        update_progress(
            &mut conn,
            &req.import_batch_id,
            total_rows,
            "mapped streaming insert running",
        )?;
    }
    if total_rows == 0 {
        let msg = format!(
            "RAW streaming import produced zero rows: table={}, delimiter={}, file={}; the file may contain only a header or use an unsupported encoding/line ending",
            spec.table,
            delimiter_label(delimiter),
            req.file_path
        );
        mark_registry_failed(&mut conn, &req.import_batch_id, &spec.table, 0);
        mark_failed(&mut conn, &req.import_batch_id, &msg);
        return Err(msg);
    }
    let verified_rows =
        verify_imported_rows(&mut conn, &spec.table, &req.import_batch_id, total_rows)?;
    if verified_rows == 0 {
        let msg = format!(
            "RAW streaming import inserted rows but none are visible for batch {} in table {}",
            req.import_batch_id, spec.table
        );
        mark_registry_failed(&mut conn, &req.import_batch_id, &spec.table, 0);
        mark_failed(&mut conn, &req.import_batch_id, &msg);
        return Err(msg);
    }
    finalize_batch(
        &mut conn,
        &req.import_batch_id,
        verified_rows,
        verified_rows,
        &spec.table,
        "mapped streaming insert finished",
    )?;
    Ok(format!(
        "mapped streaming insert finished: table={}, rows={verified_rows}, delimiter={}",
        spec.table,
        delimiter_label(delimiter)
    ))
}

fn load_header_aliases(conn: &mut mysql::PooledConn, data_type: &str) -> HeaderAliases {
    let rows: Result<Vec<(String, String)>, _> = conn.exec(
        "SELECT target_column, source_header FROM cfg_import_field_mapping WHERE data_type=? AND active_flag=1 ORDER BY target_column, priority, source_header",
        (data_type,),
    );
    let mut aliases: HeaderAliases = HashMap::new();
    if let Ok(rows) = rows {
        for (target_column, source_header) in rows {
            aliases
                .entry(normalize_header(&target_column))
                .or_default()
                .push(normalize_header(&source_header));
        }
    }
    aliases
}

fn read_headers(file_path: &str, delimiter: u8) -> Result<StringRecord, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_path(file_path)
        .map_err(|err| format!("failed to open CSV for LOAD DATA mapping: {err}"))?;
    reader
        .headers()
        .map_err(|err| format!("failed to read CSV headers for LOAD DATA mapping: {err}"))
        .cloned()
}

fn header_index(headers: &StringRecord) -> HashMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(index, header)| (normalize_header(header), index))
        .collect()
}

fn row_to_values(
    import_batch_id: &str,
    source_file_name: &str,
    source_line_no: u64,
    columns: &[&str],
    header_index: &HashMap<String, usize>,
    aliases: &HeaderAliases,
    row: &StringRecord,
) -> Vec<String> {
    let mut values = vec![
        sql_literal(import_batch_id),
        sql_literal(source_file_name),
        source_line_no.to_string(),
    ];
    for col in columns {
        values.push(sql_literal(
            value_for_column(col, header_index, aliases, row).unwrap_or(""),
        ));
    }
    values
}

fn value_for_column<'a>(
    column: &str,
    header_index: &HashMap<String, usize>,
    aliases: &HeaderAliases,
    row: &'a StringRecord,
) -> Option<&'a str> {
    let normalized = normalize_header(column);
    if let Some(value) = header_index.get(&normalized).and_then(|idx| row.get(*idx)) {
        return Some(value);
    }
    if let Some(alias_list) = aliases.get(&normalized) {
        for alias in alias_list {
            if let Some(value) = header_index.get(alias).and_then(|idx| row.get(*idx)) {
                return Some(value);
            }
        }
    }
    None
}

fn source_index_for_column(
    column: &str,
    header_index: &HashMap<String, usize>,
    aliases: &HeaderAliases,
) -> Option<usize> {
    let normalized = normalize_header(column);
    header_index.get(&normalized).copied().or_else(|| {
        aliases.get(&normalized).and_then(|items| {
            items
                .iter()
                .find_map(|alias| header_index.get(alias).copied())
        })
    })
}

fn source_file_name(file_path: &str) -> String {
    Path::new(file_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string())
}

fn sql_literal(value: &str) -> String {
    if value.trim().is_empty() {
        "NULL".to_string()
    } else {
        format!("'{}'", escape_sql_literal(value))
    }
}

fn flush_rows(
    conn: &mut mysql::PooledConn,
    spec: &RawSpec,
    rows: &[Vec<String>],
) -> Result<(), String> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut insert_columns = vec!["import_batch_id", "source_file_name", "source_line_no"];
    insert_columns.extend_from_slice(spec.columns);
    let values = rows
        .iter()
        .map(|row| format!("({})", row.join(", ")))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT INTO `{}` ({}) VALUES {values}",
        spec.table,
        insert_columns.join(", ")
    );
    conn.query_drop(sql)
        .map_err(|err| format!("failed to insert mapped streaming RAW rows: {err}"))
}

fn update_progress(
    conn: &mut mysql::PooledConn,
    batch_id: &str,
    rows: u64,
    message: &str,
) -> Result<(), String> {
    conn.exec_drop(
        "UPDATE meta_import_batch SET imported_rows=?, message=? WHERE import_batch_id=?",
        (rows, message, batch_id),
    )
    .map_err(|err| format!("failed to update import progress: {err}"))
}

fn finalize_batch(
    conn: &mut mysql::PooledConn,
    batch_id: &str,
    total_rows: u64,
    imported_rows: u64,
    physical_table: &str,
    message: &str,
) -> Result<(), String> {
    conn.exec_drop("UPDATE meta_import_batch SET status='success', total_rows=?, imported_rows=?, finished_at=NOW(), message=? WHERE import_batch_id=?", (total_rows, imported_rows, message, batch_id)).map_err(|err| format!("failed to finalize import batch: {err}"))?;
    conn.exec_drop(
        "UPDATE meta_batch_table_registry SET row_count=?, status='loaded', updated_at=NOW() WHERE import_batch_id=? AND physical_table_name=?",
        (imported_rows, batch_id, physical_table),
    )
    .map_err(|err| format!("failed to finalize RAW table registry: {err}"))
}

fn mark_failed(conn: &mut mysql::PooledConn, batch_id: &str, message: &str) {
    let _ = conn.exec_drop("UPDATE meta_import_batch SET status='failed', finished_at=NOW(), message=? WHERE import_batch_id=?", (message, batch_id));
}

fn mark_registry_failed(
    conn: &mut mysql::PooledConn,
    batch_id: &str,
    physical_table: &str,
    row_count: u64,
) {
    let _ = conn.exec_drop(
        "UPDATE meta_batch_table_registry SET row_count=?, status='failed', updated_at=NOW() WHERE import_batch_id=? AND physical_table_name=?",
        (row_count, batch_id, physical_table),
    );
}

fn verify_imported_rows(
    conn: &mut mysql::PooledConn,
    physical_table: &str,
    batch_id: &str,
    reported_rows: u64,
) -> Result<u64, String> {
    let table = batch_tables::sanitize_identifier(physical_table)?;
    if reported_rows > 0 {
        let visible: Option<u8> = conn
            .exec_first(
                format!("SELECT 1 FROM `{table}` WHERE import_batch_id=? LIMIT 1"),
                (batch_id,),
            )
            .map_err(|err| format!("failed to verify RAW rows in {table}: {err}"))?;
        return Ok(if visible.is_some() { reported_rows } else { 0 });
    }
    conn.exec_first::<u64, _, _>(
        format!("SELECT COUNT(*) FROM `{table}` WHERE import_batch_id=?"),
        (batch_id,),
    )
    .map(|value| value.unwrap_or(0))
    .map_err(|err| format!("failed to count RAW rows in {table}: {err}"))
}

fn load_data_warnings(conn: &mut mysql::PooledConn) -> Vec<String> {
    conn.query_map(
        "SHOW WARNINGS LIMIT 20",
        |(level, code, message): (String, u16, String)| format!("{level} {code}: {message}"),
    )
    .unwrap_or_default()
}

fn load_data_field_terminator(delimiter: u8) -> Result<&'static str, String> {
    match delimiter {
        b',' => Ok(","),
        b'\t' => Ok("\\t"),
        b';' => Ok(";"),
        other => Err(format!("unsupported CSV delimiter byte: {other}")),
    }
}

fn delimiter_label(delimiter: u8) -> &'static str {
    match delimiter {
        b'\t' => "tab",
        b';' => "semicolon",
        _ => "comma",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        load_data_field_terminator, source_index_for_column, value_for_column, HeaderAliases,
    };
    use csv::StringRecord;
    use std::collections::HashMap;

    #[test]
    fn value_for_column_missing_optional_returns_none_without_positional_fallback() {
        let headers = StringRecord::from(vec!["ID", "Subscriber Account"]);
        let row = StringRecord::from(vec!["1", "acct-001"]);
        let header_index: HashMap<String, usize> = headers
            .iter()
            .enumerate()
            .map(|(index, header)| (crate::header_normalizer::normalize_header(header), index))
            .collect();
        let aliases: HeaderAliases = HashMap::new();

        assert_eq!(
            value_for_column("user_type", &header_index, &aliases, &row),
            None
        );
    }

    #[test]
    fn load_data_mapping_resolves_reordered_alias_headers() {
        let headers = StringRecord::from(vec!["Application", "Subscriber Account"]);
        let header_index: HashMap<String, usize> = headers
            .iter()
            .enumerate()
            .map(|(index, header)| (crate::header_normalizer::normalize_header(header), index))
            .collect();
        let mut aliases: HeaderAliases = HashMap::new();
        aliases.insert(
            "user_account".to_string(),
            vec!["subscriber_account".to_string()],
        );

        assert_eq!(
            source_index_for_column("user_account", &header_index, &aliases),
            Some(1)
        );
        assert_eq!(
            source_index_for_column("application", &header_index, &aliases),
            Some(0)
        );
    }

    #[test]
    fn load_data_uses_only_supported_detected_delimiters() {
        assert_eq!(load_data_field_terminator(b',').unwrap(), ",");
        assert_eq!(load_data_field_terminator(b'\t').unwrap(), "\\t");
        assert_eq!(load_data_field_terminator(b';').unwrap(), ";");
        assert!(load_data_field_terminator(b'|').is_err());
    }
}

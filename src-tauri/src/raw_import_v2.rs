use std::collections::HashMap;
use std::path::Path;

use csv::StringRecord;
use mysql::prelude::*;

use crate::db;
use crate::models::RawLoadRequest;
use crate::sql_runner::escape_sql_literal;

#[derive(Clone, Copy)]
struct RawSpec {
    table: &'static str,
    columns: &'static [&'static str],
}

type HeaderAliases = HashMap<String, Vec<String>>;

const TCP_COLUMNS: &[&str] = &[
    "user_account", "user_mac", "user_type", "universal_video_applications", "statistics_duration",
    "local_ip_address", "server_ip", "device_characteristic", "device_description", "bras", "olt", "pon", "wan_type",
    "vmos", "connection_establishment_success_rate", "connection_establishment_delay_ms",
    "upstream_data_transmission_rtt_ms", "downstream_data_transmission_rtt_ms", "network_side_rtt_ms", "subscriber_side_rtt_ms",
    "user_avg_download_rate_kbps", "max_single_flow_rate_kbps", "network_side_downstream_packet_loss_rate", "user_side_downstream_packet_loss_rate",
    "network_side_upstream_packet_loss_rate", "user_side_upstream_packet_loss_rate", "throughput_avg_bandwidth_kbps", "user_avg_effective_download_rate_kbps",
    "download_fluency", "downloaded_data_volume_kb", "effective_download_duration_s", "video_download_duration_s", "wifi_delay_ms",
];

const GAME_COLUMNS: &[&str] = &[
    "user_account", "user_mac", "user_type", "application_protocol", "statistical_time",
    "local_ip_address", "server_ip", "device_characteristic", "device_description", "bras", "olt", "pon", "wan_type",
    "mos", "connection_establishment_success_rate", "connection_establishment_delay_ms",
    "upstream_data_transmission_rtt_ms", "downstream_data_transmission_rtt_ms", "network_side_rtt_ms", "subscriber_side_rtt_ms",
    "network_side_downstream_packet_loss_rate", "user_side_downstream_packet_loss_rate", "upstream_rtt_jitter_ms", "downstream_rtt_jitter_ms",
    "game_duration_s", "single_flow_rate_kbps", "wifi_delay_ms",
];

const CRM_COLUMNS: &[&str] = &[
    "crm_user_id", "user_account", "user_mac", "current_plan_name", "current_plan_speed_mbps", "current_arpu",
    "contract_status", "arrears_flag", "blacklist_flag",
];

const COVERAGE_COLUMNS: &[&str] = &[
    "area_key", "city", "neighborhood", "hp", "hc", "ftth_available_flag", "build_priority",
];

const REACHABILITY_COLUMNS: &[&str] = &[
    "crm_user_id", "user_account", "phone_available_flag", "sms_available_flag", "app_push_available_flag", "last_contact_result",
];

pub fn start_raw_load(req: RawLoadRequest) -> Result<String, String> {
    let mode = req.mode.clone().unwrap_or_else(|| if req.settings.local_infile.unwrap_or(true) { "load_data".to_string() } else { "streaming_insert".to_string() }).to_lowercase();
    if mode == "streaming_insert" || mode == "insert" || mode == "fallback" {
        streaming_insert(req)
    } else {
        mapped_load_data_or_fallback(req)
    }
}

fn raw_spec(data_type: &str) -> Result<RawSpec, String> {
    match data_type.to_lowercase().as_str() {
        "tcp" => Ok(RawSpec { table: "raw_tcp_detail_import", columns: TCP_COLUMNS }),
        "game" => Ok(RawSpec { table: "raw_game_detail_import", columns: GAME_COLUMNS }),
        "crm" => Ok(RawSpec { table: "raw_crm_user_import", columns: CRM_COLUMNS }),
        "coverage" => Ok(RawSpec { table: "raw_ftth_coverage_import", columns: COVERAGE_COLUMNS }),
        "reachability" => Ok(RawSpec { table: "raw_reachability_import", columns: REACHABILITY_COLUMNS }),
        other => Err(format!("unsupported raw data type: {other}")),
    }
}

fn mapped_load_data_or_fallback(req: RawLoadRequest) -> Result<String, String> {
    let spec = raw_spec(&req.data_type)?;
    let mut conn = db::conn(&req.settings)?;
    let aliases = load_header_aliases(&mut conn, &req.data_type);
    let header_check = header_order_matches(&req.file_path, spec.columns, &aliases)?;
    if !header_check {
        let _ = conn.exec_drop(
            "UPDATE meta_import_batch SET message='LOAD DATA column order mismatch; switched to streaming mapped insert' WHERE import_batch_id=?",
            (&req.import_batch_id,),
        );
        drop(conn);
        return streaming_insert(req);
    }
    load_data(req, spec)
}

fn load_data(req: RawLoadRequest, spec: RawSpec) -> Result<String, String> {
    let mut conn = db::conn(&req.settings)?;
    let file_name = source_file_name(&req.file_path);
    let path = escape_sql_literal(&req.file_path.replace('\\', "/"));
    let batch_id = escape_sql_literal(&req.import_batch_id);
    let source_name = escape_sql_literal(&file_name);
    let columns = spec.columns.join(", ");
    conn.exec_drop("UPDATE meta_import_batch SET status='running', started_at=NOW(), total_rows=NULL, imported_rows=0, message='raw mapped load_data started' WHERE import_batch_id=?", (&req.import_batch_id,)).map_err(|err| format!("failed to mark batch running: {err}"))?;
    let sql = format!("LOAD DATA LOCAL INFILE '{path}' INTO TABLE {} CHARACTER SET utf8mb4 FIELDS TERMINATED BY ',' ENCLOSED BY '\"' LINES TERMINATED BY '\n' IGNORE 1 LINES ({columns}) SET import_batch_id='{batch_id}', source_file_name='{source_name}', source_line_no=NULL", spec.table);
    match conn.query_drop(sql) {
        Ok(_) => {
            let rows = conn.affected_rows();
            finalize_batch(&mut conn, &req.import_batch_id, rows, rows, "raw mapped load_data finished")?;
            Ok(format!("raw mapped load_data finished: table={}, rows={rows}", spec.table))
        }
        Err(err) => {
            let msg = format!("raw mapped load_data failed: {err}");
            mark_failed(&mut conn, &req.import_batch_id, &msg);
            Err(msg)
        }
    }
}

fn streaming_insert(req: RawLoadRequest) -> Result<String, String> {
    let spec = raw_spec(&req.data_type)?;
    let mut conn = db::conn(&req.settings)?;
    let aliases = load_header_aliases(&mut conn, &req.data_type);
    let file_name = source_file_name(&req.file_path);
    conn.exec_drop("UPDATE meta_import_batch SET status='running', started_at=NOW(), total_rows=NULL, imported_rows=0, message='mapped streaming insert started' WHERE import_batch_id=?", (&req.import_batch_id,)).map_err(|err| format!("failed to mark batch running: {err}"))?;

    let mut reader = csv::ReaderBuilder::new().flexible(true).from_path(&req.file_path).map_err(|err| format!("failed to open CSV for mapped streaming insert: {err}"))?;
    let headers = reader.headers().map_err(|err| format!("failed to read CSV headers: {err}"))?.clone();
    let header_index = header_index(&headers);
    let mut rows = Vec::with_capacity(500);
    let mut source_line_no = 1_u64;
    let mut total_rows = 0_u64;

    for row in reader.records() {
        source_line_no += 1;
        let row = row.map_err(|err| format!("failed to read CSV row {source_line_no}: {err}"))?;
        rows.push(row_to_values(&req.import_batch_id, &file_name, source_line_no, spec.columns, &header_index, &aliases, &row));
        if rows.len() >= 500 {
            flush_rows(&mut conn, spec, &rows)?;
            total_rows += rows.len() as u64;
            update_progress(&mut conn, &req.import_batch_id, total_rows, "mapped streaming insert running")?;
            rows.clear();
        }
    }
    if !rows.is_empty() {
        flush_rows(&mut conn, spec, &rows)?;
        total_rows += rows.len() as u64;
        update_progress(&mut conn, &req.import_batch_id, total_rows, "mapped streaming insert running")?;
    }
    finalize_batch(&mut conn, &req.import_batch_id, total_rows, total_rows, "mapped streaming insert finished")?;
    Ok(format!("mapped streaming insert finished: table={}, rows={total_rows}", spec.table))
}

fn load_header_aliases(conn: &mut mysql::PooledConn, data_type: &str) -> HeaderAliases {
    let rows: Result<Vec<(String, String)>, _> = conn.exec(
        "SELECT target_column, source_header FROM cfg_import_field_mapping WHERE data_type=? AND active_flag=1 ORDER BY target_column, priority, source_header",
        (data_type,),
    );
    let mut aliases: HeaderAliases = HashMap::new();
    if let Ok(rows) = rows {
        for (target_column, source_header) in rows {
            aliases.entry(normalize_header(&target_column)).or_default().push(normalize_header(&source_header));
        }
    }
    aliases
}

fn header_order_matches(file_path: &str, columns: &[&str], aliases: &HeaderAliases) -> Result<bool, String> {
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_path(file_path).map_err(|err| format!("failed to open CSV for header check: {err}"))?;
    let headers = reader.headers().map_err(|err| format!("failed to read CSV headers for header check: {err}"))?.clone();
    if headers.len() < columns.len() { return Ok(false); }
    for (index, column) in columns.iter().enumerate() {
        let Some(header) = headers.get(index) else { return Ok(false); };
        if !header_matches_column(header, column, aliases) { return Ok(false); }
    }
    Ok(true)
}

fn header_index(headers: &StringRecord) -> HashMap<String, usize> {
    headers.iter().enumerate().map(|(index, header)| (normalize_header(header), index)).collect()
}

fn row_to_values(import_batch_id: &str, source_file_name: &str, source_line_no: u64, columns: &[&str], header_index: &HashMap<String, usize>, aliases: &HeaderAliases, row: &StringRecord) -> Vec<String> {
    let mut values = vec![sql_literal(import_batch_id), sql_literal(source_file_name), source_line_no.to_string()];
    for (pos, col) in columns.iter().enumerate() {
        values.push(sql_literal(value_for_column(col, pos, header_index, aliases, row).unwrap_or("")));
    }
    values
}

fn value_for_column<'a>(column: &str, pos: usize, header_index: &HashMap<String, usize>, aliases: &HeaderAliases, row: &'a StringRecord) -> Option<&'a str> {
    let normalized = normalize_header(column);
    if let Some(value) = header_index.get(&normalized).and_then(|idx| row.get(*idx)) { return Some(value); }
    if let Some(alias_list) = aliases.get(&normalized) {
        for alias in alias_list {
            if let Some(value) = header_index.get(alias).and_then(|idx| row.get(*idx)) { return Some(value); }
        }
    }
    row.get(pos)
}

fn header_matches_column(header: &str, column: &str, aliases: &HeaderAliases) -> bool {
    let header = normalize_header(header);
    let column = normalize_header(column);
    header == column || aliases.get(&column).map(|items| items.iter().any(|alias| alias == &header)).unwrap_or(false)
}

fn normalize_header(value: &str) -> String {
    value.trim().trim_start_matches('\u{feff}').to_lowercase().replace(' ', "_").replace('-', "_")
}

fn source_file_name(file_path: &str) -> String {
    Path::new(file_path).file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| file_path.to_string())
}

fn sql_literal(value: &str) -> String {
    if value.trim().is_empty() { "NULL".to_string() } else { format!("'{}'", escape_sql_literal(value)) }
}

fn flush_rows(conn: &mut mysql::PooledConn, spec: RawSpec, rows: &[Vec<String>]) -> Result<(), String> {
    if rows.is_empty() { return Ok(()); }
    let mut insert_columns = vec!["import_batch_id", "source_file_name", "source_line_no"];
    insert_columns.extend_from_slice(spec.columns);
    let values = rows.iter().map(|row| format!("({})", row.join(", "))).collect::<Vec<_>>().join(", ");
    let sql = format!("INSERT INTO {} ({}) VALUES {values}", spec.table, insert_columns.join(", "));
    conn.query_drop(sql).map_err(|err| format!("failed to insert mapped streaming RAW rows: {err}"))
}

fn update_progress(conn: &mut mysql::PooledConn, batch_id: &str, rows: u64, message: &str) -> Result<(), String> {
    conn.exec_drop("UPDATE meta_import_batch SET imported_rows=?, message=? WHERE import_batch_id=?", (rows, message, batch_id)).map_err(|err| format!("failed to update import progress: {err}"))
}

fn finalize_batch(conn: &mut mysql::PooledConn, batch_id: &str, total_rows: u64, imported_rows: u64, message: &str) -> Result<(), String> {
    conn.exec_drop("UPDATE meta_import_batch SET status='success', total_rows=?, imported_rows=?, finished_at=NOW(), message=? WHERE import_batch_id=?", (total_rows, imported_rows, message, batch_id)).map_err(|err| format!("failed to finalize import batch: {err}"))
}

fn mark_failed(conn: &mut mysql::PooledConn, batch_id: &str, message: &str) {
    let _ = conn.exec_drop("UPDATE meta_import_batch SET status='failed', finished_at=NOW(), message=? WHERE import_batch_id=?", (message, batch_id));
}

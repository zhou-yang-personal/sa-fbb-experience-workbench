use std::net::Ipv4Addr;

use mysql::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::batch_tables;
use crate::db;
use crate::migrations;
use crate::models::{ack, CommandAck, MySqlSettings};

#[derive(Debug, Clone, Serialize)]
pub struct AccessRuleSetRow {
    pub rule_set_id: String,
    pub version: i64,
    pub rule_set_name: String,
    pub status: String,
    pub rule_count: i64,
    pub published_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessIpRangeRow {
    pub rule_id: String,
    pub rule_set_id: String,
    pub rule_name: String,
    pub cidr: Option<String>,
    pub start_ip: String,
    pub end_ip: String,
    pub access_type: String,
    pub priority: i32,
    pub enabled: bool,
    pub notes: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessRuleUpsertRequest {
    pub settings: MySqlSettings,
    pub rule_set_id: String,
    pub rule_id: Option<String>,
    pub rule_name: String,
    pub cidr: Option<String>,
    pub start_ip: Option<String>,
    pub end_ip: Option<String>,
    pub access_type: String,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessRuleDeleteRequest {
    pub settings: MySqlSettings,
    pub rule_set_id: String,
    pub rule_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessRulePublishRequest {
    pub settings: MySqlSettings,
    pub rule_set_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessRuleApplyBatchRequest {
    pub settings: MySqlSettings,
    pub rule_set_id: String,
    pub import_batch_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessRulePreviewRequest {
    pub settings: MySqlSettings,
    pub rule_set_id: String,
    pub import_batch_id: String,
    pub sample_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessRuleValidationResult {
    pub valid: bool,
    pub rule_count: i64,
    pub enabled_rule_count: i64,
    pub conflict_count: i64,
    pub invalid_rule_count: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessRulePreviewResult {
    pub sample_ip_count: i64,
    pub classified_ip_count: i64,
    pub cable_ip_count: i64,
    pub ftth_ip_count: i64,
    pub other_ip_count: i64,
    pub unmatched_ip_count: i64,
    pub coverage_pct: f64,
    pub sample_limit: u64,
    pub message: String,
}

fn prepare(settings: &MySqlSettings) -> Result<mysql::PooledConn, String> {
    migrations::ensure_access_schema(settings)?;
    db::conn(settings)
}

fn fetch_rule_set(
    conn: &mut mysql::PooledConn,
    rule_set_id: &str,
) -> Result<AccessRuleSetRow, String> {
    conn.exec_first(
        "SELECT s.rule_set_id, CAST(s.version AS SIGNED), s.rule_set_name, s.status, CAST(COUNT(r.rule_id) AS SIGNED), DATE_FORMAT(s.published_at, '%Y-%m-%d %H:%i:%s'), DATE_FORMAT(s.updated_at, '%Y-%m-%d %H:%i:%s') FROM meta_access_rule_set s LEFT JOIN dim_access_ip_range r ON r.rule_set_id=s.rule_set_id WHERE s.rule_set_id=? GROUP BY s.rule_set_id, s.version, s.rule_set_name, s.status, s.published_at, s.updated_at",
        (rule_set_id,),
    )
    .map_err(|err| format!("failed to read access rule set: {err}"))?
    .map(|(rule_set_id, version, rule_set_name, status, rule_count, published_at, updated_at)| AccessRuleSetRow {
        rule_set_id,
        version,
        rule_set_name,
        status,
        rule_count,
        published_at,
        updated_at,
    })
    .ok_or_else(|| format!("access rule set not found: {rule_set_id}"))
}

fn ensure_draft(conn: &mut mysql::PooledConn, rule_set_id: &str) -> Result<(), String> {
    let status: Option<String> = conn
        .exec_first(
            "SELECT status FROM meta_access_rule_set WHERE rule_set_id=?",
            (rule_set_id,),
        )
        .map_err(|err| format!("failed to inspect access rule set: {err}"))?;
    match status.as_deref() {
        Some("draft") => Ok(()),
        Some(other) => Err(format!(
            "access rule set is {other}; only draft rule sets can be edited"
        )),
        None => Err(format!("access rule set not found: {rule_set_id}")),
    }
}

fn normalize_access_type(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "CABLE" | "FTTH" | "OTHER" => Ok(normalized),
        _ => Err("access_type must be CABLE, FTTH, or OTHER".to_string()),
    }
}

fn ipv4_number(value: &str) -> Result<u32, String> {
    value
        .trim()
        .parse::<Ipv4Addr>()
        .map(u32::from)
        .map_err(|_| format!("invalid IPv4 address: {}", value.trim()))
}

fn normalize_range(
    cidr: Option<&str>,
    start_ip: Option<&str>,
    end_ip: Option<&str>,
) -> Result<(Option<String>, String, String, u32, u32), String> {
    if let Some(cidr_text) = cidr.map(str::trim).filter(|value| !value.is_empty()) {
        let (address, prefix_text) = cidr_text
            .split_once('/')
            .ok_or_else(|| "CIDR must look like 10.20.0.0/16".to_string())?;
        let ip = ipv4_number(address)?;
        let prefix = prefix_text
            .parse::<u8>()
            .map_err(|_| format!("invalid CIDR prefix: {prefix_text}"))?;
        if prefix > 32 {
            return Err("CIDR prefix must be between 0 and 32".to_string());
        }
        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        let start = ip & mask;
        let end = start | !mask;
        return Ok((
            Some(format!("{}/{}", Ipv4Addr::from(start), prefix)),
            Ipv4Addr::from(start).to_string(),
            Ipv4Addr::from(end).to_string(),
            start,
            end,
        ));
    }
    let start_text = start_ip
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "CIDR or start_ip is required".to_string())?;
    let end_text = end_ip
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "end_ip is required when CIDR is empty".to_string())?;
    let start = ipv4_number(start_text)?;
    let end = ipv4_number(end_text)?;
    if start > end {
        return Err("start_ip must be less than or equal to end_ip".to_string());
    }
    Ok((
        None,
        Ipv4Addr::from(start).to_string(),
        Ipv4Addr::from(end).to_string(),
        start,
        end,
    ))
}

#[tauri::command]
pub fn access_rule_list_sets(settings: MySqlSettings) -> Result<Vec<AccessRuleSetRow>, String> {
    let mut conn = prepare(&settings)?;
    conn.query_map(
        "SELECT s.rule_set_id, CAST(s.version AS SIGNED), s.rule_set_name, s.status, CAST(COUNT(r.rule_id) AS SIGNED), DATE_FORMAT(s.published_at, '%Y-%m-%d %H:%i:%s'), DATE_FORMAT(s.updated_at, '%Y-%m-%d %H:%i:%s') FROM meta_access_rule_set s LEFT JOIN dim_access_ip_range r ON r.rule_set_id=s.rule_set_id GROUP BY s.rule_set_id, s.version, s.rule_set_name, s.status, s.published_at, s.updated_at ORDER BY s.version DESC",
        |(rule_set_id, version, rule_set_name, status, rule_count, published_at, updated_at)| AccessRuleSetRow {
            rule_set_id,
            version,
            rule_set_name,
            status,
            rule_count,
            published_at,
            updated_at,
        },
    )
    .map_err(|err| format!("failed to list access rule sets: {err}"))
}

#[tauri::command]
pub fn access_rule_get_or_create_draft(
    settings: MySqlSettings,
) -> Result<AccessRuleSetRow, String> {
    let mut conn = prepare(&settings)?;
    let existing: Option<String> = conn
        .query_first(
            "SELECT rule_set_id FROM meta_access_rule_set WHERE status='draft' ORDER BY version DESC LIMIT 1",
        )
        .map_err(|err| format!("failed to find access rule draft: {err}"))?;
    if let Some(rule_set_id) = existing {
        return fetch_rule_set(&mut conn, &rule_set_id);
    }

    let version: i64 = conn
        .query_first::<i64, _>(
            "SELECT CAST(COALESCE(MAX(version),0)+1 AS SIGNED) FROM meta_access_rule_set",
        )
        .map_err(|err| format!("failed to allocate access rule version: {err}"))?
        .unwrap_or(1);
    let rule_set_id = format!("ACCESS_{}", Uuid::new_v4().simple());
    conn.exec_drop(
        "INSERT INTO meta_access_rule_set (rule_set_id, version, rule_set_name, status, notes) VALUES (?, ?, ?, 'draft', 'Draft created from the latest published access rule set')",
        (&rule_set_id, version, format!("Access classification v{version}")),
    )
    .map_err(|err| format!("failed to create access rule draft: {err}"))?;

    let published: Option<String> = conn
        .query_first(
            "SELECT rule_set_id FROM meta_access_rule_set WHERE status='published' ORDER BY published_at DESC, version DESC LIMIT 1",
        )
        .map_err(|err| format!("failed to resolve published access rules: {err}"))?;
    if let Some(source_rule_set_id) = published {
        conn.exec_drop(
            "INSERT INTO dim_access_ip_range (rule_id, rule_set_id, rule_name, cidr, start_ip, end_ip, start_ip_num, end_ip_num, access_type, priority, enabled, notes) SELECT CONCAT('IPR_', REPLACE(UUID(),'-','')), ?, rule_name, cidr, start_ip, end_ip, start_ip_num, end_ip_num, access_type, priority, enabled, notes FROM dim_access_ip_range WHERE rule_set_id=?",
            (&rule_set_id, source_rule_set_id),
        )
        .map_err(|err| format!("failed to copy published access rules into draft: {err}"))?;
    }
    fetch_rule_set(&mut conn, &rule_set_id)
}

#[tauri::command]
pub fn access_rule_list(
    settings: MySqlSettings,
    rule_set_id: String,
) -> Result<Vec<AccessIpRangeRow>, String> {
    let mut conn = prepare(&settings)?;
    conn.exec_map(
        "SELECT rule_id, rule_set_id, rule_name, cidr, start_ip, end_ip, access_type, priority, enabled, notes, DATE_FORMAT(updated_at, '%Y-%m-%d %H:%i:%s') FROM dim_access_ip_range WHERE rule_set_id=? ORDER BY priority, start_ip_num, end_ip_num, rule_name",
        (&rule_set_id,),
        |(rule_id, rule_set_id, rule_name, cidr, start_ip, end_ip, access_type, priority, enabled, notes, updated_at): (String, String, String, Option<String>, String, String, String, i32, u8, Option<String>, String)| AccessIpRangeRow {
            rule_id,
            rule_set_id,
            rule_name,
            cidr,
            start_ip,
            end_ip,
            access_type,
            priority,
            enabled: enabled != 0,
            notes,
            updated_at,
        },
    )
    .map_err(|err| format!("failed to list access IP ranges: {err}"))
}

#[tauri::command]
pub fn access_rule_upsert(req: AccessRuleUpsertRequest) -> Result<AccessIpRangeRow, String> {
    let mut conn = prepare(&req.settings)?;
    ensure_draft(&mut conn, &req.rule_set_id)?;
    let rule_name = req.rule_name.trim();
    if rule_name.is_empty() {
        return Err("rule_name is required".to_string());
    }
    let access_type = normalize_access_type(&req.access_type)?;
    let (cidr, start_ip, end_ip, start_num, end_num) = normalize_range(
        req.cidr.as_deref(),
        req.start_ip.as_deref(),
        req.end_ip.as_deref(),
    )?;
    let enabled = req.enabled.unwrap_or(true);
    let rule_id = req
        .rule_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("IPR_{}", Uuid::new_v4().simple()));
    if enabled {
        let overlap: Option<(String, String)> = conn
            .exec_first(
                "SELECT rule_name, CONCAT(start_ip, ' - ', end_ip) FROM dim_access_ip_range WHERE rule_set_id=? AND enabled=1 AND rule_id<>? AND NOT (? < start_ip_num OR ? > end_ip_num) ORDER BY priority LIMIT 1",
                (&req.rule_set_id, &rule_id, end_num as u64, start_num as u64),
            )
            .map_err(|err| format!("failed to validate access range overlap: {err}"))?;
        if let Some((name, range)) = overlap {
            return Err(format!("IP range overlaps enabled rule '{name}' ({range})"));
        }
    }
    let exists: Option<u8> = conn
        .exec_first(
            "SELECT 1 FROM dim_access_ip_range WHERE rule_id=? AND rule_set_id=?",
            (&rule_id, &req.rule_set_id),
        )
        .map_err(|err| format!("failed to inspect access rule: {err}"))?;
    if exists.is_some() {
        conn.exec_drop(
            "UPDATE dim_access_ip_range SET rule_name=?, cidr=?, start_ip=?, end_ip=?, start_ip_num=?, end_ip_num=?, access_type=?, priority=?, enabled=?, notes=?, updated_at=NOW() WHERE rule_id=? AND rule_set_id=?",
            (rule_name, &cidr, &start_ip, &end_ip, start_num as u64, end_num as u64, &access_type, req.priority.unwrap_or(100), enabled, req.notes.as_deref(), &rule_id, &req.rule_set_id),
        )
        .map_err(|err| format!("failed to update access rule: {err}"))?;
    } else {
        conn.exec_drop(
            "INSERT INTO dim_access_ip_range (rule_id, rule_set_id, rule_name, cidr, start_ip, end_ip, start_ip_num, end_ip_num, access_type, priority, enabled, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (&rule_id, &req.rule_set_id, rule_name, &cidr, &start_ip, &end_ip, start_num as u64, end_num as u64, &access_type, req.priority.unwrap_or(100), enabled, req.notes.as_deref()),
        )
        .map_err(|err| format!("failed to create access rule: {err}"))?;
    }
    conn.exec_first(
        "SELECT rule_id, rule_set_id, rule_name, cidr, start_ip, end_ip, access_type, priority, enabled, notes, DATE_FORMAT(updated_at, '%Y-%m-%d %H:%i:%s') FROM dim_access_ip_range WHERE rule_id=?",
        (&rule_id,),
    )
    .map_err(|err| format!("failed to read saved access rule: {err}"))?
    .map(|(rule_id, rule_set_id, rule_name, cidr, start_ip, end_ip, access_type, priority, enabled, notes, updated_at): (String, String, String, Option<String>, String, String, String, i32, u8, Option<String>, String)| AccessIpRangeRow {
        rule_id,
        rule_set_id,
        rule_name,
        cidr,
        start_ip,
        end_ip,
        access_type,
        priority,
        enabled: enabled != 0,
        notes,
        updated_at,
    })
    .ok_or_else(|| "saved access rule was not found".to_string())
}

#[tauri::command]
pub fn access_rule_delete(req: AccessRuleDeleteRequest) -> Result<CommandAck, String> {
    let mut conn = prepare(&req.settings)?;
    ensure_draft(&mut conn, &req.rule_set_id)?;
    conn.exec_drop(
        "DELETE FROM dim_access_ip_range WHERE rule_set_id=? AND rule_id=?",
        (&req.rule_set_id, &req.rule_id),
    )
    .map_err(|err| format!("failed to delete access rule: {err}"))?;
    if conn.affected_rows() == 0 {
        return Err("access rule not found in the selected draft".to_string());
    }
    Ok(ack("access rule deleted from draft"))
}

fn validate_rule_set_internal(
    conn: &mut mysql::PooledConn,
    rule_set_id: &str,
) -> Result<AccessRuleValidationResult, String> {
    let (rule_count, enabled_rule_count, invalid_rule_count): (i64, i64, i64) = conn
        .exec_first(
            "SELECT CAST(COUNT(*) AS SIGNED), CAST(COALESCE(SUM(enabled=1),0) AS SIGNED), CAST(COALESCE(SUM(start_ip_num>end_ip_num OR access_type NOT IN ('CABLE','FTTH','OTHER')),0) AS SIGNED) FROM dim_access_ip_range WHERE rule_set_id=?",
            (rule_set_id,),
        )
        .map_err(|err| format!("failed to validate access rules: {err}"))?
        .unwrap_or((0, 0, 0));
    let conflict_count: i64 = conn
        .exec_first(
            "SELECT CAST(COUNT(*) AS SIGNED) FROM dim_access_ip_range a JOIN dim_access_ip_range b ON b.rule_set_id=a.rule_set_id AND b.rule_id>a.rule_id AND b.enabled=1 AND a.enabled=1 AND NOT (a.end_ip_num < b.start_ip_num OR a.start_ip_num > b.end_ip_num) WHERE a.rule_set_id=?",
            (rule_set_id,),
        )
        .map_err(|err| format!("failed to detect access rule conflicts: {err}"))?
        .unwrap_or(0);
    let valid = enabled_rule_count > 0 && invalid_rule_count == 0 && conflict_count == 0;
    let message = if valid {
        format!("{enabled_rule_count} enabled IPv4 ranges are ready to publish")
    } else if enabled_rule_count == 0 {
        "At least one enabled IPv4 range is required".to_string()
    } else {
        format!("Resolve {conflict_count} overlaps and {invalid_rule_count} invalid rules before publishing")
    };
    Ok(AccessRuleValidationResult {
        valid,
        rule_count,
        enabled_rule_count,
        conflict_count,
        invalid_rule_count,
        message,
    })
}

#[tauri::command]
pub fn access_rule_validate(
    settings: MySqlSettings,
    rule_set_id: String,
) -> Result<AccessRuleValidationResult, String> {
    let mut conn = prepare(&settings)?;
    validate_rule_set_internal(&mut conn, &rule_set_id)
}

#[tauri::command]
pub fn access_rule_publish(req: AccessRulePublishRequest) -> Result<AccessRuleSetRow, String> {
    let mut conn = prepare(&req.settings)?;
    ensure_draft(&mut conn, &req.rule_set_id)?;
    let validation = validate_rule_set_internal(&mut conn, &req.rule_set_id)?;
    if !validation.valid {
        return Err(validation.message);
    }
    let mut tx = conn
        .start_transaction(mysql::TxOpts::default())
        .map_err(|err| format!("failed to start access rule publish transaction: {err}"))?;
    tx.query_drop("UPDATE meta_access_rule_set SET status='archived', updated_at=NOW() WHERE status='published'")
        .map_err(|err| format!("failed to archive previous access rule set: {err}"))?;
    tx.exec_drop(
        "UPDATE meta_access_rule_set SET status='published', published_at=NOW(), updated_at=NOW() WHERE rule_set_id=? AND status='draft'",
        (&req.rule_set_id,),
    )
    .map_err(|err| format!("failed to publish access rule set: {err}"))?;
    if tx.affected_rows() != 1 {
        return Err("access rule draft changed before publish; refresh and retry".to_string());
    }
    tx.commit()
        .map_err(|err| format!("failed to commit access rule publish: {err}"))?;
    fetch_rule_set(&mut conn, &req.rule_set_id)
}

#[tauri::command]
pub fn access_rule_apply_to_batch(req: AccessRuleApplyBatchRequest) -> Result<CommandAck, String> {
    let mut conn = prepare(&req.settings)?;
    let version: Option<i64> = conn
        .exec_first(
            "SELECT CAST(version AS SIGNED) FROM meta_access_rule_set WHERE rule_set_id=? AND status='published'",
            (&req.rule_set_id,),
        )
        .map_err(|err| format!("failed to inspect published access rule set: {err}"))?;
    let Some(version) = version else {
        return Err("only a published access rule set can be applied to a batch".to_string());
    };
    conn.exec_drop(
        "UPDATE meta_import_batch SET access_rule_set_id=?, access_rule_set_version=?, message=CONCAT(COALESCE(message,''), '; access rules v', ?) WHERE import_batch_id=?",
        (&req.rule_set_id, version, version, &req.import_batch_id),
    )
    .map_err(|err| format!("failed to assign access rules to batch: {err}"))?;
    if conn.affected_rows() == 0 {
        return Err(format!("import batch not found: {}", req.import_batch_id));
    }
    Ok(ack(format!(
        "access rule set v{version} assigned to {}; rerun CLEAN/DWS/ADS to apply it",
        req.import_batch_id
    )))
}

#[tauri::command]
pub fn access_rule_preview(
    req: AccessRulePreviewRequest,
) -> Result<AccessRulePreviewResult, String> {
    let mut conn = prepare(&req.settings)?;
    let data_type: Option<String> = conn
        .exec_first(
            "SELECT data_type FROM meta_import_batch WHERE import_batch_id=?",
            (&req.import_batch_id,),
        )
        .map_err(|err| format!("failed to inspect preview batch: {err}"))?;
    let Some(data_type) = data_type else {
        return Err(format!("import batch not found: {}", req.import_batch_id));
    };
    let raw_base = match data_type.to_ascii_lowercase().as_str() {
        "tcp" => "raw_tcp_detail_import",
        "game" => "raw_game_detail_import",
        _ => return Err(format!(
            "access preview is only available for TCP or Game batches, not data_type={data_type}"
        )),
    };
    let raw_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, raw_base)?;
    let safe_table = batch_tables::sanitize_identifier(&raw_table)?;
    let sample_limit = req.sample_limit.unwrap_or(50_000).clamp(100, 100_000);
    let sql = format!(
        "WITH sample AS (SELECT DISTINCT TRIM(local_ip_address) AS ip_address, INET_ATON(TRIM(local_ip_address)) AS ip_num FROM `{safe_table}` WHERE import_batch_id=? AND INET_ATON(TRIM(local_ip_address)) IS NOT NULL LIMIT {sample_limit}) SELECT CAST(COUNT(*) AS SIGNED), CAST(COALESCE(SUM(r.rule_id IS NOT NULL),0) AS SIGNED), CAST(COALESCE(SUM(r.access_type='CABLE'),0) AS SIGNED), CAST(COALESCE(SUM(r.access_type='FTTH'),0) AS SIGNED), CAST(COALESCE(SUM(r.access_type='OTHER'),0) AS SIGNED), CAST(COALESCE(SUM(r.rule_id IS NULL),0) AS SIGNED) FROM sample s LEFT JOIN dim_access_ip_range r ON r.rule_set_id=? AND r.enabled=1 AND s.ip_num BETWEEN r.start_ip_num AND r.end_ip_num"
    );
    let (
        sample_ip_count,
        classified_ip_count,
        cable_ip_count,
        ftth_ip_count,
        other_ip_count,
        unmatched_ip_count,
    ): (i64, i64, i64, i64, i64, i64) = conn
        .exec_first(sql, (&req.import_batch_id, &req.rule_set_id))
        .map_err(|err| format!("failed to preview access classification: {err}"))?
        .unwrap_or((0, 0, 0, 0, 0, 0));
    let coverage_pct = if sample_ip_count > 0 {
        classified_ip_count as f64 / sample_ip_count as f64 * 100.0
    } else {
        0.0
    };
    Ok(AccessRulePreviewResult {
        sample_ip_count,
        classified_ip_count,
        cable_ip_count,
        ftth_ip_count,
        other_ip_count,
        unmatched_ip_count,
        coverage_pct,
        sample_limit,
        message: "Preview counts distinct valid IPv4 addresses from a bounded RAW sample; it does not modify the batch".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{normalize_access_type, normalize_range};

    #[test]
    fn cidr_is_normalized_to_network_bounds() {
        let (cidr, start, end, start_num, end_num) =
            normalize_range(Some("10.20.3.9/16"), None, None).expect("valid CIDR");
        assert_eq!(cidr.as_deref(), Some("10.20.0.0/16"));
        assert_eq!(start, "10.20.0.0");
        assert_eq!(end, "10.20.255.255");
        assert!(start_num < end_num);
    }

    #[test]
    fn rejects_reversed_range_and_unknown_access_type() {
        assert!(normalize_range(None, Some("10.0.0.10"), Some("10.0.0.1")).is_err());
        assert!(normalize_access_type("dsl").is_err());
    }
}

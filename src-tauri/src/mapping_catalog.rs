use mysql::prelude::*;

use crate::db;
use crate::models::{MetricCard, MySqlSettings};
use crate::sql_runner;

pub const APP_VERSION: &str = "1.0.33";
pub const MAPPING_SEED_VERSION: &str = "1.0.33";

const MAP_SEED: &str = include_str!("../../database/seeds/002_default_mapping_seed.sql");

const CRITICAL_ALIASES: &[(&str, &str, &str)] = &[
    ("tcp", "user_account", "subscriber_account"),
    ("tcp", "user_account", "Subscriber Account"),
    (
        "tcp",
        "downloaded_data_volume_kb",
        "Downloaded Data Volume (KB)",
    ),
    (
        "tcp",
        "effective_download_duration_s",
        "Effective Download Duration (s)",
    ),
    ("tcp", "user_type", "user_type"),
    ("game", "user_type", "user_type"),
];

pub fn ensure_import_mapping_catalog(settings: &MySqlSettings) -> Result<u64, String> {
    let rows = sql_runner::execute_script(settings, MAP_SEED)?;
    let mut conn = db::conn(settings)?;
    ensure_meta_app_config(&mut conn)?;
    upsert_config(&mut conn, "app_version", APP_VERSION)?;
    upsert_config(&mut conn, "mapping_seed_version", MAPPING_SEED_VERSION)?;
    upsert_config_now(&mut conn, "mapping_seed_applied_at")?;
    Ok(rows)
}

pub fn check_import_mapping_catalog(settings: &MySqlSettings) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(settings)?;
    ensure_meta_app_config(&mut conn)?;
    let app_version =
        get_config(&mut conn, "app_version")?.unwrap_or_else(|| "unknown".to_string());
    let seed_version =
        get_config(&mut conn, "mapping_seed_version")?.unwrap_or_else(|| "unknown".to_string());
    let applied_at =
        get_config(&mut conn, "mapping_seed_applied_at")?.unwrap_or_else(|| "unknown".to_string());
    let missing = missing_critical_aliases(&mut conn)?;
    let stale = seed_version != MAPPING_SEED_VERSION || !missing.is_empty();
    let mut cards = vec![
        MetricCard {
            label: "app_version".to_string(),
            value: app_version,
            hint: "meta_app_config".to_string(),
        },
        MetricCard {
            label: "mapping_seed_version".to_string(),
            value: seed_version,
            hint: format!("expected={MAPPING_SEED_VERSION}, applied_at={applied_at}"),
        },
        MetricCard {
            label: "stale_catalog".to_string(),
            value: if stale { "yes" } else { "no" }.to_string(),
            hint: if missing.is_empty() {
                "critical aliases present".to_string()
            } else {
                format!("missing_examples={}", missing.join("|"))
            },
        },
        MetricCard {
            label: "missing_critical_alias_count".to_string(),
            value: missing.len().to_string(),
            hint: missing
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join("|"),
        },
    ];
    for data_type in ["tcp", "game", "crm", "coverage", "reachability"] {
        let required_count: Option<i64> = conn
            .exec_first(
                "SELECT CAST(COUNT(DISTINCT target_column) AS SIGNED) FROM cfg_import_field_mapping WHERE data_type=? AND active_flag=1 AND required_flag=1",
                (data_type,),
            )
            .map_err(|err| format!("failed to inspect required mappings: {err}"))?;
        cards.push(MetricCard {
            label: format!("{data_type}_required_targets"),
            value: required_count.unwrap_or(0).to_string(),
            hint: format!("data_type={data_type}"),
        });
    }
    Ok(cards)
}

pub fn ensure_meta_app_config(conn: &mut mysql::PooledConn) -> Result<(), String> {
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS meta_app_config (
          config_key VARCHAR(128) NOT NULL PRIMARY KEY,
          config_value VARCHAR(512) NOT NULL,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    )
    .map_err(|err| format!("failed to ensure meta_app_config: {err}"))
}

fn missing_critical_aliases(conn: &mut mysql::PooledConn) -> Result<Vec<String>, String> {
    let mut missing = Vec::new();
    for (data_type, target_column, source_header) in CRITICAL_ALIASES {
        let exists: Option<u8> = conn
            .exec_first(
                "SELECT 1 FROM cfg_import_field_mapping WHERE data_type=? AND target_column=? AND source_header=? AND active_flag=1 LIMIT 1",
                (data_type, target_column, source_header),
            )
            .map_err(|err| format!("failed to inspect critical alias: {err}"))?;
        if exists.is_none() {
            missing.push(format!("{data_type}.{target_column}:{source_header}"));
        }
    }
    Ok(missing)
}

fn upsert_config(conn: &mut mysql::PooledConn, key: &str, value: &str) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO meta_app_config (config_key, config_value) VALUES (?, ?) ON DUPLICATE KEY UPDATE config_value=VALUES(config_value), updated_at=NOW()",
        (key, value),
    )
    .map_err(|err| format!("failed to upsert app config {key}: {err}"))
}

fn upsert_config_now(conn: &mut mysql::PooledConn, key: &str) -> Result<(), String> {
    conn.exec_drop(
        "INSERT INTO meta_app_config (config_key, config_value) VALUES (?, DATE_FORMAT(NOW(), '%Y-%m-%d %H:%i:%s')) ON DUPLICATE KEY UPDATE config_value=DATE_FORMAT(NOW(), '%Y-%m-%d %H:%i:%s'), updated_at=NOW()",
        (key,),
    )
    .map_err(|err| format!("failed to upsert app config {key}: {err}"))
}

fn get_config(conn: &mut mysql::PooledConn, key: &str) -> Result<Option<String>, String> {
    conn.exec_first(
        "SELECT config_value FROM meta_app_config WHERE config_key=?",
        (key,),
    )
    .map_err(|err| format!("failed to query app config {key}"))
}

#[cfg(test)]
mod tests {
    use super::{CRITICAL_ALIASES, MAPPING_SEED_VERSION, MAP_SEED};

    #[test]
    fn catalog_seed_baseline_contains_subscriber_account_alias() {
        assert!(MAP_SEED.contains("('tcp','user_account','subscriber_account',1"));
        assert!(MAP_SEED.contains("('tcp','user_account','Subscriber Account',1"));
    }

    #[test]
    fn catalog_seed_makes_sa_user_type_optional() {
        assert!(MAP_SEED.contains("('tcp','user_type','user_type',0"));
        assert!(MAP_SEED.contains("('game','user_type','user_type',0"));
    }

    #[test]
    fn critical_aliases_track_universal_video_contract() {
        assert_eq!(MAPPING_SEED_VERSION, "1.0.33");
        assert!(CRITICAL_ALIASES.iter().any(|item| *item == ("tcp", "user_account", "subscriber_account")));
    }
}

use mysql::prelude::*;

use crate::models::MySqlSettings;
use crate::sql_runner;

const CORE_SCHEMA: &str = include_str!("../../database/migrations/001_core_schema.sql");
const EXT_SCHEMA: &str = include_str!("../../database/migrations/002_complete_app_schema.sql");
const MAP_SCHEMA: &str = include_str!("../../database/migrations/003_mapping_schema.sql");
const OBS_SCHEMA: &str = include_str!("../../database/migrations/004_observability_schema.sql");
const PIPELINE_SCHEMA: &str = include_str!("../../database/migrations/005_pipeline_schema.sql");
const ANALYTICS_SCHEMA: &str =
    include_str!("../../database/migrations/006_analytics_dashboard_schema.sql");
const ACCESS_SCHEMA: &str =
    include_str!("../../database/migrations/007_access_classification_schema.sql");
const EXPERIENCE_POLICY_SCHEMA: &str =
    include_str!("../../database/migrations/008_experience_analysis_policy_schema.sql");
const INVESTIGATION_SCHEMA: &str =
    include_str!("../../database/migrations/009_investigation_workspace_schema.sql");
const AGGREGATION_CHECKPOINT_SCHEMA: &str =
    include_str!("../../database/migrations/010_aggregation_checkpoint_schema.sql");
const DECISION_WORKSPACE_SCHEMA: &str =
    include_str!("../../database/migrations/011_decision_workspace_schema.sql");
const APP_MAPPING_SEED: &str = include_str!("../../database/seeds/001_app_mapping_seed.sql");
const MAP_SEED: &str = include_str!("../../database/seeds/002_default_mapping_seed.sql");

pub fn init_database(settings: &MySqlSettings) -> Result<String, String> {
    let core_rows = sql_runner::execute_script(settings, CORE_SCHEMA)?;
    let ext_rows = sql_runner::execute_script(settings, EXT_SCHEMA)?;
    let map_rows = sql_runner::execute_script(settings, MAP_SCHEMA)?;
    let obs_rows = sql_runner::execute_script(settings, OBS_SCHEMA)?;
    let pipeline_rows = sql_runner::execute_script(settings, PIPELINE_SCHEMA)?;
    let analytics_rows = sql_runner::execute_script(settings, ANALYTICS_SCHEMA)?;
    let access_rows = sql_runner::execute_script(settings, ACCESS_SCHEMA)?;
    let experience_policy_rows = sql_runner::execute_script(settings, EXPERIENCE_POLICY_SCHEMA)?;
    let investigation_rows = sql_runner::execute_script(settings, INVESTIGATION_SCHEMA)?;
    let aggregation_checkpoint_rows =
        sql_runner::execute_script(settings, AGGREGATION_CHECKPOINT_SCHEMA)?;
    ensure_decision_workspace_schema(settings)?;
    let decision_workspace_rows = 0;
    let mut conn = crate::db::conn(settings)?;
    ensure_access_columns_for_table(&mut conn, "meta_import_batch")?;
    ensure_access_columns_for_table(&mut conn, "meta_access_rule_set")?;
    ensure_access_columns_for_table(&mut conn, "meta_analysis_run")?;
    ensure_access_columns_for_table(&mut conn, "meta_decision_rule_profile")?;
    ensure_access_columns_for_table(&mut conn, "dwd_tcp_detail_clean")?;
    ensure_access_columns_for_table(&mut conn, "dwd_game_detail_clean")?;
    let seed_rows = sql_runner::execute_script(settings, APP_MAPPING_SEED)?;
    let map_seed_rows = sql_runner::execute_script(settings, MAP_SEED)?;
    crate::mapping_catalog::ensure_import_mapping_catalog(settings)?;
    Ok(format!("database initialized: core={core_rows}, ext={ext_rows}, map={map_rows}, obs={obs_rows}, pipeline={pipeline_rows}, analytics={analytics_rows}, access={access_rows}, experience_policy={experience_policy_rows}, investigation={investigation_rows}, aggregation_checkpoint={aggregation_checkpoint_rows}, decision_workspace={decision_workspace_rows}, seed={seed_rows}, map_seed={map_seed_rows}"))
}

pub fn ensure_access_schema(settings: &MySqlSettings) -> Result<(), String> {
    sql_runner::execute_script(settings, ACCESS_SCHEMA)?;
    let mut conn = crate::db::conn(settings)?;
    ensure_access_columns_for_table(&mut conn, "meta_import_batch")?;
    ensure_access_columns_for_table(&mut conn, "meta_access_rule_set")?;
    ensure_access_columns_for_table(&mut conn, "meta_analysis_run")?;
    ensure_access_columns_for_table(&mut conn, "dwd_tcp_detail_clean")?;
    ensure_access_columns_for_table(&mut conn, "dwd_game_detail_clean")
}

pub fn ensure_experience_policy_schema(settings: &MySqlSettings) -> Result<(), String> {
    sql_runner::execute_script(settings, EXPERIENCE_POLICY_SCHEMA)?;
    sql_runner::execute_script(settings, INVESTIGATION_SCHEMA)?;
    sql_runner::execute_script(settings, AGGREGATION_CHECKPOINT_SCHEMA)?;
    ensure_decision_workspace_schema(settings)?;
    let mut conn = crate::db::conn(settings)?;
    ensure_access_columns_for_table(&mut conn, "meta_access_rule_set")?;
    ensure_access_columns_for_table(&mut conn, "meta_analysis_run")?;
    ensure_access_columns_for_table(&mut conn, "meta_decision_rule_profile")
}

pub fn ensure_decision_workspace_schema(settings: &MySqlSettings) -> Result<(), String> {
    let create_only = DECISION_WORKSPACE_SCHEMA
        .split_once("INSERT IGNORE INTO meta_decision_rule_profile")
        .map(|(prefix, _)| prefix)
        .ok_or_else(|| "decision workspace migration is missing its seed boundary".to_string())?;
    sql_runner::execute_script(settings, create_only)?;
    let mut conn = crate::db::conn(settings)?;
    ensure_access_columns_for_table(&mut conn, "meta_decision_rule_profile")?;
    drop(conn);
    sql_runner::execute_script(settings, DECISION_WORKSPACE_SCHEMA).map(|_| ())
}

pub fn ensure_aggregation_checkpoint_schema(settings: &MySqlSettings) -> Result<(), String> {
    sql_runner::execute_script(settings, AGGREGATION_CHECKPOINT_SCHEMA).map(|_| ())
}

fn column_exists(conn: &mut mysql::PooledConn, table: &str, column: &str) -> Result<bool, String> {
    let count: Option<u64> = conn
        .exec_first(
            "SELECT COUNT(*) FROM information_schema.columns WHERE table_schema=DATABASE() AND table_name=? AND column_name=?",
            (table, column),
        )
        .map_err(|err| format!("failed to inspect {table}.{column}: {err}"))?;
    Ok(count.unwrap_or(0) > 0)
}

pub fn ensure_access_columns_for_table(
    conn: &mut mysql::PooledConn,
    table: &str,
) -> Result<(), String> {
    let safe = crate::batch_tables::sanitize_identifier(table)?;
    let definitions: &[(&str, &str)] = if table == "meta_import_batch" {
        &[
            ("access_rule_set_id", "VARCHAR(64) NULL"),
            ("access_rule_set_version", "BIGINT NULL"),
        ]
    } else if table == "meta_access_rule_set" {
        &[(
            "default_access_type",
            "VARCHAR(32) NULL AFTER rule_set_name",
        )]
    } else if table == "meta_analysis_run" {
        &[
            ("access_rule_set_id", "VARCHAR(64) NULL"),
            ("access_rule_set_version", "BIGINT NULL"),
            ("others_access_type", "VARCHAR(32) NULL"),
            ("experience_policy_id", "VARCHAR(64) NULL"),
            ("experience_policy_version", "BIGINT NULL"),
        ]
    } else if table == "meta_decision_rule_profile" {
        &[
            ("opportunity_min_active_days", "BIGINT NOT NULL DEFAULT 2"),
            ("opportunity_min_observations", "BIGINT NOT NULL DEFAULT 10"),
            ("speed_upgrade_min_conditions", "BIGINT NOT NULL DEFAULT 2"),
            ("app_bundle_min_active_days", "BIGINT NOT NULL DEFAULT 3"),
            ("sufficient_app_users", "BIGINT NOT NULL DEFAULT 30"),
            ("sufficient_app_observations", "BIGINT NOT NULL DEFAULT 100"),
            ("attention_app_poor_rate_pct", "DECIMAL(9,4) NOT NULL DEFAULT 10"),
            ("attention_app_persistent_user_rate_pct", "DECIMAL(9,4) NOT NULL DEFAULT 5"),
            ("severe_app_poor_rate_pct", "DECIMAL(9,4) NOT NULL DEFAULT 40"),
            ("severe_app_persistent_user_rate_pct", "DECIMAL(9,4) NOT NULL DEFAULT 20"),
            ("severe_app_severe_user_rate_pct", "DECIMAL(9,4) NOT NULL DEFAULT 10"),
        ]
    } else if table.starts_with("dwd_tcp_detail_clean") {
        &[
            ("source_user_type", "VARCHAR(32) NULL"),
            ("local_ip_address", "VARCHAR(255) NULL"),
            ("server_ip", "TEXT NULL"),
            (
                "access_type_source",
                "VARCHAR(32) NOT NULL DEFAULT 'UNMATCHED'",
            ),
            (
                "access_type_confidence",
                "VARCHAR(32) NOT NULL DEFAULT 'LOW'",
            ),
            ("access_rule_id", "VARCHAR(64) NULL"),
            ("access_rule_set_version", "BIGINT NULL"),
            ("effective_duration_hours", "DECIMAL(24,6) NULL"),
            ("video_duration_hours", "DECIMAL(24,6) NULL"),
            ("avg_download_mbps", "DECIMAL(18,6) NULL"),
            ("throughput_mbps", "DECIMAL(18,6) NULL"),
            ("max_single_flow_mbps", "DECIMAL(18,6) NULL"),
            ("connection_success_pct", "DECIMAL(18,6) NULL"),
            ("connection_delay_ms", "DECIMAL(18,6) NULL"),
            ("download_fluency", "DECIMAL(18,6) NULL"),
            ("upstream_rtt_ms", "DECIMAL(18,6) NULL"),
            ("downstream_rtt_ms", "DECIMAL(18,6) NULL"),
            ("user_up_loss", "DECIMAL(18,6) NULL"),
            ("network_up_loss", "DECIMAL(18,6) NULL"),
        ]
    } else if table.starts_with("dwd_game_detail_clean") {
        &[
            ("source_user_type", "VARCHAR(32) NULL"),
            ("local_ip_address", "VARCHAR(255) NULL"),
            ("server_ip", "TEXT NULL"),
            ("access_type_source", "VARCHAR(32) NOT NULL DEFAULT 'UNMATCHED'"),
            ("access_type_confidence", "VARCHAR(32) NOT NULL DEFAULT 'LOW'"),
            ("access_rule_id", "VARCHAR(64) NULL"),
            ("access_rule_set_version", "BIGINT NULL"),
        ]
    } else if table.starts_with("dws_user_app_period_experience_v2") {
        &[
            ("total_effective_duration_hours", "DECIMAL(24,6) NOT NULL DEFAULT 0"),
            ("total_video_duration_hours", "DECIMAL(24,6) NOT NULL DEFAULT 0"),
            ("active_days", "BIGINT NOT NULL DEFAULT 0"),
            ("avg_download_mbps", "DECIMAL(18,6) NULL"),
            ("avg_throughput_mbps", "DECIMAL(18,6) NULL"),
            ("avg_max_single_flow_mbps", "DECIMAL(18,6) NULL"),
            ("avg_connection_success_pct", "DECIMAL(18,6) NULL"),
            ("avg_connection_delay_ms", "DECIMAL(18,6) NULL"),
            ("avg_download_fluency", "DECIMAL(18,6) NULL"),
            ("avg_upstream_rtt_ms", "DECIMAL(18,6) NULL"),
            ("avg_downstream_rtt_ms", "DECIMAL(18,6) NULL"),
            ("avg_user_up_loss_pct", "DECIMAL(18,6) NULL"),
            ("avg_network_up_loss_pct", "DECIMAL(18,6) NULL"),
        ]
    } else if table.starts_with("dws_user_app_hourly_experience_v2") {
        &[
            ("observation_rows", "BIGINT NOT NULL DEFAULT 0"),
            ("total_download_gb", "DECIMAL(24,6) NOT NULL DEFAULT 0"),
            ("total_effective_duration_hours", "DECIMAL(24,6) NOT NULL DEFAULT 0"),
            ("total_video_duration_hours", "DECIMAL(24,6) NOT NULL DEFAULT 0"),
            ("avg_effective_download_mbps", "DECIMAL(18,6) NULL"),
            ("avg_download_mbps", "DECIMAL(18,6) NULL"),
        ]
    } else {
        return Ok(());
    };
    for (column, definition) in definitions {
        if !column_exists(conn, &safe, column)? {
            conn.query_drop(format!(
                "ALTER TABLE `{safe}` ADD COLUMN `{column}` {definition}"
            ))
            .map_err(|err| format!("failed to add {safe}.{column}: {err}"))?;
        }
    }
    if table == "meta_access_rule_set" {
        let column_shape: Option<(String, Option<String>)> = conn
            .exec_first(
                "SELECT IS_NULLABLE, COLUMN_DEFAULT FROM information_schema.columns WHERE table_schema=DATABASE() AND table_name='meta_access_rule_set' AND column_name='default_access_type'",
                (),
            )
            .map_err(|err| format!("failed to inspect meta_access_rule_set.default_access_type definition: {err}"))?;
        if column_shape
            .as_ref()
            .is_some_and(|(nullable, default)| nullable != "YES" || default.is_some())
        {
            conn.query_drop(
                "ALTER TABLE meta_access_rule_set MODIFY COLUMN default_access_type VARCHAR(32) NULL DEFAULT NULL AFTER rule_set_name",
            )
            .map_err(|err| format!("failed to make Others mapping explicit and nullable: {err}"))?;
        }
        conn.query_drop(
            "UPDATE meta_access_rule_set SET default_access_type=NULL, notes='Draft requires an explicit Others mapping before publish', updated_at=NOW() WHERE status='draft' AND notes='Draft created from the latest published access rule set'",
        )
        .map_err(|err| format!("failed to migrate legacy implicit Others draft: {err}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AGGREGATION_CHECKPOINT_SCHEMA, ANALYTICS_SCHEMA, DECISION_WORKSPACE_SCHEMA};

    #[test]
    fn network_hotspot_schema_uses_bounded_indexes() {
        assert!(ANALYTICS_SCHEMA.contains("id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY"));
        assert!(ANALYTICS_SCHEMA
            .contains("INDEX ix_hotspot_batch (import_batch_id, bras(128), olt(128), pon(128))"));
        assert!(
            !ANALYTICS_SCHEMA.contains("PRIMARY KEY (analysis_run_id, bras, olt, pon, user_type)")
        );
    }

    #[test]
    fn aggregation_checkpoint_schema_has_idempotent_partition_key() {
        assert!(AGGREGATION_CHECKPOINT_SCHEMA.contains(
            "UNIQUE KEY uk_aggregation_partition (analysis_run_id, stage_name, subtask_name, partition_date, partition_hour)"
        ));
        assert!(AGGREGATION_CHECKPOINT_SCHEMA.contains("attempt_count INT NOT NULL DEFAULT 0"));
        assert!(AGGREGATION_CHECKPOINT_SCHEMA.contains("connection_id BIGINT UNSIGNED NULL"));
    }


    #[test]
    fn decision_workspace_rules_and_opportunities_are_versioned() {
        assert!(DECISION_WORKSPACE_SCHEMA.contains("meta_decision_rule_profile"));
        assert!(DECISION_WORKSPACE_SCHEMA.contains("meta_analysis_run_decision_binding"));
        assert!(DECISION_WORKSPACE_SCHEMA.contains("ads_opportunity_user_v3"));
        assert!(DECISION_WORKSPACE_SCHEMA.contains("minimum_app_users"));
    }
}

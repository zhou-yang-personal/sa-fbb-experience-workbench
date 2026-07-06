use std::collections::{HashMap, HashSet};

use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::job_runner::JobStep;
use crate::models::MySqlSettings;
use crate::sql_runner;

#[derive(Debug)]
struct JoinRule {
    left_alias: String,
    left_column: String,
    right_alias: String,
    right_column: String,
}

pub fn build_final_fusion_step(settings: &MySqlSettings, import_batch_id: &str, analysis_run_id: &str) -> Result<JobStep<'static>, String> {
    let crm_on = join_condition(settings, "crm", "c.user_account = b.user_key OR c.user_mac = b.user_key OR c.crm_user_id = b.user_key")?;
    let reach_on = join_condition(settings, "reach", "r.crm_user_id = c.crm_user_id OR r.user_account = c.user_account OR r.user_account = b.user_key")?;
    let coverage_on = join_condition(settings, "coverage", "cv.area_key = c.crm_user_id OR cv.area_key = c.user_account OR cv.area_key = b.user_key")?;
    let ads_lead = batch_tables::resolve_table(settings, import_batch_id, "ads_migration_lead_user")?;
    let final_lead = batch_tables::resolve_table(settings, import_batch_id, "ads_final_marketing_lead_user")?;
    let raw_crm = batch_tables::resolve_table(settings, import_batch_id, "raw_crm_user_import")?;
    let raw_reach = batch_tables::resolve_table(settings, import_batch_id, "raw_reachability_import")?;
    let raw_coverage = batch_tables::resolve_table(settings, import_batch_id, "raw_ftth_coverage_import")?;
    let sql_template = format!(
        "DELETE FROM {final_lead} WHERE analysis_run_id = :analysis_run_id;\n\n\
INSERT INTO {final_lead} (analysis_run_id, user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer)\n\
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id),\n\
base AS (SELECT * FROM {ads_lead} WHERE analysis_run_id = (SELECT analysis_run_id FROM params) AND user_key IS NOT NULL AND TRIM(user_key) <> '' AND user_key <> 'UNKNOWN'),\n\
crm AS (SELECT * FROM {raw_crm} WHERE import_batch_id = (SELECT import_batch_id FROM params)),\n\
reach AS (SELECT * FROM {raw_reach} WHERE import_batch_id = (SELECT import_batch_id FROM params)),\n\
coverage AS (SELECT * FROM {raw_coverage} WHERE import_batch_id = (SELECT import_batch_id FROM params))\n\
SELECT p.analysis_run_id, b.user_key, MAX(c.crm_user_id), b.lead_type, b.demand_score, b.migration_motive_score, MAX(c.current_plan_name), MAX(c.current_arpu),\n\
COALESCE(MAX(cv.ftth_available_flag), 'UNKNOWN') AS ftth_available_flag,\n\
CASE WHEN MAX(r.phone_available_flag) = 'Y' OR MAX(r.sms_available_flag) = 'Y' OR MAX(r.app_push_available_flag) = 'Y' THEN 'Y' ELSE 'N' END AS reachable_flag,\n\
CASE\n\
  WHEN MAX(c.crm_user_id) IS NULL THEN 'IDENTITY_MAPPING_REQUIRED'\n\
  WHEN MAX(UPPER(COALESCE(c.blacklist_flag, 'N'))) = 'Y' THEN 'EXCLUDE_BLACKLIST'\n\
  WHEN MAX(UPPER(COALESCE(c.arrears_flag, 'N'))) = 'Y' THEN 'ARREARS_CHECK_FIRST'\n\
  WHEN MAX(UPPER(COALESCE(c.contract_status, ''))) IN ('LOCKED', 'IN_CONTRACT', 'CONTRACT_LOCK') THEN 'CONTRACT_CHECK_FIRST'\n\
  WHEN b.lead_type LIKE 'A2_%' THEN 'NETWORK_OPTIMIZATION_FIRST'\n\
  WHEN b.lead_type LIKE 'A1_%' AND COALESCE(MAX(cv.ftth_available_flag), 'UNKNOWN') = 'Y' AND (MAX(r.phone_available_flag) = 'Y' OR MAX(r.sms_available_flag) = 'Y' OR MAX(r.app_push_available_flag) = 'Y') THEN 'MARKET_FIBER_UPSELL'\n\
  WHEN b.lead_type LIKE 'A1_%' AND COALESCE(MAX(cv.ftth_available_flag), 'UNKNOWN') = 'Y' THEN 'REACHABILITY_FIX_FIRST'\n\
  WHEN b.lead_type LIKE 'A1_%' THEN 'BUILD_OR_COVERAGE_CHECK'\n\
  WHEN b.lead_type LIKE 'B_%' THEN 'NURTURE_POOL'\n\
  WHEN b.lead_type LIKE 'C_%' THEN 'FTTH_SPEED_UPSELL'\n\
  ELSE 'OBSERVE'\n\
END AS final_action, b.recommended_offer\n\
FROM base b\n\
JOIN params p\n\
LEFT JOIN crm c ON {crm_on}\n\
LEFT JOIN reach r ON {reach_on}\n\
LEFT JOIN coverage cv ON {coverage_on}\n\
GROUP BY p.analysis_run_id, b.user_key, b.lead_type, b.demand_score, b.migration_motive_score, b.recommended_offer;"
    );
    let sql = sql_runner::bind_batch_params(&sql_template, import_batch_id, Some(analysis_run_id));
    Ok(JobStep {
        step_name: "final_marketing_lead_fusion_configured",
        source_table: Box::leak(format!("{ads_lead},{raw_crm},{raw_coverage},{raw_reach}").into_boxed_str()),
        target_table: Box::leak(final_lead.into_boxed_str()),
        sql_template: "configured_final_fusion",
        sql,
    })
}

fn join_condition(settings: &MySqlSettings, scope: &str, fallback: &str) -> Result<String, String> {
    let mut conn = db::conn(settings)?;
    let rules: Vec<JoinRule> = conn.exec_map(
        "SELECT left_alias, left_column, right_alias, right_column FROM cfg_final_join_rule WHERE rule_scope=? AND active_flag=1 ORDER BY priority, id",
        (scope,),
        |(left_alias, left_column, right_alias, right_column)| JoinRule { left_alias, left_column, right_alias, right_column },
    ).map_err(|err| format!("failed to query final join rules for {scope}: {err}"))?;
    let allowed = allowed_columns();
    let mut parts = Vec::new();
    for rule in rules {
        if is_allowed(&allowed, &rule.left_alias, &rule.left_column) && is_allowed(&allowed, &rule.right_alias, &rule.right_column) {
            parts.push(format!("{}.{} = {}.{}", rule.left_alias, rule.left_column, rule.right_alias, rule.right_column));
        }
    }
    if parts.is_empty() { Ok(fallback.to_string()) } else { Ok(parts.join(" OR ")) }
}

fn is_allowed(allowed: &HashMap<&'static str, HashSet<&'static str>>, alias: &str, column: &str) -> bool {
    allowed.get(alias).map(|columns| columns.contains(column)).unwrap_or(false)
}

fn allowed_columns() -> HashMap<&'static str, HashSet<&'static str>> {
    let mut map = HashMap::new();
    map.insert("b", ["user_key"].into_iter().collect());
    map.insert("c", ["crm_user_id", "user_account", "user_mac"].into_iter().collect());
    map.insert("r", ["crm_user_id", "user_account"].into_iter().collect());
    map.insert("cv", ["area_key"].into_iter().collect());
    map
}

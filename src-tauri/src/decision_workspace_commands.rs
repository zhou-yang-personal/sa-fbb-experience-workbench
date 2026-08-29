use std::io::Write;

use mysql::prelude::*;
use mysql::{params, Params, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::batch_tables;
use crate::db;
use crate::models::{
    ack, CommandAck, DashboardRequest, EtlRequest, MetricCard, OpportunityExportRequest,
};

#[derive(Clone, Debug, Serialize)]
pub struct OpportunityCandidateRow {
    pub user_key: String,
    pub opportunity_type: String,
    pub opportunity_level: String,
    pub user_type: String,
    pub active_days: i64,
    pub observation_rows: i64,
    pub total_download_gb: f64,
    pub total_effective_duration_hours: f64,
    pub avg_effective_download_mbps: Option<f64>,
    pub avg_wifi_delay_ms: Option<f64>,
    pub avg_subscriber_rtt_ms: Option<f64>,
    pub avg_network_rtt_ms: Option<f64>,
    pub avg_user_loss_pct: Option<f64>,
    pub avg_network_loss_pct: Option<f64>,
    pub primary_app: Option<String>,
    pub primary_app_active_days: i64,
    pub primary_app_observations: i64,
    pub evidence_value: Option<f64>,
    pub evidence_unit: Option<String>,
    pub evidence_summary: String,
    pub data_limitation_code: Option<String>,
    pub rule_profile_version: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct OpportunityCandidatePage {
    pub rows: Vec<OpportunityCandidateRow>,
    pub total: i64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionRuleProfileRow {
    pub rule_profile_id: String,
    pub version: i64,
    pub profile_name: String,
    pub status: String,
    pub minimum_user_observations: i64,
    pub minimum_app_users: i64,
    pub minimum_app_observations: i64,
    pub persistent_poor_rate_pct: f64,
    pub problem_app_poor_rate_pct: f64,
    pub problem_app_persistent_user_rate_pct: f64,
    pub heavy_traffic_gb: f64,
    pub heavy_usage_hours: f64,
    pub peak_hour_start: i64,
    pub peak_hour_end: i64,
    pub migration_min_traffic_gb: f64,
    pub speed_upgrade_min_traffic_gb: f64,
    pub speed_upgrade_max_effective_mbps: f64,
    pub mesh_min_wifi_delay_ms: f64,
    pub app_bundle_min_observations: i64,
    pub opportunity_min_active_days: i64,
    pub opportunity_min_observations: i64,
    pub speed_upgrade_min_conditions: i64,
    pub app_bundle_min_active_days: i64,
    pub sufficient_app_users: i64,
    pub sufficient_app_observations: i64,
    pub attention_app_poor_rate_pct: f64,
    pub attention_app_persistent_user_rate_pct: f64,
    pub severe_app_poor_rate_pct: f64,
    pub severe_app_persistent_user_rate_pct: f64,
    pub severe_app_severe_user_rate_pct: f64,
    pub mesh_min_coverage_pct: f64,
    pub mesh_min_rtt_delta_ms: f64,
    pub mesh_min_loss_delta_pct: f64,
    pub distribution_thresholds: serde_json::Value,
    pub notes: Option<String>,
    pub updated_at: String,
}

fn decode_rule(row: mysql::Row) -> Result<DecisionRuleProfileRow, String> {
    Ok(DecisionRuleProfileRow {
        rule_profile_id: row.get(0).unwrap_or_default(),
        version: row.get(1).unwrap_or_default(),
        profile_name: row.get(2).unwrap_or_default(),
        status: row.get(3).unwrap_or_default(),
        minimum_user_observations: row.get(4).unwrap_or_default(),
        minimum_app_users: row.get(5).unwrap_or_default(),
        minimum_app_observations: row.get(6).unwrap_or_default(),
        persistent_poor_rate_pct: row.get(7).unwrap_or_default(),
        problem_app_poor_rate_pct: row.get(8).unwrap_or_default(),
        problem_app_persistent_user_rate_pct: row.get(9).unwrap_or_default(),
        heavy_traffic_gb: row.get(10).unwrap_or_default(),
        heavy_usage_hours: row.get(11).unwrap_or_default(),
        peak_hour_start: row.get(12).unwrap_or_default(),
        peak_hour_end: row.get(13).unwrap_or_default(),
        migration_min_traffic_gb: row.get(14).unwrap_or_default(),
        speed_upgrade_min_traffic_gb: row.get(15).unwrap_or_default(),
        speed_upgrade_max_effective_mbps: row.get(16).unwrap_or_default(),
        mesh_min_wifi_delay_ms: row.get(17).unwrap_or_default(),
        app_bundle_min_observations: row.get(18).unwrap_or_default(),
        opportunity_min_active_days: row.get(19).unwrap_or_default(),
        opportunity_min_observations: row.get(20).unwrap_or_default(),
        speed_upgrade_min_conditions: row.get(21).unwrap_or_default(),
        app_bundle_min_active_days: row.get(22).unwrap_or_default(),
        sufficient_app_users: row.get(23).unwrap_or_default(),
        sufficient_app_observations: row.get(24).unwrap_or_default(),
        attention_app_poor_rate_pct: row.get(25).unwrap_or_default(),
        attention_app_persistent_user_rate_pct: row.get(26).unwrap_or_default(),
        severe_app_poor_rate_pct: row.get(27).unwrap_or_default(),
        severe_app_persistent_user_rate_pct: row.get(28).unwrap_or_default(),
        severe_app_severe_user_rate_pct: row.get(29).unwrap_or_default(),
        mesh_min_coverage_pct: row.get(30).unwrap_or_default(),
        mesh_min_rtt_delta_ms: row.get(31).unwrap_or_default(),
        mesh_min_loss_delta_pct: row.get(32).unwrap_or_default(),
        distribution_thresholds: row.get::<Option<String>, _>(33).flatten().and_then(|value| serde_json::from_str(&value).ok()).unwrap_or_else(default_distribution_thresholds),
        notes: row.get(34),
        updated_at: row.get(35).unwrap_or_default(),
    })
}

fn default_distribution_thresholds() -> serde_json::Value {
    serde_json::json!({"traffic_daily_gb":[1,5,15],"duration_daily_hours":[1,3,6],"peak_daily_hours":[0.5,1.5,3],"observations_daily":[5,20,50],"rate_mbps":[1,5,20,50],"rtt_ms":[30,60,100,200],"loss_pct":[0.5,1,3]})
}

#[derive(Clone)]
struct DistributionThresholds {
    traffic: Vec<f64>, duration: Vec<f64>, peak: Vec<f64>, observations: Vec<f64>,
    rate: Vec<f64>, rtt: Vec<f64>, loss: Vec<f64>,
}

fn distribution_thresholds_for_run(conn: &mut mysql::PooledConn, run_id: &str) -> Result<DistributionThresholds, String> {
    let json: Option<String> = conn.exec_first("SELECT CAST(COALESCE(p.distribution_thresholds,JSON_OBJECT('traffic_daily_gb',JSON_ARRAY(1,5,15),'duration_daily_hours',JSON_ARRAY(1,3,6),'peak_daily_hours',JSON_ARRAY(0.5,1.5,3),'observations_daily',JSON_ARRAY(5,20,50),'rate_mbps',JSON_ARRAY(1,5,20,50),'rtt_ms',JSON_ARRAY(30,60,100,200),'loss_pct',JSON_ARRAY(0.5,1,3))) AS CHAR) FROM meta_analysis_run_decision_binding b JOIN meta_decision_rule_profile p ON p.rule_profile_id=b.rule_profile_id AND p.version=b.rule_profile_version WHERE b.analysis_run_id=?", (run_id,)).map_err(|err| format!("failed to load distribution thresholds: {err}"))?;
    let value=json.and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok()).unwrap_or_else(default_distribution_thresholds);
    let array=|key:&str,defaults:&[f64]| value.get(key).and_then(|item| item.as_array()).map(|items| items.iter().filter_map(|item| item.as_f64()).collect::<Vec<_>>()).filter(|items| items.len()==defaults.len()).unwrap_or_else(|| defaults.to_vec());
    Ok(DistributionThresholds { traffic:array("traffic_daily_gb",&[1.,5.,15.]),duration:array("duration_daily_hours",&[1.,3.,6.]),peak:array("peak_daily_hours",&[0.5,1.5,3.]),observations:array("observations_daily",&[5.,20.,50.]),rate:array("rate_mbps",&[1.,5.,20.,50.]),rtt:array("rtt_ms",&[30.,60.,100.,200.]),loss:array("loss_pct",&[0.5,1.,3.]) })
}

const RULE_SELECT: &str = "SELECT rule_profile_id,version,profile_name,status,minimum_user_observations,minimum_app_users,minimum_app_observations,CAST(persistent_poor_rate_pct AS DOUBLE),CAST(problem_app_poor_rate_pct AS DOUBLE),CAST(problem_app_persistent_user_rate_pct AS DOUBLE),CAST(heavy_traffic_gb AS DOUBLE),CAST(heavy_usage_hours AS DOUBLE),CAST(peak_hour_start AS SIGNED),CAST(peak_hour_end AS SIGNED),CAST(migration_min_traffic_gb AS DOUBLE),CAST(speed_upgrade_min_traffic_gb AS DOUBLE),CAST(speed_upgrade_max_effective_mbps AS DOUBLE),CAST(mesh_min_wifi_delay_ms AS DOUBLE),CAST(app_bundle_min_observations AS SIGNED),CAST(opportunity_min_active_days AS SIGNED),CAST(opportunity_min_observations AS SIGNED),CAST(speed_upgrade_min_conditions AS SIGNED),CAST(app_bundle_min_active_days AS SIGNED),CAST(sufficient_app_users AS SIGNED),CAST(sufficient_app_observations AS SIGNED),CAST(attention_app_poor_rate_pct AS DOUBLE),CAST(attention_app_persistent_user_rate_pct AS DOUBLE),CAST(severe_app_poor_rate_pct AS DOUBLE),CAST(severe_app_persistent_user_rate_pct AS DOUBLE),CAST(severe_app_severe_user_rate_pct AS DOUBLE),CAST(mesh_min_coverage_pct AS DOUBLE),CAST(mesh_min_rtt_delta_ms AS DOUBLE),CAST(mesh_min_loss_delta_pct AS DOUBLE),CAST(COALESCE(distribution_thresholds,JSON_OBJECT('traffic_daily_gb',JSON_ARRAY(1,5,15),'duration_daily_hours',JSON_ARRAY(1,3,6),'peak_daily_hours',JSON_ARRAY(0.5,1.5,3),'observations_daily',JSON_ARRAY(5,20,50),'rate_mbps',JSON_ARRAY(1,5,20,50),'rtt_ms',JSON_ARRAY(30,60,100,200),'loss_pct',JSON_ARRAY(0.5,1,3))) AS CHAR),notes,DATE_FORMAT(updated_at,'%Y-%m-%d %H:%i:%s') FROM meta_decision_rule_profile";

#[tauri::command]
pub fn decision_rule_list(settings: crate::models::MySqlSettings) -> Result<Vec<DecisionRuleProfileRow>, String> {
    let mut conn = db::conn(&settings)?;
    let rows = conn.query_iter(format!("{RULE_SELECT} ORDER BY version DESC"))
        .map_err(|err| format!("failed to list decision rules: {err}"))?
        .map(|row| decode_rule(row.map_err(|err| format!("failed to decode decision rule: {err}"))?))
        .collect();
    rows
}

#[tauri::command]
pub fn decision_rule_create_draft(settings: crate::models::MySqlSettings) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&settings)?;
    let mut conn = db::conn(&settings)?;
    let existing: Option<String> = conn.exec_first("SELECT rule_profile_id FROM meta_decision_rule_profile WHERE status='draft' ORDER BY version DESC LIMIT 1", ()).map_err(|err| format!("failed to inspect decision draft: {err}"))?;
    if let Some(id) = existing { return Ok(ack(format!("existing decision rule draft reused: {id}"))); }
    let id = format!("DECISION_RULE_{}", Uuid::new_v4().simple());
    conn.exec_drop(format!("INSERT INTO meta_decision_rule_profile (rule_profile_id,version,profile_name,status,minimum_user_observations,minimum_app_users,minimum_app_observations,persistent_poor_rate_pct,problem_app_poor_rate_pct,problem_app_persistent_user_rate_pct,heavy_traffic_gb,heavy_usage_hours,peak_hour_start,peak_hour_end,migration_min_traffic_gb,speed_upgrade_min_traffic_gb,speed_upgrade_max_effective_mbps,mesh_min_wifi_delay_ms,app_bundle_min_observations,opportunity_min_active_days,opportunity_min_observations,speed_upgrade_min_conditions,app_bundle_min_active_days,sufficient_app_users,sufficient_app_observations,attention_app_poor_rate_pct,attention_app_persistent_user_rate_pct,severe_app_poor_rate_pct,severe_app_persistent_user_rate_pct,severe_app_severe_user_rate_pct,mesh_min_coverage_pct,mesh_min_rtt_delta_ms,mesh_min_loss_delta_pct,distribution_thresholds,rule_snapshot,notes) SELECT ?,COALESCE((SELECT MAX(version)+1 FROM (SELECT version FROM meta_decision_rule_profile) versions),1),CONCAT(profile_name,' copy'),'draft',minimum_user_observations,minimum_app_users,minimum_app_observations,persistent_poor_rate_pct,problem_app_poor_rate_pct,problem_app_persistent_user_rate_pct,heavy_traffic_gb,heavy_usage_hours,peak_hour_start,peak_hour_end,migration_min_traffic_gb,speed_upgrade_min_traffic_gb,speed_upgrade_max_effective_mbps,mesh_min_wifi_delay_ms,app_bundle_min_observations,opportunity_min_active_days,opportunity_min_observations,speed_upgrade_min_conditions,app_bundle_min_active_days,sufficient_app_users,sufficient_app_observations,attention_app_poor_rate_pct,attention_app_persistent_user_rate_pct,severe_app_poor_rate_pct,severe_app_persistent_user_rate_pct,severe_app_severe_user_rate_pct,mesh_min_coverage_pct,mesh_min_rtt_delta_ms,mesh_min_loss_delta_pct,COALESCE(distribution_thresholds,JSON_OBJECT('traffic_daily_gb',JSON_ARRAY(1,5,15),'duration_daily_hours',JSON_ARRAY(1,3,6),'peak_daily_hours',JSON_ARRAY(0.5,1.5,3),'observations_daily',JSON_ARRAY(5,20,50),'rate_mbps',JSON_ARRAY(1,5,20,50),'rtt_ms',JSON_ARRAY(30,60,100,200),'loss_pct',JSON_ARRAY(0.5,1,3))),rule_snapshot,'Draft cloned from latest published profile' FROM meta_decision_rule_profile WHERE status='published' ORDER BY version DESC LIMIT 1"), (&id,)).map_err(|err| format!("failed to create decision rule draft: {err}"))?;
    Ok(ack(format!("decision rule draft created: {id}")))
}

#[tauri::command]
pub fn decision_rule_update(settings: crate::models::MySqlSettings, rule: DecisionRuleProfileRow) -> Result<CommandAck, String> {
    if rule.status != "draft" { return Err("only a draft decision rule can be edited".to_string()); }
    if rule.minimum_user_observations < 1 || rule.minimum_app_users < 1 || rule.minimum_app_observations < 1 || rule.sufficient_app_users < rule.minimum_app_users || rule.sufficient_app_observations < rule.minimum_app_observations || rule.opportunity_min_active_days < 1 || rule.opportunity_min_observations < 1 || rule.speed_upgrade_min_conditions < 1 || rule.app_bundle_min_active_days < 1 { return Err("sample thresholds must be positive and sufficient must be at least limited".to_string()); }
    let rates = [rule.persistent_poor_rate_pct, rule.attention_app_poor_rate_pct, rule.attention_app_persistent_user_rate_pct, rule.problem_app_poor_rate_pct, rule.problem_app_persistent_user_rate_pct, rule.severe_app_poor_rate_pct, rule.severe_app_persistent_user_rate_pct, rule.severe_app_severe_user_rate_pct, rule.mesh_min_coverage_pct];
    if rates.iter().any(|value| !(0.0..=100.0).contains(value)) { return Err("rate thresholds must be between 0 and 100".to_string()); }
    if rule.attention_app_poor_rate_pct > rule.problem_app_poor_rate_pct
        || rule.problem_app_poor_rate_pct > rule.severe_app_poor_rate_pct
        || rule.attention_app_persistent_user_rate_pct
            > rule.problem_app_persistent_user_rate_pct
        || rule.problem_app_persistent_user_rate_pct
            > rule.severe_app_persistent_user_rate_pct
    {
        return Err("App thresholds must increase from attention to problem to severe".to_string());
    }
    if !(0..=23).contains(&rule.peak_hour_start) || !(0..=23).contains(&rule.peak_hour_end) { return Err("peak hours must be between 0 and 23".to_string()); }
    if rule.mesh_min_rtt_delta_ms < 0.0 || rule.mesh_min_loss_delta_pct < 0.0 { return Err("mesh evidence deltas cannot be negative".to_string()); }
    for key in ["traffic_daily_gb","duration_daily_hours","peak_daily_hours","observations_daily","rate_mbps","rtt_ms","loss_pct"] {
        let values=rule.distribution_thresholds.get(key).and_then(|value| value.as_array()).ok_or_else(|| format!("distribution threshold {key} must be an array"))?;
        if values.is_empty() || values.iter().any(|value| value.as_f64().is_none_or(|number| number < 0.0)) || values.windows(2).any(|pair| pair[0].as_f64().unwrap_or_default() >= pair[1].as_f64().unwrap_or_default()) { return Err(format!("distribution threshold {key} must contain increasing non-negative numbers")); }
    }
    let distribution_thresholds=serde_json::to_string(&rule.distribution_thresholds).map_err(|err| format!("failed to encode distribution thresholds: {err}"))?;
    let mut conn = db::conn(&settings)?;
    conn.exec_drop("UPDATE meta_decision_rule_profile SET profile_name=:name,minimum_user_observations=:min_user_obs,minimum_app_users=:min_app_users,minimum_app_observations=:min_app_obs,sufficient_app_users=:sufficient_users,sufficient_app_observations=:sufficient_obs,persistent_poor_rate_pct=:persistent_rate,attention_app_poor_rate_pct=:attention_poor,attention_app_persistent_user_rate_pct=:attention_persistent,problem_app_poor_rate_pct=:app_poor_rate,problem_app_persistent_user_rate_pct=:app_persistent_rate,severe_app_poor_rate_pct=:severe_poor,severe_app_persistent_user_rate_pct=:severe_persistent,severe_app_severe_user_rate_pct=:severe_user,heavy_traffic_gb=:heavy_traffic,heavy_usage_hours=:heavy_hours,peak_hour_start=:peak_start,peak_hour_end=:peak_end,migration_min_traffic_gb=:migration_traffic,speed_upgrade_min_traffic_gb=:upgrade_traffic,speed_upgrade_max_effective_mbps=:upgrade_rate,mesh_min_wifi_delay_ms=:mesh_delay,mesh_min_coverage_pct=:mesh_coverage,mesh_min_rtt_delta_ms=:mesh_rtt_delta,mesh_min_loss_delta_pct=:mesh_loss_delta,app_bundle_min_observations=:bundle_obs,opportunity_min_active_days=:opportunity_days,opportunity_min_observations=:opportunity_obs,speed_upgrade_min_conditions=:upgrade_conditions,app_bundle_min_active_days=:bundle_days,distribution_thresholds=:distribution_thresholds,notes=:notes,rule_snapshot=JSON_OBJECT('minimum_user_observations',:min_user_obs,'minimum_app_users',:min_app_users,'minimum_app_observations',:min_app_obs,'sufficient_app_users',:sufficient_users,'sufficient_app_observations',:sufficient_obs,'persistent_poor_rate_pct',:persistent_rate,'attention_app_poor_rate_pct',:attention_poor,'attention_app_persistent_user_rate_pct',:attention_persistent,'problem_app_poor_rate_pct',:app_poor_rate,'problem_app_persistent_user_rate_pct',:app_persistent_rate,'severe_app_poor_rate_pct',:severe_poor,'severe_app_persistent_user_rate_pct',:severe_persistent,'severe_app_severe_user_rate_pct',:severe_user,'heavy_traffic_gb',:heavy_traffic,'heavy_usage_hours',:heavy_hours,'peak_hour_start',:peak_start,'peak_hour_end',:peak_end,'migration_min_traffic_gb',:migration_traffic,'speed_upgrade_min_traffic_gb',:upgrade_traffic,'speed_upgrade_max_effective_mbps',:upgrade_rate,'mesh_min_wifi_delay_ms',:mesh_delay,'mesh_min_coverage_pct',:mesh_coverage,'mesh_min_rtt_delta_ms',:mesh_rtt_delta,'mesh_min_loss_delta_pct',:mesh_loss_delta,'app_bundle_min_observations',:bundle_obs,'opportunity_min_active_days',:opportunity_days,'opportunity_min_observations',:opportunity_obs,'speed_upgrade_min_conditions',:upgrade_conditions,'app_bundle_min_active_days',:bundle_days,'distribution_thresholds',JSON_EXTRACT(:distribution_thresholds,'$')) WHERE rule_profile_id=:id AND status='draft'", params! {
        "name" => &rule.profile_name,
        "min_user_obs" => rule.minimum_user_observations,
        "min_app_users" => rule.minimum_app_users,
        "min_app_obs" => rule.minimum_app_observations,
        "sufficient_users" => rule.sufficient_app_users,
        "sufficient_obs" => rule.sufficient_app_observations,
        "persistent_rate" => rule.persistent_poor_rate_pct,
        "attention_poor" => rule.attention_app_poor_rate_pct,
        "attention_persistent" => rule.attention_app_persistent_user_rate_pct,
        "app_poor_rate" => rule.problem_app_poor_rate_pct,
        "app_persistent_rate" => rule.problem_app_persistent_user_rate_pct,
        "severe_poor" => rule.severe_app_poor_rate_pct,
        "severe_persistent" => rule.severe_app_persistent_user_rate_pct,
        "severe_user" => rule.severe_app_severe_user_rate_pct,
        "heavy_traffic" => rule.heavy_traffic_gb,
        "heavy_hours" => rule.heavy_usage_hours,
        "peak_start" => rule.peak_hour_start,
        "peak_end" => rule.peak_hour_end,
        "migration_traffic" => rule.migration_min_traffic_gb,
        "upgrade_traffic" => rule.speed_upgrade_min_traffic_gb,
        "upgrade_rate" => rule.speed_upgrade_max_effective_mbps,
        "mesh_delay" => rule.mesh_min_wifi_delay_ms,
        "mesh_coverage" => rule.mesh_min_coverage_pct,
        "mesh_rtt_delta" => rule.mesh_min_rtt_delta_ms,
        "mesh_loss_delta" => rule.mesh_min_loss_delta_pct,
        "bundle_obs" => rule.app_bundle_min_observations,
        "opportunity_days" => rule.opportunity_min_active_days,
        "opportunity_obs" => rule.opportunity_min_observations,
        "upgrade_conditions" => rule.speed_upgrade_min_conditions,
        "bundle_days" => rule.app_bundle_min_active_days,
        "distribution_thresholds" => &distribution_thresholds,
        "notes" => &rule.notes,
        "id" => &rule.rule_profile_id,
    }).map_err(|err| format!("failed to update decision rule: {err}"))?;
    Ok(ack(format!("decision rule draft saved: {}", rule.rule_profile_id)))
}

#[tauri::command]
pub fn decision_rule_publish(settings: crate::models::MySqlSettings, rule_profile_id: String) -> Result<CommandAck, String> {
    let mut conn = db::conn(&settings)?;
    conn.exec_drop("UPDATE meta_decision_rule_profile SET status='published',published_at=UTC_TIMESTAMP() WHERE rule_profile_id=? AND status='draft'", (&rule_profile_id,)).map_err(|err| format!("failed to publish decision rule: {err}"))?;
    if conn.affected_rows() != 1 { return Err("decision rule draft not found or already published".to_string()); }
    Ok(ack(format!("decision rule published: {rule_profile_id}")))
}

fn row_string(row: &mysql::Row, index: usize, fallback: &str) -> String {
    row.get_opt::<Option<String>, _>(index)
        .and_then(Result::ok)
        .flatten()
        .unwrap_or_else(|| fallback.to_string())
}

fn row_i64(row: &mysql::Row, index: usize) -> i64 {
    row.get_opt::<Option<i64>, _>(index)
        .and_then(Result::ok)
        .flatten()
        .unwrap_or_default()
}

fn row_f64(row: &mysql::Row, index: usize) -> Option<f64> {
    row.get_opt::<Option<f64>, _>(index)
        .and_then(Result::ok)
        .flatten()
}

fn rate(numerator: i64, denominator: i64) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 * 100.0 / denominator as f64)
}

fn fmt_optional(value: Option<f64>, decimals: usize) -> String {
    value
        .map(|item| format!("{item:.decimals$}"))
        .unwrap_or_else(|| "NA".to_string())
}

fn source_table(req: &DashboardRequest) -> Result<String, String> {
    batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_app_period_experience_v2",
    )
}

fn ensure_run_source(req: &DashboardRequest, table: &str) -> Result<(), String> {
    let mut conn = db::conn(&req.settings)?;
    if batch_tables::table_has_analysis_run(&mut conn, table, &req.run_id())? {
        Ok(())
    } else {
        Err(format!(
            "decision workspace source is not ready: analysis_run_id={}, table={table}; rebuild DWS/ADS from the data center",
            req.run_id()
        ))
    }
}

fn require_rule_binding(
    conn: &mut mysql::PooledConn,
    analysis_run_id: &str,
) -> Result<(), String> {
    let binding_exists: Option<i64> = conn
        .exec_first(
            "SELECT 1 FROM meta_analysis_run_decision_binding WHERE analysis_run_id=? LIMIT 1",
            (analysis_run_id,),
        )
        .map_err(|err| format!("failed to verify decision rule binding: {err}"))?;
    binding_exists
        .map(|_| ())
        .ok_or_else(|| format!("decision rule binding is not ready for analysis_run_id={analysis_run_id}; generate analysis results from the data center"))
}

#[tauri::command]
pub fn decision_get_metric_panorama(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_metric_panorama", || {
        let table = source_table(&req)?;
        ensure_run_source(&req, &table)?;
        let mut conn = db::conn(&req.settings)?;
        let run_id = req.run_id();
        require_rule_binding(&mut conn, &run_id)?;
        let sql = format!(
            "WITH rules AS (SELECT p.minimum_user_observations,p.persistent_poor_rate_pct FROM meta_decision_rule_profile p JOIN meta_analysis_run_decision_binding b ON b.rule_profile_id=p.rule_profile_id AND b.rule_profile_version=p.version WHERE b.analysis_run_id=?), users AS (SELECT user_key,SUM(total_download_gb) traffic_gb,SUM(total_game_hours) game_hours,SUM(total_effective_duration_hours) effective_hours,SUM(observation_rows) observation_rows,SUM(valid_obs_rows) valid_obs_rows,SUM(poor_obs_rows) poor_obs_rows,MAX(CASE WHEN valid_obs_rows>=r.minimum_user_observations AND poor_obs_rows*100.0/NULLIF(valid_obs_rows,0)>=r.persistent_poor_rate_pct THEN 1 ELSE 0 END) persistent_flag,MAX(severe_poor_user_flag) severe_flag FROM `{table}` CROSS JOIN rules r WHERE analysis_run_id=? GROUP BY user_key,r.minimum_user_observations,r.persistent_poor_rate_pct), metrics AS (SELECT COUNT(*) users,COALESCE(SUM(traffic_gb),0) traffic_gb,COALESCE(SUM(game_hours),0) game_hours,COALESCE(SUM(effective_hours),0) effective_hours,COALESCE(SUM(observation_rows),0) observation_rows,COALESCE(SUM(valid_obs_rows),0) valid_obs_rows,COALESCE(SUM(poor_obs_rows),0) poor_obs_rows,COALESCE(SUM(persistent_flag),0) persistent_users,COALESCE(SUM(severe_flag),0) severe_users FROM users), quality AS (SELECT SUM(effective_download_mbps_sum)/NULLIF(SUM(effective_download_mbps_count),0) effective_mbps,SUM(subscriber_rtt_sum)/NULLIF(SUM(subscriber_rtt_count),0) subscriber_rtt,SUM(network_rtt_sum)/NULLIF(SUM(network_rtt_count),0) network_rtt,SUM(user_loss_sum)/NULLIF(SUM(user_loss_count),0) user_loss,SUM(network_loss_sum)/NULLIF(SUM(network_loss_count),0) network_loss,SUM(avg_download_mbps*valid_obs_rows)/NULLIF(SUM(CASE WHEN avg_download_mbps IS NOT NULL THEN valid_obs_rows ELSE 0 END),0) download_mbps,SUM(avg_throughput_mbps*valid_obs_rows)/NULLIF(SUM(CASE WHEN avg_throughput_mbps IS NOT NULL THEN valid_obs_rows ELSE 0 END),0) throughput_mbps,SUM(avg_connection_success_pct*valid_obs_rows)/NULLIF(SUM(CASE WHEN avg_connection_success_pct IS NOT NULL THEN valid_obs_rows ELSE 0 END),0) connection_success,SUM(avg_connection_delay_ms*valid_obs_rows)/NULLIF(SUM(CASE WHEN avg_connection_delay_ms IS NOT NULL THEN valid_obs_rows ELSE 0 END),0) connection_delay,SUM(avg_download_fluency*valid_obs_rows)/NULLIF(SUM(CASE WHEN avg_download_fluency IS NOT NULL THEN valid_obs_rows ELSE 0 END),0) download_fluency FROM `{table}` WHERE analysis_run_id=?) SELECT CAST(users AS SIGNED),CAST(traffic_gb AS DOUBLE),CAST(game_hours AS DOUBLE),CAST(observation_rows AS SIGNED),CAST(valid_obs_rows AS SIGNED),CAST(poor_obs_rows AS SIGNED),CAST(persistent_users AS SIGNED),CAST(severe_users AS SIGNED),CAST(effective_mbps AS DOUBLE),CAST(subscriber_rtt AS DOUBLE),CAST(network_rtt AS DOUBLE),CAST(user_loss AS DOUBLE),CAST(network_loss AS DOUBLE),CAST(effective_hours AS DOUBLE),CAST(download_mbps AS DOUBLE),CAST(throughput_mbps AS DOUBLE),CAST(connection_success AS DOUBLE),CAST(connection_delay AS DOUBLE),CAST(download_fluency AS DOUBLE) FROM metrics CROSS JOIN quality"
        );
        let row: mysql::Row = conn
            .exec_first(sql, (&run_id, &run_id, &run_id))
            .map_err(|err| format!("failed to query metric panorama: {err}"))?
            .ok_or_else(|| "metric panorama returned no aggregate row".to_string())?;
        let users = row_i64(&row, 0);
        let traffic = row_f64(&row, 1).unwrap_or_default();
        let game_hours = row_f64(&row, 2).unwrap_or_default();
        let obs = row_i64(&row, 3);
        let valid = row_i64(&row, 4);
        let poor = row_i64(&row, 5);
        let persistent = row_i64(&row, 6);
        let severe = row_i64(&row, 7);
        let values = [
            ("有效用户", users.to_string(), format!("metric=users, numerator={users}, denominator={users}, sample_size={users}, unit=users, availability=AVAILABLE")),
            ("总流量", format!("{traffic:.2}"), format!("metric=traffic, value={traffic:.6}, unit=GB, sample_size={users}, availability=AVAILABLE")),
            ("独立 Game 时长", format!("{game_hours:.2}"), format!("metric=game_duration, value={game_hours:.6}, unit=hours, sample_size={users}, availability=PARTIAL_GAME_ONLY, limitation=仅来自独立 Game 文件；未导入时不可解释为真实 0")),
            ("有效业务时长", fmt_optional(row_f64(&row, 13), 2), format!("metric=effective_duration, unit=hours, sample_size={users}, availability={}", if row_f64(&row, 13).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("有效观测", valid.to_string(), format!("metric=observations, numerator={valid}, denominator={obs}, sample_size={obs}, unit=rows, availability=AVAILABLE")),
            ("差体验观测占比", fmt_optional(rate(poor, valid), 2), format!("metric=poor_observation_rate, numerator={poor}, denominator={valid}, sample_size={valid}, unit=percent, availability={}", if valid > 0 { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("持续质差用户占比", fmt_optional(rate(persistent, users), 2), format!("metric=persistent_user_rate, numerator={persistent}, denominator={users}, sample_size={users}, unit=percent, availability={}", if users > 0 { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("严重质差用户占比", fmt_optional(rate(severe, users), 2), format!("metric=severe_user_rate, numerator={severe}, denominator={users}, sample_size={users}, unit=percent, availability={}", if users > 0 { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("视频有效下载速率", fmt_optional(row_f64(&row, 8), 2), format!("metric=effective_download_rate, unit=Mbps, sample_size={valid}, availability={}", if row_f64(&row, 8).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("平均下载速率", fmt_optional(row_f64(&row, 14), 2), format!("metric=average_download_rate, unit=Mbps, sample_size={valid}, availability={}", if row_f64(&row, 14).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("平均吞吐带宽", fmt_optional(row_f64(&row, 15), 2), format!("metric=throughput, unit=Mbps, sample_size={valid}, availability={}", if row_f64(&row, 15).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("建连成功率", fmt_optional(row_f64(&row, 16), 2), format!("metric=connection_success, unit=percent, sample_size={valid}, availability={}", if row_f64(&row, 16).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("建连时延", fmt_optional(row_f64(&row, 17), 2), format!("metric=connection_delay, unit=ms, sample_size={valid}, availability={}", if row_f64(&row, 17).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("下载流畅度", fmt_optional(row_f64(&row, 18), 2), format!("metric=download_fluency, unit=score, sample_size={valid}, availability={}, limitation=业务语义仍需真实数据校准", if row_f64(&row, 18).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("用户侧时延", fmt_optional(row_f64(&row, 9), 2), format!("metric=subscriber_rtt, unit=ms, sample_size={valid}, availability={}", if row_f64(&row, 9).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("网络侧时延", fmt_optional(row_f64(&row, 10), 2), format!("metric=network_rtt, unit=ms, sample_size={valid}, availability={}", if row_f64(&row, 10).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("用户侧丢包", fmt_optional(row_f64(&row, 11), 4), format!("metric=user_loss, unit=percent, sample_size={valid}, availability={}", if row_f64(&row, 11).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("网络侧丢包", fmt_optional(row_f64(&row, 12), 4), format!("metric=network_loss, unit=percent, sample_size={valid}, availability={}", if row_f64(&row, 12).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("上行速率", "—".to_string(), "metric=upstream_rate, unit=Mbps, availability=UNAVAILABLE, limitation=当前已导入源字段不包含可用于分析的上行速率".to_string()),
        ];
        Ok(values.into_iter().map(|(label, value, hint)| MetricCard { label: label.to_string(), value, hint }).collect())
    })
}

#[tauri::command]
pub fn decision_get_app_panorama(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_app_panorama", || {
        let table = source_table(&req)?;
        ensure_run_source(&req, &table)?;
        let mut conn = db::conn(&req.settings)?;
        let run_id = req.run_id();
        require_rule_binding(&mut conn, &run_id)?;
        let page_size = req.page_size(200, 500);
        let offset = req.offset(200, 500);
        let keyword = req.keyword_like();
        let sql = format!(
            "WITH rules AS (SELECT p.* FROM meta_decision_rule_profile p JOIN meta_analysis_run_decision_binding b ON b.rule_profile_id=p.rule_profile_id AND b.rule_profile_version=p.version WHERE b.analysis_run_id=?), apps AS (SELECT app_name,MAX(app_category) app_category,COUNT(DISTINCT user_key) observed_users,SUM(observation_rows) observation_rows,SUM(valid_obs_rows) valid_obs_rows,SUM(poor_obs_rows) poor_obs_rows,COUNT(DISTINCT CASE WHEN valid_obs_rows>=r.minimum_user_observations THEN user_key END) eligible_users,COUNT(DISTINCT CASE WHEN valid_obs_rows>=r.minimum_user_observations AND poor_obs_rows*100.0/NULLIF(valid_obs_rows,0)>=r.persistent_poor_rate_pct THEN user_key END) persistent_users,COUNT(DISTINCT CASE WHEN severe_poor_user_flag=1 THEN user_key END) severe_users,SUM(total_download_gb) traffic_gb,SUM(total_effective_duration_hours) duration_hours,SUM(effective_download_mbps_sum)/NULLIF(SUM(effective_download_mbps_count),0) effective_mbps,SUM(subscriber_rtt_sum)/NULLIF(SUM(subscriber_rtt_count),0) subscriber_rtt,SUM(network_rtt_sum)/NULLIF(SUM(network_rtt_count),0) network_rtt,SUM(user_loss_sum)/NULLIF(SUM(user_loss_count),0) user_loss,SUM(network_loss_sum)/NULLIF(SUM(network_loss_count),0) network_loss FROM `{table}` CROSS JOIN rules r WHERE analysis_run_id=? AND (? IS NULL OR app_name LIKE ? OR app_category LIKE ?) GROUP BY app_name,r.minimum_user_observations,r.persistent_poor_rate_pct) SELECT a.app_name,a.app_category,CAST(a.observed_users AS SIGNED),CAST(a.eligible_users AS SIGNED),CAST(a.observation_rows AS SIGNED),CAST(a.valid_obs_rows AS SIGNED),CAST(a.poor_obs_rows AS SIGNED),CAST(a.persistent_users AS SIGNED),CAST(a.severe_users AS SIGNED),CAST(a.traffic_gb AS DOUBLE),CAST(a.duration_hours AS DOUBLE),CAST(a.effective_mbps AS DOUBLE),CAST(a.subscriber_rtt AS DOUBLE),CAST(a.network_rtt AS DOUBLE),CAST(a.user_loss AS DOUBLE),CAST(a.network_loss AS DOUBLE),r.minimum_app_users,r.minimum_app_observations,r.sufficient_app_users,r.sufficient_app_observations,r.attention_app_poor_rate_pct,r.attention_app_persistent_user_rate_pct,r.problem_app_poor_rate_pct,r.problem_app_persistent_user_rate_pct,r.severe_app_poor_rate_pct,r.severe_app_persistent_user_rate_pct,r.severe_app_severe_user_rate_pct,r.version FROM apps a CROSS JOIN rules r ORDER BY a.observed_users DESC,a.traffic_gb DESC LIMIT ? OFFSET ?"
        );
        let rows = conn.exec_iter(sql, (&run_id, &run_id, keyword.clone(), keyword.clone(), keyword, page_size, offset))
            .map_err(|err| format!("failed to query App panorama: {err}"))?;
        rows.map(|row| {
            let row = row.map_err(|err| format!("failed to decode App panorama: {err}"))?;
            let app = row_string(&row, 0, "UNKNOWN_APP");
            let category = row_string(&row, 1, "other");
            let observed = row_i64(&row, 2);
            let eligible = row_i64(&row, 3);
            let observations = row_i64(&row, 4);
            let valid = row_i64(&row, 5);
            let poor = row_i64(&row, 6);
            let persistent = row_i64(&row, 7);
            let severe = row_i64(&row, 8);
            let min_users = row_i64(&row, 16);
            let min_obs = row_i64(&row, 17);
            let sufficient_users = row_i64(&row, 18);
            let sufficient_obs = row_i64(&row, 19);
            let attention_poor = row_f64(&row, 20).unwrap_or_default();
            let attention_persistent = row_f64(&row, 21).unwrap_or_default();
            let problem_poor = row_f64(&row, 22).unwrap_or_default();
            let problem_persistent = row_f64(&row, 23).unwrap_or_default();
            let severe_poor = row_f64(&row, 24).unwrap_or_default();
            let severe_persistent = row_f64(&row, 25).unwrap_or_default();
            let severe_user_threshold = row_f64(&row, 26).unwrap_or_default();
            let poor_rate = rate(poor, valid);
            let persistent_rate = rate(persistent, eligible);
            let severe_user_rate = rate(severe, eligible);
            let sample_status = if eligible < min_users || valid < min_obs { "INSUFFICIENT_SAMPLE" } else if eligible < sufficient_users || valid < sufficient_obs { "LIMITED_SAMPLE" } else { "SUFFICIENT" };
            let status = if sample_status == "INSUFFICIENT_SAMPLE" { "INSUFFICIENT" } else if sample_status == "LIMITED_SAMPLE" { "LIMITED" } else if poor_rate.unwrap_or_default() >= severe_poor || persistent_rate.unwrap_or_default() >= severe_persistent || severe_user_rate.unwrap_or_default() >= severe_user_threshold { "SEVERE" } else if poor_rate.unwrap_or_default() >= problem_poor || persistent_rate.unwrap_or_default() >= problem_persistent { "PROBLEM" } else if poor_rate.unwrap_or_default() >= attention_poor || persistent_rate.unwrap_or_default() >= attention_persistent { "WATCH" } else { "NORMAL" };
            Ok(MetricCard {
                label: app.clone(),
                value: observed.to_string(),
                hint: format!("source=dws_user_app_period_experience_v2, perspective=app, app_name={app}, app_category={category}, observed_users={observed}, eligible_users={eligible}, observation_rows={observations}, valid_obs_rows={valid}, poor_obs_rows={poor}, poor_observation_rate_pct={}, persistent_poor_users={persistent}, persistent_poor_user_rate_pct={}, severe_poor_users={severe}, severe_poor_user_rate_pct={}, traffic_gb={}, duration_hours={}, effective_download_mbps={}, subscriber_rtt_ms={}, network_rtt_ms={}, user_loss_pct={}, network_loss_pct={}, sample_status={sample_status}, insight_status={status}, minimum_app_users={min_users}, minimum_app_observations={min_obs}, sufficient_app_users={sufficient_users}, sufficient_app_observations={sufficient_obs}, attention_poor_rate_pct={attention_poor:.4}, attention_persistent_rate_pct={attention_persistent:.4}, problem_poor_rate_pct={problem_poor:.4}, problem_persistent_rate_pct={problem_persistent:.4}, severe_poor_rate_pct={severe_poor:.4}, severe_persistent_rate_pct={severe_persistent:.4}, severe_user_rate_pct={severe_user_threshold:.4}, rule_version={}", fmt_optional(poor_rate, 4), fmt_optional(persistent_rate, 4), fmt_optional(severe_user_rate, 4), fmt_optional(row_f64(&row, 9), 4), fmt_optional(row_f64(&row, 10), 4), fmt_optional(row_f64(&row, 11), 4), fmt_optional(row_f64(&row, 12), 4), fmt_optional(row_f64(&row, 13), 4), fmt_optional(row_f64(&row, 14), 6), fmt_optional(row_f64(&row, 15), 6), row_i64(&row, 27))
            })
        }).collect()
    })
}

#[tauri::command]
pub fn decision_get_user_distributions(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_user_distributions", || {
        let table = source_table(&req)?;
        let hourly_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_app_hourly_experience_v2")?;
        ensure_run_source(&req, &table)?;
        ensure_run_source(&req, &hourly_table)?;
        let mut conn = db::conn(&req.settings)?;
        let run_id = req.run_id();
        require_rule_binding(&mut conn, &run_id)?;
        let thresholds=distribution_thresholds_for_run(&mut conn,&run_id)?;
        let [traffic0,traffic1,traffic2]=thresholds.traffic[..] else { unreachable!() };
        let [duration0,duration1,duration2]=thresholds.duration[..] else { unreachable!() };
        let [peak0,peak1,peak2]=thresholds.peak[..] else { unreachable!() };
        let [obs0,obs1,obs2]=thresholds.observations[..] else { unreachable!() };
        let [rate0,rate1,rate2,rate3]=thresholds.rate[..] else { unreachable!() };
        let [rtt0,rtt1,rtt2,rtt3]=thresholds.rtt[..] else { unreachable!() };
        let [loss0,loss1,loss2]=thresholds.loss[..] else { unreachable!() };
        let keyword = req.keyword_like();
        let sql = format!(
            "WITH rules AS (SELECT p.minimum_user_observations,p.persistent_poor_rate_pct,p.peak_hour_start,p.peak_hour_end FROM meta_decision_rule_profile p JOIN meta_analysis_run_decision_binding b ON b.rule_profile_id=p.rule_profile_id AND b.rule_profile_version=p.version WHERE b.analysis_run_id=?), activity AS (SELECT h.user_key,COUNT(DISTINCT h.stat_date) active_days,SUM(CASE WHEN h.hour_of_day BETWEEN r.peak_hour_start AND r.peak_hour_end THEN h.total_effective_duration_hours ELSE 0 END) peak_hours FROM `{hourly_table}` h CROSS JOIN rules r WHERE h.analysis_run_id=? GROUP BY h.user_key,r.peak_hour_start,r.peak_hour_end), users AS (SELECT p.user_key,SUM(total_download_gb) traffic_gb,SUM(total_game_hours) game_hours,SUM(total_effective_duration_hours) effective_hours,MAX(a.peak_hours) peak_hours,MAX(a.active_days) active_days,SUM(observation_rows) observations,AVG(avg_effective_download_mbps) effective_mbps,AVG(avg_download_mbps) download_mbps,AVG(avg_subscriber_rtt_ms) subscriber_rtt,AVG(avg_network_rtt_ms) network_rtt,AVG(avg_user_loss_pct) user_loss,AVG(avg_network_loss_pct) network_loss,MAX(CASE WHEN valid_obs_rows>=r.minimum_user_observations AND poor_obs_rows*100.0/NULLIF(valid_obs_rows,0)>=r.persistent_poor_rate_pct THEN 1 ELSE 0 END) persistent_flag FROM `{table}` p CROSS JOIN rules r LEFT JOIN activity a ON a.user_key=p.user_key WHERE analysis_run_id=? AND (? IS NULL OR app_name LIKE ?) GROUP BY p.user_key,r.minimum_user_observations,r.persistent_poor_rate_pct), bands AS (SELECT 'TRAFFIC_DAILY' dimension,CASE WHEN traffic_gb/NULLIF(active_days,0)<1 THEN '<1 GB/day' WHEN traffic_gb/NULLIF(active_days,0)<5 THEN '1–5 GB/day' WHEN traffic_gb/NULLIF(active_days,0)<15 THEN '5–15 GB/day' ELSE '≥15 GB/day' END band,COUNT(*) users FROM users GROUP BY 2 UNION ALL SELECT 'DURATION_EFFECTIVE_DAILY',CASE WHEN effective_hours/NULLIF(active_days,0)<1 THEN '<1 h/day' WHEN effective_hours/NULLIF(active_days,0)<3 THEN '1–3 h/day' WHEN effective_hours/NULLIF(active_days,0)<6 THEN '3–6 h/day' ELSE '≥6 h/day' END,COUNT(*) FROM users WHERE active_days>0 GROUP BY 2 UNION ALL SELECT 'DURATION_PEAK_DAILY',CASE WHEN peak_hours/NULLIF(active_days,0)<0.5 THEN '<0.5 h/day' WHEN peak_hours/NULLIF(active_days,0)<1.5 THEN '0.5–1.5 h/day' WHEN peak_hours/NULLIF(active_days,0)<3 THEN '1.5–3 h/day' ELSE '≥3 h/day' END,COUNT(*) FROM users WHERE active_days>0 GROUP BY 2 UNION ALL SELECT 'DURATION_GAME_ONLY',CASE WHEN game_hours=0 THEN '0 h / 未导入或无游戏时长' WHEN game_hours<1 THEN '<1 h' WHEN game_hours<5 THEN '1–5 h' WHEN game_hours<10 THEN '5–10 h' ELSE '≥10 h' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'OBSERVATIONS_DAILY',CASE WHEN observations=0 THEN '0 条/day' WHEN observations/NULLIF(active_days,0)<=5 THEN '1–5 条/day' WHEN observations/NULLIF(active_days,0)<=20 THEN '6–20 条/day' WHEN observations/NULLIF(active_days,0)<=50 THEN '21–50 条/day' ELSE '>50 条/day' END,COUNT(*) FROM users WHERE active_days>0 GROUP BY 2 UNION ALL SELECT 'EFFECTIVE_RATE',CASE WHEN effective_mbps IS NULL THEN '不可用' WHEN effective_mbps<1 THEN '<1 Mbps' WHEN effective_mbps<5 THEN '1–5 Mbps' WHEN effective_mbps<20 THEN '5–20 Mbps' WHEN effective_mbps<50 THEN '20–50 Mbps' ELSE '≥50 Mbps' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'AVERAGE_DOWNLOAD_RATE',CASE WHEN download_mbps IS NULL THEN '不可用' WHEN download_mbps<1 THEN '<1 Mbps' WHEN download_mbps<5 THEN '1–5 Mbps' WHEN download_mbps<20 THEN '5–20 Mbps' WHEN download_mbps<50 THEN '20–50 Mbps' ELSE '≥50 Mbps' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'SUBSCRIBER_RTT',CASE WHEN subscriber_rtt IS NULL THEN '不可用' WHEN subscriber_rtt<30 THEN '<30 ms' WHEN subscriber_rtt<60 THEN '30–60 ms' WHEN subscriber_rtt<100 THEN '60–100 ms' WHEN subscriber_rtt<200 THEN '100–200 ms' ELSE '≥200 ms' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'NETWORK_RTT',CASE WHEN network_rtt IS NULL THEN '不可用' WHEN network_rtt<30 THEN '<30 ms' WHEN network_rtt<60 THEN '30–60 ms' WHEN network_rtt<100 THEN '60–100 ms' WHEN network_rtt<200 THEN '100–200 ms' ELSE '≥200 ms' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'USER_LOSS',CASE WHEN user_loss IS NULL THEN '不可用' WHEN user_loss=0 THEN '0%' WHEN user_loss<0.5 THEN '0–0.5%' WHEN user_loss<1 THEN '0.5–1%' WHEN user_loss<3 THEN '1–3%' ELSE '≥3%' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'NETWORK_LOSS',CASE WHEN network_loss IS NULL THEN '不可用' WHEN network_loss=0 THEN '0%' WHEN network_loss<0.5 THEN '0–0.5%' WHEN network_loss<1 THEN '0.5–1%' WHEN network_loss<3 THEN '1–3%' ELSE '≥3%' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'EXPERIENCE',CASE WHEN persistent_flag=1 THEN '持续质差用户' ELSE '非持续质差用户' END,COUNT(*) FROM users GROUP BY 2) SELECT dimension,band,CAST(users AS SIGNED) FROM bands ORDER BY dimension,users DESC"
        );
        let sql=sql
          .replace("AVG(avg_effective_download_mbps) effective_mbps,AVG(avg_download_mbps) download_mbps,AVG(avg_subscriber_rtt_ms) subscriber_rtt,AVG(avg_network_rtt_ms) network_rtt,AVG(avg_user_loss_pct) user_loss,AVG(avg_network_loss_pct) network_loss", "SUM(effective_download_mbps_sum)/NULLIF(SUM(effective_download_mbps_count),0) effective_mbps,SUM(avg_download_mbps*valid_obs_rows)/NULLIF(SUM(CASE WHEN avg_download_mbps IS NOT NULL THEN valid_obs_rows ELSE 0 END),0) download_mbps,SUM(subscriber_rtt_sum)/NULLIF(SUM(subscriber_rtt_count),0) subscriber_rtt,SUM(network_rtt_sum)/NULLIF(SUM(network_rtt_count),0) network_rtt,SUM(user_loss_sum)/NULLIF(SUM(user_loss_count),0) user_loss,SUM(network_loss_sum)/NULLIF(SUM(network_loss_count),0) network_loss")
          .replace("CASE WHEN traffic_gb/NULLIF(active_days,0)<1 THEN '<1 GB/day' WHEN traffic_gb/NULLIF(active_days,0)<5 THEN '1–5 GB/day' WHEN traffic_gb/NULLIF(active_days,0)<15 THEN '5–15 GB/day' ELSE '≥15 GB/day' END",&format!("CASE WHEN traffic_gb/NULLIF(active_days,0)<{traffic0} THEN '<{traffic0} GB/day' WHEN traffic_gb/NULLIF(active_days,0)<{traffic1} THEN '{traffic0}–{traffic1} GB/day' WHEN traffic_gb/NULLIF(active_days,0)<{traffic2} THEN '{traffic1}–{traffic2} GB/day' ELSE '≥{traffic2} GB/day' END"))
          .replace("CASE WHEN effective_hours/NULLIF(active_days,0)<1 THEN '<1 h/day' WHEN effective_hours/NULLIF(active_days,0)<3 THEN '1–3 h/day' WHEN effective_hours/NULLIF(active_days,0)<6 THEN '3–6 h/day' ELSE '≥6 h/day' END",&format!("CASE WHEN effective_hours/NULLIF(active_days,0)<{duration0} THEN '<{duration0} h/day' WHEN effective_hours/NULLIF(active_days,0)<{duration1} THEN '{duration0}–{duration1} h/day' WHEN effective_hours/NULLIF(active_days,0)<{duration2} THEN '{duration1}–{duration2} h/day' ELSE '≥{duration2} h/day' END"))
          .replace("CASE WHEN peak_hours/NULLIF(active_days,0)<0.5 THEN '<0.5 h/day' WHEN peak_hours/NULLIF(active_days,0)<1.5 THEN '0.5–1.5 h/day' WHEN peak_hours/NULLIF(active_days,0)<3 THEN '1.5–3 h/day' ELSE '≥3 h/day' END",&format!("CASE WHEN peak_hours/NULLIF(active_days,0)<{peak0} THEN '<{peak0} h/day' WHEN peak_hours/NULLIF(active_days,0)<{peak1} THEN '{peak0}–{peak1} h/day' WHEN peak_hours/NULLIF(active_days,0)<{peak2} THEN '{peak1}–{peak2} h/day' ELSE '≥{peak2} h/day' END"))
          .replace("CASE WHEN observations=0 THEN '0 条/day' WHEN observations/NULLIF(active_days,0)<=5 THEN '1–5 条/day' WHEN observations/NULLIF(active_days,0)<=20 THEN '6–20 条/day' WHEN observations/NULLIF(active_days,0)<=50 THEN '21–50 条/day' ELSE '>50 条/day' END",&format!("CASE WHEN observations=0 THEN '0 条/day' WHEN observations/NULLIF(active_days,0)<={obs0} THEN '≤{obs0} 条/day' WHEN observations/NULLIF(active_days,0)<={obs1} THEN '{obs0}–{obs1} 条/day' WHEN observations/NULLIF(active_days,0)<={obs2} THEN '{obs1}–{obs2} 条/day' ELSE '>{obs2} 条/day' END"))
          .replace("CASE WHEN effective_mbps IS NULL THEN '不可用' WHEN effective_mbps<1 THEN '<1 Mbps' WHEN effective_mbps<5 THEN '1–5 Mbps' WHEN effective_mbps<20 THEN '5–20 Mbps' WHEN effective_mbps<50 THEN '20–50 Mbps' ELSE '≥50 Mbps' END",&format!("CASE WHEN effective_mbps IS NULL THEN '不可用' WHEN effective_mbps<{rate0} THEN '<{rate0} Mbps' WHEN effective_mbps<{rate1} THEN '{rate0}–{rate1} Mbps' WHEN effective_mbps<{rate2} THEN '{rate1}–{rate2} Mbps' WHEN effective_mbps<{rate3} THEN '{rate2}–{rate3} Mbps' ELSE '≥{rate3} Mbps' END"))
          .replace("CASE WHEN download_mbps IS NULL THEN '不可用' WHEN download_mbps<1 THEN '<1 Mbps' WHEN download_mbps<5 THEN '1–5 Mbps' WHEN download_mbps<20 THEN '5–20 Mbps' WHEN download_mbps<50 THEN '20–50 Mbps' ELSE '≥50 Mbps' END",&format!("CASE WHEN download_mbps IS NULL THEN '不可用' WHEN download_mbps<{rate0} THEN '<{rate0} Mbps' WHEN download_mbps<{rate1} THEN '{rate0}–{rate1} Mbps' WHEN download_mbps<{rate2} THEN '{rate1}–{rate2} Mbps' WHEN download_mbps<{rate3} THEN '{rate2}–{rate3} Mbps' ELSE '≥{rate3} Mbps' END"))
          .replace("CASE WHEN subscriber_rtt IS NULL THEN '不可用' WHEN subscriber_rtt<30 THEN '<30 ms' WHEN subscriber_rtt<60 THEN '30–60 ms' WHEN subscriber_rtt<100 THEN '60–100 ms' WHEN subscriber_rtt<200 THEN '100–200 ms' ELSE '≥200 ms' END",&format!("CASE WHEN subscriber_rtt IS NULL THEN '不可用' WHEN subscriber_rtt<{rtt0} THEN '<{rtt0} ms' WHEN subscriber_rtt<{rtt1} THEN '{rtt0}–{rtt1} ms' WHEN subscriber_rtt<{rtt2} THEN '{rtt1}–{rtt2} ms' WHEN subscriber_rtt<{rtt3} THEN '{rtt2}–{rtt3} ms' ELSE '≥{rtt3} ms' END"))
          .replace("CASE WHEN network_rtt IS NULL THEN '不可用' WHEN network_rtt<30 THEN '<30 ms' WHEN network_rtt<60 THEN '30–60 ms' WHEN network_rtt<100 THEN '60–100 ms' WHEN network_rtt<200 THEN '100–200 ms' ELSE '≥200 ms' END",&format!("CASE WHEN network_rtt IS NULL THEN '不可用' WHEN network_rtt<{rtt0} THEN '<{rtt0} ms' WHEN network_rtt<{rtt1} THEN '{rtt0}–{rtt1} ms' WHEN network_rtt<{rtt2} THEN '{rtt1}–{rtt2} ms' WHEN network_rtt<{rtt3} THEN '{rtt2}–{rtt3} ms' ELSE '≥{rtt3} ms' END"))
          .replace("CASE WHEN user_loss IS NULL THEN '不可用' WHEN user_loss=0 THEN '0%' WHEN user_loss<0.5 THEN '0–0.5%' WHEN user_loss<1 THEN '0.5–1%' WHEN user_loss<3 THEN '1–3%' ELSE '≥3%' END",&format!("CASE WHEN user_loss IS NULL THEN '不可用' WHEN user_loss=0 THEN '0%' WHEN user_loss<{loss0} THEN '0–{loss0}%' WHEN user_loss<{loss1} THEN '{loss0}–{loss1}%' WHEN user_loss<{loss2} THEN '{loss1}–{loss2}%' ELSE '≥{loss2}%' END"))
          .replace("CASE WHEN network_loss IS NULL THEN '不可用' WHEN network_loss=0 THEN '0%' WHEN network_loss<0.5 THEN '0–0.5%' WHEN network_loss<1 THEN '0.5–1%' WHEN network_loss<3 THEN '1–3%' ELSE '≥3%' END",&format!("CASE WHEN network_loss IS NULL THEN '不可用' WHEN network_loss=0 THEN '0%' WHEN network_loss<{loss0} THEN '0–{loss0}%' WHEN network_loss<{loss1} THEN '{loss0}–{loss1}%' WHEN network_loss<{loss2} THEN '{loss1}–{loss2}%' ELSE '≥{loss2}%' END"));
        let rows = conn.exec_iter(sql, (&run_id, &run_id, &run_id, keyword.clone(), keyword)).map_err(|err| format!("failed to query user distributions: {err}"))?;
        rows.map(|row| {
            let row = row.map_err(|err| format!("failed to decode user distribution: {err}"))?;
            let dimension = row_string(&row, 0, "UNKNOWN");
            let band = row_string(&row, 1, "Unknown");
            let users = row_i64(&row, 2);
            let availability = if dimension == "DURATION_GAME_ONLY" { "PARTIAL_GAME_ONLY" } else { "AVAILABLE" };
            Ok(MetricCard { label: band.clone(), value: users.to_string(), hint: format!("perspective=user, dimension={dimension}, band={band}, users={users}, unit=users, availability={availability}") })
        }).collect()
    })
}

#[tauri::command]
pub fn decision_get_quality_overview(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_quality_overview", || {
        let table = source_table(&req)?;
        ensure_run_source(&req, &table)?;
        let mut conn = db::conn(&req.settings)?;
        let sql = format!(
            "SELECT 'USER_SIDE' side,COUNT(DISTINCT user_key) users,SUM(poor_subscriber_rtt_obs+poor_user_loss_obs) evidence_obs FROM `{table}` WHERE analysis_run_id=? AND (poor_subscriber_rtt_obs+poor_user_loss_obs)>0 UNION ALL SELECT 'NETWORK_SIDE',COUNT(DISTINCT user_key),SUM(poor_network_rtt_obs+poor_network_loss_obs) FROM `{table}` WHERE analysis_run_id=? AND (poor_network_rtt_obs+poor_network_loss_obs)>0 UNION ALL SELECT 'APPLICATION_EXPERIENCE',COUNT(DISTINCT user_key),SUM(poor_vmos_obs+poor_mos_obs+poor_jitter_obs) FROM `{table}` WHERE analysis_run_id=? AND (poor_vmos_obs+poor_mos_obs+poor_jitter_obs)>0 UNION ALL SELECT 'EVIDENCE_INSUFFICIENT',COUNT(DISTINCT user_key),0 FROM `{table}` WHERE analysis_run_id=? AND persistent_poor_user_flag=1 AND (poor_subscriber_rtt_obs+poor_user_loss_obs+poor_network_rtt_obs+poor_network_loss_obs+poor_vmos_obs+poor_mos_obs+poor_jitter_obs)=0"
        );
        let run = req.run_id();
        let rows = conn.exec_iter(sql, (&run, &run, &run, &run)).map_err(|err| format!("failed to query quality overview: {err}"))?;
        rows.map(|row| {
            let row = row.map_err(|err| format!("failed to decode quality overview: {err}"))?;
            let side = row_string(&row, 0, "EVIDENCE_INSUFFICIENT");
            let users = row_i64(&row, 1);
            let evidence = row_i64(&row, 2);
            Ok(MetricCard { label: side.clone(), value: users.to_string(), hint: format!("issue_side={side}, affected_users={users}, evidence_observations={evidence}, interpretation=evidence_direction_not_confirmed_root_cause") })
        }).collect()
    })
}

#[tauri::command]
pub fn decision_get_access_compare(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_access_compare", || {
        let mut conn = db::conn(&req.settings)?;
        let rows = conn.exec_iter("SELECT user_type,CAST(observed_users AS SIGNED),CAST(eligible_users AS SIGNED),CAST(valid_obs_rows AS SIGNED),CAST(poor_obs_rows AS SIGNED),CAST(persistent_poor_users AS SIGNED),CAST(severe_poor_users AS SIGNED),CAST(total_download_gb AS DOUBLE),CAST(total_effective_duration_hours AS DOUBLE),CAST(avg_effective_download_mbps AS DOUBLE),CAST(avg_subscriber_rtt_ms AS DOUBLE),CAST(avg_network_rtt_ms AS DOUBLE),CAST(avg_user_loss_pct AS DOUBLE),CAST(avg_network_loss_pct AS DOUBLE) FROM ads_access_overview_v2 WHERE analysis_run_id=? AND user_type IN ('CABLE','FTTH') ORDER BY FIELD(user_type,'CABLE','FTTH')", (req.run_id(),)).map_err(|err| format!("failed to query access compare: {err}"))?;
        rows.map(|row| {
            let row = row.map_err(|err| format!("failed to decode access compare: {err}"))?;
            let access = row_string(&row, 0, "UNAVAILABLE");
            let users = row_i64(&row, 1);
            let eligible = row_i64(&row, 2);
            let valid = row_i64(&row, 3);
            let poor = row_i64(&row, 4);
            let persistent = row_i64(&row, 5);
            let severe = row_i64(&row, 6);
            Ok(MetricCard { label: access.clone(), value: users.to_string(), hint: format!("source=ads_access_overview_v2, access_type={access}, users={users}, eligible_users={eligible}, valid_obs_rows={valid}, poor_obs_rows={poor}, poor_observation_rate_pct={}, persistent_poor_users={persistent}, persistent_poor_user_rate_pct={}, severe_poor_users={severe}, severe_poor_user_rate_pct={}, traffic_gb={}, duration_hours={}, effective_download_mbps={}, subscriber_rtt_ms={}, network_rtt_ms={}, user_loss_pct={}, network_loss_pct={}", fmt_optional(rate(poor, valid),4), fmt_optional(rate(persistent,eligible),4), fmt_optional(rate(severe,eligible),4), fmt_optional(row_f64(&row,7),4), fmt_optional(row_f64(&row,8),4), fmt_optional(row_f64(&row,9),4), fmt_optional(row_f64(&row,10),4), fmt_optional(row_f64(&row,11),4), fmt_optional(row_f64(&row,12),6), fmt_optional(row_f64(&row,13),6)) })
        }).collect()
    })
}

#[tauri::command]
pub fn decision_get_access_hourly(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_access_hourly", || {
        let mut conn = db::conn(&req.settings)?;
        let rows = conn.exec_iter("SELECT user_type,CAST(hour_of_day AS SIGNED),CAST(observed_users AS SIGNED),CAST(observation_rows AS SIGNED),CAST(valid_obs_rows AS SIGNED),CAST(poor_obs_rows AS SIGNED),CAST(total_download_gb AS DOUBLE),CAST(total_effective_duration_hours AS DOUBLE),CAST(avg_effective_download_mbps AS DOUBLE),CAST(avg_subscriber_rtt_ms AS DOUBLE),CAST(avg_network_rtt_ms AS DOUBLE),CAST(avg_user_loss_pct AS DOUBLE),CAST(avg_network_loss_pct AS DOUBLE) FROM ads_access_hourly_v2 WHERE analysis_run_id=? AND user_type IN ('CABLE','FTTH') ORDER BY hour_of_day,FIELD(user_type,'CABLE','FTTH')", (req.run_id(),)).map_err(|err| format!("failed to query access hourly: {err}"))?;
        rows.map(|row| {
            let row = row.map_err(|err| format!("failed to decode access hourly: {err}"))?;
            let access = row_string(&row,0,"UNKNOWN"); let hour = row_i64(&row,1); let users = row_i64(&row,2); let valid = row_i64(&row,4); let poor = row_i64(&row,5);
            Ok(MetricCard { label: format!("{hour:02}:00 {access}"), value: users.to_string(), hint: format!("dimension=ACCESS_HOURLY, access_type={access}, hour={hour}, users={users}, observations={}, valid_obs_rows={valid}, poor_obs_rows={poor}, poor_observation_rate_pct={}, traffic_gb={}, duration_hours={}, effective_download_mbps={}, subscriber_rtt_ms={}, network_rtt_ms={}, user_loss_pct={}, network_loss_pct={}", row_i64(&row,3), fmt_optional(rate(poor,valid),4), fmt_optional(row_f64(&row,6),4), fmt_optional(row_f64(&row,7),4), fmt_optional(row_f64(&row,8),4), fmt_optional(row_f64(&row,9),4), fmt_optional(row_f64(&row,10),4), fmt_optional(row_f64(&row,11),6), fmt_optional(row_f64(&row,12),6)) })
        }).collect()
    })
}

#[tauri::command]
pub fn decision_get_panorama_hourly(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_panorama_hourly", || {
        let mut conn=db::conn(&req.settings)?;
        let rows=conn.exec_iter("SELECT CAST(hour_of_day AS SIGNED),CAST(observed_users AS SIGNED),CAST(observation_rows AS SIGNED),CAST(valid_obs_rows AS SIGNED),CAST(poor_obs_rows AS SIGNED),CAST(total_download_gb AS DOUBLE),CAST(total_effective_duration_hours AS DOUBLE),CAST(avg_effective_download_mbps AS DOUBLE),CAST(avg_subscriber_rtt_ms AS DOUBLE),CAST(avg_network_rtt_ms AS DOUBLE),CAST(avg_user_loss_pct AS DOUBLE),CAST(avg_network_loss_pct AS DOUBLE) FROM ads_access_hourly_v2 WHERE analysis_run_id=? AND user_type='ALL' ORDER BY hour_of_day",(req.run_id(),)).map_err(|err| format!("failed to query panorama hourly: {err}"))?;
        rows.map(|row| { let row=row.map_err(|err| format!("failed to decode panorama hourly: {err}"))?; let hour=row_i64(&row,0); let users=row_i64(&row,1); let valid=row_i64(&row,3); let poor=row_i64(&row,4); Ok(MetricCard { label: format!("{hour:02}:00"),value:users.to_string(),hint:format!("dimension=PANORAMA_HOURLY, hour={hour}, users={users}, observations={}, valid_obs_rows={valid}, poor_obs_rows={poor}, poor_observation_rate_pct={}, traffic_gb={}, duration_hours={}, effective_download_mbps={}, subscriber_rtt_ms={}, network_rtt_ms={}, user_loss_pct={}, network_loss_pct={}",row_i64(&row,2),fmt_optional(rate(poor,valid),4),fmt_optional(row_f64(&row,5),4),fmt_optional(row_f64(&row,6),4),fmt_optional(row_f64(&row,7),4),fmt_optional(row_f64(&row,8),4),fmt_optional(row_f64(&row,9),4),fmt_optional(row_f64(&row,10),6),fmt_optional(row_f64(&row,11),6)) }) }).collect()
    })
}

#[tauri::command]
pub fn decision_get_access_user_bands(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_access_user_bands", || {
        let mut conn = db::conn(&req.settings)?;
        let rows = conn.exec_iter("SELECT user_type,dimension_code,band_code,band_label,CAST(band_order AS SIGNED),CAST(users AS SIGNED) FROM ads_access_user_band_v2 WHERE analysis_run_id=? AND user_type IN ('CABLE','FTTH') ORDER BY dimension_code,band_order,FIELD(user_type,'CABLE','FTTH')", (req.run_id(),)).map_err(|err| format!("failed to query access user bands: {err}"))?;
        rows.map(|row| { let row = row.map_err(|err| format!("failed to decode access user band: {err}"))?; let access=row_string(&row,0,"UNKNOWN"); let dimension=row_string(&row,1,"UNKNOWN"); let code=row_string(&row,2,"UNKNOWN"); let label=row_string(&row,3,"Unknown"); let order=row_i64(&row,4); let users=row_i64(&row,5); Ok(MetricCard { label: label.clone(), value: users.to_string(), hint: format!("dimension={dimension}, band_code={code}, band_order={order}, access_type={access}, users={users}, unit=users") }) }).collect()
    })
}

#[tauri::command]
pub fn decision_get_access_apps(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_access_apps", || {
        let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_app_access_period_experience_v2")?;
        ensure_run_source(&req,&table)?;
        let mut conn = db::conn(&req.settings)?;
        let limit=req.page_size(500,1000);
        let sql=format!("SELECT app_name,user_type,app_category,CAST(observed_users AS SIGNED),CAST(eligible_users AS SIGNED),CAST(valid_obs_rows AS SIGNED),CAST(poor_obs_rows AS SIGNED),CAST(persistent_poor_users AS SIGNED),sample_status,CAST(total_download_gb AS DOUBLE),CAST(avg_effective_download_mbps AS DOUBLE),CAST(avg_subscriber_rtt_ms AS DOUBLE),CAST(avg_network_rtt_ms AS DOUBLE),CAST(avg_user_loss_pct AS DOUBLE),CAST(avg_network_loss_pct AS DOUBLE) FROM `{table}` WHERE analysis_run_id=? AND user_type IN ('CABLE','FTTH') ORDER BY observed_users DESC LIMIT ?");
        let rows=conn.exec_iter(sql,(req.run_id(),limit)).map_err(|err| format!("failed to query access App compare: {err}"))?;
        rows.map(|row| { let row=row.map_err(|err| format!("failed to decode access App compare: {err}"))?; let app=row_string(&row,0,"UNKNOWN_APP"); let access=row_string(&row,1,"UNKNOWN"); let observed=row_i64(&row,3); let eligible=row_i64(&row,4); let valid=row_i64(&row,5); let poor=row_i64(&row,6); let persistent=row_i64(&row,7); Ok(MetricCard { label: app.clone(), value: observed.to_string(), hint: format!("dimension=ACCESS_APP, app_name={app}, access_type={access}, app_category={}, observed_users={observed}, eligible_users={eligible}, valid_obs_rows={valid}, poor_obs_rows={poor}, poor_observation_rate_pct={}, persistent_poor_users={persistent}, persistent_poor_user_rate_pct={}, sample_status={}, traffic_gb={}, effective_download_mbps={}, subscriber_rtt_ms={}, network_rtt_ms={}, user_loss_pct={}, network_loss_pct={}",row_string(&row,2,"other"),fmt_optional(rate(poor,valid),4),fmt_optional(rate(persistent,eligible),4),row_string(&row,8,"INSUFFICIENT_SAMPLE"),fmt_optional(row_f64(&row,9),4),fmt_optional(row_f64(&row,10),4),fmt_optional(row_f64(&row,11),4),fmt_optional(row_f64(&row,12),4),fmt_optional(row_f64(&row,13),6),fmt_optional(row_f64(&row,14),6)) }) }).collect()
    })
}

pub(crate) fn materialize_access_user_core(req: EtlRequest) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&req.settings)?;
    let run_id = opportunity_run_id(&req);
    let period=batch_tables::resolve_table(&req.settings,&req.import_batch_id,"dws_user_app_period_experience_v2")?;
    let daily=batch_tables::resolve_table(&req.settings,&req.import_batch_id,"dws_user_daily_profile")?;
    let mut conn=db::conn(&req.settings)?;
    if !batch_tables::table_has_analysis_run(&mut conn,&period,&run_id)? { return Err(format!("access period source not ready for analysis_run_id={run_id}")); }
    ensure_opportunity_rules(&mut conn,&req,&run_id)?;
    conn.exec_drop("DELETE FROM dws_user_access_period_v2 WHERE analysis_run_id=?",(&run_id,)).map_err(|err| format!("failed to reset access user core: {err}"))?;
    let sql=format!(r#"
      INSERT INTO dws_user_access_period_v2
        (analysis_run_id,import_batch_id,user_key,user_type,observation_rows,valid_obs_rows,poor_obs_rows,severe_obs_rows,
         persistent_poor_user_flag,severe_poor_user_flag,total_download_gb,total_effective_duration_hours,active_days,
         effective_download_mbps_sum,effective_download_mbps_count,subscriber_rtt_sum,subscriber_rtt_count,
         network_rtt_sum,network_rtt_count,user_loss_sum,user_loss_count,network_loss_sum,network_loss_count)
      WITH policy AS (
        SELECT p.* FROM meta_analysis_run_policy_binding b JOIN meta_experience_analysis_policy p
          ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version
        WHERE b.analysis_run_id=?
      ), users AS (
        SELECT p.analysis_run_id,p.import_batch_id,p.user_key,p.user_type,SUM(p.observation_rows) observation_rows,
          SUM(p.valid_obs_rows) valid_obs_rows,SUM(p.poor_obs_rows) poor_obs_rows,SUM(p.severe_obs_rows) severe_obs_rows,
          SUM(p.total_download_gb) total_download_gb,SUM(p.total_effective_duration_hours) total_effective_duration_hours,
          SUM(p.effective_download_mbps_sum) effective_download_mbps_sum,SUM(p.effective_download_mbps_count) effective_download_mbps_count,
          SUM(p.subscriber_rtt_sum) subscriber_rtt_sum,SUM(p.subscriber_rtt_count) subscriber_rtt_count,
          SUM(p.network_rtt_sum) network_rtt_sum,SUM(p.network_rtt_count) network_rtt_count,
          SUM(p.user_loss_sum) user_loss_sum,SUM(p.user_loss_count) user_loss_count,
          SUM(p.network_loss_sum) network_loss_sum,SUM(p.network_loss_count) network_loss_count
        FROM `{period}` p WHERE p.analysis_run_id=? GROUP BY p.analysis_run_id,p.import_batch_id,p.user_key,p.user_type
      ), active AS (
        SELECT user_key,user_type,COUNT(DISTINCT stat_date) active_days FROM `{daily}` WHERE import_batch_id=? GROUP BY user_key,user_type
      )
      SELECT u.analysis_run_id,u.import_batch_id,u.user_key,u.user_type,u.observation_rows,u.valid_obs_rows,u.poor_obs_rows,u.severe_obs_rows,
        CASE WHEN u.valid_obs_rows>=p.persistent_min_valid_obs AND u.poor_obs_rows>=p.persistent_min_poor_obs AND u.poor_obs_rows*100.0/NULLIF(u.valid_obs_rows,0)>=p.persistent_min_poor_rate_pct THEN 1 ELSE 0 END,
        CASE WHEN u.valid_obs_rows>=p.severe_user_min_valid_obs AND u.severe_obs_rows>=p.severe_user_min_severe_obs AND u.severe_obs_rows*100.0/NULLIF(u.valid_obs_rows,0)>=p.severe_user_min_severe_rate_pct THEN 1 ELSE 0 END,
        u.total_download_gb,u.total_effective_duration_hours,COALESCE(a.active_days,0),u.effective_download_mbps_sum,u.effective_download_mbps_count,
        u.subscriber_rtt_sum,u.subscriber_rtt_count,u.network_rtt_sum,u.network_rtt_count,u.user_loss_sum,u.user_loss_count,u.network_loss_sum,u.network_loss_count
      FROM users u LEFT JOIN active a ON a.user_key=u.user_key AND a.user_type=u.user_type CROSS JOIN policy p
    "#);
    conn.exec_drop(sql,(&run_id,&run_id,&req.import_batch_id)).map_err(|err| format!("failed to materialize access user core: {err}"))?;
    Ok(ack(format!("access user core ready: analysis_run_id={run_id}, affected_rows={}",conn.affected_rows())))
}

pub(crate) fn materialize_access_overview(req: EtlRequest) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&req.settings)?;
    let run_id=opportunity_run_id(&req); let mut conn=db::conn(&req.settings)?;
    conn.exec_drop("DELETE FROM ads_access_overview_v2 WHERE analysis_run_id=?",(&run_id,)).map_err(|err| format!("failed to reset access overview: {err}"))?;
    conn.exec_drop("INSERT INTO ads_access_overview_v2 (analysis_run_id,import_batch_id,user_type,observed_users,eligible_users,observation_rows,valid_obs_rows,poor_obs_rows,persistent_poor_users,severe_poor_users,total_download_gb,total_effective_duration_hours,avg_effective_download_mbps,avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,avg_network_loss_pct,effective_download_mbps_sum,effective_download_mbps_count,subscriber_rtt_sum,subscriber_rtt_count,network_rtt_sum,network_rtt_count,user_loss_sum,user_loss_count,network_loss_sum,network_loss_count) SELECT u.analysis_run_id,MAX(u.import_batch_id),u.user_type,COUNT(*),SUM(u.valid_obs_rows>=p.persistent_min_valid_obs),SUM(u.observation_rows),SUM(u.valid_obs_rows),SUM(u.poor_obs_rows),SUM(u.persistent_poor_user_flag),SUM(u.severe_poor_user_flag),SUM(u.total_download_gb),SUM(u.total_effective_duration_hours),SUM(u.effective_download_mbps_sum)/NULLIF(SUM(u.effective_download_mbps_count),0),SUM(u.subscriber_rtt_sum)/NULLIF(SUM(u.subscriber_rtt_count),0),SUM(u.network_rtt_sum)/NULLIF(SUM(u.network_rtt_count),0),SUM(u.user_loss_sum)/NULLIF(SUM(u.user_loss_count),0),SUM(u.network_loss_sum)/NULLIF(SUM(u.network_loss_count),0),SUM(u.effective_download_mbps_sum),SUM(u.effective_download_mbps_count),SUM(u.subscriber_rtt_sum),SUM(u.subscriber_rtt_count),SUM(u.network_rtt_sum),SUM(u.network_rtt_count),SUM(u.user_loss_sum),SUM(u.user_loss_count),SUM(u.network_loss_sum),SUM(u.network_loss_count) FROM dws_user_access_period_v2 u JOIN meta_analysis_run_policy_binding b ON b.analysis_run_id=u.analysis_run_id JOIN meta_experience_analysis_policy p ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version WHERE u.analysis_run_id=? GROUP BY u.analysis_run_id,u.user_type",(&run_id,)).map_err(|err| format!("failed to materialize access overview: {err}"))?;
    Ok(ack(format!("access overview ready: analysis_run_id={run_id}, affected_rows={}",conn.affected_rows())))
}

pub(crate) fn materialize_access_hourly(req: EtlRequest) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&req.settings)?;
    let run_id=opportunity_run_id(&req); let hourly=batch_tables::resolve_table(&req.settings,&req.import_batch_id,"dws_user_app_hourly_experience_v2")?; let mut conn=db::conn(&req.settings)?;
    if !batch_tables::table_has_analysis_run(&mut conn,&hourly,&run_id)? { return Err(format!("access hourly source not ready for analysis_run_id={run_id}")); }
    conn.exec_drop("DELETE FROM ads_access_hourly_v2 WHERE analysis_run_id=?",(&run_id,)).map_err(|err| format!("failed to reset access hourly: {err}"))?;
    let sql=format!("INSERT INTO ads_access_hourly_v2 (analysis_run_id,import_batch_id,user_type,hour_of_day,observed_users,observation_rows,valid_obs_rows,poor_obs_rows,total_download_gb,total_effective_duration_hours,avg_effective_download_mbps,avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,avg_network_loss_pct,effective_download_mbps_sum,effective_download_mbps_count,subscriber_rtt_sum,subscriber_rtt_count,network_rtt_sum,network_rtt_count,user_loss_sum,user_loss_count,network_loss_sum,network_loss_count) SELECT analysis_run_id,MAX(import_batch_id),user_type,hour_of_day,COUNT(DISTINCT user_key),SUM(observation_rows),SUM(valid_obs_rows),SUM(poor_obs_rows),SUM(total_download_gb),SUM(total_effective_duration_hours),SUM(effective_download_mbps_sum)/NULLIF(SUM(effective_download_mbps_count),0),SUM(subscriber_rtt_sum)/NULLIF(SUM(subscriber_rtt_count),0),SUM(network_rtt_sum)/NULLIF(SUM(network_rtt_count),0),SUM(user_loss_sum)/NULLIF(SUM(user_loss_count),0),SUM(network_loss_sum)/NULLIF(SUM(network_loss_count),0),SUM(effective_download_mbps_sum),SUM(effective_download_mbps_count),SUM(subscriber_rtt_sum),SUM(subscriber_rtt_count),SUM(network_rtt_sum),SUM(network_rtt_count),SUM(user_loss_sum),SUM(user_loss_count),SUM(network_loss_sum),SUM(network_loss_count) FROM `{hourly}` WHERE analysis_run_id=? GROUP BY analysis_run_id,user_type,hour_of_day");
    conn.exec_drop(sql,(&run_id,)).map_err(|err| format!("failed to materialize access hourly: {err}"))?;
    conn.exec_drop("INSERT INTO ads_access_hourly_v2 (analysis_run_id,import_batch_id,user_type,hour_of_day,observed_users,observation_rows,valid_obs_rows,poor_obs_rows,total_download_gb,total_effective_duration_hours,avg_effective_download_mbps,avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,avg_network_loss_pct,effective_download_mbps_sum,effective_download_mbps_count,subscriber_rtt_sum,subscriber_rtt_count,network_rtt_sum,network_rtt_count,user_loss_sum,user_loss_count,network_loss_sum,network_loss_count) SELECT analysis_run_id,MAX(import_batch_id),'ALL',hour_of_day,SUM(observed_users),SUM(observation_rows),SUM(valid_obs_rows),SUM(poor_obs_rows),SUM(total_download_gb),SUM(total_effective_duration_hours),SUM(effective_download_mbps_sum)/NULLIF(SUM(effective_download_mbps_count),0),SUM(subscriber_rtt_sum)/NULLIF(SUM(subscriber_rtt_count),0),SUM(network_rtt_sum)/NULLIF(SUM(network_rtt_count),0),SUM(user_loss_sum)/NULLIF(SUM(user_loss_count),0),SUM(network_loss_sum)/NULLIF(SUM(network_loss_count),0),SUM(effective_download_mbps_sum),SUM(effective_download_mbps_count),SUM(subscriber_rtt_sum),SUM(subscriber_rtt_count),SUM(network_rtt_sum),SUM(network_rtt_count),SUM(user_loss_sum),SUM(user_loss_count),SUM(network_loss_sum),SUM(network_loss_count) FROM ads_access_hourly_v2 WHERE analysis_run_id=? AND user_type<>'ALL' GROUP BY analysis_run_id,hour_of_day",(&run_id,)).map_err(|err| format!("failed to materialize panorama hourly rollup: {err}"))?;
    Ok(ack(format!("access hourly ready: analysis_run_id={run_id}, affected_rows={}",conn.affected_rows())))
}

pub(crate) fn materialize_access_bands(req: EtlRequest) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&req.settings)?;
    let run_id=opportunity_run_id(&req); let mut conn=db::conn(&req.settings)?;
    let thresholds=distribution_thresholds_for_run(&mut conn,&run_id)?;
    let [traffic0,traffic1,traffic2]=thresholds.traffic[..] else { unreachable!() };
    let [duration0,duration1,duration2]=thresholds.duration[..] else { unreachable!() };
    let [rate0,rate1,rate2,rate3]=thresholds.rate[..] else { unreachable!() };
    let [rtt0,rtt1,rtt2,_]=thresholds.rtt[..] else { unreachable!() };
    let [loss0,loss1,loss2]=thresholds.loss[..] else { unreachable!() };
    conn.exec_drop("DELETE FROM ads_access_user_band_v2 WHERE analysis_run_id=?",(&run_id,)).map_err(|err| format!("failed to reset access bands: {err}"))?;
    let sql=r#"INSERT INTO ads_access_user_band_v2 (analysis_run_id,import_batch_id,user_type,dimension_code,band_code,band_label,band_order,users)
      WITH users AS (SELECT *,total_download_gb/NULLIF(active_days,0) daily_traffic,total_effective_duration_hours/NULLIF(active_days,0) daily_duration,effective_download_mbps_sum/NULLIF(effective_download_mbps_count,0) effective_mbps,subscriber_rtt_sum/NULLIF(subscriber_rtt_count,0) subscriber_rtt,network_rtt_sum/NULLIF(network_rtt_count,0) network_rtt,user_loss_sum/NULLIF(user_loss_count,0) user_loss,network_loss_sum/NULLIF(network_loss_count,0) network_loss FROM dws_user_access_period_v2 WHERE analysis_run_id=?), bands AS (
      SELECT analysis_run_id,import_batch_id,user_type,'TRAFFIC_DAILY' dimension_code,CASE WHEN daily_traffic<1 THEN 'LT_1' WHEN daily_traffic<5 THEN '1_5' WHEN daily_traffic<15 THEN '5_15' ELSE 'GE_15' END band_code,CASE WHEN daily_traffic<1 THEN '<1 GB/day' WHEN daily_traffic<5 THEN '1–5 GB/day' WHEN daily_traffic<15 THEN '5–15 GB/day' ELSE '≥15 GB/day' END band_label,CASE WHEN daily_traffic<1 THEN 1 WHEN daily_traffic<5 THEN 2 WHEN daily_traffic<15 THEN 3 ELSE 4 END band_order FROM users WHERE active_days>0
      UNION ALL SELECT analysis_run_id,import_batch_id,user_type,'DURATION_EFFECTIVE_DAILY',CASE WHEN daily_duration<1 THEN 'LT_1' WHEN daily_duration<3 THEN '1_3' WHEN daily_duration<6 THEN '3_6' ELSE 'GE_6' END,CASE WHEN daily_duration<1 THEN '<1 h/day' WHEN daily_duration<3 THEN '1–3 h/day' WHEN daily_duration<6 THEN '3–6 h/day' ELSE '≥6 h/day' END,CASE WHEN daily_duration<1 THEN 1 WHEN daily_duration<3 THEN 2 WHEN daily_duration<6 THEN 3 ELSE 4 END FROM users WHERE active_days>0
      UNION ALL SELECT analysis_run_id,import_batch_id,user_type,'EFFECTIVE_RATE',CASE WHEN effective_mbps IS NULL THEN 'NA' WHEN effective_mbps<1 THEN 'LT_1' WHEN effective_mbps<5 THEN '1_5' WHEN effective_mbps<20 THEN '5_20' WHEN effective_mbps<50 THEN '20_50' ELSE 'GE_50' END,CASE WHEN effective_mbps IS NULL THEN '不可用' WHEN effective_mbps<1 THEN '<1 Mbps' WHEN effective_mbps<5 THEN '1–5 Mbps' WHEN effective_mbps<20 THEN '5–20 Mbps' WHEN effective_mbps<50 THEN '20–50 Mbps' ELSE '≥50 Mbps' END,CASE WHEN effective_mbps IS NULL THEN 0 WHEN effective_mbps<1 THEN 1 WHEN effective_mbps<5 THEN 2 WHEN effective_mbps<20 THEN 3 WHEN effective_mbps<50 THEN 4 ELSE 5 END FROM users
      UNION ALL SELECT analysis_run_id,import_batch_id,user_type,'SUBSCRIBER_RTT',CASE WHEN subscriber_rtt IS NULL THEN 'NA' WHEN subscriber_rtt<30 THEN 'LT_30' WHEN subscriber_rtt<60 THEN '30_60' WHEN subscriber_rtt<100 THEN '60_100' ELSE 'GE_100' END,CASE WHEN subscriber_rtt IS NULL THEN '不可用' WHEN subscriber_rtt<30 THEN '<30 ms' WHEN subscriber_rtt<60 THEN '30–60 ms' WHEN subscriber_rtt<100 THEN '60–100 ms' ELSE '≥100 ms' END,CASE WHEN subscriber_rtt IS NULL THEN 0 WHEN subscriber_rtt<30 THEN 1 WHEN subscriber_rtt<60 THEN 2 WHEN subscriber_rtt<100 THEN 3 ELSE 4 END FROM users
      UNION ALL SELECT analysis_run_id,import_batch_id,user_type,'NETWORK_RTT',CASE WHEN network_rtt IS NULL THEN 'NA' WHEN network_rtt<30 THEN 'LT_30' WHEN network_rtt<60 THEN '30_60' WHEN network_rtt<100 THEN '60_100' ELSE 'GE_100' END,CASE WHEN network_rtt IS NULL THEN '不可用' WHEN network_rtt<30 THEN '<30 ms' WHEN network_rtt<60 THEN '30–60 ms' WHEN network_rtt<100 THEN '60–100 ms' ELSE '≥100 ms' END,CASE WHEN network_rtt IS NULL THEN 0 WHEN network_rtt<30 THEN 1 WHEN network_rtt<60 THEN 2 WHEN network_rtt<100 THEN 3 ELSE 4 END FROM users
      UNION ALL SELECT analysis_run_id,import_batch_id,user_type,'USER_LOSS',CASE WHEN user_loss IS NULL THEN 'NA' WHEN user_loss=0 THEN 'ZERO' WHEN user_loss<0.5 THEN '0_05' WHEN user_loss<1 THEN '05_1' WHEN user_loss<3 THEN '1_3' ELSE 'GE_3' END,CASE WHEN user_loss IS NULL THEN '不可用' WHEN user_loss=0 THEN '0%' WHEN user_loss<0.5 THEN '0–0.5%' WHEN user_loss<1 THEN '0.5–1%' WHEN user_loss<3 THEN '1–3%' ELSE '≥3%' END,CASE WHEN user_loss IS NULL THEN 0 WHEN user_loss=0 THEN 1 WHEN user_loss<0.5 THEN 2 WHEN user_loss<1 THEN 3 WHEN user_loss<3 THEN 4 ELSE 5 END FROM users
      UNION ALL SELECT analysis_run_id,import_batch_id,user_type,'NETWORK_LOSS',CASE WHEN network_loss IS NULL THEN 'NA' WHEN network_loss=0 THEN 'ZERO' WHEN network_loss<0.5 THEN '0_05' WHEN network_loss<1 THEN '05_1' WHEN network_loss<3 THEN '1_3' ELSE 'GE_3' END,CASE WHEN network_loss IS NULL THEN '不可用' WHEN network_loss=0 THEN '0%' WHEN network_loss<0.5 THEN '0–0.5%' WHEN network_loss<1 THEN '0.5–1%' WHEN network_loss<3 THEN '1–3%' ELSE '≥3%' END,CASE WHEN network_loss IS NULL THEN 0 WHEN network_loss=0 THEN 1 WHEN network_loss<0.5 THEN 2 WHEN network_loss<1 THEN 3 WHEN network_loss<3 THEN 4 ELSE 5 END FROM users
      UNION ALL SELECT analysis_run_id,import_batch_id,user_type,'EXPERIENCE',IF(persistent_poor_user_flag=1,'PERSISTENT','NON_PERSISTENT'),IF(persistent_poor_user_flag=1,'持续质差用户','非持续质差用户'),IF(persistent_poor_user_flag=1,2,1) FROM users)
      SELECT analysis_run_id,MAX(import_batch_id),user_type,dimension_code,band_code,MAX(band_label),MAX(band_order),COUNT(*) FROM bands GROUP BY analysis_run_id,user_type,dimension_code,band_code"#.to_string()
      .replace("CASE WHEN daily_traffic<1 THEN 'LT_1' WHEN daily_traffic<5 THEN '1_5' WHEN daily_traffic<15 THEN '5_15' ELSE 'GE_15' END",&format!("CASE WHEN daily_traffic<{traffic0} THEN 'B1' WHEN daily_traffic<{traffic1} THEN 'B2' WHEN daily_traffic<{traffic2} THEN 'B3' ELSE 'B4' END"))
      .replace("CASE WHEN daily_traffic<1 THEN '<1 GB/day' WHEN daily_traffic<5 THEN '1–5 GB/day' WHEN daily_traffic<15 THEN '5–15 GB/day' ELSE '≥15 GB/day' END",&format!("CASE WHEN daily_traffic<{traffic0} THEN '<{traffic0} GB/day' WHEN daily_traffic<{traffic1} THEN '{traffic0}–{traffic1} GB/day' WHEN daily_traffic<{traffic2} THEN '{traffic1}–{traffic2} GB/day' ELSE '≥{traffic2} GB/day' END"))
      .replace("CASE WHEN daily_traffic<1 THEN 1 WHEN daily_traffic<5 THEN 2 WHEN daily_traffic<15 THEN 3 ELSE 4 END",&format!("CASE WHEN daily_traffic<{traffic0} THEN 1 WHEN daily_traffic<{traffic1} THEN 2 WHEN daily_traffic<{traffic2} THEN 3 ELSE 4 END"))
      .replace("CASE WHEN daily_duration<1 THEN 'LT_1' WHEN daily_duration<3 THEN '1_3' WHEN daily_duration<6 THEN '3_6' ELSE 'GE_6' END",&format!("CASE WHEN daily_duration<{duration0} THEN 'B1' WHEN daily_duration<{duration1} THEN 'B2' WHEN daily_duration<{duration2} THEN 'B3' ELSE 'B4' END"))
      .replace("CASE WHEN daily_duration<1 THEN '<1 h/day' WHEN daily_duration<3 THEN '1–3 h/day' WHEN daily_duration<6 THEN '3–6 h/day' ELSE '≥6 h/day' END",&format!("CASE WHEN daily_duration<{duration0} THEN '<{duration0} h/day' WHEN daily_duration<{duration1} THEN '{duration0}–{duration1} h/day' WHEN daily_duration<{duration2} THEN '{duration1}–{duration2} h/day' ELSE '≥{duration2} h/day' END"))
      .replace("CASE WHEN daily_duration<1 THEN 1 WHEN daily_duration<3 THEN 2 WHEN daily_duration<6 THEN 3 ELSE 4 END",&format!("CASE WHEN daily_duration<{duration0} THEN 1 WHEN daily_duration<{duration1} THEN 2 WHEN daily_duration<{duration2} THEN 3 ELSE 4 END"))
      .replace("CASE WHEN effective_mbps IS NULL THEN 'NA' WHEN effective_mbps<1 THEN 'LT_1' WHEN effective_mbps<5 THEN '1_5' WHEN effective_mbps<20 THEN '5_20' WHEN effective_mbps<50 THEN '20_50' ELSE 'GE_50' END",&format!("CASE WHEN effective_mbps IS NULL THEN 'NA' WHEN effective_mbps<{rate0} THEN 'B1' WHEN effective_mbps<{rate1} THEN 'B2' WHEN effective_mbps<{rate2} THEN 'B3' WHEN effective_mbps<{rate3} THEN 'B4' ELSE 'B5' END"))
      .replace("CASE WHEN effective_mbps IS NULL THEN '不可用' WHEN effective_mbps<1 THEN '<1 Mbps' WHEN effective_mbps<5 THEN '1–5 Mbps' WHEN effective_mbps<20 THEN '5–20 Mbps' WHEN effective_mbps<50 THEN '20–50 Mbps' ELSE '≥50 Mbps' END",&format!("CASE WHEN effective_mbps IS NULL THEN '不可用' WHEN effective_mbps<{rate0} THEN '<{rate0} Mbps' WHEN effective_mbps<{rate1} THEN '{rate0}–{rate1} Mbps' WHEN effective_mbps<{rate2} THEN '{rate1}–{rate2} Mbps' WHEN effective_mbps<{rate3} THEN '{rate2}–{rate3} Mbps' ELSE '≥{rate3} Mbps' END"))
      .replace("CASE WHEN effective_mbps IS NULL THEN 0 WHEN effective_mbps<1 THEN 1 WHEN effective_mbps<5 THEN 2 WHEN effective_mbps<20 THEN 3 WHEN effective_mbps<50 THEN 4 ELSE 5 END",&format!("CASE WHEN effective_mbps IS NULL THEN 0 WHEN effective_mbps<{rate0} THEN 1 WHEN effective_mbps<{rate1} THEN 2 WHEN effective_mbps<{rate2} THEN 3 WHEN effective_mbps<{rate3} THEN 4 ELSE 5 END"))
      .replace("subscriber_rtt<30",&format!("subscriber_rtt<{rtt0}")).replace("subscriber_rtt<60",&format!("subscriber_rtt<{rtt1}")).replace("subscriber_rtt<100",&format!("subscriber_rtt<{rtt2}"))
      .replace("network_rtt<30",&format!("network_rtt<{rtt0}")).replace("network_rtt<60",&format!("network_rtt<{rtt1}")).replace("network_rtt<100",&format!("network_rtt<{rtt2}"))
      .replace("user_loss<0.5",&format!("user_loss<{loss0}")).replace("user_loss<1",&format!("user_loss<{loss1}")).replace("user_loss<3",&format!("user_loss<{loss2}"))
      .replace("network_loss<0.5",&format!("network_loss<{loss0}")).replace("network_loss<1",&format!("network_loss<{loss1}")).replace("network_loss<3",&format!("network_loss<{loss2}"))
      .replace("'<30 ms'",&format!("'<{rtt0} ms'" )).replace("'30–60 ms'",&format!("'{rtt0}–{rtt1} ms'" )).replace("'60–100 ms'",&format!("'{rtt1}–{rtt2} ms'" )).replace("'≥100 ms'",&format!("'≥{rtt2} ms'" ))
      .replace("'0–0.5%'",&format!("'0–{loss0}%'" )).replace("'0.5–1%'",&format!("'{loss0}–{loss1}%'" )).replace("'1–3%'",&format!("'{loss1}–{loss2}%'" )).replace("'≥3%'",&format!("'≥{loss2}%'" ));
    conn.exec_drop(sql,(&run_id,)).map_err(|err| format!("failed to materialize access bands: {err}"))?;
    Ok(ack(format!("access user bands ready: analysis_run_id={run_id}, affected_rows={}",conn.affected_rows())))
}

fn opportunity_run_id(req: &EtlRequest) -> String {
    req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string())
}

fn ensure_opportunity_rules(conn: &mut mysql::PooledConn, req: &EtlRequest, run_id: &str) -> Result<(), String> {
    conn.exec_drop("INSERT IGNORE INTO meta_analysis_run_decision_binding (analysis_run_id,import_batch_id,rule_profile_id,rule_profile_version,rule_snapshot) SELECT ?,?,rule_profile_id,version,COALESCE(rule_snapshot,JSON_OBJECT()) FROM meta_decision_rule_profile WHERE status='published' ORDER BY version DESC LIMIT 1", (run_id,&req.import_batch_id)).map_err(|err| format!("failed to bind decision rules: {err}"))?;
    require_rule_binding(conn, run_id)
}

pub(crate) fn materialize_opportunity_features(req: EtlRequest) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&req.settings)?;
    let run_id = opportunity_run_id(&req);
    let period = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_app_period_experience_v2")?;
    let daily = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?;
    let mut conn = db::conn(&req.settings)?;
    if !batch_tables::table_has_analysis_run(&mut conn, &period, &run_id)? {
        return Err(format!("opportunity period source not ready for analysis_run_id={run_id}"));
    }
    ensure_opportunity_rules(&mut conn, &req, &run_id)?;
    conn.exec_drop("DELETE FROM dws_user_opportunity_feature_v2 WHERE analysis_run_id=?", (&run_id,)).map_err(|err| format!("failed to reset opportunity features: {err}"))?;
    let sql = format!(r#"
      INSERT INTO dws_user_opportunity_feature_v2
        (analysis_run_id,import_batch_id,user_key,user_type,active_days,observation_rows,
         total_download_gb,total_effective_duration_hours,avg_effective_download_mbps,
         avg_wifi_delay_ms,avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,
         avg_network_loss_pct,primary_app,primary_app_active_days,primary_app_observations,
         rule_profile_version)
      WITH app_rows AS (
        SELECT p.*,
          ROW_NUMBER() OVER (PARTITION BY p.user_key ORDER BY p.observation_rows DESC,p.total_download_gb DESC,p.app_name) AS app_rank
        FROM `{period}` p WHERE p.analysis_run_id=?
      ), users AS (
        SELECT user_key,
          CASE WHEN SUM(user_type='CABLE')>0 THEN 'CABLE' WHEN SUM(user_type='FTTH')>0 THEN 'FTTH'
               WHEN SUM(user_type='OTHER')>0 THEN 'OTHER' ELSE 'UNKNOWN' END user_type,
          SUM(observation_rows) observation_rows,SUM(total_download_gb) total_download_gb,
          SUM(total_effective_duration_hours) total_effective_duration_hours,
          SUM(effective_download_mbps_sum)/NULLIF(SUM(effective_download_mbps_count),0) avg_effective_download_mbps,
          SUM(avg_wifi_delay_ms*valid_obs_rows)/NULLIF(SUM(CASE WHEN avg_wifi_delay_ms IS NOT NULL THEN valid_obs_rows ELSE 0 END),0) avg_wifi_delay_ms,
          SUM(subscriber_rtt_sum)/NULLIF(SUM(subscriber_rtt_count),0) avg_subscriber_rtt_ms,
          SUM(network_rtt_sum)/NULLIF(SUM(network_rtt_count),0) avg_network_rtt_ms,
          SUM(user_loss_sum)/NULLIF(SUM(user_loss_count),0) avg_user_loss_pct,
          SUM(network_loss_sum)/NULLIF(SUM(network_loss_count),0) avg_network_loss_pct
        FROM app_rows GROUP BY user_key
      ), active AS (
        SELECT user_key,COUNT(DISTINCT stat_date) active_days FROM `{daily}`
        WHERE import_batch_id=? GROUP BY user_key
      ), primary_apps AS (
        SELECT user_key,app_name,active_days,observation_rows FROM app_rows WHERE app_rank=1
      ), rules AS (
        SELECT p.version FROM meta_decision_rule_profile p JOIN meta_analysis_run_decision_binding b
          ON b.rule_profile_id=p.rule_profile_id AND b.rule_profile_version=p.version WHERE b.analysis_run_id=?
      )
      SELECT ?,?,u.user_key,u.user_type,COALESCE(a.active_days,0),u.observation_rows,
        u.total_download_gb,u.total_effective_duration_hours,u.avg_effective_download_mbps,
        u.avg_wifi_delay_ms,u.avg_subscriber_rtt_ms,u.avg_network_rtt_ms,u.avg_user_loss_pct,
        u.avg_network_loss_pct,p.app_name,COALESCE(p.active_days,0),COALESCE(p.observation_rows,0),r.version
      FROM users u LEFT JOIN active a ON a.user_key=u.user_key
      LEFT JOIN primary_apps p ON p.user_key=u.user_key CROSS JOIN rules r
    "#);
    conn.exec_drop(sql, (&run_id,&req.import_batch_id,&run_id,&run_id,&req.import_batch_id)).map_err(|err| format!("failed to materialize opportunity features: {err}"))?;
    Ok(ack(format!("opportunity features ready: analysis_run_id={run_id}, affected_rows={}", conn.affected_rows())))
}

fn reset_staging_type(conn: &mut mysql::PooledConn, run_id: &str, kind: &str) -> Result<(), String> {
    conn.exec_drop("DELETE FROM stg_opportunity_user_v3 WHERE analysis_run_id=? AND opportunity_type=?", (run_id,kind)).map_err(|err| format!("failed to reset {kind} opportunity staging: {err}"))
}

const OPPORTUNITY_STAGING_COLUMNS: &str = "analysis_run_id,import_batch_id,user_key,opportunity_type,opportunity_level,user_type,active_days,observation_rows,total_download_gb,total_effective_duration_hours,avg_effective_download_mbps,avg_wifi_delay_ms,avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,avg_network_loss_pct,primary_app,primary_app_active_days,primary_app_observations,evidence_value,evidence_unit,evidence_summary,data_limitation_code,rule_profile_version";
const OPPORTUNITY_FEATURE_COLUMNS: &str = "u.analysis_run_id,u.import_batch_id,u.user_key,TYPE_TOKEN,LEVEL_TOKEN,u.user_type,u.active_days,u.observation_rows,u.total_download_gb,u.total_effective_duration_hours,u.avg_effective_download_mbps,u.avg_wifi_delay_ms,u.avg_subscriber_rtt_ms,u.avg_network_rtt_ms,u.avg_user_loss_pct,u.avg_network_loss_pct,u.primary_app,u.primary_app_active_days,u.primary_app_observations,EVIDENCE_VALUE,EVIDENCE_UNIT,EVIDENCE_SUMMARY,NULL,r.version";

fn materialize_opportunity_type(req: EtlRequest, kind: &str) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&req.settings)?;
    let run_id = opportunity_run_id(&req);
    let mut conn = db::conn(&req.settings)?;
    ensure_opportunity_rules(&mut conn, &req, &run_id)?;
    reset_staging_type(&mut conn, &run_id, kind)?;
    let (level, evidence_value, evidence_unit, evidence_summary, predicate) = match kind {
        "MIGRATION" => (
            "CASE WHEN u.total_download_gb/NULLIF(u.active_days,0)>=r.heavy_traffic_gb THEN 'HIGH' ELSE 'STANDARD' END",
            "u.total_download_gb/NULLIF(u.active_days,0)", "'GB/day'",
            "CONCAT('Cable; active_days=',u.active_days,'; daily traffic=',ROUND(u.total_download_gb/NULLIF(u.active_days,0),2),' GB; daily duration=',ROUND(u.total_effective_duration_hours/NULLIF(u.active_days,0),2),' h; primary app=',COALESCE(u.primary_app,'NA'))",
            "u.user_type='CABLE' AND u.active_days>=r.opportunity_min_active_days AND u.observation_rows>=r.opportunity_min_observations AND (u.total_download_gb/NULLIF(u.active_days,0)>=r.migration_min_traffic_gb OR u.total_effective_duration_hours/NULLIF(u.active_days,0)>=r.heavy_usage_hours OR u.observation_rows/NULLIF(u.active_days,0)>=r.app_bundle_min_observations)"
        ),
        "SPEED_UPGRADE" => (
            "CASE WHEN u.total_download_gb/NULLIF(u.active_days,0)>=r.heavy_traffic_gb THEN 'HIGH' ELSE 'STANDARD' END",
            "u.total_download_gb/NULLIF(u.active_days,0)", "'GB/day'",
            "CONCAT('demand conditions matched; daily traffic=',ROUND(u.total_download_gb/NULLIF(u.active_days,0),2),' GB; daily duration=',ROUND(u.total_effective_duration_hours/NULLIF(u.active_days,0),2),' h; effective rate=',COALESCE(ROUND(u.avg_effective_download_mbps,2),'NA'),' Mbps')",
            "u.active_days>=r.opportunity_min_active_days AND u.observation_rows>=r.opportunity_min_observations AND u.avg_effective_download_mbps IS NOT NULL AND u.avg_effective_download_mbps<=r.speed_upgrade_max_effective_mbps AND ((u.total_download_gb/NULLIF(u.active_days,0)>=r.speed_upgrade_min_traffic_gb)+(u.total_effective_duration_hours/NULLIF(u.active_days,0)>=r.heavy_usage_hours+1)+(u.observation_rows/NULLIF(u.active_days,0)>=r.app_bundle_min_observations*3))>=r.speed_upgrade_min_conditions"
        ),
        "MESH_AP" => (
            "'STANDARD'", "COALESCE(u.avg_wifi_delay_ms,u.avg_subscriber_rtt_ms-u.avg_network_rtt_ms)", "'ms'",
            "CONCAT('wifi delay=',COALESCE(ROUND(u.avg_wifi_delay_ms,2),'NA'),' ms; RTT delta=',COALESCE(ROUND(u.avg_subscriber_rtt_ms-u.avg_network_rtt_ms,2),'NA'),' ms; loss delta=',COALESCE(ROUND(u.avg_user_loss_pct-u.avg_network_loss_pct,2),'NA'),' pct')",
            "u.active_days>=r.opportunity_min_active_days AND u.observation_rows>=r.opportunity_min_observations AND (u.avg_wifi_delay_ms>=r.mesh_min_wifi_delay_ms OR u.avg_subscriber_rtt_ms-u.avg_network_rtt_ms>=r.mesh_min_rtt_delta_ms OR u.avg_user_loss_pct-u.avg_network_loss_pct>=r.mesh_min_loss_delta_pct) AND c.coverage_pct>=r.mesh_min_coverage_pct"
        ),
        "APP_BUNDLE" => (
            "CASE WHEN u.primary_app_observations/NULLIF(u.primary_app_active_days,0)>=r.app_bundle_min_observations*3 THEN 'HIGH' ELSE 'STANDARD' END",
            "u.primary_app_observations/NULLIF(u.primary_app_active_days,0)", "'observations/day'",
            "CONCAT('frequent app=',COALESCE(u.primary_app,'NA'),'; active_days=',u.primary_app_active_days,'; daily observations=',ROUND(u.primary_app_observations/NULLIF(u.primary_app_active_days,0),2))",
            "u.primary_app IS NOT NULL AND u.primary_app_active_days>=r.app_bundle_min_active_days AND u.primary_app_observations/NULLIF(u.primary_app_active_days,0)>=r.app_bundle_min_observations"
        ),
        _ => return Err(format!("unsupported opportunity type: {kind}")),
    };
    let select = OPPORTUNITY_FEATURE_COLUMNS.replace("TYPE_TOKEN", &format!("'{kind}'")).replace("LEVEL_TOKEN", level).replace("EVIDENCE_VALUE", evidence_value).replace("EVIDENCE_UNIT", evidence_unit).replace("EVIDENCE_SUMMARY", evidence_summary);
    let coverage = if kind == "MESH_AP" { "CROSS JOIN (SELECT COUNT(CASE WHEN avg_wifi_delay_ms IS NOT NULL OR (avg_subscriber_rtt_ms IS NOT NULL AND avg_network_rtt_ms IS NOT NULL) OR (avg_user_loss_pct IS NOT NULL AND avg_network_loss_pct IS NOT NULL) THEN 1 END)*100.0/NULLIF(COUNT(*),0) coverage_pct FROM dws_user_opportunity_feature_v2 WHERE analysis_run_id=?) c" } else { "" };
    let sql = format!("INSERT INTO stg_opportunity_user_v3 ({OPPORTUNITY_STAGING_COLUMNS}) SELECT {select} FROM dws_user_opportunity_feature_v2 u JOIN meta_analysis_run_decision_binding b ON b.analysis_run_id=u.analysis_run_id JOIN meta_decision_rule_profile r ON r.rule_profile_id=b.rule_profile_id AND r.version=b.rule_profile_version {coverage} WHERE u.analysis_run_id=? AND {predicate}");
    if kind == "MESH_AP" {
        conn.exec_drop(sql, (&run_id,&run_id)).map_err(|err| format!("failed to materialize {kind} opportunities: {err}"))?;
    } else {
        conn.exec_drop(sql, (&run_id,)).map_err(|err| format!("failed to materialize {kind} opportunities: {err}"))?;
    }
    Ok(ack(format!("{kind} opportunities staged: analysis_run_id={run_id}, affected_rows={}", conn.affected_rows())))
}

pub(crate) fn materialize_opportunity_migration(req: EtlRequest) -> Result<CommandAck, String> { materialize_opportunity_type(req, "MIGRATION") }
pub(crate) fn materialize_opportunity_speed_upgrade(req: EtlRequest) -> Result<CommandAck, String> { materialize_opportunity_type(req, "SPEED_UPGRADE") }
pub(crate) fn materialize_opportunity_mesh(req: EtlRequest) -> Result<CommandAck, String> { materialize_opportunity_type(req, "MESH_AP") }
pub(crate) fn materialize_opportunity_app_bundle(req: EtlRequest) -> Result<CommandAck, String> { materialize_opportunity_type(req, "APP_BUNDLE") }

pub(crate) fn publish_opportunities(req: EtlRequest) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&req.settings)?;
    let run_id = opportunity_run_id(&req);
    let mut conn = db::conn(&req.settings)?;
    ensure_opportunity_rules(&mut conn, &req, &run_id)?;
    let mut tx = conn.start_transaction(mysql::TxOpts::default()).map_err(|err| format!("failed to start opportunity publish transaction: {err}"))?;
    tx.exec_drop("DELETE FROM ads_opportunity_summary_v3 WHERE analysis_run_id=?", (&run_id,)).map_err(|err| format!("failed to reset published opportunity summary: {err}"))?;
    tx.exec_drop("DELETE FROM ads_opportunity_user_v3 WHERE analysis_run_id=?", (&run_id,)).map_err(|err| format!("failed to reset published opportunity users: {err}"))?;
    tx.exec_drop(format!("INSERT INTO ads_opportunity_user_v3 ({OPPORTUNITY_STAGING_COLUMNS}) SELECT {OPPORTUNITY_STAGING_COLUMNS} FROM stg_opportunity_user_v3 WHERE analysis_run_id=?"), (&run_id,)).map_err(|err| format!("failed to publish opportunity users: {err}"))?;
    tx.exec_drop("INSERT INTO ads_opportunity_summary_v3 (analysis_run_id,import_batch_id,opportunity_type,candidate_users,high_priority_users,total_evidence_value,evidence_unit,availability_status,data_limitation_code,rule_profile_version) WITH kinds AS (SELECT 'MIGRATION' kind UNION ALL SELECT 'SPEED_UPGRADE' UNION ALL SELECT 'MESH_AP' UNION ALL SELECT 'APP_BUNDLE'), coverage AS (SELECT COUNT(CASE WHEN avg_wifi_delay_ms IS NOT NULL OR (avg_subscriber_rtt_ms IS NOT NULL AND avg_network_rtt_ms IS NOT NULL) OR (avg_user_loss_pct IS NOT NULL AND avg_network_loss_pct IS NOT NULL) THEN 1 END)*100.0/NULLIF(COUNT(*),0) pct FROM dws_user_opportunity_feature_v2 WHERE analysis_run_id=?) SELECT ?,?,k.kind,COUNT(u.user_key),COALESCE(SUM(u.opportunity_level='HIGH'),0),SUM(u.evidence_value),MAX(u.evidence_unit),CASE WHEN k.kind='MESH_AP' AND COALESCE(c.pct,0)<r.mesh_min_coverage_pct THEN 'UNAVAILABLE' ELSE 'AVAILABLE' END,CASE WHEN k.kind='MESH_AP' AND COALESCE(c.pct,0)<r.mesh_min_coverage_pct THEN 'HOME_SIDE_EVIDENCE_COVERAGE_BELOW_THRESHOLD' ELSE NULL END,r.version FROM kinds k CROSS JOIN coverage c JOIN meta_analysis_run_decision_binding b ON b.analysis_run_id=? JOIN meta_decision_rule_profile r ON r.rule_profile_id=b.rule_profile_id AND r.version=b.rule_profile_version LEFT JOIN ads_opportunity_user_v3 u ON u.analysis_run_id=? AND u.opportunity_type=k.kind GROUP BY k.kind,c.pct,r.mesh_min_coverage_pct,r.version", (&run_id,&run_id,&req.import_batch_id,&run_id,&run_id)).map_err(|err| format!("failed to publish opportunity summary: {err}"))?;
    tx.commit().map_err(|err| format!("failed to commit opportunity publication: {err}"))?;
    Ok(ack(format!("opportunities atomically published: analysis_run_id={run_id}")))
}

pub(crate) fn materialize_opportunities(req: EtlRequest) -> Result<CommandAck, String> {
    materialize_opportunity_features(req.clone())?;
    materialize_opportunity_migration(req.clone())?;
    materialize_opportunity_speed_upgrade(req.clone())?;
    materialize_opportunity_mesh(req.clone())?;
    materialize_opportunity_app_bundle(req.clone())?;
    publish_opportunities(req)
}

#[tauri::command]
pub fn decision_materialize_opportunities(req: EtlRequest) -> Result<CommandAck, String> {
    let _lock = db::acquire_named_lock(&req.settings, db::AGGREGATION_LOCK_NAME)?;
    materialize_opportunities(req)
}

#[tauri::command]
pub fn decision_get_opportunities(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    crate::command_guard::run("decision_get_opportunities", || {
        let mut conn = db::conn(&req.settings)?;
        let rows = conn.exec_iter("SELECT opportunity_type,CAST(candidate_users AS SIGNED),CAST(high_priority_users AS SIGNED),CAST(total_evidence_value AS DOUBLE),evidence_unit,availability_status,COALESCE(data_limitation_code,''),rule_profile_version FROM ads_opportunity_summary_v3 WHERE analysis_run_id=? ORDER BY FIELD(opportunity_type,'MIGRATION','SPEED_UPGRADE','MESH_AP','APP_BUNDLE')", (req.run_id(),)).map_err(|err| format!("failed to query opportunity summary: {err}"))?;
        rows.map(|row| {
            let row = row.map_err(|err| format!("failed to decode opportunity summary: {err}"))?;
            let kind = row_string(&row,0,"UNKNOWN");
            let candidates = row_i64(&row,1);
            let high = row_i64(&row,2);
            Ok(MetricCard { label: kind.clone(), value: candidates.to_string(), hint: format!("opportunity_type={kind}, candidate_users={candidates}, high_priority_users={high}, evidence_value={}, evidence_unit={}, availability_status={}, data_limitation_code={}, rule_version={}", fmt_optional(row_f64(&row,3),4), row_string(&row,4,""), row_string(&row,5,"UNAVAILABLE"), row_string(&row,6,""), row_i64(&row,7)) })
        }).collect()
    })
}

#[tauri::command]
pub fn decision_get_opportunity_candidates(req: DashboardRequest) -> Result<OpportunityCandidatePage, String> {
    crate::command_guard::run("decision_get_opportunity_candidates", || {
        let run_id = req.run_id();
        let page = req.page();
        let page_size = req.page_size(50, 200);
        let offset = req.offset(50, 200);
        let opportunity_type = req.opportunity_type.as_deref().and_then(|value| match value {
            "MIGRATION" | "SPEED_UPGRADE" | "MESH_AP" | "APP_BUNDLE" => Some(value.to_string()),
            _ => None,
        });
        let keyword = req.keyword_like();
        let mut conn = db::conn(&req.settings)?;
        let total: Option<i64> = conn.exec_first(
            "SELECT COUNT(*) FROM ads_opportunity_user_v3 WHERE analysis_run_id=? AND (? IS NULL OR opportunity_type=?) AND (? IS NULL OR user_key LIKE ? OR primary_app LIKE ?)",
            (&run_id,&opportunity_type,&opportunity_type,&keyword,&keyword,&keyword),
        ).map_err(|err| format!("failed to count opportunity candidates: {err}"))?;
        let rows = conn.exec_iter(
            "SELECT user_key,opportunity_type,opportunity_level,user_type,CAST(active_days AS SIGNED),CAST(observation_rows AS SIGNED),CAST(total_download_gb AS DOUBLE),CAST(total_effective_duration_hours AS DOUBLE),CAST(avg_effective_download_mbps AS DOUBLE),CAST(avg_wifi_delay_ms AS DOUBLE),CAST(avg_subscriber_rtt_ms AS DOUBLE),CAST(avg_network_rtt_ms AS DOUBLE),CAST(avg_user_loss_pct AS DOUBLE),CAST(avg_network_loss_pct AS DOUBLE),primary_app,CAST(primary_app_active_days AS SIGNED),CAST(primary_app_observations AS SIGNED),CAST(evidence_value AS DOUBLE),evidence_unit,evidence_summary,data_limitation_code,CAST(rule_profile_version AS SIGNED) FROM ads_opportunity_user_v3 WHERE analysis_run_id=? AND (? IS NULL OR opportunity_type=?) AND (? IS NULL OR user_key LIKE ? OR primary_app LIKE ?) ORDER BY (opportunity_level='HIGH') DESC,evidence_value DESC,user_key LIMIT ? OFFSET ?",
            (&run_id,&opportunity_type,&opportunity_type,&keyword,&keyword,&keyword,page_size,offset),
        ).map_err(|err| format!("failed to query opportunity candidates: {err}"))?;
        let rows = rows.map(|row| {
            let row = row.map_err(|err| format!("failed to decode opportunity candidate: {err}"))?;
            Ok(OpportunityCandidateRow {
                user_key: row.get(0).unwrap_or_default(),
                opportunity_type: row.get(1).unwrap_or_default(),
                opportunity_level: row.get(2).unwrap_or_default(),
                user_type: row.get(3).unwrap_or_default(),
                active_days: row.get(4).unwrap_or_default(),
                observation_rows: row.get(5).unwrap_or_default(),
                total_download_gb: row.get(6).unwrap_or_default(),
                total_effective_duration_hours: row.get(7).unwrap_or_default(),
                avg_effective_download_mbps: row.get(8),
                avg_wifi_delay_ms: row.get(9),
                avg_subscriber_rtt_ms: row.get(10),
                avg_network_rtt_ms: row.get(11),
                avg_user_loss_pct: row.get(12),
                avg_network_loss_pct: row.get(13),
                primary_app: row.get(14),
                primary_app_active_days: row.get(15).unwrap_or_default(),
                primary_app_observations: row.get(16).unwrap_or_default(),
                evidence_value: row.get(17),
                evidence_unit: row.get(18),
                evidence_summary: row.get(19).unwrap_or_default(),
                data_limitation_code: row.get(20),
                rule_profile_version: row.get(21).unwrap_or_default(),
            })
        }).collect::<Result<Vec<_>, String>>()?;
        Ok(OpportunityCandidatePage { rows, total: total.unwrap_or(0), page, page_size })
    })
}

fn export_string(value: Option<String>) -> String {
    value.unwrap_or_default()
}

fn export_number(value: Option<f64>) -> String {
    value.map(|number| number.to_string()).unwrap_or_default()
}

#[tauri::command]
pub fn decision_export_opportunity_candidates_csv(
    req: OpportunityExportRequest,
) -> Result<CommandAck, String> {
    crate::command_guard::run("decision_export_opportunity_candidates_csv", || {
        if req.output_path.trim().is_empty() {
            return Err("opportunity export output path is required".to_string());
        }
        let opportunity_type = req.opportunity_type.as_deref().and_then(|value| match value {
            "MIGRATION" | "SPEED_UPGRADE" | "MESH_AP" | "APP_BUNDLE" => {
                Some(value.to_string())
            }
            _ => None,
        });
        let keyword = req.keyword.as_deref().map(str::trim).filter(|value| !value.is_empty())
            .map(|value| format!("%{}%", value.replace('%', "\\%").replace('_', "\\_")));
        let mut file = std::fs::File::create(&req.output_path)
            .map_err(|err| format!("failed to create opportunity export file: {err}"))?;
        file.write_all(&[0xEF, 0xBB, 0xBF])
            .map_err(|err| format!("failed to write opportunity export BOM: {err}"))?;
        let mut writer = csv::Writer::from_writer(file);
        writer.write_record([
            "user_ip", "opportunity_type", "opportunity_level", "access_type",
            "active_days", "observation_rows", "total_download_gb",
            "total_effective_duration_hours", "avg_effective_download_mbps",
            "avg_wifi_delay_ms", "avg_subscriber_rtt_ms", "avg_network_rtt_ms",
            "avg_user_loss_pct", "avg_network_loss_pct", "primary_app",
            "primary_app_active_days", "primary_app_observations", "evidence_value",
            "evidence_unit", "evidence_summary", "data_limitation_code",
            "rule_profile_version",
        ]).map_err(|err| format!("failed to write opportunity export header: {err}"))?;

        let mut conn = db::conn(&req.settings)?;
        let query = "SELECT user_key,opportunity_type,opportunity_level,user_type,CAST(active_days AS SIGNED),CAST(observation_rows AS SIGNED),CAST(total_download_gb AS DOUBLE),CAST(total_effective_duration_hours AS DOUBLE),CAST(avg_effective_download_mbps AS DOUBLE),CAST(avg_wifi_delay_ms AS DOUBLE),CAST(avg_subscriber_rtt_ms AS DOUBLE),CAST(avg_network_rtt_ms AS DOUBLE),CAST(avg_user_loss_pct AS DOUBLE),CAST(avg_network_loss_pct AS DOUBLE),primary_app,CAST(primary_app_active_days AS SIGNED),CAST(primary_app_observations AS SIGNED),CAST(evidence_value AS DOUBLE),evidence_unit,evidence_summary,data_limitation_code,CAST(rule_profile_version AS SIGNED) FROM ads_opportunity_user_v3 WHERE analysis_run_id=? AND import_batch_id=? AND (? IS NULL OR opportunity_type=?) AND (? IS NULL OR user_key LIKE ? OR primary_app LIKE ?) ORDER BY (opportunity_level='HIGH') DESC,evidence_value DESC,user_key";
        let values = vec![
            Value::Bytes(req.analysis_run_id.as_bytes().to_vec()),
            Value::Bytes(req.import_batch_id.as_bytes().to_vec()),
            opportunity_type.clone().map(|value| Value::Bytes(value.into_bytes())).unwrap_or(Value::NULL),
            opportunity_type.map(|value| Value::Bytes(value.into_bytes())).unwrap_or(Value::NULL),
            keyword.clone().map(|value| Value::Bytes(value.into_bytes())).unwrap_or(Value::NULL),
            keyword.clone().map(|value| Value::Bytes(value.into_bytes())).unwrap_or(Value::NULL),
            keyword.map(|value| Value::Bytes(value.into_bytes())).unwrap_or(Value::NULL),
        ];
        let rows = conn.exec_iter(query, Params::Positional(values))
            .map_err(|err| format!("failed to query opportunity candidates for export: {err}"))?;
        let mut exported_rows = 0_u64;
        for row in rows {
            let row = row.map_err(|err| format!("failed to decode opportunity export row: {err}"))?;
            let active_days: i64 = row.get(4).unwrap_or_default();
            let observation_rows: i64 = row.get(5).unwrap_or_default();
            let total_download_gb: f64 = row.get(6).unwrap_or_default();
            let total_effective_duration_hours: f64 = row.get(7).unwrap_or_default();
            let primary_app_active_days: i64 = row.get(15).unwrap_or_default();
            let primary_app_observations: i64 = row.get(16).unwrap_or_default();
            let rule_profile_version: i64 = row.get(21).unwrap_or_default();
            writer.write_record([
                row.get::<String, _>(0).unwrap_or_default(),
                row.get::<String, _>(1).unwrap_or_default(),
                row.get::<String, _>(2).unwrap_or_default(),
                row.get::<String, _>(3).unwrap_or_default(),
                active_days.to_string(), observation_rows.to_string(),
                total_download_gb.to_string(), total_effective_duration_hours.to_string(),
                export_number(row.get(8)), export_number(row.get(9)),
                export_number(row.get(10)), export_number(row.get(11)),
                export_number(row.get(12)), export_number(row.get(13)),
                export_string(row.get(14)), primary_app_active_days.to_string(),
                primary_app_observations.to_string(), export_number(row.get(17)),
                export_string(row.get(18)), row.get::<String, _>(19).unwrap_or_default(),
                export_string(row.get(20)), rule_profile_version.to_string(),
            ]).map_err(|err| format!("failed to write opportunity export row: {err}"))?;
            exported_rows += 1;
        }
        writer.flush().map_err(|err| format!("failed to flush opportunity export: {err}"))?;
        Ok(ack(format!(
            "opportunity candidates exported: rows={exported_rows}; path={}",
            req.output_path
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::rate;

    #[test]
    fn rates_preserve_unavailable_denominators() {
        assert_eq!(rate(1, 0), None);
        assert_eq!(rate(1, 4), Some(25.0));
    }
}

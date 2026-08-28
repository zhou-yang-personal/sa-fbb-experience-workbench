use mysql::prelude::*;
use mysql::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::batch_tables;
use crate::db;
use crate::models::{ack, CommandAck, DashboardRequest, EtlRequest, MetricCard};

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
    pub sufficient_app_users: i64,
    pub sufficient_app_observations: i64,
    pub attention_app_poor_rate_pct: f64,
    pub attention_app_persistent_user_rate_pct: f64,
    pub severe_app_poor_rate_pct: f64,
    pub severe_app_persistent_user_rate_pct: f64,
    pub severe_app_severe_user_rate_pct: f64,
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
        sufficient_app_users: row.get(19).unwrap_or_default(),
        sufficient_app_observations: row.get(20).unwrap_or_default(),
        attention_app_poor_rate_pct: row.get(21).unwrap_or_default(),
        attention_app_persistent_user_rate_pct: row.get(22).unwrap_or_default(),
        severe_app_poor_rate_pct: row.get(23).unwrap_or_default(),
        severe_app_persistent_user_rate_pct: row.get(24).unwrap_or_default(),
        severe_app_severe_user_rate_pct: row.get(25).unwrap_or_default(),
        notes: row.get(26),
        updated_at: row.get(27).unwrap_or_default(),
    })
}

const RULE_SELECT: &str = "SELECT rule_profile_id,version,profile_name,status,minimum_user_observations,minimum_app_users,minimum_app_observations,CAST(persistent_poor_rate_pct AS DOUBLE),CAST(problem_app_poor_rate_pct AS DOUBLE),CAST(problem_app_persistent_user_rate_pct AS DOUBLE),CAST(heavy_traffic_gb AS DOUBLE),CAST(heavy_usage_hours AS DOUBLE),CAST(peak_hour_start AS SIGNED),CAST(peak_hour_end AS SIGNED),CAST(migration_min_traffic_gb AS DOUBLE),CAST(speed_upgrade_min_traffic_gb AS DOUBLE),CAST(speed_upgrade_max_effective_mbps AS DOUBLE),CAST(mesh_min_wifi_delay_ms AS DOUBLE),CAST(app_bundle_min_observations AS SIGNED),CAST(sufficient_app_users AS SIGNED),CAST(sufficient_app_observations AS SIGNED),CAST(attention_app_poor_rate_pct AS DOUBLE),CAST(attention_app_persistent_user_rate_pct AS DOUBLE),CAST(severe_app_poor_rate_pct AS DOUBLE),CAST(severe_app_persistent_user_rate_pct AS DOUBLE),CAST(severe_app_severe_user_rate_pct AS DOUBLE),notes,DATE_FORMAT(updated_at,'%Y-%m-%d %H:%i:%s') FROM meta_decision_rule_profile";

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
    conn.exec_drop(format!("INSERT INTO meta_decision_rule_profile (rule_profile_id,version,profile_name,status,minimum_user_observations,minimum_app_users,minimum_app_observations,persistent_poor_rate_pct,problem_app_poor_rate_pct,problem_app_persistent_user_rate_pct,heavy_traffic_gb,heavy_usage_hours,peak_hour_start,peak_hour_end,migration_min_traffic_gb,speed_upgrade_min_traffic_gb,speed_upgrade_max_effective_mbps,mesh_min_wifi_delay_ms,app_bundle_min_observations,sufficient_app_users,sufficient_app_observations,attention_app_poor_rate_pct,attention_app_persistent_user_rate_pct,severe_app_poor_rate_pct,severe_app_persistent_user_rate_pct,severe_app_severe_user_rate_pct,rule_snapshot,notes) SELECT ?,COALESCE((SELECT MAX(version)+1 FROM (SELECT version FROM meta_decision_rule_profile) versions),1),CONCAT(profile_name,' copy'),'draft',minimum_user_observations,minimum_app_users,minimum_app_observations,persistent_poor_rate_pct,problem_app_poor_rate_pct,problem_app_persistent_user_rate_pct,heavy_traffic_gb,heavy_usage_hours,peak_hour_start,peak_hour_end,migration_min_traffic_gb,speed_upgrade_min_traffic_gb,speed_upgrade_max_effective_mbps,mesh_min_wifi_delay_ms,app_bundle_min_observations,sufficient_app_users,sufficient_app_observations,attention_app_poor_rate_pct,attention_app_persistent_user_rate_pct,severe_app_poor_rate_pct,severe_app_persistent_user_rate_pct,severe_app_severe_user_rate_pct,rule_snapshot,'Draft cloned from latest published profile' FROM meta_decision_rule_profile WHERE status='published' ORDER BY version DESC LIMIT 1"), (&id,)).map_err(|err| format!("failed to create decision rule draft: {err}"))?;
    Ok(ack(format!("decision rule draft created: {id}")))
}

#[tauri::command]
pub fn decision_rule_update(settings: crate::models::MySqlSettings, rule: DecisionRuleProfileRow) -> Result<CommandAck, String> {
    if rule.status != "draft" { return Err("only a draft decision rule can be edited".to_string()); }
    if rule.minimum_user_observations < 1 || rule.minimum_app_users < 1 || rule.minimum_app_observations < 1 || rule.sufficient_app_users < rule.minimum_app_users || rule.sufficient_app_observations < rule.minimum_app_observations { return Err("sample thresholds must be positive and sufficient must be at least limited".to_string()); }
    let rates = [rule.persistent_poor_rate_pct, rule.attention_app_poor_rate_pct, rule.attention_app_persistent_user_rate_pct, rule.problem_app_poor_rate_pct, rule.problem_app_persistent_user_rate_pct, rule.severe_app_poor_rate_pct, rule.severe_app_persistent_user_rate_pct, rule.severe_app_severe_user_rate_pct];
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
    let mut conn = db::conn(&settings)?;
    conn.exec_drop("UPDATE meta_decision_rule_profile SET profile_name=:name,minimum_user_observations=:min_user_obs,minimum_app_users=:min_app_users,minimum_app_observations=:min_app_obs,sufficient_app_users=:sufficient_users,sufficient_app_observations=:sufficient_obs,persistent_poor_rate_pct=:persistent_rate,attention_app_poor_rate_pct=:attention_poor,attention_app_persistent_user_rate_pct=:attention_persistent,problem_app_poor_rate_pct=:app_poor_rate,problem_app_persistent_user_rate_pct=:app_persistent_rate,severe_app_poor_rate_pct=:severe_poor,severe_app_persistent_user_rate_pct=:severe_persistent,severe_app_severe_user_rate_pct=:severe_user,heavy_traffic_gb=:heavy_traffic,heavy_usage_hours=:heavy_hours,peak_hour_start=:peak_start,peak_hour_end=:peak_end,migration_min_traffic_gb=:migration_traffic,speed_upgrade_min_traffic_gb=:upgrade_traffic,speed_upgrade_max_effective_mbps=:upgrade_rate,mesh_min_wifi_delay_ms=:mesh_delay,app_bundle_min_observations=:bundle_obs,notes=:notes,rule_snapshot=JSON_OBJECT('minimum_user_observations',:min_user_obs,'minimum_app_users',:min_app_users,'minimum_app_observations',:min_app_obs,'sufficient_app_users',:sufficient_users,'sufficient_app_observations',:sufficient_obs,'persistent_poor_rate_pct',:persistent_rate,'attention_app_poor_rate_pct',:attention_poor,'attention_app_persistent_user_rate_pct',:attention_persistent,'problem_app_poor_rate_pct',:app_poor_rate,'problem_app_persistent_user_rate_pct',:app_persistent_rate,'severe_app_poor_rate_pct',:severe_poor,'severe_app_persistent_user_rate_pct',:severe_persistent,'severe_app_severe_user_rate_pct',:severe_user,'heavy_traffic_gb',:heavy_traffic,'heavy_usage_hours',:heavy_hours,'peak_hour_start',:peak_start,'peak_hour_end',:peak_end,'migration_min_traffic_gb',:migration_traffic,'speed_upgrade_min_traffic_gb',:upgrade_traffic,'speed_upgrade_max_effective_mbps',:upgrade_rate,'mesh_min_wifi_delay_ms',:mesh_delay,'app_bundle_min_observations',:bundle_obs) WHERE rule_profile_id=:id AND status='draft'", params! {
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
        "bundle_obs" => rule.app_bundle_min_observations,
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
            "WITH rules AS (SELECT p.minimum_user_observations,p.persistent_poor_rate_pct FROM meta_decision_rule_profile p JOIN meta_analysis_run_decision_binding b ON b.rule_profile_id=p.rule_profile_id AND b.rule_profile_version=p.version WHERE b.analysis_run_id=?), users AS (SELECT user_key,SUM(total_download_gb) traffic_gb,SUM(total_game_hours) game_hours,SUM(observation_rows) observation_rows,SUM(valid_obs_rows) valid_obs_rows,SUM(poor_obs_rows) poor_obs_rows,MAX(CASE WHEN valid_obs_rows>=r.minimum_user_observations AND poor_obs_rows*100.0/NULLIF(valid_obs_rows,0)>=r.persistent_poor_rate_pct THEN 1 ELSE 0 END) persistent_flag,MAX(severe_poor_user_flag) severe_flag FROM `{table}` CROSS JOIN rules r WHERE analysis_run_id=? GROUP BY user_key,r.minimum_user_observations,r.persistent_poor_rate_pct), metrics AS (SELECT COUNT(*) users,COALESCE(SUM(traffic_gb),0) traffic_gb,COALESCE(SUM(game_hours),0) game_hours,COALESCE(SUM(observation_rows),0) observation_rows,COALESCE(SUM(valid_obs_rows),0) valid_obs_rows,COALESCE(SUM(poor_obs_rows),0) poor_obs_rows,COALESCE(SUM(persistent_flag),0) persistent_users,COALESCE(SUM(severe_flag),0) severe_users FROM users), quality AS (SELECT AVG(avg_effective_download_mbps) effective_mbps,AVG(avg_subscriber_rtt_ms) subscriber_rtt,AVG(avg_network_rtt_ms) network_rtt,AVG(avg_user_loss_pct) user_loss,AVG(avg_network_loss_pct) network_loss FROM `{table}` WHERE analysis_run_id=?) SELECT CAST(users AS SIGNED),CAST(traffic_gb AS DOUBLE),CAST(game_hours AS DOUBLE),CAST(observation_rows AS SIGNED),CAST(valid_obs_rows AS SIGNED),CAST(poor_obs_rows AS SIGNED),CAST(persistent_users AS SIGNED),CAST(severe_users AS SIGNED),CAST(effective_mbps AS DOUBLE),CAST(subscriber_rtt AS DOUBLE),CAST(network_rtt AS DOUBLE),CAST(user_loss AS DOUBLE),CAST(network_loss AS DOUBLE) FROM metrics CROSS JOIN quality"
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
            ("游戏时长", format!("{game_hours:.2}"), format!("metric=duration, value={game_hours:.6}, unit=hours, sample_size={users}, availability=PARTIAL_GAME_ONLY, limitation=TCP时长将在新聚合字段可用后补齐")),
            ("有效观测", valid.to_string(), format!("metric=observations, numerator={valid}, denominator={obs}, sample_size={obs}, unit=rows, availability=AVAILABLE")),
            ("差体验观测占比", fmt_optional(rate(poor, valid), 2), format!("metric=poor_observation_rate, numerator={poor}, denominator={valid}, sample_size={valid}, unit=percent, availability={}", if valid > 0 { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("持续质差用户占比", fmt_optional(rate(persistent, users), 2), format!("metric=persistent_user_rate, numerator={persistent}, denominator={users}, sample_size={users}, unit=percent, availability={}", if users > 0 { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("严重质差用户占比", fmt_optional(rate(severe, users), 2), format!("metric=severe_user_rate, numerator={severe}, denominator={users}, sample_size={users}, unit=percent, availability={}", if users > 0 { "AVAILABLE" } else { "UNAVAILABLE" })),
            ("视频有效下载速率", fmt_optional(row_f64(&row, 8), 2), format!("metric=effective_download_rate, unit=Mbps, sample_size={valid}, availability={}", if row_f64(&row, 8).is_some() { "AVAILABLE" } else { "UNAVAILABLE" })),
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
            "WITH rules AS (SELECT p.* FROM meta_decision_rule_profile p JOIN meta_analysis_run_decision_binding b ON b.rule_profile_id=p.rule_profile_id AND b.rule_profile_version=p.version WHERE b.analysis_run_id=?), apps AS (SELECT app_name,MAX(app_category) app_category,COUNT(DISTINCT user_key) observed_users,SUM(observation_rows) observation_rows,SUM(valid_obs_rows) valid_obs_rows,SUM(poor_obs_rows) poor_obs_rows,COUNT(DISTINCT CASE WHEN valid_obs_rows>=r.minimum_user_observations THEN user_key END) eligible_users,COUNT(DISTINCT CASE WHEN valid_obs_rows>=r.minimum_user_observations AND poor_obs_rows*100.0/NULLIF(valid_obs_rows,0)>=r.persistent_poor_rate_pct THEN user_key END) persistent_users,COUNT(DISTINCT CASE WHEN severe_poor_user_flag=1 THEN user_key END) severe_users,SUM(total_download_gb) traffic_gb,SUM(total_game_hours) duration_hours,AVG(avg_effective_download_mbps) effective_mbps,AVG(avg_subscriber_rtt_ms) subscriber_rtt,AVG(avg_network_rtt_ms) network_rtt,AVG(avg_user_loss_pct) user_loss,AVG(avg_network_loss_pct) network_loss FROM `{table}` CROSS JOIN rules r WHERE analysis_run_id=? AND (? IS NULL OR app_name LIKE ? OR app_category LIKE ?) GROUP BY app_name,r.minimum_user_observations,r.persistent_poor_rate_pct) SELECT a.app_name,a.app_category,CAST(a.observed_users AS SIGNED),CAST(a.eligible_users AS SIGNED),CAST(a.observation_rows AS SIGNED),CAST(a.valid_obs_rows AS SIGNED),CAST(a.poor_obs_rows AS SIGNED),CAST(a.persistent_users AS SIGNED),CAST(a.severe_users AS SIGNED),CAST(a.traffic_gb AS DOUBLE),CAST(a.duration_hours AS DOUBLE),CAST(a.effective_mbps AS DOUBLE),CAST(a.subscriber_rtt AS DOUBLE),CAST(a.network_rtt AS DOUBLE),CAST(a.user_loss AS DOUBLE),CAST(a.network_loss AS DOUBLE),r.minimum_app_users,r.minimum_app_observations,r.sufficient_app_users,r.sufficient_app_observations,r.attention_app_poor_rate_pct,r.attention_app_persistent_user_rate_pct,r.problem_app_poor_rate_pct,r.problem_app_persistent_user_rate_pct,r.severe_app_poor_rate_pct,r.severe_app_persistent_user_rate_pct,r.severe_app_severe_user_rate_pct,r.version FROM apps a CROSS JOIN rules r ORDER BY a.observed_users DESC,a.traffic_gb DESC LIMIT ? OFFSET ?"
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
        ensure_run_source(&req, &table)?;
        let mut conn = db::conn(&req.settings)?;
        let run_id = req.run_id();
        require_rule_binding(&mut conn, &run_id)?;
        let keyword = req.keyword_like();
        let sql = format!(
            "WITH rules AS (SELECT p.minimum_user_observations,p.persistent_poor_rate_pct FROM meta_decision_rule_profile p JOIN meta_analysis_run_decision_binding b ON b.rule_profile_id=p.rule_profile_id AND b.rule_profile_version=p.version WHERE b.analysis_run_id=?), users AS (SELECT user_key,SUM(total_download_gb) traffic_gb,SUM(total_game_hours) game_hours,SUM(observation_rows) observations,AVG(avg_effective_download_mbps) effective_mbps,AVG(avg_subscriber_rtt_ms) subscriber_rtt,AVG(avg_network_rtt_ms) network_rtt,AVG(avg_user_loss_pct) user_loss,AVG(avg_network_loss_pct) network_loss,MAX(CASE WHEN valid_obs_rows>=r.minimum_user_observations AND poor_obs_rows*100.0/NULLIF(valid_obs_rows,0)>=r.persistent_poor_rate_pct THEN 1 ELSE 0 END) persistent_flag FROM `{table}` CROSS JOIN rules r WHERE analysis_run_id=? AND (? IS NULL OR app_name LIKE ?) GROUP BY user_key,r.minimum_user_observations,r.persistent_poor_rate_pct), bands AS (SELECT 'TRAFFIC' dimension,CASE WHEN traffic_gb<1 THEN '<1 GB' WHEN traffic_gb<5 THEN '1–5 GB' WHEN traffic_gb<20 THEN '5–20 GB' WHEN traffic_gb<50 THEN '20–50 GB' ELSE '≥50 GB' END band,COUNT(*) users FROM users GROUP BY 2 UNION ALL SELECT 'DURATION_GAME_ONLY',CASE WHEN game_hours=0 THEN '0 h / 未导入游戏时长' WHEN game_hours<1 THEN '<1 h' WHEN game_hours<5 THEN '1–5 h' WHEN game_hours<10 THEN '5–10 h' ELSE '≥10 h' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'OBSERVATIONS',CASE WHEN observations<3 THEN '<3 条' WHEN observations<10 THEN '3–10 条' WHEN observations<50 THEN '10–50 条' WHEN observations<200 THEN '50–200 条' ELSE '≥200 条' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'EFFECTIVE_RATE',CASE WHEN effective_mbps IS NULL THEN '不可用' WHEN effective_mbps<5 THEN '<5 Mbps' WHEN effective_mbps<20 THEN '5–20 Mbps' WHEN effective_mbps<50 THEN '20–50 Mbps' WHEN effective_mbps<100 THEN '50–100 Mbps' ELSE '≥100 Mbps' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'SUBSCRIBER_RTT',CASE WHEN subscriber_rtt IS NULL THEN '不可用' WHEN subscriber_rtt<20 THEN '<20 ms' WHEN subscriber_rtt<50 THEN '20–50 ms' WHEN subscriber_rtt<100 THEN '50–100 ms' WHEN subscriber_rtt<200 THEN '100–200 ms' ELSE '≥200 ms' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'NETWORK_RTT',CASE WHEN network_rtt IS NULL THEN '不可用' WHEN network_rtt<20 THEN '<20 ms' WHEN network_rtt<50 THEN '20–50 ms' WHEN network_rtt<100 THEN '50–100 ms' WHEN network_rtt<200 THEN '100–200 ms' ELSE '≥200 ms' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'USER_LOSS',CASE WHEN user_loss IS NULL THEN '不可用' WHEN user_loss<0.1 THEN '<0.1%' WHEN user_loss<1 THEN '0.1–1%' WHEN user_loss<3 THEN '1–3%' ELSE '≥3%' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'NETWORK_LOSS',CASE WHEN network_loss IS NULL THEN '不可用' WHEN network_loss<0.1 THEN '<0.1%' WHEN network_loss<1 THEN '0.1–1%' WHEN network_loss<3 THEN '1–3%' ELSE '≥3%' END,COUNT(*) FROM users GROUP BY 2 UNION ALL SELECT 'EXPERIENCE',CASE WHEN persistent_flag=1 THEN '持续质差用户' ELSE '非持续质差用户' END,COUNT(*) FROM users GROUP BY 2) SELECT dimension,band,CAST(users AS SIGNED) FROM bands ORDER BY dimension,users DESC"
        );
        let rows = conn.exec_iter(sql, (&run_id, &run_id, keyword.clone(), keyword)).map_err(|err| format!("failed to query user distributions: {err}"))?;
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
        let table = source_table(&req)?;
        ensure_run_source(&req, &table)?;
        let mut conn = db::conn(&req.settings)?;
        let sql = format!(
            "SELECT user_type,COUNT(DISTINCT user_key) users,SUM(valid_obs_rows) valid_obs,SUM(poor_obs_rows) poor_obs,COUNT(DISTINCT CASE WHEN persistent_poor_user_flag=1 THEN user_key END) persistent_users,SUM(total_download_gb) traffic_gb,AVG(avg_effective_download_mbps) effective_mbps,AVG(avg_subscriber_rtt_ms) subscriber_rtt,AVG(avg_network_rtt_ms) network_rtt,AVG(avg_user_loss_pct) user_loss,AVG(avg_network_loss_pct) network_loss FROM `{table}` WHERE analysis_run_id=? GROUP BY user_type ORDER BY users DESC"
        );
        let rows = conn.exec_iter(sql, (req.run_id(),)).map_err(|err| format!("failed to query access compare: {err}"))?;
        rows.map(|row| {
            let row = row.map_err(|err| format!("failed to decode access compare: {err}"))?;
            let access = row_string(&row, 0, "UNAVAILABLE");
            let users = row_i64(&row, 1);
            let valid = row_i64(&row, 2);
            let poor = row_i64(&row, 3);
            let persistent = row_i64(&row, 4);
            Ok(MetricCard { label: access.clone(), value: users.to_string(), hint: format!("access_type={access}, users={users}, valid_obs_rows={valid}, poor_obs_rows={poor}, poor_observation_rate_pct={}, persistent_poor_users={persistent}, persistent_poor_user_rate_pct={}, traffic_gb={}, effective_download_mbps={}, subscriber_rtt_ms={}, network_rtt_ms={}, user_loss_pct={}, network_loss_pct={}", fmt_optional(rate(poor, valid),4), fmt_optional(rate(persistent,users),4), fmt_optional(row_f64(&row,5),4), fmt_optional(row_f64(&row,6),4), fmt_optional(row_f64(&row,7),4), fmt_optional(row_f64(&row,8),4), fmt_optional(row_f64(&row,9),6), fmt_optional(row_f64(&row,10),6)) })
        }).collect()
    })
}

pub(crate) fn materialize_opportunities(req: EtlRequest) -> Result<CommandAck, String> {
    crate::migrations::ensure_decision_workspace_schema(&req.settings)?;
    let run_id = req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let source = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_app_period_experience_v2")?;
    let mut conn = db::conn(&req.settings)?;
    if !batch_tables::table_has_analysis_run(&mut conn, &source, &run_id)? {
        return Err(format!("opportunity source not ready for analysis_run_id={run_id}"));
    }
    let tx = conn.start_transaction(mysql::TxOpts::default()).map_err(|err| format!("failed to start opportunity transaction: {err}"))?;
    let mut tx = tx;
    tx.exec_drop("DELETE FROM ads_opportunity_summary_v3 WHERE analysis_run_id=?", (&run_id,)).map_err(|err| format!("failed to clear opportunity summary: {err}"))?;
    tx.exec_drop("DELETE FROM ads_opportunity_user_v3 WHERE analysis_run_id=?", (&run_id,)).map_err(|err| format!("failed to clear opportunity users: {err}"))?;
    tx.exec_drop("INSERT IGNORE INTO meta_analysis_run_decision_binding (analysis_run_id,import_batch_id,rule_profile_id,rule_profile_version,rule_snapshot) SELECT ?,?,rule_profile_id,version,COALESCE(rule_snapshot,JSON_OBJECT()) FROM meta_decision_rule_profile WHERE status='published' ORDER BY version DESC LIMIT 1", (&run_id,&req.import_batch_id)).map_err(|err| format!("failed to bind decision rules: {err}"))?;
    let insert_sql = format!(
        "INSERT INTO ads_opportunity_user_v3 (analysis_run_id,import_batch_id,user_key,opportunity_type,opportunity_level,evidence_value,evidence_unit,evidence_summary,data_limitation_code,rule_profile_version) WITH rules AS (SELECT p.* FROM meta_decision_rule_profile p JOIN meta_analysis_run_decision_binding b ON b.rule_profile_id=p.rule_profile_id AND b.rule_profile_version=p.version WHERE b.analysis_run_id=?), users AS (SELECT user_key,MAX(user_type) user_type,SUM(total_download_gb) traffic_gb,SUM(total_game_hours) usage_hours,SUM(observation_rows) observations,AVG(avg_effective_download_mbps) effective_mbps,MAX(persistent_poor_user_flag) persistent_flag FROM `{source}` WHERE analysis_run_id=? GROUP BY user_key), bundles AS (SELECT user_key,MAX(app_name) app_name,SUM(observation_rows) observations,ROW_NUMBER() OVER(PARTITION BY user_key ORDER BY SUM(observation_rows) DESC,MAX(app_name)) rn FROM `{source}` WHERE analysis_run_id=? GROUP BY user_key,app_name) SELECT ?,?,u.user_key,'MIGRATION',CASE WHEN u.traffic_gb>=r.heavy_traffic_gb THEN 'HIGH' ELSE 'STANDARD' END,u.traffic_gb,'GB',CONCAT('Cable user; traffic=',ROUND(u.traffic_gb,2),' GB'),NULL,r.version FROM users u CROSS JOIN rules r WHERE u.user_type='CABLE' AND u.traffic_gb>=r.migration_min_traffic_gb UNION ALL SELECT ?,?,u.user_key,'SPEED_UPGRADE',CASE WHEN u.traffic_gb>=r.heavy_traffic_gb*2 THEN 'HIGH' ELSE 'STANDARD' END,u.traffic_gb,'GB',CONCAT('heavy traffic=',ROUND(u.traffic_gb,2),' GB; effective rate=',COALESCE(ROUND(u.effective_mbps,2),'NA'),' Mbps'),NULL,r.version FROM users u CROSS JOIN rules r WHERE u.traffic_gb>=r.speed_upgrade_min_traffic_gb AND u.effective_mbps IS NOT NULL AND u.effective_mbps<=r.speed_upgrade_max_effective_mbps UNION ALL SELECT ?,?,u.user_key,'APP_BUNDLE',CASE WHEN b.observations>=r.app_bundle_min_observations*5 THEN 'HIGH' ELSE 'STANDARD' END,b.observations,'observations',CONCAT('frequent app=',b.app_name,'; observations=',b.observations),NULL,r.version FROM users u JOIN bundles b ON b.user_key=u.user_key AND b.rn=1 CROSS JOIN rules r WHERE b.observations>=r.app_bundle_min_observations"
    );
    tx.exec_drop(insert_sql, (&run_id,&run_id,&run_id,&run_id,&req.import_batch_id,&run_id,&req.import_batch_id,&run_id,&req.import_batch_id)).map_err(|err| format!("failed to materialize opportunity users: {err}"))?;
    tx.exec_drop("INSERT INTO ads_opportunity_summary_v3 (analysis_run_id,import_batch_id,opportunity_type,candidate_users,high_priority_users,total_evidence_value,evidence_unit,availability_status,data_limitation_code,rule_profile_version) SELECT analysis_run_id,import_batch_id,opportunity_type,COUNT(*),SUM(opportunity_level='HIGH'),SUM(evidence_value),MAX(evidence_unit),'AVAILABLE',NULL,MAX(rule_profile_version) FROM ads_opportunity_user_v3 WHERE analysis_run_id=? GROUP BY analysis_run_id,import_batch_id,opportunity_type", (&run_id,)).map_err(|err| format!("failed to summarize opportunities: {err}"))?;
    tx.exec_drop("INSERT INTO ads_opportunity_summary_v3 (analysis_run_id,import_batch_id,opportunity_type,candidate_users,high_priority_users,total_evidence_value,evidence_unit,availability_status,data_limitation_code,rule_profile_version) SELECT ?,?,'MESH_AP',0,0,NULL,NULL,'UNAVAILABLE','WIFI_DELAY_NOT_AGGREGATED',rule_profile_version FROM meta_analysis_run_decision_binding WHERE analysis_run_id=?", (&run_id,&req.import_batch_id,&run_id)).map_err(|err| format!("failed to record mesh availability: {err}"))?;
    tx.commit().map_err(|err| format!("failed to commit opportunities: {err}"))?;
    Ok(ack(format!("decision opportunities materialized: analysis_run_id={run_id}")))
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

#[cfg(test)]
mod tests {
    use super::rate;

    #[test]
    fn rates_preserve_unavailable_denominators() {
        assert_eq!(rate(1, 0), None);
        assert_eq!(rate(1, 4), Some(25.0));
    }
}

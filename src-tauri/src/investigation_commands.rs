use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use mysql::prelude::*;
use mysql::{Params, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MySqlSettings};

#[derive(Debug, Clone, Serialize)]
pub struct ExperienceStatusV2 {
    analysis_run_id: String,
    import_batch_id: String,
    eligible_users: i64,
    valid_observations: i64,
    poor_observations: i64,
    poor_observation_rate_pct: Option<f64>,
    ever_affected_users: i64,
    ever_affected_user_rate_pct: Option<f64>,
    persistent_poor_users: i64,
    persistent_poor_user_rate_pct: Option<f64>,
    severe_poor_users: i64,
    severe_poor_user_rate_pct: Option<f64>,
    policy_id: String,
    policy_version: i64,
    sample_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperienceFinding {
    finding_id: String,
    finding_type: String,
    title_zh: String,
    title_en: String,
    app_category: Option<String>,
    app_name: Option<String>,
    access_type: Option<String>,
    issue_metric: Option<String>,
    issue_side: Option<String>,
    baseline_type: String,
    numerator: i64,
    denominator: i64,
    sample_size: i64,
    affected_users: i64,
    affected_user_rate_pct: Option<f64>,
    poor_observation_rate_pct: Option<f64>,
    severe_user_rate_pct: Option<f64>,
    severity: String,
    confidence: String,
    main_driver: Option<String>,
    evidence_summary: String,
    data_limitations: Option<String>,
    recommended_next_step: String,
    rule_version: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataCoverageItemV2 {
    dimension: String,
    status: String,
    available_rows: i64,
    total_rows: i64,
    coverage_pct: Option<f64>,
    limitation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunVerificationV2 {
    current_analysis_run_id: String,
    current_import_batch_id: String,
    previous_analysis_run_id: Option<String>,
    previous_import_batch_id: Option<String>,
    comparable: bool,
    comparison_reason: String,
    current_poor_observation_rate_pct: Option<f64>,
    previous_poor_observation_rate_pct: Option<f64>,
    poor_observation_rate_delta_pct: Option<f64>,
    current_persistent_poor_user_rate_pct: Option<f64>,
    previous_persistent_poor_user_rate_pct: Option<f64>,
    persistent_poor_user_rate_delta_pct: Option<f64>,
    current_severe_poor_user_rate_pct: Option<f64>,
    previous_severe_poor_user_rate_pct: Option<f64>,
    severe_poor_user_rate_delta_pct: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvestigationQueryRequest {
    settings: MySqlSettings,
    import_batch_id: String,
    analysis_run_id: String,
    app_category: Option<String>,
    app_name: Option<String>,
    access_type: Option<String>,
    user_key: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    hour_from: Option<i32>,
    hour_to: Option<i32>,
    page_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvestigationEvidenceRow {
    user_key: String,
    access_type: String,
    app_category: String,
    app_name: String,
    valid_obs_rows: i64,
    poor_obs_rows: i64,
    poor_observation_rate_pct: Option<f64>,
    persistent_poor_user: bool,
    severe_poor_user: bool,
    avg_vmos: Option<f64>,
    avg_subscriber_rtt_ms: Option<f64>,
    avg_network_rtt_ms: Option<f64>,
    avg_user_loss_pct: Option<f64>,
    avg_network_loss_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvestigationHourlyRow {
    stat_date: String,
    hour_of_day: i32,
    access_type: String,
    eligible_users: i64,
    valid_obs_rows: i64,
    poor_obs_rows: i64,
    poor_observation_rate_pct: Option<f64>,
    persistent_poor_users: i64,
    severe_poor_users: i64,
    sample_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvestigationServerIpRow {
    server_ip: String,
    observed_users: i64,
    observation_rows: i64,
    avg_subscriber_rtt_ms: Option<f64>,
    avg_network_rtt_ms: Option<f64>,
    avg_user_loss_pct: Option<f64>,
    avg_network_loss_pct: Option<f64>,
}

#[derive(Default)]
struct ServerIpAccumulator {
    users: HashSet<String>,
    observation_rows: i64,
    subscriber_rtt_sum: f64,
    subscriber_rtt_count: i64,
    network_rtt_sum: f64,
    network_rtt_count: i64,
    user_loss_sum: f64,
    user_loss_count: i64,
    network_loss_sum: f64,
    network_loss_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveInvestigationRequest {
    settings: MySqlSettings,
    import_batch_id: String,
    analysis_run_id: String,
    finding_id: Option<String>,
    title: String,
    status: Option<String>,
    context_json: String,
    notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvestigationListRequest {
    settings: MySqlSettings,
    import_batch_id: String,
    analysis_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SavedInvestigation {
    investigation_id: String,
    import_batch_id: String,
    analysis_run_id: String,
    finding_id: Option<String>,
    title: String,
    status: String,
    context_json: String,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty() && item != "ALL")
}

// mysql_common's Row::get uses the panicking FromValue conversion. Historical
// result tables can contain NULL even when the latest schema declares a column
// NOT NULL, so investigation reads must always use the fallible conversion.
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

fn row_i32(row: &mysql::Row, index: usize) -> i32 {
    row.get_opt::<Option<i32>, _>(index)
        .and_then(Result::ok)
        .flatten()
        .unwrap_or_default()
}

fn row_i8(row: &mysql::Row, index: usize) -> i8 {
    row.get_opt::<Option<i8>, _>(index)
        .and_then(Result::ok)
        .flatten()
        .unwrap_or_default()
}

fn row_f64(row: &mysql::Row, index: usize) -> Option<f64> {
    row.get_opt::<Option<f64>, _>(index)
        .and_then(Result::ok)
        .flatten()
}

#[tauri::command]
pub fn analytics_get_experience_status_v2(
    req: DashboardRequest,
) -> Result<ExperienceStatusV2, String> {
    crate::migrations::ensure_experience_policy_schema(&req.settings)?;
    let run_id = req.run_id();
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_app_period_experience_v2",
    )?;
    let mut conn = db::conn(&req.settings)?;
    type StatusTuple = (
        i64,
        i64,
        i64,
        Option<f64>,
        i64,
        Option<f64>,
        i64,
        Option<f64>,
        i64,
        Option<f64>,
    );
    let row: Option<StatusTuple> = conn.exec_first(
        format!("SELECT CAST(COUNT(DISTINCT CASE WHEN eligible_user_flag=1 THEN user_key END) AS SIGNED), CAST(COALESCE(SUM(valid_obs_rows),0) AS SIGNED), CAST(COALESCE(SUM(poor_obs_rows),0) AS SIGNED), CAST(ROUND(SUM(poor_obs_rows)*100.0/NULLIF(SUM(valid_obs_rows),0),6) AS DOUBLE), CAST(COUNT(DISTINCT CASE WHEN ever_affected_user_flag=1 THEN user_key END) AS SIGNED), CAST(ROUND(COUNT(DISTINCT CASE WHEN ever_affected_user_flag=1 THEN user_key END)*100.0/NULLIF(COUNT(DISTINCT CASE WHEN eligible_user_flag=1 THEN user_key END),0),6) AS DOUBLE), CAST(COUNT(DISTINCT CASE WHEN persistent_poor_user_flag=1 THEN user_key END) AS SIGNED), CAST(ROUND(COUNT(DISTINCT CASE WHEN persistent_poor_user_flag=1 THEN user_key END)*100.0/NULLIF(COUNT(DISTINCT CASE WHEN eligible_user_flag=1 THEN user_key END),0),6) AS DOUBLE), CAST(COUNT(DISTINCT CASE WHEN severe_poor_user_flag=1 THEN user_key END) AS SIGNED), CAST(ROUND(COUNT(DISTINCT CASE WHEN severe_poor_user_flag=1 THEN user_key END)*100.0/NULLIF(COUNT(DISTINCT CASE WHEN eligible_user_flag=1 THEN user_key END),0),6) AS DOUBLE) FROM `{table}` WHERE analysis_run_id=?"),
        (&run_id,),
    ).map_err(|err| format!("failed to query V2 experience status: {err}"))?;
    let (
        eligible,
        valid,
        poor,
        poor_rate,
        ever,
        ever_rate,
        persistent,
        persistent_rate,
        severe,
        severe_rate,
    ) = row.unwrap_or((0, 0, 0, None, 0, None, 0, None, 0, None));
    let policy: Option<(String, i64)> = conn.exec_first(
        "SELECT experience_policy_id, experience_policy_version FROM meta_analysis_run_policy_binding WHERE analysis_run_id=? AND import_batch_id=?",
        (&run_id, &req.import_batch_id),
    ).map_err(|err| format!("failed to query analysis policy binding: {err}"))?;
    let (policy_id, policy_version) = policy.unwrap_or_else(|| ("UNBOUND".into(), 0));
    Ok(ExperienceStatusV2 {
        analysis_run_id: run_id,
        import_batch_id: req.import_batch_id,
        eligible_users: eligible,
        valid_observations: valid,
        poor_observations: poor,
        poor_observation_rate_pct: poor_rate,
        ever_affected_users: ever,
        ever_affected_user_rate_pct: ever_rate,
        persistent_poor_users: persistent,
        persistent_poor_user_rate_pct: persistent_rate,
        severe_poor_users: severe,
        severe_poor_user_rate_pct: severe_rate,
        policy_id,
        policy_version,
        sample_status: if valid > 0 {
            "AVAILABLE".into()
        } else {
            "UNAVAILABLE".into()
        },
    })
}

#[tauri::command]
pub fn analytics_get_findings_v2(req: DashboardRequest) -> Result<Vec<ExperienceFinding>, String> {
    let run_id = req.run_id();
    let table =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_app_experience_v2")?;
    let mut conn = db::conn(&req.settings)?;
    let sql = format!("SELECT CONCAT('F_APP_',LEFT(HEX(a.grain_hash),20)), a.app_category, a.app_name, a.user_type, CAST(a.persistent_poor_users AS SIGNED), CAST(a.eligible_users AS SIGNED), CAST(a.valid_obs_rows AS SIGNED), CAST(a.persistent_poor_user_rate_pct AS DOUBLE), CAST(a.poor_observation_rate_pct AS DOUBLE), CAST(a.severe_poor_user_rate_pct AS DOUBLE), a.attention_level, COALESCE(a.main_issue_driver,''), COALESCE(a.evidence_summary,''), COALESCE(a.data_limitation_code,''), a.policy_version, CAST(COALESCE(p.minimum_app_eligible_users,30) AS SIGNED) FROM `{table}` a LEFT JOIN meta_analysis_run_policy_binding b ON b.analysis_run_id=a.analysis_run_id LEFT JOIN meta_experience_analysis_policy p ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version WHERE a.analysis_run_id=? AND a.sample_status='SUFFICIENT' AND a.attention_level IN ('ATTENTION','SEVERE') ORDER BY CASE a.attention_level WHEN 'SEVERE' THEN 0 ELSE 1 END, a.persistent_poor_user_rate_pct DESC, a.persistent_poor_users DESC LIMIT ?");
    let rows = conn
        .exec_iter(sql, (&run_id, req.page_size(80, 300)))
        .map_err(|err| format!("failed to query V2 findings: {err}"))?;
    let mut findings: Vec<ExperienceFinding> = rows.map(|row| {
        let row = row.map_err(|err| format!("failed to decode V2 finding: {err}"))?;
        let id = row_string(&row, 0, "F_APP_UNKNOWN");
        let category = row_string(&row, 1, "UNCLASSIFIED");
        let app = row_string(&row, 2, "UNCLASSIFIED");
        let access = row_string(&row, 3, "UNCLASSIFIED");
        let affected = row_i64(&row, 4);
        let denominator = row_i64(&row, 5);
        let sample = row_i64(&row, 6);
        let affected_rate = row_f64(&row, 7);
        let poor_rate = row_f64(&row, 8);
        let severe_rate = row_f64(&row, 9);
        let severity = row_string(&row, 10, "ATTENTION");
        let driver = row_string(&row, 11, "");
        let evidence = row_string(&row, 12, "");
        let limitation = row_string(&row, 13, "");
        let policy_version = row_i64(&row, 14);
        let min_users = row_i64(&row, 15).max(30);
        let issue_side = if driver.contains("network") { "NETWORK_SIDE" } else if driver.contains("subscriber") || driver.contains("user") || driver.contains("wifi") { "USER_SIDE" } else { "EVIDENCE_INSUFFICIENT" };
        let next_step = match issue_side { "NETWORK_SIDE" => "Validate network/path concentration before opening a network action.", "USER_SIDE" => "Review household/Wi-Fi evidence and affected users before recommending optimization.", _ => "Drill down by time, access type and affected users; current evidence is insufficient for root-cause confirmation." };
        Ok(ExperienceFinding { finding_id: id, finding_type: "PROBLEM_APP".into(), title_zh: format!("{app} 的 {access} 用户出现持续差体验"), title_en: format!("Persistent poor experience for {app} on {access}"), app_category: Some(category), app_name: Some(app), access_type: Some(access), issue_metric: Some(driver.clone()).filter(|item| !item.is_empty()), issue_side: Some(issue_side.into()), baseline_type: "POLICY_THRESHOLD".into(), numerator: affected, denominator, sample_size: sample, affected_users: affected, affected_user_rate_pct: affected_rate, poor_observation_rate_pct: poor_rate, severe_user_rate_pct: severe_rate, severity, confidence: if denominator >= min_users.saturating_mul(3) { "HIGH".into() } else { "MEDIUM".into() }, main_driver: Some(driver).filter(|item| !item.is_empty()), evidence_summary: evidence, data_limitations: Some(limitation).filter(|item| !item.is_empty()), recommended_next_step: next_step.into(), rule_version: policy_version })
    }).collect::<Result<Vec<_>, String>>()?;

    let gap_sql=format!("SELECT CONCAT('F_ACCESS_',LEFT(MD5(CONCAT_WS('|',c.analysis_run_id,c.app_category,c.app_name)),20)),c.app_category,c.app_name,CAST(c.persistent_poor_users AS SIGNED),CAST(c.eligible_users AS SIGNED),CAST(c.valid_obs_rows AS SIGNED),CAST(c.persistent_poor_user_rate_pct AS DOUBLE),CAST(c.poor_observation_rate_pct AS DOUBLE),CAST(c.severe_poor_user_rate_pct AS DOUBLE),CAST(f.persistent_poor_user_rate_pct AS DOUBLE),c.policy_version FROM `{table}` c JOIN `{table}` f ON f.analysis_run_id=c.analysis_run_id AND f.app_category=c.app_category AND f.app_name=c.app_name AND f.user_type='FTTH' AND f.sample_status='SUFFICIENT' JOIN meta_analysis_run_policy_binding b ON b.analysis_run_id=c.analysis_run_id JOIN meta_experience_analysis_policy p ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version WHERE c.analysis_run_id=? AND c.user_type='CABLE' AND c.sample_status='SUFFICIENT' AND c.persistent_poor_user_rate_pct-f.persistent_poor_user_rate_pct>=p.finding_attention_persistent_user_rate_pct ORDER BY c.persistent_poor_user_rate_pct-f.persistent_poor_user_rate_pct DESC LIMIT 60");
    let gap_rows = conn
        .exec_iter(gap_sql, (&run_id,))
        .map_err(|err| format!("failed to query Cable/FTTH gap findings: {err}"))?;
    for row in gap_rows {
        let row = row.map_err(|err| format!("failed to decode Cable/FTTH gap finding: {err}"))?;
        let finding_id = row_string(&row, 0, "F_ACCESS_UNKNOWN");
        let category = row_string(&row, 1, "UNCLASSIFIED");
        let app = row_string(&row, 2, "UNCLASSIFIED");
        let affected = row_i64(&row, 3);
        let denominator = row_i64(&row, 4);
        let sample = row_i64(&row, 5);
        let cable_rate = row_f64(&row, 6);
        let poor_rate = row_f64(&row, 7);
        let severe_rate = row_f64(&row, 8);
        let ftth_rate = row_f64(&row, 9);
        let version = row_i64(&row, 10);
        let delta = cable_rate.unwrap_or(0.0) - ftth_rate.unwrap_or(0.0);
        findings.push(ExperienceFinding { finding_id, finding_type:"ACCESS_GAP".into(), title_zh:format!("{app} 的 Cable 持续差体验率明显高于 FTTH"), title_en:format!("Cable persistent poor rate for {app} is materially above FTTH"), app_category:Some(category), app_name:Some(app), access_type:Some("CABLE".into()), issue_metric:Some("persistent_poor_user_rate_pct".into()), issue_side:Some("EVIDENCE_INSUFFICIENT".into()), baseline_type:"FTTH_PEER".into(), numerator:affected, denominator, sample_size:sample, affected_users:affected, affected_user_rate_pct:cable_rate, poor_observation_rate_pct:poor_rate, severe_user_rate_pct:severe_rate, severity:if delta>=20.0{"SEVERE".into()}else{"ATTENTION".into()}, confidence:"HIGH".into(), main_driver:Some("ACCESS_TYPE_GAP".into()), evidence_summary:format!("Cable persistent rate={:.4}%; FTTH peer={:.4}%; delta={delta:.4}pct; same app and analysis run",cable_rate.unwrap_or(0.0),ftth_rate.unwrap_or(0.0)), data_limitations:Some("Access association is evidence, not proof that Cable technology is the root cause.".into()), recommended_next_step:"Drill down by hour and affected users, then validate whether the gap persists under comparable traffic and time scope.".into(), rule_version:version });
    }
    findings.sort_by(|a, b| {
        let rank = |value: &str| if value == "SEVERE" { 0 } else { 1 };
        rank(&a.severity).cmp(&rank(&b.severity)).then_with(|| {
            b.affected_user_rate_pct
                .unwrap_or(0.0)
                .partial_cmp(&a.affected_user_rate_pct.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    findings.truncate(req.page_size(80, 300) as usize);
    Ok(findings)
}

#[tauri::command]
pub fn analytics_get_data_coverage_v2(
    req: DashboardRequest,
) -> Result<Vec<DataCoverageItemV2>, String> {
    let mut conn = db::conn(&req.settings)?;
    let raw_tcp =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "raw_tcp_detail_import")?;
    let raw_game = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "raw_game_detail_import",
    )?;
    let ads_app =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_app_experience_v2")?;
    let ads_network = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_network_hotspot_rank",
    )?;
    let dwd_tcp =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;
    let tcp = batch_tables::table_has_rows(&mut conn, &raw_tcp)?;
    let game = batch_tables::table_has_rows(&mut conn, &raw_game)?;
    let app = batch_tables::table_has_analysis_run(&mut conn, &ads_app, &req.run_id())?;
    let server_ip: Option<i8> = conn.query_first(format!("SELECT EXISTS(SELECT 1 FROM `{dwd_tcp}` WHERE server_ip IS NOT NULL AND TRIM(server_ip)<>'' LIMIT 1)")).unwrap_or(Some(0));
    let server_ip_available = server_ip.unwrap_or(0) != 0;
    let (known_network, all_network): (i64, i64) = conn.exec_first(format!("SELECT CAST(COALESCE(SUM(CASE WHEN UPPER(COALESCE(bras,'UNKNOWN')) NOT IN ('UNKNOWN','UNAVAILABLE','') OR UPPER(COALESCE(olt,'UNKNOWN')) NOT IN ('UNKNOWN','UNAVAILABLE','') OR UPPER(COALESCE(pon,'UNKNOWN')) NOT IN ('UNKNOWN','UNAVAILABLE','') THEN 1 ELSE 0 END),0) AS SIGNED), CAST(COUNT(*) AS SIGNED) FROM `{ads_network}` WHERE analysis_run_id=?"), (&req.run_id(),)).unwrap_or(Some((0,0))).unwrap_or((0,0));
    let topology_rate =
        (all_network > 0).then(|| known_network as f64 * 100.0 / all_network as f64);
    Ok(vec![
        DataCoverageItemV2 { dimension: "TCP".into(), status: if tcp { "AVAILABLE" } else { "NOT_IMPORTED" }.into(), available_rows: i64::from(tcp), total_rows: 1, coverage_pct: Some(if tcp {100.0} else {0.0}), limitation: None },
        DataCoverageItemV2 { dimension: "GAME".into(), status: if game { "AVAILABLE" } else { "NOT_IMPORTED" }.into(), available_rows: i64::from(game), total_rows: 1, coverage_pct: Some(if game {100.0} else {0.0}), limitation: (!game).then(|| "Game is a separate source; MOS, jitter and duration are not imported, not zero.".into()) },
        DataCoverageItemV2 { dimension: "APP_EXPERIENCE_V2".into(), status: if app { "AVAILABLE" } else { "UNAVAILABLE" }.into(), available_rows: i64::from(app), total_rows: 1, coverage_pct: Some(if app {100.0} else {0.0}), limitation: (!app).then(|| "Run V2 aggregation for this analysis_run_id.".into()) },
        DataCoverageItemV2 { dimension: "NETWORK_TOPOLOGY".into(), status: if known_network > 0 { "AVAILABLE" } else { "UNAVAILABLE" }.into(), available_rows: known_network, total_rows: all_network, coverage_pct: topology_rate, limitation: Some("Only real BRAS/OLT/PON values count. Missing topology is not a confirmed hotspot.".into()) },
        DataCoverageItemV2 { dimension: "SERVER_IP".into(), status: if server_ip_available { "AVAILABLE" } else if tcp { "LIMITED" } else { "NOT_IMPORTED" }.into(), available_rows: i64::from(server_ip_available), total_rows: 1, coverage_pct: server_ip_available.then_some(100.0), limitation: Some(if server_ip_available { "Available through a finding/App-scoped query capped at 200 users and 20,000 DWD observations; no full-table IP explosion." } else { "The source exists, but this batch CLEAN result does not retain usable Server IP values. Re-run CLEAN with version 1.0.50 before using controlled drill-down." }.into()) },
        DataCoverageItemV2 { dimension: "IDENTITY".into(), status: "LIMITED".into(), available_rows: 0, total_rows: 0, coverage_pct: None, limitation: Some("Account can be an IP and MAC may be missing; identity confidence is advisory.".into()) },
    ])
}

fn aggregate_run_rates(
    conn: &mut mysql::PooledConn,
    table: &str,
    run_id: &str,
) -> Result<(Option<f64>, Option<f64>, Option<f64>), String> {
    conn.exec_first(
        format!("SELECT CAST(ROUND(SUM(poor_obs_rows)*100.0/NULLIF(SUM(valid_obs_rows),0),6) AS DOUBLE),CAST(ROUND(COUNT(DISTINCT CASE WHEN persistent_poor_user_flag=1 THEN user_key END)*100.0/NULLIF(COUNT(DISTINCT CASE WHEN eligible_user_flag=1 THEN user_key END),0),6) AS DOUBLE),CAST(ROUND(COUNT(DISTINCT CASE WHEN severe_poor_user_flag=1 THEN user_key END)*100.0/NULLIF(COUNT(DISTINCT CASE WHEN eligible_user_flag=1 THEN user_key END),0),6) AS DOUBLE) FROM `{table}` WHERE analysis_run_id=?"),
        (run_id,),
    ).map(|row| row.unwrap_or((None, None, None))).map_err(|err| format!("failed to query comparable run rates: {err}"))
}

#[tauri::command]
pub fn analytics_get_run_verification_v2(
    req: DashboardRequest,
) -> Result<RunVerificationV2, String> {
    crate::migrations::ensure_experience_policy_schema(&req.settings)?;
    let current_run = req.run_id();
    let current_table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_app_period_experience_v2",
    )?;
    let mut conn = db::conn(&req.settings)?;
    let current_rates = aggregate_run_rates(&mut conn, &current_table, &current_run)?;
    let previous: Option<(String, String)> = conn.exec_first(
        "SELECT candidate.analysis_run_id,candidate.import_batch_id FROM meta_analysis_run_policy_binding current_binding JOIN meta_analysis_run current_run ON current_run.analysis_run_id=current_binding.analysis_run_id JOIN meta_analysis_run_policy_binding candidate ON candidate.analysis_run_id<>current_binding.analysis_run_id AND candidate.experience_policy_id=current_binding.experience_policy_id AND candidate.experience_policy_version=current_binding.experience_policy_version AND candidate.access_rule_set_id<=>current_binding.access_rule_set_id AND candidate.access_rule_set_version<=>current_binding.access_rule_set_version AND candidate.others_access_type<=>current_binding.others_access_type AND candidate.app_mapping_version<=>current_binding.app_mapping_version JOIN meta_analysis_run candidate_run ON candidate_run.analysis_run_id=candidate.analysis_run_id AND candidate_run.status IN ('success','degraded') AND COALESCE(candidate_run.finished_at,candidate_run.started_at)<current_run.started_at WHERE current_binding.analysis_run_id=? AND current_binding.import_batch_id=? ORDER BY COALESCE(candidate_run.finished_at,candidate_run.started_at) DESC LIMIT 1",
        (&current_run, &req.import_batch_id),
    ).map_err(|err| format!("failed to find previous comparable run: {err}"))?;
    let Some((previous_run, previous_batch)) = previous else {
        return Ok(RunVerificationV2 { current_analysis_run_id: current_run, current_import_batch_id: req.import_batch_id, previous_analysis_run_id: None, previous_import_batch_id: None, comparable: false, comparison_reason: "No earlier successful run has the same access, Others, App mapping and experience-policy versions.".into(), current_poor_observation_rate_pct: current_rates.0, previous_poor_observation_rate_pct: None, poor_observation_rate_delta_pct: None, current_persistent_poor_user_rate_pct: current_rates.1, previous_persistent_poor_user_rate_pct: None, persistent_poor_user_rate_delta_pct: None, current_severe_poor_user_rate_pct: current_rates.2, previous_severe_poor_user_rate_pct: None, severe_poor_user_rate_delta_pct: None });
    };
    let previous_table = batch_tables::resolve_table(
        &req.settings,
        &previous_batch,
        "dws_user_app_period_experience_v2",
    )?;
    let previous_rates = aggregate_run_rates(&mut conn, &previous_table, &previous_run)?;
    let delta = |current: Option<f64>, earlier: Option<f64>| {
        current.zip(earlier).map(|(now, before)| now - before)
    };
    let has_metrics = current_rates.0.is_some() && previous_rates.0.is_some();
    Ok(RunVerificationV2 {
        current_analysis_run_id: current_run,
        current_import_batch_id: req.import_batch_id,
        previous_analysis_run_id: Some(previous_run),
        previous_import_batch_id: Some(previous_batch),
        comparable: has_metrics,
        comparison_reason: if has_metrics {
            "Same versioned rules; negative delta means improvement."
        } else {
            "A version-compatible run exists, but one side has no V2 observations."
        }
        .into(),
        current_poor_observation_rate_pct: current_rates.0,
        previous_poor_observation_rate_pct: previous_rates.0,
        poor_observation_rate_delta_pct: delta(current_rates.0, previous_rates.0),
        current_persistent_poor_user_rate_pct: current_rates.1,
        previous_persistent_poor_user_rate_pct: previous_rates.1,
        persistent_poor_user_rate_delta_pct: delta(current_rates.1, previous_rates.1),
        current_severe_poor_user_rate_pct: current_rates.2,
        previous_severe_poor_user_rate_pct: previous_rates.2,
        severe_poor_user_rate_delta_pct: delta(current_rates.2, previous_rates.2),
    })
}

#[tauri::command]
pub fn analytics_get_investigation_evidence(
    req: InvestigationQueryRequest,
) -> Result<Vec<InvestigationEvidenceRow>, String> {
    let hourly_scope = req.date_from.is_some()
        || req.date_to.is_some()
        || req.hour_from.is_some()
        || req.hour_to.is_some();
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        if hourly_scope {
            "dws_user_app_hourly_experience_v2"
        } else {
            "dws_user_app_period_experience_v2"
        },
    )?;
    let mut sql = format!("SELECT user_key,user_type,app_category,app_name,CAST(valid_obs_rows AS SIGNED),CAST(poor_obs_rows AS SIGNED),CAST(poor_observation_rate_pct AS DOUBLE),persistent_poor_user_flag,severe_poor_user_flag,CAST(avg_vmos AS DOUBLE),CAST(avg_subscriber_rtt_ms AS DOUBLE),CAST(avg_network_rtt_ms AS DOUBLE),CAST(avg_user_loss_pct AS DOUBLE),CAST(avg_network_loss_pct AS DOUBLE) FROM `{table}` WHERE analysis_run_id=?");
    let mut params = vec![Value::from(req.analysis_run_id)];
    for (column, value) in [
        ("app_category", clean(req.app_category)),
        ("app_name", clean(req.app_name)),
        ("user_type", clean(req.access_type)),
        ("user_key", clean(req.user_key)),
    ] {
        if let Some(value) = value {
            sql.push_str(&format!(" AND `{column}`=?"));
            params.push(Value::from(value));
        }
    }
    for (operator, value) in [(">=", clean(req.date_from)), ("<=", clean(req.date_to))] {
        if let Some(value) = value {
            sql.push_str(&format!(" AND stat_date{operator}?"));
            params.push(Value::from(value));
        }
    }
    for (operator, value) in [(">=", req.hour_from), ("<=", req.hour_to)] {
        if let Some(value) = value.filter(|item| (0..=23).contains(item)) {
            sql.push_str(&format!(" AND hour_of_day{operator}?"));
            params.push(Value::from(value));
        }
    }
    sql.push_str(" ORDER BY severe_poor_user_flag DESC,persistent_poor_user_flag DESC,poor_observation_rate_pct DESC,valid_obs_rows DESC LIMIT ?");
    params.push(Value::from(req.page_size.unwrap_or(100).clamp(1, 500)));
    let mut conn = db::conn(&req.settings)?;
    let rows = conn
        .exec_iter(sql, Params::Positional(params))
        .map_err(|err| format!("failed to query investigation evidence: {err}"))?;
    rows.map(|row| {
        let row = row.map_err(|err| format!("failed to decode investigation evidence: {err}"))?;
        Ok(InvestigationEvidenceRow {
            user_key: row_string(&row, 0, "UNIDENTIFIED"),
            access_type: row_string(&row, 1, "UNCLASSIFIED"),
            app_category: row_string(&row, 2, "UNCLASSIFIED"),
            app_name: row_string(&row, 3, "UNCLASSIFIED"),
            valid_obs_rows: row_i64(&row, 4),
            poor_obs_rows: row_i64(&row, 5),
            poor_observation_rate_pct: row_f64(&row, 6),
            persistent_poor_user: row_i8(&row, 7) != 0,
            severe_poor_user: row_i8(&row, 8) != 0,
            avg_vmos: row_f64(&row, 9),
            avg_subscriber_rtt_ms: row_f64(&row, 10),
            avg_network_rtt_ms: row_f64(&row, 11),
            avg_user_loss_pct: row_f64(&row, 12),
            avg_network_loss_pct: row_f64(&row, 13),
        })
    })
    .collect()
}

#[tauri::command]
pub fn analytics_get_investigation_hourly(
    req: InvestigationQueryRequest,
) -> Result<Vec<InvestigationHourlyRow>, String> {
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_app_hourly_experience_v2",
    )?;
    let mut sql=format!("SELECT DATE_FORMAT(stat_date,'%Y-%m-%d'),hour_of_day,user_type,CAST(eligible_users AS SIGNED),CAST(valid_obs_rows AS SIGNED),CAST(poor_obs_rows AS SIGNED),CAST(poor_observation_rate_pct AS DOUBLE),CAST(persistent_poor_users AS SIGNED),CAST(severe_poor_users AS SIGNED),sample_status FROM `{table}` WHERE analysis_run_id=?");
    let mut params = vec![Value::from(req.analysis_run_id)];
    for (column, value) in [
        ("app_category", clean(req.app_category)),
        ("app_name", clean(req.app_name)),
        ("user_type", clean(req.access_type)),
    ] {
        if let Some(value) = value {
            sql.push_str(&format!(" AND `{column}`=?"));
            params.push(Value::from(value));
        }
    }
    for (operator, value) in [(">=", clean(req.date_from)), ("<=", clean(req.date_to))] {
        if let Some(value) = value {
            sql.push_str(&format!(" AND stat_date{operator}?"));
            params.push(Value::from(value));
        }
    }
    for (operator, value) in [(">=", req.hour_from), ("<=", req.hour_to)] {
        if let Some(value) = value.filter(|item| (0..=23).contains(item)) {
            sql.push_str(&format!(" AND hour_of_day{operator}?"));
            params.push(Value::from(value));
        }
    }
    sql.push_str(" ORDER BY stat_date,hour_of_day,user_type LIMIT ?");
    params.push(Value::from(req.page_size.unwrap_or(500).clamp(1, 2000)));
    let mut conn = db::conn(&req.settings)?;
    let rows = conn
        .exec_iter(sql, Params::Positional(params))
        .map_err(|err| format!("failed to query investigation hourly pivot: {err}"))?;
    rows.map(|row| {
        let row =
            row.map_err(|err| format!("failed to decode investigation hourly pivot: {err}"))?;
        Ok(InvestigationHourlyRow {
            stat_date: row_string(&row, 0, ""),
            hour_of_day: row_i32(&row, 1),
            access_type: row_string(&row, 2, "UNCLASSIFIED"),
            eligible_users: row_i64(&row, 3),
            valid_obs_rows: row_i64(&row, 4),
            poor_obs_rows: row_i64(&row, 5),
            poor_observation_rate_pct: row_f64(&row, 6),
            persistent_poor_users: row_i64(&row, 7),
            severe_poor_users: row_i64(&row, 8),
            sample_status: row_string(&row, 9, "INSUFFICIENT_SAMPLE"),
        })
    })
    .collect()
}

fn add_metric(value: Option<f64>, sum: &mut f64, count: &mut i64) {
    if let Some(value) = value.filter(|item| item.is_finite()) {
        *sum += value;
        *count += 1;
    }
}

fn average(sum: f64, count: i64) -> Option<f64> {
    (count > 0).then(|| sum / count as f64)
}

#[tauri::command]
pub fn analytics_get_investigation_server_ips(
    req: InvestigationQueryRequest,
) -> Result<Vec<InvestigationServerIpRow>, String> {
    const MAX_USERS: u64 = 200;
    const MAX_DWD_ROWS: u64 = 20_000;
    const MAX_IPS_PER_ROW: usize = 32;

    let app_name = clean(req.app_name.clone())
        .ok_or_else(|| "Server IP drill-down requires an explicit App scope".to_string())?;
    let hourly_scope = req.date_from.is_some()
        || req.date_to.is_some()
        || req.hour_from.is_some()
        || req.hour_to.is_some();
    let dws = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        if hourly_scope {
            "dws_user_app_hourly_experience_v2"
        } else {
            "dws_user_app_period_experience_v2"
        },
    )?;
    let dwd =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;

    let mut scoped_users =
        format!("SELECT user_key FROM `{dws}` WHERE analysis_run_id=? AND app_name=?");
    let mut params = vec![
        Value::from(req.analysis_run_id.clone()),
        Value::from(app_name.clone()),
    ];
    for (column, value) in [
        ("app_category", clean(req.app_category.clone())),
        ("user_type", clean(req.access_type.clone())),
        ("user_key", clean(req.user_key.clone())),
    ] {
        if let Some(value) = value {
            scoped_users.push_str(&format!(" AND `{column}`=?"));
            params.push(Value::from(value));
        }
    }
    for (operator, value) in [
        (">=", clean(req.date_from.clone())),
        ("<=", clean(req.date_to.clone())),
    ] {
        if let Some(value) = value {
            scoped_users.push_str(&format!(" AND stat_date{operator}?"));
            params.push(Value::from(value));
        }
    }
    for (operator, value) in [(">=", req.hour_from), ("<=", req.hour_to)] {
        if let Some(value) = value.filter(|item| (0..=23).contains(item)) {
            scoped_users.push_str(&format!(" AND hour_of_day{operator}?"));
            params.push(Value::from(value));
        }
    }
    scoped_users.push_str(" GROUP BY user_key ORDER BY MAX(severe_poor_user_flag) DESC,MAX(persistent_poor_user_flag) DESC,SUM(poor_obs_rows) DESC LIMIT ?");
    params.push(Value::from(MAX_USERS));

    let mut sql = format!("SELECT d.server_ip,d.user_key,CAST(d.subscriber_side_rtt_ms AS DOUBLE),CAST(d.network_side_rtt_ms AS DOUBLE),CAST(d.user_down_loss AS DOUBLE),CAST(d.network_down_loss AS DOUBLE) FROM `{dwd}` d JOIN ({scoped_users}) scoped ON scoped.user_key=d.user_key WHERE d.import_batch_id=? AND d.app_name=? AND d.server_ip IS NOT NULL AND TRIM(d.server_ip)<>''");
    params.push(Value::from(req.import_batch_id));
    params.push(Value::from(app_name));
    if let Some(value) = clean(req.access_type) {
        sql.push_str(" AND d.user_type=?");
        params.push(Value::from(value));
    }
    if let Some(value) = clean(req.date_from) {
        sql.push_str(" AND d.stat_date>=?");
        params.push(Value::from(value));
    }
    if let Some(value) = clean(req.date_to) {
        sql.push_str(" AND d.stat_date<=?");
        params.push(Value::from(value));
    }
    if let Some(value) = req.hour_from.filter(|item| (0..=23).contains(item)) {
        sql.push_str(" AND d.hour_of_day>=?");
        params.push(Value::from(value));
    }
    if let Some(value) = req.hour_to.filter(|item| (0..=23).contains(item)) {
        sql.push_str(" AND d.hour_of_day<=?");
        params.push(Value::from(value));
    }
    sql.push_str(" LIMIT ?");
    params.push(Value::from(MAX_DWD_ROWS));

    let mut conn = db::conn(&req.settings)?;
    let rows = conn
        .exec_iter(sql, Params::Positional(params))
        .map_err(|err| format!("failed to query controlled Server IP evidence: {err}"))?;
    let mut aggregates: HashMap<String, ServerIpAccumulator> = HashMap::new();
    for row in rows {
        let row = row.map_err(|err| format!("failed to decode Server IP evidence: {err}"))?;
        let raw = row_string(&row, 0, "");
        let user = row_string(&row, 1, "UNIDENTIFIED");
        let subscriber_rtt = row_f64(&row, 2);
        let network_rtt = row_f64(&row, 3);
        let user_loss = row_f64(&row, 4);
        let network_loss = row_f64(&row, 5);
        let mut row_ips = HashSet::new();
        for token in raw
            .split(|ch: char| ch == ';' || ch == ',' || ch == '|' || ch.is_whitespace())
            .take(MAX_IPS_PER_ROW)
        {
            let normalized = token.trim().trim_matches(|ch| ch == '[' || ch == ']');
            if normalized.parse::<IpAddr>().is_ok() && row_ips.insert(normalized.to_string()) {
                let aggregate = aggregates.entry(normalized.to_string()).or_default();
                aggregate.users.insert(user.clone());
                aggregate.observation_rows += 1;
                add_metric(
                    subscriber_rtt,
                    &mut aggregate.subscriber_rtt_sum,
                    &mut aggregate.subscriber_rtt_count,
                );
                add_metric(
                    network_rtt,
                    &mut aggregate.network_rtt_sum,
                    &mut aggregate.network_rtt_count,
                );
                add_metric(
                    user_loss,
                    &mut aggregate.user_loss_sum,
                    &mut aggregate.user_loss_count,
                );
                add_metric(
                    network_loss,
                    &mut aggregate.network_loss_sum,
                    &mut aggregate.network_loss_count,
                );
            }
        }
    }
    let mut result: Vec<_> = aggregates
        .into_iter()
        .map(|(server_ip, value)| InvestigationServerIpRow {
            server_ip,
            observed_users: value.users.len() as i64,
            observation_rows: value.observation_rows,
            avg_subscriber_rtt_ms: average(value.subscriber_rtt_sum, value.subscriber_rtt_count),
            avg_network_rtt_ms: average(value.network_rtt_sum, value.network_rtt_count),
            avg_user_loss_pct: average(value.user_loss_sum, value.user_loss_count),
            avg_network_loss_pct: average(value.network_loss_sum, value.network_loss_count),
        })
        .collect();
    result.sort_by(|a, b| {
        b.observed_users
            .cmp(&a.observed_users)
            .then_with(|| b.observation_rows.cmp(&a.observation_rows))
            .then_with(|| a.server_ip.cmp(&b.server_ip))
    });
    result.truncate(req.page_size.unwrap_or(50).clamp(1, 100) as usize);
    Ok(result)
}

fn decode_saved(row: mysql::Row) -> SavedInvestigation {
    SavedInvestigation {
        investigation_id: row_string(&row, 0, ""),
        import_batch_id: row_string(&row, 1, ""),
        analysis_run_id: row_string(&row, 2, ""),
        finding_id: row
            .get_opt::<Option<String>, _>(3)
            .and_then(Result::ok)
            .flatten(),
        title: row_string(&row, 4, "Untitled investigation"),
        status: row_string(&row, 5, "open"),
        context_json: row_string(&row, 6, "{}"),
        notes: row
            .get_opt::<Option<String>, _>(7)
            .and_then(Result::ok)
            .flatten(),
        created_at: row_string(&row, 8, ""),
        updated_at: row_string(&row, 9, ""),
    }
}

#[tauri::command]
pub fn investigation_save(req: SaveInvestigationRequest) -> Result<SavedInvestigation, String> {
    crate::migrations::ensure_experience_policy_schema(&req.settings)?;
    let context: serde_json::Value = serde_json::from_str(&req.context_json)
        .map_err(|err| format!("invalid investigation context JSON: {err}"))?;
    if !context.is_object() {
        return Err("investigation context must be a JSON object".into());
    }
    let title = req.title.trim();
    if title.is_empty() {
        return Err("investigation title is required".into());
    }
    let status = req.status.unwrap_or_else(|| "open".into()).to_lowercase();
    if !["open", "observing", "actioned", "closed"].contains(&status.as_str()) {
        return Err(format!("unsupported investigation status: {status}"));
    }
    let id = format!("INV_{}", Uuid::new_v4().simple());
    let mut conn = db::conn(&req.settings)?;
    conn.exec_drop("INSERT INTO meta_saved_investigation (investigation_id,import_batch_id,analysis_run_id,finding_id,title,status,context_json,notes) VALUES (?,?,?,?,?,?,?,?)",(&id,&req.import_batch_id,&req.analysis_run_id,&req.finding_id,title,&status,&req.context_json,&req.notes)).map_err(|err| format!("failed to save investigation: {err}"))?;
    let row = conn.exec_first("SELECT investigation_id,import_batch_id,analysis_run_id,finding_id,title,status,CAST(context_json AS CHAR),notes,DATE_FORMAT(created_at,'%Y-%m-%d %H:%i:%s'),DATE_FORMAT(updated_at,'%Y-%m-%d %H:%i:%s') FROM meta_saved_investigation WHERE investigation_id=?",(&id,)).map_err(|err| format!("failed to read saved investigation: {err}"))?.ok_or_else(|| "saved investigation not found".to_string())?;
    Ok(decode_saved(row))
}

#[tauri::command]
pub fn investigation_list(
    req: InvestigationListRequest,
) -> Result<Vec<SavedInvestigation>, String> {
    crate::migrations::ensure_experience_policy_schema(&req.settings)?;
    let mut conn = db::conn(&req.settings)?;
    let (sql, params) = if let Some(run) = clean(req.analysis_run_id) {
        ("SELECT investigation_id,import_batch_id,analysis_run_id,finding_id,title,status,CAST(context_json AS CHAR),notes,DATE_FORMAT(created_at,'%Y-%m-%d %H:%i:%s'),DATE_FORMAT(updated_at,'%Y-%m-%d %H:%i:%s') FROM meta_saved_investigation WHERE import_batch_id=? AND analysis_run_id=? ORDER BY updated_at DESC LIMIT 300",Params::Positional(vec![Value::from(req.import_batch_id),Value::from(run)]))
    } else {
        ("SELECT investigation_id,import_batch_id,analysis_run_id,finding_id,title,status,CAST(context_json AS CHAR),notes,DATE_FORMAT(created_at,'%Y-%m-%d %H:%i:%s'),DATE_FORMAT(updated_at,'%Y-%m-%d %H:%i:%s') FROM meta_saved_investigation WHERE import_batch_id=? ORDER BY updated_at DESC LIMIT 300",Params::Positional(vec![Value::from(req.import_batch_id)]))
    };
    let rows = conn
        .exec_iter(sql, params)
        .map_err(|err| format!("failed to list investigations: {err}"))?;
    rows.map(|row| {
        row.map(decode_saved)
            .map_err(|err| format!("failed to decode investigation: {err}"))
    })
    .collect()
}

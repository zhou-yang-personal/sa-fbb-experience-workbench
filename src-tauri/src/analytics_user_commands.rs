use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn order_sql(sort_by: &str) -> &'static str {
    match sort_by {
        "game_hours" | "game" => "game_hours DESC, traffic_gb DESC",
        "vmos" => "vmos ASC, traffic_gb DESC",
        "mos" => "mos ASC, traffic_gb DESC",
        "subscriber_rtt_ms" | "rtt" => "subscriber_rtt_ms DESC, traffic_gb DESC",
        "demand_score" | "demand" => "demand_score DESC, traffic_gb DESC",
        "migration_motive_score" | "motive" => "migration_motive_score DESC, traffic_gb DESC",
        "label" | "user_key" => "user_key ASC",
        _ => "traffic_gb DESC, game_hours DESC",
    }
}

#[tauri::command]
pub fn analytics_get_user_profiles(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.run_id();
    let mut conn = db::conn(&req.settings)?;
    let page_size = req.page_size(160, 500);
    let offset = req.offset(160, 500);
    let keyword = req.keyword_like();
    if let Ok(ads_table) = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_user_experience_profile") {
        let ads_count: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{ads_table}` WHERE analysis_run_id=?"), (&run_id,)).unwrap_or(Some(0));
        if ads_count.unwrap_or(0) > 0 {
            let sql = format!("SELECT user_key, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(active_days AS SIGNED) AS active_days, CAST(ROUND(COALESCE(total_traffic_gb,0),2) AS DOUBLE) AS traffic_gb, CAST(ROUND(COALESCE(game_hours,0),2) AS DOUBLE) AS game_hours, CAST(ROUND(COALESCE(avg_vmos,0),2) AS DOUBLE) AS vmos, CAST(ROUND(COALESCE(avg_mos,0),2) AS DOUBLE) AS mos, CAST(ROUND(COALESCE(avg_subscriber_rtt_ms,0),2) AS DOUBLE) AS subscriber_rtt_ms, COALESCE(lead_type,'NONE') AS lead_type, COALESCE(demand_score,0) AS demand_score, COALESCE(migration_motive_score,0) AS migration_motive_score FROM `{ads_table}` WHERE analysis_run_id=? AND (? IS NULL OR user_key LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ? OR COALESCE(lead_type,'NONE') LIKE ?) AND GREATEST(COALESCE(total_traffic_gb,0), COALESCE(game_hours,0), COALESCE(demand_score,0), COALESCE(migration_motive_score,0)) >= ? ORDER BY {} LIMIT ? OFFSET ?", order_sql(&req.sort_by()));
            return conn.exec_map(sql, (&run_id, keyword.clone(), keyword.clone(), keyword.clone(), keyword, req.min_value(), page_size, offset), |(user_key, user_type, days, gb, hours, vmos, mos, rtt, lead_type, demand, motive): (String, String, i64, f64, f64, f64, f64, f64, String, i64, i64)| MetricCard {
                let label = user_key.clone();
                MetricCard { label, value: format!("{gb:.2}"), hint: format!("source=ads_user_experience_profile, user_key={user_key}, user_type={user_type}, active_days={days}, traffic_gb={gb:.2}, game_hours={hours:.2}, vmos={vmos:.2}, mos={mos:.2}, subscriber_rtt_ms={rtt:.2}, lead_type={lead_type}, demand_score={demand}, migration_motive_score={motive}, page_size={page_size}, offset={offset}") }
            }).map_err(|err| format!("failed to query analytics user ADS profiles: {err}"));
        }
    }
    let profile_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?;
    let lead_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_migration_lead_user")?;
    let sql = format!("SELECT p.user_key, COALESCE(MAX(p.user_type),'UNKNOWN') AS user_type, CAST(COUNT(DISTINCT p.stat_date) AS SIGNED) AS active_days, CAST(ROUND(COALESCE(SUM(p.total_download_gb),0),2) AS DOUBLE) AS traffic_gb, CAST(ROUND(COALESCE(SUM(p.total_game_hours),0),2) AS DOUBLE) AS game_hours, CAST(ROUND(COALESCE(AVG(p.avg_vmos),0),2) AS DOUBLE) AS vmos, CAST(ROUND(COALESCE(AVG(p.avg_mos),0),2) AS DOUBLE) AS mos, CAST(ROUND(COALESCE(AVG(p.avg_subscriber_rtt_ms),0),2) AS DOUBLE) AS subscriber_rtt_ms, COALESCE(MAX(l.lead_type),'NONE') AS lead_type, COALESCE(MAX(l.demand_score),0) AS demand_score, COALESCE(MAX(l.migration_motive_score),0) AS migration_motive_score FROM `{profile_table}` p LEFT JOIN `{lead_table}` l ON l.analysis_run_id=? AND l.user_key=p.user_key WHERE p.import_batch_id=? AND (? IS NULL OR p.user_key LIKE ? OR COALESCE(p.user_type,'UNKNOWN') LIKE ? OR COALESCE(l.lead_type,'NONE') LIKE ?) GROUP BY p.user_key HAVING GREATEST(traffic_gb, game_hours, demand_score, migration_motive_score) >= ? ORDER BY {} LIMIT ? OFFSET ?", order_sql(&req.sort_by()));
    conn.exec_map(sql, (&run_id, &req.import_batch_id, keyword.clone(), keyword.clone(), keyword.clone(), keyword, req.min_value(), page_size, offset), |(user_key, user_type, days, gb, hours, vmos, mos, rtt, lead_type, demand, motive): (String, String, i64, f64, f64, f64, f64, f64, String, i64, i64)| MetricCard {
        let label = user_key.clone();
        MetricCard { label, value: format!("{gb:.2}"), hint: format!("source=dws_user_daily_profile, user_key={user_key}, user_type={user_type}, active_days={days}, traffic_gb={gb:.2}, game_hours={hours:.2}, vmos={vmos:.2}, mos={mos:.2}, subscriber_rtt_ms={rtt:.2}, lead_type={lead_type}, demand_score={demand}, migration_motive_score={motive}, page_size={page_size}, offset={offset}") }
    }).map_err(|err| format!("failed to query analytics user profiles: {err}"))
}

use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn run_id(req: &DashboardRequest) -> String {
    req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string())
}

#[tauri::command]
pub fn analytics_get_user_profiles(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = run_id(&req);
    let mut conn = db::conn(&req.settings)?;
    let profile_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?;
    let lead_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_migration_lead_user")?;
    let sql = format!(
        "SELECT p.user_key, COALESCE(MAX(p.user_type),'UNKNOWN'), CAST(COUNT(DISTINCT p.stat_date) AS SIGNED), CAST(ROUND(COALESCE(SUM(p.total_download_gb),0),2) AS DOUBLE), CAST(ROUND(COALESCE(SUM(p.total_game_hours),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(p.avg_vmos),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(p.avg_mos),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(p.avg_subscriber_rtt_ms),0),2) AS DOUBLE), COALESCE(MAX(l.lead_type),'NONE'), COALESCE(MAX(l.demand_score),0), COALESCE(MAX(l.migration_motive_score),0) FROM `{profile_table}` p LEFT JOIN `{lead_table}` l ON l.analysis_run_id=? AND l.user_key=p.user_key WHERE p.import_batch_id=? GROUP BY p.user_key ORDER BY SUM(p.total_download_gb) DESC, SUM(p.total_game_hours) DESC LIMIT 160"
    );
    conn.exec_map(
        sql,
        (&run_id, &req.import_batch_id),
        |(user_key, user_type, days, gb, hours, vmos, mos, rtt, lead_type, demand, motive): (String, String, i64, f64, f64, f64, f64, f64, String, i64, i64)| MetricCard {
            label: user_key,
            value: format!("{gb:.2}"),
            hint: format!("user_type={user_type}, active_days={days}, traffic_gb={gb:.2}, game_hours={hours:.2}, vmos={vmos:.2}, mos={mos:.2}, subscriber_rtt_ms={rtt:.2}, lead_type={lead_type}, demand_score={demand}, migration_motive_score={motive}"),
        },
    ).map_err(|err| format!("failed to query analytics user profiles: {err}"))
}

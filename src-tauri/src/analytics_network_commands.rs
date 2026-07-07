use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

#[tauri::command]
pub fn analytics_get_network_hotspots(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&req.settings)?;
    let profile_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?;
    let bottleneck_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_experience_bottleneck")?;
    let sql = format!(
        "SELECT COALESCE(b.bottleneck_type,'UNKNOWN'), COALESCE(p.user_type,'UNKNOWN'), CAST(COUNT(DISTINCT p.user_key) AS SIGNED), CAST(ROUND(COALESCE(AVG(b.severity_score),0),2) AS DOUBLE), CAST(ROUND(COALESCE(SUM(p.total_download_gb),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(p.avg_subscriber_rtt_ms),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(p.avg_network_rtt_ms),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(p.avg_user_down_loss),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(p.avg_network_down_loss),0),2) AS DOUBLE) FROM `{profile_table}` p LEFT JOIN `{bottleneck_table}` b ON b.import_batch_id=p.import_batch_id AND b.user_key=p.user_key WHERE p.import_batch_id=? GROUP BY COALESCE(b.bottleneck_type,'UNKNOWN'), COALESCE(p.user_type,'UNKNOWN') ORDER BY AVG(b.severity_score) DESC, COUNT(DISTINCT p.user_key) DESC LIMIT 80"
    );
    conn.exec_map(
        sql,
        (&req.import_batch_id,),
        |(bottleneck, user_type, users, severity, gb, sub_rtt, net_rtt, user_loss, net_loss): (String, String, i64, f64, f64, f64, f64, f64, f64)| MetricCard {
            label: format!("{bottleneck} {user_type}"),
            value: format!("{severity:.2}"),
            hint: format!("bottleneck={bottleneck}, user_type={user_type}, users={users}, severity={severity:.2}, traffic_gb={gb:.2}, subscriber_rtt_ms={sub_rtt:.2}, network_rtt_ms={net_rtt:.2}, user_loss_pct={user_loss:.2}, network_loss_pct={net_loss:.2}"),
        },
    ).map_err(|err| format!("failed to query analytics network hotspots: {err}"))
}

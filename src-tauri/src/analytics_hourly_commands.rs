use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

#[tauri::command]
pub fn analytics_get_hourly_trend(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_access_type_hourly_compare")?;
    let sql = format!(
        "SELECT hour_of_day, COALESCE(user_type,'UNKNOWN'), CAST(SUM(active_users) AS SIGNED), CAST(ROUND(COALESCE(AVG(avg_download_mbps),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(avg_subscriber_rtt_ms),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(avg_user_down_loss),0),2) AS DOUBLE) FROM `{table}` WHERE import_batch_id=? GROUP BY hour_of_day, COALESCE(user_type,'UNKNOWN') ORDER BY hour_of_day, user_type"
    );
    conn.exec_map(
        sql,
        (&req.import_batch_id,),
        |(hour, user_type, users, mbps, rtt, loss): (i64, String, i64, f64, f64, f64)| MetricCard {
            label: format!("h{hour:02} {user_type}"),
            value: users.to_string(),
            hint: format!("hour={hour}, user_type={user_type}, users={users}, effective_mbps={mbps:.2}, subscriber_rtt_ms={rtt:.2}, user_loss_pct={loss:.2}"),
        },
    ).map_err(|err| format!("failed to query analytics hourly trend: {err}"))
}

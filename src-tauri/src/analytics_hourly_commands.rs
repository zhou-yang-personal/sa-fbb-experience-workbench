use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn order_sql(sort_by: &str) -> &'static str {
    match sort_by {
        "effective_mbps" | "mbps" => "effective_mbps DESC, hour_of_day ASC",
        "subscriber_rtt_ms" | "rtt" => "subscriber_rtt_ms DESC, hour_of_day ASC",
        "user_loss_pct" | "loss" => "user_loss_pct DESC, hour_of_day ASC",
        "hour" | "label" => "hour_of_day ASC, user_type ASC",
        _ => "hour_of_day ASC, user_type ASC",
    }
}

#[tauri::command]
pub fn analytics_get_hourly_trend(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_access_type_hourly_compare")?;
    let page_size = req.page_size(120, 500);
    let offset = req.offset(120, 500);
    let keyword = req.keyword_like();
    let sql = format!(
        "SELECT hour_of_day, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(SUM(active_users) AS SIGNED) AS users, CAST(ROUND(COALESCE(AVG(avg_download_mbps),0),2) AS DOUBLE) AS effective_mbps, CAST(ROUND(COALESCE(AVG(avg_subscriber_rtt_ms),0),2) AS DOUBLE) AS subscriber_rtt_ms, CAST(ROUND(COALESCE(AVG(avg_user_down_loss),0),2) AS DOUBLE) AS user_loss_pct FROM `{table}` WHERE import_batch_id=? AND (? IS NULL OR COALESCE(user_type,'UNKNOWN') LIKE ?) GROUP BY hour_of_day, COALESCE(user_type,'UNKNOWN') HAVING users >= ? ORDER BY {} LIMIT ? OFFSET ?",
        order_sql(&req.sort_by())
    );
    conn.exec_map(
        sql,
        (&req.import_batch_id, keyword.clone(), keyword, req.min_value(), page_size, offset),
        |(hour, user_type, users, mbps, rtt, loss): (i64, String, i64, f64, f64, f64)| MetricCard {
            label: format!("h{hour:02} {user_type}"),
            value: users.to_string(),
            hint: format!("hour={hour}, user_type={user_type}, users={users}, effective_mbps={mbps:.2}, subscriber_rtt_ms={rtt:.2}, user_loss_pct={loss:.2}, page_size={page_size}, offset={offset}"),
        },
    ).map_err(|err| format!("failed to query analytics hourly trend: {err}"))
}

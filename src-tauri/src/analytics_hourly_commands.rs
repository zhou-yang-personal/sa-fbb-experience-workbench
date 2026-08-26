use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{AnalyticsQueryPage, DashboardRequest, MetricCard};

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
pub fn analytics_get_hourly_trend(req: DashboardRequest) -> Result<AnalyticsQueryPage, String> {
    let run_id = req.run_id();
    let mut conn = db::conn(&req.settings)?;
    let page_size = req.page_size(120, 500);
    let fetch_size = page_size.saturating_add(1);
    let offset = req.offset(120, 500);
    let keyword = req.keyword_like();
    let access_type = req.access_type_filter();
    if let Ok(ads_table) = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_hourly_experience_trend",
    ) {
        let ads_count: Option<i64> = conn
            .exec_first(
                format!(
                    "SELECT CAST(COUNT(*) AS SIGNED) FROM `{ads_table}` WHERE analysis_run_id=?"
                ),
                (&run_id,),
            )
            .unwrap_or(Some(0));
        if ads_count.unwrap_or(0) > 0 {
            let sql = format!("SELECT DATE_FORMAT(stat_date,'%Y-%m-%d'), hour_of_day, COALESCE(user_type,'UNKNOWN') AS user_type, COALESCE(app_category,'ALL') AS app_category, CAST(active_users AS SIGNED) AS users, CAST(ROUND(COALESCE(avg_effective_mbps,0),2) AS DOUBLE) AS effective_mbps, CAST(ROUND(COALESCE(avg_subscriber_rtt_ms,0),2) AS DOUBLE) AS subscriber_rtt_ms, CAST(ROUND(COALESCE(avg_user_loss_pct,0),2) AS DOUBLE) AS user_loss_pct FROM `{ads_table}` WHERE analysis_run_id=? AND (? IS NULL OR COALESCE(user_type,'UNKNOWN')=?) AND (? IS NULL OR COALESCE(user_type,'UNKNOWN') LIKE ? OR COALESCE(app_category,'ALL') LIKE ?) AND active_users >= ? ORDER BY stat_date, {} LIMIT ? OFFSET ?", order_sql(&req.sort_by()));
            let cards = conn.exec_map(sql, (&run_id, access_type.clone(), access_type.clone(), keyword.clone(), keyword.clone(), keyword, req.min_value(), fetch_size, offset), |(stat_date, hour, user_type, app_category, users, mbps, rtt, loss): (String, i64, String, String, i64, f64, f64, f64)| MetricCard {
                label: format!("{stat_date} h{hour:02} {user_type} {app_category}"),
                value: users.to_string(),
                hint: format!("source=ads_hourly_experience_trend, stat_date={stat_date}, hour={hour}, user_type={user_type}, app_category={app_category}, users={users}, effective_mbps={mbps:.2}, subscriber_rtt_ms={rtt:.2}, user_loss_pct={loss:.2}, page_size={page_size}, offset={offset}"),
            }).map_err(|err| format!("failed to query analytics hourly ADS trend: {err}"))?;
            return Ok(AnalyticsQueryPage::new(cards, &req, 120, 500, "ads_hourly_experience_trend"));
        }
    }
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_access_type_hourly_compare",
    )?;
    let sql = format!("SELECT hour_of_day, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(SUM(active_users) AS SIGNED) AS users, CAST(ROUND(COALESCE(AVG(avg_download_mbps),0),2) AS DOUBLE) AS effective_mbps, CAST(ROUND(COALESCE(AVG(avg_subscriber_rtt_ms),0),2) AS DOUBLE) AS subscriber_rtt_ms, CAST(ROUND(COALESCE(AVG(avg_user_down_loss),0),2) AS DOUBLE) AS user_loss_pct FROM `{table}` WHERE import_batch_id=? AND (? IS NULL OR COALESCE(user_type,'UNKNOWN')=?) AND (? IS NULL OR COALESCE(user_type,'UNKNOWN') LIKE ?) GROUP BY hour_of_day, COALESCE(user_type,'UNKNOWN') HAVING users >= ? ORDER BY {} LIMIT ? OFFSET ?", order_sql(&req.sort_by()));
    let cards = conn.exec_map(sql, (&req.import_batch_id, access_type.clone(), access_type, keyword.clone(), keyword, req.min_value(), fetch_size, offset), |(hour, user_type, users, mbps, rtt, loss): (i64, String, i64, f64, f64, f64)| MetricCard {
        label: format!("h{hour:02} {user_type}"),
        value: users.to_string(),
        hint: format!("source=dws_access_type_hourly_compare, hour={hour}, user_type={user_type}, users={users}, effective_mbps={mbps:.2}, subscriber_rtt_ms={rtt:.2}, user_loss_pct={loss:.2}, page_size={page_size}, offset={offset}"),
    }).map_err(|err| format!("failed to query analytics hourly trend: {err}"))?;
    Ok(AnalyticsQueryPage::new(cards, &req, 120, 500, "dws_access_type_hourly_compare"))
}

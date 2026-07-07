use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn order_sql(sort_by: &str) -> &'static str {
    match sort_by {
        "users" => "users DESC, severity DESC",
        "traffic_gb" | "traffic" => "traffic_gb DESC, severity DESC",
        "subscriber_rtt_ms" | "rtt" => "subscriber_rtt_ms DESC, severity DESC",
        "user_loss_pct" | "loss" => "user_loss_pct DESC, severity DESC",
        "label" => "bottleneck ASC, user_type ASC",
        _ => "severity DESC, users DESC",
    }
}

#[tauri::command]
pub fn analytics_get_network_hotspots(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.run_id();
    let mut conn = db::conn(&req.settings)?;
    let page_size = req.page_size(100, 500);
    let offset = req.offset(100, 500);
    let keyword = req.keyword_like();
    let ads_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_network_hotspot_rank")?;
    let ads_count: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{ads_table}` WHERE analysis_run_id=?"), (&run_id,)).map_err(|err| format!("failed to count analytics network ADS: {err}"))?;
    if ads_count.unwrap_or(0) > 0 {
        let sql = format!(
            "SELECT COALESCE(main_issue_driver,'UNKNOWN') AS bottleneck, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(affected_users AS SIGNED) AS users, CAST(ROUND(COALESCE(poor_experience_user_pct,0),2) AS DOUBLE) AS severity, CAST(ROUND(COALESCE(traffic_gb,0),2) AS DOUBLE) AS traffic_gb, CAST(ROUND(COALESCE(avg_subscriber_rtt_ms,0),2) AS DOUBLE) AS subscriber_rtt_ms, CAST(ROUND(COALESCE(avg_network_rtt_ms,0),2) AS DOUBLE) AS network_rtt_ms, CAST(ROUND(COALESCE(avg_user_loss_pct,0),2) AS DOUBLE) AS user_loss_pct, CAST(ROUND(COALESCE(avg_network_loss_pct,0),2) AS DOUBLE) AS network_loss_pct FROM `{ads_table}` WHERE analysis_run_id=? AND (? IS NULL OR COALESCE(main_issue_driver,'UNKNOWN') LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ? OR bras LIKE ? OR olt LIKE ? OR pon LIKE ?) AND COALESCE(poor_experience_user_pct,0) >= ? ORDER BY {} LIMIT ? OFFSET ?",
            order_sql(&req.sort_by())
        );
        return conn.exec_map(
            sql,
            (&run_id, keyword.clone(), keyword.clone(), keyword.clone(), keyword.clone(), keyword.clone(), keyword, req.min_value(), page_size, offset),
            |(bottleneck, user_type, users, severity, gb, sub_rtt, net_rtt, user_loss, net_loss): (String, String, i64, f64, f64, f64, f64, f64, f64)| MetricCard {
                label: format!("{bottleneck} {user_type}"),
                value: format!("{severity:.2}"),
                hint: format!("source=ads_network_hotspot_rank, bottleneck={bottleneck}, user_type={user_type}, users={users}, severity={severity:.2}, traffic_gb={gb:.2}, subscriber_rtt_ms={sub_rtt:.2}, network_rtt_ms={net_rtt:.2}, user_loss_pct={user_loss:.2}, network_loss_pct={net_loss:.2}, page_size={page_size}, offset={offset}"),
            },
        ).map_err(|err| format!("failed to query analytics network ADS hotspots: {err}"));
    }
    let profile_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?;
    let bottleneck_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_experience_bottleneck")?;
    let sql = format!(
        "SELECT COALESCE(b.bottleneck_type,'UNKNOWN') AS bottleneck, COALESCE(p.user_type,'UNKNOWN') AS user_type, CAST(COUNT(DISTINCT p.user_key) AS SIGNED) AS users, CAST(ROUND(COALESCE(AVG(b.severity_score),0),2) AS DOUBLE) AS severity, CAST(ROUND(COALESCE(SUM(p.total_download_gb),0),2) AS DOUBLE) AS traffic_gb, CAST(ROUND(COALESCE(AVG(p.avg_subscriber_rtt_ms),0),2) AS DOUBLE) AS subscriber_rtt_ms, CAST(ROUND(COALESCE(AVG(p.avg_network_rtt_ms),0),2) AS DOUBLE) AS network_rtt_ms, CAST(ROUND(COALESCE(AVG(p.avg_user_down_loss),0),2) AS DOUBLE) AS user_loss_pct, CAST(ROUND(COALESCE(AVG(p.avg_network_down_loss),0),2) AS DOUBLE) AS network_loss_pct FROM `{profile_table}` p LEFT JOIN `{bottleneck_table}` b ON b.import_batch_id=p.import_batch_id AND b.user_key=p.user_key WHERE p.import_batch_id=? AND (? IS NULL OR COALESCE(b.bottleneck_type,'UNKNOWN') LIKE ? OR COALESCE(p.user_type,'UNKNOWN') LIKE ?) GROUP BY COALESCE(b.bottleneck_type,'UNKNOWN'), COALESCE(p.user_type,'UNKNOWN') HAVING severity >= ? ORDER BY {} LIMIT ? OFFSET ?",
        order_sql(&req.sort_by())
    );
    conn.exec_map(
        sql,
        (&req.import_batch_id, keyword.clone(), keyword.clone(), keyword, req.min_value(), page_size, offset),
        |(bottleneck, user_type, users, severity, gb, sub_rtt, net_rtt, user_loss, net_loss): (String, String, i64, f64, f64, f64, f64, f64, f64)| MetricCard {
            label: format!("{bottleneck} {user_type}"),
            value: format!("{severity:.2}"),
            hint: format!("source=dws_user_experience_bottleneck, bottleneck={bottleneck}, user_type={user_type}, users={users}, severity={severity:.2}, traffic_gb={gb:.2}, subscriber_rtt_ms={sub_rtt:.2}, network_rtt_ms={net_rtt:.2}, user_loss_pct={user_loss:.2}, network_loss_pct={net_loss:.2}, page_size={page_size}, offset={offset}"),
        },
    ).map_err(|err| format!("failed to query analytics network hotspots: {err}"))
}

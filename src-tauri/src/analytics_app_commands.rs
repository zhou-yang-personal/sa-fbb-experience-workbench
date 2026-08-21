use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn order_sql(sort_by: &str) -> &'static str {
    match sort_by {
        "traffic_gb" | "traffic" => "traffic_gb DESC, users DESC",
        "duration_hours" | "duration" => "duration_hours DESC, users DESC",
        "label" | "app_category" => "app_category ASC, user_type ASC",
        _ => "users DESC, traffic_gb DESC",
    }
}

#[tauri::command]
pub fn analytics_get_app_rank(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.run_id();
    let mut conn = db::conn(&req.settings)?;
    let page_size = req.page_size(80, 500);
    let offset = req.offset(80, 500);
    let keyword = req.keyword_like();
    if let Ok(ads_table) = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_app_experience_rank",
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
            let sql = format!("SELECT COALESCE(app_category,'UNKNOWN') AS app_category, COALESCE(app_name,'ALL') AS app_name, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(active_users AS SIGNED) AS users, CAST(ROUND(COALESCE(traffic_gb,0),2) AS DOUBLE) AS traffic_gb, CAST(ROUND(COALESCE(duration_hours,0),2) AS DOUBLE) AS duration_hours, CAST(poor_experience_users AS SIGNED), CAST(ROUND(COALESCE(poor_experience_user_pct,0),2) AS DOUBLE), CAST(ROUND(COALESCE(avg_vmos,0),2) AS DOUBLE), CAST(ROUND(COALESCE(avg_mos,0),2) AS DOUBLE), CAST(ROUND(COALESCE(avg_subscriber_rtt_ms,0),2) AS DOUBLE), CAST(ROUND(COALESCE(avg_network_rtt_ms,0),2) AS DOUBLE), CAST(ROUND(COALESCE(avg_user_loss_pct,0),2) AS DOUBLE), CAST(ROUND(COALESCE(avg_network_loss_pct,0),2) AS DOUBLE), COALESCE(main_issue_driver,'') AS issue FROM `{ads_table}` WHERE analysis_run_id=? AND (? IS NULL OR COALESCE(app_category,'UNKNOWN') LIKE ? OR COALESCE(app_name,'ALL') LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ?) AND active_users >= ? ORDER BY {} LIMIT ? OFFSET ?", order_sql(&req.sort_by()));
            let rows = conn
                .exec_iter(
                    sql,
                    (
                        &run_id,
                        keyword.clone(),
                        keyword.clone(),
                        keyword.clone(),
                        keyword,
                        req.min_value(),
                        page_size,
                        offset,
                    ),
                )
                .map_err(|err| format!("failed to query analytics app ADS rank: {err}"))?;
            return rows
                .map(|row| {
                    let row = row.map_err(|err| format!("failed to decode analytics app ADS row: {err}"))?;
                    let category: String = row.get(0).unwrap_or_default();
                    let app_name: String = row.get(1).unwrap_or_default();
                    let user_type: String = row.get(2).unwrap_or_default();
                    let users: i64 = row.get(3).unwrap_or_default();
                    let gb: f64 = row.get(4).unwrap_or_default();
                    let hours: f64 = row.get(5).unwrap_or_default();
                    let poor_users: i64 = row.get(6).unwrap_or_default();
                    let poor_pct: f64 = row.get(7).unwrap_or_default();
                    let vmos: f64 = row.get(8).unwrap_or_default();
                    let mos: f64 = row.get(9).unwrap_or_default();
                    let subscriber_rtt: f64 = row.get(10).unwrap_or_default();
                    let network_rtt: f64 = row.get(11).unwrap_or_default();
                    let user_loss: f64 = row.get(12).unwrap_or_default();
                    let network_loss: f64 = row.get(13).unwrap_or_default();
                    let issue: String = row.get(14).unwrap_or_default();
                    Ok(MetricCard {
                        label: format!("{category} {app_name} {user_type}"),
                        value: users.to_string(),
                        hint: format!("source=ads_app_experience_rank, app_category={category}, app_name={app_name}, user_type={user_type}, users={users}, traffic_gb={gb:.2}, duration_hours={hours:.2}, poor_experience_users={poor_users}, poor_experience_user_pct={poor_pct:.2}, avg_vmos={vmos:.2}, avg_mos={mos:.2}, subscriber_rtt_ms={subscriber_rtt:.2}, network_rtt_ms={network_rtt:.2}, user_loss_pct={user_loss:.2}, network_loss_pct={network_loss:.2}, issue_driver={issue}, page_size={page_size}, offset={offset}"),
                    })
                })
                .collect();
        }
    }
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_app_category_daily",
    )?;
    let sql = format!("SELECT COALESCE(app_category,'UNKNOWN') AS app_category, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(SUM(active_users) AS SIGNED) AS users, CAST(ROUND(COALESCE(SUM(total_download_gb),0),2) AS DOUBLE) AS traffic_gb, CAST(ROUND(COALESCE(SUM(total_game_hours),0),2) AS DOUBLE) AS duration_hours FROM `{table}` WHERE import_batch_id=? AND (? IS NULL OR COALESCE(app_category,'UNKNOWN') LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ?) GROUP BY COALESCE(app_category,'UNKNOWN'), COALESCE(user_type,'UNKNOWN') HAVING users >= ? ORDER BY {} LIMIT ? OFFSET ?", order_sql(&req.sort_by()));
    conn.exec_map(sql, (&req.import_batch_id, keyword.clone(), keyword.clone(), keyword, req.min_value(), page_size, offset), |(category, user_type, users, gb, hours): (String, String, i64, f64, f64)| MetricCard {
        label: format!("{category} {user_type}"),
        value: users.to_string(),
        hint: format!("source=dws_app_category_daily, app_category={category}, user_type={user_type}, users={users}, traffic_gb={gb:.2}, duration_hours={hours:.2}, page_size={page_size}, offset={offset}"),
    }).map_err(|err| format!("failed to query analytics app rank: {err}"))
}

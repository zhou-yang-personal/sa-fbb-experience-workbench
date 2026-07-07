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
    let ads_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_app_experience_rank")?;
    let ads_count: Option<i64> = conn.exec_first(format!("SELECT CAST(COUNT(*) AS SIGNED) FROM `{ads_table}` WHERE analysis_run_id=?"), (&run_id,)).map_err(|err| format!("failed to count analytics app ADS: {err}"))?;
    if ads_count.unwrap_or(0) > 0 {
        let sql = format!(
            "SELECT COALESCE(app_category,'UNKNOWN') AS app_category, COALESCE(app_name,'ALL') AS app_name, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(active_users AS SIGNED) AS users, CAST(ROUND(COALESCE(traffic_gb,0),2) AS DOUBLE) AS traffic_gb, CAST(ROUND(COALESCE(duration_hours,0),2) AS DOUBLE) AS duration_hours, COALESCE(main_issue_driver,'') AS issue FROM `{ads_table}` WHERE analysis_run_id=? AND (? IS NULL OR COALESCE(app_category,'UNKNOWN') LIKE ? OR COALESCE(app_name,'ALL') LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ?) AND active_users >= ? ORDER BY {} LIMIT ? OFFSET ?",
            order_sql(&req.sort_by())
        );
        return conn.exec_map(
            sql,
            (&run_id, keyword.clone(), keyword.clone(), keyword.clone(), keyword, req.min_value(), page_size, offset),
            |(category, app_name, user_type, users, gb, hours, issue): (String, String, String, i64, f64, f64, String)| MetricCard {
                label: format!("{category} {app_name} {user_type}"),
                value: users.to_string(),
                hint: format!("source=ads_app_experience_rank, app_category={category}, app_name={app_name}, user_type={user_type}, users={users}, traffic_gb={gb:.2}, duration_hours={hours:.2}, issue_driver={issue}, page_size={page_size}, offset={offset}"),
            },
        ).map_err(|err| format!("failed to query analytics app ADS rank: {err}"));
    }
    let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_app_category_daily")?;
    let sql = format!(
        "SELECT COALESCE(app_category,'UNKNOWN') AS app_category, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(SUM(active_users) AS SIGNED) AS users, CAST(ROUND(COALESCE(SUM(total_download_gb),0),2) AS DOUBLE) AS traffic_gb, CAST(ROUND(COALESCE(SUM(total_game_hours),0),2) AS DOUBLE) AS duration_hours FROM `{table}` WHERE import_batch_id=? AND (? IS NULL OR COALESCE(app_category,'UNKNOWN') LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ?) GROUP BY COALESCE(app_category,'UNKNOWN'), COALESCE(user_type,'UNKNOWN') HAVING users >= ? ORDER BY {} LIMIT ? OFFSET ?",
        order_sql(&req.sort_by())
    );
    conn.exec_map(
        sql,
        (&req.import_batch_id, keyword.clone(), keyword.clone(), keyword, req.min_value(), page_size, offset),
        |(category, user_type, users, gb, hours): (String, String, i64, f64, f64)| MetricCard {
            label: format!("{category} {user_type}"),
            value: users.to_string(),
            hint: format!("source=dws_app_category_daily, app_category={category}, user_type={user_type}, users={users}, traffic_gb={gb:.2}, duration_hours={hours:.2}, page_size={page_size}, offset={offset}"),
        },
    ).map_err(|err| format!("failed to query analytics app rank: {err}"))
}

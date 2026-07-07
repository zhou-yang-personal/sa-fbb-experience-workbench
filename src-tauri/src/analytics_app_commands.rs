use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

#[tauri::command]
pub fn analytics_get_app_rank(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_app_category_daily")?;
    let sql = format!(
        "SELECT COALESCE(app_category,'UNKNOWN'), COALESCE(user_type,'UNKNOWN'), CAST(SUM(active_users) AS SIGNED), CAST(ROUND(COALESCE(SUM(total_download_gb),0),2) AS DOUBLE), CAST(ROUND(COALESCE(SUM(total_game_hours),0),2) AS DOUBLE) FROM `{table}` WHERE import_batch_id=? GROUP BY COALESCE(app_category,'UNKNOWN'), COALESCE(user_type,'UNKNOWN') ORDER BY SUM(active_users) DESC, SUM(total_download_gb) DESC LIMIT 80"
    );
    conn.exec_map(
        sql,
        (&req.import_batch_id,),
        |(category, user_type, users, gb, hours): (String, String, i64, f64, f64)| MetricCard {
            label: format!("{category} {user_type}"),
            value: users.to_string(),
            hint: format!("app_category={category}, user_type={user_type}, users={users}, traffic_gb={gb:.2}, duration_hours={hours:.2}"),
        },
    ).map_err(|err| format!("failed to query analytics app rank: {err}"))
}

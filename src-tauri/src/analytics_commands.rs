use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn run_id(req: &DashboardRequest) -> String {
    req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string())
}

#[tauri::command]
pub fn analytics_get_kpi_summary(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = run_id(&req);
    let mut conn = db::conn(&req.settings)?;
    let user_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dws_user_daily_profile")?;
    let lead_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_migration_lead_user")?;
    let user_sql = format!("SELECT CAST(COUNT(DISTINCT user_key) AS SIGNED), CAST(COUNT(DISTINCT CASE WHEN user_type='CABLE' THEN user_key END) AS SIGNED), CAST(COUNT(DISTINCT CASE WHEN user_type='FTTH' THEN user_key END) AS SIGNED), CAST(ROUND(COALESCE(SUM(total_download_gb),0),2) AS DOUBLE), CAST(ROUND(COALESCE(SUM(total_game_hours),0),2) AS DOUBLE) FROM `{user_table}` WHERE import_batch_id=?");
    let (users, cable, ftth, traffic, game_hours): (i64, i64, i64, f64, f64) = conn.exec_first(user_sql, (&req.import_batch_id,)).map_err(|err| format!("failed to query analytics KPI users: {err}"))?.unwrap_or((0, 0, 0, 0.0, 0.0));
    let lead_sql = format!("SELECT CAST(COUNT(DISTINCT user_key) AS SIGNED) FROM `{lead_table}` WHERE analysis_run_id=?");
    let leads: i64 = conn.exec_first(lead_sql, (&run_id,)).map_err(|err| format!("failed to query analytics KPI leads: {err}"))?.unwrap_or(0);
    Ok(vec![
        MetricCard { label: "Total Users".into(), value: users.to_string(), hint: "source=dws_user_daily_profile".into() },
        MetricCard { label: "Cable Users".into(), value: cable.to_string(), hint: "filter=user_type=CABLE".into() },
        MetricCard { label: "FTTH Users".into(), value: ftth.to_string(), hint: "filter=user_type=FTTH".into() },
        MetricCard { label: "Total Traffic GB".into(), value: format!("{traffic:.2}"), hint: "metric=sum(total_download_gb)".into() },
        MetricCard { label: "Game Hours".into(), value: format!("{game_hours:.2}"), hint: "metric=sum(total_game_hours)".into() },
        MetricCard { label: "SA Lead Users".into(), value: leads.to_string(), hint: format!("analysis_run_id={run_id}") },
    ])
}

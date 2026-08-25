use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn run_id(req: &DashboardRequest) -> String {
    req.analysis_run_id
        .clone()
        .unwrap_or_else(|| "RUN_DEFAULT".to_string())
}

#[tauri::command]
pub fn analytics_get_lead_evidence(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = run_id(&req);
    let mut conn = db::conn(&req.settings)?;
    let lead_table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_migration_lead_user",
    )?;
    let sql = format!(
        "SELECT user_key, COALESCE(user_type,'UNKNOWN'), COALESCE(lead_type,'UNKNOWN'), COALESCE(demand_score,0), COALESCE(migration_motive_score,0), COALESCE(recommended_offer,'') FROM `{lead_table}` WHERE analysis_run_id=? ORDER BY demand_score DESC, migration_motive_score DESC LIMIT 200"
    );
    conn.exec_map(
        sql,
        (&run_id,),
        |(user_key, user_type, lead_type, demand, motive, offer): (String, String, String, i64, i64, String)| MetricCard {
            label: user_key,
            value: demand.to_string(),
            hint: format!("user_type={user_type}, lead_type={lead_type}, demand_score={demand}, migration_motive_score={motive}, recommended_offer={offer}"),
        },
    ).map_err(|err| format!("failed to query analytics lead evidence: {err}"))
}

#[tauri::command]
pub fn analytics_get_lead_stage_summary(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = run_id(&req);
    let mut conn = db::conn(&req.settings)?;
    let detail_table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_lead_evidence_detail",
    )?;
    let detail_ready = batch_tables::table_has_rows(&mut conn, &detail_table)?;
    let source_table = if detail_ready {
        detail_table
    } else {
        batch_tables::resolve_table(
            &req.settings,
            &req.import_batch_id,
            "ads_migration_lead_user",
        )?
    };
    let sql = format!(
        "SELECT COALESCE(lead_type,'UNKNOWN') AS lead_type, COALESCE(user_type,'UNKNOWN') AS user_type, CAST(COUNT(DISTINCT user_key) AS SIGNED) AS users, CAST(ROUND(COALESCE(AVG(demand_score),0),2) AS DOUBLE) AS avg_demand, CAST(ROUND(COALESCE(AVG(migration_motive_score),0),2) AS DOUBLE) AS avg_motive FROM `{source_table}` WHERE analysis_run_id=? GROUP BY COALESCE(lead_type,'UNKNOWN'), COALESCE(user_type,'UNKNOWN') ORDER BY lead_type, user_type"
    );
    conn.exec_map(
        sql,
        (&run_id,),
        |(lead_type, user_type, users, avg_demand, avg_motive): (String, String, i64, f64, f64)| MetricCard {
            label: format!("{lead_type} · {user_type}"),
            value: users.to_string(),
            hint: format!("source={source_table}, lead_type={lead_type}, user_type={user_type}, users={users}, avg_demand_score={avg_demand:.2}, avg_migration_motive_score={avg_motive:.2}"),
        },
    )
    .map_err(|err| format!("failed to query full lead stage summary: {err}"))
}

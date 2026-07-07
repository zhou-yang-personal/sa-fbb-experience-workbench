use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn run_id(req: &DashboardRequest) -> String {
    req.analysis_run_id.clone().unwrap_or_else(|| "RUN_DEFAULT".to_string())
}

#[tauri::command]
pub fn analytics_get_lead_evidence(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = run_id(&req);
    let mut conn = db::conn(&req.settings)?;
    let lead_table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_migration_lead_user")?;
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

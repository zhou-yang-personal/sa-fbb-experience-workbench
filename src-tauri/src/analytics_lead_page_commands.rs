use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

#[tauri::command]
pub fn analytics_get_lead_evidence_page(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req.run_id();
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(&req.settings, &req.import_batch_id, "ads_migration_lead_user")?;
    let page_size = req.page_size(200, 500);
    let offset = req.offset(200, 500);
    let keyword = req.keyword_like();
    let sql = format!(
        "SELECT user_key, COALESCE(user_type,'UNKNOWN'), COALESCE(lead_type,'UNKNOWN'), COALESCE(demand_score,0), COALESCE(migration_motive_score,0), COALESCE(recommended_offer,'') FROM `{table}` WHERE analysis_run_id=? AND (? IS NULL OR user_key LIKE ? OR COALESCE(lead_type,'UNKNOWN') LIKE ?) AND COALESCE(demand_score,0) >= ? ORDER BY demand_score DESC, migration_motive_score DESC LIMIT ? OFFSET ?"
    );
    conn.exec_map(
        sql,
        (&run_id, keyword.clone(), keyword.clone(), keyword, req.min_value(), page_size, offset),
        |(user_key, user_type, lead_type, demand, motive, offer): (String, String, String, i64, i64, String)| MetricCard {
            let label = user_key.clone();
            MetricCard {
                label,
                value: demand.to_string(),
                hint: format!("user_key={user_key}, user_type={user_type}, lead_type={lead_type}, demand_score={demand}, migration_motive_score={motive}, recommended_offer={offer}, page_size={page_size}, offset={offset}"),
            }
        },
    ).map_err(|err| format!("failed to query analytics lead evidence page: {err}"))
}

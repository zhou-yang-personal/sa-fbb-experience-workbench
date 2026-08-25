use crate::batch_tables;
use crate::models::{ack, CommandAck, EtlRequest};
use crate::sql_runner;

const SQL: &str = include_str!("../../database/sql/dws_to_ads/003b_analytics_app_rank.sql");
const EXPERIENCE_DWS_V2_SQL: &str =
    include_str!("../../database/sql/clean_to_dws/003_experience_metrics_v2.sql");
const EXPERIENCE_ADS_V2_SQL: &str =
    include_str!("../../database/sql/dws_to_ads/004_experience_app_v2.sql");
const EXPERIENCE_HOURLY_DWS_V2_SQL: &str =
    include_str!("../../database/sql/clean_to_dws/004_experience_hourly_v2.sql");
const EXPERIENCE_HOURLY_ADS_V2_SQL: &str =
    include_str!("../../database/sql/dws_to_ads/005_experience_hourly_v2.sql");

#[tauri::command]
pub fn analytics_materialize_app_rank(req: EtlRequest) -> Result<CommandAck, String> {
    crate::migrations::ensure_experience_policy_schema(&req.settings)?;
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let run_id = req
        .analysis_run_id
        .clone()
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let bound = sql_runner::bind_batch_params(SQL, &req.import_batch_id, Some(&run_id));
    let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
    let rows = sql_runner::execute_script(&req.settings, &sql)?;
    let dws_v2_bound =
        sql_runner::bind_batch_params(EXPERIENCE_DWS_V2_SQL, &req.import_batch_id, Some(&run_id));
    let dws_v2_sql =
        batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &dws_v2_bound)?;
    let dws_v2_rows = sql_runner::execute_script(&req.settings, &dws_v2_sql)?;
    let ads_v2_bound =
        sql_runner::bind_batch_params(EXPERIENCE_ADS_V2_SQL, &req.import_batch_id, Some(&run_id));
    let ads_v2_sql =
        batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &ads_v2_bound)?;
    let ads_v2_rows = sql_runner::execute_script(&req.settings, &ads_v2_sql)?;
    let hourly_dws_bound = sql_runner::bind_batch_params(
        EXPERIENCE_HOURLY_DWS_V2_SQL,
        &req.import_batch_id,
        Some(&run_id),
    );
    let hourly_dws_sql =
        batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &hourly_dws_bound)?;
    let hourly_dws_rows = sql_runner::execute_script(&req.settings, &hourly_dws_sql)?;
    let hourly_ads_bound = sql_runner::bind_batch_params(
        EXPERIENCE_HOURLY_ADS_V2_SQL,
        &req.import_batch_id,
        Some(&run_id),
    );
    let hourly_ads_sql =
        batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &hourly_ads_bound)?;
    let hourly_ads_rows = sql_runner::execute_script(&req.settings, &hourly_ads_sql)?;
    Ok(ack(format!(
        "analytics app rank materialized: analysis_run_id={run_id}; legacy_rows={rows}; experience_v2_dws_rows={dws_v2_rows}; experience_v2_ads_rows={ads_v2_rows}; hourly_dws_rows={hourly_dws_rows}; hourly_ads_rows={hourly_ads_rows}"
    )))
}

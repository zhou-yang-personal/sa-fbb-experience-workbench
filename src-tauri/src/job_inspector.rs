use mysql::prelude::*;

use crate::db;
use crate::models::{MetricCard, MySqlSettings};

pub fn recent_steps(settings: &MySqlSettings, import_batch_id: &str, limit: u64) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(settings)?;
    conn.exec_map(
        "SELECT CONCAT(j.job_type, ':', s.step_name), s.status, CONCAT('target=', COALESCE(s.target_table,''), ', rows=', COALESCE(s.affected_rows,0), ', message=', COALESCE(s.message,'')) FROM meta_etl_job j JOIN meta_etl_job_step s ON s.job_id=j.job_id WHERE j.import_batch_id=? ORDER BY s.started_at DESC LIMIT ?",
        (import_batch_id, limit),
        |(label, value, hint): (String, String, String)| MetricCard { label, value, hint },
    ).map_err(|err| format!("failed to query ETL job steps: {err}"))
}

pub fn failed_steps(settings: &MySqlSettings, import_batch_id: &str) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(settings)?;
    conn.exec_map(
        "SELECT CONCAT(j.job_type, ':', s.step_name), s.status, COALESCE(s.message,'') FROM meta_etl_job j JOIN meta_etl_job_step s ON s.job_id=j.job_id WHERE j.import_batch_id=? AND s.status='failed' ORDER BY s.started_at DESC LIMIT 30",
        (import_batch_id,),
        |(label, value, hint): (String, String, String)| MetricCard { label, value, hint },
    ).map_err(|err| format!("failed to query failed ETL steps: {err}"))
}

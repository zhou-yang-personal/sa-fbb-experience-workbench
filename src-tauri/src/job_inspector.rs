use mysql::prelude::*;
use mysql::{Params, Value};

use crate::db;
use crate::models::{EtlJobStepRow, EtlJobStepsRequest, MetricCard, MySqlSettings};

fn string_value(value: String) -> Value {
    Value::Bytes(value.into_bytes())
}

fn normalized_filter(value: &Option<String>) -> Option<String> {
    let normalized = value.as_ref()?.trim();
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("ALL") {
        None
    } else {
        Some(normalized.to_string())
    }
}

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

pub fn job_step_rows(req: &EtlJobStepsRequest) -> Result<Vec<EtlJobStepRow>, String> {
    let mut conn = db::conn(&req.settings)?;
    let mut where_sql = vec!["j.import_batch_id=?".to_string()];
    let mut params = vec![string_value(req.import_batch_id.clone())];
    if let Some(job_id) = normalized_filter(&req.job_id) {
        where_sql.push("j.job_id=?".to_string());
        params.push(string_value(job_id));
    }
    if let Some(status) = normalized_filter(&req.status) {
        where_sql.push("s.status=?".to_string());
        params.push(string_value(status));
    }
    let limit = req.limit.unwrap_or(100).clamp(1, 500);
    params.push(Value::UInt(limit));
    let sql = format!(
        "SELECT j.job_id, j.job_type, s.step_name, s.target_table, s.status, s.affected_rows, DATE_FORMAT(s.started_at, '%Y-%m-%d %H:%i:%s'), DATE_FORMAT(s.finished_at, '%Y-%m-%d %H:%i:%s'), s.message FROM meta_etl_job j JOIN meta_etl_job_step s ON s.job_id=j.job_id WHERE {} ORDER BY COALESCE(s.started_at, j.started_at) DESC, s.id DESC LIMIT ?",
        where_sql.join(" AND ")
    );
    conn.exec_map(
        sql,
        Params::Positional(params),
        |(job_id, job_type, step_name, target_table, status, affected_rows, started_at, finished_at, message)| EtlJobStepRow {
            job_id,
            job_type,
            step_name,
            target_table,
            status,
            affected_rows,
            started_at,
            finished_at,
            message,
        },
    ).map_err(|err| format!("failed to query ETL job step rows: {err}"))
}

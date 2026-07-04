use mysql::prelude::*;
use uuid::Uuid;

use crate::db;
use crate::models::MySqlSettings;
use crate::sql_runner;

pub struct JobStep<'a> {
    pub step_name: &'a str,
    pub source_table: &'a str,
    pub target_table: &'a str,
    pub sql_template: &'a str,
    pub sql: String,
}

pub fn run_job(settings: &MySqlSettings, import_batch_id: &str, job_type: &str, steps: Vec<JobStep<'_>>) -> Result<String, String> {
    let job_id = format!("JOB_{}", Uuid::new_v4().simple());
    let mut conn = db::conn(settings)?;
    conn.exec_drop(
        "INSERT INTO meta_etl_job (job_id, import_batch_id, job_type, status, current_step, started_at) VALUES (?, ?, ?, 'running', ?, NOW())",
        (&job_id, import_batch_id, job_type, steps.first().map(|step| step.step_name).unwrap_or("start")),
    )
    .map_err(|err| format!("failed to create ETL job: {err}"))?;

    let mut total_rows = 0_u64;
    for step in steps {
        conn.exec_drop(
            "INSERT INTO meta_etl_job_step (job_id, step_name, source_table, target_table, sql_template, status, started_at) VALUES (?, ?, ?, ?, ?, 'running', NOW())",
            (&job_id, step.step_name, step.source_table, step.target_table, step.sql_template),
        )
        .map_err(|err| format!("failed to create ETL job step: {err}"))?;
        conn.exec_drop(
            "UPDATE meta_etl_job SET current_step=? WHERE job_id=?",
            (step.step_name, &job_id),
        )
        .map_err(|err| format!("failed to update ETL job current step: {err}"))?;

        match sql_runner::execute_script(settings, &step.sql) {
            Ok(rows) => {
                total_rows += rows;
                conn.exec_drop(
                    "UPDATE meta_etl_job_step SET status='success', finished_at=NOW(), affected_rows=?, message='step finished' WHERE job_id=? AND step_name=?",
                    (rows, &job_id, step.step_name),
                )
                .map_err(|err| format!("failed to mark ETL step success: {err}"))?;
            }
            Err(err) => {
                let _ = conn.exec_drop(
                    "UPDATE meta_etl_job_step SET status='failed', finished_at=NOW(), message=? WHERE job_id=? AND step_name=?",
                    (&err, &job_id, step.step_name),
                );
                let _ = conn.exec_drop(
                    "UPDATE meta_etl_job SET status='failed', finished_at=NOW(), error_message=? WHERE job_id=?",
                    (&err, &job_id),
                );
                return Err(format!("ETL job failed: job_id={job_id}, step={}, error={err}", step.step_name));
            }
        }
    }

    conn.exec_drop(
        "UPDATE meta_etl_job SET status='success', finished_at=NOW(), affected_rows=?, current_step='finished' WHERE job_id=?",
        (total_rows, &job_id),
    )
    .map_err(|err| format!("failed to mark ETL job success: {err}"))?;
    Ok(format!("ETL job finished: job_id={job_id}, affected_rows={total_rows}"))
}

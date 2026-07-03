use mysql::prelude::*;

use crate::db;
use crate::models::MySqlSettings;

pub fn split_sql_script(script: &str) -> Vec<String> {
    script
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n")
        .split(';')
        .map(str::trim)
        .filter(|stmt| !stmt.is_empty())
        .map(|stmt| format!("{stmt};"))
        .collect()
}

pub fn escape_sql_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "''")
}

pub fn bind_batch_params(sql: &str, import_batch_id: &str, analysis_run_id: Option<&str>) -> String {
    let mut bound = sql.replace(":import_batch_id", &format!("'{}'", escape_sql_literal(import_batch_id)));
    if let Some(run_id) = analysis_run_id {
        bound = bound.replace(":analysis_run_id", &format!("'{}'", escape_sql_literal(run_id)));
    }
    bound
}

pub fn execute_script(settings: &MySqlSettings, script: &str) -> Result<u64, String> {
    let mut conn = db::conn(settings)?;
    let mut total = 0_u64;
    for stmt in split_sql_script(script) {
        conn.query_drop(&stmt)
            .map_err(|err| format!("failed to execute SQL statement: {err}; statement={stmt}"))?;
        total += conn.affected_rows();
    }
    Ok(total)
}

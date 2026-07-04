use mysql::prelude::*;

use crate::db;
use crate::models::MySqlSettings;

pub fn split_sql_script(script: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut chars = script.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_backtick = false;
    let mut in_line_comment = false;
    let mut previous = '\0';

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                current.push(ch);
            }
            previous = ch;
            continue;
        }

        if !in_single_quote && !in_double_quote && !in_backtick {
            if ch == '-' && chars.peek() == Some(&'-') {
                let _ = chars.next();
                in_line_comment = true;
                previous = '-';
                continue;
            }
            if ch == '#' {
                in_line_comment = true;
                previous = ch;
                continue;
            }
        }

        match ch {
            '\'' if !in_double_quote && !in_backtick && previous != '\\' => in_single_quote = !in_single_quote,
            '"' if !in_single_quote && !in_backtick && previous != '\\' => in_double_quote = !in_double_quote,
            '`' if !in_single_quote && !in_double_quote => in_backtick = !in_backtick,
            ';' if !in_single_quote && !in_double_quote && !in_backtick => {
                let stmt = current.trim();
                if !stmt.is_empty() {
                    statements.push(format!("{stmt};"));
                }
                current.clear();
                previous = ch;
                continue;
            }
            _ => {}
        }

        current.push(ch);
        previous = ch;
    }

    let stmt = current.trim();
    if !stmt.is_empty() {
        statements.push(format!("{stmt};"));
    }
    statements
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

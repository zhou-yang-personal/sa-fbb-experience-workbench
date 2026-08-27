use mysql::prelude::*;
use std::cell::RefCell;

use crate::db;
use crate::models::MySqlSettings;

#[derive(Clone, Debug)]
pub struct SqlExecutionEvent {
    pub statement_index: usize,
    pub statement_count: usize,
    pub status: &'static str,
    pub statement_preview: String,
    pub duration_ms: i64,
    pub affected_rows: Option<u64>,
    pub error: Option<String>,
    pub connection_id: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct SqlExecutionReport {
    pub affected_rows: u64,
    pub connection_id: u64,
}

type SqlExecutionObserver = Box<dyn Fn(SqlExecutionEvent)>;

thread_local! {
    static SQL_EXECUTION_OBSERVER: RefCell<Option<SqlExecutionObserver>> = RefCell::new(None);
}

fn notify_sql_execution(event: SqlExecutionEvent) {
    SQL_EXECUTION_OBSERVER.with(|slot| {
        if let Some(observer) = slot.borrow().as_ref() {
            observer(event);
        }
    });
}

pub fn with_sql_execution_observer<T, F, O>(observer: O, action: F) -> T
where
    F: FnOnce() -> T,
    O: Fn(SqlExecutionEvent) + 'static,
{
    let previous = SQL_EXECUTION_OBSERVER.with(|slot| slot.replace(Some(Box::new(observer))));
    let result = action();
    SQL_EXECUTION_OBSERVER.with(|slot| {
        slot.replace(previous);
    });
    result
}

fn statement_preview(statement: &str) -> String {
    let normalized = statement.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut preview = normalized.chars().take(260).collect::<String>();
    if normalized.chars().count() > 260 {
        preview.push('…');
    }
    preview
}

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
            '\'' if !in_double_quote && !in_backtick && previous != '\\' => {
                in_single_quote = !in_single_quote
            }
            '"' if !in_single_quote && !in_backtick && previous != '\\' => {
                in_double_quote = !in_double_quote
            }
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

pub fn bind_batch_params(
    sql: &str,
    import_batch_id: &str,
    analysis_run_id: Option<&str>,
) -> String {
    let mut bound = sql.replace(
        ":import_batch_id",
        &format!("'{}'", escape_sql_literal(import_batch_id)),
    );
    if let Some(run_id) = analysis_run_id {
        bound = bound.replace(
            ":analysis_run_id",
            &format!("'{}'", escape_sql_literal(run_id)),
        );
    }
    bound
}

pub fn execute_script(settings: &MySqlSettings, script: &str) -> Result<u64, String> {
    let mut conn = db::conn(settings)?;
    let connection_id: Option<u64> = conn
        .query_first("SELECT CONNECTION_ID()")
        .map_err(|err| format!("failed to read MySQL connection id: {err}"))?;
    let mut total = 0_u64;
    let statements = split_sql_script(script);
    let statement_count = statements.len();
    for (offset, stmt) in statements.into_iter().enumerate() {
        let statement_index = offset + 1;
        let preview = statement_preview(&stmt);
        let started = std::time::Instant::now();
        notify_sql_execution(SqlExecutionEvent {
            statement_index,
            statement_count,
            status: "running",
            statement_preview: preview.clone(),
            duration_ms: 0,
            affected_rows: None,
            error: None,
            connection_id,
        });
        if let Err(err) = conn.query_drop(&stmt) {
            let error = err.to_string();
            notify_sql_execution(SqlExecutionEvent {
                statement_index,
                statement_count,
                status: "failed",
                statement_preview: preview,
                duration_ms: started.elapsed().as_millis().min(i64::MAX as u128) as i64,
                affected_rows: None,
                error: Some(error.clone()),
                connection_id,
            });
            return Err(format!(
                "failed to execute SQL statement: {error}; statement={stmt}"
            ));
        }
        let affected_rows = conn.affected_rows();
        total += affected_rows;
        notify_sql_execution(SqlExecutionEvent {
            statement_index,
            statement_count,
            status: "success",
            statement_preview: preview,
            duration_ms: started.elapsed().as_millis().min(i64::MAX as u128) as i64,
            affected_rows: Some(affected_rows),
            error: None,
            connection_id,
        });
    }
    Ok(total)
}

pub fn execute_script_transactional<F>(
    settings: &MySqlSettings,
    script: &str,
    on_connection: F,
) -> Result<SqlExecutionReport, String>
where
    F: FnOnce(u64) -> Result<(), String>,
{
    let mut conn = db::conn(settings)?;
    let connection_id: u64 = conn
        .query_first("SELECT CONNECTION_ID()")
        .map_err(|err| format!("failed to read MySQL connection id: {err}"))?
        .ok_or_else(|| "MySQL returned no connection id".to_string())?;
    on_connection(connection_id)?;
    let statements = split_sql_script(script);
    let statement_count = statements.len();
    let mut transaction = conn
        .start_transaction(mysql::TxOpts::default())
        .map_err(|err| format!("failed to start MySQL partition transaction: {err}"))?;
    let mut total = 0_u64;
    for (offset, stmt) in statements.into_iter().enumerate() {
        let statement_index = offset + 1;
        let preview = statement_preview(&stmt);
        let started = std::time::Instant::now();
        notify_sql_execution(SqlExecutionEvent {
            statement_index,
            statement_count,
            status: "running",
            statement_preview: preview.clone(),
            duration_ms: 0,
            affected_rows: None,
            error: None,
            connection_id: Some(connection_id),
        });
        if let Err(err) = transaction.query_drop(&stmt) {
            let error = err.to_string();
            notify_sql_execution(SqlExecutionEvent {
                statement_index,
                statement_count,
                status: "failed",
                statement_preview: preview,
                duration_ms: started.elapsed().as_millis().min(i64::MAX as u128) as i64,
                affected_rows: None,
                error: Some(error.clone()),
                connection_id: Some(connection_id),
            });
            let _ = transaction.rollback();
            return Err(format!(
                "failed to execute partition SQL statement: {error}; statement={stmt}"
            ));
        }
        let affected_rows = transaction.affected_rows();
        total = total.saturating_add(affected_rows);
        notify_sql_execution(SqlExecutionEvent {
            statement_index,
            statement_count,
            status: "success",
            statement_preview: preview,
            duration_ms: started.elapsed().as_millis().min(i64::MAX as u128) as i64,
            affected_rows: Some(affected_rows),
            error: None,
            connection_id: Some(connection_id),
        });
    }
    transaction
        .commit()
        .map_err(|err| format!("failed to commit MySQL partition transaction: {err}"))?;
    Ok(SqlExecutionReport {
        affected_rows: total,
        connection_id,
    })
}

#[cfg(test)]
mod tests {
    use super::{split_sql_script, statement_preview};

    #[test]
    fn statement_preview_is_single_line_and_bounded() {
        let sql = format!("INSERT\nINTO target_table VALUES ('{}')", "x".repeat(400));
        let preview = statement_preview(&sql);
        assert!(!preview.contains('\n'));
        assert!(preview.chars().count() <= 261);
        assert!(preview.ends_with('…'));
    }

    #[test]
    fn transactional_partition_script_keeps_delete_and_insert_separate() {
        let statements = split_sql_script(
            "DELETE FROM target WHERE stat_date='2026-08-20' AND hour_of_day=13; INSERT INTO target SELECT 1;",
        );
        assert_eq!(statements.len(), 2);
    }
}

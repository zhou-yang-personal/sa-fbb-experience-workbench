use mysql::{OptsBuilder, Pool, PooledConn};
use mysql::prelude::*;

use crate::models::MySqlSettings;

pub fn pool(settings: &MySqlSettings) -> Result<Pool, String> {
    let builder = OptsBuilder::new()
        .ip_or_hostname(Some(settings.host.clone()))
        .tcp_port(settings.port)
        .db_name(Some(settings.database.clone()))
        .user(Some(settings.user.clone()))
        .pass(Some(settings.secret.clone()));

    Pool::new(builder).map_err(|err| format!("failed to create MySQL pool: {err}"))
}

pub fn conn(settings: &MySqlSettings) -> Result<PooledConn, String> {
    pool(settings)?.get_conn().map_err(|err| format!("failed to connect MySQL: {err}"))
}

pub fn ping(settings: &MySqlSettings) -> Result<String, String> {
    let mut conn = conn(settings)?;
    let version: Option<String> = conn
        .query_first("SELECT VERSION()")
        .map_err(|err| format!("failed to query MySQL version: {err}"))?;
    Ok(format!("MySQL connected: {}", version.unwrap_or_else(|| "unknown".to_string())))
}

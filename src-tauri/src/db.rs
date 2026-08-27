use mysql::prelude::*;
use mysql::{LocalInfileHandler, OptsBuilder, Pool, PooledConn};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use crate::models::MySqlSettings;

pub const AGGREGATION_LOCK_NAME: &str = "sa_fbb_dws_ads_aggregate";

pub struct NamedLockGuard {
    conn: Option<PooledConn>,
    lock_name: String,
}

impl Drop for NamedLockGuard {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.as_mut() {
            let _: Result<Option<i8>, _> =
                conn.exec_first("SELECT RELEASE_LOCK(?)", (&self.lock_name,));
        }
    }
}

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
    pool(settings)?
        .get_conn()
        .map_err(|err| format!("failed to connect MySQL: {err}"))
}

pub fn ping(settings: &MySqlSettings) -> Result<String, String> {
    let mut conn = conn(settings)?;
    let version: Option<String> = conn
        .query_first("SELECT VERSION()")
        .map_err(|err| format!("failed to query MySQL version: {err}"))?;
    Ok(format!(
        "MySQL connected: {}",
        version.unwrap_or_else(|| "unknown".to_string())
    ))
}

pub fn acquire_named_lock(
    settings: &MySqlSettings,
    lock_name: &str,
) -> Result<NamedLockGuard, String> {
    let mut conn = conn(settings)?;
    let acquired: Option<i8> = conn
        .exec_first("SELECT GET_LOCK(?, 0)", (lock_name,))
        .map_err(|err| format!("failed to acquire MySQL task lock {lock_name}: {err}"))?;
    if acquired != Some(1) {
        return Err(format!(
            "another DWS/ADS aggregation is already running on this MySQL instance; lock={lock_name}"
        ));
    }
    Ok(NamedLockGuard {
        conn: Some(conn),
        lock_name: lock_name.to_string(),
    })
}

pub fn local_infile_handler_for_path_with_progress<F>(
    file_path: &str,
    mut on_bytes_transferred: F,
) -> Result<LocalInfileHandler, String>
where
    F: FnMut(u64) + Send + 'static,
{
    let allowed_path = std::fs::canonicalize(file_path)
        .map_err(|err| format!("failed to resolve selected LOCAL INFILE path: {err}"))?;
    if !allowed_path.is_file() {
        return Err(format!(
            "selected LOCAL INFILE path is not a file: {}",
            allowed_path.display()
        ));
    }

    Ok(LocalInfileHandler::new(move |requested_name, writer| {
        let requested_path = validate_local_infile_request(&allowed_path, requested_name)?;
        let file = File::open(requested_path)?;
        let mut reader = BufReader::with_capacity(1024 * 1024, file);
        let mut buffer = vec![0_u8; 1024 * 1024];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            writer.write_all(&buffer[..read])?;
            on_bytes_transferred(read as u64);
        }
        writer.flush()?;
        Ok(())
    }))
}

fn validate_local_infile_request(
    allowed_path: &Path,
    requested_name: &[u8],
) -> io::Result<PathBuf> {
    let requested_name = std::str::from_utf8(requested_name).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("MySQL requested a non-UTF-8 LOCAL INFILE path: {err}"),
        )
    })?;
    let requested_path = std::fs::canonicalize(requested_name)?;
    if requested_path != allowed_path {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "MySQL requested an unauthorized LOCAL INFILE path: {}",
                requested_path.display()
            ),
        ));
    }
    Ok(requested_path)
}

#[cfg(test)]
mod tests {
    use super::{
        local_infile_handler_for_path_with_progress, validate_local_infile_request,
        AGGREGATION_LOCK_NAME,
    };
    use std::io::ErrorKind;
    use std::path::Path;

    #[test]
    fn local_infile_request_accepts_only_the_selected_file() {
        let selected =
            std::fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
                .unwrap();
        let requested = selected.to_string_lossy();

        assert_eq!(
            validate_local_infile_request(&selected, requested.as_bytes()).unwrap(),
            selected
        );
    }

    #[test]
    fn local_infile_request_rejects_another_existing_file() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let selected = std::fs::canonicalize(manifest_dir.join("Cargo.toml")).unwrap();
        let requested = std::fs::canonicalize(manifest_dir.join("tauri.conf.json")).unwrap();
        let requested_text = requested.to_string_lossy();

        let error =
            validate_local_infile_request(&selected, requested_text.as_bytes()).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
    }

    #[test]
    fn local_infile_handler_rejects_a_missing_selected_file() {
        let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("missing-local-infile.csv");
        assert!(
            local_infile_handler_for_path_with_progress(&missing.to_string_lossy(), |_| {})
                .is_err()
        );
    }
    #[test]
    fn aggregation_lock_name_fits_mysql_limit() {
        assert!(!AGGREGATION_LOCK_NAME.is_empty());
        assert!(AGGREGATION_LOCK_NAME.len() <= 64);
    }
}

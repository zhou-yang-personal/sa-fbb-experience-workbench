use crate::header_normalizer::normalize_header;
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use uuid::Uuid;

static WORKSPACE_WRITER: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
pub struct DuckDbWorkspaceSettings {
    pub workspace_dir: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckDbWorkspaceStatus {
    pub workspace_dir: String,
    pub database_path: String,
    pub initialized: bool,
    pub duckdb_version: String,
    pub batch_count: u64,
    pub run_count: u64,
    pub running_run_count: u64,
    pub latest_run_id: Option<String>,
    pub latest_run_status: Option<String>,
    pub latest_run_step: Option<String>,
    pub latest_run_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DuckDbPocRequest {
    pub workspace_dir: String,
    pub file_path: String,
    pub data_type: String,
    pub batch_display_name: Option<String>,
    pub default_access_type: Option<String>,
    pub ftth_ranges: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckDbPocMetric {
    pub label: String,
    pub value: String,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckDbPocResult {
    pub import_batch_id: String,
    pub analysis_run_id: String,
    pub source_rows: u64,
    pub clean_rows: u64,
    pub hourly_rows: u64,
    pub elapsed_ms: u64,
    pub database_path: String,
    pub parquet_path: String,
    pub metrics: Vec<DuckDbPocMetric>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckDbBatchListItem {
    pub import_batch_id: String,
    pub batch_display_name: Option<String>,
    pub data_type: String,
    pub source_file_name: String,
    pub source_rows: u64,
    pub clean_rows: u64,
    pub status: String,
    pub message: Option<String>,
    pub created_at: String,
    pub latest_analysis_run_id: Option<String>,
    pub latest_run_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckDbAnalysisRunItem {
    pub analysis_run_id: String,
    pub import_batch_id: String,
    pub run_type: String,
    pub implementation_version: String,
    pub status: String,
    pub current_step: Option<String>,
    pub message: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckDbAccessSummaryRow {
    pub access_type: String,
    pub active_users: u64,
    pub observation_rows: u64,
    pub downloaded_gb: Option<f64>,
    pub avg_effective_download_mbps: Option<f64>,
    pub avg_rtt_ms: Option<f64>,
    pub avg_user_loss_pct: Option<f64>,
    pub avg_network_loss_pct: Option<f64>,
    pub avg_vmos: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckDbAccessHourlyRow {
    pub stat_date: String,
    pub hour_of_day: u8,
    #[serde(flatten)]
    pub summary: DuckDbAccessSummaryRow,
}

#[derive(Debug, Serialize)]
struct SourceManifest {
    import_batch_id: String,
    analysis_run_id: String,
    source_path: String,
    source_file_name: String,
    source_size_bytes: u64,
    source_sha256: String,
    implementation_version: String,
    data_type: String,
    created_at_utc: String,
}

#[derive(Debug, Clone)]
struct SourceColumns {
    user_account: String,
    local_ip: Option<String>,
    app: String,
    timestamp: String,
    downloaded_kb: Option<String>,
    effective_duration_s: Option<String>,
    effective_rate_kbps: Option<String>,
    downstream_rtt_ms: Option<String>,
    subscriber_rtt_ms: Option<String>,
    network_rtt_ms: Option<String>,
    user_loss_pct: Option<String>,
    network_loss_pct: Option<String>,
    vmos: Option<String>,
}

struct WorkspacePaths {
    root: PathBuf,
    database: PathBuf,
    datasets: PathBuf,
    exports: PathBuf,
    temp: PathBuf,
}

impl WorkspacePaths {
    fn from_settings(settings: &DuckDbWorkspaceSettings) -> Result<Self, String> {
        let raw = settings.workspace_dir.trim();
        if raw.is_empty() {
            return Err("工作区目录不能为空".to_string());
        }
        let root = PathBuf::from(raw);
        Ok(Self {
            database: root.join("workspace.duckdb"),
            datasets: root.join("datasets"),
            exports: root.join("exports"),
            temp: root.join("temp"),
            root,
        })
    }

    fn ensure(&self) -> Result<(), String> {
        for path in [&self.root, &self.datasets, &self.exports, &self.temp] {
            fs::create_dir_all(path)
                .map_err(|err| format!("无法创建工作区目录 {}: {err}", path.display()))?;
        }
        Ok(())
    }
}

fn open_workspace(
    settings: &DuckDbWorkspaceSettings,
) -> Result<(WorkspacePaths, Connection), String> {
    let paths = WorkspacePaths::from_settings(settings)?;
    paths.ensure()?;
    let connection = Connection::open(&paths.database)
        .map_err(|err| format!("无法打开 DuckDB 工作区 {}: {err}", paths.database.display()))?;
    let threads = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(4)
        .clamp(1, 8);
    connection
        .execute_batch(&format!(
            "PRAGMA threads={threads}; PRAGMA enable_progress_bar=false; SET preserve_insertion_order=false; SET temp_directory={}",
            sql_literal(&duckdb_path(&paths.temp))
        ))
        .map_err(|err| format!("无法初始化 DuckDB 会话: {err}"))?;
    ensure_schema(&connection)?;
    Ok((paths, connection))
}

fn ensure_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS meta_workspace (
              workspace_id VARCHAR PRIMARY KEY,
              schema_version VARCHAR NOT NULL,
              created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
              updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
            );
            INSERT INTO meta_workspace
            SELECT 'LOCAL', '2.0.0-2', current_timestamp, current_timestamp
            WHERE NOT EXISTS (SELECT 1 FROM meta_workspace WHERE workspace_id = 'LOCAL');
            UPDATE meta_workspace SET schema_version = '2.0.0-2', updated_at = current_timestamp
            WHERE workspace_id = 'LOCAL';

            CREATE TABLE IF NOT EXISTS meta_import_batch (
              import_batch_id VARCHAR PRIMARY KEY,
              batch_display_name VARCHAR,
              data_type VARCHAR NOT NULL,
              default_access_type VARCHAR NOT NULL,
              source_path VARCHAR NOT NULL,
              source_file_name VARCHAR NOT NULL,
              source_size_bytes UBIGINT NOT NULL,
              source_sha256 VARCHAR NOT NULL,
              source_rows UBIGINT NOT NULL DEFAULT 0,
              clean_rows UBIGINT NOT NULL DEFAULT 0,
              status VARCHAR NOT NULL,
              message VARCHAR,
              created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
              finished_at TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS meta_analysis_run (
              analysis_run_id VARCHAR PRIMARY KEY,
              import_batch_id VARCHAR NOT NULL,
              run_type VARCHAR NOT NULL,
              implementation_version VARCHAR NOT NULL,
              source_version VARCHAR NOT NULL,
              status VARCHAR NOT NULL,
              current_step VARCHAR,
              message VARCHAR,
              started_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
              finished_at TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS meta_pipeline_step (
              analysis_run_id VARCHAR NOT NULL,
              step_name VARCHAR NOT NULL,
              step_index INTEGER NOT NULL,
              status VARCHAR NOT NULL,
              affected_rows UBIGINT,
              message VARCHAR,
              started_at TIMESTAMP,
              finished_at TIMESTAMP,
              PRIMARY KEY (analysis_run_id, step_name)
            );

            CREATE TABLE IF NOT EXISTS meta_access_rule (
              analysis_run_id VARCHAR NOT NULL,
              rule_order INTEGER NOT NULL,
              rule_text VARCHAR NOT NULL,
              start_ip_num UBIGINT NOT NULL,
              end_ip_num UBIGINT NOT NULL,
              access_type VARCHAR NOT NULL,
              PRIMARY KEY (analysis_run_id, rule_order)
            );

            CREATE TABLE IF NOT EXISTS dws_access_hourly_poc (
              analysis_run_id VARCHAR NOT NULL,
              import_batch_id VARCHAR NOT NULL,
              stat_date DATE NOT NULL,
              hour_of_day TINYINT NOT NULL,
              access_type VARCHAR NOT NULL,
              active_users UBIGINT NOT NULL,
              observation_rows UBIGINT NOT NULL,
              downloaded_gb DOUBLE,
              avg_effective_download_mbps DOUBLE,
              avg_rtt_ms DOUBLE,
              avg_user_loss_pct DOUBLE,
              avg_network_loss_pct DOUBLE,
              avg_vmos DOUBLE,
              PRIMARY KEY (analysis_run_id, stat_date, hour_of_day, access_type)
            );

            CREATE TABLE IF NOT EXISTS ads_access_summary_poc (
              analysis_run_id VARCHAR NOT NULL,
              import_batch_id VARCHAR NOT NULL,
              access_type VARCHAR NOT NULL,
              active_users UBIGINT NOT NULL,
              observation_rows UBIGINT NOT NULL,
              downloaded_gb DOUBLE,
              avg_effective_download_mbps DOUBLE,
              avg_rtt_ms DOUBLE,
              avg_user_loss_pct DOUBLE,
              avg_network_loss_pct DOUBLE,
              avg_vmos DOUBLE,
              PRIMARY KEY (analysis_run_id, access_type)
            );
            "#,
        )
        .map_err(|err| format!("无法初始化 DuckDB schema: {err}"))
}

#[tauri::command]
pub fn duckdb_workspace_initialize(
    settings: DuckDbWorkspaceSettings,
) -> Result<DuckDbWorkspaceStatus, String> {
    let _guard = WORKSPACE_WRITER
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "DuckDB 工作区写锁已损坏".to_string())?;
    let _ = open_workspace(&settings)?;
    workspace_status_sync(&settings)
}

#[tauri::command]
pub fn duckdb_workspace_status(
    settings: DuckDbWorkspaceSettings,
) -> Result<DuckDbWorkspaceStatus, String> {
    workspace_status_sync(&settings)
}

fn open_existing_workspace(
    settings: &DuckDbWorkspaceSettings,
) -> Result<Option<(WorkspacePaths, Connection)>, String> {
    let paths = WorkspacePaths::from_settings(settings)?;
    if !paths.database.exists() {
        return Ok(None);
    }
    let connection = Connection::open(&paths.database)
        .map_err(|err| format!("无法读取 DuckDB 工作区 {}: {err}", paths.database.display()))?;
    Ok(Some((paths, connection)))
}

#[tauri::command]
pub fn duckdb_list_batches(
    settings: DuckDbWorkspaceSettings,
) -> Result<Vec<DuckDbBatchListItem>, String> {
    let Some((_, connection)) = open_existing_workspace(&settings)? else {
        return Ok(Vec::new());
    };
    let mut statement = connection.prepare(
        r#"SELECT b.import_batch_id, b.batch_display_name, b.data_type, b.source_file_name,
                  CAST(b.source_rows AS BIGINT), CAST(b.clean_rows AS BIGINT), b.status, b.message,
                  CAST(b.created_at AS VARCHAR),
                  (SELECT analysis_run_id FROM meta_analysis_run r WHERE r.import_batch_id=b.import_batch_id ORDER BY started_at DESC LIMIT 1),
                  (SELECT status FROM meta_analysis_run r WHERE r.import_batch_id=b.import_batch_id ORDER BY started_at DESC LIMIT 1)
           FROM meta_import_batch b ORDER BY b.created_at DESC"#,
    ).map_err(|err| format!("无法准备 DuckDB 批次查询: {err}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(DuckDbBatchListItem {
                import_batch_id: row.get(0)?,
                batch_display_name: row.get(1)?,
                data_type: row.get(2)?,
                source_file_name: row.get(3)?,
                source_rows: row.get::<_, i64>(4)? as u64,
                clean_rows: row.get::<_, i64>(5)? as u64,
                status: row.get(6)?,
                message: row.get(7)?,
                created_at: row.get(8)?,
                latest_analysis_run_id: row.get(9)?,
                latest_run_status: row.get(10)?,
            })
        })
        .map_err(|err| format!("无法读取 DuckDB 批次: {err}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("无法解码 DuckDB 批次: {err}"))
}

#[tauri::command]
pub fn duckdb_list_analysis_runs(
    settings: DuckDbWorkspaceSettings,
    import_batch_id: String,
) -> Result<Vec<DuckDbAnalysisRunItem>, String> {
    let Some((_, connection)) = open_existing_workspace(&settings)? else {
        return Ok(Vec::new());
    };
    let mut statement = connection.prepare(
        "SELECT analysis_run_id, import_batch_id, run_type, implementation_version, status, current_step, message, CAST(started_at AS VARCHAR), CAST(finished_at AS VARCHAR) FROM meta_analysis_run WHERE import_batch_id=? ORDER BY started_at DESC",
    ).map_err(|err| format!("无法准备 DuckDB 运行查询: {err}"))?;
    let rows = statement
        .query_map(params![import_batch_id], |row| {
            Ok(DuckDbAnalysisRunItem {
                analysis_run_id: row.get(0)?,
                import_batch_id: row.get(1)?,
                run_type: row.get(2)?,
                implementation_version: row.get(3)?,
                status: row.get(4)?,
                current_step: row.get(5)?,
                message: row.get(6)?,
                started_at: row.get(7)?,
                finished_at: row.get(8)?,
            })
        })
        .map_err(|err| format!("无法读取 DuckDB 运行: {err}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("无法解码 DuckDB 运行: {err}"))
}

#[tauri::command]
pub fn duckdb_get_access_summary(
    settings: DuckDbWorkspaceSettings,
    import_batch_id: String,
    analysis_run_id: String,
) -> Result<Vec<DuckDbAccessSummaryRow>, String> {
    let Some((_, connection)) = open_existing_workspace(&settings)? else {
        return Ok(Vec::new());
    };
    ensure_published_run(&connection, &import_batch_id, &analysis_run_id)?;
    query_access_rows(&connection, &import_batch_id, &analysis_run_id)
}

fn ensure_published_run(
    connection: &Connection,
    import_batch_id: &str,
    analysis_run_id: &str,
) -> Result<(), String> {
    let status = connection
        .query_row(
            "SELECT status FROM meta_analysis_run WHERE import_batch_id=? AND analysis_run_id=?",
            params![import_batch_id, analysis_run_id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "DuckDB 分析运行不存在或不属于当前批次".to_string())?;
    if status != "success" {
        return Err(format!("DuckDB 分析运行尚未完整发布：status={status}"));
    }
    Ok(())
}

fn query_access_rows(
    connection: &Connection,
    batch_id: &str,
    run_id: &str,
) -> Result<Vec<DuckDbAccessSummaryRow>, String> {
    let mut statement = connection.prepare(
        "SELECT access_type, CAST(active_users AS BIGINT), CAST(observation_rows AS BIGINT), downloaded_gb, avg_effective_download_mbps, avg_rtt_ms, avg_user_loss_pct, avg_network_loss_pct, avg_vmos FROM ads_access_summary_poc WHERE import_batch_id=? AND analysis_run_id=? ORDER BY access_type",
    ).map_err(|err| format!("无法准备 DuckDB Access 摘要查询: {err}"))?;
    let rows = statement
        .query_map(params![batch_id, run_id], |row| {
            Ok(DuckDbAccessSummaryRow {
                access_type: row.get(0)?,
                active_users: row.get::<_, i64>(1)? as u64,
                observation_rows: row.get::<_, i64>(2)? as u64,
                downloaded_gb: row.get(3)?,
                avg_effective_download_mbps: row.get(4)?,
                avg_rtt_ms: row.get(5)?,
                avg_user_loss_pct: row.get(6)?,
                avg_network_loss_pct: row.get(7)?,
                avg_vmos: row.get(8)?,
            })
        })
        .map_err(|err| format!("无法读取 DuckDB Access 摘要: {err}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("无法解码 DuckDB Access 摘要: {err}"))
}

#[tauri::command]
pub fn duckdb_get_access_hourly(
    settings: DuckDbWorkspaceSettings,
    import_batch_id: String,
    analysis_run_id: String,
) -> Result<Vec<DuckDbAccessHourlyRow>, String> {
    let Some((_, connection)) = open_existing_workspace(&settings)? else {
        return Ok(Vec::new());
    };
    ensure_published_run(&connection, &import_batch_id, &analysis_run_id)?;
    let mut statement = connection.prepare(
        "SELECT CAST(stat_date AS VARCHAR), CAST(hour_of_day AS INTEGER), access_type, CAST(active_users AS BIGINT), CAST(observation_rows AS BIGINT), downloaded_gb, avg_effective_download_mbps, avg_rtt_ms, avg_user_loss_pct, avg_network_loss_pct, avg_vmos FROM dws_access_hourly_poc WHERE import_batch_id=? AND analysis_run_id=? ORDER BY stat_date, hour_of_day, access_type",
    ).map_err(|err| format!("无法准备 DuckDB Access 小时查询: {err}"))?;
    let rows = statement
        .query_map(params![import_batch_id, analysis_run_id], |row| {
            Ok(DuckDbAccessHourlyRow {
                stat_date: row.get(0)?,
                hour_of_day: row.get::<_, i32>(1)? as u8,
                summary: DuckDbAccessSummaryRow {
                    access_type: row.get(2)?,
                    active_users: row.get::<_, i64>(3)? as u64,
                    observation_rows: row.get::<_, i64>(4)? as u64,
                    downloaded_gb: row.get(5)?,
                    avg_effective_download_mbps: row.get(6)?,
                    avg_rtt_ms: row.get(7)?,
                    avg_user_loss_pct: row.get(8)?,
                    avg_network_loss_pct: row.get(9)?,
                    avg_vmos: row.get(10)?,
                },
            })
        })
        .map_err(|err| format!("无法读取 DuckDB Access 小时结果: {err}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("无法解码 DuckDB Access 小时结果: {err}"))
}

fn workspace_status_sync(
    settings: &DuckDbWorkspaceSettings,
) -> Result<DuckDbWorkspaceStatus, String> {
    let paths = WorkspacePaths::from_settings(settings)?;
    if !paths.database.exists() {
        return Ok(DuckDbWorkspaceStatus {
            workspace_dir: paths.root.display().to_string(),
            database_path: paths.database.display().to_string(),
            initialized: false,
            duckdb_version: String::new(),
            batch_count: 0,
            run_count: 0,
            running_run_count: 0,
            latest_run_id: None,
            latest_run_status: None,
            latest_run_step: None,
            latest_run_message: None,
        });
    }
    let connection = Connection::open(&paths.database)
        .map_err(|err| format!("无法读取 DuckDB 工作区 {}: {err}", paths.database.display()))?;
    let duckdb_version: String = connection
        .query_row("SELECT version()", [], |row| row.get(0))
        .map_err(|err| format!("无法读取 DuckDB 版本: {err}"))?;
    let batch_count = scalar_u64(&connection, "SELECT count(*) FROM meta_import_batch")?;
    let run_count = scalar_u64(&connection, "SELECT count(*) FROM meta_analysis_run")?;
    let running_run_count = scalar_u64(
        &connection,
        "SELECT count(*) FROM meta_analysis_run WHERE status = 'running'",
    )?;
    let (latest_run_id, latest_run_status, latest_run_step, latest_run_message): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = connection
        .query_row(
            "SELECT arg_max(analysis_run_id, started_at), arg_max(status, started_at), arg_max(current_step, started_at), arg_max(message, started_at) FROM meta_analysis_run",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|err| format!("无法读取最新 DuckDB 运行: {err}"))?;
    Ok(DuckDbWorkspaceStatus {
        workspace_dir: paths.root.display().to_string(),
        database_path: paths.database.display().to_string(),
        initialized: true,
        duckdb_version,
        batch_count,
        run_count,
        running_run_count,
        latest_run_id,
        latest_run_status,
        latest_run_step,
        latest_run_message,
    })
}

#[tauri::command]
pub async fn duckdb_poc_analyze_csv(req: DuckDbPocRequest) -> Result<DuckDbPocResult, String> {
    tauri::async_runtime::spawn_blocking(move || duckdb_poc_analyze_csv_blocking(req))
        .await
        .map_err(|err| format!("DuckDB 后台任务无法完成: {err}"))?
}

/// Synchronous entry point used by the desktop worker and the reproducible
/// command-line benchmark harness under `examples/`.
pub fn duckdb_poc_analyze_csv_blocking(req: DuckDbPocRequest) -> Result<DuckDbPocResult, String> {
    let _guard = WORKSPACE_WRITER
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "DuckDB 工作区写锁已损坏".to_string())?;
    let import_batch_id = format!("BATCH_DUCK_{}", Uuid::new_v4().simple());
    let analysis_run_id = format!("RUN_DUCK_{}", Uuid::new_v4().simple());
    let settings = DuckDbWorkspaceSettings {
        workspace_dir: req.workspace_dir.clone(),
    };
    let (paths, connection) = open_workspace(&settings)?;

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        prepare_run(&connection, &req, &import_batch_id, &analysis_run_id)?;
        analyze_csv_sync(
            &connection,
            &paths,
            &req,
            &import_batch_id,
            &analysis_run_id,
        )
    }));
    match outcome {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => {
            close_failed_run(&connection, &import_batch_id, &analysis_run_id, &err);
            Err(err)
        }
        Err(payload) => {
            let reason = panic_message(payload);
            let message = format!("DuckDB 分析线程异常: {reason}");
            close_failed_run(&connection, &import_batch_id, &analysis_run_id, &message);
            Err(message)
        }
    }
}

fn prepare_run(
    connection: &Connection,
    req: &DuckDbPocRequest,
    import_batch_id: &str,
    analysis_run_id: &str,
) -> Result<(), String> {
    if req.data_type.trim().to_ascii_lowercase() != "tcp" {
        return Err("当前 DuckDB POC 先支持 TCP/视频 CSV；Game 将在下一纵切迁移".to_string());
    }
    let source_path = PathBuf::from(req.file_path.trim());
    if !source_path.is_file() {
        return Err(format!("CSV 文件不存在: {}", source_path.display()));
    }
    let metadata =
        fs::metadata(&source_path).map_err(|err| format!("无法读取 CSV 文件信息: {err}"))?;
    let file_name = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("source.csv");
    connection
        .execute(
            "INSERT INTO meta_import_batch (import_batch_id, batch_display_name, data_type, default_access_type, source_path, source_file_name, source_size_bytes, source_sha256, status, message) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'running', 'Preparing DuckDB analysis')",
            params![
                import_batch_id,
                req.batch_display_name.as_deref(),
                req.data_type.trim().to_ascii_lowercase(),
                normalized_access_type(req.default_access_type.as_deref().unwrap_or("CABLE")),
                source_path.display().to_string(),
                file_name,
                metadata.len(),
                format!("pending:size={}", metadata.len()),
            ],
        )
        .map_err(|err| format!("无法登记 DuckDB 批次: {err}"))?;
    connection
        .execute(
            "INSERT INTO meta_analysis_run (analysis_run_id, import_batch_id, run_type, implementation_version, source_version, status, current_step, message) VALUES (?, ?, 'duckdb_poc', ?, ?, 'running', 'fingerprint', 'DuckDB analysis started')",
            params![
                analysis_run_id,
                import_batch_id,
                env!("CARGO_PKG_VERSION"),
                format!("pending:size={}", metadata.len()),
            ],
        )
        .map_err(|err| format!("无法登记 DuckDB 分析运行: {err}"))?;
    for (index, step) in [
        "fingerprint",
        "csv_to_parquet",
        "hourly_aggregate",
        "publish",
    ]
    .iter()
    .enumerate()
    {
        connection
            .execute(
                "INSERT INTO meta_pipeline_step (analysis_run_id, step_name, step_index, status) VALUES (?, ?, ?, 'pending')",
                params![analysis_run_id, *step, index as i32 + 1],
            )
            .map_err(|err| format!("无法登记 DuckDB 步骤 {step}: {err}"))?;
    }
    Ok(())
}

fn analyze_csv_sync(
    connection: &Connection,
    paths: &WorkspacePaths,
    req: &DuckDbPocRequest,
    import_batch_id: &str,
    analysis_run_id: &str,
) -> Result<DuckDbPocResult, String> {
    let started = Instant::now();
    let source_path = PathBuf::from(req.file_path.trim());
    let metadata = fs::metadata(&source_path).map_err(|err| err.to_string())?;

    start_step(connection, analysis_run_id, "fingerprint")?;
    let (source_sha256, source_rows) = sha256_and_record_count(&source_path)?;
    if source_rows == 0 {
        return Err("CSV 只有表头或为空，拒绝生成空 Parquet 发布结果".to_string());
    }
    finish_step(
        connection,
        analysis_run_id,
        "fingerprint",
        None,
        "SHA-256 ready",
    )?;
    connection
        .execute(
            "UPDATE meta_import_batch SET source_sha256 = ? WHERE import_batch_id = ?",
            params![source_sha256.as_str(), import_batch_id],
        )
        .map_err(|err| format!("无法保存源文件指纹: {err}"))?;
    connection
        .execute(
            "UPDATE meta_analysis_run SET source_version = ? WHERE analysis_run_id = ?",
            params![source_sha256.as_str(), analysis_run_id],
        )
        .map_err(|err| format!("无法保存运行 source_version: {err}"))?;

    let delimiter = detect_delimiter(&source_path)?;
    let columns = inspect_columns(&source_path, delimiter)?;
    let batch_root = paths.datasets.join(import_batch_id);
    let parquet_root = batch_root.join("dwd").join("data_type=tcp");
    fs::create_dir_all(&parquet_root).map_err(|err| format!("无法创建 Parquet 目录: {err}"))?;
    let manifest_path = batch_root.join("source-manifest.json");
    let manifest = SourceManifest {
        import_batch_id: import_batch_id.to_string(),
        analysis_run_id: analysis_run_id.to_string(),
        source_path: source_path.display().to_string(),
        source_file_name: source_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("source.csv")
            .to_string(),
        source_size_bytes: metadata.len(),
        source_sha256: source_sha256.clone(),
        implementation_version: env!("CARGO_PKG_VERSION").to_string(),
        data_type: "tcp".to_string(),
        created_at_utc: chrono::Utc::now().to_rfc3339(),
    };
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("无法写入源文件清单: {err}"))?;

    replace_access_rules(
        connection,
        analysis_run_id,
        req.ftth_ranges.as_deref().unwrap_or(&[]),
    )?;
    start_step(connection, analysis_run_id, "csv_to_parquet")?;
    let csv_sql = build_clean_select(
        &source_path,
        delimiter,
        &columns,
        import_batch_id,
        analysis_run_id,
        manifest.source_file_name.as_str(),
        source_sha256.as_str(),
        req.default_access_type.as_deref().unwrap_or("CABLE"),
    );
    let copy_sql = format!(
        "COPY ({csv_sql}) TO {} (FORMAT PARQUET, COMPRESSION ZSTD, PARTITION_BY (stat_date), OVERWRITE_OR_IGNORE true)",
        sql_literal(&duckdb_path(&parquet_root))
    );
    let clean_rows = connection
        .query_row(&copy_sql, [], |row| row.get::<_, i64>(0))
        .map(|value| value.max(0) as u64)
        .map_err(|err| format!("CSV 转 Parquet 失败: {err}"))?;
    if clean_rows == 0 {
        return Err("CSV 未生成任何满足用户、应用和时间契约的有效记录，拒绝发布空结果".to_string());
    }
    let metadata_after = fs::metadata(&source_path)
        .map_err(|err| format!("Parquet 写入后无法复核源文件信息: {err}"))?;
    let modified_before = metadata.modified().ok();
    let modified_after = metadata_after.modified().ok();
    if metadata.len() != metadata_after.len()
        || (modified_before.is_some()
            && modified_after.is_some()
            && modified_before != modified_after)
    {
        return Err(
            "分析期间源 CSV 的大小或修改时间发生变化；本次结果已拒绝发布，请在文件稳定后重试"
                .to_string(),
        );
    }
    let parquet_glob = format!("{}/**/*.parquet", duckdb_path(&parquet_root));
    finish_step(
        connection,
        analysis_run_id,
        "csv_to_parquet",
        Some(clean_rows),
        "Partitioned Parquet ready",
    )?;

    start_step(connection, analysis_run_id, "hourly_aggregate")?;
    connection
        .execute(
            "DELETE FROM dws_access_hourly_poc WHERE analysis_run_id = ?",
            params![analysis_run_id],
        )
        .map_err(|err| format!("无法清理小时 POC 结果: {err}"))?;
    let hourly_sql = format!(
        r#"
        INSERT INTO dws_access_hourly_poc
        SELECT
          {run_id}, {batch_id}, stat_date, hour_of_day, access_type,
          count(DISTINCT user_key), count(*),
          sum(downloaded_kb) / 1024.0 / 1024.0,
          sum(effective_rate_kbps) FILTER (WHERE effective_rate_kbps IS NOT NULL) / nullif(count(effective_rate_kbps), 0) / 1000.0,
          sum(coalesce(subscriber_rtt_ms, downstream_rtt_ms)) FILTER (WHERE coalesce(subscriber_rtt_ms, downstream_rtt_ms) IS NOT NULL) / nullif(count(coalesce(subscriber_rtt_ms, downstream_rtt_ms)), 0),
          sum(user_loss_pct) FILTER (WHERE user_loss_pct IS NOT NULL) / nullif(count(user_loss_pct), 0),
          sum(network_loss_pct) FILTER (WHERE network_loss_pct IS NOT NULL) / nullif(count(network_loss_pct), 0),
          sum(vmos) FILTER (WHERE vmos IS NOT NULL) / nullif(count(vmos), 0)
        FROM read_parquet({parquet}, hive_partitioning=true)
        GROUP BY stat_date, hour_of_day, access_type
        "#,
        run_id = sql_literal(analysis_run_id),
        batch_id = sql_literal(import_batch_id),
        parquet = sql_literal(&parquet_glob),
    );
    connection
        .execute_batch(&hourly_sql)
        .map_err(|err| format!("DuckDB 小时聚合失败: {err}"))?;
    let hourly_rows = scalar_u64(
        connection,
        &format!(
            "SELECT count(*) FROM dws_access_hourly_poc WHERE analysis_run_id = {}",
            sql_literal(analysis_run_id)
        ),
    )?;
    finish_step(
        connection,
        analysis_run_id,
        "hourly_aggregate",
        Some(hourly_rows),
        "Hourly aggregation ready",
    )?;

    start_step(connection, analysis_run_id, "publish")?;
    connection
        .execute_batch("BEGIN TRANSACTION")
        .map_err(|err| format!("无法开始 Access POC 发布事务: {err}"))?;
    let publish_result = (|| -> Result<(), String> {
        connection
            .execute(
                "DELETE FROM ads_access_summary_poc WHERE analysis_run_id = ?",
                params![analysis_run_id],
            )
            .map_err(|err| format!("无法清理 Access POC 发布结果: {err}"))?;
        connection
            .execute_batch(&format!(
            r#"
            INSERT INTO ads_access_summary_poc
            SELECT
              {run_id}, {batch_id}, access_type,
              count(DISTINCT user_key), count(*), sum(downloaded_kb) / 1024.0 / 1024.0,
              sum(effective_rate_kbps) FILTER (WHERE effective_rate_kbps IS NOT NULL) / nullif(count(effective_rate_kbps), 0) / 1000.0,
              sum(coalesce(subscriber_rtt_ms, downstream_rtt_ms)) FILTER (WHERE coalesce(subscriber_rtt_ms, downstream_rtt_ms) IS NOT NULL) / nullif(count(coalesce(subscriber_rtt_ms, downstream_rtt_ms)), 0),
              sum(user_loss_pct) FILTER (WHERE user_loss_pct IS NOT NULL) / nullif(count(user_loss_pct), 0),
              sum(network_loss_pct) FILTER (WHERE network_loss_pct IS NOT NULL) / nullif(count(network_loss_pct), 0),
              sum(vmos) FILTER (WHERE vmos IS NOT NULL) / nullif(count(vmos), 0)
            FROM read_parquet({parquet}, hive_partitioning=true)
            GROUP BY access_type
            "#,
            run_id = sql_literal(analysis_run_id),
            batch_id = sql_literal(import_batch_id),
            parquet = sql_literal(&parquet_glob),
        ))
            .map_err(|err| format!("Access POC 原子发布失败: {err}"))?;
        connection
            .execute_batch("COMMIT")
            .map_err(|err| format!("无法提交 Access POC 发布事务: {err}"))?;
        Ok(())
    })();
    if let Err(err) = publish_result {
        let _ = connection.execute_batch("ROLLBACK");
        return Err(err);
    }
    finish_step(
        connection,
        analysis_run_id,
        "publish",
        None,
        "POC published",
    )?;
    connection
        .execute(
            "UPDATE meta_import_batch SET source_rows = ?, clean_rows = ?, status = 'success', message = 'DuckDB POC ready', finished_at = current_timestamp WHERE import_batch_id = ?",
            params![source_rows, clean_rows, import_batch_id],
        )
        .map_err(|err| format!("无法完成 DuckDB 批次: {err}"))?;
    connection
        .execute(
            "UPDATE meta_analysis_run SET status = 'success', current_step = 'finish', message = 'DuckDB + Parquet POC ready', finished_at = current_timestamp WHERE analysis_run_id = ?",
            params![analysis_run_id],
        )
        .map_err(|err| format!("无法完成 DuckDB 运行: {err}"))?;

    let summary_count = scalar_u64(
        connection,
        &format!(
            "SELECT count(*) FROM ads_access_summary_poc WHERE analysis_run_id = {}",
            sql_literal(analysis_run_id)
        ),
    )?;
    let mut metrics = vec![
        metric(
            "源 CSV 行数",
            source_rows.to_string(),
            "SHA-256 顺序读取时统计物理记录（不含表头）",
        ),
        metric("有效明细行数", clean_rows.to_string(), "Parquet DWD"),
        metric(
            "小时聚合行数",
            hourly_rows.to_string(),
            "日期 × 小时 × 接入类型",
        ),
        metric(
            "接入类型汇总",
            summary_count.to_string(),
            "Cable / FTTH / Other",
        ),
        metric(
            "耗时",
            format!("{} ms", started.elapsed().as_millis()),
            "含 SHA-256、Parquet 与聚合",
        ),
    ];
    let mut statement = connection
        .prepare(
            "SELECT access_type, CAST(active_users AS BIGINT), downloaded_gb, avg_effective_download_mbps, avg_rtt_ms, avg_user_loss_pct, avg_network_loss_pct, avg_vmos FROM ads_access_summary_poc WHERE analysis_run_id = ? ORDER BY access_type",
        )
        .map_err(|err| format!("无法读取 Access POC 摘要: {err}"))?;
    let summaries = statement
        .query_map(params![analysis_run_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<f64>>(2)?,
                row.get::<_, Option<f64>>(3)?,
                row.get::<_, Option<f64>>(4)?,
                row.get::<_, Option<f64>>(5)?,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<f64>>(7)?,
            ))
        })
        .map_err(|err| format!("无法遍历 Access POC 摘要: {err}"))?;
    for summary in summaries {
        let (access, users, traffic, rate, rtt, user_loss, network_loss, vmos) =
            summary.map_err(|err| format!("无法解码 Access POC 摘要: {err}"))?;
        metrics.push(metric(
            &format!("{access} 活跃用户"),
            users.to_string(),
            &format!(
                "流量={} GB · 有效速率={} Mbps · RTT={} ms · 用户丢包={}% · 网络丢包={}% · vMOS={}",
                fmt_optional(traffic, 3),
                fmt_optional(rate, 3),
                fmt_optional(rtt, 2),
                fmt_optional(user_loss, 3),
                fmt_optional(network_loss, 3),
                fmt_optional(vmos, 2),
            ),
        ));
    }
    Ok(DuckDbPocResult {
        import_batch_id: import_batch_id.to_string(),
        analysis_run_id: analysis_run_id.to_string(),
        source_rows,
        clean_rows,
        hourly_rows,
        elapsed_ms: started.elapsed().as_millis() as u64,
        database_path: paths.database.display().to_string(),
        parquet_path: parquet_root.display().to_string(),
        metrics,
    })
}

fn build_clean_select(
    source_path: &Path,
    delimiter: u8,
    columns: &SourceColumns,
    import_batch_id: &str,
    analysis_run_id: &str,
    source_file_name: &str,
    source_version: &str,
    default_access_type: &str,
) -> String {
    let user = string_expr(&columns.user_account);
    let local_ip = columns
        .local_ip
        .as_ref()
        .map(|value| string_expr(value))
        .unwrap_or_else(|| "NULL".to_string());
    let app = string_expr(&columns.app);
    let timestamp = timestamp_expr(&columns.timestamp);
    let downloaded = numeric_expr(columns.downloaded_kb.as_deref());
    let effective_duration = numeric_expr(columns.effective_duration_s.as_deref());
    let effective_rate = columns
        .effective_rate_kbps
        .as_deref()
        .map(|value| numeric_column(value))
        .unwrap_or_else(|| format!("CASE WHEN {effective_duration} > 0 THEN {downloaded} * 8.0 / {effective_duration} ELSE NULL END"));
    let downstream_rtt = numeric_expr(columns.downstream_rtt_ms.as_deref());
    let subscriber_rtt = numeric_expr(columns.subscriber_rtt_ms.as_deref());
    let network_rtt = numeric_expr(columns.network_rtt_ms.as_deref());
    let user_loss = numeric_expr(columns.user_loss_pct.as_deref());
    let network_loss = numeric_expr(columns.network_loss_pct.as_deref());
    let vmos = numeric_expr(columns.vmos.as_deref());
    let user_ip_num = ipv4_num_expr(&user);
    let local_ip_num = ipv4_num_expr(&local_ip);
    let ip_num = format!("coalesce({user_ip_num}, {local_ip_num})");
    let analysis_ip = format!(
        "CASE WHEN {user_ip_num} IS NOT NULL THEN {user} WHEN {local_ip_num} IS NOT NULL THEN {local_ip} ELSE NULL END"
    );
    let normalized_default = normalized_access_type(default_access_type);
    format!(
        r#"
        WITH source AS (
          SELECT * FROM read_csv({source_path}, header=true, all_varchar=true, delim={delimiter}, ignore_errors=true)
        ), typed AS (
          SELECT
            {user} AS user_key,
            {local_ip} AS local_ip,
            {analysis_ip} AS analysis_ip,
            {ip_num} AS ip_num,
            {app} AS app_name,
            {timestamp} AS statistical_time,
            {downloaded} AS downloaded_kb,
            {effective_duration} AS effective_duration_s,
            {effective_rate} AS effective_rate_kbps,
            {downstream_rtt} AS downstream_rtt_ms,
            {subscriber_rtt} AS subscriber_rtt_ms,
            {network_rtt} AS network_rtt_ms,
            {user_loss} AS user_loss_pct,
            {network_loss} AS network_loss_pct,
            {vmos} AS vmos
          FROM source
        )
        SELECT
          {import_batch_id} AS import_batch_id,
          {analysis_run_id} AS analysis_run_id,
          {source_file_name} AS source_file_name,
          {source_version} AS source_version,
          {implementation_version} AS implementation_version,
          user_key, local_ip, analysis_ip, app_name, statistical_time,
          cast(statistical_time AS DATE) AS stat_date,
          cast(extract(hour FROM statistical_time) AS TINYINT) AS hour_of_day,
          coalesce((
            SELECT access_type FROM meta_access_rule rule
            WHERE rule.analysis_run_id = {analysis_run_id}
              AND typed.ip_num BETWEEN rule.start_ip_num AND rule.end_ip_num
            ORDER BY rule.rule_order LIMIT 1
          ), {default_access_type}) AS access_type,
          downloaded_kb, effective_duration_s, effective_rate_kbps,
          downstream_rtt_ms, subscriber_rtt_ms, network_rtt_ms,
          user_loss_pct, network_loss_pct, vmos
        FROM typed
        WHERE user_key IS NOT NULL AND app_name IS NOT NULL AND statistical_time IS NOT NULL
        "#,
        source_path = sql_literal(&duckdb_path(source_path)),
        delimiter = sql_literal(&(delimiter as char).to_string()),
        import_batch_id = sql_literal(import_batch_id),
        analysis_run_id = sql_literal(analysis_run_id),
        source_file_name = sql_literal(source_file_name),
        source_version = sql_literal(source_version),
        implementation_version = sql_literal(env!("CARGO_PKG_VERSION")),
        default_access_type = sql_literal(normalized_default),
    )
}

fn normalized_access_type(value: &str) -> &'static str {
    match value.trim().to_ascii_uppercase().as_str() {
        "FTTH" => "FTTH",
        "OTHER" => "OTHER",
        _ => "CABLE",
    }
}

fn inspect_columns(path: &Path, delimiter: u8) -> Result<SourceColumns, String> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_path(path)
        .map_err(|err| format!("无法读取 CSV 表头: {err}"))?;
    let headers = reader
        .headers()
        .map_err(|err| format!("无法解析 CSV 表头: {err}"))?
        .clone();
    let normalized: HashMap<String, String> = headers
        .iter()
        .map(|header| (normalize_header(header), header.to_string()))
        .collect();
    let required = |label: &str, aliases: &[&str]| {
        resolve_column(&normalized, aliases)
            .ok_or_else(|| format!("CSV 缺少{label}；可识别字段别名: {}", aliases.join(", ")))
    };
    Ok(SourceColumns {
        user_account: required(
            "用户账号",
            &[
                "subscriber_account",
                "user_account",
                "account",
                "account_id",
                "subscriber_id",
                "user_key",
            ],
        )?,
        local_ip: resolve_column(
            &normalized,
            &[
                "local_ipv4",
                "local_ipv4_address",
                "local_ip_address",
                "local_ip",
                "subscriber_ip",
                "client_ip",
                "user_ip",
            ],
        ),
        app: required(
            "应用",
            &[
                "universal_video_applications",
                "universal_video_application",
                "application_protocol",
                "application",
                "application_name",
                "app_name",
            ],
        )?,
        timestamp: required(
            "统计时间",
            &[
                "statistics_duration",
                "statistical_time",
                "statistics_time",
                "record_time",
                "stat_time",
                "start_time",
                "collect_time",
                "timestamp",
            ],
        )?,
        downloaded_kb: resolve_column(
            &normalized,
            &[
                "downloaded_data_volume_kb",
                "downloaded_data_kb",
                "download_data_kb",
                "traffic_kb",
                "downloaded_kb",
                "data_volume_kb",
            ],
        ),
        effective_duration_s: resolve_column(
            &normalized,
            &[
                "effective_download_duration_s",
                "effective_duration_s",
                "download_effective_duration_s",
                "video_download_duration_s",
                "video_duration_s",
                "download_duration_s",
            ],
        ),
        effective_rate_kbps: resolve_column(
            &normalized,
            &[
                "user_avg_effective_download_rate_kbps",
                "user_effective_download_rate_kbps",
                "users_average_effective_download_rate_kbps",
                "user_average_effective_download_rate_kbps",
                "effective_download_rate_kbps",
                "effective_download_rate",
                "single_flow_rate_kbps",
            ],
        ),
        downstream_rtt_ms: resolve_column(
            &normalized,
            &[
                "downstream_data_transmission_rtt_ms",
                "downstream_rtt_ms",
                "downlink_rtt_ms",
                "downstream_rtt",
                "user_down_rtt_ms",
            ],
        ),
        subscriber_rtt_ms: resolve_column(
            &normalized,
            &[
                "subscriber_side_rtt_ms",
                "user_side_rtt_ms",
                "subscriber_rtt_ms",
                "user_rtt_ms",
                "avg_user_rtt_ms",
            ],
        ),
        network_rtt_ms: resolve_column(
            &normalized,
            &[
                "network_side_rtt_ms",
                "network_rtt_ms",
                "avg_network_rtt_ms",
            ],
        ),
        user_loss_pct: resolve_column(
            &normalized,
            &[
                "user_side_downstream_packet_loss_rate",
                "user_side_downstream_packet_loss_rate_pct",
                "user_down_loss_pct",
                "subscriber_packet_loss_pct",
            ],
        ),
        network_loss_pct: resolve_column(
            &normalized,
            &[
                "network_side_downstream_packet_loss_rate",
                "network_side_downstream_packet_loss_rate_pct",
                "network_down_loss_pct",
                "network_packet_loss_pct",
            ],
        ),
        vmos: resolve_column(&normalized, &["vmos", "video_mos", "vmos_score"]),
    })
}

fn resolve_column(headers: &HashMap<String, String>, aliases: &[&str]) -> Option<String> {
    aliases
        .iter()
        .find_map(|alias| headers.get(&normalize_header(alias)).cloned())
}

fn replace_access_rules(
    connection: &Connection,
    analysis_run_id: &str,
    ranges: &[String],
) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM meta_access_rule WHERE analysis_run_id = ?",
            params![analysis_run_id],
        )
        .map_err(|err| format!("无法清理接入规则: {err}"))?;
    for (index, text) in ranges.iter().enumerate() {
        if text.trim().is_empty() {
            continue;
        }
        let (start, end) = parse_ipv4_range(text)?;
        connection
            .execute(
                "INSERT INTO meta_access_rule (analysis_run_id, rule_order, rule_text, start_ip_num, end_ip_num, access_type) VALUES (?, ?, ?, ?, ?, 'FTTH')",
                params![analysis_run_id, index as i32 + 1, text.trim(), start, end],
            )
            .map_err(|err| format!("无法写入 FTTH 规则 {}: {err}", text.trim()))?;
    }
    Ok(())
}

fn parse_ipv4_range(value: &str) -> Result<(u64, u64), String> {
    let value = value.trim();
    if let Some((ip, prefix)) = value.split_once('/') {
        let ip = Ipv4Addr::from_str(ip.trim()).map_err(|_| format!("无效 IPv4 CIDR: {value}"))?;
        let prefix: u32 = prefix
            .trim()
            .parse()
            .map_err(|_| format!("无效 IPv4 CIDR 前缀: {value}"))?;
        if prefix > 32 {
            return Err(format!("IPv4 CIDR 前缀必须在 0–32: {value}"));
        }
        let raw = u32::from(ip);
        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        return Ok(((raw & mask) as u64, (raw | !mask) as u64));
    }
    if let Some((start, end)) = value.split_once('-') {
        let start =
            Ipv4Addr::from_str(start.trim()).map_err(|_| format!("无效 IPv4 起始地址: {value}"))?;
        let end =
            Ipv4Addr::from_str(end.trim()).map_err(|_| format!("无效 IPv4 结束地址: {value}"))?;
        let start = u32::from(start) as u64;
        let end = u32::from(end) as u64;
        if start > end {
            return Err(format!("IPv4 范围起始地址不能大于结束地址: {value}"));
        }
        return Ok((start, end));
    }
    let ip = Ipv4Addr::from_str(value).map_err(|_| format!("无效 IPv4 地址: {value}"))?;
    let raw = u32::from(ip) as u64;
    Ok((raw, raw))
}

fn start_step(connection: &Connection, analysis_run_id: &str, step: &str) -> Result<(), String> {
    connection
        .execute(
            "UPDATE meta_pipeline_step SET status = 'running', started_at = current_timestamp, message = NULL WHERE analysis_run_id = ? AND step_name = ?",
            params![analysis_run_id, step],
        )
        .map_err(|err| format!("无法启动步骤 {step}: {err}"))?;
    connection
        .execute(
            "UPDATE meta_analysis_run SET current_step = ? WHERE analysis_run_id = ?",
            params![step, analysis_run_id],
        )
        .map_err(|err| format!("无法更新当前步骤 {step}: {err}"))?;
    Ok(())
}

fn finish_step(
    connection: &Connection,
    analysis_run_id: &str,
    step: &str,
    affected_rows: Option<u64>,
    message: &str,
) -> Result<(), String> {
    connection
        .execute(
            "UPDATE meta_pipeline_step SET status = 'success', affected_rows = ?, message = ?, finished_at = current_timestamp WHERE analysis_run_id = ? AND step_name = ?",
            params![affected_rows, message, analysis_run_id, step],
        )
        .map_err(|err| format!("无法完成步骤 {step}: {err}"))?;
    Ok(())
}

fn close_failed_run(
    connection: &Connection,
    import_batch_id: &str,
    analysis_run_id: &str,
    message: &str,
) {
    let _ = connection.execute(
        "UPDATE meta_pipeline_step SET status = 'failed', message = ?, finished_at = current_timestamp WHERE analysis_run_id = ? AND status = 'running'",
        params![message, analysis_run_id],
    );
    let _ = connection.execute(
        "UPDATE meta_analysis_run SET status = 'failed', message = ?, finished_at = current_timestamp WHERE analysis_run_id = ?",
        params![message, analysis_run_id],
    );
    let _ = connection.execute(
        "UPDATE meta_import_batch SET status = 'failed', message = ?, finished_at = current_timestamp WHERE import_batch_id = ?",
        params![message, import_batch_id],
    );
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic".to_string()
    }
}

fn detect_delimiter(path: &Path) -> Result<u8, String> {
    let mut file = File::open(path).map_err(|err| format!("无法打开 CSV: {err}"))?;
    let mut buffer = vec![0_u8; 64 * 1024];
    let read = file
        .read(&mut buffer)
        .map_err(|err| format!("无法探测 CSV 分隔符: {err}"))?;
    let text = String::from_utf8_lossy(&buffer[..read]);
    let line = text.lines().next().unwrap_or_default();
    let choices = [b',', b'\t', b';', b'|'];
    choices
        .into_iter()
        .max_by_key(|delimiter| {
            line.as_bytes()
                .iter()
                .filter(|byte| **byte == *delimiter)
                .count()
        })
        .filter(|delimiter| line.as_bytes().contains(delimiter))
        .ok_or_else(|| "无法识别 CSV 分隔符".to_string())
}

fn sha256_and_record_count(path: &Path) -> Result<(String, u64), String> {
    let file = File::open(path).map_err(|err| format!("无法打开源文件计算 SHA-256: {err}"))?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut newline_count = 0_u64;
    let mut last_byte = None;
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|err| format!("读取源文件计算 SHA-256 失败: {err}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        newline_count += buffer[..read].iter().filter(|byte| **byte == b'\n').count() as u64;
        last_byte = buffer.get(read - 1).copied();
    }
    let physical_lines = newline_count + u64::from(last_byte.is_some_and(|byte| byte != b'\n'));
    let record_count = physical_lines.saturating_sub(1);
    Ok((format!("{:x}", hasher.finalize()), record_count))
}

fn scalar_u64(connection: &Connection, sql: &str) -> Result<u64, String> {
    connection
        .query_row(sql, [], |row| row.get::<_, i64>(0))
        .map(|value| value.max(0) as u64)
        .map_err(|err| format!("DuckDB 统计失败: {err}; sql={sql}"))
}

fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn duckdb_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn string_expr(column: &str) -> String {
    let column = sql_identifier(column);
    format!("nullif(nullif(trim({column}), ''), '--')")
}

fn numeric_column(column: &str) -> String {
    format!("try_cast({} AS DOUBLE)", string_expr(column))
}

fn numeric_expr(column: Option<&str>) -> String {
    column
        .map(numeric_column)
        .unwrap_or_else(|| "NULL::DOUBLE".to_string())
}

fn timestamp_expr(column: &str) -> String {
    let value = string_expr(column);
    format!(
        "coalesce(try_cast({value} AS TIMESTAMP), try_strptime({value}, '%d/%m/%Y %H:%M:%S'), try_strptime({value}, '%Y-%m-%d %H:%M:%S'), try_strptime({value}, '%Y/%m/%d %H:%M:%S'))"
    )
}

fn ipv4_num_expr(local_ip_expr: &str) -> String {
    format!(
        r#"CASE
          WHEN regexp_full_match({ip}, '([0-9]{{1,3}}\.){{3}}[0-9]{{1,3}}')
           AND try_cast(split_part({ip}, '.', 1) AS INTEGER) BETWEEN 0 AND 255
           AND try_cast(split_part({ip}, '.', 2) AS INTEGER) BETWEEN 0 AND 255
           AND try_cast(split_part({ip}, '.', 3) AS INTEGER) BETWEEN 0 AND 255
           AND try_cast(split_part({ip}, '.', 4) AS INTEGER) BETWEEN 0 AND 255
          THEN try_cast(split_part({ip}, '.', 1) AS UBIGINT) * 16777216
             + try_cast(split_part({ip}, '.', 2) AS UBIGINT) * 65536
             + try_cast(split_part({ip}, '.', 3) AS UBIGINT) * 256
             + try_cast(split_part({ip}, '.', 4) AS UBIGINT)
          ELSE NULL END"#,
        ip = local_ip_expr
    )
}

fn metric(label: &str, value: String, hint: &str) -> DuckDbPocMetric {
    DuckDbPocMetric {
        label: label.to_string(),
        value,
        hint: hint.to_string(),
    }
}

fn fmt_optional(value: Option<f64>, precision: usize) -> String {
    value
        .map(|number| format!("{number:.precision$}"))
        .unwrap_or_else(|| "—".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_csv_sync, duckdb_get_access_summary, duckdb_list_batches,
        duckdb_poc_analyze_csv_blocking, open_workspace, parse_ipv4_range, prepare_run,
        resolve_column, DuckDbPocRequest, DuckDbWorkspaceSettings,
    };
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn resolves_normalized_aliases() {
        let headers = HashMap::from([(
            "subscriber_account".to_string(),
            "Subscriber Account".to_string(),
        )]);
        assert_eq!(
            resolve_column(&headers, &["user_account", "subscriber_account"]),
            Some("Subscriber Account".to_string())
        );
    }

    #[test]
    fn parses_cidr_and_range() {
        assert_eq!(
            parse_ipv4_range("10.0.0.0/24").unwrap(),
            (167_772_160, 167_772_415)
        );
        assert_eq!(
            parse_ipv4_range("10.0.0.1-10.0.0.2").unwrap(),
            (167_772_161, 167_772_162)
        );
    }

    #[test]
    fn missing_workspace_has_empty_duckdb_batch_list() {
        let root = std::env::temp_dir().join(format!(
            "sa-fbb-duckdb-empty-test-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let settings = DuckDbWorkspaceSettings {
            workspace_dir: root.display().to_string(),
        };
        let batches = duckdb_list_batches(settings).unwrap();
        assert!(batches.is_empty());
        assert!(!root.join("workspace.duckdb").exists());
    }

    #[test]
    fn analyzes_small_tcp_fixture_and_closes_run() {
        let root = std::env::temp_dir().join(format!(
            "sa-fbb-duckdb-test-{}",
            uuid::Uuid::new_v4().simple()
        ));
        fs::create_dir_all(&root).unwrap();
        let csv_path = root.join("fixture.csv");
        fs::write(
            &csv_path,
            concat!(
                "Subscriber Account,Local IP Address,Universal Video Applications,Statistics Duration,Downloaded Data Volume (KB),Effective Download Duration (s),Subscriber Side RTT (ms),Network Side RTT (ms),User Side Downstream Packet Loss Rate (%),Network Side Downstream Packet Loss Rate (%),vMOS\n",
                "user-a,10.20.0.10,Youtube,2026-08-20 19:10:00,1024,10,40,20,1.5,0.5,3.8\n",
                "user-a,10.20.0.10,Youtube,2026-08-20 20:10:00,2048,10,50,25,2.0,0.7,3.5\n",
                "user-b,192.168.1.10,TikTok,2026-08-20 20:20:00,4096,20,80,45,3.0,1.0,2.9\n",
                "user-c,192.168.1.11,TikTok,invalid,1024,10,90,50,4.0,1.2,2.5\n"
            ),
        )
        .unwrap();
        let settings = DuckDbWorkspaceSettings {
            workspace_dir: root.join("workspace").display().to_string(),
        };
        let req = DuckDbPocRequest {
            workspace_dir: settings.workspace_dir.clone(),
            file_path: csv_path.display().to_string(),
            data_type: "tcp".to_string(),
            batch_display_name: Some("fixture".to_string()),
            default_access_type: Some("CABLE".to_string()),
            ftth_ranges: Some(vec!["10.20.0.0/16".to_string()]),
        };
        let batch_id = "BATCH_TEST";
        let run_id = "RUN_TEST";
        let (paths, connection) = open_workspace(&settings).unwrap();
        prepare_run(&connection, &req, batch_id, run_id).unwrap();
        let result = analyze_csv_sync(&connection, &paths, &req, batch_id, run_id).unwrap();
        assert_eq!(result.source_rows, 4);
        assert_eq!(result.clean_rows, 3);
        assert_eq!(result.hourly_rows, 3);
        let status: String = connection
            .query_row(
                "SELECT status FROM meta_analysis_run WHERE analysis_run_id = 'RUN_TEST'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "success");
        let access_types: i64 = connection
            .query_row(
                "SELECT count(*) FROM ads_access_summary_poc WHERE analysis_run_id = 'RUN_TEST'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(access_types, 2);
        drop(connection);
        let summary =
            duckdb_get_access_summary(settings.clone(), batch_id.to_string(), run_id.to_string())
                .unwrap();
        assert_eq!(summary.len(), 2);
        let batches = duckdb_list_batches(settings).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].import_batch_id, batch_id);
        assert_eq!(batches[0].latest_analysis_run_id.as_deref(), Some(run_id));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn closes_failed_metadata_when_csv_contract_is_invalid() {
        let root = std::env::temp_dir().join(format!(
            "sa-fbb-duckdb-failure-test-{}",
            uuid::Uuid::new_v4().simple()
        ));
        fs::create_dir_all(&root).unwrap();
        let csv_path = root.join("invalid.csv");
        fs::write(
            &csv_path,
            "User Account,Statistical Time\nuser-a,2026-08-20 19:10:00\n",
        )
        .unwrap();
        let settings = DuckDbWorkspaceSettings {
            workspace_dir: root.join("workspace").display().to_string(),
        };
        let result = duckdb_poc_analyze_csv_blocking(DuckDbPocRequest {
            workspace_dir: settings.workspace_dir.clone(),
            file_path: csv_path.display().to_string(),
            data_type: "tcp".to_string(),
            batch_display_name: Some("invalid fixture".to_string()),
            default_access_type: Some("CABLE".to_string()),
            ftth_ranges: None,
        });
        let error = result.unwrap_err();
        assert!(error.contains("CSV 缺少应用"), "unexpected error: {error}");
        let (_, connection) = open_workspace(&settings).unwrap();
        let running: i64 = connection
            .query_row(
                "SELECT count(*) FROM meta_analysis_run WHERE status = 'running'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let failed: i64 = connection
            .query_row(
                "SELECT count(*) FROM meta_analysis_run WHERE status = 'failed'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(running, 0);
        assert_eq!(failed, 1);
        drop(connection);
        let _ = fs::remove_dir_all(root);
    }
}

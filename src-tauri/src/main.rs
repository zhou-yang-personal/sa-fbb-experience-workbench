#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;

#[derive(Serialize)]
struct CommandAck {
    status: &'static str,
    message: String,
}

fn ok(message: impl Into<String>) -> CommandAck {
    CommandAck {
        status: "ok",
        message: message.into(),
    }
}

#[tauri::command]
fn db_test_connection() -> CommandAck {
    ok("MySQL connection command stub is ready. Real connection pool will be wired in the next implementation pass.")
}

#[tauri::command]
fn import_probe_csv(path: String) -> CommandAck {
    ok(format!("CSV probe command stub received path: {path}"))
}

#[tauri::command]
fn import_create_batch(data_type: String, source_file_name: String) -> CommandAck {
    ok(format!("Import batch stub created for {data_type}: {source_file_name}"))
}

#[tauri::command]
fn import_start_raw_load(import_batch_id: String) -> CommandAck {
    ok(format!("RAW load stub started for batch: {import_batch_id}"))
}

#[tauri::command]
fn etl_start_clean_job(import_batch_id: String) -> CommandAck {
    ok(format!("RAW to CLEAN job stub started for batch: {import_batch_id}"))
}

#[tauri::command]
fn etl_start_aggregate_job(import_batch_id: String) -> CommandAck {
    ok(format!("CLEAN to DWS/ADS job stub started for batch: {import_batch_id}"))
}

#[tauri::command]
fn dashboard_get_overview() -> CommandAck {
    ok("Overview dashboard query stub is ready.")
}

#[tauri::command]
fn leads_query_users() -> CommandAck {
    ok("Migration leads query stub is ready.")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            db_test_connection,
            import_probe_csv,
            import_create_batch,
            import_start_raw_load,
            etl_start_clean_job,
            etl_start_aggregate_job,
            dashboard_get_overview,
            leads_query_users
        ])
        .run(tauri::generate_context!())
        .expect("error while running SA FBB Experience Workbench");
}

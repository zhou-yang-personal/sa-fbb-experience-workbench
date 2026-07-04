#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config_commands;
mod dashboard_commands;
mod dataset_profile_commands;
mod db;
mod etl_commands;
mod final_fusion;
mod import_commands;
mod import_commands_mapped;
mod job_inspection_commands;
mod job_inspector;
mod job_runner;
mod lead_commands;
mod mapping_validation_commands;
mod migrations;
mod models;
mod phase_commands;
mod probe;
mod quality_result_commands;
mod raw_import;
mod raw_import_v2;
mod sql_runner;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            dashboard_commands::db_test_connection,
            dashboard_commands::db_initialize,
            import_commands::import_probe_csv,
            import_commands::import_create_batch,
            import_commands_mapped::import_start_raw_load,
            import_commands::import_get_batch_status,
            mapping_validation_commands::import_validate_mapping,
            dataset_profile_commands::dataset_profile_refresh,
            dataset_profile_commands::dataset_profile_get,
            etl_commands::etl_get_recent_jobs,
            job_inspection_commands::etl_get_recent_steps,
            job_inspection_commands::etl_get_failed_steps,
            quality_result_commands::quality_get_gate_results,
            quality_result_commands::quality_get_failed_results,
            dashboard_commands::quality_get_batch_report,
            etl_commands::etl_start_clean_job,
            etl_commands::etl_start_aggregate_job,
            dashboard_commands::dashboard_get_overview,
            lead_commands::leads_query_users,
            lead_commands::final_leads_query_users,
            lead_commands::export_leads_csv,
            lead_commands::export_final_leads_csv,
            config_commands::config_seed_defaults,
            config_commands::config_get_import_mappings,
            config_commands::config_get_join_rules,
            phase_commands::quality_run_gate,
            phase_commands::etl_run_complete_aggregates,
            phase_commands::ads_run_complete_dashboards,
            phase_commands::leads_run_final_fusion,
            phase_commands::dashboard_get_app_category,
            phase_commands::dashboard_get_experience_quality,
            phase_commands::dashboard_get_cable_fiber_compare,
            phase_commands::leads_get_final_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running SA FBB Experience Workbench");
}

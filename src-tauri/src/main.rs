#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis_commands;
mod analytics_app_commands;
mod analytics_commands;
mod analytics_hourly_commands;
mod analytics_lead_commands;
mod analytics_lead_page_commands;
mod analytics_network_commands;
mod analytics_user_commands;
mod batch_tables;
mod config_commands;
mod dashboard_commands;
mod dataset_profile_commands;
mod db;
mod etl_commands;
mod final_fusion;
mod header_normalizer;
mod import_commands;
mod import_commands_mapped;
mod import_pipeline_commands;
mod job_inspection_commands;
mod job_inspector;
mod job_runner;
mod lead_commands;
mod mapping_catalog;
mod mapping_result_commands;
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
            analysis_commands::import_list_batches,
            analysis_commands::analysis_prepare_batch_tables,
            analysis_commands::batch_get_table_registry,
            analysis_commands::analysis_get_module_status,
            analysis_commands::analysis_export_module_csv,
            analysis_commands::analysis_get_module_metrics,
            analytics_commands::analytics_get_kpi_summary,
            analytics_app_commands::analytics_get_app_rank,
            analytics_hourly_commands::analytics_get_hourly_trend,
            analytics_network_commands::analytics_get_network_hotspots,
            analytics_user_commands::analytics_get_user_profiles,
            analytics_lead_commands::analytics_get_lead_evidence,
            analytics_lead_page_commands::analytics_get_lead_evidence_page,
            import_commands::import_probe_csv,
            import_commands::import_create_batch,
            import_commands::import_current_file_atomic,
            import_pipeline_commands::import_pipeline_start,
            import_pipeline_commands::import_pipeline_get_status,
            import_pipeline_commands::import_pipeline_get_logs,
            import_commands_mapped::import_start_raw_load,
            import_commands::import_get_batch_status,
            mapping_validation_commands::import_validate_mapping,
            mapping_result_commands::import_get_mapping_results,
            mapping_result_commands::import_get_mapping_summary,
            dataset_profile_commands::dataset_profile_refresh,
            dataset_profile_commands::dataset_profile_get,
            etl_commands::etl_get_recent_jobs,
            job_inspection_commands::etl_get_recent_steps,
            job_inspection_commands::etl_get_failed_steps,
            job_inspection_commands::etl_get_job_steps,
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
            config_commands::config_check_import_catalog,
            config_commands::config_get_import_mappings,
            config_commands::config_get_join_rules,
            phase_commands::quality_run_gate,
            phase_commands::etl_run_complete_aggregates,
            phase_commands::ads_run_complete_dashboards,
            phase_commands::leads_run_final_fusion,
            phase_commands::dashboard_get_app_category,
            phase_commands::dashboard_get_experience_quality,
            phase_commands::dashboard_get_game_experience,
            phase_commands::dashboard_get_network_quality,
            phase_commands::dashboard_get_user_profile,
            phase_commands::dashboard_get_video_experience_detail,
            phase_commands::dashboard_get_cable_fiber_compare,
            phase_commands::dashboard_get_cable_fiber_hourly_detail,
            phase_commands::leads_get_final_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running SA FBB Experience Workbench");
}

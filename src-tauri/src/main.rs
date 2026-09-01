#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod access_rule_commands;
mod ads_hour;
mod ads_lead;
mod ads_net;
mod ads_user;
mod analysis_commands;
mod analytics_ads_app;
mod analytics_app_commands;
mod analytics_commands;
mod analytics_hourly_commands;
mod analytics_lead_commands;
mod analytics_lead_page_commands;
mod analytics_network_commands;
mod analytics_user_commands;
mod batch_delete_commands;
mod batch_tables;
mod command_guard;
mod config_commands;
mod dashboard_commands;
mod dataset_profile_commands;
mod decision_workspace_commands;
mod db;
mod duckdb_workspace;
mod etl_commands;
mod experience_policy_commands;
mod final_fusion;
mod header_normalizer;
mod import_commands;
mod import_commands_mapped;
mod import_pipeline_commands;
mod investigation_commands;
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

pub(crate) fn append_runtime_log(event: &str) {
    use std::io::Write;

    let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let directory = std::path::PathBuf::from(local_app_data).join("SA FBB Experience Workbench");
    if std::fs::create_dir_all(&directory).is_err() {
        return;
    }
    let path = directory.join("runtime.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let normalized = event.replace('\r', " ").replace('\n', " ");
        let _ = writeln!(
            file,
            "{} version={} {}",
            chrono::Utc::now().to_rfc3339(),
            env!("CARGO_PKG_VERSION"),
            normalized,
        );
    }
}

fn install_panic_log() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        append_runtime_log(&format!("rust_panic {panic_info}"));
        default_hook(panic_info);
    }));
}

fn main() {
    install_panic_log();
    append_runtime_log("application_start safe_startup=no_database_commands");
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            dashboard_commands::db_test_connection,
            dashboard_commands::db_initialize,
            duckdb_workspace::duckdb_workspace_initialize,
            duckdb_workspace::duckdb_workspace_status,
            duckdb_workspace::duckdb_poc_analyze_csv,
            duckdb_workspace::duckdb_list_batches,
            duckdb_workspace::duckdb_list_analysis_runs,
            duckdb_workspace::duckdb_get_access_summary,
            duckdb_workspace::duckdb_get_access_hourly,
            access_rule_commands::access_rule_list_sets,
            access_rule_commands::access_rule_get_or_create_draft,
            access_rule_commands::access_rule_list,
            access_rule_commands::access_rule_set_default_update,
            access_rule_commands::access_rule_upsert,
            access_rule_commands::access_rule_delete,
            access_rule_commands::access_rule_validate,
            access_rule_commands::access_rule_publish,
            access_rule_commands::access_rule_apply_to_batch,
            access_rule_commands::access_rule_preview,
            experience_policy_commands::experience_policy_list,
            experience_policy_commands::experience_profile_list,
            experience_policy_commands::experience_policy_create_draft,
            experience_policy_commands::experience_policy_update,
            experience_policy_commands::experience_profile_update,
            experience_policy_commands::experience_profile_clone,
            experience_policy_commands::experience_policy_publish,
            analysis_commands::import_list_batches,
            analysis_commands::analysis_list_runs,
            batch_delete_commands::import_delete_batch,
            analysis_commands::analysis_prepare_batch_tables,
            analysis_commands::batch_get_table_registry,
            analysis_commands::batch_get_table_registry_cached,
            analysis_commands::analysis_get_module_status,
            analysis_commands::analysis_export_module_csv,
            analysis_commands::analysis_get_module_metrics,
            analytics_commands::analytics_get_kpi_summary,
            analytics_commands::analytics_get_data_coverage,
            decision_workspace_commands::decision_get_metric_panorama,
            decision_workspace_commands::decision_get_app_panorama,
            decision_workspace_commands::decision_get_user_distributions,
            decision_workspace_commands::decision_get_quality_overview,
            decision_workspace_commands::decision_get_access_compare,
            decision_workspace_commands::decision_get_access_hourly,
            decision_workspace_commands::decision_get_panorama_hourly,
            decision_workspace_commands::decision_get_access_user_bands,
            decision_workspace_commands::decision_get_access_apps,
            decision_workspace_commands::decision_materialize_opportunities,
            decision_workspace_commands::decision_get_opportunities,
            decision_workspace_commands::decision_get_opportunity_candidates,
            decision_workspace_commands::decision_export_opportunity_candidates_csv,
            decision_workspace_commands::decision_rule_list,
            decision_workspace_commands::decision_rule_create_draft,
            decision_workspace_commands::decision_rule_update,
            decision_workspace_commands::decision_rule_publish,
            investigation_commands::analytics_get_experience_status_v2,
            investigation_commands::analytics_get_findings_v2,
            investigation_commands::analytics_get_data_coverage_v2,
            investigation_commands::analytics_get_run_verification_v2,
            investigation_commands::analytics_get_investigation_evidence,
            investigation_commands::analytics_get_investigation_hourly,
            investigation_commands::analytics_get_investigation_server_ips,
            investigation_commands::investigation_save,
            investigation_commands::investigation_list,
            analytics_ads_app::analytics_materialize_app_rank,
            ads_hour::ads_hour,
            ads_user::ads_user,
            ads_lead::ads_lead,
            ads_net::ads_net,
            analytics_app_commands::analytics_get_app_rank,
            analytics_hourly_commands::analytics_get_hourly_trend,
            analytics_network_commands::analytics_get_network_hotspots,
            analytics_user_commands::analytics_get_user_profiles,
            analytics_user_commands::analytics_get_user_summary,
            analytics_lead_commands::analytics_get_lead_evidence,
            analytics_lead_commands::analytics_get_lead_stage_summary,
            analytics_lead_page_commands::analytics_get_lead_evidence_page,
            import_commands::import_probe_csv,
            import_commands::import_create_batch,
            import_commands::import_current_file_atomic,
            import_pipeline_commands::import_pipeline_start,
            import_pipeline_commands::import_pipeline_rebuild_batch_from_raw,
            import_pipeline_commands::import_pipeline_resume_batch,
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
        .run(tauri::generate_context!());
    match result {
        Ok(()) => append_runtime_log("application_exit normal"),
        Err(err) => {
            append_runtime_log(&format!("tauri_run_error {err}"));
            panic!("error while running SA FBB Experience Workbench: {err}");
        }
    }
}

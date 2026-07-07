use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::final_fusion;
use crate::job_runner::{self, JobStep};
use crate::models::{ack, CommandAck, DashboardRequest, EtlRequest, MetricCard, MySqlSettings};
use crate::sql_runner;

const TCP_QUALITY_SQL: &str =
    include_str!("../../database/sql/quality/001_tcp_raw_quality_gate.sql");
const GAME_QUALITY_SQL: &str =
    include_str!("../../database/sql/quality/002_game_raw_quality_gate.sql");
const COMPLETE_DWS_SQL: &str =
    include_str!("../../database/sql/clean_to_dws/002_complete_aggregates.sql");
const COMPLETE_DASHBOARD_SQL: &str =
    include_str!("../../database/sql/dws_to_ads/002_complete_dashboards.sql");

fn fetch_batch_data_type(
    settings: &MySqlSettings,
    import_batch_id: &str,
) -> Result<String, String> {
    let mut conn = db::conn(settings)?;
    let data_type: Option<String> = conn
        .exec_first(
            "SELECT data_type FROM meta_import_batch WHERE import_batch_id=? LIMIT 1",
            (import_batch_id,),
        )
        .map_err(|err| format!("failed to query batch data_type for quality gate: {err}"))?;
    Ok(data_type
        .unwrap_or_else(|| "mixed".to_string())
        .to_lowercase())
}

fn quality_templates_for_data_type(
    data_type: &str,
) -> Vec<(&'static str, &'static str, &'static str)> {
    match data_type.to_lowercase().as_str() {
        "tcp" => vec![(
            "tcp_raw_quality_gate",
            "raw_tcp_detail_import,dwd_tcp_detail_clean",
            TCP_QUALITY_SQL,
        )],
        "game" => vec![(
            "game_raw_quality_gate",
            "raw_game_detail_import,dwd_game_detail_clean",
            GAME_QUALITY_SQL,
        )],
        "mixed" => vec![
            (
                "tcp_raw_quality_gate",
                "raw_tcp_detail_import,dwd_tcp_detail_clean",
                TCP_QUALITY_SQL,
            ),
            (
                "game_raw_quality_gate",
                "raw_game_detail_import,dwd_game_detail_clean",
                GAME_QUALITY_SQL,
            ),
        ],
        _ => vec![],
    }
}

fn quality_not_applicable_sql(import_batch_id: &str, data_type: &str) -> String {
    format!(
        "INSERT INTO meta_quality_check_result (import_batch_id, check_section, check_item, metric_name, metric_value, metric_text, severity, passed) VALUES ('{}', 'raw_quality', '{}_quality_not_applicable', 'skipped', 0, 'Quality Gate is not applicable for auxiliary data_type={}', 'info', 1);",
        sql_runner::escape_sql_literal(import_batch_id),
        sql_runner::escape_sql_literal(data_type),
        sql_runner::escape_sql_literal(data_type)
    )
}

#[tauri::command]
pub fn quality_run_gate(req: EtlRequest) -> Result<CommandAck, String> {
    batch_tables::ensure_batch_tables(&req.settings, &req.import_batch_id)?;
    let data_type = fetch_batch_data_type(&req.settings, &req.import_batch_id)?;
    let mut conn = db::conn(&req.settings)?;
    conn.exec_drop(
        "DELETE FROM meta_quality_check_result WHERE import_batch_id=?",
        (&req.import_batch_id,),
    )
    .map_err(|err| format!("failed to clear quality gate results: {err}"))?;
    drop(conn);
    let templates = quality_templates_for_data_type(&data_type);
    let steps = if templates.is_empty() {
        vec![JobStep {
            step_name: "raw_quality_gate_skipped_not_applicable",
            source_table: "meta_import_batch",
            target_table: "meta_quality_check_result",
            sql_template: "quality_not_applicable",
            sql: quality_not_applicable_sql(&req.import_batch_id, &data_type),
        }]
    } else {
        templates
            .into_iter()
            .map(|(step_name, source_table, template)| {
                let bound = sql_runner::bind_batch_params(template, &req.import_batch_id, None);
                let sql =
                    batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
                Ok(JobStep {
                    step_name,
                    source_table,
                    target_table: "meta_quality_check_result",
                    sql_template: if step_name.starts_with("tcp") {
                        "001_tcp_raw_quality_gate.sql"
                    } else {
                        "002_game_raw_quality_gate.sql"
                    },
                    sql,
                })
            })
            .collect::<Result<Vec<_>, String>>()?
    };
    let message = job_runner::run_job(&req.settings, &req.import_batch_id, "quality_gate", steps)?;
    Ok(ack(message))
}

#[tauri::command]
pub fn etl_run_complete_aggregates(req: EtlRequest) -> Result<CommandAck, String> {
    let bound = sql_runner::bind_batch_params(COMPLETE_DWS_SQL, &req.import_batch_id, None);
    let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
    let dwd_tcp =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;
    let dwd_game =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_game_detail_clean")?;
    let dws_user = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_daily_profile",
    )?;
    let dws_app = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_app_category_daily",
    )?;
    let dws_access = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_access_type_hourly_compare",
    )?;
    let dws_bottleneck = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_experience_bottleneck",
    )?;
    let message = job_runner::run_job(
        &req.settings,
        &req.import_batch_id,
        "complete_dws_aggregate",
        vec![JobStep {
            step_name: "complete_dws_aggregates",
            source_table: Box::leak(format!("{dwd_tcp},{dwd_game},{dws_user}").into_boxed_str()),
            target_table: Box::leak(
                format!("{dws_app},{dws_access},{dws_bottleneck}").into_boxed_str(),
            ),
            sql_template: "002_complete_aggregates.sql",
            sql,
        }],
    )?;
    Ok(ack(message))
}

#[tauri::command]
pub fn ads_run_complete_dashboards(req: EtlRequest) -> Result<CommandAck, String> {
    let run_id = req
        .analysis_run_id
        .clone()
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let bound =
        sql_runner::bind_batch_params(COMPLETE_DASHBOARD_SQL, &req.import_batch_id, Some(&run_id));
    let sql = batch_tables::bind_batch_tables(&req.settings, &req.import_batch_id, &bound)?;
    let dws_user = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_daily_profile",
    )?;
    let dws_app = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_app_category_daily",
    )?;
    let dws_access = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_access_type_hourly_compare",
    )?;
    let ads_overview = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_dashboard_overview",
    )?;
    let ads_app = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_app_category_detail",
    )?;
    let ads_quality = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_experience_quality_summary",
    )?;
    let ads_cable = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_cable_fiber_compare",
    )?;
    let message = job_runner::run_job(
        &req.settings,
        &req.import_batch_id,
        "complete_ads_dashboard",
        vec![JobStep {
            step_name: "complete_ads_dashboards",
            source_table: Box::leak(format!("{dws_user},{dws_app},{dws_access}").into_boxed_str()),
            target_table: Box::leak(
                format!("{ads_overview},{ads_app},{ads_quality},{ads_cable}").into_boxed_str(),
            ),
            sql_template: "002_complete_dashboards.sql",
            sql,
        }],
    )?;
    Ok(ack(format!("analysis_run_id={run_id}; {message}")))
}

#[tauri::command]
pub fn leads_run_final_fusion(req: EtlRequest) -> Result<CommandAck, String> {
    let run_id = req
        .analysis_run_id
        .clone()
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let step = final_fusion::build_final_fusion_step(&req.settings, &req.import_batch_id, &run_id)?;
    let message = job_runner::run_job(
        &req.settings,
        &req.import_batch_id,
        "final_lead_fusion",
        vec![step],
    )?;
    Ok(ack(format!("analysis_run_id={run_id}; {message}")))
}

#[cfg(test)]
mod tests {
    use super::{quality_templates_for_data_type, GAME_QUALITY_SQL, TCP_QUALITY_SQL};

    #[test]
    fn quality_gate_routes_by_data_type() {
        let tcp = quality_templates_for_data_type("tcp");
        let game = quality_templates_for_data_type("game");
        let mixed = quality_templates_for_data_type("mixed");

        assert_eq!(
            tcp.iter().map(|item| item.0).collect::<Vec<_>>(),
            vec!["tcp_raw_quality_gate"]
        );
        assert_eq!(
            game.iter().map(|item| item.0).collect::<Vec<_>>(),
            vec!["game_raw_quality_gate"]
        );
        assert_eq!(
            mixed.iter().map(|item| item.0).collect::<Vec<_>>(),
            vec!["tcp_raw_quality_gate", "game_raw_quality_gate"]
        );
        assert!(quality_templates_for_data_type("crm").is_empty());
        assert!(quality_templates_for_data_type("coverage").is_empty());
        assert!(quality_templates_for_data_type("reachability").is_empty());
    }

    #[test]
    fn quality_gate_sql_uses_guarded_clean_timestamp_text() {
        for sql in [TCP_QUALITY_SQL, GAME_QUALITY_SQL] {
            assert!(sql.contains("CHAR(9)"));
            assert!(sql.contains("CHAR(10)"));
            assert!(sql.contains("CHAR(13)"));
            assert!(sql.contains("CONVERT(0xC2A0 USING utf8mb4)"));
            assert!(!sql.contains("CHAR(160)"));
            assert!(sql.contains("REGEXP_REPLACE"));
            assert!(sql.contains("[0-9]{1,2}/[0-9]{1,2}/[0-9]{4}"));
            assert!(sql.contains("stat_time_text"));
            assert!(!sql.contains("STR_TO_DATE(NULLIF(TRIM(statistics_duration)"));
            assert!(!sql.contains("STR_TO_DATE(NULLIF(TRIM(statistical_time)"));
            assert!(!sql.contains("CHAR(9), '')"));
            assert!(!sql.contains("CHAR(10), '')"));
            assert!(!sql.contains("CHAR(13), '')"));
        }
    }
}

#[tauri::command]
pub fn dashboard_get_app_category(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req
        .analysis_run_id
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_app_category_detail",
    )?;
    conn.exec_map(
        format!("SELECT app_category, CAST(SUM(active_users) AS SIGNED), CAST(ROUND(SUM(total_download_gb),2) AS DOUBLE) FROM `{table}` WHERE analysis_run_id=? GROUP BY app_category ORDER BY SUM(active_users) DESC LIMIT 20"),
        (&run_id,),
        |(category, users, gb): (String, i64, f64)| MetricCard { label: category, value: users.to_string(), hint: format!("download_gb={gb}") }
    ).map_err(|err| format!("failed to query app category detail: {err}"))
}

#[tauri::command]
pub fn dashboard_get_experience_quality(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req
        .analysis_run_id
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_experience_quality_summary",
    )?;
    conn.exec_map(
        format!("SELECT quality_dimension, COALESCE(user_type,'ALL'), CAST(ROUND(COALESCE(avg_value,0),2) AS DOUBLE), severity FROM `{table}` WHERE analysis_run_id=? ORDER BY quality_dimension, user_type"),
        (&run_id,),
        |(dimension, user_type, avg_value, severity): (String, String, f64, Option<String>)| MetricCard { label: format!("{dimension} {user_type}"), value: format!("{avg_value}"), hint: severity.unwrap_or_default() }
    ).map_err(|err| format!("failed to query experience quality: {err}"))
}

#[tauri::command]
pub fn dashboard_get_game_experience(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&req.settings)?;
    let table =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_game_detail_clean")?;
    conn.exec_map(
        format!("SELECT COALESCE(app_category,'GAME'), CAST(COUNT(DISTINCT user_key) AS SIGNED), CAST(ROUND(COALESCE(SUM(game_hours),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(mos),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(worst_latency_ms),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(worst_jitter_ms),0),2) AS DOUBLE) FROM `{table}` WHERE import_batch_id=? GROUP BY COALESCE(app_category,'GAME') ORDER BY COUNT(DISTINCT user_key) DESC LIMIT 20"),
        (&req.import_batch_id,),
        |(category, users, hours, mos, latency, jitter): (String, i64, f64, f64, f64, f64)| MetricCard {
            label: category,
            value: format!("mos={mos:.2}"),
            hint: format!("users={users}, game_hours={hours:.2}, latency_ms={latency:.2}, jitter_ms={jitter:.2}"),
        },
    ).map_err(|err| format!("failed to query game experience: {err}"))
}

#[tauri::command]
pub fn dashboard_get_network_quality(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req
        .analysis_run_id
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_experience_quality_summary",
    )?;
    conn.exec_map(
        format!("SELECT quality_dimension, COALESCE(user_type,'ALL'), CAST(COALESCE(SUM(affected_users),0) AS SIGNED), CAST(ROUND(COALESCE(AVG(avg_value),0),2) AS DOUBLE), CAST(ROUND(COALESCE(MAX(p90_value),0),2) AS DOUBLE), COALESCE(MAX(severity),'normal') FROM `{table}` WHERE analysis_run_id=? GROUP BY quality_dimension, COALESCE(user_type,'ALL') ORDER BY quality_dimension, user_type"),
        (&run_id,),
        |(dimension, user_type, users, avg_value, p90_value, severity): (String, String, i64, f64, f64, String)| MetricCard {
            label: format!("{dimension} {user_type}"),
            value: format!("{avg_value:.2}"),
            hint: format!("affected_users={users}, p90={p90_value:.2}, severity={severity}"),
        },
    ).map_err(|err| format!("failed to query network quality: {err}"))
}

#[tauri::command]
pub fn dashboard_get_user_profile(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_daily_profile",
    )?;
    conn.exec_map(
        format!("SELECT user_key, COALESCE(MAX(user_type),'UNKNOWN'), CAST(ROUND(COALESCE(SUM(total_download_gb),0),2) AS DOUBLE), CAST(ROUND(COALESCE(SUM(total_game_hours),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(avg_vmos),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(avg_mos),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(avg_subscriber_rtt_ms),0),2) AS DOUBLE) FROM `{table}` WHERE import_batch_id=? GROUP BY user_key ORDER BY SUM(total_download_gb) DESC, SUM(total_game_hours) DESC LIMIT 30"),
        (&req.import_batch_id,),
        |(user_key, user_type, gb, game_hours, vmos, mos, rtt): (String, String, f64, f64, f64, f64, f64)| MetricCard {
            label: user_key,
            value: format!("{} GB", gb),
            hint: format!("type={user_type}, game_hours={game_hours:.2}, vmos={vmos:.2}, mos={mos:.2}, subscriber_rtt_ms={rtt:.2}"),
        },
    ).map_err(|err| format!("failed to query user profile: {err}"))
}

#[tauri::command]
pub fn dashboard_get_video_experience_detail(
    req: DashboardRequest,
) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&req.settings)?;
    let table =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;
    conn.exec_map(
        format!("SELECT COALESCE(app_category,'VIDEO'), CAST(COUNT(DISTINCT user_key) AS SIGNED), CAST(ROUND(COALESCE(SUM(downloaded_gb),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(vmos),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(effective_download_mbps),0),2) AS DOUBLE), CAST(ROUND(COALESCE(AVG(subscriber_side_rtt_ms),0),2) AS DOUBLE) FROM `{table}` WHERE import_batch_id=? GROUP BY COALESCE(app_category,'VIDEO') ORDER BY SUM(downloaded_gb) DESC LIMIT 20"),
        (&req.import_batch_id,),
        |(category, users, gb, vmos, mbps, rtt): (String, i64, f64, f64, f64, f64)| MetricCard {
            label: category,
            value: format!("vmos={vmos:.2}"),
            hint: format!("users={users}, download_gb={gb:.2}, effective_mbps={mbps:.2}, subscriber_rtt_ms={rtt:.2}"),
        },
    ).map_err(|err| format!("failed to query video experience detail: {err}"))
}

#[tauri::command]
pub fn dashboard_get_cable_fiber_compare(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req
        .analysis_run_id
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_cable_fiber_compare",
    )?;
    conn.exec_map(
        format!("SELECT metric_key, CAST(ROUND(AVG(cable_value),2) AS DOUBLE), CAST(ROUND(AVG(ftth_value),2) AS DOUBLE), CAST(ROUND(AVG(diff_value),2) AS DOUBLE) FROM `{table}` WHERE analysis_run_id=? GROUP BY metric_key ORDER BY metric_key"),
        (&run_id,),
        |(metric, cable, ftth, diff): (String, Option<f64>, Option<f64>, Option<f64>)| MetricCard { label: metric, value: format!("diff={:.2}", diff.unwrap_or(0.0)), hint: format!("cable={:.2}, ftth={:.2}", cable.unwrap_or(0.0), ftth.unwrap_or(0.0)) }
    ).map_err(|err| format!("failed to query cable fiber compare: {err}"))
}

#[tauri::command]
pub fn dashboard_get_cable_fiber_hourly_detail(
    req: DashboardRequest,
) -> Result<Vec<MetricCard>, String> {
    let run_id = req
        .analysis_run_id
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_cable_fiber_compare",
    )?;
    conn.exec_map(
        format!("SELECT CONCAT(metric_key, ' h', LPAD(hour_of_day, 2, '0')), CAST(ROUND(AVG(COALESCE(cable_value,0)),2) AS DOUBLE), CAST(ROUND(AVG(COALESCE(ftth_value,0)),2) AS DOUBLE), CAST(ROUND(AVG(COALESCE(diff_value,0)),2) AS DOUBLE), CAST(COUNT(*) AS SIGNED) FROM `{table}` WHERE analysis_run_id=? GROUP BY metric_key, hour_of_day ORDER BY metric_key, hour_of_day"),
        (&run_id,),
        |(label, cable, ftth, diff, rows): (String, f64, f64, f64, i64)| MetricCard {
            label,
            value: format!("diff={diff:.2}"),
            hint: format!("cable={cable:.2}, ftth={ftth:.2}, rows={rows}"),
        },
    ).map_err(|err| format!("failed to query cable fiber hourly detail: {err}"))
}

#[tauri::command]
pub fn leads_get_final_summary(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = req
        .analysis_run_id
        .unwrap_or_else(|| "RUN_DEFAULT".to_string());
    let mut conn = db::conn(&req.settings)?;
    let table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_final_marketing_lead_user",
    )?;
    conn.exec_map(
        format!("SELECT COALESCE(final_action,'UNKNOWN'), CAST(COUNT(*) AS SIGNED) FROM `{table}` WHERE analysis_run_id=? GROUP BY COALESCE(final_action,'UNKNOWN') ORDER BY COUNT(*) DESC"),
        (&run_id,),
        |(action, count): (String, i64)| MetricCard { label: action, value: count.to_string(), hint: "final marketing lead action".to_string() }
    ).map_err(|err| format!("failed to query final lead summary: {err}"))
}

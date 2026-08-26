use mysql::prelude::*;

use crate::batch_tables;
use crate::db;
use crate::models::{DashboardRequest, MetricCard};

fn run_id(req: &DashboardRequest) -> String {
    req.analysis_run_id
        .clone()
        .unwrap_or_else(|| "RUN_DEFAULT".to_string())
}

#[tauri::command]
pub fn analytics_get_kpi_summary(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let run_id = run_id(&req);
    let mut conn = db::conn(&req.settings)?;
    let user_table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_user_daily_profile",
    )?;
    let lead_table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_migration_lead_user",
    )?;
    let access_type = req.access_type_filter();
    let keyword = req.keyword_like();
    let user_sql = format!("SELECT CAST(COUNT(DISTINCT user_key) AS SIGNED), CAST(COUNT(DISTINCT CASE WHEN user_type='CABLE' THEN user_key END) AS SIGNED), CAST(COUNT(DISTINCT CASE WHEN user_type='FTTH' THEN user_key END) AS SIGNED), CAST(ROUND(COALESCE(SUM(total_download_gb),0),2) AS DOUBLE), CAST(ROUND(COALESCE(SUM(total_game_hours),0),2) AS DOUBLE) FROM `{user_table}` WHERE import_batch_id=? AND (? IS NULL OR COALESCE(user_type,'UNKNOWN')=?) AND (? IS NULL OR user_key LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ?)");
    let (users, cable, ftth, traffic, game_hours): (i64, i64, i64, f64, f64) = conn
        .exec_first(user_sql, (&req.import_batch_id, access_type.clone(), access_type.clone(), keyword.clone(), keyword.clone(), keyword.clone()))
        .map_err(|err| format!("failed to query analytics KPI users: {err}"))?
        .unwrap_or((0, 0, 0, 0.0, 0.0));
    let raw_game = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "raw_game_detail_import",
    )?;
    let game_available = batch_tables::table_has_rows(&mut conn, &raw_game)?;
    let lead_sql = format!("SELECT CAST(COUNT(DISTINCT CASE WHEN LEFT(lead_type,3)='A1_' OR LEFT(lead_type,2) IN ('B_','C_') THEN user_key END) AS SIGNED) FROM `{lead_table}` WHERE analysis_run_id=? AND (? IS NULL OR COALESCE(user_type,'UNKNOWN')=?) AND (? IS NULL OR user_key LIKE ? OR COALESCE(lead_type,'UNKNOWN') LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ?)");
    let leads: i64 = conn
        .exec_first(lead_sql, (&run_id, access_type.clone(), access_type.clone(), keyword.clone(), keyword.clone(), keyword.clone(), keyword.clone()))
        .map_err(|err| format!("failed to query analytics KPI leads: {err}"))?
        .unwrap_or(0);
    let problem_apps = if let Ok(table) = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_app_experience_v2",
    ) {
        if batch_tables::table_has_analysis_run(&mut conn, &table, &run_id).unwrap_or(false) {
            let sql = format!("SELECT CAST(COUNT(DISTINCT app_name) AS SIGNED) FROM `{table}` WHERE analysis_run_id=? AND sample_status='SUFFICIENT' AND persistent_poor_users>0 AND (? IS NULL OR user_type=?) AND (? IS NULL OR app_category LIKE ? OR app_name LIKE ? OR user_type LIKE ?) AND eligible_users>=?");
            conn.exec_first::<i64, _, _>(sql, (&run_id, access_type.clone(), access_type.clone(), keyword.clone(), keyword.clone(), keyword.clone(), keyword.clone(), req.min_value()))
                .map_err(|err| format!("failed to query full problem App KPI: {err}"))?
        } else {
            None
        }
    } else {
        None
    };
    let severe_hotspots = if let Ok(table) = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "ads_network_hotspot_rank",
    ) {
        let sql = format!("SELECT CAST(COUNT(DISTINCT CONCAT_WS('|',NULLIF(bras,'UNKNOWN'),NULLIF(olt,'UNKNOWN'),NULLIF(pon,'UNKNOWN'))) AS SIGNED) FROM `{table}` WHERE analysis_run_id=? AND main_issue_driver='NETWORK_SIDE_SEVERE' AND (COALESCE(bras,'UNKNOWN')<>'UNKNOWN' OR COALESCE(olt,'UNKNOWN')<>'UNKNOWN' OR COALESCE(pon,'UNKNOWN')<>'UNKNOWN') AND (? IS NULL OR COALESCE(user_type,'UNKNOWN')=?) AND (? IS NULL OR COALESCE(main_issue_driver,'UNKNOWN') LIKE ? OR COALESCE(user_type,'UNKNOWN') LIKE ? OR bras LIKE ? OR olt LIKE ? OR pon LIKE ?) AND affected_users>=?");
        conn.exec_first::<i64, _, _>(sql, (&run_id, access_type.clone(), access_type.clone(), keyword.clone(), keyword.clone(), keyword.clone(), keyword.clone(), keyword.clone(), keyword.clone(), req.min_value()))
            .map_err(|err| format!("failed to query full network hotspot KPI: {err}"))?
            .unwrap_or(0)
    } else {
        0
    };
    let hourly_table = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "dws_access_type_hourly_compare",
    )?;
    let coverage_sql = format!("SELECT CAST(ROUND(CASE WHEN COALESCE(SUM(active_users),0)=0 THEN 0 ELSE 100.0*SUM(CASE WHEN COALESCE(user_type,'UNKNOWN')<>'UNKNOWN' THEN active_users ELSE 0 END)/SUM(active_users) END,2) AS DOUBLE) FROM `{hourly_table}` WHERE import_batch_id=? AND (? IS NULL OR COALESCE(user_type,'UNKNOWN')=?) AND (? IS NULL OR COALESCE(user_type,'UNKNOWN') LIKE ?) AND active_users>=?");
    let access_coverage: f64 = conn
        .exec_first(coverage_sql, (&req.import_batch_id, access_type.clone(), access_type.clone(), keyword.clone(), keyword.clone(), req.min_value()))
        .map_err(|err| format!("failed to query full access coverage KPI: {err}"))?
        .unwrap_or(0.0);
    Ok(vec![
        MetricCard {
            label: "Total Users".into(),
            value: users.to_string(),
            hint: "source=dws_user_daily_profile".into(),
        },
        MetricCard {
            label: "Cable Users".into(),
            value: cable.to_string(),
            hint: "filter=user_type=CABLE".into(),
        },
        MetricCard {
            label: "FTTH Users".into(),
            value: ftth.to_string(),
            hint: "filter=user_type=FTTH".into(),
        },
        MetricCard {
            label: "Total Traffic GB".into(),
            value: format!("{traffic:.2}"),
            hint: "metric=sum(total_download_gb)".into(),
        },
        MetricCard {
            label: "Game Hours".into(),
            value: if game_available {
                format!("{game_hours:.2}")
            } else {
                "NOT_IMPORTED".into()
            },
            hint: format!("metric=sum(total_game_hours), game_dataset_available={game_available}"),
        },
        MetricCard {
            label: "Qualified Opportunity Users".into(),
            value: leads.to_string(),
            hint: format!(
                "analysis_run_id={run_id}, included_stages=A1|B|C, excluded_stages=A0|A2|D"
            ),
        },
        MetricCard {
            label: "Access Classification Coverage".into(),
            value: format!("{access_coverage:.2}"),
            hint: "metric=known_access_active_users/all_active_users, unit=pct, scope=full_filtered_aggregate".into(),
        },
        MetricCard {
            label: "Problem Apps".into(),
            value: problem_apps
                .map(|value| value.to_string())
                .unwrap_or_else(|| "NOT_AVAILABLE".to_string()),
            hint: "metric=count_distinct(app_name), filter=sample_status=SUFFICIENT AND persistent_poor_users>0, scope=full_filtered_aggregate, source=ads_app_experience_v2".into(),
        },
        MetricCard {
            label: "Network Severe Hotspots".into(),
            value: severe_hotspots.to_string(),
            hint: "metric=count_distinct(real_topology), filter=main_issue_driver=NETWORK_SIDE_SEVERE, scope=full_filtered_aggregate".into(),
        },
    ])
}

#[tauri::command]
pub fn analytics_get_data_coverage(req: DashboardRequest) -> Result<Vec<MetricCard>, String> {
    let mut conn = db::conn(&req.settings)?;
    crate::migrations::ensure_access_columns_for_table(&mut conn, "meta_access_rule_set")?;
    let raw_tcp =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "raw_tcp_detail_import")?;
    let raw_game = batch_tables::resolve_table(
        &req.settings,
        &req.import_batch_id,
        "raw_game_detail_import",
    )?;
    let clean_tcp =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_tcp_detail_clean")?;
    let clean_game =
        batch_tables::resolve_table(&req.settings, &req.import_batch_id, "dwd_game_detail_clean")?;
    let tcp_raw_ready = batch_tables::table_has_rows(&mut conn, &raw_tcp)?;
    let game_raw_ready = batch_tables::table_has_rows(&mut conn, &raw_game)?;
    let tcp_clean_ready = batch_tables::table_has_rows(&mut conn, &clean_tcp)?;
    let game_clean_ready = batch_tables::table_has_rows(&mut conn, &clean_game)?;
    let context: Option<(String, Option<String>, Option<i64>, Option<String>)> = conn
        .exec_first(
            "SELECT b.data_type, b.access_rule_set_id, b.access_rule_set_version, s.default_access_type FROM meta_import_batch b LEFT JOIN meta_access_rule_set s ON s.rule_set_id=b.access_rule_set_id WHERE b.import_batch_id=?",
            (&req.import_batch_id,),
        )
        .map_err(|err| format!("failed to query analytics coverage context: {err}"))?;
    let (data_type, rule_set_id, rule_set_version, default_access_type) =
        context.unwrap_or_else(|| ("unknown".to_string(), None, None, None));
    let default_access_type = default_access_type.unwrap_or_else(|| "UNKNOWN".to_string());
    Ok(vec![
        MetricCard {
            label: "TCP Dataset".into(),
            value: if tcp_raw_ready {
                "AVAILABLE"
            } else {
                "NOT_IMPORTED"
            }
            .into(),
            hint: format!(
                "data_type={data_type}, raw_ready={tcp_raw_ready}, clean_ready={tcp_clean_ready}"
            ),
        },
        MetricCard {
            label: "Game Dataset".into(),
            value: if game_raw_ready {
                "AVAILABLE"
            } else {
                "NOT_IMPORTED"
            }
            .into(),
            hint: format!(
                "data_type={data_type}, raw_ready={game_raw_ready}, clean_ready={game_clean_ready}"
            ),
        },
        MetricCard {
            label: "Access Classification".into(),
            value: default_access_type.clone(),
            hint: format!(
                "rule_set_id={}, rule_set_version={}, unmatched_default={default_access_type}",
                rule_set_id.unwrap_or_else(|| "NONE".to_string()),
                rule_set_version
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "NONE".to_string())
            ),
        },
    ])
}

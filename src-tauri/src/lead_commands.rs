use mysql::prelude::*;
use mysql::{Params, Value};

use crate::db;
use crate::models::{ack, CommandAck, ExportLeadsRequest, FinalLeadUserRow, LeadUserRow, LeadsQueryRequest};

fn string_value(value: String) -> Value {
    Value::Bytes(value.into_bytes())
}

fn normalized_filter(value: &Option<String>) -> Option<String> {
    let normalized = value.as_ref()?.trim();
    if normalized.is_empty() || normalized.eq_ignore_ascii_case("ALL") {
        None
    } else {
        Some(normalized.to_string())
    }
}

fn keyword_filter(value: &Option<String>) -> Option<String> {
    let normalized = value.as_ref()?.trim().to_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(format!("%{normalized}%"))
    }
}

fn normalized_actions(values: &Option<Vec<String>>) -> Vec<String> {
    values
        .as_ref()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let normalized = item.trim();
                    if normalized.is_empty() || normalized.eq_ignore_ascii_case("ALL") {
                        None
                    } else {
                        Some(normalized.to_string())
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn leads_query_users(req: LeadsQueryRequest) -> Result<Vec<LeadUserRow>, String> {
    let mut conn = db::conn(&req.settings)?;
    let page = req.page.unwrap_or(1).max(1);
    let page_size = req.page_size.unwrap_or(100).clamp(1, 1000);
    let offset = (page - 1) * page_size;
    let mut where_sql = vec!["analysis_run_id=?".to_string()];
    let mut params = vec![string_value(req.analysis_run_id.clone())];
    if let Some(lead_type) = normalized_filter(&req.lead_type) {
        where_sql.push("lead_type=?".to_string());
        params.push(string_value(lead_type));
    }
    if let Some(keyword) = keyword_filter(&req.keyword) {
        where_sql.push("LOWER(CONCAT_WS(' ', user_key, IFNULL(user_type,''), lead_type, IFNULL(recommended_offer,''))) LIKE ?".to_string());
        params.push(string_value(keyword));
    }
    params.push(Value::UInt(page_size));
    params.push(Value::UInt(offset));
    let sql = format!(
        "SELECT user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer FROM ads_migration_lead_user WHERE {} ORDER BY demand_score DESC, migration_motive_score DESC LIMIT ? OFFSET ?",
        where_sql.join(" AND ")
    );
    conn.exec_map(
        sql,
        Params::Positional(params),
        |(user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer)| LeadUserRow { user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer },
    ).map_err(|err| format!("failed to query leads: {err}"))
}

#[tauri::command]
pub fn final_leads_query_users(req: LeadsQueryRequest) -> Result<Vec<FinalLeadUserRow>, String> {
    let mut conn = db::conn(&req.settings)?;
    let page = req.page.unwrap_or(1).max(1);
    let page_size = req.page_size.unwrap_or(100).clamp(1, 1000);
    let offset = (page - 1) * page_size;
    let mut where_sql = vec!["analysis_run_id=?".to_string()];
    let mut params = vec![string_value(req.analysis_run_id.clone())];
    if let Some(final_action) = normalized_filter(&req.final_action) {
        where_sql.push("COALESCE(final_action, 'UNKNOWN')=?".to_string());
        params.push(string_value(final_action));
    }
    if let Some(keyword) = keyword_filter(&req.keyword) {
        where_sql.push("LOWER(CONCAT_WS(' ', user_key, IFNULL(crm_user_id,''), lead_type, IFNULL(final_action,''), IFNULL(recommended_offer,''), IFNULL(current_plan_name,''), IFNULL(ftth_available_flag,''), IFNULL(reachable_flag,''))) LIKE ?".to_string());
        params.push(string_value(keyword));
    }
    params.push(Value::UInt(page_size));
    params.push(Value::UInt(offset));
    let sql = format!(
        "SELECT user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer FROM ads_final_marketing_lead_user WHERE {} ORDER BY final_action, demand_score DESC, migration_motive_score DESC LIMIT ? OFFSET ?",
        where_sql.join(" AND ")
    );
    conn.exec_map(
        sql,
        Params::Positional(params),
        |(user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer)| FinalLeadUserRow { user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer },
    ).map_err(|err| format!("failed to query final leads: {err}"))
}

#[tauri::command]
pub fn export_leads_csv(req: ExportLeadsRequest) -> Result<CommandAck, String> {
    let mut conn = db::conn(&req.settings)?;
    let mut writer = csv::Writer::from_path(&req.output_path).map_err(|err| format!("failed to create export file: {err}"))?;
    writer.write_record(["user_key", "user_type", "lead_type", "demand_score", "migration_motive_score", "recommended_offer"]).map_err(|err| err.to_string())?;
    let mut exported_rows = 0_u64;
    let mut offset = 0_u64;
    loop {
        let rows: Vec<LeadUserRow> = conn.exec_map(
            "SELECT user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer FROM ads_migration_lead_user WHERE analysis_run_id=? ORDER BY demand_score DESC, migration_motive_score DESC LIMIT 1000 OFFSET ?",
            (&req.analysis_run_id, offset),
            |(user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer)| LeadUserRow { user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer },
        ).map_err(|err| format!("failed to query leads for export: {err}"))?;
        if rows.is_empty() { break; }
        for row in rows {
            writer.write_record([row.user_key, row.user_type.unwrap_or_default(), row.lead_type, row.demand_score.to_string(), row.migration_motive_score.to_string(), row.recommended_offer.unwrap_or_default()]).map_err(|err| err.to_string())?;
            exported_rows += 1;
        }
        offset += 1000;
    }
    writer.flush().map_err(|err| err.to_string())?;
    Ok(ack(format!("leads exported to {}, rows={exported_rows}", req.output_path)))
}

#[tauri::command]
pub fn export_final_leads_csv(req: ExportLeadsRequest) -> Result<CommandAck, String> {
    let mut conn = db::conn(&req.settings)?;
    let final_actions = normalized_actions(&req.final_actions);
    let mut where_sql = vec!["analysis_run_id=?".to_string()];
    let mut base_params = vec![string_value(req.analysis_run_id.clone())];
    if !final_actions.is_empty() {
        let placeholders = std::iter::repeat("?").take(final_actions.len()).collect::<Vec<_>>().join(", ");
        where_sql.push(format!("COALESCE(final_action, 'UNKNOWN') IN ({placeholders})"));
        for action in &final_actions {
            base_params.push(string_value(action.clone()));
        }
    }
    let sql = format!(
        "SELECT user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer FROM ads_final_marketing_lead_user WHERE {} ORDER BY final_action, demand_score DESC, migration_motive_score DESC LIMIT ? OFFSET ?",
        where_sql.join(" AND ")
    );
    let mut writer = csv::Writer::from_path(&req.output_path).map_err(|err| format!("failed to create final lead export file: {err}"))?;
    writer.write_record(["user_key", "crm_user_id", "lead_type", "demand_score", "migration_motive_score", "current_plan_name", "current_arpu", "ftth_available_flag", "reachable_flag", "final_action", "recommended_offer"]).map_err(|err| err.to_string())?;
    let mut exported_rows = 0_u64;
    let mut offset = 0_u64;
    loop {
        let mut query_params = base_params.clone();
        query_params.push(Value::UInt(1000));
        query_params.push(Value::UInt(offset));
        let rows: Vec<FinalLeadUserRow> = conn.exec_map(
            sql.as_str(),
            Params::Positional(query_params),
            |(user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer)| FinalLeadUserRow { user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer },
        ).map_err(|err| format!("failed to query final leads for export: {err}"))?;
        if rows.is_empty() { break; }
        for row in rows {
            writer.write_record([row.user_key, row.crm_user_id.unwrap_or_default(), row.lead_type, row.demand_score.to_string(), row.migration_motive_score.to_string(), row.current_plan_name.unwrap_or_default(), row.current_arpu.map(|v| v.to_string()).unwrap_or_default(), row.ftth_available_flag.unwrap_or_default(), row.reachable_flag.unwrap_or_default(), row.final_action.unwrap_or_default(), row.recommended_offer.unwrap_or_default()]).map_err(|err| err.to_string())?;
            exported_rows += 1;
        }
        offset += 1000;
    }
    writer.flush().map_err(|err| err.to_string())?;
    let filter_text = if final_actions.is_empty() { "final_actions=ALL".to_string() } else { format!("final_actions={}", final_actions.join("|")) };
    Ok(ack(format!("final leads exported to {}, rows={exported_rows}, {filter_text}", req.output_path)))
}

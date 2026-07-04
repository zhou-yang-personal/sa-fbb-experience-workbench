use mysql::prelude::*;

use crate::db;
use crate::models::{ack, CommandAck, ExportLeadsRequest, FinalLeadUserRow, LeadUserRow, LeadsQueryRequest};

#[tauri::command]
pub fn leads_query_users(req: LeadsQueryRequest) -> Result<Vec<LeadUserRow>, String> {
    let mut conn = db::conn(&req.settings)?;
    let page = req.page.unwrap_or(1).max(1);
    let page_size = req.page_size.unwrap_or(100).clamp(1, 1000);
    let offset = (page - 1) * page_size;
    conn.exec_map(
        "SELECT user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer FROM ads_migration_lead_user WHERE analysis_run_id=? ORDER BY demand_score DESC, migration_motive_score DESC LIMIT ? OFFSET ?",
        (&req.analysis_run_id, page_size, offset),
        |(user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer)| LeadUserRow { user_key, user_type, lead_type, demand_score, migration_motive_score, recommended_offer },
    ).map_err(|err| format!("failed to query leads: {err}"))
}

#[tauri::command]
pub fn final_leads_query_users(req: LeadsQueryRequest) -> Result<Vec<FinalLeadUserRow>, String> {
    let mut conn = db::conn(&req.settings)?;
    let page = req.page.unwrap_or(1).max(1);
    let page_size = req.page_size.unwrap_or(100).clamp(1, 1000);
    let offset = (page - 1) * page_size;
    conn.exec_map(
        "SELECT user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer FROM ads_final_marketing_lead_user WHERE analysis_run_id=? ORDER BY final_action, demand_score DESC, migration_motive_score DESC LIMIT ? OFFSET ?",
        (&req.analysis_run_id, page_size, offset),
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
    let mut writer = csv::Writer::from_path(&req.output_path).map_err(|err| format!("failed to create final lead export file: {err}"))?;
    writer.write_record(["user_key", "crm_user_id", "lead_type", "demand_score", "migration_motive_score", "current_plan_name", "current_arpu", "ftth_available_flag", "reachable_flag", "final_action", "recommended_offer"]).map_err(|err| err.to_string())?;
    let mut exported_rows = 0_u64;
    let mut offset = 0_u64;
    loop {
        let rows: Vec<FinalLeadUserRow> = conn.exec_map(
            "SELECT user_key, crm_user_id, lead_type, demand_score, migration_motive_score, current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer FROM ads_final_marketing_lead_user WHERE analysis_run_id=? ORDER BY final_action, demand_score DESC, migration_motive_score DESC LIMIT 1000 OFFSET ?",
            (&req.analysis_run_id, offset),
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
    Ok(ack(format!("final leads exported to {}, rows={exported_rows}", req.output_path)))
}

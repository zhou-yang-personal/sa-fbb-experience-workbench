use mysql::prelude::*;
use mysql::{Params, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db;
use crate::models::{CommandAck, MySqlSettings};

#[derive(Debug, Clone, Serialize)]
pub struct ExperiencePolicyRow {
    policy_id: String,
    version: i64,
    policy_name: String,
    status: String,
    persistent_min_valid_obs: i64,
    persistent_min_poor_obs: i64,
    persistent_min_poor_rate_pct: f64,
    severe_user_min_valid_obs: i64,
    severe_user_min_severe_obs: i64,
    severe_user_min_severe_rate_pct: f64,
    minimum_app_eligible_users: i64,
    minimum_app_valid_obs: i64,
    finding_attention_persistent_user_rate_pct: f64,
    finding_severe_user_rate_pct: f64,
    notes: Option<String>,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppExperienceProfileRow {
    profile_id: String,
    policy_id: String,
    profile_code: String,
    profile_name: String,
    data_type: String,
    app_category: Option<String>,
    priority: i32,
    enabled: bool,
    poor_vmos_below: Option<f64>,
    poor_mos_below: Option<f64>,
    poor_subscriber_rtt_ms_at_least: Option<f64>,
    poor_network_rtt_ms_at_least: Option<f64>,
    poor_user_loss_pct_at_least: Option<f64>,
    poor_network_loss_pct_at_least: Option<f64>,
    poor_jitter_ms_at_least: Option<f64>,
    severe_vmos_below: Option<f64>,
    severe_mos_below: Option<f64>,
    severe_subscriber_rtt_ms_at_least: Option<f64>,
    severe_network_rtt_ms_at_least: Option<f64>,
    severe_user_loss_pct_at_least: Option<f64>,
    severe_network_loss_pct_at_least: Option<f64>,
    severe_jitter_ms_at_least: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyIdRequest { settings: MySqlSettings, policy_id: String }

#[derive(Debug, Clone, Deserialize)]
pub struct CloneProfileRequest { settings: MySqlSettings, policy_id: String, source_profile_id: String }

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePolicyRequest {
    settings: MySqlSettings,
    policy_id: String,
    policy_name: String,
    persistent_min_valid_obs: i64,
    persistent_min_poor_obs: i64,
    persistent_min_poor_rate_pct: f64,
    severe_user_min_valid_obs: i64,
    severe_user_min_severe_obs: i64,
    severe_user_min_severe_rate_pct: f64,
    minimum_app_eligible_users: i64,
    minimum_app_valid_obs: i64,
    finding_attention_persistent_user_rate_pct: f64,
    finding_severe_user_rate_pct: f64,
    notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileRequest {
    settings: MySqlSettings,
    policy_id: String,
    profile_id: String,
    profile_name: String,
    app_category: Option<String>,
    priority: i32,
    enabled: bool,
    poor_vmos_below: Option<f64>,
    poor_mos_below: Option<f64>,
    poor_subscriber_rtt_ms_at_least: Option<f64>,
    poor_network_rtt_ms_at_least: Option<f64>,
    poor_user_loss_pct_at_least: Option<f64>,
    poor_network_loss_pct_at_least: Option<f64>,
    poor_jitter_ms_at_least: Option<f64>,
    severe_vmos_below: Option<f64>,
    severe_mos_below: Option<f64>,
    severe_subscriber_rtt_ms_at_least: Option<f64>,
    severe_network_rtt_ms_at_least: Option<f64>,
    severe_user_loss_pct_at_least: Option<f64>,
    severe_network_loss_pct_at_least: Option<f64>,
    severe_jitter_ms_at_least: Option<f64>,
}

fn ensure(settings: &MySqlSettings) -> Result<(), String> { crate::migrations::ensure_experience_policy_schema(settings) }

#[tauri::command]
pub fn experience_policy_list(settings: MySqlSettings) -> Result<Vec<ExperiencePolicyRow>, String> {
    ensure(&settings)?;
    let mut conn = db::conn(&settings)?;
    let rows = conn.query_iter("SELECT policy_id,version,policy_name,status,CAST(persistent_min_valid_obs AS SIGNED),CAST(persistent_min_poor_obs AS SIGNED),CAST(persistent_min_poor_rate_pct AS DOUBLE),CAST(severe_user_min_valid_obs AS SIGNED),CAST(severe_user_min_severe_obs AS SIGNED),CAST(severe_user_min_severe_rate_pct AS DOUBLE),CAST(minimum_app_eligible_users AS SIGNED),CAST(minimum_app_valid_obs AS SIGNED),CAST(finding_attention_persistent_user_rate_pct AS DOUBLE),CAST(finding_severe_user_rate_pct AS DOUBLE),notes,DATE_FORMAT(updated_at,'%Y-%m-%d %H:%i:%s') FROM meta_experience_analysis_policy ORDER BY version DESC").map_err(|err| format!("failed to list experience policies: {err}"))?;
    rows.map(|row| { let row=row.map_err(|err|format!("failed to decode experience policy: {err}"))?; Ok(ExperiencePolicyRow { policy_id:row.get(0).unwrap_or_default(),version:row.get(1).unwrap_or_default(),policy_name:row.get(2).unwrap_or_default(),status:row.get(3).unwrap_or_default(),persistent_min_valid_obs:row.get(4).unwrap_or_default(),persistent_min_poor_obs:row.get(5).unwrap_or_default(),persistent_min_poor_rate_pct:row.get(6).unwrap_or_default(),severe_user_min_valid_obs:row.get(7).unwrap_or_default(),severe_user_min_severe_obs:row.get(8).unwrap_or_default(),severe_user_min_severe_rate_pct:row.get(9).unwrap_or_default(),minimum_app_eligible_users:row.get(10).unwrap_or_default(),minimum_app_valid_obs:row.get(11).unwrap_or_default(),finding_attention_persistent_user_rate_pct:row.get(12).unwrap_or_default(),finding_severe_user_rate_pct:row.get(13).unwrap_or_default(),notes:row.get(14),updated_at:row.get(15).unwrap_or_default() }) }).collect()
}

#[tauri::command]
pub fn experience_profile_list(req: PolicyIdRequest) -> Result<Vec<AppExperienceProfileRow>, String> {
    ensure(&req.settings)?;
    let mut conn=db::conn(&req.settings)?;
    let rows=conn.exec_iter("SELECT profile_id,policy_id,profile_code,profile_name,data_type,app_category,priority,enabled,CAST(poor_vmos_below AS DOUBLE),CAST(poor_mos_below AS DOUBLE),CAST(poor_subscriber_rtt_ms_at_least AS DOUBLE),CAST(poor_network_rtt_ms_at_least AS DOUBLE),CAST(poor_user_loss_pct_at_least AS DOUBLE),CAST(poor_network_loss_pct_at_least AS DOUBLE),CAST(poor_jitter_ms_at_least AS DOUBLE),CAST(severe_vmos_below AS DOUBLE),CAST(severe_mos_below AS DOUBLE),CAST(severe_subscriber_rtt_ms_at_least AS DOUBLE),CAST(severe_network_rtt_ms_at_least AS DOUBLE),CAST(severe_user_loss_pct_at_least AS DOUBLE),CAST(severe_network_loss_pct_at_least AS DOUBLE),CAST(severe_jitter_ms_at_least AS DOUBLE) FROM dim_app_experience_profile WHERE policy_id=? ORDER BY data_type,priority,profile_code",(&req.policy_id,)).map_err(|err|format!("failed to list experience profiles: {err}"))?;
    rows.map(|row| { let row=row.map_err(|err|format!("failed to decode experience profile: {err}"))?; Ok(AppExperienceProfileRow { profile_id:row.get(0).unwrap_or_default(),policy_id:row.get(1).unwrap_or_default(),profile_code:row.get(2).unwrap_or_default(),profile_name:row.get(3).unwrap_or_default(),data_type:row.get(4).unwrap_or_default(),app_category:row.get(5),priority:row.get(6).unwrap_or_default(),enabled:row.get::<i8,_>(7).unwrap_or_default()!=0,poor_vmos_below:row.get(8),poor_mos_below:row.get(9),poor_subscriber_rtt_ms_at_least:row.get(10),poor_network_rtt_ms_at_least:row.get(11),poor_user_loss_pct_at_least:row.get(12),poor_network_loss_pct_at_least:row.get(13),poor_jitter_ms_at_least:row.get(14),severe_vmos_below:row.get(15),severe_mos_below:row.get(16),severe_subscriber_rtt_ms_at_least:row.get(17),severe_network_rtt_ms_at_least:row.get(18),severe_user_loss_pct_at_least:row.get(19),severe_network_loss_pct_at_least:row.get(20),severe_jitter_ms_at_least:row.get(21) }) }).collect()
}

#[tauri::command]
pub fn experience_policy_create_draft(settings: MySqlSettings) -> Result<CommandAck, String> {
    ensure(&settings)?;
    let mut conn=db::conn(&settings)?;
    let source: Option<String>=conn.query_first("SELECT policy_id FROM meta_experience_analysis_policy WHERE status='published' ORDER BY version DESC LIMIT 1").map_err(|err|format!("failed to find published policy: {err}"))?;
    let source=source.ok_or_else(||"no published experience policy to clone".to_string())?;
    let version: i64=conn.query_first("SELECT COALESCE(MAX(version),0)+1 FROM meta_experience_analysis_policy").map_err(|err|format!("failed to allocate policy version: {err}"))?.unwrap_or(1);
    let id=format!("POLICY_DRAFT_{}",Uuid::new_v4().simple());
    conn.exec_drop("INSERT INTO meta_experience_analysis_policy (policy_id,version,policy_name,status,persistent_min_valid_obs,persistent_min_poor_obs,persistent_min_poor_rate_pct,severe_user_min_valid_obs,severe_user_min_severe_obs,severe_user_min_severe_rate_pct,minimum_app_eligible_users,minimum_app_valid_obs,finding_attention_persistent_user_rate_pct,finding_severe_user_rate_pct,notes) SELECT ?,?,CONCAT(policy_name,' draft'),'draft',persistent_min_valid_obs,persistent_min_poor_obs,persistent_min_poor_rate_pct,severe_user_min_valid_obs,severe_user_min_severe_obs,severe_user_min_severe_rate_pct,minimum_app_eligible_users,minimum_app_valid_obs,finding_attention_persistent_user_rate_pct,finding_severe_user_rate_pct,notes FROM meta_experience_analysis_policy WHERE policy_id=?",(&id,version,&source)).map_err(|err|format!("failed to create policy draft: {err}"))?;
    conn.exec_drop("INSERT INTO dim_app_experience_profile (profile_id,policy_id,profile_code,profile_name,data_type,app_category,priority,enabled,poor_vmos_below,poor_mos_below,poor_subscriber_rtt_ms_at_least,poor_network_rtt_ms_at_least,poor_user_loss_pct_at_least,poor_network_loss_pct_at_least,poor_jitter_ms_at_least,severe_vmos_below,severe_mos_below,severe_subscriber_rtt_ms_at_least,severe_network_rtt_ms_at_least,severe_user_loss_pct_at_least,severe_network_loss_pct_at_least,severe_jitter_ms_at_least) SELECT CONCAT('PROFILE_',REPLACE(UUID(),'-','')),?,profile_code,profile_name,data_type,app_category,priority,enabled,poor_vmos_below,poor_mos_below,poor_subscriber_rtt_ms_at_least,poor_network_rtt_ms_at_least,poor_user_loss_pct_at_least,poor_network_loss_pct_at_least,poor_jitter_ms_at_least,severe_vmos_below,severe_mos_below,severe_subscriber_rtt_ms_at_least,severe_network_rtt_ms_at_least,severe_user_loss_pct_at_least,severe_network_loss_pct_at_least,severe_jitter_ms_at_least FROM dim_app_experience_profile WHERE policy_id=?",(&id,&source)).map_err(|err|format!("failed to clone policy profiles: {err}"))?;
    Ok(CommandAck{status:"success".into(),message:format!("draft created: policy_id={id}, version={version}")})
}

fn valid_rate(value:f64)->bool { value.is_finite() && (0.0..=100.0).contains(&value) }

#[tauri::command]
pub fn experience_policy_update(req: UpdatePolicyRequest) -> Result<CommandAck,String> {
    ensure(&req.settings)?;
    if req.policy_name.trim().is_empty() || req.persistent_min_valid_obs<1 || req.persistent_min_poor_obs<1 || req.severe_user_min_valid_obs<1 || req.severe_user_min_severe_obs<1 || req.minimum_app_eligible_users<1 || req.minimum_app_valid_obs<1 || ![req.persistent_min_poor_rate_pct,req.severe_user_min_severe_rate_pct,req.finding_attention_persistent_user_rate_pct,req.finding_severe_user_rate_pct].into_iter().all(valid_rate) { return Err("invalid policy thresholds; counts must be positive and rates must be 0..100".into()); }
    let mut conn=db::conn(&req.settings)?;
    let editable:bool=conn.exec_first("SELECT EXISTS(SELECT 1 FROM meta_experience_analysis_policy WHERE policy_id=? AND status='draft')",(&req.policy_id,)).map_err(|err|format!("failed to validate policy draft: {err}"))?.unwrap_or(false);
    if !editable { return Err("policy is not an editable draft".into()); }
    let params=Params::Positional(vec![Value::from(req.policy_name.trim()),Value::from(req.persistent_min_valid_obs),Value::from(req.persistent_min_poor_obs),Value::from(req.persistent_min_poor_rate_pct),Value::from(req.severe_user_min_valid_obs),Value::from(req.severe_user_min_severe_obs),Value::from(req.severe_user_min_severe_rate_pct),Value::from(req.minimum_app_eligible_users),Value::from(req.minimum_app_valid_obs),Value::from(req.finding_attention_persistent_user_rate_pct),Value::from(req.finding_severe_user_rate_pct),Value::from(req.notes),Value::from(req.policy_id)]);
    conn.exec_drop("UPDATE meta_experience_analysis_policy SET policy_name=?,persistent_min_valid_obs=?,persistent_min_poor_obs=?,persistent_min_poor_rate_pct=?,severe_user_min_valid_obs=?,severe_user_min_severe_obs=?,severe_user_min_severe_rate_pct=?,minimum_app_eligible_users=?,minimum_app_valid_obs=?,finding_attention_persistent_user_rate_pct=?,finding_severe_user_rate_pct=?,notes=? WHERE policy_id=? AND status='draft'",params).map_err(|err|format!("failed to update policy draft: {err}"))?;
    Ok(CommandAck{status:"success".into(),message:"policy draft updated".into()})
}

#[tauri::command]
pub fn experience_profile_update(req: UpdateProfileRequest) -> Result<CommandAck,String> {
    ensure(&req.settings)?;
    let mut conn=db::conn(&req.settings)?;
    let editable:bool=conn.exec_first("SELECT EXISTS(SELECT 1 FROM dim_app_experience_profile p JOIN meta_experience_analysis_policy x ON x.policy_id=p.policy_id AND x.status='draft' WHERE p.policy_id=? AND p.profile_id=?)",(&req.policy_id,&req.profile_id)).map_err(|err|format!("failed to validate experience profile: {err}"))?.unwrap_or(false);
    if !editable { return Err("profile does not belong to an editable draft".into()); }
    let params=Params::Positional(vec![Value::from(req.profile_name),Value::from(req.app_category),Value::from(req.priority),Value::from(req.enabled),Value::from(req.poor_vmos_below),Value::from(req.poor_mos_below),Value::from(req.poor_subscriber_rtt_ms_at_least),Value::from(req.poor_network_rtt_ms_at_least),Value::from(req.poor_user_loss_pct_at_least),Value::from(req.poor_network_loss_pct_at_least),Value::from(req.poor_jitter_ms_at_least),Value::from(req.severe_vmos_below),Value::from(req.severe_mos_below),Value::from(req.severe_subscriber_rtt_ms_at_least),Value::from(req.severe_network_rtt_ms_at_least),Value::from(req.severe_user_loss_pct_at_least),Value::from(req.severe_network_loss_pct_at_least),Value::from(req.severe_jitter_ms_at_least),Value::from(req.policy_id),Value::from(req.profile_id)]);
    conn.exec_drop("UPDATE dim_app_experience_profile p JOIN meta_experience_analysis_policy x ON x.policy_id=p.policy_id AND x.status='draft' SET p.profile_name=?,p.app_category=?,p.priority=?,p.enabled=?,p.poor_vmos_below=?,p.poor_mos_below=?,p.poor_subscriber_rtt_ms_at_least=?,p.poor_network_rtt_ms_at_least=?,p.poor_user_loss_pct_at_least=?,p.poor_network_loss_pct_at_least=?,p.poor_jitter_ms_at_least=?,p.severe_vmos_below=?,p.severe_mos_below=?,p.severe_subscriber_rtt_ms_at_least=?,p.severe_network_rtt_ms_at_least=?,p.severe_user_loss_pct_at_least=?,p.severe_network_loss_pct_at_least=?,p.severe_jitter_ms_at_least=? WHERE p.policy_id=? AND p.profile_id=?",params).map_err(|err|format!("failed to update experience profile: {err}"))?;
    Ok(CommandAck{status:"success".into(),message:"experience profile updated".into()})
}

#[tauri::command]
pub fn experience_profile_clone(req: CloneProfileRequest) -> Result<CommandAck,String> {
    ensure(&req.settings)?;
    let id=format!("PROFILE_{}",Uuid::new_v4().simple());
    let code=format!("CUSTOM_{}",&id[id.len()-8..]);
    let mut conn=db::conn(&req.settings)?;
    conn.exec_drop("INSERT INTO dim_app_experience_profile (profile_id,policy_id,profile_code,profile_name,data_type,app_category,priority,enabled,poor_vmos_below,poor_mos_below,poor_subscriber_rtt_ms_at_least,poor_network_rtt_ms_at_least,poor_user_loss_pct_at_least,poor_network_loss_pct_at_least,poor_jitter_ms_at_least,severe_vmos_below,severe_mos_below,severe_subscriber_rtt_ms_at_least,severe_network_rtt_ms_at_least,severe_user_loss_pct_at_least,severe_network_loss_pct_at_least,severe_jitter_ms_at_least) SELECT ?,p.policy_id,?,CONCAT(p.profile_name,' copy'),p.data_type,NULL,GREATEST(1,p.priority-1),p.enabled,p.poor_vmos_below,p.poor_mos_below,p.poor_subscriber_rtt_ms_at_least,p.poor_network_rtt_ms_at_least,p.poor_user_loss_pct_at_least,p.poor_network_loss_pct_at_least,p.poor_jitter_ms_at_least,p.severe_vmos_below,p.severe_mos_below,p.severe_subscriber_rtt_ms_at_least,p.severe_network_rtt_ms_at_least,p.severe_user_loss_pct_at_least,p.severe_network_loss_pct_at_least,p.severe_jitter_ms_at_least FROM dim_app_experience_profile p JOIN meta_experience_analysis_policy x ON x.policy_id=p.policy_id AND x.status='draft' WHERE p.policy_id=? AND p.profile_id=?",(&id,&code,&req.policy_id,&req.source_profile_id)).map_err(|err|format!("failed to clone experience profile: {err}"))?;
    if conn.affected_rows()==0 { return Err("source profile does not belong to an editable draft".into()); }
    Ok(CommandAck{status:"success".into(),message:format!("profile cloned: {id}")})
}

#[tauri::command]
pub fn experience_policy_publish(req: PolicyIdRequest) -> Result<CommandAck,String> {
    ensure(&req.settings)?;
    let mut conn=db::conn(&req.settings)?;
    let profiles:i64=conn.exec_first("SELECT COUNT(*) FROM dim_app_experience_profile WHERE policy_id=? AND enabled=1",(&req.policy_id,)).map_err(|err|format!("failed to validate profiles: {err}"))?.unwrap_or(0);
    if profiles==0 { return Err("cannot publish a policy without an enabled experience profile".into()); }
    let mut tx=conn.start_transaction(mysql::TxOpts::default()).map_err(|err|format!("failed to start policy publish transaction: {err}"))?;
    tx.query_drop("UPDATE meta_experience_analysis_policy SET status='archived' WHERE status='published'").map_err(|err|format!("failed to archive prior policy: {err}"))?;
    tx.exec_drop("UPDATE meta_experience_analysis_policy SET status='published',published_at=NOW() WHERE policy_id=? AND status='draft'",(&req.policy_id,)).map_err(|err|format!("failed to publish policy: {err}"))?;
    if tx.affected_rows()==0 { return Err("policy is not a draft".into()); }
    tx.commit().map_err(|err|format!("failed to commit policy publish: {err}"))?;
    Ok(CommandAck{status:"success".into(),message:"experience policy published; new analysis runs will snapshot this version".into()})
}

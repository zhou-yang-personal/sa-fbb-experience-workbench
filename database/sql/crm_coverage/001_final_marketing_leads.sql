-- Final lead list after CRM, FTTH coverage and reachability fusion.
-- This keeps SA as opportunity source and applies commercial eligibility filters later.

DELETE FROM ads_final_marketing_lead_user WHERE analysis_run_id = :analysis_run_id;

INSERT INTO ads_final_marketing_lead_user (
  analysis_run_id, user_key, crm_user_id, lead_type, demand_score, migration_motive_score,
  current_plan_name, current_arpu, ftth_available_flag, reachable_flag, final_action, recommended_offer
)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id), base AS (
  SELECT * FROM ads_migration_lead_user WHERE analysis_run_id = (SELECT analysis_run_id FROM params)
), crm AS (
  SELECT * FROM raw_crm_user_import WHERE import_batch_id = (SELECT import_batch_id FROM params)
), reach AS (
  SELECT * FROM raw_reachability_import WHERE import_batch_id = (SELECT import_batch_id FROM params)
), coverage AS (
  SELECT * FROM raw_ftth_coverage_import WHERE import_batch_id = (SELECT import_batch_id FROM params)
)
SELECT
  p.analysis_run_id,
  b.user_key,
  c.crm_user_id,
  b.lead_type,
  b.demand_score,
  b.migration_motive_score,
  c.current_plan_name,
  c.current_arpu,
  COALESCE(MAX(cv.ftth_available_flag), 'UNKNOWN') AS ftth_available_flag,
  CASE
    WHEN MAX(r.phone_available_flag) = 'Y' OR MAX(r.sms_available_flag) = 'Y' OR MAX(r.app_push_available_flag) = 'Y' THEN 'Y'
    ELSE 'N'
  END AS reachable_flag,
  CASE
    WHEN b.lead_type LIKE 'A1_%' AND COALESCE(MAX(cv.ftth_available_flag), 'UNKNOWN') = 'Y' THEN 'MARKET_FIBER_UPSELL'
    WHEN b.lead_type LIKE 'A1_%' THEN 'BUILD_OR_COVERAGE_CHECK'
    WHEN b.lead_type LIKE 'A0_%' THEN 'IDENTITY_MAPPING_REQUIRED'
    WHEN b.lead_type LIKE 'A2_%' THEN 'NETWORK_OPTIMIZATION_FIRST'
    WHEN b.lead_type LIKE 'B_%' THEN 'NURTURE_POOL'
    WHEN b.lead_type LIKE 'C_%' THEN 'FTTH_SPEED_UPSELL'
    ELSE 'OBSERVE'
  END AS final_action,
  b.recommended_offer
FROM base b
JOIN params p
LEFT JOIN crm c ON c.user_account = b.user_key OR c.user_mac = b.user_key OR c.crm_user_id = b.user_key
LEFT JOIN reach r ON r.crm_user_id = c.crm_user_id OR r.user_account = c.user_account
LEFT JOIN coverage cv ON cv.area_key = c.crm_user_id OR cv.area_key = c.user_account
GROUP BY p.analysis_run_id, b.user_key, c.crm_user_id, b.lead_type, b.demand_score, b.migration_motive_score, c.current_plan_name, c.current_arpu, b.recommended_offer;

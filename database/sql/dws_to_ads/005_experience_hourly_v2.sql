DELETE FROM :ads_app_hourly_experience_v2 WHERE analysis_run_id=:analysis_run_id;

INSERT INTO :ads_app_hourly_experience_v2 (
  analysis_run_id,import_batch_id,grain_hash,stat_date,hour_of_day,user_type,app_category,
  app_name,eligible_users,valid_obs_rows,poor_obs_rows,poor_observation_rate_pct,
  persistent_poor_users,severe_poor_users,sample_status
)
WITH policy AS (
  SELECT p.* FROM meta_analysis_run_policy_binding b JOIN meta_experience_analysis_policy p ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version WHERE b.analysis_run_id=:analysis_run_id AND b.import_batch_id=:import_batch_id
), rollup AS (
  SELECT analysis_run_id,import_batch_id,stat_date,hour_of_day,user_type,app_category,app_name,
    COUNT(*) eligible_users,SUM(valid_obs_rows) valid_obs_rows,SUM(poor_obs_rows) poor_obs_rows,
    SUM(persistent_poor_user_flag) persistent_poor_users,SUM(severe_poor_user_flag) severe_poor_users
  FROM :dws_user_app_hourly_experience_v2 WHERE analysis_run_id=:analysis_run_id
  GROUP BY analysis_run_id,import_batch_id,stat_date,hour_of_day,user_type,app_category,app_name
)
SELECT r.analysis_run_id,r.import_batch_id,UNHEX(MD5(CONCAT_WS('|',r.stat_date,r.hour_of_day,r.user_type,r.app_category,r.app_name))),r.stat_date,r.hour_of_day,r.user_type,r.app_category,r.app_name,r.eligible_users,r.valid_obs_rows,r.poor_obs_rows,ROUND(r.poor_obs_rows*100.0/NULLIF(r.valid_obs_rows,0),6),r.persistent_poor_users,r.severe_poor_users,
  CASE WHEN r.eligible_users>=p.minimum_app_eligible_users AND r.valid_obs_rows>=p.minimum_app_valid_obs THEN 'SUFFICIENT' ELSE 'INSUFFICIENT_SAMPLE' END
FROM rollup r CROSS JOIN policy p;

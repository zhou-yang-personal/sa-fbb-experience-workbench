-- V2 App experience ADS. No RAW/DWD access is allowed here.

DELETE FROM :ads_app_experience_v2
WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_app_experience_v2 (
  analysis_run_id, import_batch_id, grain_hash,
  user_type, app_category, app_name, data_type, profile_code, policy_version,
  observed_users, eligible_users, valid_obs_rows, poor_obs_rows,
  poor_observation_rate_pct, ever_affected_users, ever_affected_user_rate_pct,
  persistent_poor_users, persistent_poor_user_rate_pct,
  severe_poor_users, severe_poor_user_rate_pct,
  sample_status, attention_level, main_issue_driver,
  data_limitation_code, evidence_summary
)
WITH
params AS (
  SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id
),
policy_config AS (
  SELECT p.*
  FROM meta_analysis_run_policy_binding b
  JOIN meta_experience_analysis_policy p
    ON p.policy_id = b.experience_policy_id
   AND p.version = b.experience_policy_version
  JOIN params x
    ON x.analysis_run_id = b.analysis_run_id
   AND x.import_batch_id = b.import_batch_id
)
SELECT
  d.analysis_run_id,
  d.import_batch_id,
  d.grain_hash,
  d.user_type,
  d.app_category,
  d.app_name,
  d.data_type,
  d.profile_code,
  d.policy_version,
  d.observed_users,
  d.eligible_users,
  d.valid_obs_rows,
  d.poor_obs_rows,
  d.poor_observation_rate_pct,
  d.ever_affected_users,
  d.ever_affected_user_rate_pct,
  d.persistent_poor_users,
  d.persistent_poor_user_rate_pct,
  d.severe_poor_users,
  d.severe_poor_user_rate_pct,
  d.sample_status,
  CASE
    WHEN d.sample_status <> 'SUFFICIENT' THEN 'INSUFFICIENT_SAMPLE'
    WHEN d.severe_poor_user_rate_pct >= pc.finding_severe_user_rate_pct THEN 'SEVERE'
    WHEN d.persistent_poor_user_rate_pct >= pc.finding_attention_persistent_user_rate_pct THEN 'ATTENTION'
    ELSE 'HEALTHY'
  END,
  d.main_issue_driver,
  CASE WHEN d.sample_status <> 'SUFFICIENT' THEN 'MINIMUM_SAMPLE_NOT_MET' ELSE NULL END,
  CONCAT(
    'source=dws_app_access_period_experience_v2',
    '; observed_users=', d.observed_users,
    '; eligible_users=', d.eligible_users,
    '; valid_obs=', d.valid_obs_rows,
    '; poor_obs=', d.poor_obs_rows,
    '; ever_affected_users=', d.ever_affected_users,
    '; persistent_poor_users=', d.persistent_poor_users,
    '; severe_poor_users=', d.severe_poor_users,
    '; policy_version=', d.policy_version
  )
FROM :dws_app_access_period_experience_v2 d
JOIN params x
  ON x.analysis_run_id = d.analysis_run_id
 AND x.import_batch_id = d.import_batch_id
CROSS JOIN policy_config pc;

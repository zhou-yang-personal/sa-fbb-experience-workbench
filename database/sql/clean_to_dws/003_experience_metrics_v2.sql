-- Policy-bound V2 experience metrics.
-- Parameters: :import_batch_id, :analysis_run_id and batch-table placeholders.
-- This script reads DWD once per data type and never scans RAW.

DELETE FROM :dws_user_app_period_experience_v2
WHERE analysis_run_id = :analysis_run_id;

DELETE FROM :dws_app_access_period_experience_v2
WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :dws_user_app_period_experience_v2 (
  analysis_run_id, import_batch_id, grain_hash,
  user_key, user_type, app_category, app_name, data_type,
  profile_code, policy_version,
  observation_rows, valid_obs_rows, poor_obs_rows, severe_obs_rows,
  poor_observation_rate_pct, severe_observation_rate_pct,
  eligible_user_flag, ever_affected_user_flag,
  persistent_poor_user_flag, severe_poor_user_flag,
  poor_vmos_obs, poor_mos_obs, poor_subscriber_rtt_obs,
  poor_network_rtt_obs, poor_user_loss_obs, poor_network_loss_obs,
  poor_jitter_obs, total_download_gb, total_game_hours,
  avg_effective_download_mbps, avg_vmos, avg_mos,
  avg_subscriber_rtt_ms, avg_network_rtt_ms,
  avg_user_loss_pct, avg_network_loss_pct, avg_jitter_ms
)
WITH
params AS (
  SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id
),
policy_config AS (
  SELECT
    b.analysis_run_id,
    b.import_batch_id,
    b.access_rule_set_id,
    b.others_access_type,
    p.policy_id,
    p.version AS policy_version,
    p.persistent_min_valid_obs,
    p.persistent_min_poor_obs,
    p.persistent_min_poor_rate_pct,
    p.severe_user_min_valid_obs,
    p.severe_user_min_severe_obs,
    p.severe_user_min_severe_rate_pct
  FROM meta_analysis_run_policy_binding b
  JOIN meta_experience_analysis_policy p
    ON p.policy_id = b.experience_policy_id
   AND p.version = b.experience_policy_version
  JOIN params x
    ON x.analysis_run_id = b.analysis_run_id
   AND x.import_batch_id = b.import_batch_id
  WHERE p.status = 'published'
    AND b.others_access_type IN ('CABLE', 'FTTH', 'OTHER')
),
tcp_observations AS (
  SELECT
    pc.analysis_run_id,
    d.import_batch_id,
    d.user_key,
    CASE
      WHEN INET_ATON(d.local_ip_address) IS NULL THEN 'UNAVAILABLE'
      WHEN ar.rule_id IS NOT NULL THEN ar.access_type
      ELSE pc.others_access_type
    END AS user_type,
    COALESCE(NULLIF(d.app_category, ''), 'other') AS app_category,
    COALESCE(NULLIF(d.app_name, ''), 'UNKNOWN_APP') AS app_name,
    'tcp' AS data_type,
    ep.profile_code,
    pc.policy_version,
    COALESCE(d.downloaded_gb, 0) AS downloaded_gb,
    CAST(0 AS DECIMAL(24,6)) AS game_hours,
    d.effective_download_mbps,
    d.vmos,
    CAST(NULL AS DECIMAL(18,6)) AS mos,
    d.subscriber_side_rtt_ms,
    d.network_side_rtt_ms,
    d.user_down_loss,
    d.network_down_loss,
    CAST(NULL AS DECIMAL(18,6)) AS jitter_ms,
    CASE WHEN d.vmos IS NOT NULL OR d.subscriber_side_rtt_ms IS NOT NULL
                   OR d.network_side_rtt_ms IS NOT NULL OR d.user_down_loss IS NOT NULL
                   OR d.network_down_loss IS NOT NULL THEN 1 ELSE 0 END AS valid_flag,
    CASE WHEN ep.poor_vmos_below IS NOT NULL AND d.vmos < ep.poor_vmos_below THEN 1 ELSE 0 END AS poor_vmos_flag,
    0 AS poor_mos_flag,
    CASE WHEN ep.poor_subscriber_rtt_ms_at_least IS NOT NULL AND d.subscriber_side_rtt_ms >= ep.poor_subscriber_rtt_ms_at_least THEN 1 ELSE 0 END AS poor_subscriber_rtt_flag,
    CASE WHEN ep.poor_network_rtt_ms_at_least IS NOT NULL AND d.network_side_rtt_ms >= ep.poor_network_rtt_ms_at_least THEN 1 ELSE 0 END AS poor_network_rtt_flag,
    CASE WHEN ep.poor_user_loss_pct_at_least IS NOT NULL AND d.user_down_loss >= ep.poor_user_loss_pct_at_least THEN 1 ELSE 0 END AS poor_user_loss_flag,
    CASE WHEN ep.poor_network_loss_pct_at_least IS NOT NULL AND d.network_down_loss >= ep.poor_network_loss_pct_at_least THEN 1 ELSE 0 END AS poor_network_loss_flag,
    0 AS poor_jitter_flag,
    CASE WHEN
      (ep.severe_vmos_below IS NOT NULL AND d.vmos < ep.severe_vmos_below)
      OR (ep.severe_subscriber_rtt_ms_at_least IS NOT NULL AND d.subscriber_side_rtt_ms >= ep.severe_subscriber_rtt_ms_at_least)
      OR (ep.severe_network_rtt_ms_at_least IS NOT NULL AND d.network_side_rtt_ms >= ep.severe_network_rtt_ms_at_least)
      OR (ep.severe_user_loss_pct_at_least IS NOT NULL AND d.user_down_loss >= ep.severe_user_loss_pct_at_least)
      OR (ep.severe_network_loss_pct_at_least IS NOT NULL AND d.network_down_loss >= ep.severe_network_loss_pct_at_least)
      THEN 1 ELSE 0 END AS severe_flag
  FROM :dwd_tcp_detail_clean d
  JOIN policy_config pc ON pc.import_batch_id = d.import_batch_id
  JOIN dim_app_experience_profile ep
    ON ep.profile_id = (
      SELECT ep2.profile_id
      FROM dim_app_experience_profile ep2
      WHERE ep2.policy_id = pc.policy_id
        AND ep2.data_type = 'tcp'
        AND ep2.enabled = 1
        AND (ep2.app_category IS NULL OR LOWER(ep2.app_category) = LOWER(COALESCE(NULLIF(d.app_category, ''), 'other')))
      ORDER BY CASE WHEN ep2.app_category IS NULL THEN 1 ELSE 0 END, ep2.priority, ep2.profile_code
      LIMIT 1
    )
  LEFT JOIN dim_access_ip_range ar
    ON ar.rule_set_id = pc.access_rule_set_id
   AND ar.enabled = 1
   AND INET_ATON(d.local_ip_address) BETWEEN ar.start_ip_num AND ar.end_ip_num
  WHERE d.import_batch_id = (SELECT import_batch_id FROM params)
    AND d.user_key IS NOT NULL
    AND TRIM(d.user_key) <> ''
    AND d.user_key <> 'UNKNOWN'
),
game_observations AS (
  SELECT
    pc.analysis_run_id,
    d.import_batch_id,
    d.user_key,
    CASE
      WHEN INET_ATON(d.local_ip_address) IS NULL THEN 'UNAVAILABLE'
      WHEN ar.rule_id IS NOT NULL THEN ar.access_type
      ELSE pc.others_access_type
    END AS user_type,
    COALESCE(NULLIF(d.app_category, ''), 'game') AS app_category,
    COALESCE(NULLIF(d.app_name, ''), 'UNKNOWN_APP') AS app_name,
    'game' AS data_type,
    ep.profile_code,
    pc.policy_version,
    CAST(0 AS DECIMAL(24,6)) AS downloaded_gb,
    COALESCE(d.game_hours, 0) AS game_hours,
    CAST(NULL AS DECIMAL(18,6)) AS effective_download_mbps,
    CAST(NULL AS DECIMAL(18,6)) AS vmos,
    d.mos,
    d.worst_latency_ms AS subscriber_side_rtt_ms,
    CAST(NULL AS DECIMAL(18,6)) AS network_side_rtt_ms,
    d.worst_loss AS user_down_loss,
    CAST(NULL AS DECIMAL(18,6)) AS network_down_loss,
    d.worst_jitter_ms AS jitter_ms,
    CASE WHEN d.mos IS NOT NULL OR d.worst_latency_ms IS NOT NULL
                   OR d.worst_loss IS NOT NULL OR d.worst_jitter_ms IS NOT NULL THEN 1 ELSE 0 END AS valid_flag,
    0 AS poor_vmos_flag,
    CASE WHEN ep.poor_mos_below IS NOT NULL AND d.mos < ep.poor_mos_below THEN 1 ELSE 0 END AS poor_mos_flag,
    CASE WHEN ep.poor_subscriber_rtt_ms_at_least IS NOT NULL AND d.worst_latency_ms >= ep.poor_subscriber_rtt_ms_at_least THEN 1 ELSE 0 END AS poor_subscriber_rtt_flag,
    0 AS poor_network_rtt_flag,
    CASE WHEN ep.poor_user_loss_pct_at_least IS NOT NULL AND d.worst_loss >= ep.poor_user_loss_pct_at_least THEN 1 ELSE 0 END AS poor_user_loss_flag,
    0 AS poor_network_loss_flag,
    CASE WHEN ep.poor_jitter_ms_at_least IS NOT NULL AND d.worst_jitter_ms >= ep.poor_jitter_ms_at_least THEN 1 ELSE 0 END AS poor_jitter_flag,
    CASE WHEN
      (ep.severe_mos_below IS NOT NULL AND d.mos < ep.severe_mos_below)
      OR (ep.severe_subscriber_rtt_ms_at_least IS NOT NULL AND d.worst_latency_ms >= ep.severe_subscriber_rtt_ms_at_least)
      OR (ep.severe_user_loss_pct_at_least IS NOT NULL AND d.worst_loss >= ep.severe_user_loss_pct_at_least)
      OR (ep.severe_jitter_ms_at_least IS NOT NULL AND d.worst_jitter_ms >= ep.severe_jitter_ms_at_least)
      THEN 1 ELSE 0 END AS severe_flag
  FROM :dwd_game_detail_clean d
  JOIN policy_config pc ON pc.import_batch_id = d.import_batch_id
  JOIN dim_app_experience_profile ep
    ON ep.profile_id = (
      SELECT ep2.profile_id
      FROM dim_app_experience_profile ep2
      WHERE ep2.policy_id = pc.policy_id
        AND ep2.data_type = 'game'
        AND ep2.enabled = 1
        AND (ep2.app_category IS NULL OR LOWER(ep2.app_category) = LOWER(COALESCE(NULLIF(d.app_category, ''), 'game')))
      ORDER BY CASE WHEN ep2.app_category IS NULL THEN 1 ELSE 0 END, ep2.priority, ep2.profile_code
      LIMIT 1
    )
  LEFT JOIN dim_access_ip_range ar
    ON ar.rule_set_id = pc.access_rule_set_id
   AND ar.enabled = 1
   AND INET_ATON(d.local_ip_address) BETWEEN ar.start_ip_num AND ar.end_ip_num
  WHERE d.import_batch_id = (SELECT import_batch_id FROM params)
    AND d.user_key IS NOT NULL
    AND TRIM(d.user_key) <> ''
    AND d.user_key <> 'UNKNOWN'
),
observations AS (
  SELECT * FROM tcp_observations
  UNION ALL
  SELECT * FROM game_observations
),
flagged AS (
  SELECT
    o.*,
    CASE WHEN o.valid_flag = 1 AND (
      o.poor_vmos_flag = 1 OR o.poor_mos_flag = 1
      OR o.poor_subscriber_rtt_flag = 1 OR o.poor_network_rtt_flag = 1
      OR o.poor_user_loss_flag = 1 OR o.poor_network_loss_flag = 1
      OR o.poor_jitter_flag = 1
    ) THEN 1 ELSE 0 END AS poor_flag
  FROM observations o
),
user_app AS (
  SELECT
    f.analysis_run_id,
    f.import_batch_id,
    f.user_key,
    f.user_type,
    f.app_category,
    f.app_name,
    f.data_type,
    f.profile_code,
    f.policy_version,
    COUNT(*) AS observation_rows,
    SUM(f.valid_flag) AS valid_obs_rows,
    SUM(f.poor_flag) AS poor_obs_rows,
    SUM(f.severe_flag) AS severe_obs_rows,
    SUM(f.poor_vmos_flag) AS poor_vmos_obs,
    SUM(f.poor_mos_flag) AS poor_mos_obs,
    SUM(f.poor_subscriber_rtt_flag) AS poor_subscriber_rtt_obs,
    SUM(f.poor_network_rtt_flag) AS poor_network_rtt_obs,
    SUM(f.poor_user_loss_flag) AS poor_user_loss_obs,
    SUM(f.poor_network_loss_flag) AS poor_network_loss_obs,
    SUM(f.poor_jitter_flag) AS poor_jitter_obs,
    SUM(f.downloaded_gb) AS total_download_gb,
    SUM(f.game_hours) AS total_game_hours,
    AVG(f.effective_download_mbps) AS avg_effective_download_mbps,
    AVG(f.vmos) AS avg_vmos,
    AVG(f.mos) AS avg_mos,
    AVG(f.subscriber_side_rtt_ms) AS avg_subscriber_rtt_ms,
    AVG(f.network_side_rtt_ms) AS avg_network_rtt_ms,
    AVG(f.user_down_loss) AS avg_user_loss_pct,
    AVG(f.network_down_loss) AS avg_network_loss_pct,
    AVG(f.jitter_ms) AS avg_jitter_ms
  FROM flagged f
  GROUP BY
    f.analysis_run_id, f.import_batch_id, f.user_key, f.user_type,
    f.app_category, f.app_name, f.data_type, f.profile_code, f.policy_version
)
SELECT
  u.analysis_run_id,
  u.import_batch_id,
  UNHEX(MD5(CONCAT_WS('|', u.user_key, u.user_type, u.app_category, u.app_name, u.data_type))),
  u.user_key,
  u.user_type,
  u.app_category,
  u.app_name,
  u.data_type,
  u.profile_code,
  u.policy_version,
  u.observation_rows,
  u.valid_obs_rows,
  u.poor_obs_rows,
  u.severe_obs_rows,
  ROUND(u.poor_obs_rows * 100.0 / NULLIF(u.valid_obs_rows, 0), 6),
  ROUND(u.severe_obs_rows * 100.0 / NULLIF(u.valid_obs_rows, 0), 6),
  CASE WHEN u.valid_obs_rows >= pc.persistent_min_valid_obs THEN 1 ELSE 0 END,
  CASE WHEN u.valid_obs_rows >= pc.persistent_min_valid_obs AND u.poor_obs_rows > 0 THEN 1 ELSE 0 END,
  CASE WHEN u.valid_obs_rows >= pc.persistent_min_valid_obs
             AND u.poor_obs_rows >= pc.persistent_min_poor_obs
             AND u.poor_obs_rows * 100.0 / NULLIF(u.valid_obs_rows, 0) >= pc.persistent_min_poor_rate_pct
       THEN 1 ELSE 0 END,
  CASE WHEN u.valid_obs_rows >= pc.severe_user_min_valid_obs
             AND u.severe_obs_rows >= pc.severe_user_min_severe_obs
             AND u.severe_obs_rows * 100.0 / NULLIF(u.valid_obs_rows, 0) >= pc.severe_user_min_severe_rate_pct
       THEN 1 ELSE 0 END,
  u.poor_vmos_obs,
  u.poor_mos_obs,
  u.poor_subscriber_rtt_obs,
  u.poor_network_rtt_obs,
  u.poor_user_loss_obs,
  u.poor_network_loss_obs,
  u.poor_jitter_obs,
  u.total_download_gb,
  u.total_game_hours,
  u.avg_effective_download_mbps,
  u.avg_vmos,
  u.avg_mos,
  u.avg_subscriber_rtt_ms,
  u.avg_network_rtt_ms,
  u.avg_user_loss_pct,
  u.avg_network_loss_pct,
  u.avg_jitter_ms
FROM user_app u
JOIN policy_config pc ON pc.analysis_run_id = u.analysis_run_id;

INSERT INTO :dws_app_access_period_experience_v2 (
  analysis_run_id, import_batch_id, grain_hash,
  user_type, app_category, app_name, data_type, profile_code, policy_version,
  observed_users, eligible_users, observation_rows, valid_obs_rows,
  poor_obs_rows, severe_obs_rows, poor_observation_rate_pct,
  ever_affected_users, ever_affected_user_rate_pct,
  persistent_poor_users, persistent_poor_user_rate_pct,
  severe_poor_users, severe_poor_user_rate_pct,
  sample_status, main_issue_driver,
  total_download_gb, total_game_hours, avg_effective_download_mbps,
  avg_vmos, avg_mos, avg_subscriber_rtt_ms, avg_network_rtt_ms,
  avg_user_loss_pct, avg_network_loss_pct, avg_jitter_ms
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
  JOIN params x ON x.analysis_run_id = b.analysis_run_id
  WHERE b.import_batch_id = x.import_batch_id
),
app_rollup AS (
  SELECT
    u.analysis_run_id,
    u.import_batch_id,
    u.user_type,
    u.app_category,
    u.app_name,
    u.data_type,
    u.profile_code,
    u.policy_version,
    COUNT(*) AS observed_users,
    SUM(u.eligible_user_flag) AS eligible_users,
    SUM(u.observation_rows) AS observation_rows,
    SUM(u.valid_obs_rows) AS valid_obs_rows,
    SUM(u.poor_obs_rows) AS poor_obs_rows,
    SUM(u.severe_obs_rows) AS severe_obs_rows,
    SUM(u.ever_affected_user_flag) AS ever_affected_users,
    SUM(u.persistent_poor_user_flag) AS persistent_poor_users,
    SUM(u.severe_poor_user_flag) AS severe_poor_users,
    SUM(u.poor_vmos_obs) AS poor_vmos_obs,
    SUM(u.poor_mos_obs) AS poor_mos_obs,
    SUM(u.poor_subscriber_rtt_obs) AS poor_subscriber_rtt_obs,
    SUM(u.poor_network_rtt_obs) AS poor_network_rtt_obs,
    SUM(u.poor_user_loss_obs) AS poor_user_loss_obs,
    SUM(u.poor_network_loss_obs) AS poor_network_loss_obs,
    SUM(u.poor_jitter_obs) AS poor_jitter_obs,
    SUM(u.total_download_gb) AS total_download_gb,
    SUM(u.total_game_hours) AS total_game_hours,
    AVG(u.avg_effective_download_mbps) AS avg_effective_download_mbps,
    AVG(u.avg_vmos) AS avg_vmos,
    AVG(u.avg_mos) AS avg_mos,
    AVG(u.avg_subscriber_rtt_ms) AS avg_subscriber_rtt_ms,
    AVG(u.avg_network_rtt_ms) AS avg_network_rtt_ms,
    AVG(u.avg_user_loss_pct) AS avg_user_loss_pct,
    AVG(u.avg_network_loss_pct) AS avg_network_loss_pct,
    AVG(u.avg_jitter_ms) AS avg_jitter_ms
  FROM :dws_user_app_period_experience_v2 u
  JOIN params x ON x.analysis_run_id = u.analysis_run_id
  GROUP BY
    u.analysis_run_id, u.import_batch_id, u.user_type, u.app_category,
    u.app_name, u.data_type, u.profile_code, u.policy_version
)
SELECT
  a.analysis_run_id,
  a.import_batch_id,
  UNHEX(MD5(CONCAT_WS('|', a.user_type, a.app_category, a.app_name, a.data_type))),
  a.user_type,
  a.app_category,
  a.app_name,
  a.data_type,
  a.profile_code,
  a.policy_version,
  a.observed_users,
  a.eligible_users,
  a.observation_rows,
  a.valid_obs_rows,
  a.poor_obs_rows,
  a.severe_obs_rows,
  ROUND(a.poor_obs_rows * 100.0 / NULLIF(a.valid_obs_rows, 0), 6),
  a.ever_affected_users,
  ROUND(a.ever_affected_users * 100.0 / NULLIF(a.eligible_users, 0), 6),
  a.persistent_poor_users,
  ROUND(a.persistent_poor_users * 100.0 / NULLIF(a.eligible_users, 0), 6),
  a.severe_poor_users,
  ROUND(a.severe_poor_users * 100.0 / NULLIF(a.eligible_users, 0), 6),
  CASE WHEN a.eligible_users >= pc.minimum_app_eligible_users
             AND a.valid_obs_rows >= pc.minimum_app_valid_obs
       THEN 'SUFFICIENT' ELSE 'INSUFFICIENT_SAMPLE' END,
  CASE
    WHEN a.poor_network_rtt_obs + a.poor_network_loss_obs >= GREATEST(
      a.poor_vmos_obs + a.poor_mos_obs,
      a.poor_subscriber_rtt_obs + a.poor_user_loss_obs + a.poor_jitter_obs
    ) AND a.poor_network_rtt_obs + a.poor_network_loss_obs > 0 THEN 'NETWORK_SIDE'
    WHEN a.poor_subscriber_rtt_obs + a.poor_user_loss_obs + a.poor_jitter_obs >= GREATEST(
      a.poor_vmos_obs + a.poor_mos_obs,
      a.poor_network_rtt_obs + a.poor_network_loss_obs
    ) AND a.poor_subscriber_rtt_obs + a.poor_user_loss_obs + a.poor_jitter_obs > 0 THEN 'USER_SIDE'
    WHEN a.poor_vmos_obs > 0 THEN 'LOW_VMOS'
    WHEN a.poor_mos_obs > 0 THEN 'LOW_MOS'
    ELSE 'NO_DOMINANT_DRIVER'
  END,
  a.total_download_gb,
  a.total_game_hours,
  a.avg_effective_download_mbps,
  a.avg_vmos,
  a.avg_mos,
  a.avg_subscriber_rtt_ms,
  a.avg_network_rtt_ms,
  a.avg_user_loss_pct,
  a.avg_network_loss_pct,
  a.avg_jitter_ms
FROM app_rollup a
CROSS JOIN policy_config pc;

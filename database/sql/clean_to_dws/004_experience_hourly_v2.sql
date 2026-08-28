-- User × App × hour reusable experience grain. One date/hour per transaction; reads DWD, never RAW.
/* hourly_v2 partition=:partition_date/:partition_hour delete */
DELETE FROM :dws_user_app_hourly_experience_v2
WHERE analysis_run_id=:analysis_run_id
  AND stat_date=:partition_date
  AND hour_of_day=:partition_hour;

/* hourly_v2 partition=:partition_date/:partition_hour insert */
INSERT INTO :dws_user_app_hourly_experience_v2 (
  analysis_run_id,import_batch_id,grain_hash,stat_date,hour_of_day,user_key,user_type,
  app_category,app_name,data_type,profile_code,policy_version,valid_obs_rows,poor_obs_rows,
  severe_obs_rows,poor_observation_rate_pct,persistent_poor_user_flag,severe_poor_user_flag,
  observation_rows,total_download_gb,total_effective_duration_hours,total_video_duration_hours,
  avg_effective_download_mbps,avg_download_mbps,
  avg_vmos,avg_mos,avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,
  avg_network_loss_pct,avg_jitter_ms
)
WITH policy AS (
  SELECT b.analysis_run_id,b.import_batch_id,p.*
  FROM meta_analysis_run_policy_binding b
  JOIN meta_experience_analysis_policy p ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version
  WHERE b.analysis_run_id=:analysis_run_id AND b.import_batch_id=:import_batch_id
), tcp AS (
  SELECT p.analysis_run_id,d.import_batch_id,d.stat_date,d.hour_of_day,d.user_key,
    COALESCE(NULLIF(d.user_type,''),'UNAVAILABLE') user_type,
    COALESCE(NULLIF(d.app_category,''),'other') app_category,COALESCE(NULLIF(d.app_name,''),'UNKNOWN_APP') app_name,
    'tcp' data_type,ep.profile_code,p.version policy_version,
    COALESCE(d.downloaded_gb,0) downloaded_gb,
    COALESCE(d.effective_duration_hours,0) effective_duration_hours,
    COALESCE(d.video_duration_hours,0) video_duration_hours,
    d.effective_download_mbps,d.avg_download_mbps,
    CASE WHEN d.vmos IS NOT NULL OR d.subscriber_side_rtt_ms IS NOT NULL OR d.network_side_rtt_ms IS NOT NULL OR d.user_down_loss IS NOT NULL OR d.network_down_loss IS NOT NULL THEN 1 ELSE 0 END valid_flag,
    CASE WHEN (ep.poor_vmos_below IS NOT NULL AND d.vmos<ep.poor_vmos_below) OR (ep.poor_subscriber_rtt_ms_at_least IS NOT NULL AND d.subscriber_side_rtt_ms>=ep.poor_subscriber_rtt_ms_at_least) OR (ep.poor_network_rtt_ms_at_least IS NOT NULL AND d.network_side_rtt_ms>=ep.poor_network_rtt_ms_at_least) OR (ep.poor_user_loss_pct_at_least IS NOT NULL AND d.user_down_loss>=ep.poor_user_loss_pct_at_least) OR (ep.poor_network_loss_pct_at_least IS NOT NULL AND d.network_down_loss>=ep.poor_network_loss_pct_at_least) THEN 1 ELSE 0 END poor_flag,
    CASE WHEN (ep.severe_vmos_below IS NOT NULL AND d.vmos<ep.severe_vmos_below) OR (ep.severe_subscriber_rtt_ms_at_least IS NOT NULL AND d.subscriber_side_rtt_ms>=ep.severe_subscriber_rtt_ms_at_least) OR (ep.severe_network_rtt_ms_at_least IS NOT NULL AND d.network_side_rtt_ms>=ep.severe_network_rtt_ms_at_least) OR (ep.severe_user_loss_pct_at_least IS NOT NULL AND d.user_down_loss>=ep.severe_user_loss_pct_at_least) OR (ep.severe_network_loss_pct_at_least IS NOT NULL AND d.network_down_loss>=ep.severe_network_loss_pct_at_least) THEN 1 ELSE 0 END severe_flag,
    d.vmos,NULL mos,d.subscriber_side_rtt_ms,d.network_side_rtt_ms,d.user_down_loss user_loss,d.network_down_loss network_loss,NULL jitter
  FROM :dwd_tcp_detail_clean d JOIN policy p ON p.import_batch_id=d.import_batch_id
  JOIN dim_app_experience_profile ep ON ep.profile_id=(
    SELECT ep2.profile_id FROM dim_app_experience_profile ep2
    WHERE ep2.policy_id=p.policy_id AND ep2.data_type='tcp' AND ep2.enabled=1
      AND (ep2.app_category IS NULL OR LOWER(ep2.app_category)=LOWER(COALESCE(NULLIF(d.app_category,''),'other')))
    ORDER BY CASE WHEN ep2.app_category IS NULL THEN 1 ELSE 0 END,ep2.priority,ep2.profile_code LIMIT 1)
  WHERE d.import_batch_id=:import_batch_id
    AND d.stat_date=:partition_date
    AND d.hour_of_day=:partition_hour
    AND d.user_key IS NOT NULL AND d.user_key<>'UNKNOWN'
), game AS (
  SELECT p.analysis_run_id,d.import_batch_id,d.stat_date,d.hour_of_day,d.user_key,
    COALESCE(NULLIF(d.user_type,''),'UNAVAILABLE') user_type,
    COALESCE(NULLIF(d.app_category,''),'game') app_category,COALESCE(NULLIF(d.app_name,''),'UNKNOWN_APP') app_name,
    'game' data_type,ep.profile_code,p.version policy_version,
    CAST(0 AS DECIMAL(24,6)) downloaded_gb,CAST(0 AS DECIMAL(24,6)) effective_duration_hours,
    CAST(0 AS DECIMAL(24,6)) video_duration_hours,CAST(NULL AS DECIMAL(18,6)) effective_download_mbps,
    CAST(NULL AS DECIMAL(18,6)) avg_download_mbps,
    CASE WHEN d.mos IS NOT NULL OR d.worst_latency_ms IS NOT NULL OR d.worst_loss IS NOT NULL OR d.worst_jitter_ms IS NOT NULL THEN 1 ELSE 0 END valid_flag,
    CASE WHEN (ep.poor_mos_below IS NOT NULL AND d.mos<ep.poor_mos_below) OR (ep.poor_subscriber_rtt_ms_at_least IS NOT NULL AND d.worst_latency_ms>=ep.poor_subscriber_rtt_ms_at_least) OR (ep.poor_user_loss_pct_at_least IS NOT NULL AND d.worst_loss>=ep.poor_user_loss_pct_at_least) OR (ep.poor_jitter_ms_at_least IS NOT NULL AND d.worst_jitter_ms>=ep.poor_jitter_ms_at_least) THEN 1 ELSE 0 END poor_flag,
    CASE WHEN (ep.severe_mos_below IS NOT NULL AND d.mos<ep.severe_mos_below) OR (ep.severe_subscriber_rtt_ms_at_least IS NOT NULL AND d.worst_latency_ms>=ep.severe_subscriber_rtt_ms_at_least) OR (ep.severe_user_loss_pct_at_least IS NOT NULL AND d.worst_loss>=ep.severe_user_loss_pct_at_least) OR (ep.severe_jitter_ms_at_least IS NOT NULL AND d.worst_jitter_ms>=ep.severe_jitter_ms_at_least) THEN 1 ELSE 0 END severe_flag,
    NULL vmos,d.mos,d.worst_latency_ms subscriber_side_rtt_ms,NULL network_side_rtt_ms,d.worst_loss user_loss,NULL network_loss,d.worst_jitter_ms jitter
  FROM :dwd_game_detail_clean d JOIN policy p ON p.import_batch_id=d.import_batch_id
  JOIN dim_app_experience_profile ep ON ep.profile_id=(
    SELECT ep2.profile_id FROM dim_app_experience_profile ep2
    WHERE ep2.policy_id=p.policy_id AND ep2.data_type='game' AND ep2.enabled=1
      AND (ep2.app_category IS NULL OR LOWER(ep2.app_category)=LOWER(COALESCE(NULLIF(d.app_category,''),'game')))
    ORDER BY CASE WHEN ep2.app_category IS NULL THEN 1 ELSE 0 END,ep2.priority,ep2.profile_code LIMIT 1)
  WHERE d.import_batch_id=:import_batch_id
    AND d.stat_date=:partition_date
    AND d.hour_of_day=:partition_hour
    AND d.user_key IS NOT NULL AND d.user_key<>'UNKNOWN'
), hourly AS (
  SELECT o.analysis_run_id,o.import_batch_id,o.stat_date,o.hour_of_day,o.user_key,o.user_type,o.app_category,o.app_name,o.data_type,o.profile_code,o.policy_version,
    COUNT(*) observation_rows,SUM(o.downloaded_gb) total_download_gb,SUM(o.effective_duration_hours) total_effective_duration_hours,SUM(o.video_duration_hours) total_video_duration_hours,
    AVG(o.effective_download_mbps) avg_effective_download_mbps,AVG(o.avg_download_mbps) avg_download_mbps,
    SUM(o.valid_flag) valid_obs_rows,SUM(CASE WHEN o.valid_flag=1 THEN o.poor_flag ELSE 0 END) poor_obs_rows,SUM(CASE WHEN o.valid_flag=1 THEN o.severe_flag ELSE 0 END) severe_obs_rows,
    AVG(o.vmos) avg_vmos,AVG(o.mos) avg_mos,AVG(o.subscriber_side_rtt_ms) avg_subscriber_rtt_ms,AVG(o.network_side_rtt_ms) avg_network_rtt_ms,AVG(o.user_loss) avg_user_loss_pct,AVG(o.network_loss) avg_network_loss_pct,AVG(o.jitter) avg_jitter_ms
  FROM (SELECT * FROM tcp UNION ALL SELECT * FROM game) o
  GROUP BY o.analysis_run_id,o.import_batch_id,o.stat_date,o.hour_of_day,o.user_key,o.user_type,o.app_category,o.app_name,o.data_type,o.profile_code,o.policy_version
)
SELECT h.analysis_run_id,h.import_batch_id,UNHEX(MD5(CONCAT_WS('|',h.stat_date,h.hour_of_day,h.user_key,h.user_type,h.app_category,h.app_name,h.data_type))),h.stat_date,h.hour_of_day,h.user_key,h.user_type,h.app_category,h.app_name,h.data_type,h.profile_code,h.policy_version,
  h.valid_obs_rows,h.poor_obs_rows,h.severe_obs_rows,ROUND(h.poor_obs_rows*100.0/NULLIF(h.valid_obs_rows,0),6),
  CASE WHEN h.valid_obs_rows>=p.persistent_min_valid_obs AND h.poor_obs_rows>=p.persistent_min_poor_obs AND h.poor_obs_rows*100.0/NULLIF(h.valid_obs_rows,0)>=p.persistent_min_poor_rate_pct THEN 1 ELSE 0 END,
  CASE WHEN h.valid_obs_rows>=p.severe_user_min_valid_obs AND h.severe_obs_rows>=p.severe_user_min_severe_obs AND h.severe_obs_rows*100.0/NULLIF(h.valid_obs_rows,0)>=p.severe_user_min_severe_rate_pct THEN 1 ELSE 0 END,
  h.observation_rows,h.total_download_gb,h.total_effective_duration_hours,h.total_video_duration_hours,
  h.avg_effective_download_mbps,h.avg_download_mbps,
  h.avg_vmos,h.avg_mos,h.avg_subscriber_rtt_ms,h.avg_network_rtt_ms,h.avg_user_loss_pct,h.avg_network_loss_pct,h.avg_jitter_ms
FROM hourly h JOIN policy p ON p.analysis_run_id=h.analysis_run_id;

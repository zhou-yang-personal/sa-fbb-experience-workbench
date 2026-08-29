-- Period and App/access DWS derived only from the shared hourly core.
-- No DWD/RAW scan is allowed here. Weighted averages use the core SUM/COUNT state.

DELETE FROM :dws_user_app_period_experience_v2 WHERE analysis_run_id=:analysis_run_id;
DELETE FROM :dws_app_access_period_experience_v2 WHERE analysis_run_id=:analysis_run_id;

INSERT INTO :dws_user_app_period_experience_v2 (
  analysis_run_id,import_batch_id,grain_hash,user_key,user_type,app_category,app_name,data_type,
  profile_code,policy_version,observation_rows,valid_obs_rows,poor_obs_rows,severe_obs_rows,
  poor_observation_rate_pct,severe_observation_rate_pct,eligible_user_flag,ever_affected_user_flag,
  persistent_poor_user_flag,severe_poor_user_flag,
  poor_vmos_obs,poor_mos_obs,poor_subscriber_rtt_obs,poor_network_rtt_obs,
  poor_user_loss_obs,poor_network_loss_obs,poor_jitter_obs,
  total_download_gb,total_game_hours,total_effective_duration_hours,total_video_duration_hours,active_days,
  avg_effective_download_mbps,avg_vmos,avg_mos,avg_download_mbps,avg_throughput_mbps,
  avg_max_single_flow_mbps,avg_connection_success_pct,avg_connection_delay_ms,avg_download_fluency,
  avg_upstream_rtt_ms,avg_downstream_rtt_ms,avg_user_up_loss_pct,avg_network_up_loss_pct,avg_wifi_delay_ms,
  avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,avg_network_loss_pct,avg_jitter_ms,
  effective_download_mbps_sum,effective_download_mbps_count,vmos_sum,vmos_count,mos_sum,mos_count,
  subscriber_rtt_sum,subscriber_rtt_count,network_rtt_sum,network_rtt_count,
  user_loss_sum,user_loss_count,network_loss_sum,network_loss_count,jitter_sum,jitter_count
)
WITH policy AS (
  SELECT b.analysis_run_id,b.import_batch_id,p.*
  FROM meta_analysis_run_policy_binding b
  JOIN meta_experience_analysis_policy p
    ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version
  WHERE b.analysis_run_id=:analysis_run_id AND b.import_batch_id=:import_batch_id
), user_app AS (
  SELECT h.analysis_run_id,h.import_batch_id,h.user_key,h.user_type,h.app_category,h.app_name,
    h.data_type,h.profile_code,h.policy_version,
    SUM(h.observation_rows) observation_rows,SUM(h.valid_obs_rows) valid_obs_rows,
    SUM(h.poor_obs_rows) poor_obs_rows,SUM(h.severe_obs_rows) severe_obs_rows,
    SUM(h.poor_vmos_obs) poor_vmos_obs,SUM(h.poor_mos_obs) poor_mos_obs,
    SUM(h.poor_subscriber_rtt_obs) poor_subscriber_rtt_obs,
    SUM(h.poor_network_rtt_obs) poor_network_rtt_obs,
    SUM(h.poor_user_loss_obs) poor_user_loss_obs,SUM(h.poor_network_loss_obs) poor_network_loss_obs,
    SUM(h.poor_jitter_obs) poor_jitter_obs,
    SUM(h.total_download_gb) total_download_gb,SUM(h.total_game_hours) total_game_hours,
    SUM(h.total_effective_duration_hours) total_effective_duration_hours,
    SUM(h.total_video_duration_hours) total_video_duration_hours,
    COUNT(DISTINCT h.stat_date) active_days,
    SUM(h.effective_download_mbps_sum) effective_download_mbps_sum,SUM(h.effective_download_mbps_count) effective_download_mbps_count,
    SUM(h.avg_download_mbps_sum) avg_download_mbps_sum,SUM(h.avg_download_mbps_count) avg_download_mbps_count,
    SUM(h.throughput_mbps_sum) throughput_mbps_sum,SUM(h.throughput_mbps_count) throughput_mbps_count,
    SUM(h.max_single_flow_mbps_sum) max_single_flow_mbps_sum,SUM(h.max_single_flow_mbps_count) max_single_flow_mbps_count,
    SUM(h.connection_success_sum) connection_success_sum,SUM(h.connection_success_count) connection_success_count,
    SUM(h.connection_delay_sum) connection_delay_sum,SUM(h.connection_delay_count) connection_delay_count,
    SUM(h.download_fluency_sum) download_fluency_sum,SUM(h.download_fluency_count) download_fluency_count,
    SUM(h.upstream_rtt_sum) upstream_rtt_sum,SUM(h.upstream_rtt_count) upstream_rtt_count,
    SUM(h.downstream_rtt_sum) downstream_rtt_sum,SUM(h.downstream_rtt_count) downstream_rtt_count,
    SUM(h.user_up_loss_sum) user_up_loss_sum,SUM(h.user_up_loss_count) user_up_loss_count,
    SUM(h.network_up_loss_sum) network_up_loss_sum,SUM(h.network_up_loss_count) network_up_loss_count,
    SUM(h.wifi_delay_sum) wifi_delay_sum,SUM(h.wifi_delay_count) wifi_delay_count,
    SUM(h.vmos_sum) vmos_sum,SUM(h.vmos_count) vmos_count,SUM(h.mos_sum) mos_sum,SUM(h.mos_count) mos_count,
    SUM(h.subscriber_rtt_sum) subscriber_rtt_sum,SUM(h.subscriber_rtt_count) subscriber_rtt_count,
    SUM(h.network_rtt_sum) network_rtt_sum,SUM(h.network_rtt_count) network_rtt_count,
    SUM(h.user_loss_sum) user_loss_sum,SUM(h.user_loss_count) user_loss_count,
    SUM(h.network_loss_sum) network_loss_sum,SUM(h.network_loss_count) network_loss_count,
    SUM(h.jitter_sum) jitter_sum,SUM(h.jitter_count) jitter_count
  FROM :dws_user_app_hourly_experience_v2 h
  WHERE h.analysis_run_id=:analysis_run_id AND h.import_batch_id=:import_batch_id
  GROUP BY h.analysis_run_id,h.import_batch_id,h.user_key,h.user_type,h.app_category,h.app_name,
    h.data_type,h.profile_code,h.policy_version
)
SELECT u.analysis_run_id,u.import_batch_id,
  UNHEX(MD5(CONCAT_WS('|',u.user_key,u.user_type,u.app_category,u.app_name,u.data_type))),
  u.user_key,u.user_type,u.app_category,u.app_name,u.data_type,u.profile_code,u.policy_version,
  u.observation_rows,u.valid_obs_rows,u.poor_obs_rows,u.severe_obs_rows,
  ROUND(u.poor_obs_rows*100.0/NULLIF(u.valid_obs_rows,0),6),
  ROUND(u.severe_obs_rows*100.0/NULLIF(u.valid_obs_rows,0),6),
  CASE WHEN u.valid_obs_rows>=p.persistent_min_valid_obs THEN 1 ELSE 0 END,
  CASE WHEN u.valid_obs_rows>=p.persistent_min_valid_obs AND u.poor_obs_rows>0 THEN 1 ELSE 0 END,
  CASE WHEN u.valid_obs_rows>=p.persistent_min_valid_obs AND u.poor_obs_rows>=p.persistent_min_poor_obs
    AND u.poor_obs_rows*100.0/NULLIF(u.valid_obs_rows,0)>=p.persistent_min_poor_rate_pct THEN 1 ELSE 0 END,
  CASE WHEN u.valid_obs_rows>=p.severe_user_min_valid_obs AND u.severe_obs_rows>=p.severe_user_min_severe_obs
    AND u.severe_obs_rows*100.0/NULLIF(u.valid_obs_rows,0)>=p.severe_user_min_severe_rate_pct THEN 1 ELSE 0 END,
  u.poor_vmos_obs,u.poor_mos_obs,u.poor_subscriber_rtt_obs,u.poor_network_rtt_obs,
  u.poor_user_loss_obs,u.poor_network_loss_obs,u.poor_jitter_obs,
  u.total_download_gb,u.total_game_hours,u.total_effective_duration_hours,u.total_video_duration_hours,u.active_days,
  u.effective_download_mbps_sum/NULLIF(u.effective_download_mbps_count,0),
  u.vmos_sum/NULLIF(u.vmos_count,0),u.mos_sum/NULLIF(u.mos_count,0),
  u.avg_download_mbps_sum/NULLIF(u.avg_download_mbps_count,0),
  u.throughput_mbps_sum/NULLIF(u.throughput_mbps_count,0),
  u.max_single_flow_mbps_sum/NULLIF(u.max_single_flow_mbps_count,0),
  u.connection_success_sum/NULLIF(u.connection_success_count,0),
  u.connection_delay_sum/NULLIF(u.connection_delay_count,0),
  u.download_fluency_sum/NULLIF(u.download_fluency_count,0),
  u.upstream_rtt_sum/NULLIF(u.upstream_rtt_count,0),u.downstream_rtt_sum/NULLIF(u.downstream_rtt_count,0),
  u.user_up_loss_sum/NULLIF(u.user_up_loss_count,0),u.network_up_loss_sum/NULLIF(u.network_up_loss_count,0),
  u.wifi_delay_sum/NULLIF(u.wifi_delay_count,0),
  u.subscriber_rtt_sum/NULLIF(u.subscriber_rtt_count,0),u.network_rtt_sum/NULLIF(u.network_rtt_count,0),
  u.user_loss_sum/NULLIF(u.user_loss_count,0),u.network_loss_sum/NULLIF(u.network_loss_count,0),
  u.jitter_sum/NULLIF(u.jitter_count,0),
  u.effective_download_mbps_sum,u.effective_download_mbps_count,u.vmos_sum,u.vmos_count,u.mos_sum,u.mos_count,
  u.subscriber_rtt_sum,u.subscriber_rtt_count,u.network_rtt_sum,u.network_rtt_count,
  u.user_loss_sum,u.user_loss_count,u.network_loss_sum,u.network_loss_count,u.jitter_sum,u.jitter_count
FROM user_app u JOIN policy p ON p.analysis_run_id=u.analysis_run_id;

INSERT INTO :dws_app_access_period_experience_v2 (
  analysis_run_id,import_batch_id,grain_hash,user_type,app_category,app_name,data_type,profile_code,policy_version,
  observed_users,eligible_users,observation_rows,valid_obs_rows,poor_obs_rows,severe_obs_rows,
  poor_observation_rate_pct,ever_affected_users,ever_affected_user_rate_pct,
  persistent_poor_users,persistent_poor_user_rate_pct,severe_poor_users,severe_poor_user_rate_pct,
  sample_status,main_issue_driver,total_download_gb,total_game_hours,avg_effective_download_mbps,
  avg_vmos,avg_mos,avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,avg_network_loss_pct,avg_jitter_ms
)
WITH policy AS (
  SELECT p.* FROM meta_analysis_run_policy_binding b
  JOIN meta_experience_analysis_policy p
    ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version
  WHERE b.analysis_run_id=:analysis_run_id AND b.import_batch_id=:import_batch_id
), app_rollup AS (
  SELECT u.analysis_run_id,u.import_batch_id,u.user_type,u.app_category,u.app_name,u.data_type,u.profile_code,u.policy_version,
    COUNT(*) observed_users,SUM(u.eligible_user_flag) eligible_users,
    SUM(u.observation_rows) observation_rows,SUM(u.valid_obs_rows) valid_obs_rows,
    SUM(u.poor_obs_rows) poor_obs_rows,SUM(u.severe_obs_rows) severe_obs_rows,
    SUM(u.ever_affected_user_flag) ever_affected_users,
    SUM(u.persistent_poor_user_flag) persistent_poor_users,SUM(u.severe_poor_user_flag) severe_poor_users,
    SUM(u.poor_vmos_obs) poor_vmos_obs,SUM(u.poor_mos_obs) poor_mos_obs,
    SUM(u.poor_subscriber_rtt_obs) poor_subscriber_rtt_obs,SUM(u.poor_network_rtt_obs) poor_network_rtt_obs,
    SUM(u.poor_user_loss_obs) poor_user_loss_obs,SUM(u.poor_network_loss_obs) poor_network_loss_obs,
    SUM(u.poor_jitter_obs) poor_jitter_obs,SUM(u.total_download_gb) total_download_gb,SUM(u.total_game_hours) total_game_hours,
    SUM(u.effective_download_mbps_sum) effective_download_mbps_sum,SUM(u.effective_download_mbps_count) effective_download_mbps_count,
    SUM(u.vmos_sum) vmos_sum,SUM(u.vmos_count) vmos_count,SUM(u.mos_sum) mos_sum,SUM(u.mos_count) mos_count,
    SUM(u.subscriber_rtt_sum) subscriber_rtt_sum,SUM(u.subscriber_rtt_count) subscriber_rtt_count,
    SUM(u.network_rtt_sum) network_rtt_sum,SUM(u.network_rtt_count) network_rtt_count,
    SUM(u.user_loss_sum) user_loss_sum,SUM(u.user_loss_count) user_loss_count,
    SUM(u.network_loss_sum) network_loss_sum,SUM(u.network_loss_count) network_loss_count,
    SUM(u.jitter_sum) jitter_sum,SUM(u.jitter_count) jitter_count
  FROM :dws_user_app_period_experience_v2 u
  WHERE u.analysis_run_id=:analysis_run_id
  GROUP BY u.analysis_run_id,u.import_batch_id,u.user_type,u.app_category,u.app_name,u.data_type,u.profile_code,u.policy_version
)
SELECT a.analysis_run_id,a.import_batch_id,
  UNHEX(MD5(CONCAT_WS('|',a.user_type,a.app_category,a.app_name,a.data_type))),
  a.user_type,a.app_category,a.app_name,a.data_type,a.profile_code,a.policy_version,
  a.observed_users,a.eligible_users,a.observation_rows,a.valid_obs_rows,a.poor_obs_rows,a.severe_obs_rows,
  ROUND(a.poor_obs_rows*100.0/NULLIF(a.valid_obs_rows,0),6),
  a.ever_affected_users,ROUND(a.ever_affected_users*100.0/NULLIF(a.eligible_users,0),6),
  a.persistent_poor_users,ROUND(a.persistent_poor_users*100.0/NULLIF(a.eligible_users,0),6),
  a.severe_poor_users,ROUND(a.severe_poor_users*100.0/NULLIF(a.eligible_users,0),6),
  CASE WHEN a.eligible_users>=p.minimum_app_eligible_users AND a.valid_obs_rows>=p.minimum_app_valid_obs
    THEN 'SUFFICIENT' ELSE 'INSUFFICIENT_SAMPLE' END,
  CASE
    WHEN a.poor_network_rtt_obs+a.poor_network_loss_obs>=GREATEST(a.poor_vmos_obs+a.poor_mos_obs,a.poor_subscriber_rtt_obs+a.poor_user_loss_obs+a.poor_jitter_obs)
      AND a.poor_network_rtt_obs+a.poor_network_loss_obs>0 THEN 'NETWORK_SIDE'
    WHEN a.poor_subscriber_rtt_obs+a.poor_user_loss_obs+a.poor_jitter_obs>=GREATEST(a.poor_vmos_obs+a.poor_mos_obs,a.poor_network_rtt_obs+a.poor_network_loss_obs)
      AND a.poor_subscriber_rtt_obs+a.poor_user_loss_obs+a.poor_jitter_obs>0 THEN 'USER_SIDE'
    WHEN a.poor_vmos_obs>0 THEN 'LOW_VMOS' WHEN a.poor_mos_obs>0 THEN 'LOW_MOS' ELSE 'NO_DOMINANT_DRIVER' END,
  a.total_download_gb,a.total_game_hours,
  a.effective_download_mbps_sum/NULLIF(a.effective_download_mbps_count,0),
  a.vmos_sum/NULLIF(a.vmos_count,0),a.mos_sum/NULLIF(a.mos_count,0),
  a.subscriber_rtt_sum/NULLIF(a.subscriber_rtt_count,0),a.network_rtt_sum/NULLIF(a.network_rtt_count,0),
  a.user_loss_sum/NULLIF(a.user_loss_count,0),a.network_loss_sum/NULLIF(a.network_loss_count,0),
  a.jitter_sum/NULLIF(a.jitter_count,0)
FROM app_rollup a CROSS JOIN policy p;

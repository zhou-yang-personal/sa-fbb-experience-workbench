-- Shared user × App × hour aggregation core.
-- One bounded date/hour partition per transaction. Every average keeps SUM/COUNT
-- state so downstream DWS/ADS can roll up accurately without rescanning DWD.

DELETE FROM :dws_user_app_hourly_experience_v2
WHERE analysis_run_id=:analysis_run_id
  AND stat_date=:partition_date
  AND hour_of_day=:partition_hour;

INSERT INTO :dws_user_app_hourly_experience_v2 (
  analysis_run_id,import_batch_id,grain_hash,stat_date,hour_of_day,user_key,user_type,
  app_category,app_name,data_type,profile_code,policy_version,
  observation_rows,valid_obs_rows,poor_obs_rows,severe_obs_rows,
  poor_observation_rate_pct,persistent_poor_user_flag,severe_poor_user_flag,
  poor_vmos_obs,poor_mos_obs,poor_subscriber_rtt_obs,poor_network_rtt_obs,
  poor_user_loss_obs,poor_network_loss_obs,poor_jitter_obs,
  total_download_gb,total_game_hours,total_effective_duration_hours,total_video_duration_hours,
  avg_effective_download_mbps,avg_download_mbps,avg_throughput_mbps,avg_max_single_flow_mbps,
  avg_connection_success_pct,avg_connection_delay_ms,avg_download_fluency,
  avg_upstream_rtt_ms,avg_downstream_rtt_ms,avg_user_up_loss_pct,avg_network_up_loss_pct,avg_wifi_delay_ms,
  avg_vmos,avg_mos,avg_subscriber_rtt_ms,avg_network_rtt_ms,
  avg_user_loss_pct,avg_network_loss_pct,avg_jitter_ms,
  effective_download_mbps_sum,effective_download_mbps_count,
  avg_download_mbps_sum,avg_download_mbps_count,throughput_mbps_sum,throughput_mbps_count,
  max_single_flow_mbps_sum,max_single_flow_mbps_count,
  connection_success_sum,connection_success_count,connection_delay_sum,connection_delay_count,
  download_fluency_sum,download_fluency_count,upstream_rtt_sum,upstream_rtt_count,
  downstream_rtt_sum,downstream_rtt_count,user_up_loss_sum,user_up_loss_count,
  network_up_loss_sum,network_up_loss_count,wifi_delay_sum,wifi_delay_count,
  vmos_sum,vmos_count,mos_sum,mos_count,
  subscriber_rtt_sum,subscriber_rtt_count,network_rtt_sum,network_rtt_count,
  user_loss_sum,user_loss_count,network_loss_sum,network_loss_count,jitter_sum,jitter_count
)
WITH policy AS (
  SELECT b.analysis_run_id,b.import_batch_id,p.*
  FROM meta_analysis_run_policy_binding b
  JOIN meta_experience_analysis_policy p
    ON p.policy_id=b.experience_policy_id AND p.version=b.experience_policy_version
  WHERE b.analysis_run_id=:analysis_run_id AND b.import_batch_id=:import_batch_id
), tcp AS (
  SELECT p.analysis_run_id,d.import_batch_id,d.stat_date,d.hour_of_day,d.user_key,
    COALESCE(NULLIF(d.user_type,''),'UNAVAILABLE') user_type,
    COALESCE(NULLIF(d.app_category,''),'other') app_category,
    COALESCE(NULLIF(d.app_name,''),'UNKNOWN_APP') app_name,
    'tcp' data_type,ep.profile_code,p.version policy_version,
    COALESCE(d.downloaded_gb,0) downloaded_gb,
    CAST(0 AS DECIMAL(24,6)) game_hours,
    COALESCE(d.effective_duration_hours,0) effective_duration_hours,
    COALESCE(d.video_duration_hours,0) video_duration_hours,
    d.effective_download_mbps,d.avg_download_mbps,d.throughput_mbps,d.max_single_flow_mbps,
    d.connection_success_pct,d.connection_delay_ms,d.download_fluency,
    d.upstream_rtt_ms,d.downstream_rtt_ms,d.user_up_loss,d.network_up_loss,d.wifi_delay_ms,
    d.vmos,CAST(NULL AS DECIMAL(18,6)) mos,
    d.subscriber_side_rtt_ms,d.network_side_rtt_ms,
    d.user_down_loss user_loss,d.network_down_loss network_loss,
    CAST(NULL AS DECIMAL(18,6)) jitter,
    CASE WHEN d.vmos IS NOT NULL OR d.subscriber_side_rtt_ms IS NOT NULL
      OR d.network_side_rtt_ms IS NOT NULL OR d.user_down_loss IS NOT NULL
      OR d.network_down_loss IS NOT NULL THEN 1 ELSE 0 END valid_flag,
    CASE WHEN ep.poor_vmos_below IS NOT NULL AND d.vmos<ep.poor_vmos_below THEN 1 ELSE 0 END poor_vmos_flag,
    0 poor_mos_flag,
    CASE WHEN ep.poor_subscriber_rtt_ms_at_least IS NOT NULL AND d.subscriber_side_rtt_ms>=ep.poor_subscriber_rtt_ms_at_least THEN 1 ELSE 0 END poor_subscriber_rtt_flag,
    CASE WHEN ep.poor_network_rtt_ms_at_least IS NOT NULL AND d.network_side_rtt_ms>=ep.poor_network_rtt_ms_at_least THEN 1 ELSE 0 END poor_network_rtt_flag,
    CASE WHEN ep.poor_user_loss_pct_at_least IS NOT NULL AND d.user_down_loss>=ep.poor_user_loss_pct_at_least THEN 1 ELSE 0 END poor_user_loss_flag,
    CASE WHEN ep.poor_network_loss_pct_at_least IS NOT NULL AND d.network_down_loss>=ep.poor_network_loss_pct_at_least THEN 1 ELSE 0 END poor_network_loss_flag,
    0 poor_jitter_flag,
    CASE WHEN (ep.severe_vmos_below IS NOT NULL AND d.vmos<ep.severe_vmos_below)
      OR (ep.severe_subscriber_rtt_ms_at_least IS NOT NULL AND d.subscriber_side_rtt_ms>=ep.severe_subscriber_rtt_ms_at_least)
      OR (ep.severe_network_rtt_ms_at_least IS NOT NULL AND d.network_side_rtt_ms>=ep.severe_network_rtt_ms_at_least)
      OR (ep.severe_user_loss_pct_at_least IS NOT NULL AND d.user_down_loss>=ep.severe_user_loss_pct_at_least)
      OR (ep.severe_network_loss_pct_at_least IS NOT NULL AND d.network_down_loss>=ep.severe_network_loss_pct_at_least)
      THEN 1 ELSE 0 END severe_flag
  FROM :dwd_tcp_detail_clean d
  JOIN policy p ON p.import_batch_id=d.import_batch_id
  JOIN dim_app_experience_profile ep ON ep.profile_id=(
    SELECT ep2.profile_id FROM dim_app_experience_profile ep2
    WHERE ep2.policy_id=p.policy_id AND ep2.data_type='tcp' AND ep2.enabled=1
      AND (ep2.app_category IS NULL OR LOWER(ep2.app_category)=LOWER(COALESCE(NULLIF(d.app_category,''),'other')))
    ORDER BY CASE WHEN ep2.app_category IS NULL THEN 1 ELSE 0 END,ep2.priority,ep2.profile_code LIMIT 1)
  WHERE d.import_batch_id=:import_batch_id
    AND d.stat_date=:partition_date AND d.hour_of_day=:partition_hour
    AND d.user_key IS NOT NULL AND d.user_key<>'' AND d.user_key<>'UNKNOWN'
), game AS (
  SELECT p.analysis_run_id,d.import_batch_id,d.stat_date,d.hour_of_day,d.user_key,
    COALESCE(NULLIF(d.user_type,''),'UNAVAILABLE') user_type,
    COALESCE(NULLIF(d.app_category,''),'game') app_category,
    COALESCE(NULLIF(d.app_name,''),'UNKNOWN_APP') app_name,
    'game' data_type,ep.profile_code,p.version policy_version,
    CAST(0 AS DECIMAL(24,6)) downloaded_gb,COALESCE(d.game_hours,0) game_hours,
    CAST(0 AS DECIMAL(24,6)) effective_duration_hours,
    CAST(0 AS DECIMAL(24,6)) video_duration_hours,
    CAST(NULL AS DECIMAL(18,6)) effective_download_mbps,
    CAST(NULL AS DECIMAL(18,6)) avg_download_mbps,
    CAST(NULL AS DECIMAL(18,6)) throughput_mbps,
    CAST(NULL AS DECIMAL(18,6)) max_single_flow_mbps,
    CAST(NULL AS DECIMAL(18,6)) connection_success_pct,
    CAST(NULL AS DECIMAL(18,6)) connection_delay_ms,
    CAST(NULL AS DECIMAL(18,6)) download_fluency,
    CAST(NULL AS DECIMAL(18,6)) upstream_rtt_ms,
    CAST(NULL AS DECIMAL(18,6)) downstream_rtt_ms,
    CAST(NULL AS DECIMAL(18,6)) user_up_loss,
    CAST(NULL AS DECIMAL(18,6)) network_up_loss,d.wifi_delay_ms,
    CAST(NULL AS DECIMAL(18,6)) vmos,d.mos,
    d.worst_latency_ms subscriber_side_rtt_ms,
    CAST(NULL AS DECIMAL(18,6)) network_side_rtt_ms,
    d.worst_loss user_loss,CAST(NULL AS DECIMAL(18,6)) network_loss,d.worst_jitter_ms jitter,
    CASE WHEN d.mos IS NOT NULL OR d.worst_latency_ms IS NOT NULL
      OR d.worst_loss IS NOT NULL OR d.worst_jitter_ms IS NOT NULL THEN 1 ELSE 0 END valid_flag,
    0 poor_vmos_flag,
    CASE WHEN ep.poor_mos_below IS NOT NULL AND d.mos<ep.poor_mos_below THEN 1 ELSE 0 END poor_mos_flag,
    CASE WHEN ep.poor_subscriber_rtt_ms_at_least IS NOT NULL AND d.worst_latency_ms>=ep.poor_subscriber_rtt_ms_at_least THEN 1 ELSE 0 END poor_subscriber_rtt_flag,
    0 poor_network_rtt_flag,
    CASE WHEN ep.poor_user_loss_pct_at_least IS NOT NULL AND d.worst_loss>=ep.poor_user_loss_pct_at_least THEN 1 ELSE 0 END poor_user_loss_flag,
    0 poor_network_loss_flag,
    CASE WHEN ep.poor_jitter_ms_at_least IS NOT NULL AND d.worst_jitter_ms>=ep.poor_jitter_ms_at_least THEN 1 ELSE 0 END poor_jitter_flag,
    CASE WHEN (ep.severe_mos_below IS NOT NULL AND d.mos<ep.severe_mos_below)
      OR (ep.severe_subscriber_rtt_ms_at_least IS NOT NULL AND d.worst_latency_ms>=ep.severe_subscriber_rtt_ms_at_least)
      OR (ep.severe_user_loss_pct_at_least IS NOT NULL AND d.worst_loss>=ep.severe_user_loss_pct_at_least)
      OR (ep.severe_jitter_ms_at_least IS NOT NULL AND d.worst_jitter_ms>=ep.severe_jitter_ms_at_least)
      THEN 1 ELSE 0 END severe_flag
  FROM :dwd_game_detail_clean d
  JOIN policy p ON p.import_batch_id=d.import_batch_id
  JOIN dim_app_experience_profile ep ON ep.profile_id=(
    SELECT ep2.profile_id FROM dim_app_experience_profile ep2
    WHERE ep2.policy_id=p.policy_id AND ep2.data_type='game' AND ep2.enabled=1
      AND (ep2.app_category IS NULL OR LOWER(ep2.app_category)=LOWER(COALESCE(NULLIF(d.app_category,''),'game')))
    ORDER BY CASE WHEN ep2.app_category IS NULL THEN 1 ELSE 0 END,ep2.priority,ep2.profile_code LIMIT 1)
  WHERE d.import_batch_id=:import_batch_id
    AND d.stat_date=:partition_date AND d.hour_of_day=:partition_hour
    AND d.user_key IS NOT NULL AND d.user_key<>'' AND d.user_key<>'UNKNOWN'
), observations AS (
  SELECT * FROM tcp UNION ALL SELECT * FROM game
), hourly AS (
  SELECT o.analysis_run_id,o.import_batch_id,o.stat_date,o.hour_of_day,o.user_key,o.user_type,
    o.app_category,o.app_name,o.data_type,o.profile_code,o.policy_version,
    COUNT(*) observation_rows,SUM(o.valid_flag) valid_obs_rows,
    SUM(CASE WHEN o.valid_flag=1 AND (o.poor_vmos_flag=1 OR o.poor_mos_flag=1
      OR o.poor_subscriber_rtt_flag=1 OR o.poor_network_rtt_flag=1
      OR o.poor_user_loss_flag=1 OR o.poor_network_loss_flag=1 OR o.poor_jitter_flag=1) THEN 1 ELSE 0 END) poor_obs_rows,
    SUM(CASE WHEN o.valid_flag=1 THEN o.severe_flag ELSE 0 END) severe_obs_rows,
    SUM(o.poor_vmos_flag) poor_vmos_obs,SUM(o.poor_mos_flag) poor_mos_obs,
    SUM(o.poor_subscriber_rtt_flag) poor_subscriber_rtt_obs,
    SUM(o.poor_network_rtt_flag) poor_network_rtt_obs,
    SUM(o.poor_user_loss_flag) poor_user_loss_obs,
    SUM(o.poor_network_loss_flag) poor_network_loss_obs,SUM(o.poor_jitter_flag) poor_jitter_obs,
    SUM(o.downloaded_gb) total_download_gb,SUM(o.game_hours) total_game_hours,
    SUM(o.effective_duration_hours) total_effective_duration_hours,
    SUM(o.video_duration_hours) total_video_duration_hours,
    COALESCE(SUM(o.effective_download_mbps),0) effective_download_mbps_sum,COUNT(o.effective_download_mbps) effective_download_mbps_count,
    COALESCE(SUM(o.avg_download_mbps),0) avg_download_mbps_sum,COUNT(o.avg_download_mbps) avg_download_mbps_count,
    COALESCE(SUM(o.throughput_mbps),0) throughput_mbps_sum,COUNT(o.throughput_mbps) throughput_mbps_count,
    COALESCE(SUM(o.max_single_flow_mbps),0) max_single_flow_mbps_sum,COUNT(o.max_single_flow_mbps) max_single_flow_mbps_count,
    COALESCE(SUM(o.connection_success_pct),0) connection_success_sum,COUNT(o.connection_success_pct) connection_success_count,
    COALESCE(SUM(o.connection_delay_ms),0) connection_delay_sum,COUNT(o.connection_delay_ms) connection_delay_count,
    COALESCE(SUM(o.download_fluency),0) download_fluency_sum,COUNT(o.download_fluency) download_fluency_count,
    COALESCE(SUM(o.upstream_rtt_ms),0) upstream_rtt_sum,COUNT(o.upstream_rtt_ms) upstream_rtt_count,
    COALESCE(SUM(o.downstream_rtt_ms),0) downstream_rtt_sum,COUNT(o.downstream_rtt_ms) downstream_rtt_count,
    COALESCE(SUM(o.user_up_loss),0) user_up_loss_sum,COUNT(o.user_up_loss) user_up_loss_count,
    COALESCE(SUM(o.network_up_loss),0) network_up_loss_sum,COUNT(o.network_up_loss) network_up_loss_count,
    COALESCE(SUM(o.wifi_delay_ms),0) wifi_delay_sum,COUNT(o.wifi_delay_ms) wifi_delay_count,
    COALESCE(SUM(o.vmos),0) vmos_sum,COUNT(o.vmos) vmos_count,
    COALESCE(SUM(o.mos),0) mos_sum,COUNT(o.mos) mos_count,
    COALESCE(SUM(o.subscriber_side_rtt_ms),0) subscriber_rtt_sum,COUNT(o.subscriber_side_rtt_ms) subscriber_rtt_count,
    COALESCE(SUM(o.network_side_rtt_ms),0) network_rtt_sum,COUNT(o.network_side_rtt_ms) network_rtt_count,
    COALESCE(SUM(o.user_loss),0) user_loss_sum,COUNT(o.user_loss) user_loss_count,
    COALESCE(SUM(o.network_loss),0) network_loss_sum,COUNT(o.network_loss) network_loss_count,
    COALESCE(SUM(o.jitter),0) jitter_sum,COUNT(o.jitter) jitter_count
  FROM observations o
  GROUP BY o.analysis_run_id,o.import_batch_id,o.stat_date,o.hour_of_day,o.user_key,o.user_type,
    o.app_category,o.app_name,o.data_type,o.profile_code,o.policy_version
)
SELECT h.analysis_run_id,h.import_batch_id,
  UNHEX(MD5(CONCAT_WS('|',h.stat_date,h.hour_of_day,h.user_key,h.user_type,h.app_category,h.app_name,h.data_type))),
  h.stat_date,h.hour_of_day,h.user_key,h.user_type,h.app_category,h.app_name,h.data_type,h.profile_code,h.policy_version,
  h.observation_rows,h.valid_obs_rows,h.poor_obs_rows,h.severe_obs_rows,
  ROUND(h.poor_obs_rows*100.0/NULLIF(h.valid_obs_rows,0),6),
  CASE WHEN h.valid_obs_rows>=p.persistent_min_valid_obs AND h.poor_obs_rows>=p.persistent_min_poor_obs
    AND h.poor_obs_rows*100.0/NULLIF(h.valid_obs_rows,0)>=p.persistent_min_poor_rate_pct THEN 1 ELSE 0 END,
  CASE WHEN h.valid_obs_rows>=p.severe_user_min_valid_obs AND h.severe_obs_rows>=p.severe_user_min_severe_obs
    AND h.severe_obs_rows*100.0/NULLIF(h.valid_obs_rows,0)>=p.severe_user_min_severe_rate_pct THEN 1 ELSE 0 END,
  h.poor_vmos_obs,h.poor_mos_obs,h.poor_subscriber_rtt_obs,h.poor_network_rtt_obs,
  h.poor_user_loss_obs,h.poor_network_loss_obs,h.poor_jitter_obs,
  h.total_download_gb,h.total_game_hours,h.total_effective_duration_hours,h.total_video_duration_hours,
  h.effective_download_mbps_sum/NULLIF(h.effective_download_mbps_count,0),
  h.avg_download_mbps_sum/NULLIF(h.avg_download_mbps_count,0),
  h.throughput_mbps_sum/NULLIF(h.throughput_mbps_count,0),
  h.max_single_flow_mbps_sum/NULLIF(h.max_single_flow_mbps_count,0),
  h.connection_success_sum/NULLIF(h.connection_success_count,0),
  h.connection_delay_sum/NULLIF(h.connection_delay_count,0),
  h.download_fluency_sum/NULLIF(h.download_fluency_count,0),
  h.upstream_rtt_sum/NULLIF(h.upstream_rtt_count,0),
  h.downstream_rtt_sum/NULLIF(h.downstream_rtt_count,0),
  h.user_up_loss_sum/NULLIF(h.user_up_loss_count,0),
  h.network_up_loss_sum/NULLIF(h.network_up_loss_count,0),
  h.wifi_delay_sum/NULLIF(h.wifi_delay_count,0),
  h.vmos_sum/NULLIF(h.vmos_count,0),h.mos_sum/NULLIF(h.mos_count,0),
  h.subscriber_rtt_sum/NULLIF(h.subscriber_rtt_count,0),h.network_rtt_sum/NULLIF(h.network_rtt_count,0),
  h.user_loss_sum/NULLIF(h.user_loss_count,0),h.network_loss_sum/NULLIF(h.network_loss_count,0),
  h.jitter_sum/NULLIF(h.jitter_count,0),
  h.effective_download_mbps_sum,h.effective_download_mbps_count,
  h.avg_download_mbps_sum,h.avg_download_mbps_count,h.throughput_mbps_sum,h.throughput_mbps_count,
  h.max_single_flow_mbps_sum,h.max_single_flow_mbps_count,
  h.connection_success_sum,h.connection_success_count,h.connection_delay_sum,h.connection_delay_count,
  h.download_fluency_sum,h.download_fluency_count,h.upstream_rtt_sum,h.upstream_rtt_count,
  h.downstream_rtt_sum,h.downstream_rtt_count,h.user_up_loss_sum,h.user_up_loss_count,
  h.network_up_loss_sum,h.network_up_loss_count,h.wifi_delay_sum,h.wifi_delay_count,
  h.vmos_sum,h.vmos_count,h.mos_sum,h.mos_count,
  h.subscriber_rtt_sum,h.subscriber_rtt_count,h.network_rtt_sum,h.network_rtt_count,
  h.user_loss_sum,h.user_loss_count,h.network_loss_sum,h.network_loss_count,h.jitter_sum,h.jitter_count
FROM hourly h JOIN policy p ON p.analysis_run_id=h.analysis_run_id;

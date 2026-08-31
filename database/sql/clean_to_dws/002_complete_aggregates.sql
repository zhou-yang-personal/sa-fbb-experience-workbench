-- Compatibility DWS derived from the shared user × App × hour/period core.
-- These tables remain available to 1.0.x dashboards without another DWD scan.

DELETE FROM :dws_app_daily WHERE import_batch_id=:import_batch_id;
DELETE FROM :dws_app_user_summary WHERE import_batch_id=:import_batch_id;
DELETE FROM :dws_app_category_daily WHERE import_batch_id=:import_batch_id;
DELETE FROM :dws_access_type_hourly_compare WHERE import_batch_id=:import_batch_id;
DELETE FROM :dws_user_experience_bottleneck WHERE import_batch_id=:import_batch_id;

INSERT INTO :dws_app_daily (
  import_batch_id,stat_date,user_type,app_category,app_name,observation_rows,active_users,
  total_download_gb,total_game_hours,avg_effective_download_mbps,avg_vmos,avg_mos,
  avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_loss_pct,avg_network_loss_pct,poor_experience_users
)
WITH user_day AS (
  SELECT h.import_batch_id,h.stat_date,h.user_type,h.app_category,h.app_name,h.user_key,
    SUM(h.observation_rows) observation_rows,SUM(h.total_download_gb) total_download_gb,
    SUM(h.total_game_hours) total_game_hours,SUM(h.poor_obs_rows) poor_obs_rows,
    SUM(h.effective_download_mbps_sum) effective_sum,SUM(h.effective_download_mbps_count) effective_count,
    SUM(h.vmos_sum) vmos_sum,SUM(h.vmos_count) vmos_count,SUM(h.mos_sum) mos_sum,SUM(h.mos_count) mos_count,
    SUM(h.subscriber_rtt_sum) subscriber_sum,SUM(h.subscriber_rtt_count) subscriber_count,
    SUM(h.network_rtt_sum) network_sum,SUM(h.network_rtt_count) network_count,
    SUM(h.user_loss_sum) user_loss_sum,SUM(h.user_loss_count) user_loss_count,
    SUM(h.network_loss_sum) network_loss_sum,SUM(h.network_loss_count) network_loss_count
  FROM :dws_user_app_hourly_experience_v2 h
  WHERE h.analysis_run_id=:analysis_run_id AND h.import_batch_id=:import_batch_id
  GROUP BY h.import_batch_id,h.stat_date,h.user_type,h.app_category,h.app_name,h.user_key
)
SELECT import_batch_id,stat_date,user_type,app_category,app_name,SUM(observation_rows),COUNT(*),
  SUM(total_download_gb),SUM(total_game_hours),SUM(effective_sum)/NULLIF(SUM(effective_count),0),
  SUM(vmos_sum)/NULLIF(SUM(vmos_count),0),SUM(mos_sum)/NULLIF(SUM(mos_count),0),
  SUM(subscriber_sum)/NULLIF(SUM(subscriber_count),0),SUM(network_sum)/NULLIF(SUM(network_count),0),
  SUM(user_loss_sum)/NULLIF(SUM(user_loss_count),0),SUM(network_loss_sum)/NULLIF(SUM(network_loss_count),0),
  SUM(CASE WHEN poor_obs_rows>0 THEN 1 ELSE 0 END)
FROM user_day GROUP BY import_batch_id,stat_date,user_type,app_category,app_name;

INSERT INTO :dws_app_user_summary (
  import_batch_id,user_key,user_type,app_category,app_name,observation_rows,total_download_gb,total_game_hours,
  avg_effective_download_mbps,avg_vmos,avg_mos,avg_subscriber_rtt_ms,avg_network_rtt_ms,
  avg_user_loss_pct,avg_network_loss_pct,poor_experience_flag
)
SELECT import_batch_id,user_key,user_type,app_category,app_name,observation_rows,total_download_gb,total_game_hours,
  avg_effective_download_mbps,avg_vmos,avg_mos,avg_subscriber_rtt_ms,avg_network_rtt_ms,
  avg_user_loss_pct,avg_network_loss_pct,persistent_poor_user_flag
FROM :dws_user_app_period_experience_v2
WHERE analysis_run_id=:analysis_run_id AND import_batch_id=:import_batch_id;

INSERT INTO :dws_app_category_daily (
  import_batch_id,stat_date,user_type,app_category,active_users,total_download_gb,total_game_hours,
  avg_vmos,avg_mos,avg_effective_download_mbps
)
SELECT h.import_batch_id,h.stat_date,h.user_type,h.app_category,COUNT(DISTINCT h.user_key),
  SUM(h.total_download_gb),SUM(h.total_game_hours),
  SUM(h.vmos_sum)/NULLIF(SUM(h.vmos_count),0),SUM(h.mos_sum)/NULLIF(SUM(h.mos_count),0),
  SUM(h.effective_download_mbps_sum)/NULLIF(SUM(h.effective_download_mbps_count),0)
FROM :dws_user_app_hourly_experience_v2 h
WHERE h.analysis_run_id=:analysis_run_id AND h.import_batch_id=:import_batch_id
GROUP BY h.import_batch_id,h.stat_date,h.user_type,h.app_category;

INSERT INTO :dws_access_type_hourly_compare (
  import_batch_id,stat_date,hour_of_day,user_type,active_users,avg_vmos,avg_mos,
  avg_subscriber_rtt_ms,avg_network_rtt_ms,avg_user_down_loss,avg_network_down_loss,avg_download_mbps
)
SELECT h.import_batch_id,h.stat_date,h.hour_of_day,h.user_type,COUNT(DISTINCT h.user_key),
  SUM(h.vmos_sum)/NULLIF(SUM(h.vmos_count),0),SUM(h.mos_sum)/NULLIF(SUM(h.mos_count),0),
  SUM(h.subscriber_rtt_sum)/NULLIF(SUM(h.subscriber_rtt_count),0),
  SUM(h.network_rtt_sum)/NULLIF(SUM(h.network_rtt_count),0),
  SUM(h.user_loss_sum)/NULLIF(SUM(h.user_loss_count),0),
  SUM(h.network_loss_sum)/NULLIF(SUM(h.network_loss_count),0),
  SUM(h.effective_download_mbps_sum)/NULLIF(SUM(h.effective_download_mbps_count),0)
FROM :dws_user_app_hourly_experience_v2 h
WHERE h.analysis_run_id=:analysis_run_id AND h.import_batch_id=:import_batch_id
GROUP BY h.import_batch_id,h.stat_date,h.hour_of_day,h.user_type;

REPLACE INTO :dws_user_experience_bottleneck (import_batch_id,user_key,bottleneck_type,severity_score,evidence)
WITH profile AS (
  SELECT * FROM :dws_user_daily_profile WHERE import_batch_id=:import_batch_id
)
SELECT import_batch_id,user_key,
  CASE
    WHEN avg_network_down_loss>=2 OR avg_network_rtt_ms>=100 THEN 'NETWORK_SIDE_SEVERE'
    WHEN avg_user_down_loss>=1 OR avg_subscriber_rtt_ms>=50 THEN 'USER_SIDE_OR_WIFI_PRESSURE'
    WHEN peak_row_pct>=35 THEN 'PEAK_HOUR_PRESSURE'
    ELSE 'APP_DEMAND_HIGH_BUT_EXPERIENCE_OK' END,
  CAST(LEAST(100,COALESCE(avg_network_down_loss,0)*20+COALESCE(avg_user_down_loss,0)*15+
    COALESCE(avg_network_rtt_ms,0)/2+COALESCE(avg_subscriber_rtt_ms,0)/2+COALESCE(peak_row_pct,0)/2) AS SIGNED),
  CONCAT('network_rtt=',COALESCE(avg_network_rtt_ms,0),', subscriber_rtt=',COALESCE(avg_subscriber_rtt_ms,0),', peak_pct=',COALESCE(peak_row_pct,0))
FROM profile WHERE user_key IS NOT NULL AND TRIM(user_key)<>'' AND user_key<>'UNKNOWN';

DELETE FROM :ads_app_experience_rank WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_app_experience_rank (
  analysis_run_id, import_batch_id, app_category, app_name, user_type,
  active_users, traffic_gb, duration_hours, avg_effective_mbps,
  avg_vmos, avg_mos, avg_subscriber_rtt_ms, avg_network_rtt_ms,
  avg_user_loss_pct, avg_network_loss_pct, poor_experience_users,
  poor_experience_user_pct, main_issue_driver, evidence_summary
)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id)
SELECT
  params.analysis_run_id,
  params.import_batch_id,
  COALESCE(a.app_category, 'UNKNOWN'),
  COALESCE(a.app_name, 'UNKNOWN_APP'),
  COALESCE(a.user_type, 'UNKNOWN'),
  COUNT(DISTINCT a.user_key),
  SUM(a.total_download_gb),
  SUM(a.total_game_hours),
  SUM(a.avg_effective_download_mbps * a.observation_rows) / NULLIF(SUM(CASE WHEN a.avg_effective_download_mbps IS NOT NULL THEN a.observation_rows ELSE 0 END),0),
  SUM(a.avg_vmos * a.observation_rows) / NULLIF(SUM(CASE WHEN a.avg_vmos IS NOT NULL THEN a.observation_rows ELSE 0 END),0),
  SUM(a.avg_mos * a.observation_rows) / NULLIF(SUM(CASE WHEN a.avg_mos IS NOT NULL THEN a.observation_rows ELSE 0 END),0),
  SUM(a.avg_subscriber_rtt_ms * a.observation_rows) / NULLIF(SUM(CASE WHEN a.avg_subscriber_rtt_ms IS NOT NULL THEN a.observation_rows ELSE 0 END),0),
  SUM(a.avg_network_rtt_ms * a.observation_rows) / NULLIF(SUM(CASE WHEN a.avg_network_rtt_ms IS NOT NULL THEN a.observation_rows ELSE 0 END),0),
  SUM(a.avg_user_loss_pct * a.observation_rows) / NULLIF(SUM(CASE WHEN a.avg_user_loss_pct IS NOT NULL THEN a.observation_rows ELSE 0 END),0),
  SUM(a.avg_network_loss_pct * a.observation_rows) / NULLIF(SUM(CASE WHEN a.avg_network_loss_pct IS NOT NULL THEN a.observation_rows ELSE 0 END),0),
  SUM(a.poor_experience_flag),
  ROUND(SUM(a.poor_experience_flag) / NULLIF(COUNT(DISTINCT a.user_key),0) * 100, 4),
  CASE
    WHEN AVG(a.avg_network_loss_pct) >= 2 OR AVG(a.avg_network_rtt_ms) >= 100 THEN 'NETWORK_SIDE_SEVERE'
    WHEN AVG(a.avg_user_loss_pct) >= 1 OR AVG(a.avg_subscriber_rtt_ms) >= 50 THEN 'USER_SIDE_OR_WIFI_PRESSURE'
    WHEN AVG(a.avg_vmos) < 3.5 THEN 'LOW_VMOS'
    WHEN AVG(a.avg_mos) < 3.5 THEN 'LOW_MOS'
    ELSE 'EXPERIENCE_OK'
  END,
  CONCAT('source=dws_app_user_summary; users=', COUNT(DISTINCT a.user_key), '; observations=', SUM(a.observation_rows))
FROM :dws_app_user_summary a, params
WHERE a.import_batch_id = params.import_batch_id
GROUP BY params.analysis_run_id, params.import_batch_id, COALESCE(a.app_category, 'UNKNOWN'), COALESCE(a.app_name, 'UNKNOWN_APP'), COALESCE(a.user_type, 'UNKNOWN');

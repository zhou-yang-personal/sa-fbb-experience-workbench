DELETE FROM :ads_hourly_experience_trend WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_hourly_experience_trend (
  analysis_run_id, import_batch_id, stat_date, hour_of_day, user_type, app_category,
  active_users, traffic_gb, duration_hours, avg_effective_mbps,
  avg_vmos, avg_mos, avg_subscriber_rtt_ms, avg_network_rtt_ms,
  avg_user_loss_pct, avg_network_loss_pct, poor_experience_users
)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id)
SELECT
  params.analysis_run_id,
  params.import_batch_id,
  h.stat_date,
  h.hour_of_day,
  COALESCE(h.user_type, 'UNKNOWN'),
  'ALL',
  SUM(h.active_users),
  0,
  0,
  AVG(h.avg_download_mbps),
  AVG(h.avg_vmos),
  AVG(h.avg_mos),
  AVG(h.avg_subscriber_rtt_ms),
  AVG(h.avg_network_rtt_ms),
  AVG(h.avg_user_down_loss),
  AVG(h.avg_network_down_loss),
  SUM(CASE WHEN COALESCE(h.avg_vmos, 5) < 3.5 OR COALESCE(h.avg_mos, 5) < 3.5 OR COALESCE(h.avg_subscriber_rtt_ms, 0) > 100 OR COALESCE(h.avg_user_down_loss, 0) > 2 THEN h.active_users ELSE 0 END)
FROM :dws_access_type_hourly_compare h, params
WHERE h.import_batch_id = params.import_batch_id
GROUP BY params.analysis_run_id, params.import_batch_id, h.stat_date, h.hour_of_day, COALESCE(h.user_type, 'UNKNOWN');

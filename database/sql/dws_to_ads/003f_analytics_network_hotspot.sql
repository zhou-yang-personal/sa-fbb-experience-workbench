DELETE FROM :ads_network_hotspot_rank WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_network_hotspot_rank (
  analysis_run_id, import_batch_id, bras, olt, pon, user_type,
  affected_users, traffic_gb, avg_subscriber_rtt_ms, avg_network_rtt_ms,
  avg_user_loss_pct, avg_network_loss_pct, avg_wifi_delay_ms, poor_experience_user_pct,
  main_issue_driver, suggested_action, evidence_summary
)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id), observations AS (
  SELECT import_batch_id, user_key, COALESCE(user_type,'UNKNOWN') AS user_type,
         COALESCE(bras,'UNKNOWN') AS bras, COALESCE(olt,'UNKNOWN') AS olt, COALESCE(pon,'UNKNOWN') AS pon,
         COALESCE(downloaded_gb,0) AS traffic_gb, subscriber_side_rtt_ms, network_side_rtt_ms,
         user_down_loss, network_down_loss, wifi_delay_ms,
         CASE WHEN vmos < 3.5 OR subscriber_side_rtt_ms >= 50 OR network_side_rtt_ms >= 100 OR user_down_loss >= 1 OR network_down_loss >= 2 THEN 1 ELSE 0 END AS poor_flag
  FROM :dwd_tcp_detail_clean
  WHERE import_batch_id = (SELECT import_batch_id FROM params) AND user_key <> 'UNKNOWN'
  UNION ALL
  SELECT import_batch_id, user_key, COALESCE(user_type,'UNKNOWN'),
         COALESCE(bras,'UNKNOWN'), COALESCE(olt,'UNKNOWN'), COALESCE(pon,'UNKNOWN'),
         0, worst_latency_ms, NULL, worst_loss, NULL, wifi_delay_ms,
         CASE WHEN mos < 3.5 OR worst_latency_ms >= 100 OR worst_loss >= 1 THEN 1 ELSE 0 END
  FROM :dwd_game_detail_clean
  WHERE import_batch_id = (SELECT import_batch_id FROM params) AND user_key <> 'UNKNOWN'
)
SELECT
  params.analysis_run_id,
  params.import_batch_id,
  o.bras,
  o.olt,
  o.pon,
  o.user_type,
  COUNT(DISTINCT CASE WHEN o.poor_flag=1 THEN o.user_key END),
  SUM(o.traffic_gb),
  AVG(o.subscriber_side_rtt_ms),
  AVG(o.network_side_rtt_ms),
  AVG(o.user_down_loss),
  AVG(o.network_down_loss),
  AVG(o.wifi_delay_ms),
  ROUND(COUNT(DISTINCT CASE WHEN o.poor_flag=1 THEN o.user_key END) / NULLIF(COUNT(DISTINCT o.user_key),0) * 100, 4),
  CASE
    WHEN AVG(o.network_down_loss) >= 2 OR AVG(o.network_side_rtt_ms) >= 100 THEN 'NETWORK_SIDE_SEVERE'
    WHEN AVG(o.wifi_delay_ms) >= 30 OR AVG(o.user_down_loss) >= 1 OR AVG(o.subscriber_side_rtt_ms) >= 50 THEN 'USER_SIDE_OR_WIFI_PRESSURE'
    ELSE 'EXPERIENCE_REVIEW'
  END,
  CASE
    WHEN AVG(o.network_down_loss) >= 2 OR AVG(o.network_side_rtt_ms) >= 100 THEN 'NETWORK_CHECK'
    WHEN AVG(o.wifi_delay_ms) >= 30 OR AVG(o.user_down_loss) >= 1 OR AVG(o.subscriber_side_rtt_ms) >= 50 THEN 'HOME_WIFI_REVIEW'
    ELSE 'MONITOR'
  END,
  CONCAT('source=dwd_tcp+dwd_game; observed_users=', COUNT(DISTINCT o.user_key), '; poor_users=', COUNT(DISTINCT CASE WHEN o.poor_flag=1 THEN o.user_key END))
FROM observations o, params
GROUP BY params.analysis_run_id, params.import_batch_id, o.bras, o.olt, o.pon, o.user_type;

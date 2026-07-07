DELETE FROM :ads_network_hotspot_rank WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_network_hotspot_rank (
  analysis_run_id, import_batch_id, bras, olt, pon, user_type,
  affected_users, traffic_gb, avg_subscriber_rtt_ms, avg_network_rtt_ms,
  avg_user_loss_pct, avg_network_loss_pct, poor_experience_user_pct,
  main_issue_driver, suggested_action, evidence_summary
)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id)
SELECT
  params.analysis_run_id,
  params.import_batch_id,
  COALESCE(b.bottleneck_type, 'UNKNOWN'),
  'UNKNOWN',
  'UNKNOWN',
  COALESCE(p.user_type, 'UNKNOWN'),
  COUNT(DISTINCT p.user_key),
  SUM(p.total_download_gb),
  AVG(p.avg_subscriber_rtt_ms),
  AVG(p.avg_network_rtt_ms),
  AVG(p.avg_user_down_loss),
  AVG(p.avg_network_down_loss),
  AVG(CASE WHEN COALESCE(b.severity_score, 0) > 0 THEN 100 ELSE 0 END),
  COALESCE(b.bottleneck_type, 'UNKNOWN'),
  CASE WHEN COALESCE(b.bottleneck_type, 'UNKNOWN') = 'network' THEN 'network_check' ELSE 'experience_review' END,
  CONCAT('source=dws_user_experience_bottleneck; users=', COUNT(DISTINCT p.user_key))
FROM :dws_user_daily_profile p
LEFT JOIN :dws_user_experience_bottleneck b ON b.import_batch_id = p.import_batch_id AND b.user_key = p.user_key
JOIN params ON p.import_batch_id = params.import_batch_id
GROUP BY params.analysis_run_id, params.import_batch_id, COALESCE(b.bottleneck_type, 'UNKNOWN'), COALESCE(p.user_type, 'UNKNOWN');

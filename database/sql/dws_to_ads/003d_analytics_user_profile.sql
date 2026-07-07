DELETE FROM :ads_user_experience_profile WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_user_experience_profile (
  analysis_run_id, import_batch_id, user_key, user_type, active_days,
  total_traffic_gb, game_hours, avg_vmos, avg_mos,
  avg_subscriber_rtt_ms, avg_network_rtt_ms, avg_user_loss_pct, avg_network_loss_pct,
  bottleneck_side, issue_driver, lead_type, demand_score, migration_motive_score,
  recommended_offer, evidence_summary
)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id)
SELECT
  params.analysis_run_id,
  params.import_batch_id,
  p.user_key,
  COALESCE(MAX(p.user_type), 'UNKNOWN'),
  COUNT(DISTINCT p.stat_date),
  SUM(p.total_download_gb),
  SUM(p.total_game_hours),
  AVG(p.avg_vmos),
  AVG(p.avg_mos),
  AVG(p.avg_subscriber_rtt_ms),
  AVG(p.avg_network_rtt_ms),
  AVG(p.avg_user_down_loss),
  AVG(p.avg_network_down_loss),
  COALESCE(MAX(b.bottleneck_type), 'UNKNOWN'),
  COALESCE(MAX(b.bottleneck_type), 'UNKNOWN'),
  COALESCE(MAX(l.lead_type), 'NONE'),
  COALESCE(MAX(l.demand_score), 0),
  COALESCE(MAX(l.migration_motive_score), 0),
  COALESCE(MAX(l.recommended_offer), ''),
  CONCAT('source=dws_user_daily_profile; days=', COUNT(DISTINCT p.stat_date))
FROM :dws_user_daily_profile p
LEFT JOIN :dws_user_experience_bottleneck b ON b.import_batch_id = p.import_batch_id AND b.user_key = p.user_key
LEFT JOIN :ads_migration_lead_user l ON l.analysis_run_id = (SELECT analysis_run_id FROM params) AND l.user_key = p.user_key
JOIN params ON p.import_batch_id = params.import_batch_id
GROUP BY params.analysis_run_id, params.import_batch_id, p.user_key;

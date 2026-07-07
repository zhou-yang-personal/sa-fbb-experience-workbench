DELETE FROM :ads_lead_evidence_detail WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_lead_evidence_detail (
  analysis_run_id, import_batch_id, user_key, user_type, lead_type,
  demand_score, migration_motive_score, recommended_offer,
  total_traffic_gb, game_hours, avg_vmos, avg_mos,
  bottleneck_side, issue_driver, evidence_summary
)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id)
SELECT
  params.analysis_run_id,
  params.import_batch_id,
  l.user_key,
  COALESCE(l.user_type, MAX(p.user_type), 'UNKNOWN'),
  COALESCE(l.lead_type, 'UNKNOWN'),
  COALESCE(l.demand_score, 0),
  COALESCE(l.migration_motive_score, 0),
  COALESCE(l.recommended_offer, ''),
  COALESCE(SUM(p.total_download_gb), 0),
  COALESCE(SUM(p.total_game_hours), 0),
  AVG(p.avg_vmos),
  AVG(p.avg_mos),
  COALESCE(MAX(b.bottleneck_type), 'UNKNOWN'),
  COALESCE(MAX(b.bottleneck_type), 'UNKNOWN'),
  CONCAT('source=ads_migration_lead_user; profile_rows=', COUNT(p.user_key))
FROM :ads_migration_lead_user l
JOIN params ON l.analysis_run_id = params.analysis_run_id
LEFT JOIN :dws_user_daily_profile p ON p.import_batch_id = params.import_batch_id AND p.user_key = l.user_key
LEFT JOIN :dws_user_experience_bottleneck b ON b.import_batch_id = params.import_batch_id AND b.user_key = l.user_key
GROUP BY params.analysis_run_id, params.import_batch_id, l.user_key, l.user_type, l.lead_type, l.demand_score, l.migration_motive_score, l.recommended_offer;

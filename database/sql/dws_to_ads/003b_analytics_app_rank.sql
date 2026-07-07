DELETE FROM :ads_app_experience_rank WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_app_experience_rank (
  analysis_run_id, import_batch_id, app_category, app_name, user_type,
  active_users, traffic_gb, duration_hours, avg_effective_mbps,
  avg_vmos, avg_mos, main_issue_driver, evidence_summary
)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id)
SELECT
  params.analysis_run_id,
  params.import_batch_id,
  COALESCE(a.app_category, 'UNKNOWN'),
  COALESCE(a.app_category, 'UNKNOWN'),
  COALESCE(a.user_type, 'UNKNOWN'),
  SUM(a.active_users),
  SUM(a.total_download_gb),
  SUM(a.total_game_hours),
  AVG(a.avg_effective_download_mbps),
  AVG(a.avg_vmos),
  AVG(a.avg_mos),
  CASE
    WHEN AVG(a.avg_vmos) < 3.5 THEN 'low_vmos'
    WHEN AVG(a.avg_mos) < 3.5 THEN 'low_mos'
    ELSE 'normal'
  END,
  CONCAT('source=dws_app_category_daily; rows=', COUNT(*))
FROM :dws_app_category_daily a, params
WHERE a.import_batch_id = params.import_batch_id
GROUP BY params.analysis_run_id, params.import_batch_id, COALESCE(a.app_category, 'UNKNOWN'), COALESCE(a.user_type, 'UNKNOWN');

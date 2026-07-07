-- Analytics KPI ADS summary.

REPLACE INTO ads_dashboard_kpi_summary (analysis_run_id, import_batch_id, kpi_group, kpi_key, kpi_label, kpi_value, kpi_unit, kpi_hint, display_order)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id),
p AS (SELECT * FROM dws_user_daily_profile WHERE import_batch_id = (SELECT import_batch_id FROM params)),
l AS (SELECT * FROM ads_migration_lead_user WHERE analysis_run_id = (SELECT analysis_run_id FROM params))
SELECT analysis_run_id, import_batch_id, 'overview', 'total_users', 'Total Users', COUNT(DISTINCT user_key), 'users', 'distinct DWS users', 10 FROM p, params
UNION ALL SELECT analysis_run_id, import_batch_id, 'overview', 'cable_users', 'Cable Users', COUNT(DISTINCT CASE WHEN user_type='CABLE' THEN user_key END), 'users', 'Cable users', 20 FROM p, params
UNION ALL SELECT analysis_run_id, import_batch_id, 'overview', 'ftth_users', 'FTTH Users', COUNT(DISTINCT CASE WHEN user_type='FTTH' THEN user_key END), 'users', 'FTTH users', 30 FROM p, params
UNION ALL SELECT analysis_run_id, import_batch_id, 'traffic', 'total_traffic_gb', 'Total Traffic', COALESCE(ROUND(SUM(total_download_gb),2),0), 'GB', 'sum total_download_gb', 40 FROM p, params
UNION ALL SELECT analysis_run_id, import_batch_id, 'game', 'game_hours', 'Game Hours', COALESCE(ROUND(SUM(total_game_hours),2),0), 'hours', 'sum total_game_hours', 50 FROM p, params
UNION ALL SELECT analysis_run_id, import_batch_id, 'quality', 'poor_experience_users', 'Poor Experience Users', COUNT(DISTINCT CASE WHEN COALESCE(avg_vmos,5)<3.5 OR COALESCE(avg_mos,5)<3.5 OR COALESCE(avg_subscriber_rtt_ms,0)>100 OR COALESCE(avg_user_down_loss,0)>2 THEN user_key END), 'users', 'basic QoE risk rules', 60 FROM p, params
UNION ALL SELECT analysis_run_id, import_batch_id, 'lead', 'sa_lead_users', 'SA Lead Users', COUNT(DISTINCT user_key), 'users', 'ads_migration_lead_user rows', 70 FROM l, params;

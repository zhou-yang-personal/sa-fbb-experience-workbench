-- Complete ADS dashboard generation.

DELETE FROM ads_dashboard_overview WHERE analysis_run_id = :analysis_run_id;
DELETE FROM ads_app_category_detail WHERE analysis_run_id = :analysis_run_id;
DELETE FROM ads_experience_quality_summary WHERE analysis_run_id = :analysis_run_id;
DELETE FROM ads_cable_fiber_compare WHERE analysis_run_id = :analysis_run_id;

INSERT INTO ads_dashboard_overview (analysis_run_id, metric_key, metric_label, metric_value, metric_hint)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id), p AS (
  SELECT * FROM dws_user_daily_profile WHERE dws_user_daily_profile.import_batch_id = (SELECT import_batch_id FROM params)
)
SELECT MAX(params.analysis_run_id), 'clean_users', 'Clean Users', CAST(COUNT(DISTINCT user_key) AS CHAR), 'DWS distinct users' FROM p, params
UNION ALL
SELECT MAX(params.analysis_run_id), 'video_gb', 'Video GB', CAST(COALESCE(ROUND(SUM(total_download_gb),2),0) AS CHAR), 'total_download_gb' FROM p, params
UNION ALL
SELECT MAX(params.analysis_run_id), 'game_hours', 'Game Hours', CAST(COALESCE(ROUND(SUM(total_game_hours),2),0) AS CHAR), 'total_game_hours' FROM p, params
UNION ALL
SELECT MAX(params.analysis_run_id), 'cable_users', 'Cable Users', CAST(COUNT(DISTINCT CASE WHEN user_type='CABLE' THEN user_key END) AS CHAR), 'Cable user count' FROM p, params;

INSERT INTO ads_app_category_detail (analysis_run_id, app_category, user_type, active_users, total_download_gb, total_game_hours, avg_experience_score)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id)
SELECT analysis_run_id, COALESCE(app_category, 'UNKNOWN') AS app_category, COALESCE(user_type, 'UNKNOWN') AS user_type,
       SUM(active_users), SUM(total_download_gb), SUM(total_game_hours), AVG(COALESCE(avg_vmos, avg_mos))
FROM dws_app_category_daily, params
WHERE dws_app_category_daily.import_batch_id = (SELECT import_batch_id FROM params)
GROUP BY analysis_run_id, COALESCE(app_category, 'UNKNOWN'), COALESCE(user_type, 'UNKNOWN');

INSERT INTO ads_experience_quality_summary (analysis_run_id, quality_dimension, user_type, affected_users, avg_value, p90_value, severity)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id), hourly AS (
  SELECT * FROM dws_access_type_hourly_compare WHERE dws_access_type_hourly_compare.import_batch_id = (SELECT import_batch_id FROM params)
)
SELECT analysis_run_id, 'subscriber_rtt', COALESCE(user_type, 'UNKNOWN'), SUM(active_users), AVG(avg_subscriber_rtt_ms), MAX(avg_subscriber_rtt_ms),
       CASE WHEN AVG(avg_subscriber_rtt_ms) >= 50 THEN 'warning' ELSE 'normal' END
FROM hourly, params GROUP BY analysis_run_id, COALESCE(user_type, 'UNKNOWN')
UNION ALL
SELECT analysis_run_id, 'user_down_loss', COALESCE(user_type, 'UNKNOWN'), SUM(active_users), AVG(avg_user_down_loss), MAX(avg_user_down_loss),
       CASE WHEN AVG(avg_user_down_loss) >= 1 THEN 'warning' ELSE 'normal' END
FROM hourly, params GROUP BY analysis_run_id, COALESCE(user_type, 'UNKNOWN')
UNION ALL
SELECT analysis_run_id, 'download_mbps', COALESCE(user_type, 'UNKNOWN'), SUM(active_users), AVG(avg_download_mbps), MAX(avg_download_mbps),
       CASE WHEN AVG(avg_download_mbps) < 20 THEN 'warning' ELSE 'normal' END
FROM hourly, params GROUP BY analysis_run_id, COALESCE(user_type, 'UNKNOWN');

INSERT INTO ads_cable_fiber_compare (analysis_run_id, stat_date, hour_of_day, metric_key, cable_value, ftth_value, diff_value)
WITH params AS (SELECT :analysis_run_id AS analysis_run_id, :import_batch_id AS import_batch_id), hourly AS (
  SELECT * FROM dws_access_type_hourly_compare WHERE dws_access_type_hourly_compare.import_batch_id = (SELECT import_batch_id FROM params)
), pivoted AS (
  SELECT stat_date, hour_of_day,
         MAX(CASE WHEN user_type='CABLE' THEN avg_subscriber_rtt_ms END) AS cable_rtt,
         MAX(CASE WHEN user_type='FTTH' THEN avg_subscriber_rtt_ms END) AS ftth_rtt,
         MAX(CASE WHEN user_type='CABLE' THEN avg_user_down_loss END) AS cable_loss,
         MAX(CASE WHEN user_type='FTTH' THEN avg_user_down_loss END) AS ftth_loss,
         MAX(CASE WHEN user_type='CABLE' THEN avg_download_mbps END) AS cable_down,
         MAX(CASE WHEN user_type='FTTH' THEN avg_download_mbps END) AS ftth_down
  FROM hourly
  WHERE stat_date IS NOT NULL AND hour_of_day IS NOT NULL
  GROUP BY stat_date, hour_of_day
)
SELECT analysis_run_id, stat_date, hour_of_day, 'subscriber_rtt', cable_rtt, ftth_rtt, cable_rtt - ftth_rtt FROM pivoted, params
UNION ALL
SELECT analysis_run_id, stat_date, hour_of_day, 'user_down_loss', cable_loss, ftth_loss, cable_loss - ftth_loss FROM pivoted, params
UNION ALL
SELECT analysis_run_id, stat_date, hour_of_day, 'download_mbps', cable_down, ftth_down, cable_down - ftth_down FROM pivoted, params;

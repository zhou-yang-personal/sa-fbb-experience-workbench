-- Complete DWS aggregates for dashboards.

REPLACE INTO dws_app_category_daily (
  import_batch_id, stat_date, user_type, app_category, active_users, total_download_gb, total_game_hours, avg_vmos, avg_mos, avg_effective_download_mbps
)
WITH params AS (SELECT :import_batch_id AS import_batch_id), video AS (
  SELECT import_batch_id, stat_date, COALESCE(user_type, 'UNKNOWN') AS user_type, COALESCE(app_category, 'UNKNOWN') AS app_category,
         COUNT(DISTINCT user_key) AS active_users,
         SUM(COALESCE(downloaded_gb,0)) AS total_download_gb,
         0 AS total_game_hours,
         AVG(vmos) AS avg_vmos,
         NULL AS avg_mos,
         AVG(effective_download_mbps) AS avg_effective_download_mbps
  FROM dwd_tcp_detail_clean
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
    AND stat_date IS NOT NULL
    AND user_key IS NOT NULL
    AND TRIM(user_key) <> ''
  GROUP BY import_batch_id, stat_date, COALESCE(user_type, 'UNKNOWN'), COALESCE(app_category, 'UNKNOWN')
), game AS (
  SELECT import_batch_id, stat_date, COALESCE(user_type, 'UNKNOWN') AS user_type, COALESCE(app_category, 'UNKNOWN') AS app_category,
         COUNT(DISTINCT user_key) AS active_users,
         0 AS total_download_gb,
         SUM(COALESCE(game_hours,0)) AS total_game_hours,
         NULL AS avg_vmos,
         AVG(mos) AS avg_mos,
         NULL AS avg_effective_download_mbps
  FROM dwd_game_detail_clean
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
    AND stat_date IS NOT NULL
    AND user_key IS NOT NULL
    AND TRIM(user_key) <> ''
  GROUP BY import_batch_id, stat_date, COALESCE(user_type, 'UNKNOWN'), COALESCE(app_category, 'UNKNOWN')
)
SELECT * FROM video
UNION ALL
SELECT * FROM game;

REPLACE INTO dws_access_type_hourly_compare (
  import_batch_id, stat_date, hour_of_day, user_type, active_users, avg_vmos, avg_mos, avg_subscriber_rtt_ms, avg_network_rtt_ms, avg_user_down_loss, avg_network_down_loss, avg_download_mbps
)
WITH params AS (SELECT :import_batch_id AS import_batch_id), video AS (
  SELECT import_batch_id, stat_date, hour_of_day, COALESCE(user_type, 'UNKNOWN') AS user_type,
         COUNT(DISTINCT user_key) AS active_users,
         AVG(vmos) AS avg_vmos,
         NULL AS avg_mos,
         AVG(subscriber_side_rtt_ms) AS avg_subscriber_rtt_ms,
         AVG(network_side_rtt_ms) AS avg_network_rtt_ms,
         AVG(user_down_loss) AS avg_user_down_loss,
         AVG(network_down_loss) AS avg_network_down_loss,
         AVG(effective_download_mbps) AS avg_download_mbps
  FROM dwd_tcp_detail_clean
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
    AND stat_date IS NOT NULL
    AND hour_of_day IS NOT NULL
    AND user_key IS NOT NULL
    AND TRIM(user_key) <> ''
  GROUP BY import_batch_id, stat_date, hour_of_day, COALESCE(user_type, 'UNKNOWN')
), game AS (
  SELECT import_batch_id, stat_date, hour_of_day, COALESCE(user_type, 'UNKNOWN') AS user_type,
         COUNT(DISTINCT user_key) AS active_users,
         NULL AS avg_vmos,
         AVG(mos) AS avg_mos,
         AVG(worst_latency_ms) AS avg_subscriber_rtt_ms,
         NULL AS avg_network_rtt_ms,
         AVG(worst_loss) AS avg_user_down_loss,
         NULL AS avg_network_down_loss,
         NULL AS avg_download_mbps
  FROM dwd_game_detail_clean
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
    AND stat_date IS NOT NULL
    AND hour_of_day IS NOT NULL
    AND user_key IS NOT NULL
    AND TRIM(user_key) <> ''
  GROUP BY import_batch_id, stat_date, hour_of_day, COALESCE(user_type, 'UNKNOWN')
)
SELECT * FROM video
UNION ALL
SELECT * FROM game;

REPLACE INTO dws_user_experience_bottleneck (import_batch_id, user_key, bottleneck_type, severity_score, evidence)
WITH params AS (SELECT :import_batch_id AS import_batch_id), profile AS (
  SELECT * FROM dws_user_daily_profile WHERE import_batch_id = (SELECT import_batch_id FROM params)
)
SELECT import_batch_id, user_key,
       CASE
         WHEN avg_network_down_loss >= 2 OR avg_network_rtt_ms >= 100 THEN 'NETWORK_SIDE_SEVERE'
         WHEN avg_user_down_loss >= 1 OR avg_subscriber_rtt_ms >= 50 THEN 'USER_SIDE_OR_WIFI_PRESSURE'
         WHEN peak_row_pct >= 35 THEN 'PEAK_HOUR_PRESSURE'
         ELSE 'APP_DEMAND_HIGH_BUT_EXPERIENCE_OK'
       END AS bottleneck_type,
       CAST(LEAST(100,
         COALESCE(avg_network_down_loss,0) * 20 + COALESCE(avg_user_down_loss,0) * 15 +
         COALESCE(avg_network_rtt_ms,0) / 2 + COALESCE(avg_subscriber_rtt_ms,0) / 2 +
         COALESCE(peak_row_pct,0) / 2
       ) AS SIGNED) AS severity_score,
       CONCAT('network_rtt=', COALESCE(avg_network_rtt_ms,0), ', subscriber_rtt=', COALESCE(avg_subscriber_rtt_ms,0), ', peak_pct=', COALESCE(peak_row_pct,0)) AS evidence
FROM profile
WHERE user_key IS NOT NULL AND TRIM(user_key) <> '';

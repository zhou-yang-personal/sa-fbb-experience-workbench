-- CLEAN/DWD → DWS user daily profile baseline

REPLACE INTO dws_user_daily_profile (
  import_batch_id,
  user_key,
  user_type,
  stat_date,
  video_rows,
  game_rows,
  total_download_gb,
  total_game_hours,
  avg_vmos,
  avg_mos,
  avg_subscriber_rtt_ms,
  avg_network_rtt_ms,
  avg_user_down_loss,
  avg_network_down_loss,
  peak_row_pct
)
WITH params AS (
  SELECT :import_batch_id AS import_batch_id
), video AS (
  SELECT
    import_batch_id,
    user_key,
    COALESCE(MAX(user_type), 'UNKNOWN') AS user_type,
    stat_date,
    COUNT(*) AS video_rows,
    SUM(COALESCE(downloaded_gb, 0)) AS total_download_gb,
    AVG(vmos) AS avg_vmos,
    AVG(subscriber_side_rtt_ms) AS avg_subscriber_rtt_ms,
    AVG(network_side_rtt_ms) AS avg_network_rtt_ms,
    AVG(user_down_loss) AS avg_user_down_loss,
    AVG(network_down_loss) AS avg_network_down_loss,
    SUM(CASE WHEN hour_of_day BETWEEN 18 AND 22 THEN 1 ELSE 0 END) / COUNT(*) * 100 AS peak_row_pct
  FROM dwd_tcp_detail_clean
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
    AND user_key IS NOT NULL
    AND TRIM(user_key) <> ''
    AND user_key <> 'UNKNOWN'
    AND stat_date IS NOT NULL
  GROUP BY import_batch_id, user_key, stat_date
), game AS (
  SELECT
    import_batch_id,
    user_key,
    COALESCE(MAX(user_type), 'UNKNOWN') AS user_type,
    stat_date,
    COUNT(*) AS game_rows,
    SUM(COALESCE(game_hours, 0)) AS total_game_hours,
    AVG(mos) AS avg_mos
  FROM dwd_game_detail_clean
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
    AND user_key IS NOT NULL
    AND TRIM(user_key) <> ''
    AND user_key <> 'UNKNOWN'
    AND stat_date IS NOT NULL
  GROUP BY import_batch_id, user_key, stat_date
), combined AS (
  SELECT
    COALESCE(v.import_batch_id, g.import_batch_id) AS import_batch_id,
    COALESCE(v.user_key, g.user_key) AS user_key,
    COALESCE(v.user_type, g.user_type, 'UNKNOWN') AS user_type,
    COALESCE(v.stat_date, g.stat_date) AS stat_date,
    COALESCE(v.video_rows, 0) AS video_rows,
    COALESCE(g.game_rows, 0) AS game_rows,
    COALESCE(v.total_download_gb, 0) AS total_download_gb,
    COALESCE(g.total_game_hours, 0) AS total_game_hours,
    v.avg_vmos,
    g.avg_mos,
    v.avg_subscriber_rtt_ms,
    v.avg_network_rtt_ms,
    v.avg_user_down_loss,
    v.avg_network_down_loss,
    v.peak_row_pct
  FROM video v
  LEFT JOIN game g ON g.import_batch_id = v.import_batch_id AND g.user_key = v.user_key AND g.stat_date = v.stat_date
  UNION ALL
  SELECT
    g.import_batch_id,
    g.user_key,
    g.user_type,
    g.stat_date,
    0,
    g.game_rows,
    0,
    g.total_game_hours,
    NULL,
    g.avg_mos,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL
  FROM game g
  LEFT JOIN video v ON v.import_batch_id = g.import_batch_id AND v.user_key = g.user_key AND v.stat_date = g.stat_date
  WHERE v.user_key IS NULL
)
SELECT * FROM combined;

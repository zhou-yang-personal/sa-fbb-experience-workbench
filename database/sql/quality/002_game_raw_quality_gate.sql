INSERT INTO meta_quality_check_result (import_batch_id, check_section, check_item, metric_name, metric_value, metric_text, severity, passed)
WITH params AS (SELECT :import_batch_id AS import_batch_id), batch AS (
  SELECT import_batch_id, data_type, total_rows, imported_rows
  FROM meta_import_batch
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
), raw_normalized AS (
  SELECT
    r.*,
    NULLIF(TRIM(REGEXP_REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(r.statistical_time, ''), CHAR(9), ' '), CHAR(10), ' '), CHAR(13), ' '), CONVERT(0xC2A0 USING utf8mb4), ' '), '[[:space:]]+', ' ')), '') AS stat_time_text
  FROM raw_game_detail_import r
  JOIN params p ON p.import_batch_id = r.import_batch_id
), raw_parsed AS (
  SELECT
    r.*,
    CASE
      WHEN r.stat_time_text REGEXP '^[0-9]{1,2}/[0-9]{1,2}/[0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%d/%m/%Y %H:%i:%s')
      WHEN r.stat_time_text REGEXP '^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%Y-%m-%d %H:%i:%s')
      WHEN r.stat_time_text REGEXP '^[0-9]{1,2}/[0-9]{1,2}/[0-9]{4} [0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%d/%m/%Y %H:%i')
      WHEN r.stat_time_text REGEXP '^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%Y-%m-%d %H:%i')
      ELSE NULL
    END AS parsed_stat_time
  FROM raw_normalized r
), raw_counts AS (
  SELECT 'game' AS data_type,
         COUNT(*) AS row_cnt,
         COUNT(DISTINCT NULLIF(TRIM(user_account), '')) AS user_account_cnt,
         COUNT(DISTINCT NULLIF(TRIM(user_mac), '')) AS user_mac_cnt,
         COUNT(DISTINCT NULLIF(TRIM(application_protocol), '')) AS app_cnt,
         MIN(parsed_stat_time) AS min_time,
         MAX(parsed_stat_time) AS max_time,
         COUNT(DISTINCT HOUR(parsed_stat_time)) AS active_hours,
         COALESCE(SUM(CASE WHEN stat_time_text IS NOT NULL AND parsed_stat_time IS NULL THEN 1 ELSE 0 END), 0) AS invalid_time_rows,
         COALESCE(SUM(CASE WHEN NULLIF(TRIM(user_account), '') IS NULL THEN 1 ELSE 0 END), 0) AS empty_account_rows,
         COALESCE(SUM(CASE WHEN UPPER(TRIM(COALESCE(user_type, ''))) LIKE '%CABLE%' OR UPPER(TRIM(COALESCE(wan_type, ''))) LIKE '%CABLE%' THEN 1 ELSE 0 END), 0) AS cable_rows,
         COALESCE(SUM(CASE WHEN UPPER(TRIM(COALESCE(user_type, ''))) LIKE '%FTTH%' OR UPPER(TRIM(COALESCE(user_type, ''))) LIKE '%FIBER%' THEN 1 ELSE 0 END), 0) AS ftth_rows,
         COALESCE(SUM(CASE WHEN NULLIF(TRIM(bras), '') IS NULL OR UPPER(TRIM(bras)) = 'UNKNOWN' THEN 1 ELSE 0 END), 0) AS unknown_bras_rows,
         COALESCE(SUM(CASE WHEN NULLIF(TRIM(olt), '') IS NULL OR UPPER(TRIM(olt)) = 'UNKNOWN' THEN 1 ELSE 0 END), 0) AS unknown_olt_rows,
         COALESCE(SUM(CASE WHEN NULLIF(TRIM(pon), '') IS NULL OR UPPER(TRIM(pon)) = 'UNKNOWN' THEN 1 ELSE 0 END), 0) AS unknown_pon_rows,
         COALESCE(SUM(CASE WHEN NULLIF(TRIM(wan_type), '') IS NULL OR UPPER(TRIM(wan_type)) = 'UNKNOWN' THEN 1 ELSE 0 END), 0) AS unknown_wan_rows
  FROM raw_parsed
), clean_counts AS (
  SELECT 'game' AS data_type,
         COUNT(*) AS clean_rows,
         COALESCE(SUM(CASE WHEN data_quality_flag = 'WARN_UNKNOWN_USER_KEY' THEN 1 ELSE 0 END), 0) AS warn_unknown_user_key_rows,
         COALESCE(SUM(CASE WHEN data_quality_flag = 'WARN_INVALID_STAT_TIME' THEN 1 ELSE 0 END), 0) AS warn_invalid_stat_time_rows,
         COALESCE(SUM(CASE WHEN data_quality_flag = 'WARN_UNKNOWN_ACCESS_TYPE' THEN 1 ELSE 0 END), 0) AS warn_unknown_access_type_rows,
         COALESCE(SUM(CASE WHEN data_quality_flag = 'OK' THEN 1 ELSE 0 END), 0) AS ok_rows
  FROM dwd_game_detail_clean WHERE import_batch_id = (SELECT import_batch_id FROM params)
)
SELECT import_batch_id, 'raw_quality', 'game_row_count', 'row_cnt', row_cnt, NULL,
       CASE WHEN row_cnt = 0 THEN 'error' ELSE 'info' END,
       CASE WHEN row_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT p.import_batch_id, 'raw_quality', 'game_csv_vs_raw_rows', 'row_diff', COALESCE(b.total_rows, 0) - COALESCE(b.imported_rows, 0),
       CONCAT('total_rows=', COALESCE(b.total_rows, 0), ', imported_rows=', COALESCE(b.imported_rows, 0)),
       CASE WHEN COALESCE(b.total_rows, 0) <> COALESCE(b.imported_rows, 0) THEN 'warning' ELSE 'info' END,
       CASE WHEN COALESCE(b.total_rows, 0) <> COALESCE(b.imported_rows, 0) THEN 0 ELSE 1 END
FROM batch b, params p
UNION ALL
SELECT import_batch_id, 'raw_quality', 'game_identity', 'user_account_cnt', user_account_cnt, CONCAT('user_mac_cnt=', user_mac_cnt),
       CASE WHEN user_account_cnt = 0 AND user_mac_cnt = 0 THEN 'error' ELSE 'info' END,
       CASE WHEN user_account_cnt = 0 AND user_mac_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', 'game_empty_account_pct', 'empty_account_pct',
       CASE WHEN row_cnt = 0 THEN 100 ELSE ROUND(empty_account_rows / row_cnt * 100, 2) END, NULL,
       CASE WHEN row_cnt = 0 OR empty_account_rows / row_cnt > 0.5 THEN 'warning' ELSE 'info' END,
       CASE WHEN row_cnt = 0 OR empty_account_rows / row_cnt > 0.8 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', 'game_access_type_mix', 'cable_rows', cable_rows, CONCAT('ftth_rows=', ftth_rows),
       CASE WHEN cable_rows = 0 OR ftth_rows = 0 THEN 'warning' ELSE 'info' END,
       CASE WHEN cable_rows = 0 OR ftth_rows = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', 'game_time_range', 'active_hours', active_hours, CONCAT('min_time=', COALESCE(CAST(min_time AS CHAR), 'NULL'), ', max_time=', COALESCE(CAST(max_time AS CHAR), 'NULL'), ', invalid_time_rows=', invalid_time_rows),
       CASE WHEN active_hours = 0 THEN 'error' WHEN active_hours < 6 OR invalid_time_rows > 0 THEN 'warning' ELSE 'info' END,
       CASE WHEN active_hours = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', 'game_app_count', 'app_cnt', app_cnt, NULL,
       CASE WHEN app_cnt = 0 THEN 'error' WHEN app_cnt < 3 THEN 'warning' ELSE 'info' END,
       CASE WHEN app_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', 'game_topology_unknown_pct', 'unknown_bras_pct',
       CASE WHEN row_cnt = 0 THEN 100 ELSE ROUND(unknown_bras_rows / row_cnt * 100, 2) END,
       CONCAT('unknown_olt_pct=', CASE WHEN row_cnt = 0 THEN 100 ELSE ROUND(unknown_olt_rows / row_cnt * 100, 2) END,
              ', unknown_pon_pct=', CASE WHEN row_cnt = 0 THEN 100 ELSE ROUND(unknown_pon_rows / row_cnt * 100, 2) END,
              ', unknown_wan_pct=', CASE WHEN row_cnt = 0 THEN 100 ELSE ROUND(unknown_wan_rows / row_cnt * 100, 2) END),
       CASE WHEN row_cnt = 0 OR unknown_bras_rows / row_cnt > 0.8 THEN 'warning' ELSE 'info' END,
       CASE WHEN row_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'clean_quality', 'game_clean_row_count', 'clean_rows', clean_rows,
       CONCAT('ok_rows=', ok_rows),
       CASE WHEN clean_rows = 0 THEN 'warning' ELSE 'info' END,
       1
FROM clean_counts, params
UNION ALL
SELECT import_batch_id, 'clean_quality', 'game_unknown_user_key_pct', 'warn_unknown_user_key_pct',
       CASE WHEN clean_rows = 0 THEN 0 ELSE ROUND(warn_unknown_user_key_rows / clean_rows * 100, 2) END,
       CONCAT('warn_unknown_user_key_rows=', warn_unknown_user_key_rows),
       CASE WHEN clean_rows > 0 AND warn_unknown_user_key_rows / clean_rows > 0.05 THEN 'warning' ELSE 'info' END,
       CASE WHEN clean_rows > 0 AND warn_unknown_user_key_rows / clean_rows > 0.2 THEN 0 ELSE 1 END
FROM clean_counts, params
UNION ALL
SELECT import_batch_id, 'clean_quality', 'game_invalid_stat_time_pct', 'warn_invalid_stat_time_pct',
       CASE WHEN clean_rows = 0 THEN 0 ELSE ROUND(warn_invalid_stat_time_rows / clean_rows * 100, 2) END,
       CONCAT('warn_invalid_stat_time_rows=', warn_invalid_stat_time_rows),
       CASE WHEN clean_rows > 0 AND warn_invalid_stat_time_rows / clean_rows > 0.01 THEN 'warning' ELSE 'info' END,
       CASE WHEN clean_rows > 0 AND warn_invalid_stat_time_rows / clean_rows > 0.05 THEN 0 ELSE 1 END
FROM clean_counts, params
UNION ALL
SELECT import_batch_id, 'clean_quality', 'game_unknown_access_type_pct', 'warn_unknown_access_type_pct',
       CASE WHEN clean_rows = 0 THEN 0 ELSE ROUND(warn_unknown_access_type_rows / clean_rows * 100, 2) END,
       CONCAT('warn_unknown_access_type_rows=', warn_unknown_access_type_rows),
       CASE WHEN clean_rows > 0 AND warn_unknown_access_type_rows / clean_rows > 0.3 THEN 'warning' ELSE 'info' END,
       CASE WHEN clean_rows > 0 AND warn_unknown_access_type_rows / clean_rows > 0.6 THEN 0 ELSE 1 END
FROM clean_counts, params;

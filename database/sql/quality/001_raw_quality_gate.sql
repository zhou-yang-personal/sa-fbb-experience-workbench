-- RAW quality gate for TCP and Game batches.
-- Runtime must bind :import_batch_id before execution.

DELETE FROM meta_quality_check_result WHERE import_batch_id = :import_batch_id;

INSERT INTO meta_quality_check_result (import_batch_id, check_section, check_item, metric_name, metric_value, metric_text, severity, passed)
WITH params AS (SELECT :import_batch_id AS import_batch_id), batch AS (
  SELECT import_batch_id, data_type, total_rows, imported_rows
  FROM meta_import_batch
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
), raw_counts AS (
  SELECT 'tcp' AS data_type,
         COUNT(*) AS row_cnt,
         COUNT(DISTINCT user_account) AS user_account_cnt,
         COUNT(DISTINCT user_mac) AS user_mac_cnt,
         COUNT(DISTINCT universal_video_applications) AS app_cnt,
         MIN(STR_TO_DATE(statistics_duration, '%d/%m/%Y %H:%i:%s')) AS min_time,
         MAX(STR_TO_DATE(statistics_duration, '%d/%m/%Y %H:%i:%s')) AS max_time,
         COUNT(DISTINCT HOUR(STR_TO_DATE(statistics_duration, '%d/%m/%Y %H:%i:%s'))) AS active_hours,
         SUM(CASE WHEN user_account IS NULL OR user_account = '' THEN 1 ELSE 0 END) AS empty_account_rows,
         SUM(CASE WHEN UPPER(user_type) = 'CABLE' THEN 1 ELSE 0 END) AS cable_rows,
         SUM(CASE WHEN UPPER(user_type) = 'FTTH' THEN 1 ELSE 0 END) AS ftth_rows,
         SUM(CASE WHEN bras IS NULL OR UPPER(bras) = 'UNKNOWN' THEN 1 ELSE 0 END) AS unknown_bras_rows,
         SUM(CASE WHEN olt IS NULL OR UPPER(olt) = 'UNKNOWN' THEN 1 ELSE 0 END) AS unknown_olt_rows,
         SUM(CASE WHEN pon IS NULL OR UPPER(pon) = 'UNKNOWN' THEN 1 ELSE 0 END) AS unknown_pon_rows,
         SUM(CASE WHEN wan_type IS NULL OR UPPER(wan_type) = 'UNKNOWN' THEN 1 ELSE 0 END) AS unknown_wan_rows
  FROM raw_tcp_detail_import WHERE import_batch_id = (SELECT import_batch_id FROM params)
  UNION ALL
  SELECT 'game',
         COUNT(*),
         COUNT(DISTINCT user_account),
         COUNT(DISTINCT user_mac),
         COUNT(DISTINCT application_protocol),
         MIN(STR_TO_DATE(statistical_time, '%d/%m/%Y %H:%i:%s')),
         MAX(STR_TO_DATE(statistical_time, '%d/%m/%Y %H:%i:%s')),
         COUNT(DISTINCT HOUR(STR_TO_DATE(statistical_time, '%d/%m/%Y %H:%i:%s'))),
         SUM(CASE WHEN user_account IS NULL OR user_account = '' THEN 1 ELSE 0 END),
         SUM(CASE WHEN UPPER(user_type) = 'CABLE' THEN 1 ELSE 0 END),
         SUM(CASE WHEN UPPER(user_type) = 'FTTH' THEN 1 ELSE 0 END),
         SUM(CASE WHEN bras IS NULL OR UPPER(bras) = 'UNKNOWN' THEN 1 ELSE 0 END),
         SUM(CASE WHEN olt IS NULL OR UPPER(olt) = 'UNKNOWN' THEN 1 ELSE 0 END),
         SUM(CASE WHEN pon IS NULL OR UPPER(pon) = 'UNKNOWN' THEN 1 ELSE 0 END),
         SUM(CASE WHEN wan_type IS NULL OR UPPER(wan_type) = 'UNKNOWN' THEN 1 ELSE 0 END)
  FROM raw_game_detail_import WHERE import_batch_id = (SELECT import_batch_id FROM params)
)
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_row_count'), 'row_cnt', row_cnt, NULL,
       CASE WHEN row_cnt = 0 THEN 'error' ELSE 'info' END,
       CASE WHEN row_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT p.import_batch_id, 'raw_quality', CONCAT(b.data_type, '_csv_vs_raw_rows'), 'row_diff', COALESCE(b.total_rows, 0) - COALESCE(b.imported_rows, 0),
       CONCAT('total_rows=', COALESCE(b.total_rows, 0), ', imported_rows=', COALESCE(b.imported_rows, 0)),
       CASE WHEN COALESCE(b.total_rows, 0) <> COALESCE(b.imported_rows, 0) THEN 'warning' ELSE 'info' END,
       CASE WHEN COALESCE(b.total_rows, 0) <> COALESCE(b.imported_rows, 0) THEN 0 ELSE 1 END
FROM batch b, params p
UNION ALL
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_identity'), 'user_account_cnt', user_account_cnt, CONCAT('user_mac_cnt=', user_mac_cnt),
       CASE WHEN user_account_cnt = 0 AND user_mac_cnt = 0 THEN 'error' ELSE 'info' END,
       CASE WHEN user_account_cnt = 0 AND user_mac_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_empty_account_pct'), 'empty_account_pct',
       CASE WHEN row_cnt = 0 THEN 100 ELSE empty_account_rows / row_cnt * 100 END, NULL,
       CASE WHEN row_cnt = 0 OR empty_account_rows / row_cnt > 0.5 THEN 'warning' ELSE 'info' END,
       CASE WHEN row_cnt = 0 OR empty_account_rows / row_cnt > 0.8 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_access_type_mix'), 'cable_rows', cable_rows, CONCAT('ftth_rows=', ftth_rows),
       CASE WHEN cable_rows = 0 OR ftth_rows = 0 THEN 'warning' ELSE 'info' END,
       CASE WHEN cable_rows = 0 OR ftth_rows = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_time_range'), 'active_hours', active_hours, CONCAT('min_time=', COALESCE(CAST(min_time AS CHAR), 'NULL'), ', max_time=', COALESCE(CAST(max_time AS CHAR), 'NULL')),
       CASE WHEN active_hours = 0 THEN 'error' WHEN active_hours < 6 THEN 'warning' ELSE 'info' END,
       CASE WHEN active_hours = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_app_count'), 'app_cnt', app_cnt, NULL,
       CASE WHEN app_cnt = 0 THEN 'error' WHEN app_cnt < 3 THEN 'warning' ELSE 'info' END,
       CASE WHEN app_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_topology_unknown_pct'), 'unknown_bras_pct',
       CASE WHEN row_cnt = 0 THEN 100 ELSE unknown_bras_rows / row_cnt * 100 END,
       CONCAT('unknown_olt_pct=', CASE WHEN row_cnt = 0 THEN 100 ELSE ROUND(unknown_olt_rows / row_cnt * 100, 2) END,
              ', unknown_pon_pct=', CASE WHEN row_cnt = 0 THEN 100 ELSE ROUND(unknown_pon_rows / row_cnt * 100, 2) END,
              ', unknown_wan_pct=', CASE WHEN row_cnt = 0 THEN 100 ELSE ROUND(unknown_wan_rows / row_cnt * 100, 2) END),
       CASE WHEN row_cnt = 0 OR unknown_bras_rows / row_cnt > 0.8 THEN 'warning' ELSE 'info' END,
       CASE WHEN row_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params;

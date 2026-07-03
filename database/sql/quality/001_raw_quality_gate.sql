-- RAW quality gate for TCP and Game batches.
-- Runtime must bind :import_batch_id before execution.

DELETE FROM meta_quality_check_result WHERE import_batch_id = :import_batch_id;

INSERT INTO meta_quality_check_result (import_batch_id, check_section, check_item, metric_name, metric_value, metric_text, severity, passed)
WITH params AS (SELECT :import_batch_id AS import_batch_id), raw_counts AS (
  SELECT 'tcp' AS data_type, COUNT(*) AS row_cnt,
         COUNT(DISTINCT user_account) AS user_account_cnt,
         COUNT(DISTINCT user_mac) AS user_mac_cnt,
         SUM(CASE WHEN user_account IS NULL OR user_account = '' THEN 1 ELSE 0 END) AS empty_account_rows,
         SUM(CASE WHEN UPPER(user_type) = 'CABLE' THEN 1 ELSE 0 END) AS cable_rows,
         SUM(CASE WHEN UPPER(user_type) = 'FTTH' THEN 1 ELSE 0 END) AS ftth_rows,
         SUM(CASE WHEN bras IS NULL OR UPPER(bras) = 'UNKNOWN' THEN 1 ELSE 0 END) AS unknown_bras_rows
  FROM raw_tcp_detail_import WHERE import_batch_id = (SELECT import_batch_id FROM params)
  UNION ALL
  SELECT 'game', COUNT(*), COUNT(DISTINCT user_account), COUNT(DISTINCT user_mac),
         SUM(CASE WHEN user_account IS NULL OR user_account = '' THEN 1 ELSE 0 END),
         SUM(CASE WHEN UPPER(user_type) = 'CABLE' THEN 1 ELSE 0 END),
         SUM(CASE WHEN UPPER(user_type) = 'FTTH' THEN 1 ELSE 0 END),
         SUM(CASE WHEN bras IS NULL OR UPPER(bras) = 'UNKNOWN' THEN 1 ELSE 0 END)
  FROM raw_game_detail_import WHERE import_batch_id = (SELECT import_batch_id FROM params)
)
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_row_count'), 'row_cnt', row_cnt, NULL,
       CASE WHEN row_cnt = 0 THEN 'error' ELSE 'info' END,
       CASE WHEN row_cnt = 0 THEN 0 ELSE 1 END
FROM raw_counts, params
UNION ALL
SELECT import_batch_id, 'raw_quality', CONCAT(data_type, '_identity'), 'user_account_cnt', user_account_cnt, NULL,
       CASE WHEN user_account_cnt = 0 THEN 'error' ELSE 'info' END,
       CASE WHEN user_account_cnt = 0 THEN 0 ELSE 1 END
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
FROM raw_counts, params;

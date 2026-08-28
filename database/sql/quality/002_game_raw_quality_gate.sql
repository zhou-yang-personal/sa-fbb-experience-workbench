-- Post-clean quality gate for Game data; reuses normalized DWD columns.
INSERT INTO meta_quality_check_result (import_batch_id, check_section, check_item, metric_name, metric_value, metric_text, severity, passed)
WITH params AS (
  SELECT :import_batch_id AS import_batch_id
), batch AS (
  SELECT import_batch_id,total_rows,imported_rows FROM meta_import_batch
  WHERE import_batch_id=(SELECT import_batch_id FROM params)
), counts AS (
  SELECT
    COUNT(*) AS row_cnt,
    COUNT(DISTINCT NULLIF(NULLIF(TRIM(user_account),''),'--')) AS user_account_cnt,
    COUNT(DISTINCT NULLIF(NULLIF(TRIM(user_mac),''),'--')) AS user_mac_cnt,
    COUNT(DISTINCT NULLIF(NULLIF(TRIM(app_name),''),'UNKNOWN_APP')) AS app_cnt,
    MIN(stat_time) AS min_time,MAX(stat_time) AS max_time,COUNT(DISTINCT hour_of_day) AS active_hours,
    COALESCE(SUM(data_quality_flag='WARN_INVALID_STAT_TIME'),0) AS invalid_time_rows,
    COALESCE(SUM(NULLIF(NULLIF(TRIM(user_account),''),'--') IS NULL),0) AS empty_account_rows,
    COALESCE(SUM(user_type='CABLE'),0) AS cable_rows,COALESCE(SUM(user_type='FTTH'),0) AS ftth_rows,
    COALESCE(SUM(NULLIF(TRIM(bras),'') IS NULL OR UPPER(TRIM(bras))='UNKNOWN'),0) AS unknown_bras_rows,
    COALESCE(SUM(NULLIF(TRIM(olt),'') IS NULL OR UPPER(TRIM(olt))='UNKNOWN'),0) AS unknown_olt_rows,
    COALESCE(SUM(NULLIF(TRIM(pon),'') IS NULL OR UPPER(TRIM(pon))='UNKNOWN'),0) AS unknown_pon_rows,
    COALESCE(SUM(data_quality_flag='WARN_UNKNOWN_USER_KEY'),0) AS warn_unknown_user_key_rows,
    COALESCE(SUM(data_quality_flag='WARN_UNKNOWN_ACCESS_TYPE'),0) AS warn_unknown_access_type_rows,
    COALESCE(SUM(data_quality_flag='OK'),0) AS ok_rows
  FROM :dwd_game_detail_clean WHERE import_batch_id=(SELECT import_batch_id FROM params)
)
SELECT p.import_batch_id,'clean_quality','game_row_count','row_cnt',c.row_cnt,'source=normalized_dwd',CASE WHEN c.row_cnt=0 THEN 'error' ELSE 'info' END,CASE WHEN c.row_cnt=0 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'raw_quality','game_csv_vs_raw_rows','row_diff',COALESCE(b.total_rows,0)-COALESCE(b.imported_rows,0),CONCAT('total_rows=',COALESCE(b.total_rows,0),', imported_rows=',COALESCE(b.imported_rows,0)),CASE WHEN COALESCE(b.total_rows,0)<>COALESCE(b.imported_rows,0) THEN 'warning' ELSE 'info' END,CASE WHEN COALESCE(b.total_rows,0)<>COALESCE(b.imported_rows,0) THEN 0 ELSE 1 END FROM batch b CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_identity','user_account_cnt',c.user_account_cnt,CONCAT('user_mac_cnt=',c.user_mac_cnt),CASE WHEN c.user_account_cnt=0 AND c.user_mac_cnt=0 THEN 'error' ELSE 'info' END,CASE WHEN c.user_account_cnt=0 AND c.user_mac_cnt=0 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_empty_account_pct','empty_account_pct',CASE WHEN c.row_cnt=0 THEN 100 ELSE ROUND(c.empty_account_rows/c.row_cnt*100,2) END,NULL,CASE WHEN c.row_cnt=0 OR c.empty_account_rows/c.row_cnt>0.5 THEN 'warning' ELSE 'info' END,CASE WHEN c.row_cnt=0 OR c.empty_account_rows/c.row_cnt>0.8 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_access_type_mix','cable_rows',c.cable_rows,CONCAT('ftth_rows=',c.ftth_rows),CASE WHEN c.cable_rows=0 OR c.ftth_rows=0 THEN 'warning' ELSE 'info' END,CASE WHEN c.row_cnt=0 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_time_range','active_hours',c.active_hours,CONCAT('min_time=',COALESCE(CAST(c.min_time AS CHAR),'NULL'),', max_time=',COALESCE(CAST(c.max_time AS CHAR),'NULL'),', invalid_time_rows=',c.invalid_time_rows),CASE WHEN c.active_hours=0 THEN 'error' WHEN c.active_hours<6 OR c.invalid_time_rows>0 THEN 'warning' ELSE 'info' END,CASE WHEN c.active_hours=0 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_app_count','app_cnt',c.app_cnt,NULL,CASE WHEN c.app_cnt=0 THEN 'error' WHEN c.app_cnt<3 THEN 'warning' ELSE 'info' END,CASE WHEN c.app_cnt=0 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_topology_unknown_pct','unknown_bras_pct',CASE WHEN c.row_cnt=0 THEN 100 ELSE ROUND(c.unknown_bras_rows/c.row_cnt*100,2) END,CONCAT('unknown_olt_pct=',CASE WHEN c.row_cnt=0 THEN 100 ELSE ROUND(c.unknown_olt_rows/c.row_cnt*100,2) END,', unknown_pon_pct=',CASE WHEN c.row_cnt=0 THEN 100 ELSE ROUND(c.unknown_pon_rows/c.row_cnt*100,2) END),CASE WHEN c.row_cnt=0 OR c.unknown_bras_rows/c.row_cnt>0.8 THEN 'warning' ELSE 'info' END,CASE WHEN c.row_cnt=0 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_clean_row_count','clean_rows',c.row_cnt,CONCAT('ok_rows=',c.ok_rows),CASE WHEN c.row_cnt=0 THEN 'error' ELSE 'info' END,CASE WHEN c.row_cnt=0 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_unknown_user_key_pct','warn_unknown_user_key_pct',CASE WHEN c.row_cnt=0 THEN 0 ELSE ROUND(c.warn_unknown_user_key_rows/c.row_cnt*100,2) END,CONCAT('warn_unknown_user_key_rows=',c.warn_unknown_user_key_rows),CASE WHEN c.row_cnt>0 AND c.warn_unknown_user_key_rows/c.row_cnt>0.05 THEN 'warning' ELSE 'info' END,CASE WHEN c.row_cnt>0 AND c.warn_unknown_user_key_rows/c.row_cnt>0.2 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_invalid_stat_time_pct','warn_invalid_stat_time_pct',CASE WHEN c.row_cnt=0 THEN 0 ELSE ROUND(c.invalid_time_rows/c.row_cnt*100,2) END,CONCAT('warn_invalid_stat_time_rows=',c.invalid_time_rows),CASE WHEN c.row_cnt>0 AND c.invalid_time_rows/c.row_cnt>0.01 THEN 'warning' ELSE 'info' END,CASE WHEN c.row_cnt>0 AND c.invalid_time_rows/c.row_cnt>0.05 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p
UNION ALL
SELECT p.import_batch_id,'clean_quality','game_unknown_access_type_pct','warn_unknown_access_type_pct',CASE WHEN c.row_cnt=0 THEN 0 ELSE ROUND(c.warn_unknown_access_type_rows/c.row_cnt*100,2) END,CONCAT('warn_unknown_access_type_rows=',c.warn_unknown_access_type_rows),CASE WHEN c.row_cnt>0 AND c.warn_unknown_access_type_rows/c.row_cnt>0.3 THEN 'warning' ELSE 'info' END,CASE WHEN c.row_cnt>0 AND c.warn_unknown_access_type_rows/c.row_cnt>0.6 THEN 0 ELSE 1 END FROM counts c CROSS JOIN params p;

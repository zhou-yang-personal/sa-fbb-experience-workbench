-- RAW TCP → DWD TCP clean baseline
-- Parameters are expressed with CTE instead of SET @var statements.

DELETE FROM :dwd_tcp_detail_clean WHERE import_batch_id = :import_batch_id;

INSERT INTO :dwd_tcp_detail_clean (
  import_batch_id,
  user_key,
  key_confidence,
  user_account,
  user_mac,
  source_user_type,
  user_type,
  local_ip_address,
  server_ip,
  access_type_source,
  access_type_confidence,
  access_rule_id,
  access_rule_set_version,
  app_name,
  app_category,
  stat_time,
  stat_date,
  hour_of_day,
  downloaded_gb,
  effective_download_mbps,
  vmos,
  subscriber_side_rtt_ms,
  network_side_rtt_ms,
  user_down_loss,
  network_down_loss,
  wifi_delay_ms,
  bras,
  olt,
  pon,
  data_quality_flag
)
WITH params AS (
  SELECT :import_batch_id AS import_batch_id
), raw_normalized AS (
  SELECT
    r.*,
    NULLIF(TRIM(r.user_account), '') AS account_key,
    NULLIF(TRIM(r.user_mac), '') AS mac_key,
    NULLIF(TRIM(r.local_ip_address), '') AS ip_key,
    NULLIF(TRIM(REGEXP_REPLACE(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(r.statistics_duration, ''), CHAR(9), ' '), CHAR(10), ' '), CHAR(13), ' '), CONVERT(0xC2A0 USING utf8mb4), ' '), '[[:space:]]+', ' ')), '') AS stat_time_text
  FROM :raw_tcp_detail_import r
  JOIN params p ON p.import_batch_id = r.import_batch_id
), parsed AS (
  SELECT
    r.*,
    CASE
      WHEN r.stat_time_text REGEXP '^[0-9]{1,2}/[0-9]{1,2}/[0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%d/%m/%Y %H:%i:%s')
      WHEN r.stat_time_text REGEXP '^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%Y-%m-%d %H:%i:%s')
      WHEN r.stat_time_text REGEXP '^[0-9]{1,2}/[0-9]{1,2}/[0-9]{4} [0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%d/%m/%Y %H:%i')
      WHEN r.stat_time_text REGEXP '^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%Y-%m-%d %H:%i')
      ELSE NULL
    END AS parsed_stat_time,
    CASE
      WHEN UPPER(TRIM(COALESCE(r.user_type, ''))) LIKE '%FTTH%' OR UPPER(TRIM(COALESCE(r.user_type, ''))) LIKE '%FIBER%' THEN 'FTTH'
      WHEN UPPER(TRIM(COALESCE(r.user_type, ''))) LIKE '%CABLE%' OR UPPER(TRIM(COALESCE(r.wan_type, ''))) LIKE '%CABLE%' THEN 'CABLE'
      ELSE 'UNKNOWN'
    END AS source_user_type,
    CASE WHEN INET_ATON(r.account_key) IS NOT NULL THEN r.account_key WHEN INET_ATON(r.ip_key) IS NOT NULL THEN r.ip_key ELSE NULL END AS analysis_ip_key,
    COALESCE(INET_ATON(r.account_key), INET_ATON(r.ip_key)) AS ip_num
  FROM raw_normalized r
), normalized AS (
  SELECT
    r.import_batch_id,
    COALESCE(r.analysis_ip_key, 'UNKNOWN') AS user_key,
    CASE WHEN INET_ATON(r.account_key) IS NOT NULL THEN 'IP_USER_ACCOUNT' WHEN INET_ATON(r.ip_key) IS NOT NULL THEN 'IP_LOCAL_ADDRESS' ELSE 'IP_UNAVAILABLE' END AS key_confidence,
    r.account_key AS user_account,
    r.mac_key AS user_mac,
    r.source_user_type,
    CASE
      WHEN r.ip_num IS NULL THEN 'UNKNOWN'
      WHEN ar.rule_id IS NOT NULL THEN ar.access_type
      WHEN ars.default_access_type IN ('CABLE', 'FTTH', 'OTHER') THEN ars.default_access_type
      ELSE 'UNKNOWN'
    END AS user_type,
    r.analysis_ip_key AS local_ip_address,
    NULLIF(TRIM(r.server_ip), '') AS server_ip,
    CASE
      WHEN r.ip_num IS NULL THEN 'UNAVAILABLE_IP'
      WHEN ar.rule_id IS NOT NULL THEN 'IP_RULE'
      WHEN ars.default_access_type IN ('CABLE', 'FTTH', 'OTHER') THEN 'RULE_SET_OTHERS'
      ELSE 'UNMATCHED'
    END AS access_type_source,
    CASE
      WHEN r.ip_num IS NULL THEN 'LOW'
      WHEN ar.rule_id IS NOT NULL THEN 'HIGH'
      WHEN ars.default_access_type IN ('CABLE', 'FTTH', 'OTHER') THEN 'HIGH'
      ELSE 'LOW'
    END AS access_type_confidence,
    ar.rule_id AS access_rule_id,
    b.access_rule_set_version,
    COALESCE(NULLIF(TRIM(m.standard_app_name), ''), NULLIF(TRIM(r.universal_video_applications), ''), 'UNKNOWN_APP') AS app_name,
    COALESCE(NULLIF(TRIM(m.app_category), ''), 'other') AS app_category,
    r.parsed_stat_time AS stat_time,
    CAST(NULLIF(NULLIF(TRIM(r.downloaded_data_volume_kb), ''), '--') AS DECIMAL(24,6)) / 1024 / 1024 AS downloaded_gb,
    CAST(NULLIF(NULLIF(TRIM(r.user_avg_effective_download_rate_kbps), ''), '--') AS DECIMAL(18,6)) / 1000 AS effective_download_mbps,
    CAST(NULLIF(NULLIF(TRIM(r.vmos), ''), '--') AS DECIMAL(18,6)) AS vmos,
    CAST(NULLIF(NULLIF(TRIM(r.subscriber_side_rtt_ms), ''), '--') AS DECIMAL(18,6)) AS subscriber_side_rtt_ms,
    CAST(NULLIF(NULLIF(TRIM(r.network_side_rtt_ms), ''), '--') AS DECIMAL(18,6)) AS network_side_rtt_ms,
    CAST(NULLIF(NULLIF(TRIM(r.user_side_downstream_packet_loss_rate), ''), '--') AS DECIMAL(18,6)) AS user_down_loss,
    CAST(NULLIF(NULLIF(TRIM(r.network_side_downstream_packet_loss_rate), ''), '--') AS DECIMAL(18,6)) AS network_down_loss,
    CAST(NULLIF(NULLIF(TRIM(r.wifi_delay_ms), ''), '--') AS DECIMAL(18,6)) AS wifi_delay_ms,
    NULLIF(TRIM(r.bras), '') AS bras,
    NULLIF(TRIM(r.olt), '') AS olt,
    NULLIF(TRIM(r.pon), '') AS pon
  FROM parsed r
  LEFT JOIN dim_app_mapping m ON m.raw_app_name = r.universal_video_applications
  LEFT JOIN meta_import_batch b ON b.import_batch_id = r.import_batch_id
  LEFT JOIN meta_access_rule_set ars ON ars.rule_set_id = b.access_rule_set_id
  LEFT JOIN dim_access_ip_range ar ON ar.rule_set_id = b.access_rule_set_id AND ar.enabled = 1 AND r.ip_num BETWEEN ar.start_ip_num AND ar.end_ip_num
)
SELECT
  import_batch_id,
  user_key,
  key_confidence,
  user_account,
  user_mac,
  source_user_type,
  user_type,
  local_ip_address,
  server_ip,
  access_type_source,
  access_type_confidence,
  access_rule_id,
  access_rule_set_version,
  app_name,
  app_category,
  stat_time,
  DATE(stat_time),
  HOUR(stat_time),
  downloaded_gb,
  effective_download_mbps,
  vmos,
  subscriber_side_rtt_ms,
  network_side_rtt_ms,
  user_down_loss,
  network_down_loss,
  wifi_delay_ms,
  bras,
  olt,
  pon,
  CASE
    WHEN user_key = 'UNKNOWN' THEN 'WARN_UNKNOWN_USER_KEY'
    WHEN stat_time IS NULL THEN 'WARN_INVALID_STAT_TIME'
    WHEN user_type = 'UNKNOWN' THEN 'WARN_UNKNOWN_ACCESS_TYPE'
    ELSE 'OK'
  END AS data_quality_flag
FROM normalized
WHERE user_key IS NOT NULL AND TRIM(user_key) <> '';

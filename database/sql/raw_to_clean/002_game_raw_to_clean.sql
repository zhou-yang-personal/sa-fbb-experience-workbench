-- RAW Game → DWD Game clean baseline

DELETE FROM dwd_game_detail_clean WHERE import_batch_id = :import_batch_id;

INSERT INTO dwd_game_detail_clean (
  import_batch_id,
  user_key,
  key_confidence,
  user_account,
  user_mac,
  user_type,
  app_name,
  app_category,
  stat_time,
  stat_date,
  hour_of_day,
  game_hours,
  mos,
  worst_latency_ms,
  worst_loss,
  worst_jitter_ms,
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
    NULLIF(TRIM(REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(r.statistical_time, ''), CHAR(9), ''), CHAR(10), ''), CHAR(13), ''), CHAR(160), ' ')), '') AS stat_time_text
  FROM raw_game_detail_import r
  JOIN params p ON p.import_batch_id = r.import_batch_id
), parsed AS (
  SELECT
    r.*,
    CASE
      WHEN r.stat_time_text REGEXP '^[0-9]{2}/[0-9]{2}/[0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%d/%m/%Y %H:%i:%s')
      WHEN r.stat_time_text REGEXP '^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%Y-%m-%d %H:%i:%s')
      WHEN r.stat_time_text REGEXP '^[0-9]{2}/[0-9]{2}/[0-9]{4} [0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%d/%m/%Y %H:%i')
      WHEN r.stat_time_text REGEXP '^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}$' THEN STR_TO_DATE(r.stat_time_text, '%Y-%m-%d %H:%i')
      ELSE NULL
    END AS parsed_stat_time
  FROM raw_normalized r
), normalized AS (
  SELECT
    r.import_batch_id,
    CASE
      WHEN r.account_key IS NOT NULL AND r.account_key <> '--' AND r.account_key NOT REGEXP '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$' THEN r.account_key
      WHEN r.mac_key IS NOT NULL AND r.mac_key <> '--' THEN r.mac_key
      WHEN r.account_key REGEXP '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$' THEN r.account_key
      WHEN r.ip_key IS NOT NULL AND r.ip_key <> '--' THEN r.ip_key
      ELSE 'UNKNOWN'
    END AS user_key,
    CASE
      WHEN r.account_key IS NOT NULL AND r.account_key <> '--' AND r.account_key NOT REGEXP '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$' THEN 'HIGH_ACCOUNT_KEY'
      WHEN r.mac_key IS NOT NULL AND r.mac_key <> '--' THEN 'MEDIUM_MAC_USER_KEY'
      WHEN r.account_key REGEXP '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$' OR r.ip_key IS NOT NULL THEN 'LOW_IP_ONLY_KEY'
      ELSE 'UNKNOWN_KEY'
    END AS key_confidence,
    r.account_key AS user_account,
    r.mac_key AS user_mac,
    CASE
      WHEN UPPER(TRIM(COALESCE(r.user_type, ''))) LIKE '%FTTH%' OR UPPER(TRIM(COALESCE(r.user_type, ''))) LIKE '%FIBER%' THEN 'FTTH'
      WHEN UPPER(TRIM(COALESCE(r.user_type, ''))) LIKE '%CABLE%' OR UPPER(TRIM(COALESCE(r.wan_type, ''))) LIKE '%CABLE%' THEN 'CABLE'
      ELSE 'UNKNOWN'
    END AS user_type,
    COALESCE(NULLIF(TRIM(m.standard_app_name), ''), NULLIF(TRIM(r.application_protocol), ''), 'UNKNOWN_APP') AS app_name,
    COALESCE(NULLIF(TRIM(m.app_category), ''), 'game') AS app_category,
    r.parsed_stat_time AS stat_time,
    CAST(NULLIF(NULLIF(TRIM(r.game_duration_s), ''), '--') AS DECIMAL(18,6)) / 3600 AS game_hours,
    CAST(NULLIF(NULLIF(TRIM(r.mos), ''), '--') AS DECIMAL(18,6)) AS mos,
    GREATEST(
      COALESCE(CAST(NULLIF(NULLIF(TRIM(r.subscriber_side_rtt_ms), ''), '--') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(NULLIF(TRIM(r.network_side_rtt_ms), ''), '--') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(NULLIF(TRIM(r.downstream_data_transmission_rtt_ms), ''), '--') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(NULLIF(TRIM(r.upstream_data_transmission_rtt_ms), ''), '--') AS DECIMAL(18,6)), 0)
    ) AS worst_latency_ms,
    GREATEST(
      COALESCE(CAST(NULLIF(NULLIF(TRIM(r.user_side_downstream_packet_loss_rate), ''), '--') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(NULLIF(TRIM(r.network_side_downstream_packet_loss_rate), ''), '--') AS DECIMAL(18,6)), 0)
    ) AS worst_loss,
    GREATEST(
      COALESCE(CAST(NULLIF(NULLIF(TRIM(r.upstream_rtt_jitter_ms), ''), '--') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(NULLIF(TRIM(r.downstream_rtt_jitter_ms), ''), '--') AS DECIMAL(18,6)), 0)
    ) AS worst_jitter_ms,
    CAST(NULLIF(NULLIF(TRIM(r.wifi_delay_ms), ''), '--') AS DECIMAL(18,6)) AS wifi_delay_ms,
    NULLIF(TRIM(r.bras), '') AS bras,
    NULLIF(TRIM(r.olt), '') AS olt,
    NULLIF(TRIM(r.pon), '') AS pon
  FROM parsed r
  LEFT JOIN dim_app_mapping m ON m.raw_app_name = r.application_protocol
)
SELECT
  import_batch_id,
  user_key,
  key_confidence,
  user_account,
  user_mac,
  user_type,
  app_name,
  app_category,
  stat_time,
  DATE(stat_time),
  HOUR(stat_time),
  game_hours,
  mos,
  worst_latency_ms,
  worst_loss,
  worst_jitter_ms,
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

-- RAW TCP → DWD TCP clean baseline
-- Parameters are expressed with CTE instead of SET @var statements.

INSERT INTO dwd_tcp_detail_clean (
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
), normalized AS (
  SELECT
    r.import_batch_id,
    CASE
      WHEN r.user_account IS NOT NULL AND r.user_account <> '' AND r.user_account NOT REGEXP '^[0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+$' THEN r.user_account
      WHEN r.user_mac IS NOT NULL AND r.user_mac <> '' AND r.user_mac <> '--' THEN r.user_mac
      ELSE COALESCE(r.user_account, r.local_ip_address, 'UNKNOWN')
    END AS user_key,
    CASE
      WHEN r.user_account IS NOT NULL AND r.user_account <> '' AND r.user_account NOT REGEXP '^[0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+$' THEN 'HIGH_ACCOUNT_KEY'
      WHEN r.user_mac IS NOT NULL AND r.user_mac <> '' AND r.user_mac <> '--' THEN 'MEDIUM_MAC_USER_KEY'
      WHEN r.user_account REGEXP '^[0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+$' THEN 'LOW_IP_ONLY_KEY'
      ELSE 'UNKNOWN_KEY'
    END AS key_confidence,
    r.user_account,
    r.user_mac,
    UPPER(TRIM(r.user_type)) AS user_type,
    COALESCE(m.standard_app_name, r.universal_video_applications) AS app_name,
    COALESCE(m.app_category, 'other') AS app_category,
    STR_TO_DATE(r.statistics_duration, '%d/%m/%Y %H:%i:%s') AS stat_time,
    CAST(NULLIF(r.downloaded_data_volume_kb, '') AS DECIMAL(24,6)) / 1024 / 1024 AS downloaded_gb,
    CAST(NULLIF(r.user_avg_effective_download_rate_kbps, '') AS DECIMAL(18,6)) / 1000 AS effective_download_mbps,
    CAST(NULLIF(r.vmos, '') AS DECIMAL(18,6)) AS vmos,
    CAST(NULLIF(r.subscriber_side_rtt_ms, '') AS DECIMAL(18,6)) AS subscriber_side_rtt_ms,
    CAST(NULLIF(r.network_side_rtt_ms, '') AS DECIMAL(18,6)) AS network_side_rtt_ms,
    CAST(NULLIF(r.user_side_downstream_packet_loss_rate, '') AS DECIMAL(18,6)) AS user_down_loss,
    CAST(NULLIF(r.network_side_downstream_packet_loss_rate, '') AS DECIMAL(18,6)) AS network_down_loss,
    CAST(NULLIF(r.wifi_delay_ms, '') AS DECIMAL(18,6)) AS wifi_delay_ms,
    r.bras,
    r.olt,
    r.pon
  FROM raw_tcp_detail_import r
  JOIN params p ON p.import_batch_id = r.import_batch_id
  LEFT JOIN dim_app_mapping m ON m.raw_app_name = r.universal_video_applications
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
  CASE WHEN stat_time IS NULL OR user_key = 'UNKNOWN' THEN 'WARN' ELSE 'OK' END
FROM normalized;

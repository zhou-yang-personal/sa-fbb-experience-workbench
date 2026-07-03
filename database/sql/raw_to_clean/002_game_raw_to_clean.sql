-- RAW Game → DWD Game clean baseline

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
    COALESCE(m.standard_app_name, r.application_protocol) AS app_name,
    COALESCE(m.app_category, 'game') AS app_category,
    STR_TO_DATE(r.statistical_time, '%d/%m/%Y %H:%i:%s') AS stat_time,
    CAST(NULLIF(r.game_duration_s, '') AS DECIMAL(18,6)) / 3600 AS game_hours,
    CAST(NULLIF(r.mos, '') AS DECIMAL(18,6)) AS mos,
    GREATEST(
      COALESCE(CAST(NULLIF(r.subscriber_side_rtt_ms, '') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(r.network_side_rtt_ms, '') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(r.downstream_data_transmission_rtt_ms, '') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(r.upstream_data_transmission_rtt_ms, '') AS DECIMAL(18,6)), 0)
    ) AS worst_latency_ms,
    GREATEST(
      COALESCE(CAST(NULLIF(r.user_side_downstream_packet_loss_rate, '') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(r.network_side_downstream_packet_loss_rate, '') AS DECIMAL(18,6)), 0)
    ) AS worst_loss,
    GREATEST(
      COALESCE(CAST(NULLIF(r.upstream_rtt_jitter_ms, '') AS DECIMAL(18,6)), 0),
      COALESCE(CAST(NULLIF(r.downstream_rtt_jitter_ms, '') AS DECIMAL(18,6)), 0)
    ) AS worst_jitter_ms,
    CAST(NULLIF(r.wifi_delay_ms, '') AS DECIMAL(18,6)) AS wifi_delay_ms,
    r.bras,
    r.olt,
    r.pon
  FROM raw_game_detail_import r
  JOIN params p ON p.import_batch_id = r.import_batch_id
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
  CASE WHEN stat_time IS NULL OR user_key = 'UNKNOWN' THEN 'WARN' ELSE 'OK' END
FROM normalized;

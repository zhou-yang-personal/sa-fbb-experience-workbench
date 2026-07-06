-- SA FBB Experience Workbench core schema baseline
-- MySQL 8.0

CREATE TABLE IF NOT EXISTS meta_import_batch (
  import_batch_id VARCHAR(64) PRIMARY KEY,
  batch_display_name VARCHAR(255) NULL,
  data_type VARCHAR(32) NOT NULL,
  source_file_name VARCHAR(512) NOT NULL,
  source_file_path TEXT NULL,
  source_file_size_bytes BIGINT NULL,
  source_file_hash VARCHAR(128) NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'pending',
  total_rows BIGINT NULL,
  imported_rows BIGINT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  started_at DATETIME NULL,
  finished_at DATETIME NULL,
  message TEXT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_etl_job (
  job_id VARCHAR(64) PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  job_type VARCHAR(64) NOT NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'pending',
  current_step VARCHAR(128) NULL,
  started_at DATETIME NULL,
  finished_at DATETIME NULL,
  duration_ms BIGINT NULL,
  affected_rows BIGINT NULL,
  error_code VARCHAR(64) NULL,
  error_message TEXT NULL,
  INDEX ix_batch_type (import_batch_id, job_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_etl_job_step (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  job_id VARCHAR(64) NOT NULL,
  step_name VARCHAR(128) NOT NULL,
  source_table VARCHAR(128) NULL,
  target_table VARCHAR(128) NULL,
  sql_template VARCHAR(255) NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'pending',
  started_at DATETIME NULL,
  finished_at DATETIME NULL,
  affected_rows BIGINT NULL,
  message TEXT NULL,
  INDEX ix_job_step (job_id, step_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_quality_check_result (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  check_section VARCHAR(64) NOT NULL,
  check_item VARCHAR(128) NOT NULL,
  metric_name VARCHAR(128) NOT NULL,
  metric_value DECIMAL(24, 6) NULL,
  metric_text TEXT NULL,
  severity VARCHAR(32) NOT NULL DEFAULT 'info',
  passed TINYINT(1) NOT NULL DEFAULT 1,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_batch_section (import_batch_id, check_section)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dim_app_mapping (
  raw_app_name VARCHAR(255) NOT NULL PRIMARY KEY,
  standard_app_name VARCHAR(255) NOT NULL,
  app_category VARCHAR(64) NOT NULL,
  invalid_app_flag TINYINT(1) NOT NULL DEFAULT 0,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dim_threshold_config (
  config_key VARCHAR(128) PRIMARY KEY,
  config_value VARCHAR(255) NOT NULL,
  value_type VARCHAR(32) NOT NULL DEFAULT 'string',
  description TEXT NULL,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS raw_tcp_detail_import (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  source_file_name VARCHAR(512) NULL,
  source_line_no BIGINT NULL,
  user_account VARCHAR(255) NULL,
  user_mac VARCHAR(255) NULL,
  user_type VARCHAR(255) NULL,
  universal_video_applications VARCHAR(255) NULL,
  statistics_duration VARCHAR(64) NULL,
  local_ip_address VARCHAR(255) NULL,
  server_ip TEXT NULL,
  device_characteristic VARCHAR(255) NULL,
  device_description VARCHAR(255) NULL,
  bras VARCHAR(255) NULL,
  olt VARCHAR(255) NULL,
  pon VARCHAR(255) NULL,
  wan_type VARCHAR(255) NULL,
  vmos VARCHAR(64) NULL,
  connection_establishment_success_rate VARCHAR(64) NULL,
  connection_establishment_delay_ms VARCHAR(64) NULL,
  upstream_data_transmission_rtt_ms VARCHAR(64) NULL,
  downstream_data_transmission_rtt_ms VARCHAR(64) NULL,
  network_side_rtt_ms VARCHAR(64) NULL,
  subscriber_side_rtt_ms VARCHAR(64) NULL,
  user_avg_download_rate_kbps VARCHAR(64) NULL,
  max_single_flow_rate_kbps VARCHAR(64) NULL,
  network_side_downstream_packet_loss_rate VARCHAR(64) NULL,
  user_side_downstream_packet_loss_rate VARCHAR(64) NULL,
  network_side_upstream_packet_loss_rate VARCHAR(64) NULL,
  user_side_upstream_packet_loss_rate VARCHAR(64) NULL,
  throughput_avg_bandwidth_kbps VARCHAR(64) NULL,
  user_avg_effective_download_rate_kbps VARCHAR(64) NULL,
  download_fluency VARCHAR(64) NULL,
  downloaded_data_volume_kb VARCHAR(64) NULL,
  effective_download_duration_s VARCHAR(64) NULL,
  video_download_duration_s VARCHAR(64) NULL,
  wifi_delay_ms VARCHAR(64) NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_batch (import_batch_id),
  INDEX ix_batch_user (import_batch_id, user_account(64))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS raw_game_detail_import (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  source_file_name VARCHAR(512) NULL,
  source_line_no BIGINT NULL,
  user_account VARCHAR(255) NULL,
  user_mac VARCHAR(255) NULL,
  user_type VARCHAR(64) NULL,
  application_protocol VARCHAR(255) NULL,
  statistical_time VARCHAR(64) NULL,
  local_ip_address VARCHAR(255) NULL,
  server_ip TEXT NULL,
  device_characteristic VARCHAR(255) NULL,
  device_description VARCHAR(255) NULL,
  bras VARCHAR(255) NULL,
  olt VARCHAR(255) NULL,
  pon VARCHAR(255) NULL,
  wan_type VARCHAR(255) NULL,
  mos VARCHAR(64) NULL,
  connection_establishment_success_rate VARCHAR(64) NULL,
  connection_establishment_delay_ms VARCHAR(64) NULL,
  upstream_data_transmission_rtt_ms VARCHAR(64) NULL,
  downstream_data_transmission_rtt_ms VARCHAR(64) NULL,
  network_side_rtt_ms VARCHAR(64) NULL,
  subscriber_side_rtt_ms VARCHAR(64) NULL,
  network_side_downstream_packet_loss_rate VARCHAR(64) NULL,
  user_side_downstream_packet_loss_rate VARCHAR(64) NULL,
  upstream_rtt_jitter_ms VARCHAR(64) NULL,
  downstream_rtt_jitter_ms VARCHAR(64) NULL,
  game_duration_s VARCHAR(64) NULL,
  single_flow_rate_kbps VARCHAR(64) NULL,
  wifi_delay_ms VARCHAR(64) NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_batch (import_batch_id),
  INDEX ix_batch_user (import_batch_id, user_account(64))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dwd_tcp_detail_clean (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  user_key VARCHAR(255) NOT NULL,
  key_confidence VARCHAR(64) NOT NULL,
  user_account VARCHAR(255) NULL,
  user_mac VARCHAR(255) NULL,
  user_type VARCHAR(32) NULL,
  app_name VARCHAR(255) NULL,
  app_category VARCHAR(64) NULL,
  stat_time DATETIME NULL,
  stat_date DATE NULL,
  hour_of_day TINYINT NULL,
  downloaded_gb DECIMAL(24, 6) NULL,
  effective_download_mbps DECIMAL(18, 6) NULL,
  vmos DECIMAL(18, 6) NULL,
  subscriber_side_rtt_ms DECIMAL(18, 6) NULL,
  network_side_rtt_ms DECIMAL(18, 6) NULL,
  user_down_loss DECIMAL(18, 6) NULL,
  network_down_loss DECIMAL(18, 6) NULL,
  wifi_delay_ms DECIMAL(18, 6) NULL,
  bras VARCHAR(255) NULL,
  olt VARCHAR(255) NULL,
  pon VARCHAR(255) NULL,
  data_quality_flag VARCHAR(64) NOT NULL DEFAULT 'OK',
  INDEX ix_batch_user_time (import_batch_id, user_key, stat_time),
  INDEX ix_batch_category (import_batch_id, app_category, user_type, stat_date)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dwd_game_detail_clean (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  user_key VARCHAR(255) NOT NULL,
  key_confidence VARCHAR(64) NOT NULL,
  user_account VARCHAR(255) NULL,
  user_mac VARCHAR(255) NULL,
  user_type VARCHAR(32) NULL,
  app_name VARCHAR(255) NULL,
  app_category VARCHAR(64) NULL,
  stat_time DATETIME NULL,
  stat_date DATE NULL,
  hour_of_day TINYINT NULL,
  game_hours DECIMAL(18, 6) NULL,
  mos DECIMAL(18, 6) NULL,
  worst_latency_ms DECIMAL(18, 6) NULL,
  worst_loss DECIMAL(18, 6) NULL,
  worst_jitter_ms DECIMAL(18, 6) NULL,
  wifi_delay_ms DECIMAL(18, 6) NULL,
  bras VARCHAR(255) NULL,
  olt VARCHAR(255) NULL,
  pon VARCHAR(255) NULL,
  data_quality_flag VARCHAR(64) NOT NULL DEFAULT 'OK',
  INDEX ix_batch_user_time (import_batch_id, user_key, stat_time),
  INDEX ix_batch_category (import_batch_id, app_category, user_type, stat_date)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dws_user_daily_profile (
  import_batch_id VARCHAR(64) NOT NULL,
  user_key VARCHAR(255) NOT NULL,
  user_type VARCHAR(32) NULL,
  stat_date DATE NOT NULL,
  video_rows BIGINT NOT NULL DEFAULT 0,
  game_rows BIGINT NOT NULL DEFAULT 0,
  total_download_gb DECIMAL(24, 6) NOT NULL DEFAULT 0,
  total_game_hours DECIMAL(18, 6) NOT NULL DEFAULT 0,
  avg_vmos DECIMAL(18, 6) NULL,
  avg_mos DECIMAL(18, 6) NULL,
  avg_subscriber_rtt_ms DECIMAL(18, 6) NULL,
  avg_network_rtt_ms DECIMAL(18, 6) NULL,
  avg_user_down_loss DECIMAL(18, 6) NULL,
  avg_network_down_loss DECIMAL(18, 6) NULL,
  peak_row_pct DECIMAL(18, 6) NULL,
  PRIMARY KEY (import_batch_id, user_key, stat_date)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS ads_migration_lead_user (
  analysis_run_id VARCHAR(64) NOT NULL,
  import_batch_id VARCHAR(64) NOT NULL,
  user_key VARCHAR(255) NOT NULL,
  user_type VARCHAR(32) NULL,
  lead_type VARCHAR(128) NOT NULL,
  demand_score INT NOT NULL DEFAULT 0,
  migration_motive_score INT NOT NULL DEFAULT 0,
  recommended_offer VARCHAR(512) NULL,
  PRIMARY KEY (analysis_run_id, user_key),
  INDEX ix_batch_lead (import_batch_id, lead_type, demand_score)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

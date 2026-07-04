-- Complete application schema extension for Phase 1-7.
-- MySQL 8.0. Keep customer CSV and database exports outside Git.

CREATE TABLE IF NOT EXISTS meta_analysis_run (
  analysis_run_id VARCHAR(64) PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  run_type VARCHAR(64) NOT NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'pending',
  started_at DATETIME NULL,
  finished_at DATETIME NULL,
  message TEXT NULL,
  INDEX ix_batch_run (import_batch_id, run_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_export_job (
  export_job_id VARCHAR(64) PRIMARY KEY,
  analysis_run_id VARCHAR(64) NOT NULL,
  export_type VARCHAR(64) NOT NULL,
  output_path TEXT NOT NULL,
  row_count BIGINT NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'pending',
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at DATETIME NULL,
  message TEXT NULL,
  INDEX ix_run_export (analysis_run_id, export_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS raw_crm_user_import (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  source_file_name VARCHAR(512) NULL,
  source_line_no BIGINT NULL,
  crm_user_id VARCHAR(255) NULL,
  user_account VARCHAR(255) NULL,
  user_mac VARCHAR(255) NULL,
  current_plan_name VARCHAR(255) NULL,
  current_plan_speed_mbps DECIMAL(18,6) NULL,
  current_arpu DECIMAL(18,6) NULL,
  contract_status VARCHAR(64) NULL,
  arrears_flag VARCHAR(32) NULL,
  blacklist_flag VARCHAR(32) NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_batch_account (import_batch_id, user_account),
  INDEX ix_batch_mac (import_batch_id, user_mac)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS raw_ftth_coverage_import (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  source_file_name VARCHAR(512) NULL,
  source_line_no BIGINT NULL,
  area_key VARCHAR(255) NULL,
  city VARCHAR(255) NULL,
  neighborhood VARCHAR(255) NULL,
  hp INT NULL,
  hc INT NULL,
  ftth_available_flag VARCHAR(32) NULL,
  build_priority VARCHAR(64) NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_batch_area (import_batch_id, area_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS raw_reachability_import (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  source_file_name VARCHAR(512) NULL,
  source_line_no BIGINT NULL,
  crm_user_id VARCHAR(255) NULL,
  user_account VARCHAR(255) NULL,
  phone_available_flag VARCHAR(32) NULL,
  sms_available_flag VARCHAR(32) NULL,
  app_push_available_flag VARCHAR(32) NULL,
  last_contact_result VARCHAR(255) NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_batch_user (import_batch_id, crm_user_id, user_account)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dws_app_category_daily (
  import_batch_id VARCHAR(64) NOT NULL,
  stat_date DATE NOT NULL,
  user_type VARCHAR(32) NULL,
  app_category VARCHAR(64) NOT NULL,
  active_users BIGINT NOT NULL DEFAULT 0,
  total_download_gb DECIMAL(24,6) NOT NULL DEFAULT 0,
  total_game_hours DECIMAL(18,6) NOT NULL DEFAULT 0,
  avg_vmos DECIMAL(18,6) NULL,
  avg_mos DECIMAL(18,6) NULL,
  avg_effective_download_mbps DECIMAL(18,6) NULL,
  PRIMARY KEY (import_batch_id, stat_date, user_type, app_category)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dws_access_type_hourly_compare (
  import_batch_id VARCHAR(64) NOT NULL,
  stat_date DATE NOT NULL,
  hour_of_day TINYINT NOT NULL,
  user_type VARCHAR(32) NOT NULL,
  active_users BIGINT NOT NULL DEFAULT 0,
  avg_vmos DECIMAL(18,6) NULL,
  avg_mos DECIMAL(18,6) NULL,
  avg_subscriber_rtt_ms DECIMAL(18,6) NULL,
  avg_network_rtt_ms DECIMAL(18,6) NULL,
  avg_user_down_loss DECIMAL(18,6) NULL,
  avg_network_down_loss DECIMAL(18,6) NULL,
  avg_download_mbps DECIMAL(18,6) NULL,
  PRIMARY KEY (import_batch_id, stat_date, hour_of_day, user_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dws_user_experience_bottleneck (
  import_batch_id VARCHAR(64) NOT NULL,
  user_key VARCHAR(255) NOT NULL,
  bottleneck_type VARCHAR(64) NOT NULL,
  severity_score INT NOT NULL DEFAULT 0,
  evidence TEXT NULL,
  PRIMARY KEY (import_batch_id, user_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS ads_dashboard_overview (
  analysis_run_id VARCHAR(64) NOT NULL,
  metric_key VARCHAR(128) NOT NULL,
  metric_label VARCHAR(255) NOT NULL,
  metric_value VARCHAR(255) NOT NULL,
  metric_hint VARCHAR(512) NULL,
  PRIMARY KEY (analysis_run_id, metric_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS ads_app_category_detail (
  analysis_run_id VARCHAR(64) NOT NULL,
  app_category VARCHAR(64) NOT NULL,
  user_type VARCHAR(32) NULL,
  active_users BIGINT NOT NULL DEFAULT 0,
  total_download_gb DECIMAL(24,6) NOT NULL DEFAULT 0,
  total_game_hours DECIMAL(18,6) NOT NULL DEFAULT 0,
  avg_experience_score DECIMAL(18,6) NULL,
  PRIMARY KEY (analysis_run_id, app_category, user_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS ads_experience_quality_summary (
  analysis_run_id VARCHAR(64) NOT NULL,
  quality_dimension VARCHAR(64) NOT NULL,
  user_type VARCHAR(32) NULL,
  affected_users BIGINT NOT NULL DEFAULT 0,
  avg_value DECIMAL(18,6) NULL,
  p90_value DECIMAL(18,6) NULL,
  severity VARCHAR(32) NULL,
  PRIMARY KEY (analysis_run_id, quality_dimension, user_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS ads_cable_fiber_compare (
  analysis_run_id VARCHAR(64) NOT NULL,
  stat_date DATE NOT NULL,
  hour_of_day TINYINT NOT NULL,
  metric_key VARCHAR(64) NOT NULL,
  cable_value DECIMAL(18,6) NULL,
  ftth_value DECIMAL(18,6) NULL,
  diff_value DECIMAL(18,6) NULL,
  PRIMARY KEY (analysis_run_id, stat_date, hour_of_day, metric_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS ads_final_marketing_lead_user (
  analysis_run_id VARCHAR(64) NOT NULL,
  user_key VARCHAR(255) NOT NULL,
  crm_user_id VARCHAR(255) NULL,
  lead_type VARCHAR(128) NOT NULL,
  demand_score INT NOT NULL DEFAULT 0,
  migration_motive_score INT NOT NULL DEFAULT 0,
  current_plan_name VARCHAR(255) NULL,
  current_arpu DECIMAL(18,6) NULL,
  ftth_available_flag VARCHAR(32) NULL,
  reachable_flag VARCHAR(32) NULL,
  final_action VARCHAR(512) NULL,
  recommended_offer VARCHAR(512) NULL,
  PRIMARY KEY (analysis_run_id, user_key),
  INDEX ix_final_action (analysis_run_id, final_action(64))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

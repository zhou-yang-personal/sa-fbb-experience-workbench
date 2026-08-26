-- Versioned IPv4 access classification rules.
-- Rules are small configuration dimensions consumed by MySQL DWD transforms.

CREATE TABLE IF NOT EXISTS meta_access_rule_set (
  rule_set_id VARCHAR(64) NOT NULL PRIMARY KEY,
  version BIGINT NOT NULL,
  rule_set_name VARCHAR(255) NOT NULL,
  default_access_type VARCHAR(32) NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'draft',
  notes TEXT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  published_at DATETIME NULL,
  UNIQUE KEY uk_access_rule_set_version (version),
  INDEX ix_access_rule_set_status (status, published_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dim_access_ip_range (
  rule_id VARCHAR(64) NOT NULL PRIMARY KEY,
  rule_set_id VARCHAR(64) NOT NULL,
  rule_name VARCHAR(255) NOT NULL,
  cidr VARCHAR(64) NULL,
  start_ip VARCHAR(45) NOT NULL,
  end_ip VARCHAR(45) NOT NULL,
  start_ip_num BIGINT UNSIGNED NOT NULL,
  end_ip_num BIGINT UNSIGNED NOT NULL,
  access_type VARCHAR(32) NOT NULL,
  priority INT NOT NULL DEFAULT 100,
  enabled TINYINT(1) NOT NULL DEFAULT 1,
  notes TEXT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX ix_access_rule_set (rule_set_id, enabled, priority),
  INDEX ix_access_rule_range (rule_set_id, start_ip_num, end_ip_num),
  UNIQUE KEY uk_access_rule_name (rule_set_id, rule_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dws_app_daily (
  import_batch_id VARCHAR(64) NOT NULL,
  stat_date DATE NOT NULL,
  user_type VARCHAR(32) NOT NULL DEFAULT 'UNKNOWN',
  app_category VARCHAR(64) NOT NULL,
  app_name VARCHAR(255) NOT NULL,
  observation_rows BIGINT NOT NULL DEFAULT 0,
  active_users BIGINT NOT NULL DEFAULT 0,
  total_download_gb DECIMAL(24,6) NOT NULL DEFAULT 0,
  total_game_hours DECIMAL(18,6) NOT NULL DEFAULT 0,
  avg_effective_download_mbps DECIMAL(18,6) NULL,
  avg_vmos DECIMAL(18,6) NULL,
  avg_mos DECIMAL(18,6) NULL,
  avg_subscriber_rtt_ms DECIMAL(18,6) NULL,
  avg_network_rtt_ms DECIMAL(18,6) NULL,
  avg_user_loss_pct DECIMAL(18,6) NULL,
  avg_network_loss_pct DECIMAL(18,6) NULL,
  poor_experience_users BIGINT NOT NULL DEFAULT 0,
  PRIMARY KEY (import_batch_id, stat_date, user_type, app_category, app_name),
  INDEX ix_app_daily_rank (import_batch_id, app_name, user_type, active_users)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS dws_app_user_summary (
  import_batch_id VARCHAR(64) NOT NULL,
  user_key VARCHAR(255) NOT NULL,
  user_type VARCHAR(32) NOT NULL DEFAULT 'UNKNOWN',
  app_category VARCHAR(64) NOT NULL,
  app_name VARCHAR(255) NOT NULL,
  observation_rows BIGINT NOT NULL DEFAULT 0,
  total_download_gb DECIMAL(24,6) NOT NULL DEFAULT 0,
  total_game_hours DECIMAL(18,6) NOT NULL DEFAULT 0,
  avg_effective_download_mbps DECIMAL(18,6) NULL,
  avg_vmos DECIMAL(18,6) NULL,
  avg_mos DECIMAL(18,6) NULL,
  avg_subscriber_rtt_ms DECIMAL(18,6) NULL,
  avg_network_rtt_ms DECIMAL(18,6) NULL,
  avg_user_loss_pct DECIMAL(18,6) NULL,
  avg_network_loss_pct DECIMAL(18,6) NULL,
  poor_experience_flag TINYINT(1) NOT NULL DEFAULT 0,
  PRIMARY KEY (import_batch_id, user_key, user_type, app_category, app_name),
  INDEX ix_app_user_rank (import_batch_id, app_name, user_type, poor_experience_flag)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT IGNORE INTO dim_threshold_config (config_key, config_value, value_type, description) VALUES
  ('lead.demand_threshold', '60', 'number', 'Minimum demand score for high-demand lead cohorts'),
  ('lead.motive_threshold', '40', 'number', 'Minimum migration motive score for A1 candidates'),
  ('quality.network_rtt_severe_ms', '100', 'number', 'Network-side RTT threshold used for severe-fault exclusion'),
  ('quality.network_loss_severe_pct', '2', 'number', 'Network-side downstream loss threshold used for severe-fault exclusion');

CREATE TABLE IF NOT EXISTS cfg_import_field_mapping (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  data_type VARCHAR(64) NOT NULL,
  target_column VARCHAR(128) NOT NULL,
  source_header VARCHAR(255) NOT NULL,
  required_flag TINYINT NOT NULL DEFAULT 0,
  default_value VARCHAR(512) NULL,
  transform_rule VARCHAR(128) NULL,
  active_flag TINYINT NOT NULL DEFAULT 1,
  priority INT NOT NULL DEFAULT 100,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NULL ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE KEY uk_import_mapping (data_type, target_column, source_header),
  INDEX ix_import_mapping_active (data_type, active_flag, priority)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS cfg_final_join_rule (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  rule_scope VARCHAR(64) NOT NULL,
  left_alias VARCHAR(32) NOT NULL,
  left_column VARCHAR(128) NOT NULL,
  right_alias VARCHAR(32) NOT NULL,
  right_column VARCHAR(128) NOT NULL,
  active_flag TINYINT NOT NULL DEFAULT 1,
  priority INT NOT NULL DEFAULT 100,
  rule_comment VARCHAR(512) NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NULL ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE KEY uk_final_join_rule (rule_scope, left_alias, left_column, right_alias, right_column),
  INDEX ix_final_join_rule_active (rule_scope, active_flag, priority)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_import_row_error (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  source_file_name VARCHAR(512) NULL,
  source_line_no BIGINT NULL,
  data_type VARCHAR(64) NULL,
  error_type VARCHAR(128) NOT NULL,
  error_message TEXT NULL,
  raw_row_sample TEXT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_import_error_batch (import_batch_id, source_line_no),
  INDEX ix_import_error_type (import_batch_id, error_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_mapping_validation_result (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  data_type VARCHAR(64) NOT NULL,
  target_column VARCHAR(128) NOT NULL,
  matched_source_header VARCHAR(255) NULL,
  required_flag TINYINT NOT NULL DEFAULT 0,
  match_status VARCHAR(32) NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_mapping_validation_batch (import_batch_id, data_type, match_status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_dataset_profile (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  data_type VARCHAR(64) NOT NULL,
  profile_key VARCHAR(128) NOT NULL,
  profile_value VARCHAR(512) NULL,
  profile_number DECIMAL(24,6) NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_dataset_profile (import_batch_id, data_type, profile_key),
  INDEX ix_dataset_profile_batch (import_batch_id, data_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

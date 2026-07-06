CREATE TABLE IF NOT EXISTS meta_pipeline_run (
  pipeline_run_id VARCHAR(64) NOT NULL PRIMARY KEY,
  import_batch_id VARCHAR(64) NULL,
  analysis_run_id VARCHAR(64) NULL,
  data_type VARCHAR(32) NOT NULL,
  source_file_name VARCHAR(512) NULL,
  batch_display_name VARCHAR(255) NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'pending',
  current_step VARCHAR(64) NULL,
  total_steps INT NOT NULL DEFAULT 0,
  completed_steps INT NOT NULL DEFAULT 0,
  percent DECIMAL(5,2) NOT NULL DEFAULT 0,
  started_at DATETIME NULL,
  finished_at DATETIME NULL,
  elapsed_ms BIGINT NOT NULL DEFAULT 0,
  message TEXT NULL,
  error_message TEXT NULL,
  final_fusion_status VARCHAR(32) NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX ix_pipeline_status (status, created_at),
  INDEX ix_pipeline_batch (import_batch_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_pipeline_step (
  pipeline_run_id VARCHAR(64) NOT NULL,
  step_index INT NOT NULL,
  step_name VARCHAR(64) NOT NULL,
  step_label VARCHAR(128) NOT NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'pending',
  started_at DATETIME NULL,
  finished_at DATETIME NULL,
  elapsed_ms BIGINT NOT NULL DEFAULT 0,
  message TEXT NULL,
  error_message TEXT NULL,
  PRIMARY KEY (pipeline_run_id, step_index),
  INDEX ix_pipeline_step_status (pipeline_run_id, status),
  INDEX ix_pipeline_step_name (step_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_pipeline_log (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  pipeline_run_id VARCHAR(64) NOT NULL,
  seq INT NOT NULL,
  ts DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  level VARCHAR(16) NOT NULL DEFAULT 'info',
  step_name VARCHAR(64) NULL,
  message TEXT NOT NULL,
  elapsed_ms BIGINT NOT NULL DEFAULT 0,
  UNIQUE KEY uk_pipeline_seq (pipeline_run_id, seq),
  INDEX ix_pipeline_log (pipeline_run_id, id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

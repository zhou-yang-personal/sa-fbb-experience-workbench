-- Saved investigation state for the V2 analysis workflow.
-- Findings themselves are generated from per-batch ADS at query time so they
-- always reflect the selected immutable analysis run and policy binding.

CREATE TABLE IF NOT EXISTS meta_saved_investigation (
  investigation_id VARCHAR(64) NOT NULL PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  analysis_run_id VARCHAR(64) NOT NULL,
  finding_id VARCHAR(96) NULL,
  title VARCHAR(255) NOT NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'open',
  context_json JSON NOT NULL,
  notes TEXT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX ix_investigation_run (analysis_run_id, status, updated_at),
  INDEX ix_investigation_batch (import_batch_id, updated_at),
  INDEX ix_investigation_finding (finding_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

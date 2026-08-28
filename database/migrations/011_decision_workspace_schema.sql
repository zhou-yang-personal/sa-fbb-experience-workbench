-- Decision workspace V3: versioned business rules and explainable opportunity outputs.
-- Additive and run-scoped. Existing V2 DWS/ADS tables remain the analytical source.

CREATE TABLE IF NOT EXISTS meta_decision_rule_profile (
  rule_profile_id VARCHAR(64) NOT NULL PRIMARY KEY,
  version BIGINT NOT NULL,
  profile_name VARCHAR(255) NOT NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'draft',
  minimum_user_observations BIGINT NOT NULL,
  minimum_app_users BIGINT NOT NULL,
  minimum_app_observations BIGINT NOT NULL,
  persistent_poor_rate_pct DECIMAL(9,4) NOT NULL,
  problem_app_poor_rate_pct DECIMAL(9,4) NOT NULL,
  problem_app_persistent_user_rate_pct DECIMAL(9,4) NOT NULL,
  heavy_traffic_gb DECIMAL(18,6) NOT NULL,
  heavy_usage_hours DECIMAL(18,6) NOT NULL,
  peak_hour_start TINYINT NOT NULL,
  peak_hour_end TINYINT NOT NULL,
  migration_min_traffic_gb DECIMAL(18,6) NOT NULL,
  speed_upgrade_min_traffic_gb DECIMAL(18,6) NOT NULL,
  speed_upgrade_max_effective_mbps DECIMAL(18,6) NOT NULL,
  mesh_min_wifi_delay_ms DECIMAL(18,6) NOT NULL,
  app_bundle_min_observations BIGINT NOT NULL,
  opportunity_min_active_days BIGINT NOT NULL,
  opportunity_min_observations BIGINT NOT NULL,
  speed_upgrade_min_conditions BIGINT NOT NULL,
  app_bundle_min_active_days BIGINT NOT NULL,
  sufficient_app_users BIGINT NOT NULL,
  sufficient_app_observations BIGINT NOT NULL,
  attention_app_poor_rate_pct DECIMAL(9,4) NOT NULL,
  attention_app_persistent_user_rate_pct DECIMAL(9,4) NOT NULL,
  severe_app_poor_rate_pct DECIMAL(9,4) NOT NULL,
  severe_app_persistent_user_rate_pct DECIMAL(9,4) NOT NULL,
  severe_app_severe_user_rate_pct DECIMAL(9,4) NOT NULL,
  mesh_min_coverage_pct DECIMAL(9,4) NOT NULL,
  mesh_min_rtt_delta_ms DECIMAL(18,6) NOT NULL,
  mesh_min_loss_delta_pct DECIMAL(18,6) NOT NULL,
  rule_snapshot JSON NULL,
  notes TEXT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  published_at DATETIME NULL,
  UNIQUE KEY uk_decision_rule_version (version),
  INDEX ix_decision_rule_status (status, published_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_analysis_run_decision_binding (
  analysis_run_id VARCHAR(64) NOT NULL PRIMARY KEY,
  import_batch_id VARCHAR(64) NOT NULL,
  rule_profile_id VARCHAR(64) NOT NULL,
  rule_profile_version BIGINT NOT NULL,
  rule_snapshot JSON NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX ix_decision_binding_batch (import_batch_id, analysis_run_id),
  INDEX ix_decision_binding_profile (rule_profile_id, rule_profile_version)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS meta_aggregation_subtask_checkpoint (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  pipeline_run_id VARCHAR(64) NOT NULL,
  import_batch_id VARCHAR(64) NOT NULL,
  analysis_run_id VARCHAR(64) NOT NULL,
  stage_name VARCHAR(64) NOT NULL,
  subtask_name VARCHAR(64) NOT NULL,
  status VARCHAR(32) NOT NULL DEFAULT 'pending',
  attempt_count INT NOT NULL DEFAULT 0,
  started_at DATETIME NULL,
  finished_at DATETIME NULL,
  duration_ms BIGINT NOT NULL DEFAULT 0,
  message TEXT NULL,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE KEY uk_aggregation_subtask (analysis_run_id, stage_name, subtask_name),
  INDEX ix_aggregation_subtask_pipeline (pipeline_run_id, status, updated_at),
  INDEX ix_aggregation_subtask_batch (import_batch_id, analysis_run_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS ads_opportunity_user_v3 (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  analysis_run_id VARCHAR(64) NOT NULL,
  import_batch_id VARCHAR(64) NOT NULL,
  user_key VARCHAR(255) NOT NULL,
  opportunity_type VARCHAR(64) NOT NULL,
  opportunity_level VARCHAR(32) NOT NULL,
  evidence_value DECIMAL(24,6) NULL,
  evidence_unit VARCHAR(32) NULL,
  evidence_summary TEXT NOT NULL,
  data_limitation_code VARCHAR(64) NULL,
  rule_profile_version BIGINT NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_opportunity_user_v3 (analysis_run_id, user_key(128), opportunity_type),
  INDEX ix_opportunity_type_v3 (analysis_run_id, opportunity_type, opportunity_level),
  INDEX ix_opportunity_batch_v3 (import_batch_id, user_key(128))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS ads_opportunity_summary_v3 (
  id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  analysis_run_id VARCHAR(64) NOT NULL,
  import_batch_id VARCHAR(64) NOT NULL,
  opportunity_type VARCHAR(64) NOT NULL,
  candidate_users BIGINT NOT NULL DEFAULT 0,
  high_priority_users BIGINT NOT NULL DEFAULT 0,
  total_evidence_value DECIMAL(24,6) NULL,
  evidence_unit VARCHAR(32) NULL,
  availability_status VARCHAR(32) NOT NULL,
  data_limitation_code VARCHAR(64) NULL,
  rule_profile_version BIGINT NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_opportunity_summary_v3 (analysis_run_id, opportunity_type),
  INDEX ix_opportunity_summary_batch_v3 (import_batch_id, analysis_run_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT IGNORE INTO meta_decision_rule_profile (
  rule_profile_id, version, profile_name, status,
  minimum_user_observations, minimum_app_users, minimum_app_observations,
  persistent_poor_rate_pct, problem_app_poor_rate_pct,
  problem_app_persistent_user_rate_pct, heavy_traffic_gb, heavy_usage_hours,
  peak_hour_start, peak_hour_end, migration_min_traffic_gb,
  speed_upgrade_min_traffic_gb, speed_upgrade_max_effective_mbps,
  mesh_min_wifi_delay_ms, app_bundle_min_observations,
  opportunity_min_active_days, opportunity_min_observations,
  speed_upgrade_min_conditions, app_bundle_min_active_days,
  sufficient_app_users, sufficient_app_observations,
  attention_app_poor_rate_pct, attention_app_persistent_user_rate_pct,
  severe_app_poor_rate_pct, severe_app_persistent_user_rate_pct,
  severe_app_severe_user_rate_pct,
  mesh_min_coverage_pct, mesh_min_rtt_delta_ms, mesh_min_loss_delta_pct,
  rule_snapshot, notes, published_at
) VALUES (
  'DECISION_RULE_DEFAULT_V1', 1, 'Decision workspace default V1', 'published',
  3, 10, 30, 30.0, 20.0, 10.0, 20.0, 3.0,
  20, 23, 5.0, 10.0, 20.0, 30.0, 10,
  2, 10, 2, 3,
  30, 100, 10.0, 5.0, 40.0, 20.0, 10.0,
  30.0, 30.0, 1.0,
  JSON_OBJECT(
    'minimum_user_observations',3,
    'minimum_app_users',10,
    'minimum_app_observations',30,
    'persistent_poor_rate_pct',30.0,
    'problem_app_poor_rate_pct',20.0,
    'problem_app_persistent_user_rate_pct',10.0,
    'heavy_traffic_gb',20.0,
    'heavy_usage_hours',3.0,
    'peak_hour_start',20,
    'peak_hour_end',23,
    'migration_min_traffic_gb',5.0,
    'speed_upgrade_min_traffic_gb',10.0,
    'speed_upgrade_max_effective_mbps',20.0,
    'mesh_min_wifi_delay_ms',30.0,
    'app_bundle_min_observations',10,
    'opportunity_min_active_days',2,
    'opportunity_min_observations',10,
    'speed_upgrade_min_conditions',2,
    'app_bundle_min_active_days',3,
    'sufficient_app_users',30,
    'sufficient_app_observations',100,
    'attention_app_poor_rate_pct',10.0,
    'attention_app_persistent_user_rate_pct',5.0,
    'severe_app_poor_rate_pct',40.0,
    'severe_app_persistent_user_rate_pct',20.0,
    'severe_app_severe_user_rate_pct',10.0
    ,'mesh_min_coverage_pct',30.0
    ,'mesh_min_rtt_delta_ms',30.0
    ,'mesh_min_loss_delta_pct',1.0
  ),
  'Editable defaults. Clone to a draft, validate, then publish a new immutable version.',
  CURRENT_TIMESTAMP
);

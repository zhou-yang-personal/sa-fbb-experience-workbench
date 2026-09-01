export type ImportDataType = 'tcp' | 'game' | 'crm' | 'coverage' | 'reachability';
export type JobStatus = 'pending' | 'running' | 'success' | 'failed' | 'cancelled';
export type DashboardChartKind = 'bar' | 'radar';
export type ExecutionLogStatus = 'success' | 'failure';
export type ActionRunStatus = 'idle' | 'running' | 'success' | 'failure';
export type PipelineStepStatus = 'not_started' | 'running' | 'success' | 'warning' | 'failed';
export type RuntimeEngine = 'duckdb' | 'mysql_compat';

export interface MySqlSettings {
  host: string;
  port: number;
  database: string;
  user: string;
  secret: string;
  local_infile?: boolean;
}

export interface DuckDbWorkspaceSettings {
  workspace_dir: string;
}

export interface DuckDbWorkspaceStatus {
  workspace_dir: string;
  database_path: string;
  initialized: boolean;
  duckdb_version: string;
  batch_count: number;
  run_count: number;
  running_run_count: number;
  latest_run_id?: string;
  latest_run_status?: string;
  latest_run_step?: string;
  latest_run_message?: string;
}

export interface DuckDbPocRequest {
  workspace_dir: string;
  file_path: string;
  data_type: 'tcp';
  batch_display_name?: string;
  default_access_type?: 'CABLE' | 'FTTH' | 'OTHER';
  ftth_ranges?: string[];
}

export interface DuckDbPocResult {
  import_batch_id: string;
  analysis_run_id: string;
  source_rows: number;
  clean_rows: number;
  hourly_rows: number;
  elapsed_ms: number;
  database_path: string;
  parquet_path: string;
  metrics: MetricCard[];
}

export interface DuckDbBatchListItem {
  import_batch_id: string;
  batch_display_name?: string;
  data_type: string;
  source_file_name: string;
  source_rows: number;
  clean_rows: number;
  status: string;
  message?: string;
  created_at: string;
  latest_analysis_run_id?: string;
  latest_run_status?: string;
}

export interface DuckDbAnalysisRunItem {
  analysis_run_id: string;
  import_batch_id: string;
  run_type: string;
  implementation_version: string;
  status: string;
  current_step?: string;
  message?: string;
  started_at: string;
  finished_at?: string;
}

export interface DuckDbAccessSummaryRow {
  access_type: string;
  active_users: number;
  observation_rows: number;
  downloaded_gb?: number;
  avg_effective_download_mbps?: number;
  avg_rtt_ms?: number;
  avg_user_loss_pct?: number;
  avg_network_loss_pct?: number;
  avg_vmos?: number;
}

export interface DuckDbAccessHourlyRow extends DuckDbAccessSummaryRow {
  stat_date: string;
  hour_of_day: number;
}

export interface CommandAck {
  status: string;
  message: string;
}

export interface CsvProbeResult {
  path: string;
  file_name: string;
  file_size_bytes: number;
  sha256: string;
  delimiter: string;
  headers: string[];
  preview_rows: string[][];
}

export interface ImportBatchResult {
  import_batch_id: string;
  batch_display_name?: string;
  data_type: string;
  source_file_name: string;
  status: string;
}

export interface ImportCurrentFileResult {
  batch: ImportBatchResult;
  mapping_summary: MetricCard[];
  mapping_results: MetricCard[];
  raw_status: MetricCard[];
  profile: MetricCard[];
  message: string;
}

export type ImportPipelineStatusValue = 'pending' | 'running' | 'success' | 'failed' | 'degraded' | 'canceled';

export interface ImportPipelineStartResult {
  pipeline_run_id: string;
  import_batch_id?: string;
  analysis_run_id: string;
  status: ImportPipelineStatusValue | string;
}

export interface ImportPipelineResumeInput {
  importBatchId: string;
  analysisRunId?: string;
  confirmOriginalProcessStopped?: boolean;
}

export interface ImportPipelineStepRow {
  step_index: number;
  step_name: string;
  step_label: string;
  status: string;
  started_at?: string;
  finished_at?: string;
  elapsed_ms: number;
  message?: string;
  error_message?: string;
}

export interface ImportPipelineStatus {
  pipeline_run_id: string;
  status: ImportPipelineStatusValue | string;
  current_step?: string;
  percent: number;
  started_at?: string;
  finished_at?: string;
  elapsed_ms: number;
  import_batch_id?: string;
  analysis_run_id?: string;
  failed_step?: string;
  error_message?: string;
  final_fusion_status?: string;
  message?: string;
  steps: ImportPipelineStepRow[];
}

export interface ImportPipelineLogRow {
  sequence: number;
  timestamp: string;
  level: string;
  step_name?: string;
  message: string;
  elapsed_ms: number;
}

export interface BatchListItem {
  import_batch_id: string;
  batch_display_name?: string;
  data_type: string;
  source_file_name: string;
  status: string;
  total_rows?: number;
  imported_rows?: number;
  analysis_run_id?: string;
  pipeline_run_id?: string;
  pipeline_status?: string;
  pipeline_message?: string;
}

export interface AnalysisRunOption {
  analysis_run_id: string;
  import_batch_id: string;
  run_type: string;
  status: string;
  started_at?: string;
  finished_at?: string;
  message?: string;
  pipeline_linked: boolean;
  v2_period_ready: boolean;
  v2_app_ads_ready: boolean;
  v2_hourly_ready: boolean;
  experience_policy_id?: string;
  experience_policy_version?: number;
}

export interface BatchTableRegistryRow {
  import_batch_id: string;
  layer: string;
  data_type: string;
  logical_table_name: string;
  base_table_name: string;
  physical_table_name: string;
  row_count: number;
  status: string;
}

export interface ModuleStatusRow {
  import_batch_id: string;
  analysis_run_id?: string;
  module_id: string;
  module_name: string;
  enabled: boolean;
  data_type?: string;
  missing_required_fields?: string;
  missing_tables?: string;
  row_count: number;
  status_text?: string;
}

export interface MetricCard {
  label: string;
  value: string;
  hint: string;
}

export interface OpportunityCandidateRow {
  user_key: string;
  opportunity_type: string;
  opportunity_level: string;
  user_type: string;
  active_days: number;
  observation_rows: number;
  total_download_gb: number;
  total_effective_duration_hours: number;
  avg_effective_download_mbps?: number;
  avg_wifi_delay_ms?: number;
  avg_subscriber_rtt_ms?: number;
  avg_network_rtt_ms?: number;
  avg_user_loss_pct?: number;
  avg_network_loss_pct?: number;
  primary_app?: string;
  primary_app_active_days: number;
  primary_app_observations: number;
  evidence_value?: number;
  evidence_unit?: string;
  evidence_summary: string;
  data_limitation_code?: string;
  rule_profile_version: number;
}

export interface OpportunityCandidatePage {
  rows: OpportunityCandidateRow[];
  total: number;
  page: number;
  page_size: number;
}

export interface DashboardOverview {
  metrics: MetricCard[];
}

export interface DashboardChartGroup {
  title: string;
  kind: DashboardChartKind;
  metrics: MetricCard[];
}

export interface ExecutionLogEntry {
  id: string;
  command: string;
  status: ExecutionLogStatus;
  started_at: string;
  finished_at: string;
  duration_ms: number;
  message: string;
  result_preview?: string;
}

export interface ActionState {
  status: ActionRunStatus;
  message?: string;
  started_at?: string;
  finished_at?: string;
  duration_ms?: number;
}

export interface EtlJobStepRow {
  job_id: string;
  job_type: string;
  step_name: string;
  target_table?: string;
  status: string;
  affected_rows?: number;
  started_at?: string;
  finished_at?: string;
  message?: string;
}

export interface EtlJobStepsQuery {
  jobId?: string;
  status?: string;
  limit?: number;
}

export interface LeadQueryParams {
  page?: number;
  pageSize?: number;
  leadType?: string;
  finalAction?: string;
  keyword?: string;
}

export interface FinalLeadExportOptions {
  finalActions?: string[];
}

export interface LeadUserRow {
  user_key: string;
  user_type?: string;
  lead_type: string;
  demand_score: number;
  migration_motive_score: number;
  recommended_offer?: string;
}

export interface FinalLeadUserRow {
  user_key: string;
  crm_user_id?: string;
  lead_type: string;
  demand_score: number;
  migration_motive_score: number;
  current_plan_name?: string;
  current_arpu?: number;
  ftth_available_flag?: string;
  reachable_flag?: string;
  final_action?: string;
  recommended_offer?: string;
}

export interface ImportBatchSummary {
  importBatchId: string;
  batchDisplayName?: string;
  dataType: ImportDataType;
  sourceFileName: string;
  totalRows: number;
  importedRows: number;
  status: JobStatus;
}

export interface MigrationLeadSummary {
  leadType: string;
  userCount: number;
  avgDemandScore: number;
  avgMigrationMotiveScore: number;
  recommendedAction: string;
}

export interface AccessRuleSetRow {
  rule_set_id: string;
  version: number;
  rule_set_name: string;
  /**
   * Final access type assigned to Others (IPs that match no explicit range).
   * Drafts may leave this unset, but they cannot be published in that state.
   */
  default_access_type: 'CABLE' | 'FTTH' | 'OTHER' | null | string;
  status: 'draft' | 'published' | 'archived' | string;
  rule_count: number;
  published_at?: string;
  updated_at: string;
}

export interface AccessIpRangeRow {
  rule_id: string;
  rule_set_id: string;
  rule_name: string;
  cidr?: string;
  start_ip: string;
  end_ip: string;
  access_type: 'CABLE' | 'FTTH' | 'OTHER' | string;
  priority: number;
  enabled: boolean;
  notes?: string;
  updated_at: string;
}

export interface AccessRuleInput {
  ruleSetId: string;
  ruleId?: string;
  ruleName: string;
  cidr?: string;
  startIp?: string;
  endIp?: string;
  accessType: 'CABLE' | 'FTTH' | 'OTHER';
  priority?: number;
  enabled?: boolean;
  notes?: string;
}

export interface AccessRuleValidationResult {
  valid: boolean;
  others_configured: boolean;
  others_access_type?: 'CABLE' | 'FTTH' | 'OTHER' | null;
  rule_count: number;
  enabled_rule_count: number;
  conflict_count: number;
  invalid_rule_count: number;
  message: string;
}

export interface AccessRulePreviewResult {
  sample_ip_count: number;
  classified_ip_count: number;
  cable_ip_count: number;
  ftth_ip_count: number;
  other_ip_count: number;
  fallback_ip_count: number;
  others_ip_count: number;
  unmatched_ip_count: number;
  coverage_pct: number;
  sample_limit: number;
  message: string;
}

export type AnalysisBaselineType = 'PREVIOUS_COMPARABLE_RUN' | 'FTTH_PEER' | 'NON_PEAK' | 'POLICY_THRESHOLD';

/**
 * Shared investigation context. Empty values mean that the dimension is not
 * constrained; batch and run remain owned by the workbench controller.
 */
export interface AnalysisContext {
  data_type?: string;
  app_category?: string;
  app_name?: string;
  access_type?: string;
  date_from?: string;
  date_to?: string;
  hour_from?: number;
  hour_to?: number;
  issue_metric?: string;
  issue_side?: string;
  user_key?: string;
  server_ip?: string;
  bras?: string;
  network_object?: string;
  baseline_type?: AnalysisBaselineType;
  finding_id?: string;
}

export type AnalysisContextKey = keyof AnalysisContext;

export interface ExperienceStatusV2 {
  analysis_run_id: string;
  import_batch_id: string;
  eligible_users: number;
  valid_observations: number;
  poor_observations: number;
  poor_observation_rate_pct?: number;
  ever_affected_users: number;
  ever_affected_user_rate_pct?: number;
  persistent_poor_users: number;
  persistent_poor_user_rate_pct?: number;
  severe_poor_users: number;
  severe_poor_user_rate_pct?: number;
  policy_id: string;
  policy_version: number;
  sample_status: string;
}

export interface ExperienceFinding {
  finding_id: string;
  finding_type: string;
  title_zh: string;
  title_en: string;
  app_category?: string;
  app_name?: string;
  access_type?: string;
  issue_metric?: string;
  issue_side?: string;
  baseline_type: string;
  numerator: number;
  denominator: number;
  sample_size: number;
  affected_users: number;
  affected_user_rate_pct?: number;
  poor_observation_rate_pct?: number;
  severe_user_rate_pct?: number;
  severity: string;
  confidence: string;
  main_driver?: string;
  evidence_summary: string;
  data_limitations?: string;
  recommended_next_step: string;
  rule_version: number;
}

export interface DataCoverageItemV2 {
  dimension: string;
  status: 'AVAILABLE' | 'NOT_IMPORTED' | 'UNAVAILABLE' | 'INSUFFICIENT_SAMPLE' | string;
  available_rows: number;
  total_rows: number;
  coverage_pct?: number;
  limitation?: string;
}

export interface RunVerificationV2 {
  current_analysis_run_id: string;
  current_import_batch_id: string;
  previous_analysis_run_id?: string;
  previous_import_batch_id?: string;
  comparable: boolean;
  comparison_reason: string;
  current_poor_observation_rate_pct?: number;
  previous_poor_observation_rate_pct?: number;
  poor_observation_rate_delta_pct?: number;
  current_persistent_poor_user_rate_pct?: number;
  previous_persistent_poor_user_rate_pct?: number;
  persistent_poor_user_rate_delta_pct?: number;
  current_severe_poor_user_rate_pct?: number;
  previous_severe_poor_user_rate_pct?: number;
  severe_poor_user_rate_delta_pct?: number;
}

export interface InvestigationEvidenceRow {
  user_key: string;
  access_type: string;
  app_category: string;
  app_name: string;
  valid_obs_rows: number;
  poor_obs_rows: number;
  poor_observation_rate_pct?: number;
  persistent_poor_user: boolean;
  severe_poor_user: boolean;
  avg_vmos?: number;
  avg_subscriber_rtt_ms?: number;
  avg_network_rtt_ms?: number;
  avg_user_loss_pct?: number;
  avg_network_loss_pct?: number;
}

export interface InvestigationHourlyRow {
  stat_date: string;
  hour_of_day: number;
  access_type: string;
  eligible_users: number;
  valid_obs_rows: number;
  poor_obs_rows: number;
  poor_observation_rate_pct?: number;
  persistent_poor_users: number;
  severe_poor_users: number;
  sample_status: string;
}

export interface InvestigationServerIpRow {
  server_ip: string;
  observed_users: number;
  observation_rows: number;
  avg_subscriber_rtt_ms?: number;
  avg_network_rtt_ms?: number;
  avg_user_loss_pct?: number;
  avg_network_loss_pct?: number;
}

export interface SavedInvestigation {
  investigation_id: string;
  import_batch_id: string;
  analysis_run_id: string;
  finding_id?: string;
  title: string;
  status: string;
  context_json: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface ExperiencePolicyRow {
  policy_id: string;
  version: number;
  policy_name: string;
  status: string;
  persistent_min_valid_obs: number;
  persistent_min_poor_obs: number;
  persistent_min_poor_rate_pct: number;
  severe_user_min_valid_obs: number;
  severe_user_min_severe_obs: number;
  severe_user_min_severe_rate_pct: number;
  minimum_app_eligible_users: number;
  minimum_app_valid_obs: number;
  finding_attention_persistent_user_rate_pct: number;
  finding_severe_user_rate_pct: number;
  notes?: string;
  updated_at: string;
}

export interface AppExperienceProfileRow {
  profile_id: string;
  policy_id: string;
  profile_code: string;
  profile_name: string;
  data_type: string;
  app_category?: string;
  priority: number;
  enabled: boolean;
  poor_vmos_below?: number;
  poor_mos_below?: number;
  poor_subscriber_rtt_ms_at_least?: number;
  poor_network_rtt_ms_at_least?: number;
  poor_user_loss_pct_at_least?: number;
  poor_network_loss_pct_at_least?: number;
  poor_jitter_ms_at_least?: number;
  severe_vmos_below?: number;
  severe_mos_below?: number;
  severe_subscriber_rtt_ms_at_least?: number;
  severe_network_rtt_ms_at_least?: number;
  severe_user_loss_pct_at_least?: number;
  severe_network_loss_pct_at_least?: number;
  severe_jitter_ms_at_least?: number;
}

export interface DecisionRuleProfileRow {
  rule_profile_id: string;
  version: number;
  profile_name: string;
  status: string;
  minimum_user_observations: number;
  minimum_app_users: number;
  minimum_app_observations: number;
  persistent_poor_rate_pct: number;
  problem_app_poor_rate_pct: number;
  problem_app_persistent_user_rate_pct: number;
  heavy_traffic_gb: number;
  heavy_usage_hours: number;
  peak_hour_start: number;
  peak_hour_end: number;
  migration_min_traffic_gb: number;
  speed_upgrade_min_traffic_gb: number;
  speed_upgrade_max_effective_mbps: number;
  mesh_min_wifi_delay_ms: number;
  app_bundle_min_observations: number;
  opportunity_min_active_days: number;
  opportunity_min_observations: number;
  speed_upgrade_min_conditions: number;
  app_bundle_min_active_days: number;
  sufficient_app_users: number;
  sufficient_app_observations: number;
  attention_app_poor_rate_pct: number;
  attention_app_persistent_user_rate_pct: number;
  severe_app_poor_rate_pct: number;
  severe_app_persistent_user_rate_pct: number;
  severe_app_severe_user_rate_pct: number;
  mesh_min_coverage_pct: number;
  mesh_min_rtt_delta_ms: number;
  mesh_min_loss_delta_pct: number;
  distribution_thresholds: {
    traffic_daily_gb: number[];
    duration_daily_hours: number[];
    peak_daily_hours: number[];
    observations_daily: number[];
    rate_mbps: number[];
    rtt_ms: number[];
    loss_pct: number[];
  };
  notes?: string;
  updated_at: string;
}

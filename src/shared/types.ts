export type ImportDataType = 'tcp' | 'game' | 'crm' | 'coverage' | 'reachability';
export type JobStatus = 'pending' | 'running' | 'success' | 'failed' | 'cancelled';
export type DashboardChartKind = 'bar' | 'radar';
export type ExecutionLogStatus = 'success' | 'failure';
export type ActionRunStatus = 'idle' | 'running' | 'success' | 'failure';
export type PipelineStepStatus = 'not_started' | 'running' | 'success' | 'warning' | 'failed';

export interface MySqlSettings {
  host: string;
  port: number;
  database: string;
  user: string;
  secret: string;
  local_infile?: boolean;
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

export interface BatchListItem {
  import_batch_id: string;
  batch_display_name?: string;
  data_type: string;
  source_file_name: string;
  status: string;
  total_rows?: number;
  imported_rows?: number;
  analysis_run_id?: string;
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

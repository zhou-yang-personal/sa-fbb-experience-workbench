export type ImportDataType = 'tcp' | 'game' | 'crm' | 'coverage' | 'reachability';
export type JobStatus = 'pending' | 'running' | 'success' | 'failed' | 'cancelled';

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
  data_type: string;
  source_file_name: string;
  status: string;
}

export interface MetricCard {
  label: string;
  value: string;
  hint: string;
}

export interface DashboardOverview {
  metrics: MetricCard[];
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

export type ImportDataType = 'tcp' | 'game' | 'crm' | 'coverage' | 'reachability';

export type JobStatus = 'pending' | 'running' | 'success' | 'failed' | 'cancelled';

export interface ImportBatchSummary {
  importBatchId: string;
  dataType: ImportDataType;
  sourceFileName: string;
  totalRows: number;
  importedRows: number;
  status: JobStatus;
}

export interface DashboardMetric {
  label: string;
  value: string;
  hint: string;
}

export interface MigrationLeadSummary {
  leadType: string;
  userCount: number;
  avgDemandScore: number;
  avgMigrationMotiveScore: number;
  recommendedAction: string;
}

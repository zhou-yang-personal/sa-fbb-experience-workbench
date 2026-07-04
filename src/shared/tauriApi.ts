import { invoke } from '@tauri-apps/api/core';
import type { CommandAck, CsvProbeResult, DashboardOverview, FinalLeadUserRow, ImportBatchResult, LeadUserRow, MySqlSettings, MetricCard } from './types';

export const api = {
  dbTestConnection: (settings: MySqlSettings) => invoke<CommandAck>('db_test_connection', { settings }),
  dbInitialize: (settings: MySqlSettings) => invoke<CommandAck>('db_initialize', { settings }),
  importProbeCsv: (path: string) => invoke<CsvProbeResult>('import_probe_csv', { path }),
  importCreateBatch: (settings: MySqlSettings, dataType: string, filePath: string) =>
    invoke<ImportBatchResult>('import_create_batch', { req: { settings, data_type: dataType, file_path: filePath } }),
  importStartRawLoad: (settings: MySqlSettings, importBatchId: string, dataType: string, filePath: string, mode?: string) =>
    invoke<CommandAck>('import_start_raw_load', { req: { settings, import_batch_id: importBatchId, data_type: dataType, file_path: filePath, mode } }),
  importGetBatchStatus: (settings: MySqlSettings, importBatchId: string) =>
    invoke<MetricCard[]>('import_get_batch_status', { settings, importBatchId }),
  etlGetRecentJobs: (settings: MySqlSettings, importBatchId: string) =>
    invoke<MetricCard[]>('etl_get_recent_jobs', { settings, importBatchId }),
  qualityGetBatchReport: (settings: MySqlSettings, importBatchId: string) =>
    invoke<MetricCard[]>('quality_get_batch_report', { settings, importBatchId }),
  etlStartCleanJob: (settings: MySqlSettings, importBatchId: string) =>
    invoke<CommandAck>('etl_start_clean_job', { req: { settings, import_batch_id: importBatchId } }),
  etlStartAggregateJob: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<CommandAck>('etl_start_aggregate_job', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  dashboardGetOverview: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<DashboardOverview>('dashboard_get_overview', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  leadsQueryUsers: (settings: MySqlSettings, analysisRunId: string) =>
    invoke<LeadUserRow[]>('leads_query_users', { req: { settings, analysis_run_id: analysisRunId, page: 1, page_size: 100 } }),
  finalLeadsQueryUsers: (settings: MySqlSettings, analysisRunId: string) =>
    invoke<FinalLeadUserRow[]>('final_leads_query_users', { req: { settings, analysis_run_id: analysisRunId, page: 1, page_size: 100 } }),
  exportLeadsCsv: (settings: MySqlSettings, analysisRunId: string, outputPath: string) =>
    invoke<CommandAck>('export_leads_csv', { req: { settings, analysis_run_id: analysisRunId, output_path: outputPath } }),
  exportFinalLeadsCsv: (settings: MySqlSettings, analysisRunId: string, outputPath: string) =>
    invoke<CommandAck>('export_final_leads_csv', { req: { settings, analysis_run_id: analysisRunId, output_path: outputPath } }),
  configSeedDefaults: (settings: MySqlSettings) => invoke<CommandAck>('config_seed_defaults', { settings }),
  configGetImportMappings: (settings: MySqlSettings, dataType: string) =>
    invoke<MetricCard[]>('config_get_import_mappings', { settings, dataType }),
  configGetJoinRules: (settings: MySqlSettings) => invoke<MetricCard[]>('config_get_join_rules', { settings }),
};

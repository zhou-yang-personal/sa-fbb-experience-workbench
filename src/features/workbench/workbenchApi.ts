import { invoke } from '@tauri-apps/api/core';
import type { CommandAck, DashboardOverview, FinalLeadExportOptions, FinalLeadUserRow, ImportBatchResult, LeadQueryParams, LeadUserRow, MetricCard, MySqlSettings } from '../../shared/types';

function normalizeFilter(value?: string) {
  const normalized = value?.trim();
  return normalized && normalized !== 'ALL' ? normalized : undefined;
}

function leadQueryRequest(settings: MySqlSettings, analysisRunId: string, params?: LeadQueryParams) {
  return {
    settings,
    analysis_run_id: analysisRunId,
    page: params?.page ?? 1,
    page_size: params?.pageSize ?? 100,
    lead_type: normalizeFilter(params?.leadType),
    final_action: normalizeFilter(params?.finalAction),
    keyword: normalizeFilter(params?.keyword),
  };
}

export const workbenchApi = {
  testDb: (settings: MySqlSettings) => invoke<CommandAck>('db_test_connection', { settings }),
  initDb: (settings: MySqlSettings) => invoke<CommandAck>('db_initialize', { settings }),
  seedConfig: (settings: MySqlSettings) => invoke<CommandAck>('config_seed_defaults', { settings }),
  probeCsv: (path: string) => invoke('import_probe_csv', { path }),
  createBatch: (settings: MySqlSettings, dataType: string, filePath: string, batchDisplayName?: string) =>
    invoke<ImportBatchResult>('import_create_batch', { req: { settings, data_type: dataType, file_path: filePath, batch_display_name: batchDisplayName?.trim() || undefined } }),
  validateMapping: (settings: MySqlSettings, importBatchId: string, dataType: string, filePath: string) =>
    invoke<CommandAck>('import_validate_mapping', { settings, importBatchId, dataType, filePath }),
  loadRaw: (settings: MySqlSettings, importBatchId: string, dataType: string, filePath: string, mode: string) =>
    invoke<CommandAck>('import_start_raw_load', { req: { settings, import_batch_id: importBatchId, data_type: dataType, file_path: filePath, mode } }),
  importStatus: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('import_get_batch_status', { settings, importBatchId }),
  importMappings: (settings: MySqlSettings, dataType: string) => invoke<MetricCard[]>('config_get_import_mappings', { settings, dataType }),
  joinRules: (settings: MySqlSettings) => invoke<MetricCard[]>('config_get_join_rules', { settings }),
  qualityBasic: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('quality_get_batch_report', { settings, importBatchId }),
  qualityGate: (settings: MySqlSettings, importBatchId: string) => invoke<CommandAck>('quality_run_gate', { req: { settings, import_batch_id: importBatchId } }),
  jobs: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('etl_get_recent_jobs', { settings, importBatchId }),
  clean: (settings: MySqlSettings, importBatchId: string) => invoke<CommandAck>('etl_start_clean_job', { req: { settings, import_batch_id: importBatchId } }),
  aggregate: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<CommandAck>('etl_start_aggregate_job', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  overview: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<DashboardOverview>('dashboard_get_overview', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  appCategory: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_app_category', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  experience: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_experience_quality', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  cableFiber: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_cable_fiber_compare', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  fuse: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<CommandAck>('leads_run_final_fusion', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  leadSummary: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('leads_get_final_summary', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  leads: (settings: MySqlSettings, analysisRunId: string, params?: LeadQueryParams) =>
    invoke<LeadUserRow[]>('leads_query_users', { req: leadQueryRequest(settings, analysisRunId, params) }),
  finalLeads: (settings: MySqlSettings, analysisRunId: string, params?: LeadQueryParams) =>
    invoke<FinalLeadUserRow[]>('final_leads_query_users', { req: leadQueryRequest(settings, analysisRunId, params) }),
  exportLeads: (settings: MySqlSettings, analysisRunId: string, outputPath: string) =>
    invoke<CommandAck>('export_leads_csv', { req: { settings, analysis_run_id: analysisRunId, output_path: outputPath } }),
  exportFinal: (settings: MySqlSettings, analysisRunId: string, outputPath: string, options?: FinalLeadExportOptions) =>
    invoke<CommandAck>('export_final_leads_csv', { req: { settings, analysis_run_id: analysisRunId, output_path: outputPath, final_actions: options?.finalActions?.length ? options.finalActions : undefined } }),
};

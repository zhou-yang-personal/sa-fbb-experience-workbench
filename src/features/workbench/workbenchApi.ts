import { invoke } from '@tauri-apps/api/core';
import type { BatchListItem, BatchTableRegistryRow, CommandAck, DashboardOverview, FinalLeadExportOptions, FinalLeadUserRow, ImportBatchResult, ImportCurrentFileResult, LeadQueryParams, LeadUserRow, MetricCard, ModuleStatusRow, MySqlSettings } from '../../shared/types';

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
  listBatches: (settings: MySqlSettings, dataType?: string) => invoke<BatchListItem[]>('import_list_batches', { settings, dataType }),
  prepareBatchTables: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('analysis_prepare_batch_tables', { settings, importBatchId }),
  batchTableRegistry: (settings: MySqlSettings, importBatchId: string) => invoke<BatchTableRegistryRow[]>('batch_get_table_registry', { settings, importBatchId }),
  moduleStatus: (settings: MySqlSettings, importBatchId: string, analysisRunId?: string) => invoke<ModuleStatusRow[]>('analysis_get_module_status', { settings, importBatchId, analysisRunId }),
  moduleMetrics: (settings: MySqlSettings, importBatchId: string, analysisRunId?: string) => invoke<MetricCard[]>('analysis_get_module_metrics', { settings, importBatchId, analysisRunId }),
  exportModule: (settings: MySqlSettings, importBatchId: string, analysisRunId: string | undefined, moduleId: string, outputPath: string) =>
    invoke<CommandAck>('analysis_export_module_csv', { settings, importBatchId, analysisRunId, moduleId, outputPath }),
  seedConfig: (settings: MySqlSettings) => invoke<CommandAck>('config_seed_defaults', { settings }),
  checkImportCatalog: (settings: MySqlSettings) => invoke<MetricCard[]>('config_check_import_catalog', { settings }),
  probeCsv: (path: string) => invoke('import_probe_csv', { path }),
  createBatch: (settings: MySqlSettings, dataType: string, filePath: string, batchDisplayName?: string) =>
    invoke<ImportBatchResult>('import_create_batch', { req: { settings, data_type: dataType, file_path: filePath, batch_display_name: batchDisplayName?.trim() || undefined } }),
  validateMapping: (settings: MySqlSettings, importBatchId: string, dataType: string, filePath: string) =>
    invoke<CommandAck>('import_validate_mapping', { settings, importBatchId, dataType, filePath }),
  loadRaw: (settings: MySqlSettings, importBatchId: string, dataType: string, filePath: string, mode: string) =>
    invoke<CommandAck>('import_start_raw_load', { req: { settings, import_batch_id: importBatchId, data_type: dataType, file_path: filePath, mode } }),
  importCurrentFile: (settings: MySqlSettings, dataType: string, filePath: string, batchDisplayName: string, mode: string) =>
    invoke<ImportCurrentFileResult>('import_current_file_atomic', { req: { settings, data_type: dataType, file_path: filePath, batch_display_name: batchDisplayName, mode } }),
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
  gameExperience: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_game_experience', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  networkQuality: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_network_quality', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  userProfile: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_user_profile', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  videoDetail: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_video_experience_detail', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  cableFiber: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_cable_fiber_compare', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  cableFiberHourly: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_cable_fiber_hourly_detail', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
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

import { invoke } from '@tauri-apps/api/core';
import type { AccessIpRangeRow, AccessRuleInput, AccessRulePreviewResult, AccessRuleSetRow, AccessRuleValidationResult, AnalysisRunOption, AppExperienceProfileRow, BatchListItem, BatchTableRegistryRow, CommandAck, CsvProbeResult, DashboardOverview, DecisionRuleProfileRow, DuckDbPocRequest, DuckDbPocResult, DuckDbWorkspaceSettings, DuckDbWorkspaceStatus, ExperiencePolicyRow, FinalLeadExportOptions, FinalLeadUserRow, ImportBatchResult, ImportCurrentFileResult, ImportPipelineLogRow, ImportPipelineStartResult, ImportPipelineStatus, LeadQueryParams, LeadUserRow, MetricCard, ModuleStatusRow, MySqlSettings } from '../../shared/types';

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
  initializeDuckDbWorkspace: (settings: DuckDbWorkspaceSettings) =>
    invoke<DuckDbWorkspaceStatus>('duckdb_workspace_initialize', { settings }),
  duckDbWorkspaceStatus: (settings: DuckDbWorkspaceSettings) =>
    invoke<DuckDbWorkspaceStatus>('duckdb_workspace_status', { settings }),
  analyzeCsvWithDuckDb: (req: DuckDbPocRequest) =>
    invoke<DuckDbPocResult>('duckdb_poc_analyze_csv', { req }),
  testDb: (settings: MySqlSettings) => invoke<CommandAck>('db_test_connection', { settings }),
  initDb: (settings: MySqlSettings) => invoke<CommandAck>('db_initialize', { settings }),
  accessRuleSets: (settings: MySqlSettings) => invoke<AccessRuleSetRow[]>('access_rule_list_sets', { settings }),
  accessRuleDraft: (settings: MySqlSettings) => invoke<AccessRuleSetRow>('access_rule_get_or_create_draft', { settings }),
  accessRules: (settings: MySqlSettings, ruleSetId: string) => invoke<AccessIpRangeRow[]>('access_rule_list', { settings, ruleSetId }),
  updateAccessRuleDefault: (settings: MySqlSettings, ruleSetId: string, defaultAccessType: string) => invoke<AccessRuleSetRow>('access_rule_set_default_update', { req: { settings, rule_set_id: ruleSetId, default_access_type: defaultAccessType } }),
  saveAccessRule: (settings: MySqlSettings, input: AccessRuleInput) => invoke<AccessIpRangeRow>('access_rule_upsert', { req: {
    settings,
    rule_set_id: input.ruleSetId,
    rule_id: input.ruleId,
    rule_name: input.ruleName,
    cidr: input.cidr?.trim() || undefined,
    start_ip: input.startIp?.trim() || undefined,
    end_ip: input.endIp?.trim() || undefined,
    access_type: input.accessType,
    priority: input.priority ?? 100,
    enabled: input.enabled ?? true,
    notes: input.notes?.trim() || undefined,
  } }),
  deleteAccessRule: (settings: MySqlSettings, ruleSetId: string, ruleId: string) => invoke<CommandAck>('access_rule_delete', { req: { settings, rule_set_id: ruleSetId, rule_id: ruleId } }),
  validateAccessRules: (settings: MySqlSettings, ruleSetId: string) => invoke<AccessRuleValidationResult>('access_rule_validate', { settings, ruleSetId }),
  publishAccessRules: (settings: MySqlSettings, ruleSetId: string) => invoke<AccessRuleSetRow>('access_rule_publish', { req: { settings, rule_set_id: ruleSetId } }),
  applyAccessRulesToBatch: (settings: MySqlSettings, ruleSetId: string, importBatchId: string) => invoke<CommandAck>('access_rule_apply_to_batch', { req: { settings, rule_set_id: ruleSetId, import_batch_id: importBatchId } }),
  previewAccessRules: (settings: MySqlSettings, ruleSetId: string, importBatchId: string, sampleLimit = 50_000) => invoke<AccessRulePreviewResult>('access_rule_preview', { req: { settings, rule_set_id: ruleSetId, import_batch_id: importBatchId, sample_limit: sampleLimit } }),
  experiencePolicies: (settings: MySqlSettings) => invoke<ExperiencePolicyRow[]>('experience_policy_list', { settings }),
  experienceProfiles: (settings: MySqlSettings, policyId: string) => invoke<AppExperienceProfileRow[]>('experience_profile_list', { req: { settings, policy_id: policyId } }),
  createExperiencePolicyDraft: (settings: MySqlSettings) => invoke<CommandAck>('experience_policy_create_draft', { settings }),
  updateExperiencePolicy: (settings: MySqlSettings, policy: ExperiencePolicyRow) => invoke<CommandAck>('experience_policy_update', { req: { settings, ...policy } }),
  updateExperienceProfile: (settings: MySqlSettings, profile: AppExperienceProfileRow) => invoke<CommandAck>('experience_profile_update', { req: { settings, ...profile } }),
  cloneExperienceProfile: (settings: MySqlSettings, policyId: string, sourceProfileId: string) => invoke<CommandAck>('experience_profile_clone', { req: { settings, policy_id: policyId, source_profile_id: sourceProfileId } }),
  publishExperiencePolicy: (settings: MySqlSettings, policyId: string) => invoke<CommandAck>('experience_policy_publish', { req: { settings, policy_id: policyId } }),
  decisionRules: (settings: MySqlSettings) => invoke<DecisionRuleProfileRow[]>('decision_rule_list', { settings }),
  createDecisionRuleDraft: (settings: MySqlSettings) => invoke<CommandAck>('decision_rule_create_draft', { settings }),
  updateDecisionRule: (settings: MySqlSettings, rule: DecisionRuleProfileRow) => invoke<CommandAck>('decision_rule_update', { settings, rule }),
  publishDecisionRule: (settings: MySqlSettings, ruleProfileId: string) => invoke<CommandAck>('decision_rule_publish', { settings, ruleProfileId }),
  listBatches: (settings: MySqlSettings, dataType?: string) => invoke<BatchListItem[]>('import_list_batches', { settings, dataType }),
  listAnalysisRuns: (settings: MySqlSettings, importBatchId: string) => invoke<AnalysisRunOption[]>('analysis_list_runs', { settings, importBatchId }),
  deleteBatch: (settings: MySqlSettings, importBatchId: string) => invoke<CommandAck>('import_delete_batch', { req: { settings, import_batch_id: importBatchId } }),
  prepareBatchTables: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('analysis_prepare_batch_tables', { settings, importBatchId }),
  batchTableRegistry: (settings: MySqlSettings, importBatchId: string) => invoke<BatchTableRegistryRow[]>('batch_get_table_registry', { settings, importBatchId }),
  cachedBatchTableRegistry: (settings: MySqlSettings, importBatchId: string) => invoke<BatchTableRegistryRow[]>('batch_get_table_registry_cached', { settings, importBatchId }),
  moduleStatus: (settings: MySqlSettings, importBatchId: string, analysisRunId?: string) => invoke<ModuleStatusRow[]>('analysis_get_module_status', { settings, importBatchId, analysisRunId }),
  moduleMetrics: (settings: MySqlSettings, importBatchId: string, analysisRunId?: string) => invoke<MetricCard[]>('analysis_get_module_metrics', { settings, importBatchId, analysisRunId }),
  exportModule: (settings: MySqlSettings, importBatchId: string, analysisRunId: string | undefined, moduleId: string, outputPath: string) =>
    invoke<CommandAck>('analysis_export_module_csv', { settings, importBatchId, analysisRunId, moduleId, outputPath }),
  seedConfig: (settings: MySqlSettings) => invoke<CommandAck>('config_seed_defaults', { settings }),
  checkImportCatalog: (settings: MySqlSettings) => invoke<MetricCard[]>('config_check_import_catalog', { settings }),
  probeCsv: (path: string) => invoke<CsvProbeResult>('import_probe_csv', { path }),
  createBatch: (settings: MySqlSettings, dataType: string, filePath: string, batchDisplayName?: string, accessRuleSetId?: string) =>
    invoke<ImportBatchResult>('import_create_batch', { req: { settings, data_type: dataType, file_path: filePath, batch_display_name: batchDisplayName?.trim() || undefined, access_rule_set_id: accessRuleSetId?.trim() || undefined } }),
  validateMapping: (settings: MySqlSettings, importBatchId: string, dataType: string, filePath: string) =>
    invoke<CommandAck>('import_validate_mapping', { settings, importBatchId, dataType, filePath }),
  loadRaw: (settings: MySqlSettings, importBatchId: string, dataType: string, filePath: string, mode: string) =>
    invoke<CommandAck>('import_start_raw_load', { req: { settings, import_batch_id: importBatchId, data_type: dataType, file_path: filePath, mode } }),
  importCurrentFile: (settings: MySqlSettings, dataType: string, filePath: string, batchDisplayName: string, mode: string, accessRuleSetId?: string) =>
    invoke<ImportCurrentFileResult>('import_current_file_atomic', { req: { settings, data_type: dataType, file_path: filePath, batch_display_name: batchDisplayName, mode, access_rule_set_id: accessRuleSetId?.trim() || undefined } }),
  pipelineStart: (settings: MySqlSettings, dataType: string, filePath: string, batchDisplayName: string, importMode: string, analysisRunId?: string, accessRuleSetId?: string) =>
    invoke<ImportPipelineStartResult>('import_pipeline_start', { req: { settings, data_type: dataType, file_path: filePath, batch_display_name: batchDisplayName, import_mode: importMode, analysis_run_id: analysisRunId, access_rule_set_id: accessRuleSetId?.trim() || undefined } }),
  pipelineResume: (settings: MySqlSettings, importBatchId: string, analysisRunId?: string, confirmOriginalProcessStopped = false) =>
    invoke<ImportPipelineStartResult>('import_pipeline_resume_batch', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId?.trim() || undefined, confirm_original_process_stopped: confirmOriginalProcessStopped } }),
  pipelineRebuildFromRaw: (settings: MySqlSettings, importBatchId: string, confirmOriginalProcessStopped = false) =>
    invoke<ImportPipelineStartResult>('import_pipeline_rebuild_batch_from_raw', { req: { settings, import_batch_id: importBatchId, confirm_original_process_stopped: confirmOriginalProcessStopped } }),
  pipelineStatus: (settings: MySqlSettings, pipelineRunId: string) =>
    invoke<ImportPipelineStatus>('import_pipeline_get_status', { req: { settings, pipeline_run_id: pipelineRunId } }),
  pipelineLogs: (settings: MySqlSettings, pipelineRunId: string, afterSequence?: number) =>
    invoke<ImportPipelineLogRow[]>('import_pipeline_get_logs', { req: { settings, pipeline_run_id: pipelineRunId, after_sequence: afterSequence } }),
  importStatus: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('import_get_batch_status', { settings, importBatchId }),
  importMappings: (settings: MySqlSettings, dataType: string) => invoke<MetricCard[]>('config_get_import_mappings', { settings, dataType }),
  joinRules: (settings: MySqlSettings) => invoke<MetricCard[]>('config_get_join_rules', { settings }),
  qualityBasic: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('quality_get_batch_report', { settings, importBatchId }),
  qualityGate: (settings: MySqlSettings, importBatchId: string) => invoke<CommandAck>('quality_run_gate', { req: { settings, import_batch_id: importBatchId } }),
  jobs: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('etl_get_recent_jobs', { settings, importBatchId }),
  clean: (settings: MySqlSettings, importBatchId: string) => invoke<CommandAck>('etl_start_clean_job', { req: { settings, import_batch_id: importBatchId } }),
  aggregate: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<CommandAck>('etl_start_aggregate_job', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  completeAggregates: (settings: MySqlSettings, importBatchId: string, analysisRunId?: string) =>
    invoke<CommandAck>('etl_run_complete_aggregates', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  completeDashboards: (settings: MySqlSettings, importBatchId: string, analysisRunId?: string) =>
    invoke<CommandAck>('ads_run_complete_dashboards', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  analyticsKpis: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('analytics_get_kpi_summary', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
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

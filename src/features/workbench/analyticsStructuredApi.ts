import { invoke } from '@tauri-apps/api/core';
import type { AnalysisContext, CommandAck, DataCoverageItemV2, ExperienceFinding, ExperienceStatusV2, InvestigationEvidenceRow, InvestigationHourlyRow, InvestigationServerIpRow, MetricCard, MySqlSettings, OpportunityCandidatePage, RunVerificationV2, SavedInvestigation } from '../../shared/types';

export type StructuredAnalyticsQuery = {
  page?: number;
  pageSize?: number;
  keyword?: string;
  sortBy?: string;
  minValue?: number;
  opportunityType?: string;
};

function req(settings: MySqlSettings, importBatchId: string, analysisRunId: string, query: StructuredAnalyticsQuery = {}) {
  return { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId, page: query.page, page_size: query.pageSize, keyword: query.keyword, sort_by: query.sortBy, min_value: query.minValue, opportunity_type: query.opportunityType } };
}

function etlReq(settings: MySqlSettings, importBatchId: string, analysisRunId: string) {
  return { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } };
}

export const analyticsStructuredApi = {
  experienceStatusV2: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<ExperienceStatusV2>('analytics_get_experience_status_v2', req(settings, importBatchId, analysisRunId)),
  findingsV2: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, pageSize = 100) => invoke<ExperienceFinding[]>('analytics_get_findings_v2', req(settings, importBatchId, analysisRunId, { pageSize })),
  coverageV2: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<DataCoverageItemV2[]>('analytics_get_data_coverage_v2', req(settings, importBatchId, analysisRunId)),
  runVerificationV2: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<RunVerificationV2>('analytics_get_run_verification_v2', req(settings, importBatchId, analysisRunId)),
  investigationEvidence: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, context: AnalysisContext, pageSize = 100) => invoke<InvestigationEvidenceRow[]>('analytics_get_investigation_evidence', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId, app_category: context.app_category, app_name: context.app_name, access_type: context.access_type, user_key: context.user_key, date_from: context.date_from, date_to: context.date_to, hour_from: context.hour_from, hour_to: context.hour_to, page_size: pageSize } }),
  investigationHourly: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, context: AnalysisContext, pageSize = 500) => invoke<InvestigationHourlyRow[]>('analytics_get_investigation_hourly', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId, app_category: context.app_category, app_name: context.app_name, access_type: context.access_type, date_from: context.date_from, date_to: context.date_to, hour_from: context.hour_from, hour_to: context.hour_to, page_size: pageSize } }),
  investigationServerIps: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, context: AnalysisContext, pageSize = 50) => invoke<InvestigationServerIpRow[]>('analytics_get_investigation_server_ips', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId, app_category: context.app_category, app_name: context.app_name, access_type: context.access_type, user_key: context.user_key, date_from: context.date_from, date_to: context.date_to, hour_from: context.hour_from, hour_to: context.hour_to, page_size: pageSize } }),
  saveInvestigation: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, title: string, context: AnalysisContext, notes?: string) => invoke<SavedInvestigation>('investigation_save', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId, finding_id: context.finding_id, title, status: 'open', context_json: JSON.stringify(context), notes } }),
  investigations: (settings: MySqlSettings, importBatchId: string, analysisRunId?: string) => invoke<SavedInvestigation[]>('investigation_list', { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } }),
  coverage: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('analytics_get_data_coverage', req(settings, importBatchId, analysisRunId)),
  kpis: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('analytics_get_kpi_summary', req(settings, importBatchId, analysisRunId, query)),
  appRank: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('analytics_get_app_rank', req(settings, importBatchId, analysisRunId, query)),
  hourlyTrend: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('analytics_get_hourly_trend', req(settings, importBatchId, analysisRunId, query)),
  networkHotspots: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('analytics_get_network_hotspots', req(settings, importBatchId, analysisRunId, query)),
  userProfiles: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('analytics_get_user_profiles', req(settings, importBatchId, analysisRunId, query)),
  userSummary: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('analytics_get_user_summary', req(settings, importBatchId, analysisRunId)),
  leadEvidence: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('analytics_get_lead_evidence_page', req(settings, importBatchId, analysisRunId, query)),
  leadSummary: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('analytics_get_lead_stage_summary', req(settings, importBatchId, analysisRunId)),
  materializeAppRank: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<CommandAck>('analytics_materialize_app_rank', etlReq(settings, importBatchId, analysisRunId)),
  materializeHourly: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<CommandAck>('ads_hour', etlReq(settings, importBatchId, analysisRunId)),
  materializeUser: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<CommandAck>('ads_user', etlReq(settings, importBatchId, analysisRunId)),
  materializeLead: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<CommandAck>('ads_lead', etlReq(settings, importBatchId, analysisRunId)),
  materializeNetwork: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<CommandAck>('ads_net', etlReq(settings, importBatchId, analysisRunId)),
  decisionMetricPanorama: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('decision_get_metric_panorama', req(settings, importBatchId, analysisRunId)),
  decisionAppPanorama: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('decision_get_app_panorama', req(settings, importBatchId, analysisRunId, query)),
  decisionUserDistributions: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('decision_get_user_distributions', req(settings, importBatchId, analysisRunId, query)),
  decisionQualityOverview: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('decision_get_quality_overview', req(settings, importBatchId, analysisRunId)),
  decisionAccessCompare: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('decision_get_access_compare', req(settings, importBatchId, analysisRunId)),
  decisionAccessHourly: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('decision_get_access_hourly', req(settings, importBatchId, analysisRunId)),
  decisionPanoramaHourly: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('decision_get_panorama_hourly', req(settings, importBatchId, analysisRunId)),
  decisionAccessUserBands: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('decision_get_access_user_bands', req(settings, importBatchId, analysisRunId)),
  decisionAccessApps: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<MetricCard[]>('decision_get_access_apps', req(settings, importBatchId, analysisRunId, query)),
  decisionMaterializeOpportunities: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<CommandAck>('decision_materialize_opportunities', etlReq(settings, importBatchId, analysisRunId)),
  decisionOpportunities: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => invoke<MetricCard[]>('decision_get_opportunities', req(settings, importBatchId, analysisRunId)),
  decisionOpportunityCandidates: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) => invoke<OpportunityCandidatePage>('decision_get_opportunity_candidates', req(settings, importBatchId, analysisRunId, query)),
};

import { invoke } from '@tauri-apps/api/core';
import type { CommandAck, MetricCard, MySqlSettings } from '../../shared/types';

export type StructuredAnalyticsQuery = {
  page?: number;
  pageSize?: number;
  keyword?: string;
  sortBy?: string;
  minValue?: number;
};

function req(settings: MySqlSettings, importBatchId: string, analysisRunId: string, query: StructuredAnalyticsQuery = {}) {
  return {
    req: {
      settings,
      import_batch_id: importBatchId,
      analysis_run_id: analysisRunId,
      page: query.page,
      page_size: query.pageSize,
      keyword: query.keyword,
      sort_by: query.sortBy,
      min_value: query.minValue,
    },
  };
}

function etlReq(settings: MySqlSettings, importBatchId: string, analysisRunId: string) {
  return { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } };
}

export const analyticsStructuredApi = {
  kpis: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) =>
    invoke<MetricCard[]>('analytics_get_kpi_summary', req(settings, importBatchId, analysisRunId, query)),
  appRank: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) =>
    invoke<MetricCard[]>('analytics_get_app_rank', req(settings, importBatchId, analysisRunId, query)),
  hourlyTrend: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) =>
    invoke<MetricCard[]>('analytics_get_hourly_trend', req(settings, importBatchId, analysisRunId, query)),
  networkHotspots: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) =>
    invoke<MetricCard[]>('analytics_get_network_hotspots', req(settings, importBatchId, analysisRunId, query)),
  userProfiles: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) =>
    invoke<MetricCard[]>('analytics_get_user_profiles', req(settings, importBatchId, analysisRunId, query)),
  leadEvidence: (settings: MySqlSettings, importBatchId: string, analysisRunId: string, query?: StructuredAnalyticsQuery) =>
    invoke<MetricCard[]>('analytics_get_lead_evidence_page', req(settings, importBatchId, analysisRunId, query)),
  materializeAppRank: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<CommandAck>('analytics_materialize_app_rank', etlReq(settings, importBatchId, analysisRunId)),
};

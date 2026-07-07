import { invoke } from '@tauri-apps/api/core';
import type { MetricCard, MySqlSettings } from '../../shared/types';

function req(settings: MySqlSettings, importBatchId: string, analysisRunId: string) {
  return { req: { settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId } };
}

export const analyticsStructuredApi = {
  kpis: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('analytics_get_kpi_summary', req(settings, importBatchId, analysisRunId)),
  appRank: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('analytics_get_app_rank', req(settings, importBatchId, analysisRunId)),
  hourlyTrend: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('analytics_get_hourly_trend', req(settings, importBatchId, analysisRunId)),
  networkHotspots: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('analytics_get_network_hotspots', req(settings, importBatchId, analysisRunId)),
  userProfiles: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('analytics_get_user_profiles', req(settings, importBatchId, analysisRunId)),
  leadEvidence: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('analytics_get_lead_evidence', req(settings, importBatchId, analysisRunId)),
};

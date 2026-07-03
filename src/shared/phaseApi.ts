import { invoke } from '@tauri-apps/api/core';
import type { CommandAck, MetricCard, MySqlSettings } from './types';

const etlReq = (settings: MySqlSettings, importBatchId: string, analysisRunId?: string) => ({ settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId });
const dashReq = (settings: MySqlSettings, importBatchId: string, analysisRunId: string) => ({ settings, import_batch_id: importBatchId, analysis_run_id: analysisRunId });

export const phaseApi = {
  qualityRunGate: (settings: MySqlSettings, importBatchId: string) =>
    invoke<CommandAck>('quality_run_gate', { req: etlReq(settings, importBatchId) }),
  etlRunCompleteAggregates: (settings: MySqlSettings, importBatchId: string) =>
    invoke<CommandAck>('etl_run_complete_aggregates', { req: etlReq(settings, importBatchId) }),
  adsRunCompleteDashboards: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<CommandAck>('ads_run_complete_dashboards', { req: etlReq(settings, importBatchId, analysisRunId) }),
  leadsRunFinalFusion: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<CommandAck>('leads_run_final_fusion', { req: etlReq(settings, importBatchId, analysisRunId) }),
  dashboardGetAppCategory: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_app_category', { req: dashReq(settings, importBatchId, analysisRunId) }),
  dashboardGetExperienceQuality: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_experience_quality', { req: dashReq(settings, importBatchId, analysisRunId) }),
  dashboardGetCableFiberCompare: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('dashboard_get_cable_fiber_compare', { req: dashReq(settings, importBatchId, analysisRunId) }),
  leadsGetFinalSummary: (settings: MySqlSettings, importBatchId: string, analysisRunId: string) =>
    invoke<MetricCard[]>('leads_get_final_summary', { req: dashReq(settings, importBatchId, analysisRunId) }),
};

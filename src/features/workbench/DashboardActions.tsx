import type { DashboardChartGroup, DashboardOverview, MetricCard, MySqlSettings } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
  setOverview: (value: DashboardOverview) => void;
  setDashboardCharts: (value: DashboardChartGroup[]) => void;
};

function asMetrics(result: unknown) {
  return Array.isArray(result) ? result as MetricCard[] : [];
}

export function DashboardActions({ settings, importBatchId, analysisRunId, runAction, loadMetrics, setOverview, setDashboardCharts }: Props) {
  async function loadChart(title: string, kind: DashboardChartGroup['kind'], label: string, action: () => Promise<MetricCard[]>) {
    const result = await runAction(label, action);
    const metrics = asMetrics(result);
    setDashboardCharts([{ title, kind, metrics }]);
  }

  async function loadAllCharts() {
    const appCategory = asMetrics(await runAction('chart_app_category', () => workbenchApi.appCategory(settings, importBatchId, analysisRunId)));
    const experience = asMetrics(await runAction('chart_experience_quality', () => workbenchApi.experience(settings, importBatchId, analysisRunId)));
    const cableFiber = asMetrics(await runAction('chart_cable_fiber_compare', () => workbenchApi.cableFiber(settings, importBatchId, analysisRunId)));
    const finalAction = asMetrics(await runAction('chart_final_action_mix', () => workbenchApi.leadSummary(settings, importBatchId, analysisRunId)));
    setDashboardCharts([
      { title: 'App Category', kind: 'bar', metrics: appCategory },
      { title: 'Experience Quality', kind: 'radar', metrics: experience },
      { title: 'Cable vs FTTH', kind: 'bar', metrics: cableFiber },
      { title: 'Final Action Mix', kind: 'bar', metrics: finalAction },
    ]);
  }

  return (
    <>
      <button onClick={() => loadMetrics('config_get_join_rules', () => workbenchApi.joinRules(settings))}>Join规则</button>
      <button onClick={async () => { const result = await runAction('dashboard_get_overview', () => workbenchApi.overview(settings, importBatchId, analysisRunId)); if (result && typeof result === 'object' && 'metrics' in result) setOverview(result as DashboardOverview); }}>Overview</button>
      <button onClick={() => loadChart('App Category', 'bar', 'app_category_chart', () => workbenchApi.appCategory(settings, importBatchId, analysisRunId))}>应用图表</button>
      <button onClick={() => loadChart('Experience Quality', 'radar', 'experience_quality_chart', () => workbenchApi.experience(settings, importBatchId, analysisRunId))}>体验图表</button>
      <button onClick={() => loadChart('Cable vs FTTH', 'bar', 'cable_fiber_chart', () => workbenchApi.cableFiber(settings, importBatchId, analysisRunId))}>Cable/FTTH图表</button>
      <button onClick={() => loadChart('Final Action Mix', 'bar', 'final_action_mix_chart', () => workbenchApi.leadSummary(settings, importBatchId, analysisRunId))}>Action Mix图表</button>
      <button onClick={loadAllCharts}>加载多图表</button>
    </>
  );
}

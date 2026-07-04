import type { ActionState, DashboardChartGroup, DashboardOverview, MetricCard, MySqlSettings } from '../../shared/types';
import { ActionButton } from './ActionButton';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  actionStates: Record<string, ActionState>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
  setOverview: (value: DashboardOverview) => void;
  setDashboardCharts: (value: DashboardChartGroup[]) => void;
};

function asMetrics(result: unknown) { return Array.isArray(result) ? result as MetricCard[] : []; }

export function DashboardActions({ settings, importBatchId, analysisRunId, actionStates, runAction, loadMetrics, setOverview, setDashboardCharts }: Props) {
  const disabled = !importBatchId.trim() || !analysisRunId.trim();
  async function loadChart(title: string, kind: DashboardChartGroup['kind'], label: string, action: () => Promise<MetricCard[]>) {
    const result = await runAction(label, action);
    setDashboardCharts([{ title, kind, metrics: asMetrics(result) }]);
  }

  async function loadAllCharts() {
    await runAction('dashboard_refresh_all', async () => {
      const overview = await workbenchApi.overview(settings, importBatchId, analysisRunId);
      const appCategory = await workbenchApi.appCategory(settings, importBatchId, analysisRunId);
      const experience = await workbenchApi.experience(settings, importBatchId, analysisRunId);
      const cableFiber = await workbenchApi.cableFiber(settings, importBatchId, analysisRunId);
      const finalAction = await workbenchApi.leadSummary(settings, importBatchId, analysisRunId);
      setOverview(overview);
      setDashboardCharts([
        { title: 'App Category', kind: 'bar', metrics: appCategory },
        { title: 'Experience Quality', kind: 'radar', metrics: experience },
        { title: 'Cable vs FTTH', kind: 'bar', metrics: cableFiber },
        { title: 'Final Action Mix', kind: 'bar', metrics: finalAction },
      ]);
      return { overview, appCategory, experience, cableFiber, finalAction };
    });
  }

  return (
    <>
      <ActionButton actionKey="dashboard_refresh_all" actionStates={actionStates} primary label="刷新全部结果" disabled={disabled} onClick={loadAllCharts} />
      <details className="advanced-actions inline-advanced-actions">
        <summary>高级图表</summary>
        <div className="action-row">
          <ActionButton actionKey="config_get_join_rules" actionStates={actionStates} label="Join规则" onClick={() => loadMetrics('config_get_join_rules', () => workbenchApi.joinRules(settings))} />
          <ActionButton actionKey="dashboard_get_overview" actionStates={actionStates} label="Overview" disabled={disabled} onClick={async () => { const result = await runAction('dashboard_get_overview', () => workbenchApi.overview(settings, importBatchId, analysisRunId)); if (result && typeof result === 'object' && 'metrics' in result) setOverview(result as DashboardOverview); }} />
          <ActionButton actionKey="app_category_chart" actionStates={actionStates} label="应用图表" disabled={disabled} onClick={() => loadChart('App Category', 'bar', 'app_category_chart', () => workbenchApi.appCategory(settings, importBatchId, analysisRunId))} />
          <ActionButton actionKey="experience_quality_chart" actionStates={actionStates} label="体验图表" disabled={disabled} onClick={() => loadChart('Experience Quality', 'radar', 'experience_quality_chart', () => workbenchApi.experience(settings, importBatchId, analysisRunId))} />
          <ActionButton actionKey="cable_fiber_chart" actionStates={actionStates} label="Cable/FTTH图表" disabled={disabled} onClick={() => loadChart('Cable vs FTTH', 'bar', 'cable_fiber_chart', () => workbenchApi.cableFiber(settings, importBatchId, analysisRunId))} />
          <ActionButton actionKey="final_action_mix_chart" actionStates={actionStates} label="Action Mix图表" disabled={disabled} onClick={() => loadChart('Final Action Mix', 'bar', 'final_action_mix_chart', () => workbenchApi.leadSummary(settings, importBatchId, analysisRunId))} />
        </div>
      </details>
    </>
  );
}

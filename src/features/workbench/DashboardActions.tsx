import type { DashboardOverview, MetricCard, MySqlSettings } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
  setOverview: (value: DashboardOverview) => void;
};

export function DashboardActions({ settings, importBatchId, analysisRunId, runAction, loadMetrics, setOverview }: Props) {
  return (
    <>
      <button onClick={() => loadMetrics('config_get_join_rules', () => workbenchApi.joinRules(settings))}>Join规则</button>
      <button onClick={async () => { const result = await runAction('dashboard_get_overview', () => workbenchApi.overview(settings, importBatchId, analysisRunId)); if (result && typeof result === 'object' && 'metrics' in result) setOverview(result as DashboardOverview); }}>Overview</button>
      <button onClick={() => loadMetrics('app_category', () => workbenchApi.appCategory(settings, importBatchId, analysisRunId))}>应用看板</button>
      <button onClick={() => loadMetrics('experience_quality', () => workbenchApi.experience(settings, importBatchId, analysisRunId))}>体验看板</button>
      <button onClick={() => loadMetrics('cable_fiber_compare', () => workbenchApi.cableFiber(settings, importBatchId, analysisRunId))}>Cable/FTTH</button>
    </>
  );
}

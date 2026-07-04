import type { ActionState, DashboardChartGroup, DashboardOverview, MetricCard, MySqlSettings } from '../../shared/types';
import { DashboardActions } from './DashboardActions';
import { DashboardCharts } from './DashboardCharts';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  dashboardCharts: DashboardChartGroup[];
  setDashboardCharts: (value: DashboardChartGroup[]) => void;
  actionStates: Record<string, ActionState>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
  setOverview: (value: DashboardOverview) => void;
};

const dashboards = [
  'Overview: users, traffic, game hours and A1 leads',
  'App Category: active users and traffic by app category',
  'Experience Quality: RTT / PLR / VMOS / MOS summary',
  'Cable vs FTTH: metric difference and access-hour comparison',
  'Final Lead Summary: action mix for commercial execution',
];

export function DashboardCenter(props: Props) {
  return (
    <section className="panel form-panel dashboard-center-panel step-card">
      <div className="step-card-head">
        <div>
          <h2>Results：Dashboard 总览</h2>
          <p className="hero-text">统一刷新 ADS / DWS 聚合结果，不直接扫描 RAW 大表。</p>
        </div>
        <span className="step-badge">5 / 5</span>
      </div>
      <div className="primary-action-row dashboard-actions">
        <DashboardActions {...props} />
      </div>
      <ol className="pipeline-list">{dashboards.map((item) => <li key={item}>{item}</li>)}</ol>
      <DashboardCharts charts={props.dashboardCharts} />
    </section>
  );
}

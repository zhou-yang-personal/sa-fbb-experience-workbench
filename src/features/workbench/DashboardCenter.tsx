import type { DashboardOverview, MetricCard, MySqlSettings } from '../../shared/types';
import { DashboardActions } from './DashboardActions';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
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
    <section className="panel form-panel">
      <h2>Dashboard Center</h2>
      <p className="hero-text">只读取 ADS / DWS 聚合结果，不直接扫描 RAW 大表。当前 MetricGrid 会同步展示卡片与 ECharts bar panel。</p>
      <div className="action-row">
        <DashboardActions {...props} />
      </div>
      <ol className="pipeline-list">
        {dashboards.map((item) => <li key={item}>{item}</li>)}
      </ol>
    </section>
  );
}

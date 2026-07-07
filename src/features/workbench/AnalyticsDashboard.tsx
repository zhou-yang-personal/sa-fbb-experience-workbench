import { useEffect, useMemo, useRef, useState } from 'react';
import * as echarts from 'echarts';
import type { DashboardOverview, MetricCard } from '../../shared/types';
import { workbenchApi } from './workbenchApi';
import type { WorkbenchController } from './useWorkbenchController';

type ChartKind = 'bar' | 'donut' | 'radar' | 'line';
type AnalyticsTab = 'overview' | 'apps' | 'quality' | 'cable' | 'users' | 'leads';

type MetricDataset = {
  overview: MetricCard[];
  appCategory: MetricCard[];
  experience: MetricCard[];
  networkQuality: MetricCard[];
  video: MetricCard[];
  game: MetricCard[];
  cableFiber: MetricCard[];
  cableHourly: MetricCard[];
  users: MetricCard[];
  leads: MetricCard[];
};

const emptyDataset: MetricDataset = {
  overview: [],
  appCategory: [],
  experience: [],
  networkQuality: [],
  video: [],
  game: [],
  cableFiber: [],
  cableHourly: [],
  users: [],
  leads: [],
};

const tabs: Array<{ id: AnalyticsTab; label: string; hint: string }> = [
  { id: 'overview', label: '总览驾驶舱', hint: 'KPI / 流量 / 体验风险 / Lead' },
  { id: 'apps', label: '应用体验', hint: 'Top App / 视频 / 游戏' },
  { id: 'quality', label: '网络质量', hint: 'RTT / PLR / MOS / VMOS' },
  { id: 'cable', label: 'Cable vs FTTH', hint: '接入差异 / 小时趋势' },
  { id: 'users', label: '用户画像', hint: '用户需求 / 体验证据' },
  { id: 'leads', label: '迁转升套机会', hint: 'Lead Funnel / action mix' },
];

function toNumber(value: string | number | undefined) {
  if (typeof value === 'number') return Number.isFinite(value) ? value : 0;
  const parsed = Number.parseFloat(String(value ?? '').replace(/,/g, '').replace(/%/g, '').trim());
  return Number.isFinite(parsed) ? parsed : 0;
}

function compact(value: number) {
  if (!Number.isFinite(value)) return '-';
  if (Math.abs(value) >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (Math.abs(value) >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  if (Math.abs(value) >= 100) return value.toFixed(0);
  if (Math.abs(value) >= 10) return value.toFixed(1);
  return value.toFixed(2).replace(/\.00$/, '');
}

function normalizeLabel(value: string) {
  return value.length > 28 ? `${value.slice(0, 28)}…` : value;
}

function topRows(metrics: MetricCard[], limit = 12) {
  return [...metrics]
    .map((item) => ({ ...item, numeric: toNumber(item.value) }))
    .sort((a, b) => Math.abs(b.numeric) - Math.abs(a.numeric))
    .slice(0, limit);
}

function parseHint(hint: string) {
  return hint.split(/,\s+|\s+\|\s+/).reduce<Record<string, string>>((acc, part) => {
    const pos = part.indexOf('=');
    if (pos < 0) return acc;
    const key = part.slice(0, pos).trim();
    const value = part.slice(pos + 1).trim();
    if (key) acc[key] = value;
    return acc;
  }, {});
}

function metricByName(metrics: MetricCard[], names: string[]) {
  const lower = names.map((name) => name.toLowerCase());
  return metrics.find((metric) => lower.some((name) => metric.label.toLowerCase().includes(name)));
}

function buildKpis(data: MetricDataset) {
  const all = [...data.overview, ...data.appCategory, ...data.experience, ...data.cableFiber, ...data.leads, ...data.users];
  const primary = [
    { label: 'Users', metric: metricByName(all, ['user', 'active_users', 'total_users']) },
    { label: 'Traffic', metric: metricByName(all, ['traffic', 'download', 'gb']) },
    { label: 'Experience', metric: metricByName(all, ['vmos', 'mos', 'quality']) },
    { label: 'Cable gap', metric: metricByName(data.cableFiber, ['diff', 'gap', 'rtt', 'loss']) },
    { label: 'Lead users', metric: metricByName(data.leads, ['lead', 'action', 'a1']) },
    { label: 'Top app', metric: data.appCategory[0] },
  ];
  return primary.map((item) => ({
    label: item.label,
    value: item.metric?.value ?? '-',
    hint: item.metric?.label ?? '等待刷新',
  }));
}

function AnalyticsChart({ title, description, kind, metrics, height = 380 }: { title: string; description: string; kind: ChartKind; metrics: MetricCard[]; height?: number }) {
  const ref = useRef<HTMLDivElement | null>(null);
  const rows = useMemo(() => topRows(metrics, kind === 'line' ? 24 : 14), [metrics, kind]);

  useEffect(() => {
    if (!ref.current) return;
    const chart = echarts.init(ref.current);
    if (!rows.length) {
      chart.setOption({
        title: { text: title, textStyle: { color: '#e8eefc', fontSize: 15 } },
        graphic: { type: 'text', left: 'center', top: 'middle', style: { text: 'No aggregated data. Refresh dashboard after DWS/ADS is ready.', fill: '#94a3b8' } },
      });
    } else if (kind === 'donut') {
      chart.setOption({
        title: { text: title, subtext: description, textStyle: { color: '#e8eefc', fontSize: 15 }, subtextStyle: { color: '#94a3b8' } },
        tooltip: { trigger: 'item' },
        legend: { bottom: 0, textStyle: { color: '#cbd5e1' } },
        series: [{ type: 'pie', radius: ['46%', '70%'], center: ['50%', '48%'], data: rows.map((row) => ({ name: normalizeLabel(row.label), value: Math.abs(row.numeric) })) }],
      });
    } else if (kind === 'radar') {
      const maxValue = Math.max(...rows.map((row) => Math.abs(row.numeric)), 1);
      chart.setOption({
        title: { text: title, subtext: description, textStyle: { color: '#e8eefc', fontSize: 15 }, subtextStyle: { color: '#94a3b8' } },
        tooltip: {},
        radar: {
          radius: '62%',
          indicator: rows.slice(0, 8).map((row) => ({ name: normalizeLabel(row.label), max: maxValue })),
          axisName: { color: '#cbd5e1' },
          splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } },
          splitArea: { areaStyle: { color: ['rgba(15,23,42,0.28)', 'rgba(30,41,59,0.18)'] } },
        },
        series: [{ type: 'radar', data: [{ value: rows.slice(0, 8).map((row) => Math.abs(row.numeric)), name: title }] }],
      });
    } else if (kind === 'line') {
      chart.setOption({
        title: { text: title, subtext: description, textStyle: { color: '#e8eefc', fontSize: 15 }, subtextStyle: { color: '#94a3b8' } },
        tooltip: { trigger: 'axis' },
        grid: { left: 58, right: 24, top: 72, bottom: 58 },
        xAxis: { type: 'category', data: rows.map((row) => normalizeLabel(row.label)), axisLabel: { color: '#cbd5e1' } },
        yAxis: { type: 'value', axisLabel: { color: '#cbd5e1' }, splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } } },
        series: [{ type: 'line', smooth: true, areaStyle: { opacity: 0.16 }, data: rows.map((row) => row.numeric) }],
      });
    } else {
      chart.setOption({
        title: { text: title, subtext: description, textStyle: { color: '#e8eefc', fontSize: 15 }, subtextStyle: { color: '#94a3b8' } },
        tooltip: { trigger: 'axis' },
        grid: { left: 120, right: 24, top: 72, bottom: 36 },
        xAxis: { type: 'value', axisLabel: { color: '#cbd5e1' }, splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } } },
        yAxis: { type: 'category', data: rows.map((row) => normalizeLabel(row.label)).reverse(), axisLabel: { color: '#cbd5e1' } },
        series: [{ type: 'bar', data: rows.map((row) => row.numeric).reverse(), barMaxWidth: 22 }],
      });
    }
    const resize = () => chart.resize();
    window.addEventListener('resize', resize);
    return () => { window.removeEventListener('resize', resize); chart.dispose(); };
  }, [rows, title, description, kind]);

  return (
    <article className="analytics-card analytics-chart-card">
      <div className="analytics-chart" style={{ height }} ref={ref} />
      <div className="analytics-card-footnote">
        <span>Source: DWS / ADS commands</span>
        <span>Rows: {metrics.length}</span>
      </div>
    </article>
  );
}

function AnalyticsTable({ title, rows, limit = 12 }: { title: string; rows: MetricCard[]; limit?: number }) {
  return (
    <article className="analytics-card analytics-table-card">
      <div className="analytics-section-head">
        <div>
          <h3>{title}</h3>
          <p>聚合结果表。点击刷新后按当前 batch / analysis_run_id 加载。</p>
        </div>
        <span>{rows.length} rows</span>
      </div>
      <div className="analytics-table-wrap">
        <table className="analytics-table">
          <thead><tr><th>Rank</th><th>Name</th><th>Value</th><th>Details</th></tr></thead>
          <tbody>
            {rows.slice(0, limit).map((row, index) => {
              const hint = parseHint(row.hint);
              const detail = Object.keys(hint).length ? Object.entries(hint).slice(0, 5).map(([key, value]) => `${key}=${value}`).join(' · ') : row.hint;
              return <tr key={`${row.label}-${index}`}><td>{index + 1}</td><td>{row.label}</td><td>{row.value}</td><td title={row.hint}>{detail || '-'}</td></tr>;
            })}
            {!rows.length && <tr><td colSpan={4}>No data. Run analysis pipeline and refresh dashboard.</td></tr>}
          </tbody>
        </table>
      </div>
    </article>
  );
}

function KpiStrip({ items }: { items: MetricCard[] }) {
  return (
    <section className="analytics-kpi-strip">
      {items.map((item) => <article key={item.label} className="analytics-kpi-card"><span>{item.label}</span><strong>{item.value}</strong><small>{item.hint}</small></article>)}
    </section>
  );
}

export function AnalyticsDashboard({ c }: { c: WorkbenchController }) {
  const [activeTab, setActiveTab] = useState<AnalyticsTab>('overview');
  const [data, setData] = useState<MetricDataset>(emptyDataset);
  const [message, setMessage] = useState('点击“刷新分析驾驶舱”加载当前批次聚合结果。');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function refreshAll() {
    await c.runAction('analytics_dashboard_refresh_all', async () => {
      const [overviewResult, appCategory, experience, networkQuality, video, game, cableFiber, cableHourly, users, leads] = await Promise.all([
        workbenchApi.overview(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        workbenchApi.appCategory(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        workbenchApi.experience(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        workbenchApi.networkQuality(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        workbenchApi.videoDetail(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
        workbenchApi.gameExperience(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
        workbenchApi.cableFiber(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        workbenchApi.cableFiberHourly(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
        workbenchApi.userProfile(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
        workbenchApi.leadSummary(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
      ]);
      const overview = (overviewResult as DashboardOverview).metrics ?? [];
      c.setOverview(overviewResult as DashboardOverview);
      c.setDashboardCharts([
        { title: 'App Category Rank', kind: 'bar', metrics: appCategory },
        { title: 'Experience Quality', kind: 'radar', metrics: experience },
        { title: 'Cable vs FTTH', kind: 'bar', metrics: cableFiber },
        { title: 'Lead Mix', kind: 'bar', metrics: leads },
      ]);
      setData({ overview, appCategory, experience, networkQuality, video, game, cableFiber, cableHourly, users, leads });
      setMessage(`已刷新：overview=${overview.length}, apps=${appCategory.length}, quality=${experience.length}, cable=${cableFiber.length}, users=${users.length}, leads=${leads.length}`);
      return { overview, appCategory, experience, networkQuality, video, game, cableFiber, cableHourly, users, leads };
    });
  }

  useEffect(() => {
    if (!disabled) void refreshAll();
  }, [c.importBatchId, c.analysisRunId]);

  const kpis = useMemo(() => buildKpis(data), [data]);
  const videoOrApps = data.video.length ? data.video : data.appCategory;
  const gameOrApps = data.game.length ? data.game : data.appCategory;

  return (
    <section className="analytics-dashboard">
      <header className="analytics-hero">
        <div>
          <p className="eyebrow">Analytics cockpit · DWS / ADS only</p>
          <h2>数据分析驾驶舱</h2>
          <p>围绕当前批次构建大图、大表和业务诊断路径：总览、应用、体验质量、Cable vs FTTH、用户画像和迁转机会。</p>
        </div>
        <div className="analytics-hero-actions">
          <button type="button" disabled={disabled} onClick={refreshAll}>刷新分析驾驶舱</button>
          <span className={disabled ? 'status-pill status-failure' : 'status-pill status-success'}>{disabled ? 'batch / run missing' : 'ready'}</span>
        </div>
      </header>

      <div className="analytics-context-line">
        <span>Batch: {c.batchDisplayName || c.importBatchId || '-'}</span>
        <span>Run: {c.analysisRunId || '-'}</span>
        <span>Data: {c.dataType.toUpperCase()}</span>
        <span>{message}</span>
      </div>

      <KpiStrip items={kpis} />

      <nav className="analytics-tabs">
        {tabs.map((tab) => <button key={tab.id} type="button" className={activeTab === tab.id ? 'is-active' : ''} onClick={() => setActiveTab(tab.id)}><strong>{tab.label}</strong><small>{tab.hint}</small></button>)}
      </nav>

      {activeTab === 'overview' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Traffic / users by app category" description="Top app categories ranked by current metric value." kind="bar" metrics={data.appCategory} height={440} />
          <AnalyticsChart title="Lead / action mix" description="Opportunity distribution from SA Lead or Final Lead summary." kind="donut" metrics={data.leads} height={440} />
          <AnalyticsChart title="Experience risk summary" description="RTT / PLR / MOS / VMOS aggregate indicators." kind="radar" metrics={data.experience.length ? data.experience : data.networkQuality} height={420} />
          <AnalyticsTable title="Top application categories" rows={data.appCategory} />
        </div>
      )}

      {activeTab === 'apps' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Top apps / categories by demand" description="Traffic, users or duration depending on backend metric." kind="bar" metrics={data.appCategory} height={440} />
          <AnalyticsChart title="Video experience rank" description="VMOS, effective rate, connection delay and video QoE proxy metrics." kind="bar" metrics={videoOrApps} height={420} />
          <AnalyticsChart title="Game experience rank" description="MOS, latency, jitter, loss and game-hour proxy metrics." kind="bar" metrics={gameOrApps} height={420} />
          <AnalyticsTable title="Application detail table" rows={[...data.appCategory, ...data.video, ...data.game]} limit={18} />
        </div>
      )}

      {activeTab === 'quality' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Network quality radar" description="RTT, loss, VMOS/MOS and Wi-Fi delay dimensions." kind="radar" metrics={data.networkQuality.length ? data.networkQuality : data.experience} height={440} />
          <AnalyticsChart title="Quality dimensions ranking" description="Largest quality dimensions by absolute metric value." kind="bar" metrics={[...data.experience, ...data.networkQuality]} height={420} />
          <AnalyticsTable title="Quality metric table" rows={[...data.experience, ...data.networkQuality]} limit={20} />
        </div>
      )}

      {activeTab === 'cable' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Cable vs FTTH metric gap" description="Aggregated metric differences by access type." kind="bar" metrics={data.cableFiber} height={460} />
          <AnalyticsChart title="Hourly Cable / FTTH trend proxy" description="Hourly comparison from ADS cable-fiber detail where available." kind="line" metrics={data.cableHourly.length ? data.cableHourly : data.cableFiber} height={420} />
          <AnalyticsTable title="Cable vs FTTH comparison table" rows={[...data.cableFiber, ...data.cableHourly]} limit={24} />
        </div>
      )}

      {activeTab === 'users' && (
        <div className="analytics-layout">
          <AnalyticsChart title="User profile demand ranking" description="Top users by traffic, game hours or experience exposure." kind="bar" metrics={data.users} height={460} />
          <AnalyticsTable title="User profile table" rows={data.users} limit={30} />
        </div>
      )}

      {activeTab === 'leads' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Lead funnel / final action mix" description="Lead type or final action distribution." kind="bar" metrics={data.leads} height={460} />
          <AnalyticsChart title="Lead mix donut" description="Part-to-whole view of current opportunity buckets." kind="donut" metrics={data.leads} height={420} />
          <AnalyticsTable title="Lead evidence summary" rows={data.leads} limit={24} />
        </div>
      )}
    </section>
  );
}

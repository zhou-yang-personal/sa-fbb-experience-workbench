import { useEffect, useMemo, useRef, useState } from 'react';
import * as echarts from 'echarts';
import type { DashboardOverview, MetricCard } from '../../shared/types';
import { AnalyticsEvidenceTable } from './AnalyticsEvidenceTable';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import {
  buildStructuredAppChartRows,
  buildStructuredHourlyChartRows,
  buildStructuredLeadFunnelRows,
  buildStructuredNetworkChartRows,
  buildStructuredUserChartRows,
} from './analyticsStructuredCharts';
import { workbenchApi } from './workbenchApi';
import type { WorkbenchController } from './useWorkbenchController';

type ChartKind = 'bar' | 'donut' | 'radar' | 'line' | 'scatter' | 'heatmap' | 'funnel';
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

type StructuredDataset = {
  kpis: MetricCard[];
  appRank: MetricCard[];
  hourlyTrend: MetricCard[];
  networkHotspots: MetricCard[];
  userProfiles: MetricCard[];
  leadEvidence: MetricCard[];
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

const emptyStructuredDataset: StructuredDataset = {
  kpis: [],
  appRank: [],
  hourlyTrend: [],
  networkHotspots: [],
  userProfiles: [],
  leadEvidence: [],
};

const tabs: Array<{ id: AnalyticsTab; label: string; hint: string }> = [
  { id: 'overview', label: '总览驾驶舱', hint: 'Structured KPI / 流量 / Lead' },
  { id: 'apps', label: '应用体验', hint: 'Structured App Rank / 视频 / 游戏' },
  { id: 'quality', label: '网络质量', hint: 'Hourly Trend / Hotspot / QoE' },
  { id: 'cable', label: 'Cable vs FTTH', hint: '小时趋势 / 接入差异' },
  { id: 'users', label: '用户画像', hint: 'User Profile / 体验证据' },
  { id: 'leads', label: '迁转升套机会', hint: 'Lead Evidence / Funnel' },
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
  return value.length > 30 ? `${value.slice(0, 30)}…` : value;
}

function topRows(metrics: MetricCard[], limit = 12) {
  return [...metrics]
    .map((item) => ({ ...item, numeric: toNumber(item.value) }))
    .sort((a, b) => Math.abs(b.numeric) - Math.abs(a.numeric))
    .slice(0, limit);
}

function metricByName(metrics: MetricCard[], names: string[]) {
  const lower = names.map((name) => name.toLowerCase());
  return metrics.find((metric) => lower.some((name) => `${metric.label} ${metric.hint}`.toLowerCase().includes(name)));
}

function buildKpis(data: MetricDataset) {
  const all = [...data.overview, ...data.appCategory, ...data.experience, ...data.cableFiber, ...data.leads, ...data.users];
  const primary = [
    { label: 'Users', metric: metricByName(all, ['user', 'active_users', 'total_users', 'user_cnt']) },
    { label: 'Traffic', metric: metricByName(all, ['traffic', 'download', 'gb']) },
    { label: 'Experience', metric: metricByName(all, ['vmos', 'mos', 'quality']) },
    { label: 'Cable gap', metric: metricByName([...data.cableFiber, ...data.cableHourly], ['diff', 'gap', 'rtt', 'loss', 'cable']) },
    { label: 'Lead users', metric: metricByName(data.leads, ['lead', 'action', 'a1', 'user_cnt']) },
    { label: 'Top app', metric: data.appCategory[0] },
  ];
  return primary.map((item) => ({ label: item.label, value: item.metric?.value ?? '-', hint: item.metric?.label ?? '等待刷新' }));
}

function buildInsights(data: MetricDataset) {
  const topApp = topRows(data.appCategory, 1)[0];
  const topQuality = topRows([...data.experience, ...data.networkQuality, ...data.cableHourly], 1)[0];
  const topCable = topRows([...data.cableFiber, ...data.cableHourly], 1)[0];
  const topLead = topRows(data.leads, 1)[0];
  return [
    { label: '业务需求入口', value: topApp ? `${topApp.label} · ${topApp.value}` : '暂无应用排名', hint: '优先用结构化 App Rank 解释用户真实应用需求。' },
    { label: '体验风险入口', value: topQuality ? `${topQuality.label} · ${topQuality.value}` : '暂无质量指标', hint: '优先用 Hourly Trend / Hotspot 判断速率、时延、丢包、VMOS/MOS 风险。' },
    { label: '转光证据入口', value: topCable ? `${topCable.label} · ${topCable.value}` : '暂无 Cable/FTTH 差异', hint: '用于判断 Cable 与 FTTH 的可解释体验差异。' },
    { label: '机会名单入口', value: topLead ? `${topLead.label} · ${topLead.value}` : '暂无 Lead 汇总', hint: '体验差不等于升套，需结合需求、迁转动力和问题侧。' },
  ];
}

function optionForChart(kind: ChartKind, title: string, description: string, rows: ReturnType<typeof topRows>) {
  const textStyle = { color: '#e8eefc', fontSize: 15 };
  const subtextStyle = { color: '#94a3b8' };
  if (!rows.length) {
    return {
      title: { text: title, textStyle },
      graphic: { type: 'text', left: 'center', top: 'middle', style: { text: 'No aggregated data. Refresh dashboard after DWS/ADS is ready.', fill: '#94a3b8' } },
    };
  }
  if (kind === 'donut') {
    return {
      title: { text: title, subtext: description, textStyle, subtextStyle },
      tooltip: { trigger: 'item' },
      legend: { bottom: 0, textStyle: { color: '#cbd5e1' } },
      series: [{ type: 'pie', radius: ['46%', '70%'], center: ['50%', '48%'], data: rows.map((row) => ({ name: normalizeLabel(row.label), value: Math.abs(row.numeric) })) }],
    };
  }
  if (kind === 'radar') {
    const maxValue = Math.max(...rows.map((row) => Math.abs(row.numeric)), 1);
    return {
      title: { text: title, subtext: description, textStyle, subtextStyle },
      tooltip: {},
      radar: {
        radius: '62%',
        indicator: rows.slice(0, 8).map((row) => ({ name: normalizeLabel(row.label), max: maxValue })),
        axisName: { color: '#cbd5e1' },
        splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } },
        splitArea: { areaStyle: { color: ['rgba(15,23,42,0.28)', 'rgba(30,41,59,0.18)'] } },
      },
      series: [{ type: 'radar', data: [{ value: rows.slice(0, 8).map((row) => Math.abs(row.numeric)), name: title }] }],
    };
  }
  if (kind === 'line') {
    return {
      title: { text: title, subtext: description, textStyle, subtextStyle },
      tooltip: { trigger: 'axis' },
      grid: { left: 58, right: 24, top: 72, bottom: 58 },
      xAxis: { type: 'category', data: rows.map((row) => normalizeLabel(row.label)), axisLabel: { color: '#cbd5e1' } },
      yAxis: { type: 'value', axisLabel: { color: '#cbd5e1' }, splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } } },
      series: [{ type: 'line', smooth: true, areaStyle: { opacity: 0.16 }, data: rows.map((row) => row.numeric) }],
    };
  }
  if (kind === 'scatter') {
    return {
      title: { text: title, subtext: description, textStyle, subtextStyle },
      tooltip: { formatter: (params: { data: [number, number, number, string] }) => `${params.data[3]}<br/>x=${params.data[0]} y=${params.data[1]}` },
      grid: { left: 58, right: 24, top: 72, bottom: 50 },
      xAxis: { type: 'value', axisLabel: { color: '#cbd5e1' }, splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } } },
      yAxis: { type: 'value', axisLabel: { color: '#cbd5e1' }, splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } } },
      series: [{ type: 'scatter', symbolSize: (value: number[]) => Math.max(10, Math.min(42, Math.sqrt(Math.abs(value[2])) * 4)), data: rows.map((row, index) => [index + 1, row.numeric, Math.abs(row.numeric), row.label]) }],
    };
  }
  if (kind === 'heatmap') {
    const xLabels = rows.slice(0, 8).map((row) => normalizeLabel(row.label));
    const yLabels = ['demand', 'quality', 'access'];
    const values = xLabels.flatMap((_, x) => yLabels.map((__, y) => [x, y, Math.abs(rows[x]?.numeric ?? 0) * (y + 1)]));
    return {
      title: { text: title, subtext: description, textStyle, subtextStyle },
      tooltip: { position: 'top' },
      grid: { left: 80, right: 24, top: 72, bottom: 70 },
      xAxis: { type: 'category', data: xLabels, axisLabel: { color: '#cbd5e1', rotate: 25 } },
      yAxis: { type: 'category', data: yLabels, axisLabel: { color: '#cbd5e1' } },
      visualMap: { min: 0, max: Math.max(...values.map((item) => item[2] as number), 1), calculable: true, orient: 'horizontal', left: 'center', bottom: 0, textStyle: { color: '#cbd5e1' } },
      series: [{ type: 'heatmap', data: values, label: { show: false } }],
    };
  }
  if (kind === 'funnel') {
    return {
      title: { text: title, subtext: description, textStyle, subtextStyle },
      tooltip: { trigger: 'item' },
      series: [{ type: 'funnel', left: '10%', top: 72, bottom: 20, width: '80%', data: rows.map((row) => ({ name: normalizeLabel(row.label), value: Math.abs(row.numeric) })) }],
    };
  }
  return {
    title: { text: title, subtext: description, textStyle, subtextStyle },
    tooltip: { trigger: 'axis' },
    grid: { left: 130, right: 24, top: 72, bottom: 36 },
    xAxis: { type: 'value', axisLabel: { color: '#cbd5e1' }, splitLine: { lineStyle: { color: 'rgba(148,163,184,0.18)' } } },
    yAxis: { type: 'category', data: rows.map((row) => normalizeLabel(row.label)).reverse(), axisLabel: { color: '#cbd5e1' } },
    series: [{ type: 'bar', data: rows.map((row) => row.numeric).reverse(), barMaxWidth: 22 }],
  };
}

function AnalyticsChart({ title, description, kind, metrics, height = 380 }: { title: string; description: string; kind: ChartKind; metrics: MetricCard[]; height?: number }) {
  const ref = useRef<HTMLDivElement | null>(null);
  const rows = useMemo(() => topRows(metrics, kind === 'line' ? 24 : 14), [metrics, kind]);

  useEffect(() => {
    if (!ref.current) return;
    const chart = echarts.init(ref.current);
    chart.setOption(optionForChart(kind, title, description, rows));
    const resize = () => chart.resize();
    window.addEventListener('resize', resize);
    return () => { window.removeEventListener('resize', resize); chart.dispose(); };
  }, [rows, title, description, kind]);

  return (
    <article className="analytics-card analytics-chart-card">
      <div className="analytics-chart" style={{ height }} ref={ref} />
      <div className="analytics-card-footnote">
        <span>Source: Structured DWS/ADS first · legacy DWS/ADS fallback</span>
        <span>Rows: {metrics.length}</span>
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

function InsightStrip({ items }: { items: Array<{ label: string; value: string; hint: string }> }) {
  return (
    <section className="analytics-insight-strip">
      {items.map((item) => <article key={item.label}><span>{item.label}</span><strong>{item.value}</strong><small>{item.hint}</small></article>)}
    </section>
  );
}

function preferredRows(primary: MetricCard[], fallback: MetricCard[]) {
  return primary.length ? primary : fallback;
}

function structuredCount(data: StructuredDataset) {
  return data.kpis.length + data.appRank.length + data.hourlyTrend.length + data.networkHotspots.length + data.userProfiles.length + data.leadEvidence.length;
}

export function AnalyticsDashboard({ c }: { c: WorkbenchController }) {
  const [activeTab, setActiveTab] = useState<AnalyticsTab>('overview');
  const [data, setData] = useState<MetricDataset>(emptyDataset);
  const [structuredData, setStructuredData] = useState<StructuredDataset>(emptyStructuredDataset);
  const [message, setMessage] = useState('点击“刷新分析驾驶舱”加载当前批次聚合结果。');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function refreshAll() {
    await c.runAction('analytics_dashboard_refresh_all', async () => {
      const settings = c.effectiveSettings;
      const importBatchId = c.importBatchId;
      const analysisRunId = c.analysisRunId;
      const [overviewResult, appCategory, experience, networkQuality, video, game, cableFiber, cableHourly, users, leads, structuredKpis, structuredAppRank, structuredHourlyTrend, structuredNetworkHotspots, structuredUserProfiles, structuredLeadEvidence] = await Promise.all([
        workbenchApi.overview(settings, importBatchId, analysisRunId),
        workbenchApi.appCategory(settings, importBatchId, analysisRunId),
        workbenchApi.experience(settings, importBatchId, analysisRunId),
        workbenchApi.networkQuality(settings, importBatchId, analysisRunId),
        workbenchApi.videoDetail(settings, importBatchId, analysisRunId).catch(() => []),
        workbenchApi.gameExperience(settings, importBatchId, analysisRunId).catch(() => []),
        workbenchApi.cableFiber(settings, importBatchId, analysisRunId),
        workbenchApi.cableFiberHourly(settings, importBatchId, analysisRunId).catch(() => []),
        workbenchApi.userProfile(settings, importBatchId, analysisRunId).catch(() => []),
        workbenchApi.leadSummary(settings, importBatchId, analysisRunId).catch(() => []),
        analyticsStructuredApi.kpis(settings, importBatchId, analysisRunId).catch(() => []),
        analyticsStructuredApi.appRank(settings, importBatchId, analysisRunId).catch(() => []),
        analyticsStructuredApi.hourlyTrend(settings, importBatchId, analysisRunId).catch(() => []),
        analyticsStructuredApi.networkHotspots(settings, importBatchId, analysisRunId).catch(() => []),
        analyticsStructuredApi.userProfiles(settings, importBatchId, analysisRunId).catch(() => []),
        analyticsStructuredApi.leadEvidence(settings, importBatchId, analysisRunId).catch(() => []),
      ]);
      const overview = (overviewResult as DashboardOverview).metrics ?? [];
      const structured: StructuredDataset = {
        kpis: structuredKpis,
        appRank: structuredAppRank,
        hourlyTrend: structuredHourlyTrend,
        networkHotspots: structuredNetworkHotspots,
        userProfiles: structuredUserProfiles,
        leadEvidence: structuredLeadEvidence,
      };
      const structuredAppRows = buildStructuredAppChartRows(structuredAppRank);
      const structuredHourlyRows = buildStructuredHourlyChartRows(structuredHourlyTrend);
      const structuredNetworkRows = buildStructuredNetworkChartRows(structuredNetworkHotspots);
      const structuredUserRows = buildStructuredUserChartRows(structuredUserProfiles);
      const structuredLeadRows = buildStructuredLeadFunnelRows(structuredLeadEvidence);
      const preferredData: MetricDataset = {
        overview: preferredRows(structuredKpis, overview),
        appCategory: preferredRows(structuredAppRows, appCategory),
        experience,
        networkQuality: preferredRows(structuredNetworkRows, networkQuality),
        video: preferredRows(structuredAppRows, video),
        game,
        cableFiber,
        cableHourly: preferredRows(structuredHourlyRows, cableHourly),
        users: preferredRows(structuredUserRows, users),
        leads: preferredRows(structuredLeadRows, leads),
      };
      c.setOverview({
        ...(overviewResult as DashboardOverview),
        metrics: preferredData.overview,
      });
      c.setDashboardCharts([
        { title: 'Structured App Rank', kind: 'bar', metrics: preferredData.appCategory },
        { title: 'Structured Quality / Hotspot', kind: 'radar', metrics: [...preferredData.experience, ...preferredData.networkQuality, ...preferredData.cableHourly] },
        { title: 'Cable vs FTTH', kind: 'bar', metrics: preferredData.cableFiber },
        { title: 'Structured Lead Evidence', kind: 'bar', metrics: preferredData.leads },
      ]);
      setStructuredData(structured);
      setData(preferredData);
      setMessage(`已刷新：structured=${structuredCount(structured)}, overview=${preferredData.overview.length}, apps=${preferredData.appCategory.length}, quality=${preferredData.experience.length + preferredData.networkQuality.length}, cableHourly=${preferredData.cableHourly.length}, users=${preferredData.users.length}, leads=${preferredData.leads.length}`);
      return { preferredData, structured };
    });
  }

  useEffect(() => {
    if (!disabled) void refreshAll();
  }, [c.importBatchId, c.analysisRunId]);

  const kpis = useMemo(() => buildKpis(data), [data]);
  const insights = useMemo(() => buildInsights(data), [data]);
  const structuredAppChartRows = useMemo(() => buildStructuredAppChartRows(structuredData.appRank), [structuredData.appRank]);
  const structuredHourlyChartRows = useMemo(() => buildStructuredHourlyChartRows(structuredData.hourlyTrend), [structuredData.hourlyTrend]);
  const structuredNetworkChartRows = useMemo(() => buildStructuredNetworkChartRows(structuredData.networkHotspots), [structuredData.networkHotspots]);
  const structuredUserChartRows = useMemo(() => buildStructuredUserChartRows(structuredData.userProfiles), [structuredData.userProfiles]);
  const structuredLeadChartRows = useMemo(() => buildStructuredLeadFunnelRows(structuredData.leadEvidence), [structuredData.leadEvidence]);
  const videoOrApps = data.video.length ? data.video : data.appCategory;
  const gameOrApps = data.game.length ? data.game : data.appCategory;
  const qualityRows = preferredRows([...structuredHourlyChartRows, ...structuredNetworkChartRows], [...data.experience, ...data.networkQuality]);
  const appRows = preferredRows(structuredAppChartRows, [...data.appCategory, ...data.video, ...data.game]);
  const hourlyOrCableRows = preferredRows(structuredHourlyChartRows, data.cableHourly.length ? data.cableHourly : data.cableFiber);
  const networkEvidenceRows = preferredRows(structuredData.networkHotspots, data.networkQuality);
  const userEvidenceRows = preferredRows(structuredData.userProfiles, data.users);
  const leadEvidenceRows = preferredRows(structuredData.leadEvidence, data.leads);

  return (
    <section className="analytics-dashboard">
      <header className="analytics-hero">
        <div>
          <p className="eyebrow">Analytics cockpit · Structured DWS / ADS first</p>
          <h2>数据分析驾驶舱</h2>
          <p>围绕当前批次构建大图、大表和业务诊断路径。当前驾驶舱优先消费结构化 Analytics API，旧 dashboard commands 作为兼容 fallback。</p>
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
        <span>Structured rows: {structuredCount(structuredData)}</span>
        <span>{message}</span>
      </div>

      <KpiStrip items={kpis} />
      <InsightStrip items={insights} />

      <nav className="analytics-tabs">
        {tabs.map((tab) => <button key={tab.id} type="button" className={activeTab === tab.id ? 'is-active' : ''} onClick={() => setActiveTab(tab.id)}><strong>{tab.label}</strong><small>{tab.hint}</small></button>)}
      </nav>

      {activeTab === 'overview' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Structured app demand rank" description="ADS App Rank preferred; legacy app category command as fallback." kind="bar" metrics={data.appCategory} height={440} />
          <AnalyticsChart title="Structured lead / action mix" description="ADS Lead Evidence preferred; legacy Lead Summary as fallback." kind="donut" metrics={data.leads} height={440} />
          <AnalyticsChart title="Structured quality risk summary" description="Hourly Trend and Network Hotspot preferred for QoE risk." kind="radar" metrics={qualityRows} height={420} />
          <AnalyticsChart title="Demand vs quality scatter" description="Structured app demand and QoE pressure proxy." kind="scatter" metrics={[...data.appCategory, ...qualityRows]} height={420} />
          <AnalyticsEvidenceTable title="Structured KPI Evidence" rows={preferredRows(structuredData.kpis, data.overview)} />
        </div>
      )}

      {activeTab === 'apps' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Top apps / categories by demand" description="Structured App Rank transformed from ADS hint fields." kind="bar" metrics={data.appCategory} height={440} />
          <AnalyticsChart title="App demand heatmap" description="Demand / quality / access proxy matrix for top app rows." kind="heatmap" metrics={appRows} height={420} />
          <AnalyticsChart title="Video experience rank" description="VMOS, effective rate, connection delay and video QoE proxy metrics." kind="bar" metrics={videoOrApps} height={420} />
          <AnalyticsChart title="Game experience rank" description="MOS, latency, jitter, loss and game-hour proxy metrics." kind="scatter" metrics={gameOrApps} height={420} />
          <AnalyticsEvidenceTable title="Structured App Rank Evidence" rows={preferredRows(structuredData.appRank, appRows)} limit={160} />
        </div>
      )}

      {activeTab === 'quality' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Structured network quality radar" description="Hourly Trend + Network Hotspot dimensions." kind="radar" metrics={qualityRows} height={440} />
          <AnalyticsChart title="Network hotspot ranking" description="Structured network hotspot ADS table preferred." kind="bar" metrics={preferredRows(structuredNetworkChartRows, data.networkQuality)} height={420} />
          <AnalyticsChart title="Quality issue scatter" description="Proxy distribution of high-value quality dimensions." kind="scatter" metrics={qualityRows} height={420} />
          <AnalyticsEvidenceTable title="Structured Hourly Trend Evidence" rows={preferredRows(structuredData.hourlyTrend, hourlyOrCableRows)} limit={160} />
          <AnalyticsEvidenceTable title="Structured Network Hotspot Evidence" rows={networkEvidenceRows} limit={160} />
        </div>
      )}

      {activeTab === 'cable' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Cable vs FTTH metric gap" description="Aggregated metric differences by access type." kind="bar" metrics={data.cableFiber} height={460} />
          <AnalyticsChart title="Structured hourly Cable / FTTH trend" description="ADS Hourly Trend preferred; legacy hourly compare as fallback." kind="line" metrics={hourlyOrCableRows} height={420} />
          <AnalyticsChart title="Access gap heatmap" description="Hour / metric proxy matrix for Cable vs FTTH gap." kind="heatmap" metrics={[...data.cableFiber, ...hourlyOrCableRows]} height={420} />
          <AnalyticsEvidenceTable title="Structured Hourly Trend Evidence" rows={preferredRows(structuredData.hourlyTrend, [...data.cableFiber, ...data.cableHourly])} limit={180} />
        </div>
      )}

      {activeTab === 'users' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Structured user profile demand ranking" description="ADS User Profile transformed into chart rows." kind="bar" metrics={preferredRows(structuredUserChartRows, data.users)} height={460} />
          <AnalyticsChart title="User demand vs QoE proxy" description="User-level proxy scatter for demand and experience risk." kind="scatter" metrics={preferredRows(structuredUserChartRows, data.users)} height={420} />
          <AnalyticsEvidenceTable title="Structured User Profile Evidence" rows={userEvidenceRows} limit={220} />
        </div>
      )}

      {activeTab === 'leads' && (
        <div className="analytics-layout">
          <AnalyticsChart title="Structured lead funnel" description="Lead type or final action distribution from ADS Lead Evidence." kind="funnel" metrics={preferredRows(structuredLeadChartRows, data.leads)} height={460} />
          <AnalyticsChart title="Lead mix donut" description="Part-to-whole view of current opportunity buckets." kind="donut" metrics={preferredRows(structuredLeadChartRows, data.leads)} height={420} />
          <AnalyticsChart title="Lead evidence scatter" description={`Top lead bucket value scale: ${compact(topRows(preferredRows(structuredLeadChartRows, data.leads), 1)[0]?.numeric ?? 0)}`} kind="scatter" metrics={preferredRows(structuredLeadChartRows, data.leads)} height={420} />
          <AnalyticsEvidenceTable title="Structured Lead Evidence" rows={leadEvidenceRows} limit={200} />
        </div>
      )}
    </section>
  );
}

import { useEffect, useMemo, useRef, useState } from 'react';
import * as echarts from 'echarts';
import type { MetricCard } from '../../shared/types';
import { AnalyticsEvidenceTable } from './AnalyticsEvidenceTable';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import { parseMetricHint } from './analyticsStructuredCharts';
import type { WorkbenchController } from './useWorkbenchController';

type ChartKind = 'bar' | 'line' | 'donut';
export type AnalyticsTab = 'overview' | 'apps' | 'quality' | 'cable' | 'users' | 'leads';

type StructuredDataset = {
  kpis: MetricCard[];
  appRank: MetricCard[];
  hourlyTrend: MetricCard[];
  networkHotspots: MetricCard[];
  userProfiles: MetricCard[];
  leadEvidence: MetricCard[];
};

type ChartPoint = {
  label: string;
  value: number;
  series?: string;
  source?: MetricCard;
};

const emptyDataset: StructuredDataset = {
  kpis: [],
  appRank: [],
  hourlyTrend: [],
  networkHotspots: [],
  userProfiles: [],
  leadEvidence: [],
};

const pageCopy: Record<AnalyticsTab, { eyebrow: string; title: string; description: string }> = {
  overview: { eyebrow: 'Decision cockpit', title: '经营与体验总览', description: '先确认数据可信度，再查看问题影响、网络行动和合格机会。' },
  apps: { eyebrow: 'Application experience', title: '应用体验', description: '按真实 App、接入类型和问题侧识别受影响用户与业务需求。' },
  quality: { eyebrow: 'Network action', title: '网络问题定位', description: '沿 BRAS / OLT / PON 定位热点，并区分网络侧与家庭侧问题。' },
  cable: { eyebrow: 'Access benchmark', title: 'Cable vs FTTH', description: '在相同时间口径下比较速率、时延、丢包和体验结果。' },
  users: { eyebrow: 'User evidence', title: '用户洞察', description: '查看用户需求、体验、接入识别和机会证据，而不是只看一个评分。' },
  leads: { eyebrow: 'Qualified opportunity', title: '迁转升套机会', description: '先排除身份不足与网络严重异常，再查看候选和培育用户。' },
};

function numberValue(value: unknown) {
  const parsed = Number.parseFloat(String(value ?? '').replace(/,/g, '').replace(/%/g, '').trim());
  return Number.isFinite(parsed) ? parsed : 0;
}

function fromHint(row: MetricCard, key: string) {
  return numberValue(parseMetricHint(row.hint)[key]);
}

function textFromHint(row: MetricCard, key: string, fallback = '') {
  return parseMetricHint(row.hint)[key] || fallback;
}

function compact(value: number) {
  if (!Number.isFinite(value)) return '-';
  if (Math.abs(value) >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (Math.abs(value) >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  if (Math.abs(value) >= 100) return value.toFixed(0);
  if (Math.abs(value) >= 10) return value.toFixed(1);
  return value.toFixed(2).replace(/\.00$/, '');
}

function appPoints(rows: MetricCard[], key: string): ChartPoint[] {
  return rows.map((row) => {
    const access = textFromHint(row, 'user_type', 'UNKNOWN');
    const app = textFromHint(row, 'app_name', row.label);
    return { label: `${app} · ${access}`, value: fromHint(row, key), series: access, source: row };
  }).filter((row) => row.value > 0);
}

function hotspotPoints(rows: MetricCard[], key: string): ChartPoint[] {
  return rows.map((row) => {
    const detail = parseMetricHint(row.hint);
    const topology = [detail.bras, detail.olt, detail.pon].filter((item) => item && item !== 'UNKNOWN').join(' / ');
    const access = detail.user_type || 'UNKNOWN';
    return { label: `${topology || row.label} · ${access}`, value: fromHint(row, key), series: access, source: row };
  }).filter((row) => row.value > 0);
}

function hourlyPoints(rows: MetricCard[], key: string): ChartPoint[] {
  return rows.map((row) => {
    const detail = parseMetricHint(row.hint);
    const date = detail.stat_date ? detail.stat_date.slice(5) : '';
    const hour = String(detail.hour ?? '').padStart(2, '0');
    return {
      label: `${date} ${hour}:00`.trim(),
      value: fromHint(row, key),
      series: detail.user_type || 'UNKNOWN',
      source: row,
    };
  }).filter((row) => row.label && Number.isFinite(row.value));
}

function userPoints(rows: MetricCard[], key: string): ChartPoint[] {
  return rows.map((row) => ({
    label: textFromHint(row, 'user_key', row.label),
    value: fromHint(row, key),
    series: textFromHint(row, 'user_type', 'UNKNOWN'),
    source: row,
  })).filter((row) => row.value > 0);
}

function leadStagePoints(rows: MetricCard[]): ChartPoint[] {
  const grouped = new Map<string, { count: number; source: MetricCard }>();
  rows.forEach((row) => {
    const stage = textFromHint(row, 'lead_type', 'UNKNOWN');
    const current = grouped.get(stage);
    grouped.set(stage, { count: (current?.count ?? 0) + 1, source: current?.source ?? row });
  });
  return [...grouped.entries()].map(([label, item]) => ({ label, value: item.count, source: item.source }));
}

function filterRows(rows: MetricCard[], access: string, keyword: string, minUsers: number) {
  const query = keyword.trim().toLowerCase();
  return rows.filter((row) => {
    const detail = parseMetricHint(row.hint);
    const rowAccess = detail.user_type || detail.access_type || 'UNKNOWN';
    const population = detail.users ?? detail.active_users ?? detail.user_cnt;
    const users = numberValue(population);
    const accessMatches = access === 'ALL' || rowAccess === access;
    const keywordMatches = !query || `${row.label} ${row.hint}`.toLowerCase().includes(query);
    const populationMatches = minUsers <= 0 || population === undefined || users >= minUsers;
    return accessMatches && keywordMatches && populationMatches;
  });
}

function chartOption(kind: ChartKind, title: string, subtitle: string, incoming: ChartPoint[]) {
  const ink = '#172033';
  const muted = '#667085';
  const grid = '#e7ecf3';
  const palette = ['#2563eb', '#d97706', '#64748b', '#7c3aed', '#0f766e'];
  const points = kind === 'line' ? incoming : [...incoming].sort((a, b) => b.value - a.value).slice(0, 12);
  const base = {
    color: palette,
    title: { text: title, subtext: subtitle, textStyle: { color: ink, fontSize: 16, fontWeight: 650 }, subtextStyle: { color: muted, fontSize: 11, lineHeight: 16 } },
    tooltip: { trigger: kind === 'donut' ? 'item' : 'axis', confine: true },
  };
  if (!points.length) {
    return { ...base, graphic: { type: 'text', left: 'center', top: 'middle', style: { text: '当前筛选下没有可用聚合数据', fill: muted } } };
  }
  if (kind === 'donut') {
    return {
      ...base,
      legend: { bottom: 0, type: 'scroll', textStyle: { color: muted } },
      series: [{ type: 'pie', radius: ['48%', '70%'], center: ['50%', '48%'], data: points.map((point) => ({ name: point.label, value: point.value })), label: { color: ink, formatter: '{b}\n{c}' } }],
    };
  }
  if (kind === 'line') {
    const labels = [...new Set(points.map((point) => point.label))];
    const seriesNames = [...new Set(points.map((point) => point.series || 'All'))];
    return {
      ...base,
      legend: { top: 48, textStyle: { color: muted } },
      grid: { left: 58, right: 24, top: 88, bottom: 54 },
      xAxis: { type: 'category', data: labels, axisLabel: { color: muted, rotate: labels.length > 12 ? 35 : 0 }, axisLine: { lineStyle: { color: grid } } },
      yAxis: { type: 'value', scale: true, axisLabel: { color: muted }, splitLine: { lineStyle: { color: grid } } },
      series: seriesNames.map((seriesName) => ({
        name: seriesName,
        type: 'line',
        showSymbol: labels.length <= 24,
        connectNulls: false,
        data: labels.map((label) => points.find((point) => point.label === label && (point.series || 'All') === seriesName)?.value ?? null),
      })),
    };
  }
  return {
    ...base,
    grid: { left: 150, right: 28, top: 76, bottom: 34 },
    xAxis: { type: 'value', min: 0, axisLabel: { color: muted }, splitLine: { lineStyle: { color: grid } } },
    yAxis: { type: 'category', data: points.map((point) => point.label).reverse(), axisLabel: { color: ink, width: 130, overflow: 'truncate' }, axisLine: { show: false } },
    series: [{ type: 'bar', data: points.map((point) => point.value).reverse(), barMaxWidth: 22, itemStyle: { color: palette[0], borderRadius: [0, 4, 4, 0] }, label: { show: true, position: 'right', color: muted, formatter: ({ value }: { value: number }) => compact(value) } }],
  };
}

function AnalyticsChart({ title, subtitle, kind, points, onSelect, height = 380 }: { title: string; subtitle: string; kind: ChartKind; points: ChartPoint[]; onSelect?: (row: MetricCard) => void; height?: number }) {
  const ref = useRef<HTMLDivElement | null>(null);
  const effectiveKind: ChartKind = kind === 'line' && new Set(points.map((point) => point.label)).size < 8 ? 'bar' : kind;
  const renderedPoints = useMemo(() => kind === 'line' && effectiveKind === 'bar'
    ? points.map((point) => ({ ...point, label: `${point.label} · ${point.series || 'ALL'}` }))
    : points, [effectiveKind, kind, points]);
  useEffect(() => {
    if (!ref.current) return;
    const chart = echarts.init(ref.current);
    chart.setOption(chartOption(effectiveKind, title, subtitle, renderedPoints), true);
    chart.on('click', (params) => {
      const label = String(params.name ?? '');
      const source = renderedPoints.find((point) => point.label === label)?.source;
      if (source && onSelect) onSelect(source);
    });
    const resize = () => chart.resize();
    window.addEventListener('resize', resize);
    return () => { window.removeEventListener('resize', resize); chart.dispose(); };
  }, [effectiveKind, height, onSelect, renderedPoints, subtitle, title]);
  return <article className="analytics-card analytics-chart-card"><div className="analytics-chart" style={{ height }} ref={ref} /><footer><span>来源：DWS / ADS</span><span>{kind !== effectiveKind ? '时点不足 8 个，已降级为柱图' : `${points.length} data points`}</span></footer></article>;
}

function KpiStrip({ items }: { items: Array<{ label: string; value: string; hint: string; tone?: string }> }) {
  return <section className="analytics-kpi-strip">{items.map((item) => <article key={item.label} className={`analytics-kpi-card ${item.tone ? `tone-${item.tone}` : ''}`}><span>{item.label}</span><strong>{item.value}</strong><small>{item.hint}</small></article>)}</section>;
}

function EvidenceDrawer({ row, onClose }: { row: MetricCard; onClose: () => void }) {
  const detail = parseMetricHint(row.hint);
  return <div className="analytics-evidence-drawer-backdrop" role="presentation" onClick={onClose}><aside className="analytics-evidence-drawer" role="dialog" aria-modal="true" onClick={(event) => event.stopPropagation()}><header><div><p className="eyebrow">Evidence</p><h3>{row.label}</h3></div><button type="button" onClick={onClose}>关闭</button></header><div className="analytics-evidence-kv">{Object.entries(detail).map(([key, value]) => <div key={key}><span>{key}</span><strong>{value}</strong></div>)}</div></aside></div>;
}

export function AnalyticsDashboard({ c, activeView }: { c: WorkbenchController; activeView: AnalyticsTab }) {
  const [data, setData] = useState<StructuredDataset>(emptyDataset);
  const [message, setMessage] = useState('选择批次和分析运行后加载聚合结果。');
  const [failures, setFailures] = useState<string[]>([]);
  const [access, setAccess] = useState('ALL');
  const [keyword, setKeyword] = useState('');
  const [minUsers, setMinUsers] = useState(0);
  const [selectedEvidence, setSelectedEvidence] = useState<MetricCard | null>(null);
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function refreshAll() {
    if (disabled) return;
    await c.runAction('analytics_dashboard_refresh', async () => {
      const requests = [
        ['kpis', analyticsStructuredApi.kpis(c.effectiveSettings, c.importBatchId, c.analysisRunId)],
        ['appRank', analyticsStructuredApi.appRank(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 200 })],
        ['hourlyTrend', analyticsStructuredApi.hourlyTrend(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 500, sortBy: 'hour' })],
        ['networkHotspots', analyticsStructuredApi.networkHotspots(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 200, sortBy: 'users' })],
        ['userProfiles', analyticsStructuredApi.userProfiles(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 300, sortBy: 'demand' })],
        ['leadEvidence', analyticsStructuredApi.leadEvidence(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 500, sortBy: 'demand' })],
      ] as const;
      const results = await Promise.allSettled(requests.map(([, request]) => request));
      const next: StructuredDataset = { kpis: [], appRank: [], hourlyTrend: [], networkHotspots: [], userProfiles: [], leadEvidence: [] };
      const nextFailures: string[] = [];
      results.forEach((result, index) => {
        const key = requests[index][0];
        if (result.status === 'fulfilled') next[key] = result.value as MetricCard[];
        else nextFailures.push(`${key}: ${result.reason instanceof Error ? result.reason.message : String(result.reason)}`);
      });
      setData(next);
      setFailures(nextFailures);
      const rowCount = Object.values(next).reduce((sum, rows) => sum + rows.length, 0);
      setMessage(`已加载 ${rowCount} 条聚合证据；${nextFailures.length ? `${nextFailures.length} 个数据集失败` : '所有数据集可读取'}。`);
      c.setOverview({ metrics: next.kpis });
      return { rowCount, failures: nextFailures };
    });
  }

  useEffect(() => {
    if (!disabled) void refreshAll();
  }, [c.importBatchId, c.analysisRunId]);

  const filtered = useMemo(() => ({
    appRank: filterRows(data.appRank, access, keyword, minUsers),
    hourlyTrend: filterRows(data.hourlyTrend, access, keyword, minUsers),
    networkHotspots: filterRows(data.networkHotspots, access, keyword, minUsers),
    userProfiles: filterRows(data.userProfiles, access, keyword, minUsers),
    leadEvidence: filterRows(data.leadEvidence, access, keyword, minUsers),
  }), [access, data, keyword, minUsers]);

  const leadStages = useMemo(() => leadStagePoints(filtered.leadEvidence), [filtered.leadEvidence]);
  const knownHourlyUsers = filtered.hourlyTrend.reduce((sum, row) => sum + (textFromHint(row, 'user_type') === 'UNKNOWN' ? 0 : fromHint(row, 'users')), 0);
  const allHourlyUsers = filtered.hourlyTrend.reduce((sum, row) => sum + fromHint(row, 'users'), 0);
  const coverage = allHourlyUsers > 0 ? knownHourlyUsers / allHourlyUsers * 100 : 0;
  const a1 = leadStages.find((stage) => stage.label.startsWith('A1_'))?.value ?? 0;
  const a2 = leadStages.find((stage) => stage.label.startsWith('A2_'))?.value ?? 0;
  const issueApps = new Set(filtered.appRank.filter((row) => fromHint(row, 'poor_experience_user_pct') > 0).map((row) => textFromHint(row, 'app_name', row.label))).size;
  const severeHotspots = new Set(filtered.networkHotspots
    .filter((row) => textFromHint(row, 'bottleneck') === 'NETWORK_SIDE_SEVERE')
    .map((row) => {
      const detail = parseMetricHint(row.hint);
      return [detail.bras, detail.olt, detail.pon].join('|');
    })).size;
  const kpis = [
    { label: '接入分类观测覆盖率', value: `${coverage.toFixed(1)}%`, hint: '已识别 Cable/FTTH 的用户小时观测 / 全部用户小时观测', tone: coverage < 90 ? 'warning' : 'normal' },
    { label: '问题 App', value: String(issueApps), hint: '当前筛选下差体验用户占比大于 0 的真实 App' },
    { label: '严重网络热点', value: String(severeHotspots), hint: '主问题侧为 NETWORK_SIDE_SEVERE 的拓扑节点', tone: severeHotspots ? 'danger' : 'normal' },
    { label: 'A1 候选', value: String(a1), hint: '仍需 CRM、覆盖和可触达资格校验' },
    { label: 'A2 先修障', value: String(a2), hint: '网络严重异常，禁止直接营销', tone: a2 ? 'warning' : 'normal' },
  ];
  const copy = pageCopy[activeView];
  const selectEvidence = (row: MetricCard) => setSelectedEvidence(row);

  return <section className="analytics-dashboard analytics-dashboard-v3">
    <header className="workspace-page-header analytics-page-header"><div><p className="eyebrow">{copy.eyebrow}</p><h2>{copy.title}</h2><p>{copy.description}</p></div><button type="button" className="primary-button" disabled={disabled} onClick={refreshAll}>刷新当前分析</button></header>
    <section className="analytics-filter-bar" aria-label="分析筛选">
      <label>接入类型<select value={access} onChange={(event) => setAccess(event.target.value)}><option value="ALL">全部</option><option value="CABLE">Cable</option><option value="FTTH">FTTH</option><option value="UNKNOWN">Unknown</option></select></label>
      <label>搜索<input value={keyword} onChange={(event) => setKeyword(event.target.value)} placeholder="App、用户、BRAS、OLT、PON" /></label>
      <label>最小用户数<input type="number" min={0} value={minUsers} onChange={(event) => setMinUsers(Math.max(0, Number(event.target.value)))} /></label>
      <div className="filter-context"><span>Batch</span><strong>{c.batchDisplayName || c.importBatchId || '-'}</strong><small>{message}</small></div>
    </section>
    {failures.length > 0 && <section className="analytics-error-banner"><strong>部分数据集加载失败</strong>{failures.map((failure) => <span key={failure}>{failure}</span>)}</section>}
    <KpiStrip items={kpis} />

    {activeView === 'overview' && <div className="analytics-layout">
      <AnalyticsChart title="问题 App 影响用户" subtitle="唯一受影响用户；按真实 App 排序，点击查看证据" kind="bar" points={appPoints(filtered.appRank, 'poor_experience_users')} onSelect={selectEvidence} />
      <AnalyticsChart title="网络热点差体验率" subtitle="差体验唯一用户 / 节点观测唯一用户；按拓扑节点排序" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'severity')} onSelect={selectEvidence} />
      <AnalyticsChart title="机会与排除分层" subtitle="按用户计数；A0/A2 不得进入直接营销" kind="bar" points={leadStages} onSelect={selectEvidence} />
      <AnalyticsChart title="接入类型小时速率" subtitle="平均有效下载速率 Mbps；至少 8 个时点时用于趋势判断" kind="line" points={hourlyPoints(filtered.hourlyTrend, 'effective_mbps')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="总览指标与来源" rows={data.kpis} />
    </div>}

    {activeView === 'apps' && <div className="analytics-layout">
      <AnalyticsChart title="App 受影响用户" subtitle="唯一差体验用户数；不同接入类型分别保留在证据中" kind="bar" points={appPoints(filtered.appRank, 'poor_experience_users')} onSelect={selectEvidence} />
      <AnalyticsChart title="App 差体验用户占比" subtitle="差体验唯一用户 / App 观测唯一用户，单位 %" kind="bar" points={appPoints(filtered.appRank, 'poor_experience_user_pct')} onSelect={selectEvidence} />
      <AnalyticsChart title="App 业务流量" subtitle="视频下载 GB；游戏类同时查看证据中的 duration_hours" kind="bar" points={appPoints(filtered.appRank, 'traffic_gb')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="应用体验证据" rows={filtered.appRank} limit={220} />
    </div>}

    {activeView === 'quality' && <div className="analytics-layout">
      <AnalyticsChart title="拓扑节点受影响用户" subtitle="BRAS / OLT / PON 粒度的差体验唯一用户" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'users')} onSelect={selectEvidence} />
      <AnalyticsChart title="拓扑节点差体验率" subtitle="差体验唯一用户 / 节点观测唯一用户，单位 %" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'severity')} onSelect={selectEvidence} />
      <AnalyticsChart title="网络侧 RTT" subtitle="节点平均 network-side RTT，单位 ms" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'network_rtt_ms')} onSelect={selectEvidence} />
      <AnalyticsChart title="家庭侧 / Wi-Fi RTT" subtitle="节点平均 subscriber-side RTT，单位 ms" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'subscriber_rtt_ms')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="网络热点行动证据" rows={filtered.networkHotspots} limit={240} />
    </div>}

    {activeView === 'cable' && <div className="analytics-layout">
      <AnalyticsChart title="Cable / FTTH 小时有效速率" subtitle="平均有效下载速率 Mbps；按日期小时与接入类型对比" kind="line" points={hourlyPoints(filtered.hourlyTrend, 'effective_mbps')} onSelect={selectEvidence} />
      <AnalyticsChart title="Cable / FTTH 小时 RTT" subtitle="平均 subscriber-side RTT ms；同一时间口径比较" kind="line" points={hourlyPoints(filtered.hourlyTrend, 'subscriber_rtt_ms')} onSelect={selectEvidence} />
      <AnalyticsChart title="Cable / FTTH 小时用户侧丢包" subtitle="平均 user-side downstream loss，单位 %" kind="line" points={hourlyPoints(filtered.hourlyTrend, 'user_loss_pct')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="接入对比小时证据" rows={filtered.hourlyTrend} limit={300} />
    </div>}

    {activeView === 'users' && <div className="analytics-layout">
      <AnalyticsChart title="用户需求评分" subtitle="用于发现高需求，不等同于可营销资格" kind="bar" points={userPoints(filtered.userProfiles, 'demand_score')} onSelect={selectEvidence} />
      <AnalyticsChart title="用户流量" subtitle="用户分析周期总流量 GB；点击查看体验和问题侧" kind="bar" points={userPoints(filtered.userProfiles, 'traffic_gb')} onSelect={selectEvidence} />
      <AnalyticsChart title="用户游戏时长" subtitle="用户分析周期游戏时长，单位 hours" kind="bar" points={userPoints(filtered.userProfiles, 'game_hours')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="用户画像证据" rows={filtered.userProfiles} limit={300} />
    </div>}

    {activeView === 'leads' && <div className="analytics-layout">
      <AnalyticsChart title="机会与排除分层" subtitle="按唯一用户计数；A0 身份不足、A2 先修障、A1 待资格校验" kind="bar" points={leadStages} onSelect={selectEvidence} />
      <AnalyticsChart title="机会分层构成" subtitle="展示当前分析运行中的用户分层占比" kind="donut" points={leadStages} onSelect={selectEvidence} />
      <AnalyticsChart title="候选用户需求评分" subtitle="评分用于排序；最终行动仍由问题侧与资格字段决定" kind="bar" points={userPoints(filtered.leadEvidence, 'demand_score')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="迁转机会证据" rows={filtered.leadEvidence} limit={400} />
    </div>}
    {selectedEvidence && <EvidenceDrawer row={selectedEvidence} onClose={() => setSelectedEvidence(null)} />}
  </section>;
}

import { useEffect, useMemo, useRef, useState } from 'react';
import type { ECharts } from 'echarts';
import type { BatchListItem, MetricCard, MySqlSettings } from '../../shared/types';
import { AnalyticsEvidenceTable } from './AnalyticsEvidenceTable';
import { ChartExplanation } from './ChartExplanation';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import { parseMetricHint } from './analyticsStructuredCharts';
import { chartExplanationCatalog, type ChartExplanationId } from './chartExplanationCatalog';
import type { WorkbenchController } from './useWorkbenchController';

type ChartKind = 'bar' | 'line' | 'donut';
type InsightStatus = 'healthy' | 'watch' | 'problem' | 'insufficient' | 'unclassified';
export type AnalyticsTab = 'overview' | 'apps' | 'quality' | 'cable' | 'users' | 'leads';

type StructuredDataset = {
  coverage: MetricCard[];
  kpis: MetricCard[];
  appRank: MetricCard[];
  hourlyTrend: MetricCard[];
  networkHotspots: MetricCard[];
  userProfiles: MetricCard[];
  userSummary: MetricCard[];
  leadEvidence: MetricCard[];
  leadSummary: MetricCard[];
};

type DatasetKey = keyof StructuredDataset;
type TaskStatus = 'idle' | 'running' | 'stopping' | 'success' | 'partial' | 'failure' | 'stopped' | 'empty';

type DashboardTask = {
  status: TaskStatus;
  completed: number;
  total: number;
  current?: DatasetKey;
  message: string;
};

type ChartPoint = {
  label: string;
  value: number;
  series?: string;
  status?: InsightStatus;
  source?: MetricCard;
};

type ExportChartSpec = {
  id: string;
  title: string;
  subtitle: string;
  explanationId: ChartExplanationId;
  kind: ChartKind;
  points: ChartPoint[];
};

type ExportSection = {
  id: AnalyticsTab;
  title: string;
  description: string;
  charts: ExportChartSpec[];
};

type PdfReport = {
  batchId: string;
  batchName: string;
  analysisRunId: string;
  generatedAt: string;
  timeZone: string;
  filterSummary: string;
  sections: ExportSection[];
  omittedCharts: string[];
  failures: string[];
};

const emptyDataset: StructuredDataset = {
  coverage: [],
  kpis: [],
  appRank: [],
  hourlyTrend: [],
  networkHotspots: [],
  userProfiles: [],
  userSummary: [],
  leadEvidence: [],
  leadSummary: [],
};

const viewDatasets: Record<AnalyticsTab, DatasetKey[]> = {
  overview: ['coverage', 'kpis', 'appRank', 'hourlyTrend', 'networkHotspots', 'userSummary', 'leadSummary'],
  apps: ['coverage', 'appRank'],
  quality: ['coverage', 'networkHotspots'],
  cable: ['coverage', 'hourlyTrend'],
  users: ['coverage', 'userSummary', 'userProfiles'],
  leads: ['coverage', 'leadSummary', 'leadEvidence'],
};

const allDatasetKeys: DatasetKey[] = ['coverage', 'kpis', 'appRank', 'hourlyTrend', 'networkHotspots', 'userSummary', 'userProfiles', 'leadSummary', 'leadEvidence'];
const allAnalyticsTabs: AnalyticsTab[] = ['overview', 'apps', 'quality', 'cable', 'users', 'leads'];

const datasetLabels: Record<DatasetKey, string> = {
  coverage: '数据覆盖状态',
  kpis: '总览指标',
  appRank: '应用体验排行',
  hourlyTrend: '小时趋势',
  networkHotspots: '网络 / 路径证据',
  userProfiles: '用户画像',
  userSummary: '用户全量分群',
  leadEvidence: '体验驱动机会证据',
  leadSummary: '机会全量分层',
};

function hasMeaningfulEvidence(key: DatasetKey, rows: MetricCard[]) {
  if (key !== 'kpis') return rows.length > 0;
  return rows.some((row) => Math.abs(numberValue(row.value)) > 0);
}

const pageCopy: Record<AnalyticsTab, { eyebrow: string; title: string; description: string }> = {
  overview: { eyebrow: 'Comprehensive insights', title: '全量体验洞察', description: '保留完整业务与体验分布，在同一视图中高亮关注项和严重项，再进入问题调查。' },
  apps: { eyebrow: 'Application portfolio', title: '应用体验全景', description: '展示全部 App × 接入类型组合及业务规模、样本、体验状态；问题项高亮，但正常项不会被隐藏。' },
  quality: { eyebrow: 'Network / path evidence', title: '网络 / 路径证据', description: '只在真实字段与足够样本支持时展示可疑聚集；缺失拓扑不包装为热点或已确认根因。' },
  cable: { eyebrow: 'Access benchmark', title: 'Cable vs FTTH', description: '在相同时间口径下比较速率、时延、丢包和体验结果。' },
  users: { eyebrow: 'User evidence', title: '用户洞察', description: '查看用户需求、体验、接入识别和机会证据，而不是只看一个评分。' },
  leads: { eyebrow: 'Experience-driven opportunity', title: 'Cable-to-Fiber 体验机会', description: '体验 Finding 与商业机会分开；这里仅表示体验驱动候选，正式营销仍需 CRM、覆盖、套餐、欠费、黑名单和可触达资格。' },
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

function appInsightStatus(row: MetricCard): InsightStatus {
  const detail = parseMetricHint(row.hint);
  if (detail.sample_status === 'INSUFFICIENT_SAMPLE') return 'insufficient';
  if (detail.attention_level === 'SEVERE') return 'problem';
  if (detail.attention_level === 'ATTENTION') return 'watch';
  if (detail.attention_level === 'NORMAL' && detail.sample_status === 'SUFFICIENT') return 'healthy';
  return 'unclassified';
}

const insightStatusMeta: Record<InsightStatus, { zh: string; en: string; color: string; rank: number }> = {
  problem: { zh: '问题', en: 'Severe', color: '#d92d20', rank: 0 },
  watch: { zh: '关注', en: 'Attention', color: '#f79009', rank: 1 },
  healthy: { zh: '正常', en: 'Normal', color: '#12b76a', rank: 2 },
  insufficient: { zh: '样本不足', en: 'Insufficient', color: '#98a2b3', rank: 3 },
  unclassified: { zh: '未分级', en: 'Legacy / unavailable', color: '#2e90fa', rank: 4 },
};

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
    const detail = parseMetricHint(row.hint);
    const access = textFromHint(row, 'user_type', 'UNKNOWN');
    const app = textFromHint(row, 'app_name', row.label);
    const requiresSufficientSample = ['poor_experience_users', 'poor_experience_user_pct', 'persistent_poor_users', 'persistent_poor_user_rate_pct', 'severe_poor_user_rate_pct'].includes(key);
    const excluded = requiresSufficientSample && detail.sample_status === 'INSUFFICIENT_SAMPLE';
    return { label: `${app} · ${access}`, value: excluded ? 0 : fromHint(row, key), series: access, status: appInsightStatus(row), source: row };
  }).filter((row) => row.value > 0);
}

function appPopulationPoints(rows: MetricCard[], key: 'eligible_users' | 'traffic_gb'): ChartPoint[] {
  return rows.map((row) => {
    const app = textFromHint(row, 'app_name', row.label);
    const access = textFromHint(row, 'user_type', 'UNKNOWN');
    const value = key === 'eligible_users'
      ? fromHint(row, 'eligible_users') || fromHint(row, 'users')
      : fromHint(row, key);
    return { label: `${app} · ${access}`, value, series: access, status: appInsightStatus(row), source: row };
  }).filter((row) => row.value > 0);
}

function appStatusPoints(rows: MetricCard[]): ChartPoint[] {
  const counts = rows.reduce<Record<InsightStatus, number>>((result, row) => {
    result[appInsightStatus(row)] += 1;
    return result;
  }, { healthy: 0, watch: 0, problem: 0, insufficient: 0, unclassified: 0 });
  return (Object.keys(counts) as InsightStatus[])
    .filter((status) => counts[status] > 0)
    .sort((left, right) => insightStatusMeta[left].rank - insightStatusMeta[right].rank)
    .map((status) => ({ label: insightStatusMeta[status].zh, value: counts[status], status }));
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

function typicalHourlyPoints(rows: MetricCard[], key: string): ChartPoint[] {
  const grouped = new Map<string, { weighted: number; users: number; source: MetricCard }>();
  rows.forEach((row) => {
    const detail = parseMetricHint(row.hint);
    const hour = String(detail.hour ?? '').padStart(2, '0');
    const series = detail.user_type || 'UNKNOWN';
    const users = Math.max(0, fromHint(row, 'users'));
    const value = fromHint(row, key);
    if (!hour || !Number.isFinite(value) || users <= 0) return;
    const mapKey = `${hour}|${series}`;
    const current = grouped.get(mapKey);
    grouped.set(mapKey, {
      weighted: (current?.weighted ?? 0) + value * users,
      users: (current?.users ?? 0) + users,
      source: current?.source ?? row,
    });
  });
  return [...grouped.entries()].map(([mapKey, item]) => {
    const [hour, series] = mapKey.split('|');
    return { label: `${hour}:00`, series, value: item.weighted / item.users, source: item.source };
  }).sort((left, right) => left.label.localeCompare(right.label));
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
    const users = parseMetricHint(row.hint).users === undefined ? 1 : fromHint(row, 'users');
    grouped.set(stage, { count: (current?.count ?? 0) + users, source: current?.source ?? row });
  });
  return [...grouped.entries()].map(([label, item]) => ({ label, value: item.count, source: item.source }));
}

function cohortPoints(rows: MetricCard[], summaryType: string): ChartPoint[] {
  return rows.filter((row) => textFromHint(row, 'summary_type') === summaryType).map((row) => ({
    label: `${textFromHint(row, 'segment', row.label)} · ${textFromHint(row, 'user_type', 'UNKNOWN')}`,
    value: fromHint(row, 'users'),
    series: textFromHint(row, 'user_type', 'UNKNOWN'),
    source: row,
  })).filter((row) => row.value > 0);
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

function filteredDataset(data: StructuredDataset, access: string, keyword: string, minUsers: number) {
  return {
    appRank: filterRows(data.appRank, access, keyword, minUsers),
    hourlyTrend: filterRows(data.hourlyTrend, access, keyword, minUsers),
    networkHotspots: filterRows(data.networkHotspots, access, keyword, minUsers),
    userSummary: filterRows(data.userSummary, access, keyword, minUsers),
    userProfiles: filterRows(data.userProfiles, access, keyword, minUsers),
    leadSummary: filterRows(data.leadSummary, access, keyword, minUsers),
    leadEvidence: filterRows(data.leadEvidence, access, keyword, minUsers),
  };
}

function exportSections(filtered: ReturnType<typeof filteredDataset>): ExportSection[] {
  const leadStages = leadStagePoints(filtered.leadSummary);
  return [
    {
      id: 'overview',
      title: pageCopy.overview.title,
      description: pageCopy.overview.description,
      charts: [
        { id: 'overview-app-status', title: '全部 App 体验状态构成', subtitle: '完整 App × 接入组合，问题项在全量分布中高亮', explanationId: 'app_portfolio_status', kind: 'donut', points: appStatusPoints(filtered.appRank) },
        { id: 'overview-app-population', title: 'App 用户覆盖全景', subtitle: '按合格用户规模展示，颜色代表策略状态', explanationId: 'app_population', kind: 'bar', points: appPopulationPoints(filtered.appRank, 'eligible_users') },
        { id: 'overview-app-users', title: '高亮问题 App 持续差体验用户', subtitle: '满足持续性与最低样本规则的唯一用户', explanationId: 'app_affected_users', kind: 'bar', points: appPoints(filtered.appRank, 'persistent_poor_users') },
        { id: 'overview-user-demand', title: '全量用户需求分层', subtitle: '完整人群的需求结构', explanationId: 'user_demand_band', kind: 'bar', points: cohortPoints(filtered.userSummary, 'demand_band') },
        { id: 'overview-hotspots', title: '网络 / 路径可疑聚集', subtitle: '只有真实网络对象和足够样本才可解释为热点', explanationId: 'topology_poor_user_rate', kind: 'bar', points: hotspotPoints(filtered.networkHotspots, 'severity') },
        { id: 'overview-leads', title: '机会与排除分层', subtitle: '按用户计数；A0/A2 不得进入直接营销', explanationId: 'lead_stage', kind: 'bar', points: leadStages },
        { id: 'overview-hourly-rate', title: '典型日接入类型速率', subtitle: '按活跃用户加权的 7 日小时均值，Mbps', explanationId: 'typical_effective_rate', kind: 'line', points: typicalHourlyPoints(filtered.hourlyTrend, 'effective_mbps') },
      ],
    },
    {
      id: 'apps',
      title: pageCopy.apps.title,
      description: pageCopy.apps.description,
      charts: [
        { id: 'apps-status', title: '全部 App 体验状态构成', subtitle: '完整 App × 接入组合，问题项在全量分布中高亮', explanationId: 'app_portfolio_status', kind: 'donut', points: appStatusPoints(filtered.appRank) },
        { id: 'apps-population', title: 'App 用户覆盖全景', subtitle: '按合格用户规模展示，颜色代表策略状态', explanationId: 'app_population', kind: 'bar', points: appPopulationPoints(filtered.appRank, 'eligible_users') },
        { id: 'apps-users', title: '高亮：持续差体验用户', subtitle: '满足持续性与最低样本规则的唯一用户', explanationId: 'app_affected_users', kind: 'bar', points: appPoints(filtered.appRank, 'persistent_poor_users') },
        { id: 'apps-persistent-rate', title: 'App 持续差体验用户占比', subtitle: '持续差体验用户 / 合格用户，单位 %', explanationId: 'app_affected_user_rate', kind: 'bar', points: appPoints(filtered.appRank, 'persistent_poor_user_rate_pct') },
        { id: 'apps-observation-rate', title: 'App 差体验观测占比', subtitle: '差体验观测 / 有效观测，单位 %', explanationId: 'app_poor_observation_rate', kind: 'bar', points: appPoints(filtered.appRank, 'poor_observation_rate_pct') },
        { id: 'apps-ever-rate', title: 'App 曾受影响用户占比', subtitle: '至少异常一次的合格用户 / 合格用户，单位 %', explanationId: 'app_ever_affected_user_rate', kind: 'bar', points: appPoints(filtered.appRank, 'ever_affected_user_rate_pct') },
        { id: 'apps-severe-rate', title: 'App 严重差体验用户占比', subtitle: '严重差体验用户 / 合格用户，单位 %', explanationId: 'app_severe_user_rate', kind: 'bar', points: appPoints(filtered.appRank, 'severe_poor_user_rate_pct') },
        { id: 'apps-traffic', title: 'App 业务流量', subtitle: '视频下载 GB；游戏类需结合时长证据', explanationId: 'app_tcp_traffic', kind: 'bar', points: appPoints(filtered.appRank, 'traffic_gb') },
      ],
    },
    {
      id: 'quality',
      title: pageCopy.quality.title,
      description: pageCopy.quality.description,
      charts: [
        { id: 'quality-users', title: '可识别网络对象受影响用户', subtitle: '只统计真实 BRAS / OLT / PON；缺失值不作为节点', explanationId: 'topology_affected_users', kind: 'bar', points: hotspotPoints(filtered.networkHotspots, 'users') },
        { id: 'quality-ratio', title: '网络 / 路径可疑聚集率', subtitle: '没有真实拓扑或样本不足时仅作证据，不确认热点', explanationId: 'topology_poor_user_rate', kind: 'bar', points: hotspotPoints(filtered.networkHotspots, 'severity') },
        { id: 'quality-network-rtt', title: '网络侧 RTT', subtitle: '节点平均 network-side RTT，单位 ms', explanationId: 'network_side_rtt', kind: 'bar', points: hotspotPoints(filtered.networkHotspots, 'network_rtt_ms') },
        { id: 'quality-home-rtt', title: '家庭侧 / Wi-Fi RTT', subtitle: '节点平均 subscriber-side RTT，单位 ms', explanationId: 'subscriber_side_rtt', kind: 'bar', points: hotspotPoints(filtered.networkHotspots, 'subscriber_rtt_ms') },
      ],
    },
    {
      id: 'cable',
      title: pageCopy.cable.title,
      description: pageCopy.cable.description,
      charts: [
        { id: 'cable-rate', title: 'Cable / FTTH 典型日有效速率', subtitle: '按活跃用户加权的 7 日小时均值，Mbps', explanationId: 'typical_effective_rate', kind: 'line', points: typicalHourlyPoints(filtered.hourlyTrend, 'effective_mbps') },
        { id: 'cable-rtt', title: 'Cable / FTTH 典型日 RTT', subtitle: '按活跃用户加权的 subscriber-side RTT，ms', explanationId: 'typical_subscriber_rtt', kind: 'line', points: typicalHourlyPoints(filtered.hourlyTrend, 'subscriber_rtt_ms') },
        { id: 'cable-loss', title: 'Cable / FTTH 典型日用户侧丢包', subtitle: '按活跃用户加权的 user-side downstream loss，%', explanationId: 'typical_user_loss', kind: 'line', points: typicalHourlyPoints(filtered.hourlyTrend, 'user_loss_pct') },
      ],
    },
    {
      id: 'users',
      title: pageCopy.users.title,
      description: pageCopy.users.description,
      charts: [
        { id: 'users-demand', title: '用户需求分层', subtitle: '全量用户分群；评分不等同于可营销资格', explanationId: 'user_demand_band', kind: 'bar', points: cohortPoints(filtered.userSummary, 'demand_band') },
        { id: 'users-traffic', title: '用户流量分层', subtitle: '全量用户按分析周期 TCP 流量分群', explanationId: 'user_traffic_band', kind: 'bar', points: cohortPoints(filtered.userSummary, 'traffic_band') },
        { id: 'users-bottleneck', title: '用户问题侧分布', subtitle: '全量用户按主要瓶颈侧分群', explanationId: 'user_issue_side', kind: 'bar', points: cohortPoints(filtered.userSummary, 'bottleneck_side') },
      ],
    },
    {
      id: 'leads',
      title: pageCopy.leads.title,
      description: pageCopy.leads.description,
      charts: [
        { id: 'leads-stage', title: '机会与排除分层', subtitle: '按唯一用户计数；A0 待外部资格数据、A2 先修障、A1 待资格校验', explanationId: 'lead_stage', kind: 'bar', points: leadStages },
        { id: 'leads-share', title: '机会分层构成', subtitle: '当前分析运行中的用户分层占比', explanationId: 'lead_stage_share', kind: 'donut', points: leadStages },
        { id: 'leads-demand', title: '候选用户需求评分', subtitle: '评分用于排序；最终行动仍由问题侧与资格字段决定', explanationId: 'lead_demand_score', kind: 'bar', points: userPoints(filtered.leadEvidence, 'demand_score') },
      ],
    },
  ];
}

function chartOption(kind: ChartKind, title: string, subtitle: string, incoming: ChartPoint[]) {
  const ink = '#172033';
  const muted = '#667085';
  const grid = '#e7ecf3';
  const palette = ['#2563eb', '#d97706', '#64748b', '#7c3aed', '#0f766e'];
  const points = kind === 'line' ? incoming : [...incoming].sort((a, b) => b.value - a.value).slice(0, 12);
  const pointColor = (point: ChartPoint) => point.status ? insightStatusMeta[point.status].color : palette[0];
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
      series: [{ type: 'pie', radius: ['48%', '70%'], center: ['50%', '48%'], data: points.map((point) => ({ name: point.label, value: point.value, itemStyle: { color: pointColor(point) } })), label: { color: ink, formatter: '{b}\n{c}' } }],
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
    series: [{ type: 'bar', data: points.map((point) => ({ value: point.value, itemStyle: { color: pointColor(point), borderRadius: [0, 4, 4, 0] } })).reverse(), barMaxWidth: 22, label: { show: true, position: 'right', color: muted, formatter: ({ value }: { value: number }) => compact(value) } }],
  };
}

function AnalyticsChart({ title, subtitle, explanationId, kind, points, onSelect, height = 380 }: { title: string; subtitle: string; explanationId: ChartExplanationId; kind: ChartKind; points: ChartPoint[]; onSelect?: (row: MetricCard) => void; height?: number }) {
  const ref = useRef<HTMLDivElement | null>(null);
  const effectiveKind: ChartKind = kind === 'line' && new Set(points.map((point) => point.label)).size < 8 ? 'bar' : kind;
  const renderedPoints = useMemo(() => kind === 'line' && effectiveKind === 'bar'
    ? points.map((point) => ({ ...point, label: `${point.label} · ${point.series || 'ALL'}` }))
    : points, [effectiveKind, kind, points]);
  useEffect(() => {
    if (!ref.current) return;
    let chart: ECharts | undefined;
    let disposed = false;
    const resize = () => chart?.resize();
    void import('echarts').then((echarts) => {
      if (disposed || !ref.current) return;
      chart = echarts.init(ref.current);
      chart.setOption(chartOption(effectiveKind, title, subtitle, renderedPoints), true);
      chart.on('click', (params) => {
        const label = String(params.name ?? '');
        const source = renderedPoints.find((point) => point.label === label)?.source;
        if (source && onSelect) onSelect(source);
      });
      window.addEventListener('resize', resize);
    });
    return () => {
      disposed = true;
      window.removeEventListener('resize', resize);
      chart?.dispose();
    };
  }, [effectiveKind, height, onSelect, renderedPoints, subtitle, title]);
  const explanation = chartExplanationCatalog[explanationId];
  return <article className="analytics-card analytics-chart-card">
    <div className="analytics-chart" style={{ height }} ref={ref} />
    <footer><span>来源：DWS / ADS</span><span>{kind !== effectiveKind ? '时点不足 8 个，已降级为柱图' : `${points.length} data points`}</span></footer>
    <ChartExplanation {...explanation} />
  </article>;
}

function KpiStrip({ items }: { items: Array<{ label: string; value: string; hint: string; tone?: string }> }) {
  return <section className="analytics-kpi-strip">{items.map((item) => <article key={item.label} className={`analytics-kpi-card ${item.tone ? `tone-${item.tone}` : ''}`}><span>{item.label}</span><strong>{item.value}</strong><small>{item.hint}</small></article>)}</section>;
}

function displayMetric(detail: Record<string, string>, key: string, suffix = '') {
  const value = detail[key];
  if (value === undefined || value === '' || value === 'NA' || value === 'NULL') return '—';
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return value;
  return `${compact(numeric)}${suffix}`;
}

function AppPortfolioTable({ rows, onSelect }: { rows: MetricCard[]; onSelect: (row: MetricCard) => void }) {
  const ordered = useMemo(() => [...rows].sort((left, right) => {
    const status = insightStatusMeta[appInsightStatus(left)].rank - insightStatusMeta[appInsightStatus(right)].rank;
    if (status !== 0) return status;
    return (fromHint(right, 'eligible_users') || fromHint(right, 'users')) - (fromHint(left, 'eligible_users') || fromHint(left, 'users'));
  }), [rows]);
  return <article className="analytics-card analytics-table-card app-portfolio-card">
    <div className="analytics-section-head">
      <div><h3>全部 App 体验组合</h3><p>每行是 App × 接入类型；完整保留正常、关注、问题、样本不足与旧口径未分级对象。</p></div>
      <span>{ordered.length} combinations</span>
    </div>
    <div className="app-status-legend">
      {(Object.keys(insightStatusMeta) as InsightStatus[]).map((status) => <span key={status}><i style={{ background: insightStatusMeta[status].color }} />{insightStatusMeta[status].zh} · {insightStatusMeta[status].en}</span>)}
    </div>
    <div className="analytics-table-wrap app-portfolio-table-wrap">
      <table className="analytics-table app-portfolio-table">
        <thead><tr><th>状态</th><th>App / 分类</th><th>接入</th><th>观测用户</th><th>合格用户</th><th>有效观测</th><th>差观测占比</th><th>持续差用户</th><th>持续差占比</th><th>严重占比</th><th>业务量</th><th>主要证据</th><th>下钻</th></tr></thead>
        <tbody>
          {ordered.map((row, index) => {
            const detail = parseMetricHint(row.hint);
            const status = appInsightStatus(row);
            const meta = insightStatusMeta[status];
            return <tr key={`${row.label}-${index}`} className={`app-status-row status-${status}`}>
              <td><span className={`app-status-badge status-${status}`}>{meta.zh}</span></td>
              <td><strong>{detail.app_name || row.label}</strong><small>{detail.app_category || 'UNKNOWN'}</small></td>
              <td>{detail.user_type || 'UNKNOWN'}</td>
              <td>{displayMetric(detail, 'observed_users')}</td>
              <td>{displayMetric(detail, detail.eligible_users === undefined ? 'users' : 'eligible_users')}</td>
              <td>{displayMetric(detail, 'valid_obs_rows')}</td>
              <td>{displayMetric(detail, 'poor_observation_rate_pct', '%')}</td>
              <td>{displayMetric(detail, 'persistent_poor_users')}</td>
              <td>{displayMetric(detail, 'persistent_poor_user_rate_pct', '%')}</td>
              <td>{displayMetric(detail, 'severe_poor_user_rate_pct', '%')}</td>
              <td>{displayMetric(detail, 'traffic_gb', ' GB')}</td>
              <td title={detail.issue_driver || ''}>{detail.issue_driver || (status === 'healthy' ? '当前策略未标记异常' : '—')}</td>
              <td><button type="button" onClick={() => onSelect(row)}>加入上下文</button></td>
            </tr>;
          })}
          {!ordered.length && <tr><td colSpan={13}>当前筛选下没有 App 聚合结果。</td></tr>}
        </tbody>
      </table>
    </div>
    <p className="app-portfolio-note">状态直接读取本次 analysis run 绑定策略生成的 sample_status 与 attention_level；页面不另设隐藏阈值。百分比为“—”表示不可用，不按 0 展示。</p>
  </article>;
}

function EvidenceDrawer({ row, onClose }: { row: MetricCard; onClose: () => void }) {
  const detail = parseMetricHint(row.hint);
  return <div className="analytics-evidence-drawer-backdrop" role="presentation" onClick={onClose}><aside className="analytics-evidence-drawer" role="dialog" aria-modal="true" onClick={(event) => event.stopPropagation()}><header><div><p className="eyebrow">Evidence</p><h3>{row.label}</h3></div><button type="button" onClick={onClose}>关闭</button></header><div className="analytics-evidence-kv">{Object.entries(detail).map(([key, value]) => <div key={key}><span>{key}</span><strong>{value}</strong></div>)}</div></aside></div>;
}

function PdfPreview({ report, onClose }: { report: PdfReport; onClose: () => void }) {
  const chartCount = report.sections.reduce((sum, section) => sum + section.charts.length, 0);
  return <div className="analytics-pdf-preview-backdrop" role="presentation" onClick={onClose}>
    <div className="analytics-pdf-preview-dialog" role="dialog" aria-modal="true" aria-label="全部图表 PDF 预览" onClick={(event) => event.stopPropagation()}>
      <header className="analytics-pdf-preview-toolbar">
        <div><strong>全部图表报告已就绪</strong><span>{chartCount} 张图 · 不包含明细表格</span></div>
        <div><button type="button" onClick={onClose}>关闭预览</button><button type="button" className="primary-button" onClick={() => window.print()}>打开打印 / 保存 PDF</button></div>
      </header>
      <main className="analytics-print-report">
        <section className="analytics-report-cover">
          <p className="eyebrow">SA FBB Experience Workbench</p>
          <h1>应用体验分析图表报告</h1>
          <p>按当前批次与筛选条件生成，默认包含六类决策看板中的全部非空图表，不包含明细表格。</p>
          <dl>
            <div><dt>批次</dt><dd>{report.batchName}</dd></div>
            <div><dt>import_batch_id</dt><dd>{report.batchId}</dd></div>
            <div><dt>analysis_run_id</dt><dd>{report.analysisRunId}</dd></div>
            <div><dt>筛选条件</dt><dd>{report.filterSummary}</dd></div>
            <div><dt>本地生成时间</dt><dd>{report.generatedAt}</dd></div>
            <div><dt>本地时区</dt><dd>{report.timeZone}</dd></div>
            <div><dt>数据来源</dt><dd>DWS / ADS 聚合结果</dd></div>
            <div><dt>报告内容</dt><dd>{chartCount} 张图表 · 0 张明细表</dd></div>
          </dl>
          {(report.omittedCharts.length > 0 || report.failures.length > 0) && <aside>
            {report.omittedCharts.length > 0 && <p><strong>无数据已跳过：</strong>{report.omittedCharts.join('、')}</p>}
            {report.failures.length > 0 && <p><strong>查询失败未纳入：</strong>{report.failures.join('；')}</p>}
          </aside>}
        </section>
        {report.sections.map((section) => <section className="analytics-report-section" key={section.id}>
          <header><p className="eyebrow">{pageCopy[section.id].eyebrow}</p><h2>{section.title}</h2><p>{section.description}</p></header>
          {section.charts.length > 0
            ? <div className="analytics-report-chart-grid">{section.charts.map((chart) => <AnalyticsChart key={chart.id} title={chart.title} subtitle={chart.subtitle} explanationId={chart.explanationId} kind={chart.kind} points={chart.points} height={330} />)}</div>
            : <div className="analytics-report-empty">当前筛选条件下，本看板没有可导出的非空图表。</div>}
        </section>)}
      </main>
    </div>
  </div>;
}

export function AnalyticsDashboard({ c, activeView, batchContext, onOpenImport }: { c: WorkbenchController; activeView: AnalyticsTab; batchContext?: BatchListItem; onOpenImport: () => void }) {
  const [data, setData] = useState<StructuredDataset>(emptyDataset);
  const [loadedViews, setLoadedViews] = useState<Partial<Record<AnalyticsTab, string>>>({});
  const [task, setTask] = useState<DashboardTask>({ status: 'idle', completed: 0, total: 0, message: '等待用户启动加载。' });
  const [exportTask, setExportTask] = useState<DashboardTask>({ status: 'idle', completed: 0, total: allDatasetKeys.length, message: '需要时可导出六类看板中的全部图表。' });
  const [exportFailures, setExportFailures] = useState<string[]>([]);
  const [exportFailedDatasetKeys, setExportFailedDatasetKeys] = useState<DatasetKey[]>([]);
  const [pdfReport, setPdfReport] = useState<PdfReport | null>(null);
  const [pdfPreviewOpen, setPdfPreviewOpen] = useState(false);
  const [failures, setFailures] = useState<string[]>([]);
  const [failedDatasetKeys, setFailedDatasetKeys] = useState<DatasetKey[]>([]);
  const [emptyDatasetKeys, setEmptyDatasetKeys] = useState<DatasetKey[]>([]);
  const [datasetCounts, setDatasetCounts] = useState<Partial<Record<DatasetKey, number>>>({});
  const [access, setAccess] = useState('ALL');
  const [keyword, setKeyword] = useState('');
  const [minUsers, setMinUsers] = useState(0);
  const [selectedEvidence, setSelectedEvidence] = useState<MetricCard | null>(null);
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();
  const actionBusy = Boolean(c.currentAction);
  const contextKey = `${c.importBatchId.trim()}::${c.analysisRunId.trim()}`;
  const viewLoaded = loadedViews[activeView] === contextKey;
  const stopRequested = useRef(false);
  const taskGeneration = useRef(0);
  const exportStopRequested = useRef(false);
  const exportGeneration = useRef(0);

  function loadDataset(key: DatasetKey, importBatchId = c.importBatchId, analysisRunId = c.analysisRunId, settings: MySqlSettings = c.effectiveSettings) {
    if (key === 'coverage') return analyticsStructuredApi.coverage(settings, importBatchId, analysisRunId);
    if (key === 'kpis') return analyticsStructuredApi.kpis(settings, importBatchId, analysisRunId);
    if (key === 'appRank') return analyticsStructuredApi.appRank(settings, importBatchId, analysisRunId, { pageSize: 500 });
    if (key === 'hourlyTrend') return analyticsStructuredApi.hourlyTrend(settings, importBatchId, analysisRunId, { pageSize: 500, sortBy: 'hour' });
    if (key === 'networkHotspots') return analyticsStructuredApi.networkHotspots(settings, importBatchId, analysisRunId, { pageSize: 200, sortBy: 'users' });
    if (key === 'userSummary') return analyticsStructuredApi.userSummary(settings, importBatchId, analysisRunId);
    if (key === 'userProfiles') return analyticsStructuredApi.userProfiles(settings, importBatchId, analysisRunId, { pageSize: 300, sortBy: 'demand' });
    if (key === 'leadSummary') return analyticsStructuredApi.leadSummary(settings, importBatchId, analysisRunId);
    return analyticsStructuredApi.leadEvidence(settings, importBatchId, analysisRunId, { pageSize: 500, sortBy: 'demand' });
  }

  useEffect(() => {
    taskGeneration.current += 1;
    stopRequested.current = true;
    exportGeneration.current += 1;
    exportStopRequested.current = true;
    setData(emptyDataset);
    setFailures([]);
    setFailedDatasetKeys([]);
    setEmptyDatasetKeys([]);
    setDatasetCounts({});
    setLoadedViews({});
    setTask({ status: 'idle', completed: 0, total: 0, message: disabled ? '请先选择批次并填写 analysis_run_id。' : '上下文已就绪，等待用户启动加载。' });
    setExportTask({ status: 'idle', completed: 0, total: allDatasetKeys.length, message: disabled ? '请先选择批次并填写 analysis_run_id。' : '需要时可导出六类看板中的全部图表。' });
    setExportFailures([]);
    setExportFailedDatasetKeys([]);
    setPdfReport(null);
    setPdfPreviewOpen(false);
  }, [c.importBatchId, c.analysisRunId]);

  useEffect(() => {
    setAccess(c.analysisContext.access_type ?? 'ALL');
    setKeyword(c.analysisContext.app_name ?? c.analysisContext.user_key ?? c.analysisContext.bras ?? '');
  }, [c.analysisContext.access_type, c.analysisContext.app_name, c.analysisContext.user_key, c.analysisContext.bras]);

  useEffect(() => {
    taskGeneration.current += 1;
    stopRequested.current = true;
    setFailures([]);
    setFailedDatasetKeys([]);
    setEmptyDatasetKeys([]);
    setDatasetCounts({});
    const cached = loadedViews[activeView] === contextKey;
    setTask(cached
      ? { status: 'success', completed: viewDatasets[activeView].length, total: viewDatasets[activeView].length, message: '当前看板使用本次会话缓存；需要时可手动重新加载。' }
      : { status: 'idle', completed: 0, total: 0, message: '等待用户启动当前看板加载。' });
  }, [activeView]);

  async function loadCurrentView() {
    if (disabled || actionBusy || task.status === 'running' || task.status === 'stopping') return;
    const keys = viewDatasets[activeView];
    const requestedContext = contextKey;
    const generation = taskGeneration.current + 1;
    taskGeneration.current = generation;
    stopRequested.current = false;
    setFailures([]);
    setFailedDatasetKeys([]);
    setEmptyDatasetKeys([]);
    setDatasetCounts({});
    setTask({ status: 'running', completed: 0, total: keys.length, current: keys[0], message: `正在加载：${datasetLabels[keys[0]]}` });
    const result = await c.runAction(`analytics_load_${activeView}`, async () => {
      const nextFailures: string[] = [];
      let completed = 0;
      let rowCount = 0;
      let meaningfulDatasets = 0;
      const nextEmptyKeys: DatasetKey[] = [];
      for (const key of keys) {
        if (stopRequested.current || generation !== taskGeneration.current) break;
        setTask({ status: 'running', completed, total: keys.length, current: key, message: `正在加载：${datasetLabels[key]}` });
        try {
          const rows = await loadDataset(key);
          if (generation !== taskGeneration.current) break;
          setData((current) => ({ ...current, [key]: rows }));
          setDatasetCounts((current) => ({ ...current, [key]: rows.length }));
          if (key === 'kpis') c.setOverview({ metrics: rows });
          rowCount += rows.length;
          if (hasMeaningfulEvidence(key, rows)) meaningfulDatasets += 1;
          else nextEmptyKeys.push(key);
        } catch (error) {
          nextFailures.push(`${datasetLabels[key]}: ${error instanceof Error ? error.message : String(error)}`);
          setFailedDatasetKeys((current) => [...current, key]);
        }
        completed += 1;
        setTask({ status: stopRequested.current ? 'stopping' : 'running', completed, total: keys.length, message: stopRequested.current ? '当前查询已结束，正在停止后续步骤。' : `已完成 ${completed}/${keys.length} 个数据集。` });
      }
      return { completed, total: keys.length, rowCount, meaningfulDatasets, emptyDatasetKeys: nextEmptyKeys, failures: nextFailures, stopped: stopRequested.current };
    }) as { completed: number; total: number; rowCount: number; meaningfulDatasets: number; emptyDatasetKeys: DatasetKey[]; failures: string[]; stopped: boolean } | null;

    if (generation !== taskGeneration.current) return;
    if (!result) {
      setTask({ status: 'failure', completed: 0, total: keys.length, message: '加载失败，可查看日志后重试。' });
      return;
    }
    setFailures(result.failures);
    setEmptyDatasetKeys(result.emptyDatasetKeys);
    if (result.stopped) {
      setTask({ status: 'stopped', completed: result.completed, total: result.total, message: `已停止后续加载；已完成 ${result.completed}/${result.total} 个数据集。` });
      return;
    }
    const noEvidence = result.meaningfulDatasets === 0;
    if (!noEvidence) setLoadedViews((current) => ({ ...current, [activeView]: requestedContext }));
    const incomplete = result.emptyDatasetKeys.length > 0;
    const status = noEvidence ? 'empty' : result.failures.length || incomplete ? 'partial' : 'success';
    const message = noEvidence
      ? `查询已完成，但 ${result.completed} 个数据集都没有可分析聚合数据；返回的 ${result.rowCount} 条指标结构为全 0 或空结果${result.failures.length ? `，另有 ${result.failures.length} 个查询失败` : ''}。`
      : `已加载 ${result.rowCount} 条聚合记录${result.failures.length ? `，${result.failures.length} 个查询失败` : ''}${incomplete ? `，${result.emptyDatasetKeys.length} 个数据集为空` : ''}。`;
    setTask({ status, completed: result.completed, total: result.total, message });
  }

  function stopLoading() {
    stopRequested.current = true;
    setTask((current) => ({ ...current, status: 'stopping', message: '停止请求已接收；当前查询完成后不再加载后续数据集。' }));
  }

  async function preparePdfReport() {
    if (disabled || actionBusy || exportTask.status === 'running' || exportTask.status === 'stopping') return;
    const requestedBatchId = c.importBatchId.trim();
    const requestedAnalysisRunId = c.analysisRunId.trim();
    const requestedContext = `${requestedBatchId}::${requestedAnalysisRunId}`;
    const requestedSettings = c.effectiveSettings;
    const requestedBatchName = c.batchDisplayName || batchContext?.batch_display_name || batchContext?.source_file_name || requestedBatchId;
    const requestedFilters = { access, keyword, minUsers };
    const generation = exportGeneration.current + 1;
    exportGeneration.current = generation;
    exportStopRequested.current = false;
    setPdfReport(null);
    setPdfPreviewOpen(false);
    setExportFailures([]);
    setExportFailedDatasetKeys([]);
    setExportTask({ status: 'running', completed: 0, total: allDatasetKeys.length, current: allDatasetKeys[0], message: `正在准备：${datasetLabels[allDatasetKeys[0]]}` });

    const result = await c.runAction('analytics_export_all_charts_pdf', async () => {
      const nextData: StructuredDataset = { ...emptyDataset };
      const nextFailures: string[] = [];
      const failedKeys: DatasetKey[] = [];
      let completed = 0;
      for (const key of allDatasetKeys) {
        if (exportStopRequested.current || generation !== exportGeneration.current) break;
        setExportTask({ status: 'running', completed, total: allDatasetKeys.length, current: key, message: `正在查询并准备：${datasetLabels[key]}` });
        try {
          nextData[key] = await loadDataset(key, requestedBatchId, requestedAnalysisRunId, requestedSettings);
        } catch (error) {
          const failure = `${datasetLabels[key]}: ${error instanceof Error ? error.message : String(error)}`;
          nextFailures.push(failure);
          failedKeys.push(key);
          setExportFailures((current) => [...current, failure]);
          setExportFailedDatasetKeys((current) => [...current, key]);
        }
        completed += 1;
        setExportTask({
          status: exportStopRequested.current ? 'stopping' : 'running',
          completed,
          total: allDatasetKeys.length,
          message: exportStopRequested.current ? '当前查询结束后将停止准备报告。' : `已完成 ${completed}/${allDatasetKeys.length} 个数据集。`,
        });
      }
      return { data: nextData, completed, failures: nextFailures, failedKeys, stopped: exportStopRequested.current };
    }) as { data: StructuredDataset; completed: number; failures: string[]; failedKeys: DatasetKey[]; stopped: boolean } | null;

    if (generation !== exportGeneration.current) return;
    if (!result) {
      setExportTask({ status: 'failure', completed: 0, total: allDatasetKeys.length, message: '报告准备失败，请查看执行日志后重试。' });
      return;
    }
    if (result.stopped) {
      setExportTask({ status: 'stopped', completed: result.completed, total: allDatasetKeys.length, message: `已停止报告准备；完成 ${result.completed}/${allDatasetKeys.length} 个数据集。` });
      return;
    }

    setData(result.data);
    setDatasetCounts(allDatasetKeys.reduce<Partial<Record<DatasetKey, number>>>((counts, key) => ({ ...counts, [key]: result.data[key].length }), {}));
    c.setOverview({ metrics: result.data.kpis });
    const failedSet = new Set(result.failedKeys);
    setLoadedViews((current) => {
      const next = { ...current };
      allAnalyticsTabs.forEach((tab) => {
        const keys = viewDatasets[tab];
        if (keys.every((key) => !failedSet.has(key)) && keys.some((key) => hasMeaningfulEvidence(key, result.data[key]))) next[tab] = requestedContext;
      });
      return next;
    });

    const preparedSections = exportSections(filteredDataset(result.data, requestedFilters.access, requestedFilters.keyword, requestedFilters.minUsers));
    const omittedCharts = preparedSections.flatMap((section) => section.charts.filter((chart) => chart.points.length === 0).map((chart) => `${section.title} / ${chart.title}`));
    const sections = preparedSections.map((section) => ({ ...section, charts: section.charts.filter((chart) => chart.points.length > 0) }));
    const chartCount = sections.reduce((sum, section) => sum + section.charts.length, 0);
    if (chartCount === 0) {
      setExportTask({ status: 'empty', completed: result.completed, total: allDatasetKeys.length, message: '查询完成，但当前筛选条件下没有可导出的非空图表。' });
      return;
    }
    const now = new Date();
    const timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone || '本地时区';
    const filterSummary = `接入类型=${requestedFilters.access}；关键词=${requestedFilters.keyword.trim() || '无'}；最小用户数=${requestedFilters.minUsers}`;
    setPdfReport({
      batchId: requestedBatchId,
      batchName: requestedBatchName,
      analysisRunId: requestedAnalysisRunId,
      generatedAt: now.toLocaleString(),
      timeZone,
      filterSummary,
      sections,
      omittedCharts,
      failures: result.failures,
    });
    setPdfPreviewOpen(true);
    const partial = result.failures.length > 0 || omittedCharts.length > 0;
    setExportTask({
      status: partial ? 'partial' : 'success',
      completed: result.completed,
      total: allDatasetKeys.length,
      message: `报告已准备 ${chartCount} 张图表${omittedCharts.length ? `，跳过 ${omittedCharts.length} 张空图` : ''}${result.failures.length ? `，${result.failures.length} 个查询失败` : ''}。`,
    });
  }

  function stopPdfPreparation() {
    exportStopRequested.current = true;
    setExportTask((current) => ({ ...current, status: 'stopping', message: '停止请求已接收；当前查询结束后不再准备后续数据集。' }));
  }

  const filtered = useMemo(() => filteredDataset(data, access, keyword, minUsers), [access, data, keyword, minUsers]);

  const leadStages = useMemo(() => leadStagePoints(filtered.leadSummary), [filtered.leadSummary]);
  const knownHourlyUsers = filtered.hourlyTrend.reduce((sum, row) => sum + (textFromHint(row, 'user_type') === 'UNKNOWN' ? 0 : fromHint(row, 'users')), 0);
  const allHourlyUsers = filtered.hourlyTrend.reduce((sum, row) => sum + fromHint(row, 'users'), 0);
  const coverage = allHourlyUsers > 0 ? knownHourlyUsers / allHourlyUsers * 100 : 0;
  const gameDataset = data.coverage.find((row) => row.label === 'Game Dataset');
  const gameImported = gameDataset?.value === 'AVAILABLE';
  const accessClassification = data.coverage.find((row) => row.label === 'Access Classification');
  const accessDefault = accessClassification?.value || 'UNKNOWN';
  const unknownHourlyRows = data.hourlyTrend.filter((row) => textFromHint(row, 'user_type') === 'UNKNOWN').length;
  const classificationStale = accessDefault !== 'UNKNOWN' && unknownHourlyRows > 0;
  const topologyKnownRows = data.networkHotspots.filter((row) => {
    const detail = parseMetricHint(row.hint);
    return detail.bras !== 'UNKNOWN' || detail.olt !== 'UNKNOWN' || detail.pon !== 'UNKNOWN';
  }).length;
  const a1 = leadStages.find((stage) => stage.label.startsWith('A1_'))?.value ?? 0;
  const a2 = leadStages.find((stage) => stage.label.startsWith('A2_'))?.value ?? 0;
  const appStatuses = filtered.appRank.reduce<Record<InsightStatus, number>>((result, row) => {
    result[appInsightStatus(row)] += 1;
    return result;
  }, { healthy: 0, watch: 0, problem: 0, insufficient: 0, unclassified: 0 });
  const allApps = new Set(filtered.appRank.map((row) => textFromHint(row, 'app_name', row.label))).size;
  const issueApps = new Set(filtered.appRank
    .filter((row) => ['watch', 'problem'].includes(appInsightStatus(row)))
    .map((row) => textFromHint(row, 'app_name', row.label))).size;
  const severeHotspots = new Set(filtered.networkHotspots
    .filter((row) => textFromHint(row, 'bottleneck') === 'NETWORK_SIDE_SEVERE')
    .map((row) => {
      const detail = parseMetricHint(row.hint);
      return [detail.bras, detail.olt, detail.pon].join('|');
    })).size;
  const overviewKpis = [
    { label: '接入分类观测覆盖率', value: `${coverage.toFixed(1)}%`, hint: '已识别 Cable/FTTH 的用户小时观测 / 全部用户小时观测', tone: coverage < 90 ? 'warning' : 'normal' },
    { label: '全部 App', value: String(allApps), hint: `当前已加载 ${filtered.appRank.length} 个 App × 接入组合，问题项在完整分布中高亮` },
    { label: '规则标记 App', value: String(issueApps), hint: '本次策略标记为 Attention 或 Severe 的真实 App', tone: issueApps ? 'warning' : 'normal' },
    { label: '网络侧可疑聚集', value: String(severeHotspots), hint: '主问题侧偏网络且存在可识别网络对象；仍需进一步验证', tone: severeHotspots ? 'danger' : 'normal' },
    { label: 'A1 候选', value: String(a1), hint: '仍需 CRM、覆盖和可触达资格校验' },
    { label: 'A2 先修障', value: String(a2), hint: '网络严重异常，禁止直接营销', tone: a2 ? 'warning' : 'normal' },
  ];
  const appKpis = [
    { label: '全部 App', value: String(allApps), hint: `${filtered.appRank.length} 个 App × 接入类型组合` },
    { label: '正常组合', value: String(appStatuses.healthy), hint: '样本充足，当前策略 attention_level=NORMAL' },
    { label: '关注组合', value: String(appStatuses.watch), hint: '当前策略 attention_level=ATTENTION', tone: appStatuses.watch ? 'warning' : 'normal' },
    { label: '问题组合', value: String(appStatuses.problem), hint: '当前策略 attention_level=SEVERE', tone: appStatuses.problem ? 'danger' : 'normal' },
    { label: '样本不足', value: String(appStatuses.insufficient), hint: '不进入问题排名，但保留在完整视图中' },
    { label: '未分级', value: String(appStatuses.unclassified), hint: '旧聚合或无 V2 状态；不可按正常解释', tone: appStatuses.unclassified ? 'warning' : 'normal' },
  ];
  const copy = pageCopy[activeView];
  const selectEvidence = (row: MetricCard) => {
    const detail = parseMetricHint(row.hint);
    const usable = (value?: string) => value && !['UNKNOWN', 'UNAVAILABLE', 'ALL', ''].includes(value.toUpperCase()) ? value : undefined;
    c.applyAnalysisContext({
      app_category: usable(detail.app_category),
      app_name: usable(detail.app_name),
      access_type: usable(detail.user_type ?? detail.access_type),
      user_key: usable(detail.user_key),
      bras: usable(detail.bras),
      network_object: usable([usable(detail.bras), usable(detail.olt), usable(detail.pon)].filter(Boolean).join(' / ')),
      issue_metric: usable(detail.issue_driver ?? detail.bottleneck),
      issue_side: usable(detail.issue_side ?? detail.bottleneck),
      hour_from: detail.hour_of_day === undefined ? undefined : Number(detail.hour_of_day),
      hour_to: detail.hour_of_day === undefined ? undefined : Number(detail.hour_of_day),
    });
    setSelectedEvidence(row);
  };

  const running = task.status === 'running' || task.status === 'stopping';
  const progress = task.total > 0 ? Math.round(task.completed / task.total * 100) : 0;
  const exportRunning = exportTask.status === 'running' || exportTask.status === 'stopping';
  const exportProgress = exportTask.total > 0 ? Math.round(exportTask.completed / exportTask.total * 100) : 0;
  const loadButtonLabel = task.status === 'partial' || task.status === 'failure' || task.status === 'stopped'
    ? '重试当前看板'
    : viewLoaded ? '重新加载当前看板' : '加载当前看板';

  return <section className="analytics-dashboard analytics-dashboard-v3">
    <header className="workspace-page-header analytics-page-header"><div><p className="eyebrow">{copy.eyebrow}</p><h2>{copy.title}</h2><p>{copy.description}</p></div><div className="analytics-header-actions">
      <button type="button" disabled={disabled || actionBusy || running || exportRunning} onClick={preparePdfReport}>{exportRunning ? `准备 PDF ${exportProgress}%` : '导出全部图表 PDF'}</button>
      {running
        ? <button type="button" className="danger-button" onClick={stopLoading} disabled={task.status === 'stopping'}>{task.status === 'stopping' ? '正在停止…' : '停止后续加载'}</button>
        : <button type="button" className="primary-button" disabled={disabled || actionBusy || exportRunning} onClick={loadCurrentView}>{loadButtonLabel}</button>}
    </div></header>
    <section className={`analytics-task-card task-${task.status}`} aria-live="polite">
      <div className="analytics-task-head"><div><span>按需分析任务</span><strong>{task.message}</strong></div><span className="analytics-task-status">{task.status.toUpperCase()}</span></div>
      <div className="analytics-task-progress"><span style={{ width: `${progress}%` }} /></div>
      <div className="analytics-task-plan">{viewDatasets[activeView].map((key, index) => {
        const failed = failedDatasetKeys.includes(key);
        const empty = emptyDatasetKeys.includes(key);
        const className = failed ? 'is-failed' : empty ? 'is-empty' : index < task.completed ? 'is-complete' : task.current === key && running ? 'is-running' : '';
        const count = datasetCounts[key];
        return <span key={key} className={className}>{failed ? '!' : empty ? '○' : index < task.completed ? '✓' : index + 1} {datasetLabels[key]}{count !== undefined ? ` · ${count} rows${empty && count > 0 ? ' / all 0' : ''}` : ''}</span>;
      })}</div>
      <small>切换页面不会自动发起查询；停止操作会等待当前数据库请求结束，再跳过剩余步骤。</small>
    </section>
    <section className={`analytics-task-card analytics-export-task task-${exportTask.status}`} aria-live="polite">
      <div className="analytics-task-head"><div><span>全部图表 PDF</span><strong>{exportTask.message}</strong></div><span className="analytics-task-status">{exportTask.status.toUpperCase()}</span></div>
      <div className="analytics-task-progress"><span style={{ width: `${exportProgress}%` }} /></div>
      <div className="analytics-task-plan">{allDatasetKeys.map((key, index) => {
        const failed = exportFailedDatasetKeys.includes(key);
        const className = failed ? 'is-failed' : index < exportTask.completed ? 'is-complete' : exportTask.current === key && exportRunning ? 'is-running' : '';
        return <span key={key} className={className}>{failed ? '!' : index < exportTask.completed ? '✓' : index + 1} {datasetLabels[key]}</span>;
      })}</div>
      {exportFailures.length > 0 && <div className="analytics-export-failures">{exportFailures.map((failure, index) => <span key={`${index}-${failure}`}>{failure}</span>)}</div>}
      <div className="analytics-export-task-actions">
        <small>这是独立的显式任务：锁定当前批次、analysis_run_id 与筛选条件，顺序查询聚合数据；不会导出明细表。</small>
        {exportRunning && <button type="button" className="danger-button" onClick={stopPdfPreparation} disabled={exportTask.status === 'stopping'}>{exportTask.status === 'stopping' ? '正在停止…' : '停止后续准备'}</button>}
        {!exportRunning && pdfReport && <button type="button" onClick={() => setPdfPreviewOpen(true)}>再次打开报告预览</button>}
      </div>
    </section>
    {task.status === 'empty' && <section className="analytics-empty-result-banner">
      <div>
        <strong>RAW 数据可能已经导入，但当前批次尚无可用 CLEAN/DWS/ADS 结果</strong>
        <span>RAW {batchContext?.status ?? 'unknown'} · Pipeline {batchContext?.pipeline_status ?? 'unknown'} · analysis_run_id {c.analysisRunId || 'missing'}</span>
        <small>{String(batchContext?.pipeline_status ?? '').toLowerCase() === 'failed' ? batchContext?.pipeline_message ?? '自动分析流水线失败。' : '请回到数据导入页恢复该批次的流水线日志，确认 Quality Gate、CLEAN/DWD 和 DWS/ADS 是否完成。'}</small>
      </div>
      <button type="button" onClick={onOpenImport}>查看该批次导入状态与日志</button>
    </section>}
    {!viewLoaded && !running && task.status !== 'stopped' && task.status !== 'empty' && <section className="analytics-load-gate"><div><p className="eyebrow">Ready on demand</p><h3>当前看板尚未加载</h3><p>{actionBusy ? '上一项操作仍在完成，结束后即可启动本页任务。' : `本页需要 ${viewDatasets[activeView].length} 个聚合数据集。只有点击按钮后才会查询 MySQL，不会在应用启动或切换页面时自动执行。`}</p></div><button type="button" className="primary-button" disabled={disabled || actionBusy} onClick={loadCurrentView}>开始加载 {copy.title}</button></section>}
    {(viewLoaded || running || task.status === 'partial' || task.status === 'stopped' || task.status === 'empty') && <>
    <section className="analytics-filter-bar" aria-label="分析筛选">
      <label>接入类型<select value={access} onChange={(event) => setAccess(event.target.value)}><option value="ALL">全部</option><option value="CABLE">Cable</option><option value="FTTH">FTTH</option><option value="OTHER">Other</option></select></label>
      <label>搜索<input value={keyword} onChange={(event) => setKeyword(event.target.value)} placeholder="App、用户、BRAS、OLT、PON" /></label>
      <label>最小用户数<input type="number" min={0} value={minUsers} onChange={(event) => setMinUsers(Math.max(0, Number(event.target.value)))} /></label>
      <div className="filter-context"><span>Batch</span><strong>{c.batchDisplayName || c.importBatchId || '-'}</strong><small>{task.message}</small></div>
    </section>
    {failures.length > 0 && <section className="analytics-error-banner"><strong>部分数据集加载失败</strong>{failures.map((failure) => <span key={failure}>{failure}</span>)}</section>}
    {data.coverage.length > 0 && <section className="analytics-readiness-grid" aria-label="数据可用性说明">
      <article className={classificationStale ? 'is-warning' : 'is-ready'}><span>接入分类口径</span><strong>Others（未命中显式 IP 规则）→ {accessDefault}</strong><small>{classificationStale ? `现有聚合仍包含 ${unknownHourlyRows} 条 UNKNOWN 小时记录，说明结果由旧规则生成；请在当前导入任务完成后按已发布规则重新生成 CLEAN/DWS/ADS。` : '显式 IP 网段优先；其余合法 IP 全部使用该规则版本中明确配置的 Others 映射。CSV 制式字段只保留为来源证据。'}</small></article>
      <article className={gameImported ? 'is-ready' : 'is-info'}><span>游戏数据覆盖</span><strong>{gameImported ? '已导入' : '本批次未导入'}</strong><small>{gameImported ? '游戏时长与 MOS 可用于本次分析。' : '游戏来自独立文件；本次不展示游戏时长/MOS，也不把缺失解释为 0。'}</small></article>
      {(activeView === 'quality' || activeView === 'overview') && <article className={topologyKnownRows > 0 ? 'is-warning' : 'is-info'}><span>网络拓扑覆盖</span><strong>{topologyKnownRows} 个含已知拓扑的聚合节点</strong><small>当前源数据 OLT/PON 缺失时，只能做问题侧与 BRAS 粒度判断，不能下钻到 OLT/PON。</small></article>}
    </section>}
    <KpiStrip items={activeView === 'apps' ? appKpis : overviewKpis} />

    {activeView === 'overview' && <div className="analytics-layout">
      <AnalyticsChart title="全部 App 体验状态构成" subtitle="完整 App × 接入组合；问题与关注项高亮，正常和样本不足项均保留" explanationId="app_portfolio_status" kind="donut" points={appStatusPoints(filtered.appRank)} onSelect={selectEvidence} />
      <AnalyticsChart title="App 用户覆盖全景" subtitle="按合格用户数展示业务覆盖，颜色代表策略状态" explanationId="app_population" kind="bar" points={appPopulationPoints(filtered.appRank, 'eligible_users')} onSelect={selectEvidence} />
      <AnalyticsChart title="App 业务规模与状态" subtitle="按 TCP 下载流量展示主要 App，问题状态不等同于流量高低" explanationId="app_tcp_traffic" kind="bar" points={appPopulationPoints(filtered.appRank, 'traffic_gb')} onSelect={selectEvidence} />
      <AnalyticsChart title="高亮问题 App 持续差体验用户" subtitle="从全量组合中突出满足持续性与最低样本规则的影响用户" explanationId="app_affected_users" kind="bar" points={appPoints(filtered.appRank, 'persistent_poor_users')} onSelect={selectEvidence} />
      <AnalyticsChart title="全量用户需求分层" subtitle="完整人群的需求结构；高需求不等于体验差或可营销" explanationId="user_demand_band" kind="bar" points={cohortPoints(filtered.userSummary, 'demand_band')} onSelect={selectEvidence} />
      <AnalyticsChart title="网络 / 路径可疑聚集率" subtitle="只有真实网络对象和足够样本才可进一步确认热点" explanationId="topology_poor_user_rate" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'severity')} onSelect={selectEvidence} />
      <AnalyticsChart title="机会与排除分层" subtitle="按用户计数；A0/A2 不得进入直接营销" explanationId="lead_stage" kind="bar" points={leadStages} onSelect={selectEvidence} />
      <AnalyticsChart title="典型日接入类型速率" subtitle="按活跃用户加权的 7 日小时均值，Mbps" explanationId="typical_effective_rate" kind="line" points={typicalHourlyPoints(filtered.hourlyTrend, 'effective_mbps')} onSelect={selectEvidence} />
      <AppPortfolioTable rows={filtered.appRank} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="总览指标与来源" rows={data.kpis} />
    </div>}

    {activeView === 'apps' && <div className="analytics-layout">
      <AnalyticsChart title="全部 App 体验状态构成" subtitle="完整 App × 接入组合；按已绑定策略分为正常、关注、问题、样本不足和未分级" explanationId="app_portfolio_status" kind="donut" points={appStatusPoints(filtered.appRank)} onSelect={selectEvidence} />
      <AnalyticsChart title="App 用户覆盖全景" subtitle="按合格用户规模展示全部组合，颜色用于高亮体验状态" explanationId="app_population" kind="bar" points={appPopulationPoints(filtered.appRank, 'eligible_users')} onSelect={selectEvidence} />
      <AnalyticsChart title="App TCP 下载流量与体验状态" subtitle="业务规模与体验状态放在同一视图；流量高不等于体验差" explanationId="app_tcp_traffic" kind="bar" points={appPopulationPoints(filtered.appRank, 'traffic_gb')} onSelect={selectEvidence} />
      <AnalyticsChart title="高亮：持续差体验用户" subtitle="仅在全量视图中突出满足持续性与最低样本规则的唯一用户数" explanationId="app_affected_users" kind="bar" points={appPoints(filtered.appRank, 'persistent_poor_users')} onSelect={selectEvidence} />
      <AnalyticsChart title="App 持续差体验用户占比" subtitle="持续差体验用户 / 合格用户，单位 %" explanationId="app_affected_user_rate" kind="bar" points={appPoints(filtered.appRank, 'persistent_poor_user_rate_pct')} onSelect={selectEvidence} />
      <AnalyticsChart title="App 差体验观测占比" subtitle="差体验观测 / 有效观测，单位 %" explanationId="app_poor_observation_rate" kind="bar" points={appPoints(filtered.appRank, 'poor_observation_rate_pct')} onSelect={selectEvidence} />
      <AnalyticsChart title="App 曾受影响用户占比" subtitle="至少异常一次的合格用户 / 合格用户，单位 %" explanationId="app_ever_affected_user_rate" kind="bar" points={appPoints(filtered.appRank, 'ever_affected_user_rate_pct')} onSelect={selectEvidence} />
      <AnalyticsChart title="App 严重差体验用户占比" subtitle="严重差体验用户 / 合格用户，单位 %" explanationId="app_severe_user_rate" kind="bar" points={appPoints(filtered.appRank, 'severe_poor_user_rate_pct')} onSelect={selectEvidence} />
      <AppPortfolioTable rows={filtered.appRank} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="应用体验原始聚合证据" rows={filtered.appRank} limit={500} />
    </div>}

    {activeView === 'quality' && <div className="analytics-layout">
      <AnalyticsChart title="可识别网络对象受影响用户" subtitle="只统计真实 BRAS / OLT / PON；缺失值不作为网络对象" explanationId="topology_affected_users" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'users')} onSelect={selectEvidence} />
      <AnalyticsChart title="网络 / 路径可疑聚集率" subtitle="差体验用户 / 对象观测用户；当前证据不等同于已确认根因" explanationId="topology_poor_user_rate" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'severity')} onSelect={selectEvidence} />
      <AnalyticsChart title="家庭侧 / Wi-Fi RTT" subtitle="节点平均 subscriber-side RTT，单位 ms" explanationId="subscriber_side_rtt" kind="bar" points={hotspotPoints(filtered.networkHotspots, 'subscriber_rtt_ms')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="网络 / 路径证据（非已确认根因）" rows={filtered.networkHotspots} limit={240} />
    </div>}

    {activeView === 'cable' && <div className="analytics-layout">
      <AnalyticsChart title="Cable / FTTH 典型日有效速率" subtitle="按活跃用户加权的 7 日小时均值，Mbps" explanationId="typical_effective_rate" kind="line" points={typicalHourlyPoints(filtered.hourlyTrend, 'effective_mbps')} onSelect={selectEvidence} />
      <AnalyticsChart title="Cable / FTTH 典型日 RTT" subtitle="按活跃用户加权的 subscriber-side RTT，ms" explanationId="typical_subscriber_rtt" kind="line" points={typicalHourlyPoints(filtered.hourlyTrend, 'subscriber_rtt_ms')} onSelect={selectEvidence} />
      <AnalyticsChart title="Cable / FTTH 典型日用户侧丢包" subtitle="按活跃用户加权的 user-side downstream loss，%" explanationId="typical_user_loss" kind="line" points={typicalHourlyPoints(filtered.hourlyTrend, 'user_loss_pct')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="接入对比小时证据" rows={filtered.hourlyTrend} limit={300} />
    </div>}

    {activeView === 'users' && <div className="analytics-layout">
      <AnalyticsChart title="用户需求分层" subtitle="全量用户分群；评分用于发现需求，不等同于可营销资格" explanationId="user_demand_band" kind="bar" points={cohortPoints(filtered.userSummary, 'demand_band')} onSelect={selectEvidence} />
      <AnalyticsChart title="用户流量分层" subtitle="全量用户按分析周期 TCP 流量分群" explanationId="user_traffic_band" kind="bar" points={cohortPoints(filtered.userSummary, 'traffic_band')} onSelect={selectEvidence} />
      <AnalyticsChart title="用户问题侧分布" subtitle="全量用户按主要瓶颈侧分群；游戏数据缺失不会记为零时长" explanationId="user_issue_side" kind="bar" points={cohortPoints(filtered.userSummary, 'bottleneck_side')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="用户画像证据" rows={filtered.userProfiles} limit={300} />
    </div>}

    {activeView === 'leads' && <div className="analytics-layout">
      <AnalyticsChart title="机会与排除分层" subtitle="按唯一用户计数；A0 待外部资格数据、A2 先修障、A1 待资格校验" explanationId="lead_stage" kind="bar" points={leadStages} onSelect={selectEvidence} />
      <AnalyticsChart title="机会分层构成" subtitle="展示当前分析运行中的用户分层占比" explanationId="lead_stage_share" kind="donut" points={leadStages} onSelect={selectEvidence} />
      <AnalyticsChart title="候选用户需求评分" subtitle="评分用于排序；最终行动仍由问题侧与资格字段决定" explanationId="lead_demand_score" kind="bar" points={userPoints(filtered.leadEvidence, 'demand_score')} onSelect={selectEvidence} />
      <AnalyticsEvidenceTable title="体验驱动机会证据（非可直接营销名单）" rows={filtered.leadEvidence} limit={400} />
    </div>}
    </>}
    {selectedEvidence && <EvidenceDrawer row={selectedEvidence} onClose={() => setSelectedEvidence(null)} />}
    {pdfReport && pdfPreviewOpen && <PdfPreview report={pdfReport} onClose={() => setPdfPreviewOpen(false)} />}
  </section>;
}

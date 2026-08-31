import { useEffect, useMemo, useRef, useState } from 'react';
import type { DecisionRuleProfileRow, MetricCard, OpportunityCandidatePage, OpportunityCandidateRow } from '../../shared/types';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import { parseMetricHint } from './analyticsStructuredCharts';
import { selectCsvSavePath } from './fileDialogs';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';

export type DecisionView = 'panorama' | 'quality' | 'access' | 'opportunities';
type PanoramaPerspective = 'metric' | 'app';
type FullPrintReport = {
  metrics: MetricCard[];
  apps: MetricCard[];
  users: MetricCard[];
  quality: MetricCard[];
  access: MetricCard[];
  accessHourly: MetricCard[];
  accessBands: MetricCard[];
  opportunities: MetricCard[];
};

const copy = {
  panorama: ['全景洞察', 'Panorama'],
  quality: ['质差分析', 'Poor-quality analysis'],
  access: ['Cable / FTTH 专项', 'Cable / FTTH analysis'],
  opportunities: ['潜客机会', 'Opportunities'],
} as const;

function friendlyNumber(value: string, unit?: string) {
  if (value === 'NA' || value === '—') return '—';
  const number = Number(value);
  if (!Number.isFinite(number)) return value;
  if (unit === 'GB') {
    if (number >= 1000) return `${(number / 1000).toFixed(2)} TB`;
    return `${number.toFixed(2)} GB`;
  }
  if (unit === 'percent') return `${number.toFixed(2)}%`;
  if (unit) return `${number.toLocaleString(undefined, { maximumFractionDigits: 2 })} ${unit}`;
  return number.toLocaleString(undefined, { maximumFractionDigits: 2 });
}

function CardGrid({ rows }: { rows: MetricCard[] }) {
  return <div className="decision-card-grid">{rows.map((row) => {
    const detail = parseMetricHint(row.hint);
    const unit = detail.unit;
    const unavailable = detail.availability === 'UNAVAILABLE';
    return <article className={`decision-metric-card ${unavailable ? 'is-unavailable' : ''}`} key={`${row.label}-${row.hint}`}>
      <span>{row.label}</span><strong>{friendlyNumber(row.value, unit)}</strong>
      <small>{unavailable ? `不可用：${detail.limitation || '当前数据不支持'}` : detail.denominator ? `${detail.numerator} / ${detail.denominator}；样本 ${detail.sample_size ?? detail.denominator}` : `样本 ${detail.sample_size ?? '—'}`}</small>
    </article>;
  })}</div>;
}

function Explanation({ children }: { children: React.ReactNode }) {
  return <p className="decision-explanation">{children}</p>;
}

function numeric(value: string | undefined) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function AppDecisionReason({ row, zh }: { row: MetricCard; zh: boolean }) {
  const d = parseMetricHint(row.hint);
  const status = d.insight_status ?? 'INSUFFICIENT';
  const poor = numeric(d.poor_observation_rate_pct);
  const persistent = numeric(d.persistent_poor_user_rate_pct);
  const severe = numeric(d.severe_poor_user_rate_pct);
  const thresholds = status === 'SEVERE'
    ? [numeric(d.severe_poor_rate_pct), numeric(d.severe_persistent_rate_pct), numeric(d.severe_user_rate_pct)]
    : status === 'PROBLEM'
      ? [numeric(d.problem_poor_rate_pct), numeric(d.problem_persistent_rate_pct), null]
      : [numeric(d.attention_poor_rate_pct), numeric(d.attention_persistent_rate_pct), null];
  const checks = [
    { label: zh ? '差体验观测占比' : 'Poor observation rate', actual: poor, threshold: thresholds[0], numerator: d.poor_obs_rows, denominator: d.valid_obs_rows },
    { label: zh ? '持续质差用户占比' : 'Persistent poor users', actual: persistent, threshold: thresholds[1], numerator: d.persistent_poor_users, denominator: d.eligible_users },
    ...(thresholds[2] == null ? [] : [{ label: zh ? '严重质差用户占比' : 'Severe poor users', actual: severe, threshold: thresholds[2], numerator: d.severe_poor_users, denominator: d.eligible_users }]),
  ];
  const statusLabel = status === 'SEVERE' ? (zh ? '严重' : 'Severe') : status === 'PROBLEM' ? (zh ? '问题' : 'Problem') : status === 'WATCH' ? (zh ? '关注' : 'Watch') : status === 'NORMAL' ? (zh ? '正常' : 'Normal') : status === 'LIMITED' ? (zh ? '有限样本' : 'Limited sample') : (zh ? '样本不足' : 'Insufficient sample');
  return <article className="app-decision-reason">
    <header><div><span>{zh ? '判定结论' : 'Decision'}</span><strong>{statusLabel}</strong></div><small>{zh ? '样本状态' : 'Sample'}：{d.sample_status ?? '—'} · {zh ? '规则' : 'Rule'} v{d.rule_version ?? '—'}</small></header>
    {['INSUFFICIENT', 'LIMITED'].includes(status) ? <p>{zh ? `有效用户 ${Number(d.eligible_users ?? 0).toLocaleString()}、有效观测 ${Number(d.valid_obs_rows ?? 0).toLocaleString()}，尚不足以稳定判断该 App 是否存在普遍问题。` : `Eligible users ${Number(d.eligible_users ?? 0).toLocaleString()} and valid observations ${Number(d.valid_obs_rows ?? 0).toLocaleString()} are not sufficient for a stable App-level conclusion.`}</p> : <div className="decision-check-grid">{checks.map((check) => { const hit = check.actual != null && check.threshold != null && check.actual >= check.threshold; return <div key={check.label} className={hit ? 'is-hit' : ''}><span>{check.label}</span><strong>{check.actual == null ? '—' : `${check.actual.toFixed(2)}%`} {check.threshold == null ? '' : `${hit ? '≥' : '<'} ${check.threshold.toFixed(2)}%`}</strong><small>{check.numerator ?? '—'} / {check.denominator ?? '—'}</small></div>; })}</div>}
    <p>{zh ? '这是“该 App 用户出现体验问题”的证据，不等同于已经证明 App 服务端是根因。' : 'This is evidence of poor experience among users of this App; it does not prove the App server is the root cause.'}</p>
  </article>;
}

function bandOrder(row: MetricCard) {
  const explicit = numeric(parseMetricHint(row.hint).band_order);
  if (explicit != null) return explicit;
  const label = row.label.toLowerCase();
  if (label.includes('不可用') || label.includes('unavailable')) return 99;
  if (label.startsWith('0 ') || label === '0%') return 0;
  if (label.startsWith('<')) return 1;
  if (label.startsWith('0–') || label.startsWith('0-')) return 1;
  const first = Number(label.match(/[0-9]+(?:\.[0-9]+)?/)?.[0]);
  return Number.isFinite(first) ? first + 2 : 50;
}

function AppTable({ rows, c }: { rows: MetricCard[]; c: WorkbenchController }) {
  const [selected, setSelected] = useState<MetricCard | null>(null);
  const [detailRows, setDetailRows] = useState<MetricCard[]>([]);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState('');
  const detailRequest = useRef(0);
  const zh = c.language === 'zh-CN';

  useEffect(() => {
    if (!selected) return undefined;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') closeDetail();
    };
    document.body.classList.add('app-detail-modal-open');
    window.addEventListener('keydown', closeOnEscape);
    return () => {
      document.body.classList.remove('app-detail-modal-open');
      window.removeEventListener('keydown', closeOnEscape);
    };
  }, [selected]);

  function closeDetail() {
    detailRequest.current += 1;
    setSelected(null);
    setDetailRows([]);
    setDetailError('');
    setDetailLoading(false);
  }

  async function openDetail(row: MetricCard) {
    const request = detailRequest.current + 1;
    detailRequest.current = request;
    setSelected(row);
    setDetailRows([]);
    setDetailError('');
    setDetailLoading(true);
    c.applyAnalysisContext({ app_name: row.label });
    try {
      const result = await analyticsStructuredApi.decisionUserDistributions(c.effectiveSettings, c.importBatchId, c.analysisRunId, { keyword: row.label });
      if (detailRequest.current === request) setDetailRows(result);
    } catch (error) {
      if (detailRequest.current === request) setDetailError(error instanceof Error ? error.message : String(error));
    } finally {
      if (detailRequest.current === request) setDetailLoading(false);
    }
  }

  return <>
    <div className="decision-table-wrap"><table className="decision-table"><thead><tr><th>App</th><th>{zh ? '用户' : 'Users'}</th><th>{zh ? '流量' : 'Traffic'}</th><th>{zh ? '有效时长' : 'Effective duration'}</th><th>{zh ? '有效观测' : 'Valid obs'}</th><th>{zh ? '差观测' : 'Poor obs'}</th><th>{zh ? '持续质差用户' : 'Persistent users'}</th><th>{zh ? '状态' : 'Status'}</th></tr></thead>
      <tbody>{rows.map((row) => { const d = parseMetricHint(row.hint); const status = d.insight_status ?? 'UNCLASSIFIED'; const statusLabel = status === 'SEVERE' ? (zh ? '严重' : 'Severe') : status === 'PROBLEM' ? (zh ? '问题' : 'Problem') : status === 'WATCH' ? (zh ? '关注' : 'Watch') : status === 'NORMAL' ? (zh ? '正常' : 'Normal') : status === 'LIMITED' ? (zh ? '有限样本' : 'Limited') : (zh ? '样本不足' : 'Insufficient'); return <tr key={row.label} className={`insight-${status.toLowerCase()}`} role="button" tabIndex={0} onClick={() => void openDetail(row)} onKeyDown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); void openDetail(row); } }}><td><strong>{row.label}</strong><small>{d.app_category}</small></td><td>{Number(d.observed_users ?? row.value).toLocaleString()}</td><td>{friendlyNumber(d.traffic_gb ?? 'NA', 'GB')}</td><td>{friendlyNumber(d.duration_hours ?? 'NA', 'h')}</td><td>{Number(d.valid_obs_rows ?? 0).toLocaleString()}</td><td>{d.poor_observation_rate_pct === 'NA' ? '—' : `${Number(d.poor_observation_rate_pct).toFixed(2)}%`}</td><td>{d.persistent_poor_user_rate_pct === 'NA' ? '—' : `${Number(d.persistent_poor_user_rate_pct).toFixed(2)}%`}</td><td><span className={`insight-badge is-${status.toLowerCase()}`}>{statusLabel}</span></td></tr>; })}</tbody>
    </table></div>
    {selected && (() => { const d = parseMetricHint(selected.hint); return <div className="app-detail-modal-backdrop" role="presentation" onMouseDown={closeDetail}><section className="app-detail-modal" role="dialog" aria-modal="true" aria-labelledby="app-detail-title" aria-busy={detailLoading} onMouseDown={(event) => event.stopPropagation()}><header className="app-detail-modal-head"><div><span>{zh ? 'App 详情' : 'App detail'}</span><h2 id="app-detail-title">{selected.label}</h2><p>{zh ? '判定依据 → 总体指标 → 用户分布' : 'Decision evidence → overall metrics → user distributions'}</p></div><button type="button" autoFocus onClick={closeDetail}>{zh ? '关闭' : 'Close'}</button></header><div className="app-detail-modal-body"><AppDecisionReason row={selected} zh={zh}/><article className="selected-app-detail"><div><p>{zh ? '总体值用于描述受影响范围；下方分档继续定位问题集中在哪类用户。' : 'Overall values describe the affected scope; distributions locate which user cohorts concentrate the issue.'}</p></div><div className="detail-stat-grid"><span>{zh ? '用户' : 'Users'}<strong>{d.observed_users}</strong></span><span>{zh ? '流量' : 'Traffic'}<strong>{friendlyNumber(d.traffic_gb ?? 'NA', 'GB')}</strong></span><span>{zh ? '有效速率' : 'Effective rate'}<strong>{friendlyNumber(d.effective_download_mbps ?? 'NA', 'Mbps')}</strong></span><span>{zh ? '用户侧 RTT' : 'Subscriber RTT'}<strong>{friendlyNumber(d.subscriber_rtt_ms ?? 'NA', 'ms')}</strong></span><span>{zh ? '网络侧 RTT' : 'Network RTT'}<strong>{friendlyNumber(d.network_rtt_ms ?? 'NA', 'ms')}</strong></span><span>{zh ? '规则版本' : 'Rule version'}<strong>v{d.rule_version}</strong></span></div></article>{detailLoading ? <div className="app-detail-state is-loading"><strong>{zh ? '正在加载该 App 的用户分布…' : 'Loading user distributions…'}</strong><span>{zh ? '弹框会在结果返回后自动更新。' : 'This dialog will update when the result is ready.'}</span></div> : detailError ? <div className="app-detail-state is-error"><strong>{zh ? '详情加载失败' : 'Failed to load detail'}</strong><span>{detailError}</span><button type="button" onClick={() => void openDetail(selected)}>{zh ? '重试' : 'Retry'}</button></div> : detailRows.length > 0 ? <Distribution rows={detailRows} /> : <div className="app-detail-state"><strong>{zh ? '当前 App 没有可用的用户分布' : 'No user distribution is available'}</strong><span>{zh ? '可能是样本不足或对应聚合尚未生成。' : 'The sample may be insufficient or the aggregate may not be ready.'}</span></div>}</div></section></div>; })()}
  </>;
}

function Distribution({ rows }: { rows: MetricCard[] }) {
  const groups = useMemo(() => rows.reduce<Record<string, MetricCard[]>>((all, row) => { const key = parseMetricHint(row.hint).dimension ?? 'OTHER'; (all[key] ??= []).push(row); return all; }, {}), [rows]);
  const title: Record<string, string> = { TRAFFIC_DAILY: '日均流量分档用户分布', DURATION_EFFECTIVE_DAILY: '日均有效业务时长分档', DURATION_PEAK_DAILY: '日均高峰期有效时长分档', DURATION_GAME_ONLY: '独立 Game 时长分档', OBSERVATIONS_DAILY: '日均视频观测记录分档', EFFECTIVE_RATE: '视频有效下载速率分档', AVERAGE_DOWNLOAD_RATE: '平均下载速率分档', SUBSCRIBER_RTT: '用户侧时延分档', NETWORK_RTT: '网络侧时延分档', USER_LOSS: '用户侧丢包分档', NETWORK_LOSS: '网络侧丢包分档', EXPERIENCE: '持续质差用户分布' };
  return <div className="distribution-grid">{Object.entries(groups).map(([dimension, items]) => { const ordered = [...items].sort((left, right) => bandOrder(left) - bandOrder(right)); const total = ordered.reduce((sum, item) => sum + (numeric(item.value) ?? 0), 0); return <article className="distribution-card" key={dimension}><h3>{title[dimension] ?? dimension}</h3><small className="distribution-denominator">总计 {total.toLocaleString()} 名分析用户（按 IP 去重）</small>{ordered.map((item) => { const users = numeric(item.value) ?? 0; const share = total > 0 ? users / total * 100 : 0; return <div className="distribution-row" key={item.label}><span>{item.label}</span><div aria-label={`${item.label} ${share.toFixed(1)}%`}><i style={{ width: `${share}%` }} /></div><strong>{users.toLocaleString()}<small>{share.toFixed(1)}%</small></strong></div>; })}<Explanation>柱长表示当前分档占该指标全部分析用户的比例；人数和占比同时保留。不可用单独成档，不会按 0 纳入其他区间。</Explanation></article>; })}</div>;
}

function PrintAppBars({ rows }: { rows: MetricCard[] }) {
  const printable = rows.slice(0, 40);
  const max = Math.max(1, ...printable.map((row) => Number(parseMetricHint(row.hint).observed_users ?? row.value)));
  return <div className="print-app-bars">{printable.map((row) => { const detail = parseMetricHint(row.hint); const value = Number(detail.observed_users ?? row.value); const status = detail.insight_status ?? 'INSUFFICIENT'; return <div className={`distribution-row is-${status.toLowerCase()}`} key={row.label}><span>{row.label}<small>{status}</small></span><div><i style={{ width: `${Math.max(2, value / max * 100)}%` }} /></div><strong>{value.toLocaleString()} 人</strong></div>; })}</div>;
}

function AppStatusSummary({ rows, zh }: { rows: MetricCard[]; zh: boolean }) {
  const order = ['SEVERE', 'PROBLEM', 'WATCH', 'NORMAL', 'LIMITED', 'INSUFFICIENT'];
  const labels: Record<string, [string, string]> = {
    SEVERE: ['严重', 'Severe'], PROBLEM: ['问题', 'Problem'], WATCH: ['关注', 'Watch'],
    NORMAL: ['正常', 'Normal'], LIMITED: ['有限样本', 'Limited sample'], INSUFFICIENT: ['样本不足', 'Insufficient'],
  };
  const counts = rows.reduce<Record<string, number>>((all, row) => { const status = parseMetricHint(row.hint).insight_status ?? 'INSUFFICIENT'; all[status] = (all[status] ?? 0) + 1; return all; }, {});
  return <div className="app-status-summary"><article><span>{zh ? '全部唯一 App' : 'Unique Apps'}</span><strong>{rows.length}</strong></article>{order.map((status) => <article key={status} className={`is-${status.toLowerCase()}`}><span>{labels[status][zh ? 0 : 1]}</span><strong>{counts[status] ?? 0}</strong></article>)}</div>;
}

const opportunityNames: Record<string, [string, string]> = {
  MIGRATION: ['Cable → FTTH 迁转', 'Cable → FTTH migration'],
  SPEED_UPGRADE: ['宽带升套', 'Speed upgrade'],
  MESH_AP: ['AP / Mesh 组网', 'AP / Mesh'],
  APP_BUNDLE: ['特定 App Bundle', 'App bundle'],
};

type OpportunityExplanation = { sentence: string; priority: string; checks: Array<{ label: string; actual: string; rule: string; hit: boolean }> };

function opportunityExplanation(row: OpportunityCandidateRow, rule: DecisionRuleProfileRow | null, zh: boolean): OpportunityExplanation {
  const days = Math.max(row.active_days, 1);
  const dailyTraffic = row.total_download_gb / days;
  const dailyDuration = row.total_effective_duration_hours / days;
  const dailyObservations = row.observation_rows / days;
  const appDays = Math.max(row.primary_app_active_days, 1);
  const dailyAppObservations = row.primary_app_observations / appDays;
  if (!rule) return {
    sentence: row.evidence_summary,
    priority: zh ? `当前优先级为${row.opportunity_level === 'HIGH' ? '高' : '标准'}；规则详情加载后可查看逐项门槛。当前模型没有综合分数。` : `Current priority is ${row.opportunity_level}; rule thresholds are not loaded. The current model has no composite score.`,
    checks: [],
  };
  const baseChecks = [
    { label: zh ? '活跃天数' : 'Active days', actual: `${row.active_days}`, rule: `≥ ${rule.opportunity_min_active_days}`, hit: row.active_days >= rule.opportunity_min_active_days },
    { label: zh ? '有效观测' : 'Observations', actual: row.observation_rows.toLocaleString(), rule: `≥ ${rule.opportunity_min_observations.toLocaleString()}`, hit: row.observation_rows >= rule.opportunity_min_observations },
  ];
  if (row.opportunity_type === 'MIGRATION') {
    const checks = [...baseChecks,
      { label: zh ? '日均流量' : 'Daily traffic', actual: `${dailyTraffic.toFixed(2)} GB`, rule: `≥ ${rule.migration_min_traffic_gb} GB`, hit: dailyTraffic >= rule.migration_min_traffic_gb },
      { label: zh ? '日均有效时长' : 'Daily duration', actual: `${dailyDuration.toFixed(2)} h`, rule: `≥ ${rule.heavy_usage_hours} h`, hit: dailyDuration >= rule.heavy_usage_hours },
      { label: zh ? '日均观测' : 'Daily observations', actual: dailyObservations.toFixed(1), rule: `≥ ${rule.app_bundle_min_observations}`, hit: dailyObservations >= rule.app_bundle_min_observations },
    ];
    return { sentence: zh ? `该 Cable 用户满足基础样本门槛，并命中流量、有效时长或观测频度中的至少一项，因此进入 FTTH 迁转候选。该判断表示高使用需求，不等同于已证明迁转后体验一定改善。` : `This Cable user meets the base sample gate and at least one traffic, duration or observation-frequency condition. This indicates high usage demand, not proven post-migration improvement.`, priority: zh ? `高优先级仅由日均流量是否达到 ${rule.heavy_traffic_gb} GB 决定；当前为 ${dailyTraffic.toFixed(2)} GB，所以优先级为${row.opportunity_level === 'HIGH' ? '高' : '标准'}。当前没有综合分数。` : `High priority requires daily traffic ≥ ${rule.heavy_traffic_gb} GB. Current traffic is ${dailyTraffic.toFixed(2)} GB, so priority is ${row.opportunity_level}. There is no composite score.`, checks };
  }
  if (row.opportunity_type === 'SPEED_UPGRADE') {
    const checks = [...baseChecks,
      { label: zh ? '有效下载速率' : 'Effective rate', actual: row.avg_effective_download_mbps == null ? '—' : `${row.avg_effective_download_mbps.toFixed(2)} Mbps`, rule: `≤ ${rule.speed_upgrade_max_effective_mbps} Mbps`, hit: row.avg_effective_download_mbps != null && row.avg_effective_download_mbps <= rule.speed_upgrade_max_effective_mbps },
      { label: zh ? '日均流量条件' : 'Traffic condition', actual: `${dailyTraffic.toFixed(2)} GB`, rule: `≥ ${rule.speed_upgrade_min_traffic_gb} GB`, hit: dailyTraffic >= rule.speed_upgrade_min_traffic_gb },
      { label: zh ? '日均时长条件' : 'Duration condition', actual: `${dailyDuration.toFixed(2)} h`, rule: `≥ ${rule.heavy_usage_hours + 1} h`, hit: dailyDuration >= rule.heavy_usage_hours + 1 },
      { label: zh ? '日均观测条件' : 'Observation condition', actual: dailyObservations.toFixed(1), rule: `≥ ${rule.app_bundle_min_observations * 3}`, hit: dailyObservations >= rule.app_bundle_min_observations * 3 },
    ];
    return { sentence: zh ? `该用户有效下载速率低于门槛，同时至少命中 ${rule.speed_upgrade_min_conditions} 项需求条件，因此进入升档候选。低速也可能来自网络或家庭侧问题，正式行动前应先排除故障。` : `The effective rate is below the gate and at least ${rule.speed_upgrade_min_conditions} demand conditions are met. Low rate may also reflect network or home-side impairment, which must be ruled out before an upgrade action.`, priority: zh ? `高优先级仍由日均流量是否达到 ${rule.heavy_traffic_gb} GB 决定；当前没有综合分数。` : `High priority still depends on daily traffic ≥ ${rule.heavy_traffic_gb} GB; there is no composite score.`, checks };
  }
  if (row.opportunity_type === 'MESH_AP') {
    const rttDelta = row.avg_subscriber_rtt_ms != null && row.avg_network_rtt_ms != null ? row.avg_subscriber_rtt_ms - row.avg_network_rtt_ms : null;
    const lossDelta = row.avg_user_loss_pct != null && row.avg_network_loss_pct != null ? row.avg_user_loss_pct - row.avg_network_loss_pct : null;
    const checks = [...baseChecks,
      { label: zh ? '家庭侧证据覆盖门槛' : 'Home-side evidence coverage gate', actual: zh ? '发布时已通过' : 'Passed at publication', rule: `≥ ${rule.mesh_min_coverage_pct}%`, hit: true },
      { label: 'Wi-Fi Delay', actual: row.avg_wifi_delay_ms == null ? '—' : `${row.avg_wifi_delay_ms.toFixed(2)} ms`, rule: `≥ ${rule.mesh_min_wifi_delay_ms} ms`, hit: row.avg_wifi_delay_ms != null && row.avg_wifi_delay_ms >= rule.mesh_min_wifi_delay_ms },
      { label: zh ? '用户-网络 RTT 差' : 'Subscriber-network RTT delta', actual: rttDelta == null ? '—' : `${rttDelta.toFixed(2)} ms`, rule: `≥ ${rule.mesh_min_rtt_delta_ms} ms`, hit: rttDelta != null && rttDelta >= rule.mesh_min_rtt_delta_ms },
      { label: zh ? '用户-网络丢包差' : 'User-network loss delta', actual: lossDelta == null ? '—' : `${lossDelta.toFixed(2)} pct`, rule: `≥ ${rule.mesh_min_loss_delta_pct} pct`, hit: lossDelta != null && lossDelta >= rule.mesh_min_loss_delta_pct },
    ];
    return { sentence: zh ? '该用户满足基础样本门槛，并至少命中一个家庭侧/Wi-Fi 侧体验差异条件，因此进入 AP/Mesh 候选。' : 'The user meets the base sample gate and at least one home/Wi-Fi-side difference condition.', priority: zh ? '当前 Mesh 规则没有高优分层，所有命中用户均为标准优先级；当前没有综合分数。' : 'The current Mesh rule has no high-priority tier; all matches are Standard. There is no composite score.', checks };
  }
  const checks = [
    { label: zh ? '主 App' : 'Primary App', actual: row.primary_app ?? '—', rule: zh ? '必须可识别' : 'Required', hit: Boolean(row.primary_app) },
    { label: zh ? '主 App 活跃天数' : 'Primary App days', actual: `${row.primary_app_active_days}`, rule: `≥ ${rule.app_bundle_min_active_days}`, hit: row.primary_app_active_days >= rule.app_bundle_min_active_days },
    { label: zh ? '主 App 日均观测' : 'Daily primary-App obs', actual: dailyAppObservations.toFixed(1), rule: `≥ ${rule.app_bundle_min_observations}`, hit: dailyAppObservations >= rule.app_bundle_min_observations },
  ];
  return { sentence: zh ? `主 App 为 ${row.primary_app ?? '不可用'}，活跃天数和日均观测达到门槛，因此进入 App Bundle 兴趣候选；是否可售仍需产品目录和 CRM 校验。` : `Primary App ${row.primary_app ?? 'unavailable'} meets active-day and daily-observation gates, indicating bundle interest only; sellability still requires catalog and CRM checks.`, priority: zh ? `主 App 日均观测达到基础门槛 3 倍（${rule.app_bundle_min_observations * 3}）时为高优；当前没有综合分数。` : `High priority requires daily primary-App observations ≥ ${rule.app_bundle_min_observations * 3}; there is no composite score.`, checks };
}

function OpportunityPanel({ summaries, c }: { summaries: MetricCard[]; c: WorkbenchController }) {
  const zh = c.language === 'zh-CN';
  const [kind, setKind] = useState('');
  const [keyword, setKeyword] = useState('');
  const [queryKeyword, setQueryKeyword] = useState('');
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<OpportunityCandidatePage>({ rows: [], total: 0, page: 1, page_size: 50 });
  const [selected, setSelected] = useState<OpportunityCandidateRow | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [exporting, setExporting] = useState(false);
  const [exportStatus, setExportStatus] = useState('');
  const [rule, setRule] = useState<DecisionRuleProfileRow | null>(null);

  useEffect(() => {
    let active = true;
    const boundVersion = Number(parseMetricHint(summaries[0]?.hint ?? '').rule_version);
    workbenchApi.decisionRules(c.effectiveSettings).then((rules) => {
      if (active) setRule(rules.find((item) => item.version === boundVersion) ?? null);
    }).catch(() => { if (active) setRule(null); });
    return () => { active = false; };
  }, [c.effectiveSettings, summaries]);

  useEffect(() => {
    let active = true;
    setLoading(true); setError('');
    analyticsStructuredApi.decisionOpportunityCandidates(c.effectiveSettings, c.importBatchId, c.analysisRunId, { page, pageSize: 50, keyword: queryKeyword, opportunityType: kind || undefined })
      .then((value) => { if (active) setResult(value); })
      .catch((reason) => { if (active) setError(reason instanceof Error ? reason.message : String(reason)); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [c.effectiveSettings, c.importBatchId, c.analysisRunId, kind, page, queryKeyword]);

  useEffect(() => {
    if (!selected) return undefined;
    const close = (event: KeyboardEvent) => { if (event.key === 'Escape') setSelected(null); };
    document.body.classList.add('app-detail-modal-open');
    window.addEventListener('keydown', close);
    return () => { document.body.classList.remove('app-detail-modal-open'); window.removeEventListener('keydown', close); };
  }, [selected]);

  const pageCount = Math.max(1, Math.ceil(result.total / result.page_size));
  async function exportCandidates() {
    const filterName = kind ? kind.toLowerCase() : 'all';
    const date = new Date().toISOString().slice(0, 10).replace(/-/g, '');
    const outputPath = await selectCsvSavePath(`opportunity_candidates_${filterName}_${date}.csv`);
    if (!outputPath) return;
    setExporting(true); setExportStatus(zh ? '正在导出当前筛选下的全部潜客…' : 'Exporting all filtered candidates…');
    try {
      const result = await analyticsStructuredApi.decisionExportOpportunityCandidates(c.effectiveSettings, c.importBatchId, c.analysisRunId, outputPath, { keyword: queryKeyword, opportunityType: kind || undefined });
      setExportStatus(`${zh ? '导出完成' : 'Export complete'}：${result.message}`);
    } catch (reason) {
      setExportStatus(`${zh ? '导出失败' : 'Export failed'}：${reason instanceof Error ? reason.message : String(reason)}`);
    } finally { setExporting(false); }
  }
  return <>
    <section className="decision-chapter"><h2>{zh ? '机会概览' : 'Opportunity overview'}</h2><div className="opportunity-grid">{summaries.map((row) => { const d = parseMetricHint(row.hint); const unavailable = d.availability_status === 'UNAVAILABLE'; return <button type="button" key={row.label} className={`opportunity-card ${kind === row.label ? 'is-selected' : ''} ${unavailable ? 'is-unavailable' : ''}`} onClick={() => { setKind(kind === row.label ? '' : row.label); setPage(1); }}><span>{opportunityNames[row.label]?.[zh ? 0 : 1] ?? row.label}</span><strong>{unavailable ? (zh ? '不可用' : 'Unavailable') : `${Number(row.value).toLocaleString()} ${zh ? '条机会' : 'opportunities'}`}</strong><p>{unavailable ? `${zh ? '数据限制' : 'Limitation'}：${d.data_limitation_code}` : `${zh ? '其中高优' : 'High priority'} ${Number(d.high_priority_users ?? 0).toLocaleString()} ${zh ? '条' : ''}`}</p><small>{zh ? '规则版本' : 'Rule version'} v{d.rule_version}</small></button>; })}</div><Explanation>{zh ? '底层粒度是“分析用户 × 机会类型”，同一 IP 可命中多个类型，因此这里展示机会条数，不等同于唯一用户数或正式营销名单。当前模型只有高/标准规则，没有综合分数。' : 'The grain is analysis user × opportunity type, so one IP can match multiple types. Counts are opportunity rows, not unique customers or a qualified marketing list. The current model has High/Standard rules and no composite score.'}</Explanation></section>
    <section className="decision-chapter"><div className="opportunity-list-head"><div><h2>{zh ? '潜客列表' : 'Candidate list'}</h2><p>{zh ? `共 ${result.total.toLocaleString()} 条；以 IP 作为分析用户标识。` : `${result.total.toLocaleString()} candidates; IP is the analysis user identifier.`}</p></div><div className="opportunity-actions"><form onSubmit={(event) => { event.preventDefault(); setPage(1); setQueryKeyword(keyword.trim()); }}><input value={keyword} onChange={(event) => setKeyword(event.target.value)} placeholder={zh ? '搜索用户 IP 或主 App' : 'Search IP or primary App'} /><button type="submit">{zh ? '查询' : 'Search'}</button></form><button type="button" disabled={exporting || loading || result.total === 0} onClick={exportCandidates}>{exporting ? (zh ? '正在导出…' : 'Exporting…') : (zh ? '导出潜客明细 CSV' : 'Export candidate details CSV')}</button></div></div>
      {exportStatus && <p className="opportunity-export-status">{exportStatus}</p>}
      {error ? <div className="app-detail-state is-error"><strong>{zh ? '潜客列表加载失败' : 'Failed to load candidates'}</strong><span>{error}</span></div> : loading ? <div className="app-detail-state is-loading">{zh ? '正在读取已物化潜客…' : 'Loading materialized candidates…'}</div> : result.rows.length ? <div className="decision-table-wrap"><table className="decision-table opportunity-table"><thead><tr><th>{zh ? '用户 IP' : 'User IP'}</th><th>{zh ? '机会类型' : 'Type'}</th><th>{zh ? '优先级' : 'Priority'}</th><th>{zh ? '接入制式' : 'Access'}</th><th>{zh ? '活跃天数' : 'Active days'}</th><th>{zh ? '总流量' : 'Traffic'}</th><th>{zh ? '主 App' : 'Primary App'}</th><th>{zh ? '为什么是候选' : 'Why selected'}</th></tr></thead><tbody>{result.rows.map((row) => { const explanation = opportunityExplanation(row, rule, zh); return <tr key={`${row.user_key}-${row.opportunity_type}`} role="button" tabIndex={0} onClick={() => setSelected(row)} onKeyDown={(event) => { if (event.key === 'Enter') setSelected(row); }}><td><strong>{row.user_key}</strong></td><td>{opportunityNames[row.opportunity_type]?.[zh ? 0 : 1] ?? row.opportunity_type}</td><td><span className={`opportunity-level is-${row.opportunity_level.toLowerCase()}`}>{row.opportunity_level === 'HIGH' ? (zh ? '高' : 'High') : (zh ? '标准' : 'Standard')}</span></td><td>{row.user_type}</td><td>{row.active_days}</td><td>{friendlyNumber(String(row.total_download_gb), 'GB')}</td><td>{row.primary_app || '—'}</td><td className="opportunity-reason-cell">{explanation.sentence}</td></tr>; })}</tbody></table></div> : <div className="decision-empty">{zh ? '当前筛选下没有潜客。若全部为 0，请在数据作业中心从聚合阶段继续，页面本身不会启动重计算。' : 'No candidates match. Resume aggregation in Data Jobs if results have not been built.'}</div>}
      <div className="opportunity-pagination"><button type="button" disabled={page <= 1 || loading} onClick={() => setPage((value) => value - 1)}>{zh ? '上一页' : 'Previous'}</button><span>{page} / {pageCount}</span><button type="button" disabled={page >= pageCount || loading} onClick={() => setPage((value) => value + 1)}>{zh ? '下一页' : 'Next'}</button></div>
    </section>
    {selected && (() => { const explanation = opportunityExplanation(selected, rule, zh); return <div className="app-detail-modal-backdrop" role="presentation" onMouseDown={() => setSelected(null)}><section className="app-detail-modal opportunity-detail-modal" role="dialog" aria-modal="true" aria-labelledby="opportunity-detail-title" onMouseDown={(event) => event.stopPropagation()}><header className="app-detail-modal-head"><div><span>{zh ? '潜客证据详情' : 'Candidate evidence'}</span><h2 id="opportunity-detail-title">{selected.user_key}</h2><p>{opportunityNames[selected.opportunity_type]?.[zh ? 0 : 1]}</p></div><button type="button" autoFocus onClick={() => setSelected(null)}>{zh ? '关闭' : 'Close'}</button></header><div className="app-detail-modal-body"><article className="opportunity-evidence"><h3>{zh ? '为什么进入候选池' : 'Why this user was selected'}</h3><p>{explanation.sentence}</p><div className="opportunity-rule-checks">{explanation.checks.map((check) => <div key={check.label} className={check.hit ? 'is-hit' : ''}><span>{check.hit ? '✓' : '–'} {check.label}</span><strong>{check.actual}</strong><small>{check.rule}</small></div>)}</div><h3>{zh ? '优先级如何产生' : 'How priority is assigned'}</h3><p>{explanation.priority}</p><small>{zh ? '规则版本' : 'Rule version'} v{selected.rule_profile_version}{selected.data_limitation_code ? ` · ${selected.data_limitation_code}` : ''}</small></article><div className="detail-stat-grid"><span>{zh ? '接入制式' : 'Access'}<strong>{selected.user_type}</strong></span><span>{zh ? '活跃天数' : 'Active days'}<strong>{selected.active_days}</strong></span><span>{zh ? '观测记录' : 'Observations'}<strong>{selected.observation_rows.toLocaleString()}</strong></span><span>{zh ? '总流量' : 'Traffic'}<strong>{friendlyNumber(String(selected.total_download_gb), 'GB')}</strong></span><span>{zh ? '有效时长' : 'Effective duration'}<strong>{friendlyNumber(String(selected.total_effective_duration_hours), 'h')}</strong></span><span>{zh ? '有效下载速率' : 'Effective rate'}<strong>{selected.avg_effective_download_mbps == null ? '—' : friendlyNumber(String(selected.avg_effective_download_mbps), 'Mbps')}</strong></span><span>{zh ? '主 App' : 'Primary App'}<strong>{selected.primary_app || '—'}</strong></span><span>{zh ? '主 App 活跃天数' : 'Primary App days'}<strong>{selected.primary_app_active_days}</strong></span><span>{zh ? '主 App 观测' : 'Primary App obs'}<strong>{selected.primary_app_observations.toLocaleString()}</strong></span><span>{zh ? 'Wi-Fi 时延' : 'Wi-Fi delay'}<strong>{selected.avg_wifi_delay_ms == null ? '—' : friendlyNumber(String(selected.avg_wifi_delay_ms), 'ms')}</strong></span><span>{zh ? '用户侧 RTT' : 'Subscriber RTT'}<strong>{selected.avg_subscriber_rtt_ms == null ? '—' : friendlyNumber(String(selected.avg_subscriber_rtt_ms), 'ms')}</strong></span><span>{zh ? '网络侧 RTT' : 'Network RTT'}<strong>{selected.avg_network_rtt_ms == null ? '—' : friendlyNumber(String(selected.avg_network_rtt_ms), 'ms')}</strong></span></div><Explanation>{zh ? '候选只表示当前应用体验数据支持的行动方向。正式营销资格仍需 CRM、FTTH 覆盖、套餐、欠费、黑名单、可触达和合规状态复核。' : 'This is an experience-driven action candidate. CRM, FTTH coverage, plan, arrears, blacklist, reachability and compliance still require validation.'}</Explanation></div></section></div>; })()}
  </>;
}

function AccessDeltaTable({ rows, zh }: { rows: MetricCard[]; zh: boolean }) {
  const values = Object.fromEntries(rows.map((row) => [row.label, { ...parseMetricHint(row.hint), users: row.value }])) as Record<string, Record<string, string>>;
  const cable = values.CABLE; const ftth = values.FTTH;
  if (!cable || !ftth) return <div className="decision-empty">{zh ? 'Cable 或 FTTH 一侧没有可比样本。' : 'Cable or FTTH has no comparable sample.'}</div>;
  const metrics: Array<[string, string, string, 'high' | 'low' | 'neutral']> = [
    ['users', zh ? '分析用户' : 'Users', 'users', 'neutral'], ['traffic_gb', zh ? '总流量' : 'Traffic', 'GB', 'neutral'],
    ['duration_hours', zh ? '有效业务时长' : 'Effective duration', 'h', 'neutral'], ['effective_download_mbps', zh ? '视频有效下载速率' : 'Effective rate', 'Mbps', 'high'],
    ['poor_observation_rate_pct', zh ? '差体验观测占比' : 'Poor observation rate', 'percent', 'low'], ['persistent_poor_user_rate_pct', zh ? '持续质差用户占比' : 'Persistent poor user rate', 'percent', 'low'],
    ['subscriber_rtt_ms', zh ? '用户侧 RTT' : 'Subscriber RTT', 'ms', 'low'], ['network_rtt_ms', zh ? '网络侧 RTT' : 'Network RTT', 'ms', 'low'],
    ['user_loss_pct', zh ? '用户侧丢包' : 'User loss', 'percent', 'low'], ['network_loss_pct', zh ? '网络侧丢包' : 'Network loss', 'percent', 'low'],
  ];
  return <div className="decision-table-wrap"><table className="decision-table access-delta-table"><thead><tr><th>{zh ? '指标' : 'Metric'}</th><th>Cable</th><th>FTTH</th><th>{zh ? '绝对差（Cable-FTTH）' : 'Absolute delta'}</th><th>{zh ? '相对差' : 'Relative delta'}</th><th>{zh ? '解读' : 'Reading'}</th></tr></thead><tbody>{metrics.map(([key,label,unit,direction]) => { const left=Number(cable[key]); const right=Number(ftth[key]); const available=Number.isFinite(left)&&Number.isFinite(right); const delta=left-right; const relative=right === 0 ? null : delta/Math.abs(right)*100; const cableWorse=available && (direction==='low'?delta>0:direction==='high'?delta<0:false); return <tr key={key} className={cableWorse?'comparison-worse':''}><td>{label}</td><td>{available?friendlyNumber(String(left),unit):'—'}</td><td>{available?friendlyNumber(String(right),unit):'—'}</td><td>{available?friendlyNumber(String(delta),unit):'—'}</td><td>{available&&relative!=null?`${relative>=0?'+':''}${relative.toFixed(1)}%`:'—'}</td><td>{!available?(zh?'不可比':'Unavailable'):direction==='neutral'?(zh?'规模指标，不判断好坏':'Scale only'):cableWorse?(zh?'Cable 较弱':'Cable weaker'):(zh?'Cable 未弱于 FTTH':'Cable not weaker')}</td></tr>; })}</tbody></table></div>;
}

function hourlyLineSegments(values: Array<{ hour: number; value: number }>, max: number, span: number) {
  const segments: string[] = [];
  let current: string[] = [];
  let previousHour: number | null = null;
  values.forEach(({ hour, value }) => {
    if (previousHour != null && hour !== previousHour + 1) {
      if (current.length > 1) segments.push(current.join(' '));
      current = [];
    }
    current.push(`${24 + hour * (712 / 23)},${18 + (max - value) / span * 180}`);
    previousHour = hour;
  });
  if (current.length > 1) segments.push(current.join(' '));
  return segments;
}

function AccessExperienceRadar({ rows, hourly, zh }: { rows: MetricCard[]; hourly: MetricCard[]; zh: boolean }) {
  const byAccess = Object.fromEntries(rows.map((row) => [row.label, parseMetricHint(row.hint)])) as Record<string, Record<string, string>>;
  const firstHourly = parseMetricHint(hourly[0]?.hint ?? '');
  const peakStart = numeric(firstHourly.peak_hour_start) ?? 20;
  const peakEnd = numeric(firstHourly.peak_hour_end) ?? 23;
  const clamp = (value: number) => Math.max(0, Math.min(100, value));
  const peakDelta = (access: string, overallRate: number | null) => {
    if (overallRate == null) return null;
    const peak = hourly.filter((row) => { const d = parseMetricHint(row.hint); const hour = numeric(d.hour); return d.access_type === access && hour != null && hour >= peakStart && hour <= peakEnd; });
    const valid = peak.reduce((sum, row) => sum + (numeric(parseMetricHint(row.hint).valid_obs_rows) ?? 0), 0);
    const poor = peak.reduce((sum, row) => sum + (numeric(parseMetricHint(row.hint).poor_obs_rows) ?? 0), 0);
    return valid > 0 ? poor / valid * 100 - overallRate : null;
  };
  const dimensions = [
    zh ? '下载保障' : 'Download',
    zh ? '时延体验' : 'Latency',
    zh ? '丢包体验' : 'Loss',
    zh ? '良好体验' : 'Good QoE',
    zh ? '高峰稳定' : 'Peak stability',
  ];
  const series = ['CABLE', 'FTTH'].map((access) => {
    const d = byAccess[access] ?? {};
    const effective = numeric(d.effective_download_mbps);
    const subscriberRtt = numeric(d.subscriber_rtt_ms);
    const networkRtt = numeric(d.network_rtt_ms);
    const userLoss = numeric(d.user_loss_pct);
    const networkLoss = numeric(d.network_loss_pct);
    const poorRate = numeric(d.poor_observation_rate_pct);
    const worstRtt = subscriberRtt != null && networkRtt != null ? Math.max(subscriberRtt, networkRtt) : null;
    const worstLoss = userLoss != null && networkLoss != null ? Math.max(userLoss, networkLoss) : null;
    const degradation = peakDelta(access, poorRate);
    const scores: Array<number | null> = [
      effective == null ? null : clamp(effective / 50 * 100),
      worstRtt == null ? null : clamp(100 - worstRtt / 200 * 100),
      worstLoss == null ? null : clamp(100 - worstLoss / 3 * 100),
      poorRate == null ? null : clamp(100 - poorRate / 40 * 100),
      degradation == null ? null : clamp(100 - Math.max(0, degradation) / 20 * 100),
    ];
    const raw = [
      effective == null ? '—' : `${effective.toFixed(2)} Mbps`,
      worstRtt == null ? '—' : `${worstRtt.toFixed(2)} ms`,
      worstLoss == null ? '—' : `${worstLoss.toFixed(3)}%`,
      poorRate == null ? '—' : `${(100 - poorRate).toFixed(2)}%`,
      degradation == null ? '—' : `${degradation >= 0 ? '+' : ''}${degradation.toFixed(2)} pct`,
    ];
    return { access, scores, raw, complete: scores.every((score) => score != null) };
  });
  const centerX = 215; const centerY = 165; const radius = 110;
  const point = (index: number, scale: number) => { const angle = -Math.PI / 2 + index * Math.PI * 2 / 5; return [centerX + Math.cos(angle) * radius * scale, centerY + Math.sin(angle) * radius * scale]; };
  const polygon = (scores: Array<number | null>) => scores.map((score, index) => point(index, (score ?? 0) / 100).join(',')).join(' ');
  const gridPolygon = (scale: number) => dimensions.map((_, index) => point(index, scale).join(',')).join(' ');
  return <section className="access-radar-card">
    <div className="access-radar-copy"><span>{zh ? '标准化综合概览' : 'Normalized summary'}</span><h3>{zh ? 'Cable / FTTH 五维体验雷达' : 'Cable / FTTH five-dimension radar'}</h3><p>{zh ? '用于快速发现差异，下面的原始值和专项图表才是精确证据。评分使用固定业务锚点，不按本批次两组相对排名。' : 'Use this to spot differences; raw values and detailed charts remain the precise evidence. Scores use fixed anchors, not relative ranking within this batch.'}</p></div>
    <div className="access-radar-layout"><svg className="access-radar-svg" viewBox="0 0 430 330" role="img" aria-label={zh ? 'Cable 与 FTTH 五维体验雷达图' : 'Cable and FTTH five-dimension experience radar'}>
      {[.25,.5,.75,1].map((scale)=><polygon key={scale} points={gridPolygon(scale)} fill="none" stroke="#d0d5dd" strokeWidth="1"/>)}
      {dimensions.map((label,index)=>{ const [x,y]=point(index,1.18); return <g key={label}><line x1={centerX} y1={centerY} x2={point(index,1)[0]} y2={point(index,1)[1]} stroke="#e4e7ec"/><text x={x} y={y} textAnchor={x<centerX-10?'end':x>centerX+10?'start':'middle'} dominantBaseline="middle" fontSize="12" fill="#344054">{label}</text></g>; })}
      {series[0].complete&&<polygon points={polygon(series[0].scores)} fill="rgba(181,71,8,.16)" stroke="#b54708" strokeWidth="2.5"/>}
      {series[1].complete&&<polygon points={polygon(series[1].scores)} fill="rgba(8,126,139,.15)" stroke="#087e8b" strokeWidth="2.5"/>}
      <g transform="translate(125 306)"><line x1="0" y1="0" x2="22" y2="0" stroke="#b54708" strokeWidth="3"/><text x="28" y="4" fontSize="11" fill="#475467">Cable</text><line x1="90" y1="0" x2="112" y2="0" stroke="#087e8b" strokeWidth="3"/><text x="118" y="4" fontSize="11" fill="#475467">FTTH</text></g>
    </svg><div className="access-radar-values">{dimensions.map((dimension,index)=><div key={dimension}><strong>{dimension}</strong><span><i className="is-cable"/>Cable {series[0].raw[index]}{series[0].scores[index] == null ? '' : ` · ${series[0].scores[index]!.toFixed(0)}`}</span><span><i className="is-ftth"/>FTTH {series[1].raw[index]}{series[1].scores[index] == null ? '' : ` · ${series[1].scores[index]!.toFixed(0)}`}</span></div>)}</div></div>
    {series.some((item)=>!item.complete)&&<p className="radar-coverage-warning">{zh?'至少一侧存在不可用维度，因此不闭合绘制该侧多边形；缺失值不会按 0 分处理。':'At least one side has an unavailable dimension, so its polygon is not closed; missing values are not scored as zero.'}</p>}
    <Explanation>{zh?`锚点：50 Mbps=下载满分；时延与丢包取用户侧/网络侧较差值，200 ms=时延 0 分，3%=丢包 0 分；40%=差观测达到 0 分；高峰（${peakStart}:00–${peakEnd}:59）差观测比全日高 20 个百分点=稳定性 0 分。该图是描述性比较，不代表迁转因果效果。`:`Anchors: 50 Mbps=full download score; latency and loss use the worse subscriber/network value, with 200 ms=zero latency score and 3%=zero loss score; 40% poor observations=zero good-QoE score; a 20-point peak degradation (${peakStart}:00–${peakEnd}:59)=zero stability score. This is descriptive, not causal migration evidence.`}</Explanation>
  </section>;
}

function PanoramaMetricExplorer({ metrics, distributions, apps, hourly, zh }: { metrics: MetricCard[]; distributions: MetricCard[]; apps: MetricCard[]; hourly: MetricCard[]; zh: boolean }) {
  const [metric,setMetric]=useState('traffic');
  const config:Record<string,{label:string;overall:string;app?:string;distribution?:string;hourly?:string;unit:string;lower?:boolean}>={
    traffic:{label:zh?'流量':'Traffic',overall:'traffic',app:'traffic_gb',distribution:'TRAFFIC_DAILY',hourly:'traffic_gb',unit:'GB'},
    duration:{label:zh?'有效时长':'Effective duration',overall:'effective_duration',app:'duration_hours',distribution:'DURATION_EFFECTIVE_DAILY',hourly:'duration_hours',unit:'h'},
    upstream:{label:zh?'上行速率':'Upstream rate',overall:'upstream_rate',unit:'Mbps'},
    downstream:{label:zh?'下行平均速率':'Average download rate',overall:'average_download_rate',distribution:'AVERAGE_DOWNLOAD_RATE',hourly:'average_download_mbps',unit:'Mbps'},
    effective:{label:zh?'视频有效下载速率':'Video effective rate',overall:'effective_download_rate',app:'effective_download_mbps',distribution:'EFFECTIVE_RATE',hourly:'effective_download_mbps',unit:'Mbps'},
    subscriberRtt:{label:zh?'用户侧时延':'Subscriber RTT',overall:'subscriber_rtt',app:'subscriber_rtt_ms',distribution:'SUBSCRIBER_RTT',hourly:'subscriber_rtt_ms',unit:'ms',lower:true},
    networkRtt:{label:zh?'网络侧时延':'Network RTT',overall:'network_rtt',app:'network_rtt_ms',distribution:'NETWORK_RTT',hourly:'network_rtt_ms',unit:'ms',lower:true},
    userLoss:{label:zh?'用户侧丢包':'User loss',overall:'user_loss',app:'user_loss_pct',distribution:'USER_LOSS',hourly:'user_loss_pct',unit:'percent',lower:true},
    networkLoss:{label:zh?'网络侧丢包':'Network loss',overall:'network_loss',app:'network_loss_pct',distribution:'NETWORK_LOSS',hourly:'network_loss_pct',unit:'percent',lower:true},
  };
  const current=config[metric]; const overall=metrics.find((row)=>parseMetricHint(row.hint).metric===current.overall); const bands=distributions.filter((row)=>parseMetricHint(row.hint).dimension===current.distribution);
  const ranked=current.app?apps.map((row)=>({row,value:Number(parseMetricHint(row.hint)[current.app!])})).filter((item)=>Number.isFinite(item.value)).sort((a,b)=>current.lower?b.value-a.value:b.value-a.value).slice(0,20):[];
  const hourlyValues=current.hourly?hourly.map((row)=>({hour:Number(parseMetricHint(row.hint).hour),value:Number(parseMetricHint(row.hint)[current.hourly!])})).filter((item)=>Number.isFinite(item.value)).sort((a,b)=>a.hour-b.hour):[];
  const max=Math.max(1,...hourlyValues.map((item)=>item.value)); const min=Math.min(0,...hourlyValues.map((item)=>item.value)); const span=Math.max(max-min,.000001); const lines=hourlyLineSegments(hourlyValues,max,span); const appMax=Math.max(1,...ranked.map((item)=>item.value));
  const hourlyHint=parseMetricHint(hourly[0]?.hint??''); const peakStart=numeric(hourlyHint.peak_hour_start)??20; const peakEnd=numeric(hourlyHint.peak_hour_end)??23; const peakX=24+peakStart*(712/24); const peakWidth=Math.max(1,peakEnd-peakStart+1)*(712/24);
  const overallHint=overall?parseMetricHint(overall.hint):{}; const unavailableReason=overallHint.limitation??(zh?'当前发布结果没有可解释的有效样本。':'The current publication has no interpretable valid sample.');
  return <section className="metric-explorer"><div className="metric-explorer-tabs">{Object.entries(config).map(([key,item])=><button type="button" className={metric===key?'active':''} key={key} onClick={()=>setMetric(key)}>{item.label}</button>)}</div><div className="metric-explorer-grid"><article className="metric-focus-card"><span>{current.label}</span><strong>{overall?friendlyNumber(overall.value,current.unit):'—'}</strong><p>{overallHint.availability==='UNAVAILABLE'?(zh?`当前不可用：${unavailableReason}`:`Unavailable: ${unavailableReason}`):(zh?'当前批次整体口径；下方分别展示用户分布、小时变化和 App 排序。':'Batch-wide metric with user distribution, hourly trend and App ranking below.')}</p></article>{hourlyValues.length>0?<article className="metric-hourly-card"><h3>{zh?'24 小时变化':'24-hour trend'}</h3><svg viewBox="0 0 760 225"><rect width="760" height="225" fill="#fff"/><rect x={peakX} y="18" width={peakWidth} height="180" fill="#fff4d6"/><line x1="24" y1="198" x2="736" y2="198" stroke="#d0d5dd"/>{lines.map((line,index)=><polyline key={index} points={line} fill="none" stroke="#22705d" strokeWidth="3"/>)}{hourlyValues.map((item)=><circle key={item.hour} cx={24+item.hour*(712/23)} cy={18+(max-item.value)/span*180} r="3" fill="#fff" stroke="#22705d" strokeWidth="2"/>)}{[0,6,12,18,23].map((hour)=><text key={hour} x={24+hour*(712/23)} y="218" textAnchor="middle" fontSize="10" fill="#667085">{hour}:00</text>)}</svg><small>{zh?`高峰区间 ${peakStart}:00–${peakEnd}:59；缺失小时不跨点连线。`:`Peak window ${peakStart}:00–${peakEnd}:59; missing hours are not bridged.`}</small></article>:<article className="metric-hourly-card is-unavailable"><h3>{zh?'24 小时变化':'24-hour trend'}</h3><p>{zh?'当前指标尚无可靠小时聚合。':'No reliable hourly aggregate for this metric.'}</p></article>}</div>{bands.length>0&&<section><h3>{zh?'用户分档':'User distribution'}</h3><Distribution rows={bands}/></section>}{ranked.length>0&&<section><h3>{zh?'App 排序与问题高亮':'App ranking with issue highlighting'}</h3><div className="metric-app-ranking">{ranked.map(({row,value})=>{ const status=parseMetricHint(row.hint).insight_status??'INSUFFICIENT'; return <div className={`metric-app-rank is-${status.toLowerCase()}`} key={row.label}><span>{row.label}<small>{status}</small></span><div><i style={{width:`${Math.max(2,value/appMax*100)}%`}}/></div><strong>{friendlyNumber(String(value),current.unit)}</strong></div>; })}</div><Explanation>{zh?'排序展示所有有值 App；达到关注、问题或严重规则的 App 使用状态色高亮。高值是否更差由指标方向决定，不把规模大直接解释为质差。':'All Apps with values are ranked and issue statuses are highlighted. High scale is not automatically poor experience.'}</Explanation></section>}</section>;
}

function AccessHourlyChart({ rows, zh, initialMetric = 'users', fixed = false }: { rows: MetricCard[]; zh: boolean; initialMetric?: string; fixed?: boolean }) {
  const [metric, setMetric] = useState(initialMetric);
  const metrics: Record<string, [string,string]> = { users:[zh?'用户':'Users','users'], traffic_gb:[zh?'流量':'Traffic','GB'], average_download_mbps:[zh?'平均下载速率':'Average download rate','Mbps'], effective_download_mbps:[zh?'有效速率':'Effective rate','Mbps'], poor_observation_rate_pct:[zh?'差观测占比':'Poor obs rate','percent'], subscriber_rtt_ms:[zh?'用户侧 RTT':'Subscriber RTT','ms'], network_rtt_ms:[zh?'网络侧 RTT':'Network RTT','ms'], user_loss_pct:[zh?'用户侧丢包':'User loss','percent'], network_loss_pct:[zh?'网络侧丢包':'Network loss','percent'] };
  const series = ['CABLE','FTTH'].map((access) => ({ access, values: Array.from({length:24},(_,hour) => { const row=rows.find((item) => { const d=parseMetricHint(item.hint); return d.access_type===access && Number(d.hour)===hour; }); if (!row) return null; const d=parseMetricHint(row.hint); const value=metric==='users'?Number(row.value):Number(d[metric]); return Number.isFinite(value)?value:null; }) }));
  const all=series.flatMap((item)=>item.values).filter((value): value is number => value!=null); const max=Math.max(...all,1); const min=Math.min(...all,0); const span=Math.max(max-min,0.000001);
  const lineSegments=(values:Array<number|null>)=>hourlyLineSegments(values.flatMap((value,hour)=>value==null?[]:[{hour,value}]),max,span);
  const firstHint=parseMetricHint(rows[0]?.hint??''); const peakStart=numeric(firstHint.peak_hour_start)??20; const peakEnd=numeric(firstHint.peak_hour_end)??23; const peakX=24+peakStart*(712/24); const peakWidth=Math.max(1,peakEnd-peakStart+1)*(712/24);
  return <section className="access-hourly">{fixed&&<h3>{metrics[metric][0]}</h3>}{!fixed&&<div className="access-metric-tabs">{Object.entries(metrics).map(([key,[label]])=><button type="button" className={metric===key?'active':''} key={key} onClick={()=>setMetric(key)}>{label}</button>)}</div>}<div className="access-chart-shell"><svg viewBox="0 0 760 230" role="img" aria-label={metrics[metric][0]}><rect x="0" y="0" width="760" height="230" fill="#fff"/><rect x={peakX} y="18" width={peakWidth} height="180" fill="#fff4d6"/><line x1="24" y1="198" x2="736" y2="198" stroke="#d0d5dd"/>{[0,6,12,18,23].map((hour)=><text key={hour} x={24+hour*(712/23)} y="218" textAnchor="middle" fontSize="10" fill="#667085">{hour}:00</text>)}{lineSegments(series[0].values).map((points,index)=><polyline key={`c-${index}`} points={points} fill="none" stroke="#b54708" strokeWidth="3"/>)}{lineSegments(series[1].values).map((points,index)=><polyline key={`f-${index}`} points={points} fill="none" stroke="#087e8b" strokeWidth="3"/>)}{series.map((item)=><g key={item.access}>{item.values.map((value,hour)=>value==null?null:<circle key={hour} cx={24+hour*(712/23)} cy={18+(max-value)/span*180} r="3" fill="#fff" stroke={item.access==='CABLE'?'#b54708':'#087e8b'} strokeWidth="2"/>)}</g>)}</svg><div className="access-chart-legend"><span className="is-cable">Cable</span><span className="is-ftth">FTTH</span><strong>{zh?'范围':'Range'} {friendlyNumber(String(min),metrics[metric][1])} – {friendlyNumber(String(max),metrics[metric][1])}</strong></div></div><Explanation>{zh?`同一小时、同一口径对比 Cable 与 FTTH；${peakStart}:00–${peakEnd}:59 为当前分析运行绑定的高峰区间。缺失小时不跨点连线。`:`Cable and FTTH use the same hourly grain; ${peakStart}:00–${peakEnd}:59 is the bound peak window. Missing hours are not bridged.`}</Explanation></section>;
}

function AccessBandComparison({ rows, zh }: { rows: MetricCard[]; zh: boolean }) {
  const groups=rows.reduce<Record<string,MetricCard[]>>((all,row)=>{ const key=parseMetricHint(row.hint).dimension??'OTHER'; (all[key]??=[]).push(row); return all; },{});
  const titles:Record<string,string>={TRAFFIC_DAILY:zh?'日均流量用户分布':'Daily traffic distribution',DURATION_EFFECTIVE_DAILY:zh?'日均有效时长用户分布':'Daily duration distribution',EFFECTIVE_RATE:zh?'有效下载速率用户分布':'Effective rate distribution',SUBSCRIBER_RTT:zh?'用户侧 RTT 用户分布':'Subscriber RTT distribution',NETWORK_RTT:zh?'网络侧 RTT 用户分布':'Network RTT distribution',USER_LOSS:zh?'用户侧丢包用户分布':'User loss distribution',NETWORK_LOSS:zh?'网络侧丢包用户分布':'Network loss distribution',EXPERIENCE:zh?'持续质差用户分布':'Persistent experience distribution'};
  return <div className="distribution-grid access-band-grid">{Object.entries(groups).map(([dimension,items])=>{ const bands=[...new Set(items.map((item)=>item.label))].sort((left,right)=>{ const leftRow=items.find((item)=>item.label===left); const rightRow=items.find((item)=>item.label===right); return bandOrder(leftRow!)-bandOrder(rightRow!); }); const cableTotal=items.filter((item)=>parseMetricHint(item.hint).access_type==='CABLE').reduce((sum,item)=>sum+(numeric(item.value)??0),0); const ftthTotal=items.filter((item)=>parseMetricHint(item.hint).access_type==='FTTH').reduce((sum,item)=>sum+(numeric(item.value)??0),0); return <article className="distribution-card" key={dimension}><h3>{titles[dimension]??dimension}</h3><small className="distribution-denominator">Cable n={cableTotal.toLocaleString()} · FTTH n={ftthTotal.toLocaleString()}</small>{bands.map((band)=>{ const cable=items.find((item)=>item.label===band&&parseMetricHint(item.hint).access_type==='CABLE'); const ftth=items.find((item)=>item.label===band&&parseMetricHint(item.hint).access_type==='FTTH'); const cableUsers=numeric(cable?.value)??0; const ftthUsers=numeric(ftth?.value)??0; const cableShare=cableTotal>0?cableUsers/cableTotal*100:0; const ftthShare=ftthTotal>0?ftthUsers/ftthTotal*100:0; return <div className="access-band-row" key={band}><span>{band}</span><div><i className="is-cable" style={{width:`${cableShare}%`}}/><em>{cableShare.toFixed(1)}% <small>{cableUsers.toLocaleString()}</small></em></div><div><i className="is-ftth" style={{width:`${ftthShare}%`}}/><em>{ftthShare.toFixed(1)}% <small>{ftthUsers.toLocaleString()}</small></em></div></div>; })}<Explanation>{zh?'柱长比较各接入制式内部的用户占比，避免 Cable/FTTH 总体规模不同造成误导；右侧同时保留人数。':'Bars compare within-access shares to avoid cohort-size distortion; counts remain visible.'}</Explanation></article>; })}</div>;
}

function AccessAppComparison({ rows, zh }: { rows: MetricCard[]; zh: boolean }) {
  const apps=rows.reduce<Record<string,Record<string,MetricCard>>>((all,row)=>{ const access=parseMetricHint(row.hint).access_type??'UNKNOWN'; (all[row.label]??={})[access]=row; return all; },{});
  return <div className="decision-table-wrap"><table className="decision-table access-app-table"><thead><tr><th>App</th><th>Cable {zh?'用户':'users'}</th><th>FTTH {zh?'用户':'users'}</th><th>Cable {zh?'差观测':'poor obs'}</th><th>FTTH {zh?'差观测':'poor obs'}</th><th>{zh?'差异':'Delta'}</th><th>{zh?'样本':'Sample'}</th></tr></thead><tbody>{Object.entries(apps).slice(0,100).map(([app,sides])=>{ const c=sides.CABLE&&parseMetricHint(sides.CABLE.hint); const f=sides.FTTH&&parseMetricHint(sides.FTTH.hint); const cp=Number(c?.poor_observation_rate_pct); const fp=Number(f?.poor_observation_rate_pct); const comparable=c&&f&&c.sample_status==='SUFFICIENT'&&f.sample_status==='SUFFICIENT'&&Number.isFinite(cp)&&Number.isFinite(fp); return <tr key={app} className={comparable&&cp>fp?'comparison-worse':''}><td><strong>{app}</strong></td><td>{Number(c?.observed_users??0).toLocaleString()}</td><td>{Number(f?.observed_users??0).toLocaleString()}</td><td>{Number.isFinite(cp)?`${cp.toFixed(2)}%`:'—'}</td><td>{Number.isFinite(fp)?`${fp.toFixed(2)}%`:'—'}</td><td>{comparable?`${cp-fp>=0?'+':''}${(cp-fp).toFixed(2)} pct`:'—'}</td><td>{comparable?(zh?'可比':'Comparable'):(zh?'样本不足/单侧缺失':'Insufficient/missing side')}</td></tr>; })}</tbody></table></div>;
}

function FullPrintReportView({ report, batchName, runId, zh }: { report: FullPrintReport; batchName: string; runId: string; zh: boolean }) {
  const generatedAt = new Date();
  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone || 'Local';
  return <section className="all-charts-print-report">
    <header><h1>SA FBB Experience Workbench</h1><p>{zh ? '全部洞察图表' : 'All insight charts'} · {batchName} · {runId}</p><small>{zh ? '生成时间' : 'Generated'}：{generatedAt.toLocaleString()} ({timezone}) · {zh ? '数据源：已发布 DWS / ADS' : 'Source: published DWS / ADS'}</small></header>
    {report.metrics.length > 0 && <article><h2>1. {zh ? '整体指标' : 'Overall metrics'}</h2><CardGrid rows={report.metrics} /></article>}
    {report.apps.length > 0 && <article><h2>2. {zh ? 'App 全景（按分析用户覆盖 Top 40）' : 'App panorama (Top 40 by analysis-user coverage)'}</h2><PrintAppBars rows={report.apps} /><Explanation>{zh ? '唯一 App 粒度；完整明细保留在交互页面，PDF 默认只输出图形。' : 'Unique-App grain; full detail remains in the interactive view and the PDF is chart-first.'}</Explanation></article>}
    {report.users.length > 0 && <article><h2>3. {zh ? '用户分布' : 'User distributions'}</h2><Distribution rows={report.users} /></article>}
    {report.quality.length > 0 && <article><h2>4. {zh ? '质差证据' : 'Poor-quality evidence'}</h2><CardGrid rows={report.quality} /></article>}
    {report.access.length > 0 && <article><h2>5. {zh ? 'Cable / FTTH 专项图表' : 'Cable / FTTH analysis'}</h2><AccessExperienceRadar rows={report.access} hourly={report.accessHourly} zh={zh}/>{report.accessHourly.length > 0 && ['users','traffic_gb','average_download_mbps','effective_download_mbps','poor_observation_rate_pct','subscriber_rtt_ms','network_rtt_ms','user_loss_pct','network_loss_pct'].map((metric)=><AccessHourlyChart key={metric} rows={report.accessHourly} zh={zh} initialMetric={metric} fixed />)}{report.accessBands.length > 0 && <AccessBandComparison rows={report.accessBands} zh={zh} />}</article>}
    {report.opportunities.length > 0 && <article><h2>6. {zh ? '潜客机会' : 'Opportunities'}</h2><CardGrid rows={report.opportunities} /></article>}
  </section>;
}

export function DecisionWorkspaceV3({ c, view }: { c: WorkbenchController; view: DecisionView }) {
  const [perspective, setPerspective] = useState<PanoramaPerspective>('metric');
  const [rows, setRows] = useState<MetricCard[]>([]);
  const [baselineRows, setBaselineRows] = useState<MetricCard[]>([]);
  const [distributionRows, setDistributionRows] = useState<MetricCard[]>([]);
  const [appRows, setAppRows] = useState<MetricCard[]>([]);
  const [accessHourlyRows, setAccessHourlyRows] = useState<MetricCard[]>([]);
  const [accessBandRows, setAccessBandRows] = useState<MetricCard[]>([]);
  const [accessAppRows, setAccessAppRows] = useState<MetricCard[]>([]);
  const [panoramaHourlyRows, setPanoramaHourlyRows] = useState<MetricCard[]>([]);
  const [status, setStatus] = useState('');
  const [loading, setLoading] = useState(false);
  const [printReport, setPrintReport] = useState<FullPrintReport | null>(null);
  const zh = c.language === 'zh-CN';
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim() || loading;

  useEffect(() => {
    setRows([]);
    setBaselineRows([]);
    setDistributionRows([]);
    setAppRows([]);
    setAccessHourlyRows([]); setAccessBandRows([]); setAccessAppRows([]); setPanoramaHourlyRows([]);
    setStatus(c.importBatchId && c.analysisRunId ? '' : '请先选择可分析批次。');
  }, [view, c.importBatchId, c.analysisRunId]);

  useEffect(() => {
    if (!printReport) return undefined;
    document.body.classList.add('decision-pdf-print-source');
    return () => document.body.classList.remove('decision-pdf-print-source');
  }, [printReport]);

  useEffect(() => {
    if (c.importBatchId.trim() && c.analysisRunId.trim()) void load(perspective);
  }, [view, perspective, c.importBatchId, c.analysisRunId]);

  async function load(targetPerspective = perspective) {
    if (disabled) return;
    setLoading(true); setStatus('正在读取已聚合结果…');
    try {
      let result: MetricCard[];
      setBaselineRows([]); setDistributionRows([]); setAppRows([]); setAccessHourlyRows([]); setAccessBandRows([]); setAccessAppRows([]); setPanoramaHourlyRows([]);
      if (view === 'panorama') {
        if (targetPerspective === 'metric') {
          const [metrics, distributions, apps, hourly] = await Promise.all([
            analyticsStructuredApi.decisionMetricPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId),
            analyticsStructuredApi.decisionUserDistributions(c.effectiveSettings, c.importBatchId, c.analysisRunId),
            analyticsStructuredApi.decisionAppPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 500 }),
            analyticsStructuredApi.decisionPanoramaHourly(c.effectiveSettings, c.importBatchId, c.analysisRunId),
          ]);
          result = metrics; setDistributionRows(distributions); setAppRows(apps); setPanoramaHourlyRows(hourly);
        } else {
          const [apps, metrics] = await Promise.all([
            analyticsStructuredApi.decisionAppPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 500 }),
            analyticsStructuredApi.decisionMetricPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId),
          ]);
          result = apps; setBaselineRows(metrics);
        }
      } else if (view === 'quality') {
        const [quality, apps] = await Promise.all([
          analyticsStructuredApi.decisionQualityOverview(c.effectiveSettings, c.importBatchId, c.analysisRunId),
          analyticsStructuredApi.decisionAppPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 500 }),
        ]);
        result = quality;
        setAppRows(apps.filter((row) => ['SEVERE', 'PROBLEM', 'WATCH'].includes(parseMetricHint(row.hint).insight_status ?? '')));
      } else if (view === 'access') {
        const [overview,hourly,bands,apps]=await Promise.all([
          analyticsStructuredApi.decisionAccessCompare(c.effectiveSettings,c.importBatchId,c.analysisRunId),
          analyticsStructuredApi.decisionAccessHourly(c.effectiveSettings,c.importBatchId,c.analysisRunId),
          analyticsStructuredApi.decisionAccessUserBands(c.effectiveSettings,c.importBatchId,c.analysisRunId),
          analyticsStructuredApi.decisionAccessApps(c.effectiveSettings,c.importBatchId,c.analysisRunId,{pageSize:1000}),
        ]);
        result=overview; setAccessHourlyRows(hourly); setAccessBandRows(bands); setAccessAppRows(apps);
      }
      else result = await analyticsStructuredApi.decisionOpportunities(c.effectiveSettings, c.importBatchId, c.analysisRunId);
      setRows(result); setStatus(result.length ? `已加载 ${result.length} 项聚合证据。` : '结果为空；请在数据作业中心从聚合阶段继续生成缺失结果。');
    } catch (error) { setRows([]); setStatus(error instanceof Error ? error.message : String(error)); }
    finally { setLoading(false); }
  }

  async function exportAllCharts() {
    if (disabled) return;
    setLoading(true); setStatus('正在准备全部非空图表的 PDF 报告…');
    try {
      const [metrics, apps, users, quality, access, accessHourly, accessBands, opportunities] = await Promise.all([
        analyticsStructuredApi.decisionMetricPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionAppPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 500 }),
        analyticsStructuredApi.decisionUserDistributions(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionQualityOverview(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionAccessCompare(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionAccessHourly(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionAccessUserBands(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionOpportunities(c.effectiveSettings, c.importBatchId, c.analysisRunId),
      ]);
      setPrintReport({ metrics, apps, users, quality, access, accessHourly, accessBands, opportunities });
      setStatus('全部图表已准备，正在打开系统 PDF 打印对话框。');
      const clear = () => setPrintReport(null);
      window.addEventListener('afterprint', clear, { once: true });
      await document.fonts?.ready;
      window.setTimeout(() => requestAnimationFrame(() => requestAnimationFrame(() => window.print())), 0);
      window.setTimeout(clear, 300_000);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  function switchPerspective(next: PanoramaPerspective) { setPerspective(next); setRows([]); setBaselineRows([]); setDistributionRows([]); setAppRows([]); setStatus('正在切换观察角度…'); }

  return <section className="decision-workspace">
    <header className="decision-page-head"><h1>{copy[view][zh ? 0 : 1]}</h1>
      <div className="decision-load-actions"><button type="button" disabled={disabled} onClick={exportAllCharts}>{zh ? '导出 PDF' : 'Export PDF'}</button></div></header>
    {status && <div className="decision-status">{status}</div>}
    {view === 'panorama' && <div className="perspective-tabs"><button className={perspective === 'metric' ? 'active' : ''} onClick={() => switchPerspective('metric')}>{zh ? '指标视角' : 'Metrics'}</button><button className={perspective === 'app' ? 'active' : ''} onClick={() => switchPerspective('app')}>{zh ? 'App 视角' : 'Apps'}</button></div>}
    {!rows.length && <article className="decision-empty"><strong>{c.importBatchId ? (loading ? (zh ? '正在读取已聚合结果…' : 'Loading aggregated results…') : (zh ? '当前页面没有可用结果' : 'No result is available for this view')) : (zh ? '先选择数据批次' : 'Select a batch')}</strong></article>}
    {rows.length > 0 && view === 'panorama' && perspective === 'metric' && <><section className="decision-chapter"><h2>{zh?'整体指标':'Overall metrics'}</h2><CardGrid rows={rows} /><Explanation>{zh?'先看全量整体口径；百分比保留分子、分母和样本量，不可用与 0 分开。vMOS 只作为 App 体验证据，不作为一级指标。':'Start with batch-wide metrics. Rates preserve numerator, denominator and sample size; unavailable is distinct from zero.'}</Explanation></section><section className="decision-chapter"><h2>{zh?'按指标深入':'Explore by metric'}</h2><PanoramaMetricExplorer metrics={rows} distributions={distributionRows} apps={appRows} hourly={panoramaHourlyRows} zh={zh}/></section>{appRows.length > 0 && <section className="decision-chapter"><h2>{zh?'App 覆盖全景':'App coverage panorama'}</h2><PrintAppBars rows={appRows} /><Explanation>{zh?'唯一 App 粒度，展示完整 App 覆盖并保留问题高亮；Cable/FTTH 不在这一层出现。':'Unique-App coverage with issue highlighting; access type is intentionally deferred.'}</Explanation></section>}</>}
    {rows.length > 0 && view === 'panorama' && perspective === 'app' && <>{baselineRows.length > 0 && <section className="decision-chapter"><h2>整体基线</h2><CardGrid rows={baselineRows.slice(0, 7)} /></section>}<section className="decision-chapter"><h2>全部 App 状态构成</h2><AppStatusSummary rows={rows} zh={zh} /><Explanation>状态互斥且全部按唯一 App 统计；右侧六类之和必须等于“全部唯一 App”。</Explanation></section><section className="decision-chapter"><h2>全部 App 业务规模与体验状态</h2><AppTable rows={rows} c={c} /><Explanation>每个 App 只出现一次，先展示跨制式总体规模；严重、问题、关注、正常、有限样本和样本不足互斥，因此分类数量之和等于全部 App 数。</Explanation></section></>}
    {rows.length > 0 && view === 'quality' && <><section className="decision-chapter"><h2>质差规模与证据方向</h2><CardGrid rows={rows} /><Explanation>同一用户可能同时存在多类证据，因此各类用户数不能相加当作总质差用户。网络侧证据不等于网络设备根因；BRAS/OLT/PON 定位被移到后续按需调查。</Explanation></section>{appRows.length > 0 ? <section className="decision-chapter"><h2>高亮的唯一 App</h2><AppStatusSummary rows={appRows} zh={zh} /><AppTable rows={appRows} c={c} /><Explanation>这里仍按唯一 App 展示；点击后再看该 App 的用户、速率、时延和丢包分布，不提前按接入制式拆分。</Explanation></section> : <p className="decision-empty">当前充分样本中没有达到关注门槛的 App。</p>}</>}
    {rows.length > 0 && view === 'access' && <><section className="decision-chapter"><h2>{zh?'1. 综合体验概览':'1. Experience summary'}</h2><AccessExperienceRadar rows={rows} hourly={accessHourlyRows} zh={zh}/></section><section className="decision-chapter"><h2>{zh?'2. 整体规模与指标差异':'2. Overall scale and metric deltas'}</h2><AccessDeltaTable rows={rows} zh={zh}/><Explanation>{zh?'先比较总体规模和体验指标；差异只代表当前批次中两类用户群体的统计差异，不代表同一用户迁转前后的因果效果。':'Differences compare two cohorts in this batch; they are not causal before/after migration effects.'}</Explanation></section>{accessHourlyRows.length>0&&<section className="decision-chapter"><h2>{zh?'3. 24 小时变化':'3. 24-hour trend'}</h2><AccessHourlyChart rows={accessHourlyRows} zh={zh}/></section>}{accessBandRows.length>0&&<section className="decision-chapter"><h2>{zh?'4. 用户分布对比':'4. User cohort distributions'}</h2><AccessBandComparison rows={accessBandRows} zh={zh}/></section>}{accessAppRows.length>0&&<section className="decision-chapter"><h2>{zh?'5. 同 App 对比':'5. Same-App comparison'}</h2><AccessAppComparison rows={accessAppRows} zh={zh}/><Explanation>{zh?'只有同一 App 两侧样本均充分时才计算差体验差异；单侧缺失和样本不足不会被当成 0。':'A delta is calculated only when both access cohorts have sufficient samples; missing or insufficient data is not zero.'}</Explanation></section>}</>}
    {rows.length > 0 && view === 'opportunities' && <OpportunityPanel summaries={rows} c={c} />}
    {printReport && <FullPrintReportView report={printReport} batchName={c.batchDisplayName || c.importBatchId} runId={c.analysisRunId} zh={zh} />}
  </section>;
}

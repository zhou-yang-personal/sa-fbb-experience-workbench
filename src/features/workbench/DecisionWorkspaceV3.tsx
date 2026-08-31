import { useEffect, useMemo, useRef, useState } from 'react';
import type { MetricCard, OpportunityCandidatePage, OpportunityCandidateRow } from '../../shared/types';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import { parseMetricHint } from './analyticsStructuredCharts';
import { selectCsvSavePath } from './fileDialogs';
import type { WorkbenchController } from './useWorkbenchController';

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
      <tbody>{rows.map((row) => { const d = parseMetricHint(row.hint); const status = d.insight_status ?? 'UNCLASSIFIED'; const statusLabel = status === 'SEVERE' ? '严重' : status === 'PROBLEM' ? '问题' : status === 'WATCH' ? '关注' : status === 'NORMAL' ? '正常' : status === 'LIMITED' ? '有限样本' : '样本不足'; return <tr key={row.label} className={`insight-${status.toLowerCase()}`} role="button" tabIndex={0} onClick={() => void openDetail(row)} onKeyDown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); void openDetail(row); } }}><td><strong>{row.label}</strong><small>{d.app_category}</small></td><td>{Number(d.observed_users ?? row.value).toLocaleString()}</td><td>{friendlyNumber(d.traffic_gb ?? '0', 'GB')}</td><td>{friendlyNumber(d.duration_hours ?? 'NA', 'h')}</td><td>{Number(d.valid_obs_rows ?? 0).toLocaleString()}</td><td>{d.poor_observation_rate_pct === 'NA' ? '—' : `${Number(d.poor_observation_rate_pct).toFixed(2)}%`}</td><td>{d.persistent_poor_user_rate_pct === 'NA' ? '—' : `${Number(d.persistent_poor_user_rate_pct).toFixed(2)}%`}</td><td><span className={`insight-badge is-${status.toLowerCase()}`}>{statusLabel}</span></td></tr>; })}</tbody>
    </table></div>
    {selected && (() => { const d = parseMetricHint(selected.hint); return <div className="app-detail-modal-backdrop" role="presentation" onMouseDown={closeDetail}><section className="app-detail-modal" role="dialog" aria-modal="true" aria-labelledby="app-detail-title" aria-busy={detailLoading} onMouseDown={(event) => event.stopPropagation()}><header className="app-detail-modal-head"><div><span>{zh ? 'App 详情' : 'App detail'}</span><h2 id="app-detail-title">{selected.label}</h2><p>{zh ? '总体规模与体验 → 该 App 内用户分布' : 'Overall scale and experience → user distributions within this App'}</p></div><button type="button" autoFocus onClick={closeDetail}>{zh ? '关闭' : 'Close'}</button></header><div className="app-detail-modal-body"><article className="selected-app-detail"><div><p>{zh ? '先看该 App 的总体规模和体验，再看该 App 内部用户的流量、速率、时延、丢包和质差分布。' : 'Start with overall scale and experience, then inspect this App’s user distributions.'}</p></div><div className="detail-stat-grid"><span>{zh ? '用户' : 'Users'}<strong>{d.observed_users}</strong></span><span>{zh ? '流量' : 'Traffic'}<strong>{friendlyNumber(d.traffic_gb ?? '0', 'GB')}</strong></span><span>{zh ? '有效速率' : 'Effective rate'}<strong>{friendlyNumber(d.effective_download_mbps ?? 'NA', 'Mbps')}</strong></span><span>{zh ? '用户侧 RTT' : 'Subscriber RTT'}<strong>{friendlyNumber(d.subscriber_rtt_ms ?? 'NA', 'ms')}</strong></span><span>{zh ? '网络侧 RTT' : 'Network RTT'}<strong>{friendlyNumber(d.network_rtt_ms ?? 'NA', 'ms')}</strong></span><span>{zh ? '规则版本' : 'Rule version'}<strong>v{d.rule_version}</strong></span></div></article>{detailLoading ? <div className="app-detail-state is-loading"><strong>{zh ? '正在加载该 App 的用户分布…' : 'Loading user distributions…'}</strong><span>{zh ? '弹框会在结果返回后自动更新。' : 'This dialog will update when the result is ready.'}</span></div> : detailError ? <div className="app-detail-state is-error"><strong>{zh ? '详情加载失败' : 'Failed to load detail'}</strong><span>{detailError}</span><button type="button" onClick={() => void openDetail(selected)}>{zh ? '重试' : 'Retry'}</button></div> : detailRows.length > 0 ? <Distribution rows={detailRows} /> : <div className="app-detail-state"><strong>{zh ? '当前 App 没有可用的用户分布' : 'No user distribution is available'}</strong><span>{zh ? '可能是样本不足或对应聚合尚未生成。' : 'The sample may be insufficient or the aggregate may not be ready.'}</span></div>}</div></section></div>; })()}
  </>;
}

function Distribution({ rows }: { rows: MetricCard[] }) {
  const groups = useMemo(() => rows.reduce<Record<string, MetricCard[]>>((all, row) => { const key = parseMetricHint(row.hint).dimension ?? 'OTHER'; (all[key] ??= []).push(row); return all; }, {}), [rows]);
  const title: Record<string, string> = { TRAFFIC_DAILY: '日均流量分档用户分布', DURATION_EFFECTIVE_DAILY: '日均有效业务时长分档', DURATION_PEAK_DAILY: '日均高峰期有效时长分档', DURATION_GAME_ONLY: '独立 Game 时长分档', OBSERVATIONS_DAILY: '日均视频观测记录分档', EFFECTIVE_RATE: '视频有效下载速率分档', AVERAGE_DOWNLOAD_RATE: '平均下载速率分档', SUBSCRIBER_RTT: '用户侧时延分档', NETWORK_RTT: '网络侧时延分档', USER_LOSS: '用户侧丢包分档', NETWORK_LOSS: '网络侧丢包分档', EXPERIENCE: '持续质差用户分布' };
  return <div className="distribution-grid">{Object.entries(groups).map(([dimension, items]) => { const max = Math.max(1, ...items.map((item) => Number(item.value))); return <article className="distribution-card" key={dimension}><h3>{title[dimension] ?? dimension}</h3>{items.map((item) => <div className="distribution-row" key={item.label}><span>{item.label}</span><div><i style={{ width: `${Math.max(2, Number(item.value) / max * 100)}%` }} /></div><strong>{Number(item.value).toLocaleString()} 人</strong></div>)}<Explanation>每名按 IP 去重的分析用户只进入一个分档；Game 时长来自独立 Game 文件，未导入时不能解释为真实 0。</Explanation></article>; })}</div>;
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
    <section className="decision-chapter"><h2>{zh ? '机会概览' : 'Opportunity overview'}</h2><div className="opportunity-grid">{summaries.map((row) => { const d = parseMetricHint(row.hint); const unavailable = d.availability_status === 'UNAVAILABLE'; return <button type="button" key={row.label} className={`opportunity-card ${kind === row.label ? 'is-selected' : ''} ${unavailable ? 'is-unavailable' : ''}`} onClick={() => { setKind(kind === row.label ? '' : row.label); setPage(1); }}><span>{opportunityNames[row.label]?.[zh ? 0 : 1] ?? row.label}</span><strong>{unavailable ? (zh ? '不可用' : 'Unavailable') : `${Number(row.value).toLocaleString()} ${zh ? '人' : 'users'}`}</strong><p>{unavailable ? `${zh ? '数据限制' : 'Limitation'}：${d.data_limitation_code}` : `${zh ? '高优先级' : 'High priority'} ${Number(d.high_priority_users ?? 0).toLocaleString()} ${zh ? '人' : 'users'}`}</p><small>{zh ? '规则版本' : 'Rule version'} v{d.rule_version}</small></button>; })}</div><Explanation>{zh ? '四类机会只表示应用体验数据支持的候选方向，不等于可直接营销名单。点击卡片可筛选下方潜客。' : 'These are experience-driven candidates, not a CRM-qualified marketing list. Select a card to filter candidates.'}</Explanation></section>
    <section className="decision-chapter"><div className="opportunity-list-head"><div><h2>{zh ? '潜客列表' : 'Candidate list'}</h2><p>{zh ? `共 ${result.total.toLocaleString()} 条；以 IP 作为分析用户标识。` : `${result.total.toLocaleString()} candidates; IP is the analysis user identifier.`}</p></div><div className="opportunity-actions"><form onSubmit={(event) => { event.preventDefault(); setPage(1); setQueryKeyword(keyword.trim()); }}><input value={keyword} onChange={(event) => setKeyword(event.target.value)} placeholder={zh ? '搜索用户 IP 或主 App' : 'Search IP or primary App'} /><button type="submit">{zh ? '查询' : 'Search'}</button></form><button type="button" disabled={exporting || loading || result.total === 0} onClick={exportCandidates}>{exporting ? (zh ? '正在导出…' : 'Exporting…') : (zh ? '导出潜客明细 CSV' : 'Export candidate details CSV')}</button></div></div>
      {exportStatus && <p className="opportunity-export-status">{exportStatus}</p>}
      {error ? <div className="app-detail-state is-error"><strong>{zh ? '潜客列表加载失败' : 'Failed to load candidates'}</strong><span>{error}</span></div> : loading ? <div className="app-detail-state is-loading">{zh ? '正在读取已物化潜客…' : 'Loading materialized candidates…'}</div> : result.rows.length ? <div className="decision-table-wrap"><table className="decision-table opportunity-table"><thead><tr><th>{zh ? '用户 IP' : 'User IP'}</th><th>{zh ? '机会类型' : 'Type'}</th><th>{zh ? '优先级' : 'Priority'}</th><th>{zh ? '接入制式' : 'Access'}</th><th>{zh ? '活跃天数' : 'Active days'}</th><th>{zh ? '总流量' : 'Traffic'}</th><th>{zh ? '主 App' : 'Primary App'}</th><th>{zh ? '核心证据' : 'Evidence'}</th></tr></thead><tbody>{result.rows.map((row) => <tr key={`${row.user_key}-${row.opportunity_type}`} role="button" tabIndex={0} onClick={() => setSelected(row)} onKeyDown={(event) => { if (event.key === 'Enter') setSelected(row); }}><td><strong>{row.user_key}</strong></td><td>{opportunityNames[row.opportunity_type]?.[zh ? 0 : 1] ?? row.opportunity_type}</td><td><span className={`opportunity-level is-${row.opportunity_level.toLowerCase()}`}>{row.opportunity_level === 'HIGH' ? (zh ? '高' : 'High') : (zh ? '标准' : 'Standard')}</span></td><td>{row.user_type}</td><td>{row.active_days}</td><td>{friendlyNumber(String(row.total_download_gb), 'GB')}</td><td>{row.primary_app || '—'}</td><td>{row.evidence_value == null ? '—' : `${Number(row.evidence_value).toLocaleString(undefined, { maximumFractionDigits: 2 })} ${row.evidence_unit ?? ''}`}</td></tr>)}</tbody></table></div> : <div className="decision-empty">{zh ? '当前筛选下没有潜客。若全部为 0，请在数据作业中心从聚合阶段继续，页面本身不会启动重计算。' : 'No candidates match. Resume aggregation in Data Jobs if results have not been built.'}</div>}
      <div className="opportunity-pagination"><button type="button" disabled={page <= 1 || loading} onClick={() => setPage((value) => value - 1)}>{zh ? '上一页' : 'Previous'}</button><span>{page} / {pageCount}</span><button type="button" disabled={page >= pageCount || loading} onClick={() => setPage((value) => value + 1)}>{zh ? '下一页' : 'Next'}</button></div>
    </section>
    {selected && <div className="app-detail-modal-backdrop" role="presentation" onMouseDown={() => setSelected(null)}><section className="app-detail-modal opportunity-detail-modal" role="dialog" aria-modal="true" aria-labelledby="opportunity-detail-title" onMouseDown={(event) => event.stopPropagation()}><header className="app-detail-modal-head"><div><span>{zh ? '潜客证据详情' : 'Candidate evidence'}</span><h2 id="opportunity-detail-title">{selected.user_key}</h2><p>{opportunityNames[selected.opportunity_type]?.[zh ? 0 : 1]}</p></div><button type="button" autoFocus onClick={() => setSelected(null)}>{zh ? '关闭' : 'Close'}</button></header><div className="app-detail-modal-body"><div className="detail-stat-grid"><span>{zh ? '接入制式' : 'Access'}<strong>{selected.user_type}</strong></span><span>{zh ? '活跃天数' : 'Active days'}<strong>{selected.active_days}</strong></span><span>{zh ? '观测记录' : 'Observations'}<strong>{selected.observation_rows.toLocaleString()}</strong></span><span>{zh ? '总流量' : 'Traffic'}<strong>{friendlyNumber(String(selected.total_download_gb), 'GB')}</strong></span><span>{zh ? '有效时长' : 'Effective duration'}<strong>{friendlyNumber(String(selected.total_effective_duration_hours), 'h')}</strong></span><span>{zh ? '有效下载速率' : 'Effective rate'}<strong>{selected.avg_effective_download_mbps == null ? '—' : friendlyNumber(String(selected.avg_effective_download_mbps), 'Mbps')}</strong></span><span>{zh ? '主 App' : 'Primary App'}<strong>{selected.primary_app || '—'}</strong></span><span>{zh ? '主 App 活跃天数' : 'Primary App days'}<strong>{selected.primary_app_active_days}</strong></span><span>{zh ? '主 App 观测' : 'Primary App obs'}<strong>{selected.primary_app_observations.toLocaleString()}</strong></span><span>{zh ? 'Wi-Fi 时延' : 'Wi-Fi delay'}<strong>{selected.avg_wifi_delay_ms == null ? '—' : friendlyNumber(String(selected.avg_wifi_delay_ms), 'ms')}</strong></span><span>{zh ? '用户侧 RTT' : 'Subscriber RTT'}<strong>{selected.avg_subscriber_rtt_ms == null ? '—' : friendlyNumber(String(selected.avg_subscriber_rtt_ms), 'ms')}</strong></span><span>{zh ? '网络侧 RTT' : 'Network RTT'}<strong>{selected.avg_network_rtt_ms == null ? '—' : friendlyNumber(String(selected.avg_network_rtt_ms), 'ms')}</strong></span></div><article className="opportunity-evidence"><h3>{zh ? '入选证据' : 'Selection evidence'}</h3><p>{selected.evidence_summary}</p><small>{zh ? '规则版本' : 'Rule version'} v{selected.rule_profile_version}{selected.data_limitation_code ? ` · ${selected.data_limitation_code}` : ''}</small></article><Explanation>{zh ? '该详情解释“为什么进入候选池”。正式营销资格仍需 CRM、覆盖、套餐、欠费、黑名单和可触达性数据复核。' : 'This explains candidate selection. CRM, coverage, plan, arrears, blacklist and reachability are still required for marketing qualification.'}</Explanation></div></section></div>}
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
  const max=Math.max(1,...hourlyValues.map((item)=>item.value)); const min=Math.min(0,...hourlyValues.map((item)=>item.value)); const span=Math.max(max-min,.000001); const line=hourlyValues.map((item)=>`${24+item.hour*(712/23)},${18+(max-item.value)/span*180}`).join(' '); const appMax=Math.max(1,...ranked.map((item)=>item.value));
  const overallHint=overall?parseMetricHint(overall.hint):{}; const unavailableReason=overallHint.limitation??(zh?'当前发布结果没有可解释的有效样本。':'The current publication has no interpretable valid sample.');
  return <section className="metric-explorer"><div className="metric-explorer-tabs">{Object.entries(config).map(([key,item])=><button type="button" className={metric===key?'active':''} key={key} onClick={()=>setMetric(key)}>{item.label}</button>)}</div><div className="metric-explorer-grid"><article className="metric-focus-card"><span>{current.label}</span><strong>{overall?friendlyNumber(overall.value,current.unit):'—'}</strong><p>{overallHint.availability==='UNAVAILABLE'?(zh?`当前不可用：${unavailableReason}`:`Unavailable: ${unavailableReason}`):(zh?'当前批次整体口径；下方分别展示用户分布、小时变化和 App 排序。':'Batch-wide metric with user distribution, hourly trend and App ranking below.')}</p></article>{hourlyValues.length>0?<article className="metric-hourly-card"><h3>{zh?'24 小时变化':'24-hour trend'}</h3><svg viewBox="0 0 760 225"><rect width="760" height="225" fill="#fff"/><rect x={24+18*(712/23)} y="18" width={4*(712/23)} height="180" fill="#fff4d6"/><line x1="24" y1="198" x2="736" y2="198" stroke="#d0d5dd"/><polyline points={line} fill="none" stroke="#22705d" strokeWidth="3"/>{[0,6,12,18,23].map((hour)=><text key={hour} x={24+hour*(712/23)} y="218" textAnchor="middle" fontSize="10" fill="#667085">{hour}:00</text>)}</svg></article>:<article className="metric-hourly-card is-unavailable"><h3>{zh?'24 小时变化':'24-hour trend'}</h3><p>{zh?'当前指标尚无可靠小时聚合。':'No reliable hourly aggregate for this metric.'}</p></article>}</div>{bands.length>0&&<section><h3>{zh?'用户分档':'User distribution'}</h3><Distribution rows={bands}/></section>}{ranked.length>0&&<section><h3>{zh?'App 排序与问题高亮':'App ranking with issue highlighting'}</h3><div className="metric-app-ranking">{ranked.map(({row,value})=>{ const status=parseMetricHint(row.hint).insight_status??'INSUFFICIENT'; return <div className={`metric-app-rank is-${status.toLowerCase()}`} key={row.label}><span>{row.label}<small>{status}</small></span><div><i style={{width:`${Math.max(2,value/appMax*100)}%`}}/></div><strong>{friendlyNumber(String(value),current.unit)}</strong></div>; })}</div><Explanation>{zh?'排序展示所有有值 App；达到关注、问题或严重规则的 App 使用状态色高亮。高值是否更差由指标方向决定，不把规模大直接解释为质差。':'All Apps with values are ranked and issue statuses are highlighted. High scale is not automatically poor experience.'}</Explanation></section>}</section>;
}

function AccessHourlyChart({ rows, zh, initialMetric = 'users', fixed = false }: { rows: MetricCard[]; zh: boolean; initialMetric?: string; fixed?: boolean }) {
  const [metric, setMetric] = useState(initialMetric);
  const metrics: Record<string, [string,string]> = { users:[zh?'用户':'Users','users'], traffic_gb:[zh?'流量':'Traffic','GB'], average_download_mbps:[zh?'平均下载速率':'Average download rate','Mbps'], effective_download_mbps:[zh?'有效速率':'Effective rate','Mbps'], poor_observation_rate_pct:[zh?'差观测占比':'Poor obs rate','percent'], subscriber_rtt_ms:[zh?'用户侧 RTT':'Subscriber RTT','ms'], network_rtt_ms:[zh?'网络侧 RTT':'Network RTT','ms'], user_loss_pct:[zh?'用户侧丢包':'User loss','percent'], network_loss_pct:[zh?'网络侧丢包':'Network loss','percent'] };
  const series = ['CABLE','FTTH'].map((access) => ({ access, values: Array.from({length:24},(_,hour) => { const row=rows.find((item) => { const d=parseMetricHint(item.hint); return d.access_type===access && Number(d.hour)===hour; }); if (!row) return null; const d=parseMetricHint(row.hint); const value=metric==='users'?Number(row.value):Number(d[metric]); return Number.isFinite(value)?value:null; }) }));
  const all=series.flatMap((item)=>item.values).filter((value): value is number => value!=null); const max=Math.max(...all,1); const min=Math.min(...all,0); const span=Math.max(max-min,0.000001);
  const points=(values:Array<number|null>)=>values.map((value,hour)=>value==null?null:`${24+hour*(712/23)},${18+(max-value)/span*180}`).filter(Boolean).join(' ');
  return <section className="access-hourly">{fixed&&<h3>{metrics[metric][0]}</h3>}{!fixed&&<div className="access-metric-tabs">{Object.entries(metrics).map(([key,[label]])=><button type="button" className={metric===key?'active':''} key={key} onClick={()=>setMetric(key)}>{label}</button>)}</div>}<div className="access-chart-shell"><svg viewBox="0 0 760 230" role="img" aria-label={metrics[metric][0]}><rect x="0" y="0" width="760" height="230" fill="#fff"/><rect x={24+18*(712/23)} y="18" width={4*(712/23)} height="180" fill="#fff4d6"/><line x1="24" y1="198" x2="736" y2="198" stroke="#d0d5dd"/>{[0,6,12,18,23].map((hour)=><text key={hour} x={24+hour*(712/23)} y="218" textAnchor="middle" fontSize="10" fill="#667085">{hour}:00</text>)}<polyline points={points(series[0].values)} fill="none" stroke="#b54708" strokeWidth="3"/><polyline points={points(series[1].values)} fill="none" stroke="#087e8b" strokeWidth="3"/></svg><div className="access-chart-legend"><span className="is-cable">Cable</span><span className="is-ftth">FTTH</span><strong>{zh?'范围':'Range'} {friendlyNumber(String(min),metrics[metric][1])} – {friendlyNumber(String(max),metrics[metric][1])}</strong></div></div><Explanation>{zh?'同一小时、同一口径对比 Cable 与 FTTH；18:00–22:00 为高峰区间。曲线来自预聚合小时表，页面不会扫描 RAW/DWD。':'Cable and FTTH use the same hourly grain; 18:00–22:00 is highlighted. The page reads a pre-aggregated hourly table.'}</Explanation></section>;
}

function AccessBandComparison({ rows, zh }: { rows: MetricCard[]; zh: boolean }) {
  const groups=rows.reduce<Record<string,MetricCard[]>>((all,row)=>{ const key=parseMetricHint(row.hint).dimension??'OTHER'; (all[key]??=[]).push(row); return all; },{});
  const titles:Record<string,string>={TRAFFIC_DAILY:zh?'日均流量用户分布':'Daily traffic distribution',DURATION_EFFECTIVE_DAILY:zh?'日均有效时长用户分布':'Daily duration distribution',EFFECTIVE_RATE:zh?'有效下载速率用户分布':'Effective rate distribution',SUBSCRIBER_RTT:zh?'用户侧 RTT 用户分布':'Subscriber RTT distribution',NETWORK_RTT:zh?'网络侧 RTT 用户分布':'Network RTT distribution',USER_LOSS:zh?'用户侧丢包用户分布':'User loss distribution',NETWORK_LOSS:zh?'网络侧丢包用户分布':'Network loss distribution',EXPERIENCE:zh?'持续质差用户分布':'Persistent experience distribution'};
  return <div className="distribution-grid access-band-grid">{Object.entries(groups).map(([dimension,items])=>{ const bands=[...new Set(items.map((item)=>item.label))]; const max=Math.max(1,...items.map((item)=>Number(item.value))); return <article className="distribution-card" key={dimension}><h3>{titles[dimension]??dimension}</h3>{bands.map((band)=>{ const cable=items.find((item)=>item.label===band&&parseMetricHint(item.hint).access_type==='CABLE'); const ftth=items.find((item)=>item.label===band&&parseMetricHint(item.hint).access_type==='FTTH'); return <div className="access-band-row" key={band}><span>{band}</span><div><i className="is-cable" style={{width:`${Number(cable?.value??0)/max*100}%`}}/><em>{Number(cable?.value??0).toLocaleString()}</em></div><div><i className="is-ftth" style={{width:`${Number(ftth?.value??0)/max*100}%`}}/><em>{Number(ftth?.value??0).toLocaleString()}</em></div></div>; })}</article>; })}</div>;
}

function AccessAppComparison({ rows, zh }: { rows: MetricCard[]; zh: boolean }) {
  const apps=rows.reduce<Record<string,Record<string,MetricCard>>>((all,row)=>{ const access=parseMetricHint(row.hint).access_type??'UNKNOWN'; (all[row.label]??={})[access]=row; return all; },{});
  return <div className="decision-table-wrap"><table className="decision-table access-app-table"><thead><tr><th>App</th><th>Cable {zh?'用户':'users'}</th><th>FTTH {zh?'用户':'users'}</th><th>Cable {zh?'差观测':'poor obs'}</th><th>FTTH {zh?'差观测':'poor obs'}</th><th>{zh?'差异':'Delta'}</th><th>{zh?'样本':'Sample'}</th></tr></thead><tbody>{Object.entries(apps).slice(0,100).map(([app,sides])=>{ const c=sides.CABLE&&parseMetricHint(sides.CABLE.hint); const f=sides.FTTH&&parseMetricHint(sides.FTTH.hint); const cp=Number(c?.poor_observation_rate_pct); const fp=Number(f?.poor_observation_rate_pct); const comparable=c&&f&&c.sample_status==='SUFFICIENT'&&f.sample_status==='SUFFICIENT'&&Number.isFinite(cp)&&Number.isFinite(fp); return <tr key={app} className={comparable&&cp>fp?'comparison-worse':''}><td><strong>{app}</strong></td><td>{Number(c?.observed_users??0).toLocaleString()}</td><td>{Number(f?.observed_users??0).toLocaleString()}</td><td>{Number.isFinite(cp)?`${cp.toFixed(2)}%`:'—'}</td><td>{Number.isFinite(fp)?`${fp.toFixed(2)}%`:'—'}</td><td>{comparable?`${cp-fp>=0?'+':''}${(cp-fp).toFixed(2)} pct`:'—'}</td><td>{comparable?(zh?'可比':'Comparable'):(zh?'样本不足/单侧缺失':'Insufficient/missing side')}</td></tr>; })}</tbody></table></div>;
}

function FullPrintReportView({ report, batchName, runId }: { report: FullPrintReport; batchName: string; runId: string }) {
  return <section className="all-charts-print-report">
    <header><h1>SA FBB Experience Workbench</h1><p>全部洞察图表 · {batchName} · {runId}</p><small>生成时间：{new Date().toLocaleString()} · 数据源：已聚合 DWS / ADS</small></header>
    {report.metrics.length > 0 && <article><h2>1. 整体指标</h2><CardGrid rows={report.metrics} /></article>}
    {report.apps.length > 0 && <article><h2>2. App 全景（按分析用户覆盖 Top 40）</h2><PrintAppBars rows={report.apps} /><Explanation>唯一 App 粒度；完整明细保留在交互页面，PDF 默认只输出图形。</Explanation></article>}
    {report.users.length > 0 && <article><h2>3. 用户分布</h2><Distribution rows={report.users} /></article>}
    {report.quality.length > 0 && <article><h2>4. 质差证据</h2><CardGrid rows={report.quality} /></article>}
    {report.access.length > 0 && <article><h2>5. Cable / FTTH 专项图表</h2>{report.accessHourly.length > 0 && ['users','traffic_gb','average_download_mbps','effective_download_mbps','poor_observation_rate_pct','subscriber_rtt_ms','network_rtt_ms','user_loss_pct','network_loss_pct'].map((metric)=><AccessHourlyChart key={metric} rows={report.accessHourly} zh initialMetric={metric} fixed />)}{report.accessBands.length > 0 && <AccessBandComparison rows={report.accessBands} zh />}</article>}
    {report.opportunities.length > 0 && <article><h2>6. 潜客机会</h2><CardGrid rows={report.opportunities} /></article>}
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
    {rows.length > 0 && view === 'access' && <><section className="decision-chapter"><h2>{zh?'1. 整体规模与指标差异':'1. Overall scale and metric deltas'}</h2><AccessDeltaTable rows={rows} zh={zh}/><Explanation>{zh?'先比较总体规模和体验指标；差异只代表当前批次中两类用户群体的统计差异，不代表同一用户迁转前后的因果效果。':'Differences compare two cohorts in this batch; they are not causal before/after migration effects.'}</Explanation></section>{accessHourlyRows.length>0&&<section className="decision-chapter"><h2>{zh?'2. 24 小时变化':'2. 24-hour trend'}</h2><AccessHourlyChart rows={accessHourlyRows} zh={zh}/></section>}{accessBandRows.length>0&&<section className="decision-chapter"><h2>{zh?'3. 用户分布对比':'3. User cohort distributions'}</h2><AccessBandComparison rows={accessBandRows} zh={zh}/><Explanation>{zh?'每个分档并列展示 Cable 与 FTTH 用户数，先说明群体结构，再解释均值差异。':'Cable and FTTH counts are shown side by side to explain cohort structure before interpreting averages.'}</Explanation></section>}{accessAppRows.length>0&&<section className="decision-chapter"><h2>{zh?'4. 同 App 对比':'4. Same-App comparison'}</h2><AccessAppComparison rows={accessAppRows} zh={zh}/><Explanation>{zh?'只有同一 App 两侧样本均充分时才计算差体验差异；单侧缺失和样本不足不会被当成 0。':'A delta is calculated only when both access cohorts have sufficient samples; missing or insufficient data is not zero.'}</Explanation></section>}</>}
    {rows.length > 0 && view === 'opportunities' && <OpportunityPanel summaries={rows} c={c} />}
    {printReport && <FullPrintReportView report={printReport} batchName={c.batchDisplayName || c.importBatchId} runId={c.analysisRunId} />}
  </section>;
}

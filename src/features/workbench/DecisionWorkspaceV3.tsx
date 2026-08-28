import { useEffect, useMemo, useState } from 'react';
import type { MetricCard } from '../../shared/types';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import { parseMetricHint } from './analyticsStructuredCharts';
import type { WorkbenchController } from './useWorkbenchController';

export type DecisionView = 'panorama' | 'quality' | 'access' | 'opportunities';
type PanoramaPerspective = 'metric' | 'app' | 'user';
type FullPrintReport = {
  metrics: MetricCard[];
  apps: MetricCard[];
  users: MetricCard[];
  quality: MetricCard[];
  access: MetricCard[];
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
    if (number >= 1024) return `${(number / 1024).toFixed(2)} TB`;
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
  const zh = c.language === 'zh-CN';
  return <>
    <div className="decision-table-wrap"><table className="decision-table"><thead><tr><th>App</th><th>{zh ? '用户' : 'Users'}</th><th>{zh ? '流量' : 'Traffic'}</th><th>{zh ? '有效观测' : 'Valid obs'}</th><th>{zh ? '差观测' : 'Poor obs'}</th><th>{zh ? '持续质差用户' : 'Persistent users'}</th><th>{zh ? '状态' : 'Status'}</th></tr></thead>
      <tbody>{rows.map((row) => { const d = parseMetricHint(row.hint); const status = d.insight_status ?? 'UNCLASSIFIED'; const statusLabel = status === 'SEVERE' ? '严重' : status === 'PROBLEM' ? '问题' : status === 'WATCH' ? '关注' : status === 'NORMAL' ? '正常' : status === 'LIMITED' ? '有限样本' : '样本不足'; return <tr key={row.label} className={`insight-${status.toLowerCase()}`} onClick={async () => { setSelected(row); setDetailRows([]); c.applyAnalysisContext({ app_name: row.label }); setDetailLoading(true); try { setDetailRows(await analyticsStructuredApi.decisionUserDistributions(c.effectiveSettings, c.importBatchId, c.analysisRunId, { keyword: row.label })); } catch { setDetailRows([]); } finally { setDetailLoading(false); } }}><td><strong>{row.label}</strong><small>{d.app_category}</small></td><td>{Number(d.observed_users ?? row.value).toLocaleString()}</td><td>{friendlyNumber(d.traffic_gb ?? '0', 'GB')}</td><td>{Number(d.valid_obs_rows ?? 0).toLocaleString()}</td><td>{d.poor_observation_rate_pct === 'NA' ? '—' : `${Number(d.poor_observation_rate_pct).toFixed(2)}%`}</td><td>{d.persistent_poor_user_rate_pct === 'NA' ? '—' : `${Number(d.persistent_poor_user_rate_pct).toFixed(2)}%`}</td><td><span className={`insight-badge is-${status.toLowerCase()}`}>{statusLabel}</span></td></tr>; })}</tbody>
    </table></div>
    {selected && (() => { const d = parseMetricHint(selected.hint); return <><article className="selected-app-detail"><div><h3>{selected.label}</h3><p>{zh ? '先看该 App 的总体规模和体验，再看该 App 内部用户的流量、速率、时延、丢包和质差分布。' : 'Start with overall scale and experience, then inspect this App’s user distributions.'}</p></div><div className="detail-stat-grid"><span>用户<strong>{d.observed_users}</strong></span><span>流量<strong>{friendlyNumber(d.traffic_gb ?? '0', 'GB')}</strong></span><span>有效速率<strong>{friendlyNumber(d.effective_download_mbps ?? 'NA', 'Mbps')}</strong></span><span>用户侧 RTT<strong>{friendlyNumber(d.subscriber_rtt_ms ?? 'NA', 'ms')}</strong></span><span>网络侧 RTT<strong>{friendlyNumber(d.network_rtt_ms ?? 'NA', 'ms')}</strong></span><span>规则版本<strong>v{d.rule_version}</strong></span></div></article>{detailLoading ? <p className="decision-status">正在加载该 App 用户分布…</p> : detailRows.length > 0 ? <Distribution rows={detailRows} /> : null}</>; })()}
  </>;
}

function Distribution({ rows }: { rows: MetricCard[] }) {
  const groups = useMemo(() => rows.reduce<Record<string, MetricCard[]>>((all, row) => { const key = parseMetricHint(row.hint).dimension ?? 'OTHER'; (all[key] ??= []).push(row); return all; }, {}), [rows]);
  const title: Record<string, string> = { TRAFFIC: '流量分档用户分布', DURATION_GAME_ONLY: '使用时长分档（当前仅游戏时长）', OBSERVATIONS: '观测记录分档用户分布', EFFECTIVE_RATE: '视频有效下载速率分档', SUBSCRIBER_RTT: '用户侧时延分档', NETWORK_RTT: '网络侧时延分档', USER_LOSS: '用户侧丢包分档', NETWORK_LOSS: '网络侧丢包分档', EXPERIENCE: '持续质差用户分布' };
  return <div className="distribution-grid">{Object.entries(groups).map(([dimension, items]) => { const max = Math.max(1, ...items.map((item) => Number(item.value))); return <article className="distribution-card" key={dimension}><h3>{title[dimension] ?? dimension}</h3>{items.map((item) => <div className="distribution-row" key={item.label}><span>{item.label}</span><div><i style={{ width: `${Math.max(2, Number(item.value) / max * 100)}%` }} /></div><strong>{Number(item.value).toLocaleString()} 人</strong></div>)}<Explanation>每名 IP 用户只进入一个分档；“0 h”不等于零使用，表示当前 TCP 时长尚未进入本聚合。</Explanation></article>; })}</div>;
}

function PrintAppBars({ rows }: { rows: MetricCard[] }) {
  const printable = rows.slice(0, 40);
  const max = Math.max(1, ...printable.map((row) => Number(parseMetricHint(row.hint).observed_users ?? row.value)));
  return <div className="print-app-bars">{printable.map((row) => { const detail = parseMetricHint(row.hint); const value = Number(detail.observed_users ?? row.value); return <div className="distribution-row" key={row.label}><span>{row.label}</span><div><i style={{ width: `${Math.max(2, value / max * 100)}%` }} /></div><strong>{value.toLocaleString()} 人</strong></div>; })}</div>;
}

function FullPrintReportView({ report, batchName, runId }: { report: FullPrintReport; batchName: string; runId: string }) {
  return <section className="all-charts-print-report">
    <header><h1>SA FBB Experience Workbench</h1><p>全部洞察图表 · {batchName} · {runId}</p><small>生成时间：{new Date().toLocaleString()} · 数据源：已聚合 DWS / ADS</small></header>
    {report.metrics.length > 0 && <article><h2>1. 整体指标</h2><CardGrid rows={report.metrics} /></article>}
    {report.apps.length > 0 && <article><h2>2. App 全景（按分析用户覆盖 Top 40）</h2><PrintAppBars rows={report.apps} /><Explanation>唯一 App 粒度；完整明细保留在交互页面，PDF 默认只输出图形。</Explanation></article>}
    {report.users.length > 0 && <article><h2>3. 用户分布</h2><Distribution rows={report.users} /></article>}
    {report.quality.length > 0 && <article><h2>4. 质差证据</h2><CardGrid rows={report.quality} /></article>}
    {report.access.length > 0 && <article><h2>5. Cable / FTTH 专项</h2><CardGrid rows={report.access} /></article>}
    {report.opportunities.length > 0 && <article><h2>6. 潜客机会</h2><CardGrid rows={report.opportunities} /></article>}
  </section>;
}

export function DecisionWorkspaceV3({ c, view }: { c: WorkbenchController; view: DecisionView }) {
  const [perspective, setPerspective] = useState<PanoramaPerspective>('metric');
  const [rows, setRows] = useState<MetricCard[]>([]);
  const [status, setStatus] = useState('确认批次和分析运行后，点击一次加载。切换页面不会自动查询。');
  const [loading, setLoading] = useState(false);
  const [printReport, setPrintReport] = useState<FullPrintReport | null>(null);
  const zh = c.language === 'zh-CN';
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim() || loading;

  useEffect(() => {
    setRows([]);
    setStatus(c.importBatchId && c.analysisRunId ? '当前页面尚未读取；点击加载只查询已聚合结果。' : '请先选择可分析批次。');
  }, [view, c.importBatchId, c.analysisRunId]);

  async function load(targetPerspective = perspective) {
    if (disabled) return;
    setLoading(true); setStatus('正在读取已聚合结果…');
    try {
      const result = view === 'panorama'
        ? targetPerspective === 'metric' ? await analyticsStructuredApi.decisionMetricPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId)
          : targetPerspective === 'app' ? await analyticsStructuredApi.decisionAppPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 500 })
            : await analyticsStructuredApi.decisionUserDistributions(c.effectiveSettings, c.importBatchId, c.analysisRunId)
        : view === 'quality' ? await analyticsStructuredApi.decisionQualityOverview(c.effectiveSettings, c.importBatchId, c.analysisRunId)
          : view === 'access' ? await analyticsStructuredApi.decisionAccessCompare(c.effectiveSettings, c.importBatchId, c.analysisRunId)
            : await analyticsStructuredApi.decisionOpportunities(c.effectiveSettings, c.importBatchId, c.analysisRunId);
      setRows(result); setStatus(result.length ? `已加载 ${result.length} 项聚合证据。` : '结果为空；如果是潜客页，请先生成四类机会。');
    } catch (error) { setRows([]); setStatus(error instanceof Error ? error.message : String(error)); }
    finally { setLoading(false); }
  }

  async function generateOpportunities() {
    if (disabled) return;
    setLoading(true); setStatus('正在按已发布规则生成四类潜客…');
    try { await analyticsStructuredApi.decisionMaterializeOpportunities(c.effectiveSettings, c.importBatchId, c.analysisRunId); setRows(await analyticsStructuredApi.decisionOpportunities(c.effectiveSettings, c.importBatchId, c.analysisRunId)); setStatus('四类潜客已生成；AP 组网会根据数据可用性明确标记。'); }
    catch (error) { setStatus(error instanceof Error ? error.message : String(error)); }
    finally { setLoading(false); }
  }

  async function exportAllCharts() {
    if (disabled) return;
    setLoading(true); setStatus('正在准备全部非空图表的 PDF 报告…');
    try {
      const [metrics, apps, users, quality, access, opportunities] = await Promise.all([
        analyticsStructuredApi.decisionMetricPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionAppPanorama(c.effectiveSettings, c.importBatchId, c.analysisRunId, { pageSize: 500 }),
        analyticsStructuredApi.decisionUserDistributions(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionQualityOverview(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionAccessCompare(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.decisionOpportunities(c.effectiveSettings, c.importBatchId, c.analysisRunId),
      ]);
      setPrintReport({ metrics, apps, users, quality, access, opportunities });
      setStatus('全部图表已准备，正在打开系统 PDF 打印对话框。');
      const clear = () => setPrintReport(null);
      window.addEventListener('afterprint', clear, { once: true });
      window.setTimeout(() => requestAnimationFrame(() => requestAnimationFrame(() => window.print())), 0);
      window.setTimeout(clear, 60_000);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  function switchPerspective(next: PanoramaPerspective) { setPerspective(next); setRows([]); setStatus('已切换观察角度；点击加载读取对应聚合。'); }

  return <section className="decision-workspace">
    <header className="decision-page-head"><div><span>{zh ? '业务分析工作台' : 'Business analysis workspace'}</span><h1>{copy[view][zh ? 0 : 1]}</h1><p>{view === 'panorama' ? '先完整说明整体、App 和用户分布，再高亮值得关注的部分；不会提前跳到制式或网络定位。' : view === 'quality' ? '先说明质差规模，再区分用户侧、网络侧和应用体验证据；这里表达证据方向，不宣称根因。' : view === 'access' ? '在独立专项中比较 Cable 与 FTTH 的用户、流量、速率、时延和丢包，不污染总体全景。' : '只识别迁转、升套、AP 组网和特定 App Bundle 四类潜客，与体验修复和最终营销资格分开。'}</p></div>
      <div className="decision-load-actions"><button type="button" className="primary" disabled={disabled} onClick={() => load()}>{loading ? '加载中…' : '加载当前页面'}</button>{view === 'opportunities' && <button type="button" disabled={disabled} onClick={generateOpportunities}>按规则生成潜客</button>}<button type="button" disabled={disabled} onClick={exportAllCharts}>{zh ? '导出全部图表 PDF' : 'Export all charts PDF'}</button></div></header>
    <div className="decision-status">{status}</div>
    {view === 'panorama' && <div className="perspective-tabs"><button className={perspective === 'metric' ? 'active' : ''} onClick={() => switchPerspective('metric')}>指标视角</button><button className={perspective === 'app' ? 'active' : ''} onClick={() => switchPerspective('app')}>App 视角</button><button className={perspective === 'user' ? 'active' : ''} onClick={() => switchPerspective('user')}>用户视角</button></div>}
    {!rows.length && <article className="decision-empty"><strong>{c.importBatchId ? '等待加载' : '先选择数据批次'}</strong><p>{c.importBatchId ? '页面只查询已经生成的 DWS/ADS；不会启动 RAW 扫描或后台大任务。' : '到“数据中心”选择已有批次或导入新文件。'}</p></article>}
    {rows.length > 0 && view === 'panorama' && perspective === 'metric' && <><CardGrid rows={rows} /><Explanation>指标视角回答“整体发生了什么”。百分比保留分子、分母和样本量；不可用与 0 分开。vMOS 作为 App 体验证据，不作为单独一级指标。</Explanation></>}
    {rows.length > 0 && view === 'panorama' && perspective === 'app' && <><AppTable rows={rows} c={c} /><Explanation>每个 App 只出现一次，先展示跨制式总体规模；问题、关注、正常和样本不足互斥，因此分类数量之和等于全部 App 数。</Explanation></>}
    {rows.length > 0 && view === 'panorama' && perspective === 'user' && <Distribution rows={rows} />}
    {rows.length > 0 && view === 'quality' && <><CardGrid rows={rows} /><Explanation>同一用户可能同时存在多类证据，因此各类用户数不能相加当作总质差用户。网络侧证据不等于网络设备根因；BRAS/OLT/PON 定位被移到后续按需调查。</Explanation></>}
    {rows.length > 0 && view === 'access' && <div className="access-compare-grid">{rows.map((row) => { const d = parseMetricHint(row.hint); return <article key={row.label} className="access-column"><h2>{row.label}</h2><strong>{Number(row.value).toLocaleString()} 用户</strong><dl><div><dt>流量</dt><dd>{friendlyNumber(d.traffic_gb ?? '0', 'GB')}</dd></div><div><dt>视频有效下载速率</dt><dd>{friendlyNumber(d.effective_download_mbps ?? 'NA', 'Mbps')}</dd></div><div><dt>差观测占比</dt><dd>{friendlyNumber(d.poor_observation_rate_pct ?? 'NA', 'percent')}</dd></div><div><dt>持续质差用户占比</dt><dd>{friendlyNumber(d.persistent_poor_user_rate_pct ?? 'NA', 'percent')}</dd></div><div><dt>用户侧 RTT</dt><dd>{friendlyNumber(d.subscriber_rtt_ms ?? 'NA', 'ms')}</dd></div><div><dt>网络侧 RTT</dt><dd>{friendlyNumber(d.network_rtt_ms ?? 'NA', 'ms')}</dd></div></dl></article>; })}</div>}
    {rows.length > 0 && view === 'opportunities' && <div className="opportunity-grid">{rows.map((row) => { const d = parseMetricHint(row.hint); const names: Record<string, string> = { MIGRATION: 'Cable → FTTH 迁转', SPEED_UPGRADE: '宽带升套', MESH_AP: 'AP / Mesh 组网', APP_BUNDLE: '特定 App Bundle' }; return <article key={row.label} className={`opportunity-card ${d.availability_status === 'UNAVAILABLE' ? 'is-unavailable' : ''}`}><span>{names[row.label] ?? row.label}</span><strong>{d.availability_status === 'UNAVAILABLE' ? '不可用' : `${Number(row.value).toLocaleString()} 人`}</strong><p>{d.availability_status === 'UNAVAILABLE' ? `数据限制：${d.data_limitation_code}` : `高优先级 ${Number(d.high_priority_users ?? 0).toLocaleString()} 人`}</p><small>规则版本 v{d.rule_version}</small></article>; })}</div>}
    {printReport && <FullPrintReportView report={printReport} batchName={c.batchDisplayName || c.importBatchId} runId={c.analysisRunId} />}
  </section>;
}

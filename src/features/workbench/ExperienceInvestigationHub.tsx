import { useEffect, useMemo, useState } from 'react';
import type { DataCoverageItemV2, ExperienceFinding, ExperienceStatusV2, InvestigationEvidenceRow, InvestigationHourlyRow, InvestigationServerIpRow, RunVerificationV2, SavedInvestigation } from '../../shared/types';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import type { WorkbenchController } from './useWorkbenchController';
import './ExperienceInvestigationHub.css';

export type InvestigationHubView = 'overview' | 'findings' | 'investigation' | 'investigations';

function rate(value?: number) {
  return value === undefined || value === null ? '—' : `${value.toFixed(2)}%`;
}

function number(value: number) {
  return new Intl.NumberFormat().format(value);
}

function deltaRate(value?: number) {
  if (value === undefined || value === null) return '—';
  return `${value > 0 ? '+' : ''}${value.toFixed(2)}pct`;
}

function coverageLabel(item: DataCoverageItemV2, zh: boolean) {
  const labels: Record<string, [string, string]> = {
    TCP: ['TCP 数据', 'TCP data'], GAME: ['游戏数据', 'Game data'], APP_EXPERIENCE_V2: ['体验聚合 V2', 'Experience aggregate V2'],
    NETWORK_TOPOLOGY: ['网络拓扑', 'Network topology'], SERVER_IP: ['服务端 IP', 'Server IP'], IDENTITY: ['用户身份', 'User identity'],
  };
  return labels[item.dimension]?.[zh ? 0 : 1] ?? item.dimension;
}

function statusText(status: string, zh: boolean) {
  const labels: Record<string, [string, string]> = {
    AVAILABLE: ['可用', 'Available'], NOT_IMPORTED: ['未导入', 'Not imported'], UNAVAILABLE: ['不可用', 'Unavailable'],
    LIMITED: ['能力有限', 'Limited'], INSUFFICIENT_SAMPLE: ['样本不足', 'Insufficient sample'],
  };
  return labels[status]?.[zh ? 0 : 1] ?? status;
}

export function ExperienceInvestigationHub({ c, view, onNavigate }: { c: WorkbenchController; view: InvestigationHubView; onNavigate: (view: InvestigationHubView) => void }) {
  const zh = c.language === 'zh-CN';
  const [status, setStatus] = useState<ExperienceStatusV2 | null>(null);
  const [findings, setFindings] = useState<ExperienceFinding[]>([]);
  const [coverage, setCoverage] = useState<DataCoverageItemV2[]>([]);
  const [verification, setVerification] = useState<RunVerificationV2 | null>(null);
  const [evidence, setEvidence] = useState<InvestigationEvidenceRow[]>([]);
  const [hourly, setHourly] = useState<InvestigationHourlyRow[]>([]);
  const [serverIps, setServerIps] = useState<InvestigationServerIpRow[]>([]);
  const [saved, setSaved] = useState<SavedInvestigation[]>([]);
  const [loadedContext, setLoadedContext] = useState('');
  const [evidenceLoadedContext, setEvidenceLoadedContext] = useState('');
  const [message, setMessage] = useState(zh ? '等待加载。' : 'Ready to load.');
  const contextKey = `${c.importBatchId}::${c.analysisRunId}`;
  const investigationKey = `${contextKey}::${JSON.stringify(c.analysisContext)}`;
  const selectedFinding = useMemo(() => findings.find((item) => item.finding_id === c.analysisContext.finding_id), [findings, c.analysisContext.finding_id]);
  const visibleFindings = useMemo(() => findings.filter((item) =>
    (!c.analysisContext.app_category || item.app_category === c.analysisContext.app_category)
    && (!c.analysisContext.app_name || item.app_name === c.analysisContext.app_name)
    && (!c.analysisContext.access_type || item.access_type === c.analysisContext.access_type)
    && (!c.analysisContext.issue_side || item.issue_side === c.analysisContext.issue_side)
  ), [findings, c.analysisContext.app_category, c.analysisContext.app_name, c.analysisContext.access_type, c.analysisContext.issue_side]);
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim() || Boolean(c.currentAction);

  useEffect(() => {
    setStatus(null); setFindings([]); setCoverage([]); setVerification(null); setEvidence([]); setHourly([]); setServerIps([]); setSaved([]);
    setLoadedContext(''); setEvidenceLoadedContext('');
  }, [contextKey]);

  async function loadFoundation() {
    if (disabled) return;
    const result = await c.runAction('analytics_load_experience_v2', async () => {
      const [nextStatus, nextFindings, nextCoverage, nextVerification] = await Promise.all([
        analyticsStructuredApi.experienceStatusV2(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.findingsV2(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.coverageV2(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.runVerificationV2(c.effectiveSettings, c.importBatchId, c.analysisRunId),
      ]);
      return { nextStatus, nextFindings, nextCoverage, nextVerification };
    }) as { nextStatus: ExperienceStatusV2; nextFindings: ExperienceFinding[]; nextCoverage: DataCoverageItemV2[]; nextVerification: RunVerificationV2 } | null;
    if (!result) return;
    setStatus(result.nextStatus); setFindings(result.nextFindings); setCoverage(result.nextCoverage); setVerification(result.nextVerification); setLoadedContext(contextKey);
    setMessage(zh ? `已生成 ${result.nextFindings.length} 条可解释发现。` : `${result.nextFindings.length} explainable findings loaded.`);
  }

  async function loadEvidence() {
    if (disabled) return;
    const result = await c.runAction('analytics_load_investigation_evidence', async () => {
      const [users, hours, ips] = await Promise.all([
        analyticsStructuredApi.investigationEvidence(c.effectiveSettings, c.importBatchId, c.analysisRunId, c.analysisContext, 200),
        analyticsStructuredApi.investigationHourly(c.effectiveSettings, c.importBatchId, c.analysisRunId, c.analysisContext, 1000),
        c.analysisContext.app_name ? analyticsStructuredApi.investigationServerIps(c.effectiveSettings, c.importBatchId, c.analysisRunId, c.analysisContext, 50) : Promise.resolve([]),
      ]);
      return { users, hours, ips };
    }) as { users: InvestigationEvidenceRow[]; hours: InvestigationHourlyRow[]; ips: InvestigationServerIpRow[] } | null;
    if (result) {
      setEvidence(result.users); setHourly(result.hours); setServerIps(result.ips);
      setEvidenceLoadedContext(investigationKey);
      setMessage(zh ? `已加载 ${result.users.length} 条用户证据、${result.hours.length} 条小时证据和 ${result.ips.length} 个 Server IP。` : `${result.users.length} user rows, ${result.hours.length} hourly rows and ${result.ips.length} Server IPs loaded.`);
    }
  }

  async function loadSaved() {
    if (disabled) return;
    const result = await c.runAction('investigation_list', () => analyticsStructuredApi.investigations(c.effectiveSettings, c.importBatchId, c.analysisRunId));
    if (Array.isArray(result)) setSaved(result as SavedInvestigation[]);
  }

  function openFinding(finding: ExperienceFinding) {
    c.applyAnalysisContext({ finding_id: finding.finding_id, app_category: finding.app_category, app_name: finding.app_name, access_type: finding.access_type, issue_metric: finding.issue_metric, issue_side: finding.issue_side, baseline_type: 'POLICY_THRESHOLD' });
    setEvidence([]); setHourly([]); setServerIps([]); setEvidenceLoadedContext('');
    onNavigate('investigation');
  }

  async function saveInvestigation() {
    const title = selectedFinding?.[zh ? 'title_zh' : 'title_en'] ?? `${c.analysisContext.app_name ?? 'Experience'} investigation`;
    const result = await c.runAction('investigation_save', () => analyticsStructuredApi.saveInvestigation(c.effectiveSettings, c.importBatchId, c.analysisRunId, title, c.analysisContext));
    if (result) setMessage(zh ? '调查已保存，可在“已保存调查”继续。' : 'Investigation saved.');
  }

  function restoreInvestigation(item: SavedInvestigation) {
    try { c.applyAnalysisContext(JSON.parse(item.context_json)); } catch { c.applyAnalysisContext({ finding_id: item.finding_id }); }
    onNavigate('investigation');
  }

  const loadGate = loadedContext !== contextKey;
  const statusCards = status ? [
    [zh ? '有效分析用户' : 'Eligible users', number(status.eligible_users), zh ? '满足最低观测规则的去重用户。' : 'Distinct users meeting minimum-observation rules.'],
    [zh ? '差体验观测占比' : 'Poor observation rate', rate(status.poor_observation_rate_pct), `${number(status.poor_observations)} / ${number(status.valid_observations)}`],
    [zh ? '曾受影响用户占比' : 'Ever affected user rate', rate(status.ever_affected_user_rate_pct), `${number(status.ever_affected_users)} / ${number(status.eligible_users)}`],
    [zh ? '持续差体验用户占比' : 'Persistent poor user rate', rate(status.persistent_poor_user_rate_pct), `${number(status.persistent_poor_users)} / ${number(status.eligible_users)}`],
    [zh ? '严重差体验用户占比' : 'Severe poor user rate', rate(status.severe_poor_user_rate_pct), `${number(status.severe_poor_users)} / ${number(status.eligible_users)}`],
  ] : [];

  if (view === 'investigations') return <section className="experience-hub">
    <header className="workspace-page-header"><div><p className="eyebrow">INVESTIGATIONS</p><h2>{zh ? '已保存调查' : 'Saved investigations'}</h2><p>{zh ? '保存的是分析上下文和 Finding 引用，不复制大表数据。' : 'Saved state keeps context and finding references, not large-table copies.'}</p></div><button className="primary-button" disabled={disabled} onClick={loadSaved}>{zh ? '加载调查' : 'Load investigations'}</button></header>
    <div className="investigation-list">{saved.map((item) => <article key={item.investigation_id}><div><span>{item.status}</span><h3>{item.title}</h3><p>{item.analysis_run_id} · {item.updated_at}</p></div><button onClick={() => restoreInvestigation(item)}>{zh ? '继续调查' : 'Continue'}</button></article>)}{!saved.length && <p className="empty-panel">{zh ? '尚未加载或没有已保存调查。' : 'No saved investigations loaded.'}</p>}</div>
  </section>;

  if (view === 'investigation') return <section className="experience-hub investigation-workspace-v2">
    <header className="workspace-page-header"><div><p className="eyebrow">INVESTIGATION WORKSPACE</p><h2>{selectedFinding?.[zh ? 'title_zh' : 'title_en'] ?? (zh ? '问题调查' : 'Issue investigation')}</h2><p>{zh ? '沿用顶部分析路径，逐层确认范围、证据和下一步。' : 'Keep the shared analysis path while validating scope, evidence and next action.'}</p></div><div className="hub-actions"><button disabled={disabled} onClick={saveInvestigation}>{zh ? '保存调查' : 'Save'}</button><button className="primary-button" disabled={disabled} onClick={loadEvidence}>{evidenceLoadedContext === investigationKey ? (zh ? '重新加载证据' : 'Reload evidence') : (zh ? '加载调查证据' : 'Load evidence')}</button></div></header>
    <div className="investigation-columns">
      <article className="panel"><span className="panel-kicker">{zh ? '影响范围' : 'Scope'}</span><h3>{c.analysisContext.app_name ?? (zh ? '未限定 App' : 'All apps')}</h3><dl><div><dt>{zh ? '接入制式' : 'Access'}</dt><dd>{c.analysisContext.access_type ?? 'ALL'}</dd></div><div><dt>{zh ? '问题侧' : 'Issue side'}</dt><dd>{c.analysisContext.issue_side ?? (zh ? '证据不足' : 'Insufficient evidence')}</dd></div><div><dt>{zh ? '样本' : 'Sample'}</dt><dd>{selectedFinding ? number(selectedFinding.sample_size) : '—'}</dd></div></dl></article>
      <article className="panel"><span className="panel-kicker">{zh ? '体验证据' : 'Experience evidence'}</span><h3>{selectedFinding?.main_driver ?? c.analysisContext.issue_metric ?? (zh ? '待下钻' : 'Pending drill-down')}</h3><dl><div><dt>{zh ? '持续用户' : 'Persistent users'}</dt><dd>{selectedFinding ? number(selectedFinding.affected_users) : '—'}</dd></div><div><dt>{zh ? '差观测率' : 'Poor obs rate'}</dt><dd>{rate(selectedFinding?.poor_observation_rate_pct)}</dd></div><div><dt>{zh ? '严重用户率' : 'Severe user rate'}</dt><dd>{rate(selectedFinding?.severe_user_rate_pct)}</dd></div></dl></article>
      <article className="panel action-judgement"><span className="panel-kicker">{zh ? '判断与行动' : 'Judgement & action'}</span><h3>{selectedFinding?.issue_side ?? (zh ? '当前证据不足' : 'Evidence insufficient')}</h3><p>{selectedFinding?.recommended_next_step ?? (zh ? '先加载受控下钻证据，再决定网络侧检查、家庭侧优化、继续观察或机会复核。' : 'Load controlled evidence before choosing network checks, household optimization, observation or opportunity review.')}</p><small>{zh ? '这不是已确认根因。' : 'This is not a confirmed root cause.'}</small></article>
    </div>
    <section className="hourly-pivot-v2"><header><div><h3>{zh ? '时间范围下钻' : 'Time pivot'}</h3><p>{zh ? '仅展示满足所选 App / 接入条件的小时 ADS；样本不足不会进入异常判断。' : 'Hourly ADS for the selected app/access scope; insufficient samples are excluded from anomaly judgement.'}</p></div></header><div className="hourly-pivot-grid">{hourly.map((row) => <button type="button" className={row.sample_status === 'SUFFICIENT' ? '' : 'is-insufficient'} key={`${row.stat_date}-${row.hour_of_day}-${row.access_type}`} onClick={() => c.applyAnalysisContext({ date_from: row.stat_date, date_to: row.stat_date, hour_from: row.hour_of_day, hour_to: row.hour_of_day, access_type: row.access_type })}><span>{row.stat_date} · {String(row.hour_of_day).padStart(2, '0')}:00</span><strong>{rate(row.poor_observation_rate_pct)}</strong><small>{row.access_type} · {number(row.eligible_users)} {zh ? '用户' : 'users'} · {statusText(row.sample_status, zh)}</small></button>)}{!hourly.length && <p className="empty-panel">{zh ? '加载调查证据后显示时间分布。' : 'Load investigation evidence to see time distribution.'}</p>}</div></section>
    <section className="evidence-table-v2"><header><h3>{zh ? '受影响用户证据' : 'Affected-user evidence'}</h3><span>{message}</span></header><div className="evidence-grid evidence-grid-head"><span>{zh ? '用户' : 'User'}</span><span>{zh ? '有效/差观测' : 'Valid / poor'}</span><span>{zh ? '差观测率' : 'Poor rate'}</span><span>vMOS</span><span>{zh ? '用户/网络 RTT' : 'User / network RTT'}</span><span>{zh ? '用户/网络丢包' : 'User / network loss'}</span></div>{evidence.map((row) => <button type="button" className="evidence-grid" key={`${row.user_key}-${row.app_name}`} onClick={() => c.applyAnalysisContext({ user_key: row.user_key })}><span><strong>{row.user_key}</strong><small>{row.access_type} · {row.app_name}</small></span><span>{row.valid_obs_rows} / {row.poor_obs_rows}</span><span>{rate(row.poor_observation_rate_pct)}</span><span>{row.avg_vmos?.toFixed(2) ?? '—'}</span><span>{row.avg_subscriber_rtt_ms?.toFixed(1) ?? '—'} / {row.avg_network_rtt_ms?.toFixed(1) ?? '—'}</span><span>{row.avg_user_loss_pct?.toFixed(3) ?? '—'} / {row.avg_network_loss_pct?.toFixed(3) ?? '—'}</span></button>)}{!evidence.length && <p className="empty-panel">{zh ? '点击“加载调查证据”，查询只读取 DWS，不扫描 RAW。' : 'Load evidence; the query reads DWS and never scans RAW.'}</p>}</section>
    <section className="server-ip-evidence-v2"><header><div><h3>{zh ? 'Server IP / 内容路径证据' : 'Server IP / content-path evidence'}</h3><p>{zh ? '仅对当前 App 的优先受影响用户做受控解析：最多 200 名用户、2 万条 DWD 观测；不全量拆分，也不代表已确认内容源根因。' : 'Controlled parsing for the current App: at most 200 priority users and 20,000 DWD observations; no full explosion and no confirmed content-source root cause.'}</p></div></header><div className="server-ip-grid server-ip-grid-head"><span>Server IP</span><span>{zh ? '观测用户' : 'Observed users'}</span><span>{zh ? '观测行' : 'Rows'}</span><span>{zh ? '用户/网络 RTT' : 'User / network RTT'}</span><span>{zh ? '用户/网络丢包' : 'User / network loss'}</span></div>{serverIps.map((row) => <button type="button" className="server-ip-grid" key={row.server_ip} onClick={() => c.applyAnalysisContext({ server_ip: row.server_ip })}><strong>{row.server_ip}</strong><span>{number(row.observed_users)}</span><span>{number(row.observation_rows)}</span><span>{row.avg_subscriber_rtt_ms?.toFixed(1) ?? '—'} / {row.avg_network_rtt_ms?.toFixed(1) ?? '—'}</span><span>{row.avg_user_loss_pct?.toFixed(3) ?? '—'} / {row.avg_network_loss_pct?.toFixed(3) ?? '—'}</span></button>)}{!serverIps.length && <p className="empty-panel">{c.analysisContext.app_name ? (zh ? '当前范围没有可用 Server IP；旧 CLEAN 结果需要用 1.0.50 重跑后才会保留该字段。' : 'No Server IP is available in this scope; older CLEAN results must be rebuilt with 1.0.50 to retain this field.') : (zh ? '先从 Finding 选择一个 App。' : 'Select an App from a Finding first.')}</p>}</section>
  </section>;

  return <section className="experience-hub">
    <header className="workspace-page-header"><div><p className="eyebrow">{view === 'overview' ? 'EXPERIENCE STATUS' : 'AUTO FINDINGS'}</p><h2>{view === 'overview' ? (zh ? '体验健康总览' : 'Experience health overview') : (zh ? '自动发现' : 'Auto findings')}</h2><p>{zh ? '先看状态和可信范围，再从发现进入同一上下文的调查。' : 'Review status and coverage, then investigate findings in the same context.'}</p></div><button className="primary-button" disabled={disabled} onClick={loadFoundation}>{loadGate ? (zh ? '加载 V2 分析' : 'Load V2 analysis') : (zh ? '重新加载' : 'Reload')}</button></header>
    {loadGate ? <section className="analytics-load-gate"><div><h3>{zh ? '不会默认发起大查询' : 'No automatic large query'}</h3><p>{zh ? '点击后并行读取小型 DWS/ADS 状态、Finding 和覆盖信息，不扫描 RAW。' : 'This reads small DWS/ADS status, findings and coverage in parallel; RAW is not scanned.'}</p></div></section> : <>
      {view === 'overview' && <><div className="experience-status-grid">{statusCards.map(([label, value, hint]) => <article key={label}><span>{label}</span><strong>{value}</strong><small>{hint}</small></article>)}</div><section className="hub-section"><header><div><h3>{zh ? '上一可比运行复验' : 'Previous comparable-run verification'}</h3><p>{zh ? '仅在接入规则、Others、App 映射和体验策略版本一致时比较；负向变化表示改善。' : 'Comparison requires identical access, Others, App-mapping and experience-policy versions; a negative delta means improvement.'}</p></div></header>{verification?.comparable ? <div className="verification-grid-v2"><article><span>{zh ? '差体验观测占比变化' : 'Poor observation change'}</span><strong>{deltaRate(verification.poor_observation_rate_delta_pct)}</strong><small>{rate(verification.previous_poor_observation_rate_pct)} → {rate(verification.current_poor_observation_rate_pct)}</small></article><article><span>{zh ? '持续差体验用户占比变化' : 'Persistent-user change'}</span><strong>{deltaRate(verification.persistent_poor_user_rate_delta_pct)}</strong><small>{rate(verification.previous_persistent_poor_user_rate_pct)} → {rate(verification.current_persistent_poor_user_rate_pct)}</small></article><article><span>{zh ? '严重差体验用户占比变化' : 'Severe-user change'}</span><strong>{deltaRate(verification.severe_poor_user_rate_delta_pct)}</strong><small>{rate(verification.previous_severe_poor_user_rate_pct)} → {rate(verification.current_severe_poor_user_rate_pct)}</small></article></div> : <p className="empty-panel">{verification ? (zh ? '没有找到规则版本完全一致且具备 V2 结果的上一运行，当前不做不可比的趋势结论。' : 'No earlier run has identical rule versions and usable V2 results; no incomparable trend is shown.') : '—'}</p>}<small className="verification-reason">{verification?.comparison_reason}</small></section><section className="hub-section"><header><div><h3>{zh ? '优先调查' : 'Priority findings'}</h3><p>{zh ? '不是 Top N，而是满足样本、持续性和严重性规则的异常。' : 'Rule-qualified anomalies, not a generic Top N.'}</p></div><button onClick={() => onNavigate('findings')}>{zh ? '查看全部' : 'View all'}</button></header><FindingList findings={visibleFindings.slice(0, 6)} zh={zh} onOpen={openFinding} /></section><section className="hub-section"><header><div><h3>{zh ? '数据覆盖' : 'Data coverage'}</h3><p>{zh ? '明确区分可用、未导入、不可用和能力有限。' : 'Available, not imported, unavailable and limited are distinct.'}</p></div></header><div className="coverage-grid-v2">{coverage.map((item) => <article key={item.dimension} className={`coverage-${item.status.toLowerCase()}`}><span>{coverageLabel(item, zh)}</span><strong>{statusText(item.status, zh)}</strong><small>{item.coverage_pct == null ? item.limitation : `${item.coverage_pct.toFixed(1)}% · ${item.limitation ?? ''}`}</small></article>)}</div></section></>}
      {view === 'findings' && <FindingList findings={visibleFindings} zh={zh} onOpen={openFinding} />}
    </>}
  </section>;
}

function FindingList({ findings, zh, onOpen }: { findings: ExperienceFinding[]; zh: boolean; onOpen: (finding: ExperienceFinding) => void }) {
  return <div className="finding-list-v2">{findings.map((finding) => <article key={finding.finding_id} className={`finding-${finding.severity.toLowerCase()}`}><header><span>{finding.severity} · {finding.confidence}</span><small>{finding.finding_id}</small></header><h3>{zh ? finding.title_zh : finding.title_en}</h3><p>{zh ? `影响 ${number(finding.affected_users)} / ${number(finding.denominator)} 名合格用户，持续差体验占比 ${rate(finding.affected_user_rate_pct)}。` : `${number(finding.affected_users)} / ${number(finding.denominator)} eligible users affected; persistent rate ${rate(finding.affected_user_rate_pct)}.`}</p><dl><div><dt>{zh ? '差观测率' : 'Poor obs rate'}</dt><dd>{rate(finding.poor_observation_rate_pct)}</dd></div><div><dt>{zh ? '严重用户率' : 'Severe user rate'}</dt><dd>{rate(finding.severe_user_rate_pct)}</dd></div><div><dt>{zh ? '规则版本' : 'Rule version'}</dt><dd>v{finding.rule_version}</dd></div></dl><button className="primary-button" onClick={() => onOpen(finding)}>{zh ? '进入调查' : 'Investigate'}</button></article>)}{!findings.length && <p className="empty-panel">{zh ? '当前没有满足最低样本与 Finding 门槛的问题，或尚未加载。' : 'No rule-qualified findings, or data is not loaded.'}</p>}</div>;
}

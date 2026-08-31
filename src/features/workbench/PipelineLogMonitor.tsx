import { useEffect, useMemo, useState } from 'react';
import { formatLocalDateTime, localTimeZone } from '../../shared/localDateTime';
import type { ImportPipelineLogRow, ImportPipelineStatus } from '../../shared/types';

type Props = {
  logs: ImportPipelineLogRow[];
  status: ImportPipelineStatus | null;
  polling: boolean;
  pollingError: string;
  lastPollAt: number | null;
  lastLogReceivedAt: number | null;
  autoRefreshEnabled: boolean;
  onAutoRefreshChange: (enabled: boolean) => void;
  onRefresh: () => Promise<unknown> | void;
};

const stepLabels: Record<string, string> = {
  start: '执行计划',
  prepare_resume: '复用批次检查',
  prepare_rebuild: 'RAW 重建检查',
  prepare_environment: '导入准备',
  probe_csv: 'CSV 探测',
  import_current_file_atomic: '字段映射与 RAW 入库',
  raw_quality_gate: 'CLEAN 质量验证',
  raw_to_clean: 'CLEAN/DWD 生成',
  dws_ads_aggregate: 'DWS/ADS 聚合',
  final_fusion_optional: 'Final Lead 融合',
  module_ready: 'Module Ready',
  finish: '完成',
};

function duration(ms: number) {
  const seconds = Math.max(0, Math.floor(ms / 1_000));
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  if (hours > 0) return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
  return `${seconds}s`;
}

function relativeAge(now: number, timestamp: number | null) {
  if (!timestamp) return '尚未发生';
  return `${duration(Math.max(0, now - timestamp))} 前`;
}

function isTerminal(status?: string) {
  return ['success', 'degraded', 'failed', 'canceled', 'interrupted'].includes(String(status ?? '').toLowerCase());
}

export function PipelineLogMonitor({ logs, status, polling, pollingError, lastPollAt, lastLogReceivedAt, autoRefreshEnabled, onAutoRefreshChange, onRefresh }: Props) {
  const [level, setLevel] = useState('ALL');
  const [step, setStep] = useState('ALL');
  const [keyword, setKeyword] = useState('');
  const [newestFirst, setNewestFirst] = useState(true);
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1_000);
    return () => window.clearInterval(timer);
  }, []);

  const levels = useMemo(() => [...new Set(logs.map((row) => row.level))].sort(), [logs]);
  const steps = useMemo(() => [...new Set(logs.map((row) => row.step_name).filter((value): value is string => Boolean(value)))], [logs]);
  const filtered = useMemo(() => {
    const normalizedKeyword = keyword.trim().toLowerCase();
    const rows = logs.filter((row) => {
      if (level !== 'ALL' && row.level !== level) return false;
      if (step !== 'ALL' && row.step_name !== step) return false;
      if (!normalizedKeyword) return true;
      return [row.message, row.step_name, stepLabels[row.step_name ?? ''], row.level, String(row.sequence)]
        .filter(Boolean)
        .join(' ')
        .toLowerCase()
        .includes(normalizedKeyword);
    });
    return newestFirst ? [...rows].reverse() : rows;
  }, [keyword, level, logs, newestFirst, step]);
  const visible = newestFirst ? filtered.slice(0, 600) : filtered.slice(-600);
  const newestSequence = logs.length ? logs[logs.length - 1].sequence : 0;
  const latestDatabaseActivity = [...logs].reverse().find((row) =>
    row.message.includes('SQL 已确认存活')
    || row.message.includes('未发现本批次活动 SQL')
    || row.message.includes('数据库活动探测失败'));
  const databaseActivityStatus = latestDatabaseActivity?.message.includes('SQL 已确认存活')
    ? 'SQL 已确认存活'
    : latestDatabaseActivity?.level === 'warning'
      ? '需要检查'
      : latestDatabaseActivity
        ? '当前采样无 SQL'
        : '等待首次探测';
  const running = Boolean(status) && !isTerminal(status?.status);
  const silenceMs = lastLogReceivedAt ? Math.max(0, now - lastLogReceivedAt) : 0;
  const heartbeatDelayed = running && autoRefreshEnabled && lastLogReceivedAt !== null && silenceMs >= 45_000;
  const monitorTone = pollingError ? 'status-failure' : heartbeatDelayed ? 'status-warning' : polling ? 'status-running' : autoRefreshEnabled ? 'status-success' : 'status-warning';

  async function copyFiltered() {
    const text = [
      `time_zone: ${localTimeZone()}`,
      ...filtered.map((row) => `[${row.sequence}] ${formatLocalDateTime(row.timestamp)} ${row.level} ${row.step_name ?? '-'} ${row.elapsed_ms}ms ${row.message}`),
    ].join('\n');
    await navigator.clipboard?.writeText(text);
  }

  return (
    <section className="panel form-panel pipeline-log-monitor" aria-label="流水线实时日志监控">
      <div className="log-header pipeline-log-monitor-head">
        <div>
          <p className="eyebrow">Pipeline observability</p>
          <h3>任务实时监控</h3>
          <p className="muted-row">单通道增量刷新、按 sequence 去重；DWS/ADS 长步骤每15秒核验 MySQL PROCESSLIST、任务锁、子任务和小时分片。时间按本地PC时区 {localTimeZone()} 显示。</p>
        </div>
        <div className="log-summary">
          <span className={`status-pill ${monitorTone}`}>{pollingError ? '刷新异常' : polling ? '正在刷新' : autoRefreshEnabled ? '自动刷新' : '已暂停'}</span>
          <span className="status-pill">本机时区 {localTimeZone()}</span>
          <span className="status-pill">最新 seq {newestSequence || '-'}</span>
          <span className="status-pill">已保留 {logs.length} 条</span>
        </div>
      </div>

      <div className={`pipeline-monitor-health ${heartbeatDelayed || pollingError ? 'is-warning' : ''}`}>
        <div><span>当前步骤</span><strong>{stepLabels[status?.current_step ?? ''] ?? status?.current_step ?? '-'}</strong><small>{status?.message ?? '等待任务状态'}</small></div>
        <div><span>状态轮询</span><strong>{pollingError || (autoRefreshEnabled ? '连接正常' : '用户暂停')}</strong><small>最近成功刷新：{relativeAge(now, lastPollAt)}</small></div>
        <div><span>数据库活动</span><strong>{databaseActivityStatus}</strong><small>{heartbeatDelayed ? '超过45秒未收到任何新日志；可能是数据库连接受阻或进程中断' : !autoRefreshEnabled ? '自动刷新已暂停，恢复后继续接收探测结果' : latestDatabaseActivity ? `最近探测：${formatLocalDateTime(latestDatabaseActivity.timestamp)}` : running ? '等待后端核验 SQL、任务锁和 checkpoint' : '任务未运行或已经结束'}</small></div>
        <div><span>计划进度</span><strong>{Number(status?.percent ?? 0).toFixed(0)}%</strong><small>累计运行 {duration(Number(status?.elapsed_ms ?? 0))}</small></div>
      </div>

      {pollingError && <div className="analytics-error-banner"><strong>日志刷新失败</strong><span>{pollingError}</span></div>}

      <div className="pipeline-log-toolbar">
        <label>级别<select value={level} onChange={(event) => setLevel(event.target.value)}><option value="ALL">全部级别</option>{levels.map((item) => <option key={item} value={item}>{item}</option>)}</select></label>
        <label>步骤<select value={step} onChange={(event) => setStep(event.target.value)}><option value="ALL">全部步骤</option>{steps.map((item) => <option key={item} value={item}>{stepLabels[item] ?? item}</option>)}</select></label>
        <label className="pipeline-log-search">搜索<input value={keyword} onChange={(event) => setKeyword(event.target.value)} placeholder="错误、步骤、seq、消息" /></label>
        <div className="pipeline-log-actions">
          <button type="button" onClick={() => onAutoRefreshChange(!autoRefreshEnabled)}>{autoRefreshEnabled ? '暂停自动刷新' : '恢复自动刷新'}</button>
          <button type="button" disabled={polling} onClick={() => { void onRefresh(); }}>立即刷新</button>
          <button type="button" onClick={() => setNewestFirst((value) => !value)}>{newestFirst ? '改为时间顺序' : '最新优先'}</button>
          <button type="button" disabled={!filtered.length} onClick={() => { void copyFiltered(); }}>复制筛选结果</button>
        </div>
      </div>

      <div className="pipeline-log-result-summary">
        <span>筛选结果 {filtered.length} 条</span>
        {filtered.length > visible.length && <span>为保证界面流畅，当前显示前 {visible.length} 条；复制仍包含全部筛选结果</span>}
      </div>
      <div className="log-list structured-log-list pipeline-log-list">
        {visible.map((row) => {
          const heartbeat = row.message.includes('仍在运行') || row.message.includes('任务仍存活') || row.message.includes('状态心跳');
          const sqlRunning = /SQL \d+\/\d+ RUNNING/.test(row.message);
          const tone = row.level === 'error' ? 'failure' : row.level === 'warning' ? 'warning' : heartbeat || sqlRunning ? 'heartbeat' : 'success';
          return (
            <article key={row.sequence} className={`log-entry log-entry-${tone}`}>
              <div className="log-entry-head">
                <span className={`status-pill ${row.level === 'error' ? 'status-failure' : row.level === 'warning' ? 'status-warning' : heartbeat || sqlRunning ? 'status-running' : 'status-success'}`}>{sqlRunning ? 'SQL running' : heartbeat ? 'heartbeat' : row.level}</span>
                <strong>{stepLabels[row.step_name ?? ''] ?? row.step_name ?? '-'}</strong>
                <small>计划累计 {duration(row.elapsed_ms)}</small>
              </div>
              <div className="log-meta"><span title={`UTC: ${row.timestamp}`}>{formatLocalDateTime(row.timestamp)}</span><span>seq {row.sequence}</span><span>{row.step_name ?? '-'}</span></div>
              <pre>{row.message}</pre>
            </article>
          );
        })}
        {!visible.length && <div className="pipeline-log-empty">当前筛选条件下没有日志。</div>}
      </div>
    </section>
  );
}

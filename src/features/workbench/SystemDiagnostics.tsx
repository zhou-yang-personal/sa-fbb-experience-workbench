import { useEffect, useMemo, useRef, useState } from 'react';
import type { BatchTableRegistryRow, MetricCard, ModuleStatusRow, MySqlSettings } from '../../shared/types';
import { jobApi } from './jobApi';
import { mappingApi } from './mappingApi';
import { qualityApi } from './qualityApi';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  dataType: string;
};

type DiagnosticTaskId = 'catalog' | 'mapping' | 'quality' | 'etl' | 'modules' | 'registry';
type DiagnosticTaskStatus = 'pending' | 'running' | 'success' | 'failed' | 'skipped';
type DiagnosticRunStatus = 'idle' | 'running' | 'stopping' | 'success' | 'partial' | 'stopped';

const diagnosticTasks: { id: DiagnosticTaskId; label: string; hint: string }[] = [
  { id: 'catalog', label: '映射目录', hint: '检查 Catalog 版本与完整性' },
  { id: 'mapping', label: '字段映射', hint: '读取当前批次 required 缺失项' },
  { id: 'quality', label: '质量失败项', hint: '读取已生成的 Quality Gate 结果' },
  { id: 'etl', label: 'ETL 失败项', hint: '读取当前批次失败步骤' },
  { id: 'modules', label: '模块深度检查', hint: '较重：更新一次表计数并检查模块' },
  { id: 'registry', label: 'Registry 快照', hint: '读取深度检查更新后的缓存计数' },
];

function initialTaskState() {
  return Object.fromEntries(diagnosticTasks.map((task) => [task.id, 'pending'])) as Record<DiagnosticTaskId, DiagnosticTaskStatus>;
}

export function SystemDiagnostics({ settings, importBatchId, analysisRunId, dataType }: Props) {
  const [registry, setRegistry] = useState<BatchTableRegistryRow[]>([]);
  const [statusRows, setStatusRows] = useState<ModuleStatusRow[]>([]);
  const [mappingIssues, setMappingIssues] = useState<MetricCard[]>([]);
  const [catalogHealth, setCatalogHealth] = useState<MetricCard[]>([]);
  const [qualityFailed, setQualityFailed] = useState<MetricCard[]>([]);
  const [etlFailed, setEtlFailed] = useState<MetricCard[]>([]);
  const [message, setMessage] = useState('进入本页不会自动查询 MySQL；点击开始后才执行诊断。');
  const [runStatus, setRunStatus] = useState<DiagnosticRunStatus>('idle');
  const [taskState, setTaskState] = useState<Record<DiagnosticTaskId, DiagnosticTaskStatus>>(initialTaskState);
  const [currentTask, setCurrentTask] = useState('');
  const [errors, setErrors] = useState<string[]>([]);
  const stopRequestedRef = useRef(false);
  const runGenerationRef = useRef(0);
  const contextEffectReadyRef = useRef(false);

  const completedTasks = useMemo(
    () => Object.values(taskState).filter((status) => ['success', 'failed', 'skipped'].includes(status)).length,
    [taskState],
  );
  const progress = Math.round(completedTasks / diagnosticTasks.length * 100);

  function copyText(text: string) {
    if (typeof navigator !== 'undefined' && navigator.clipboard) {
      void navigator.clipboard.writeText(text);
    }
  }

  function buildDiagnosticText() {
    const lines = [
      `batch=${importBatchId || 'missing'}`,
      `analysis_run_id=${analysisRunId || 'missing'}`,
      `data_type=${dataType || 'missing'}`,
      `diagnostic_status=${runStatus}`,
      ...diagnosticTasks.map((task) => `task.${task.id}=${taskState[task.id]}`),
      ...errors.map((error) => `error=${error}`),
      `registry=${registry.length}`,
      `modules=${statusRows.length}`,
      ...catalogHealth.map((item) => `catalog.${item.label}=${item.value}; ${item.hint}`),
      ...registry.map((row) => `${row.logical_table_name} => ${row.physical_table_name} (${row.row_count})`),
      ...statusRows.map((row) => `${row.module_name}: enabled=${row.enabled}; ${row.status_text ?? '-'}`),
    ];
    return lines.join('\n');
  }

  async function refresh() {
    if (runStatus === 'running' || runStatus === 'stopping') return;
    const generation = runGenerationRef.current + 1;
    runGenerationRef.current = generation;
    stopRequestedRef.current = false;
    setRunStatus('running');
    setTaskState(initialTaskState());
    setErrors([]);
    setCurrentTask('准备诊断任务');
    setMessage('诊断任务已启动；各检查串行执行，避免同时压垮本地 MySQL。');
    const runErrors: string[] = [];

    async function runTask<T>(id: DiagnosticTaskId, action: () => Promise<T>, apply: (value: T) => void) {
      if (generation !== runGenerationRef.current) return;
      if (stopRequestedRef.current) {
        setTaskState((state) => ({ ...state, [id]: 'skipped' }));
        return;
      }
      const task = diagnosticTasks.find((item) => item.id === id);
      setCurrentTask(task?.label ?? id);
      setTaskState((state) => ({ ...state, [id]: 'running' }));
      try {
        const value = await action();
        if (generation !== runGenerationRef.current) return;
        apply(value);
        setTaskState((state) => ({ ...state, [id]: 'success' }));
      } catch (error) {
        if (generation !== runGenerationRef.current) return;
        const detail = `${task?.label ?? id}: ${error instanceof Error ? error.message : String(error)}`;
        runErrors.push(detail);
        setErrors([...runErrors]);
        setTaskState((state) => ({ ...state, [id]: 'failed' }));
      }
    }

    await runTask('catalog', () => workbenchApi.checkImportCatalog(settings), setCatalogHealth);
    if (generation !== runGenerationRef.current) return;
    if (!importBatchId.trim()) {
      setTaskState((state) => ({ ...state, mapping: 'skipped', quality: 'skipped', etl: 'skipped', modules: 'skipped', registry: 'skipped' }));
      setCurrentTask('');
      setRunStatus(runErrors.length ? 'partial' : 'success');
      setMessage('Catalog 检查完成；未选择批次，因此已跳过批次级诊断。');
      return;
    }
    await runTask('mapping', () => mappingApi.results(settings, importBatchId, dataType), (rows) => setMappingIssues(rows.filter((item) => item.value === 'missing_required')));
    await runTask('quality', () => qualityApi.failedResults(settings, importBatchId), setQualityFailed);
    await runTask('etl', () => jobApi.failedSteps(settings, importBatchId), setEtlFailed);
    await runTask('modules', () => workbenchApi.moduleStatus(settings, importBatchId, analysisRunId || undefined), setStatusRows);
    await runTask('registry', () => workbenchApi.cachedBatchTableRegistry(settings, importBatchId), setRegistry);
    if (generation !== runGenerationRef.current) return;

    const stopped = stopRequestedRef.current;
    setCurrentTask('');
    setRunStatus(stopped ? 'stopped' : runErrors.length ? 'partial' : 'success');
    setMessage(stopped
      ? '已停止尚未开始的后续检查；正在执行的 MySQL 语句不会被强制中断。'
      : runErrors.length
        ? `诊断完成，但有 ${runErrors.length} 个检查失败；请查看任务状态或复制诊断包。`
        : '诊断完成；表计数在本轮只刷新一次，Registry 使用缓存快照。');
  }

  useEffect(() => {
    if (!contextEffectReadyRef.current) {
      contextEffectReadyRef.current = true;
      return;
    }
    runGenerationRef.current += 1;
    stopRequestedRef.current = true;
    setRegistry([]);
    setStatusRows([]);
    setMappingIssues([]);
    setCatalogHealth([]);
    setQualityFailed([]);
    setEtlFailed([]);
    setTaskState(initialTaskState());
    setErrors([]);
    setCurrentTask('');
    setRunStatus('idle');
    setMessage('诊断上下文已变化；不会自动查询，确认后请手动开始。');
  }, [settings.host, settings.port, settings.database, settings.user, settings.secret, importBatchId, analysisRunId, dataType]);

  return (
    <section className="panel form-panel">
      <div className="step-card-head">
        <div>
          <h2>系统诊断</h2>
          <p className="hero-text">查看 batch table registry、module disabled reason、row count 和批次可用性。进入页面不会自动访问数据库。</p>
        </div>
        <div className="action-row">
          <button type="button" disabled={runStatus === 'running' || runStatus === 'stopping'} onClick={() => void refresh()}>{runStatus === 'running' || runStatus === 'stopping' ? '诊断执行中…' : '开始诊断任务（较重）'}</button>
          {(runStatus === 'running' || runStatus === 'stopping') && <button type="button" disabled={runStatus === 'stopping'} onClick={() => { stopRequestedRef.current = true; setRunStatus('stopping'); setMessage('将在当前检查返回后停止后续检查。'); }}>停止后续检查</button>}
          <button type="button" onClick={() => copyText(buildDiagnosticText())}>复制诊断包</button>
        </div>
      </div>
      <div className={`analytics-task-card task-${runStatus}`} aria-live="polite">
        <div className="analytics-task-head">
          <div><span>诊断计划</span><strong>{currentTask || (runStatus === 'idle' ? '等待用户启动' : '任务已结束')}</strong></div>
          <span className="analytics-task-status">{runStatus} · {progress}%</span>
        </div>
        <div className="analytics-task-progress"><span style={{ width: `${progress}%` }} /></div>
        <div className="analytics-task-plan">
          {diagnosticTasks.map((task) => {
            const status = taskState[task.id];
            const tone = status === 'success' ? 'complete' : status;
            return <span key={task.id} className={`is-${tone}`} title={task.hint}>{task.label} · {status}</span>;
          })}
        </div>
        <small>{message}</small>
        {errors.map((error) => <small key={error} className="status-failure-text">{error}</small>)}
      </div>
      <div className="summary-pills">
        <span className="status-pill">registry {registry.length}</span>
        <span className="status-pill">modules {statusRows.length}</span>
        <span className="status-pill">catalog {catalogHealth.find((item) => item.label === 'stale_catalog')?.value ?? 'unknown'}</span>
        <span className="status-pill">batch {importBatchId || 'missing'}</span>
      </div>
      <div className="table-like">
        <div className="table-row table-head"><span>Catalog</span><span>Value</span><span>Hint</span></div>
        {catalogHealth.map((item) => (
          <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row">
            <span>{item.label}</span>
            <span>{item.value}</span>
            <span>{item.hint}</span>
          </div>
        ))}
        {!catalogHealth.length && <div className="table-row muted-row">未读取 mapping catalog health。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Logical</span><span>Physical</span><span>Rows</span><span>Status</span></div>
        {registry.map((row) => (
          <div key={`${row.import_batch_id}-${row.logical_table_name}`} className="table-row">
            <span>{row.logical_table_name}</span>
            <span>{row.physical_table_name}</span>
            <span>{row.row_count}</span>
            <span>{row.status}</span>
          </div>
        ))}
        {!registry.length && <div className="table-row muted-row">未发现 batch table registry。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Module</span><span>Enabled</span><span>Tables / Fields</span><span>Reason</span></div>
        {statusRows.map((row) => (
          <div key={`${row.import_batch_id}-${row.module_id}`} className="table-row">
            <span>{row.module_name}</span>
            <span>{row.enabled ? 'yes' : 'no'}</span>
            <span>{row.missing_tables ?? row.missing_required_fields ?? row.data_type ?? '-'}</span>
            <span>{row.status_text ?? '-'}</span>
          </div>
        ))}
        {!statusRows.length && <div className="table-row muted-row">未发现 module status。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Mapping Missing</span><span>Status</span><span>Source</span></div>
        {mappingIssues.map((item) => (
          <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row">
            <span>{item.label}</span>
            <span>{item.value}</span>
            <span>{item.hint}</span>
          </div>
        ))}
        {!mappingIssues.length && <div className="table-row muted-row">未发现 required mapping 缺失。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Quality Failed</span><span>Status</span><span>Text</span></div>
        {qualityFailed.map((item) => (
          <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row">
            <span>{item.label}</span>
            <span>{item.value}</span>
            <span>{item.hint}</span>
          </div>
        ))}
        {!qualityFailed.length && <div className="table-row muted-row">未发现 quality failed 项。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>ETL Failed Step</span><span>Status</span><span>Hint</span></div>
        {etlFailed.map((item) => (
          <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row">
            <span>{item.label}</span>
            <span>{item.value}</span>
            <span>{item.hint}</span>
          </div>
        ))}
        {!etlFailed.length && <div className="table-row muted-row">未发现 ETL failed step。</div>}
      </div>
    </section>
  );
}

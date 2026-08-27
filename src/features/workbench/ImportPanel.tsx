import { useEffect, useMemo, useRef, useState } from 'react';
import type { AccessRuleSetRow, ActionState, BatchListItem, BatchTableRegistryRow, CsvProbeResult, ImportBatchResult, ImportDataType, ImportPipelineLogRow, ImportPipelineStatus, MetricCard, ModuleStatusRow, MySqlSettings } from '../../shared/types';
import { ActionButton } from './ActionButton';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import { BatchSelector } from './BatchSelector';
import { selectCsvFile } from './fileDialogs';
import { mappingApi } from './mappingApi';
import { PipelineLogMonitor } from './PipelineLogMonitor';
import { profileApi } from './profileApi';
import { qualityApi } from './qualityApi';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  effectiveSettings: MySqlSettings;
  dataType: ImportDataType;
  setDataType: (value: ImportDataType) => void;
  importMode: 'load_data' | 'streaming_insert';
  setImportMode: (value: 'load_data' | 'streaming_insert') => void;
  filePath: string;
  setFilePath: (value: string) => void;
  importBatchId: string;
  setImportBatchId: (value: string) => void;
  batchDisplayName: string;
  setBatchDisplayName: (value: string) => void;
  batch: ImportBatchResult | null;
  setBatch: (value: ImportBatchResult | null) => void;
  createBatch: (accessRuleSetId?: string) => Promise<ImportBatchResult | null>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
  actionStates: Record<string, ActionState>;
  analysisRunId: string;
  setAnalysisRunId: (value: string) => void;
  onOpenAnalysis?: () => void;
  onOpenAccessRules?: () => void;
};

type ImportWorkspace = 'new' | 'batches' | 'tasks';

function parseHint(hint: string) {
  return hint.split(/\s+\|\s+|,\s+/).reduce<Record<string, string>>((acc, part) => {
    const separator = part.indexOf('=');
    const key = separator >= 0 ? part.slice(0, separator).trim() : part.trim();
    const value = separator >= 0 ? part.slice(separator + 1).trim() : '';
    if (key) acc[key] = value ?? '';
    return acc;
  }, {});
}

function fileName(path: string) {
  const normalized = path.replace(/\\/g, '/');
  return normalized.split('/').pop() || path;
}

function withoutExtension(name: string) {
  return name.replace(/\.[^.]+$/, '');
}

function defaultBatchName(dataType: ImportDataType, path: string) {
  const name = withoutExtension(fileName(path || 'CSV')) || 'CSV';
  const time = new Date().toISOString().slice(0, 16).replace('T', ' ');
  return `${dataType.toUpperCase()}｜${name}｜${time}`;
}

function missingRequiredMessage(items: MetricCard[]) {
  const detail = items.map((item, index) => {
    const parsed = parseHint(item.hint);
    const source = parsed.source || '未匹配到任何 CSV header';
    const candidates = parsed.alias_candidates || '未配置候选 alias';
    const normalizedAliases = parsed.normalized_aliases || '未生成 normalized alias';
    const normalizedHeaders = (parsed.normalized_csv_headers || '未读取 CSV normalized headers').split('|').slice(0, 20).join('|');
    return `${index + 1}. target=${item.label}, required_flag=${parsed.required ?? '?'}, matched=false, matched_source=${source}, candidates=[${candidates}], normalized_candidates=[${normalizedAliases}], top_normalized_csv_headers=[${normalizedHeaders}]`;
  }).join('；');
  return `字段映射存在 ${items.length} 个 required 缺失，已停止 RAW 入库：${detail}。完整 normalized headers 可在映射结果中查看。`;
}

const MAX_PIPELINE_LOGS = 5_000;

function pipelineRunStorageKey(settings: MySqlSettings) {
  return `sa-fbb.pipeline-run.v1:${settings.host}:${settings.port}:${settings.database}:${settings.user}`;
}

function readStoredPipelineRunId(settings: MySqlSettings) {
  if (typeof window === 'undefined' || !window.localStorage) return '';
  try {
    return window.localStorage.getItem(pipelineRunStorageKey(settings)) ?? '';
  } catch {
    return '';
  }
}

function storePipelineRunId(settings: MySqlSettings, pipelineRunId: string) {
  if (typeof window === 'undefined' || !window.localStorage) return;
  try {
    window.localStorage.setItem(pipelineRunStorageKey(settings), pipelineRunId);
  } catch {
    // Monitoring still works for the current mounted view when storage is unavailable.
  }
}

function clearStoredPipelineRunId(settings: MySqlSettings) {
  if (typeof window === 'undefined' || !window.localStorage) return;
  try {
    window.localStorage.removeItem(pipelineRunStorageKey(settings));
  } catch {
    // No persistent pipeline context is available, but the current view can continue.
  }
}

function mergePipelineLogs(current: ImportPipelineLogRow[], incoming: ImportPipelineLogRow[]) {
  const unique = new Map<number, ImportPipelineLogRow>();
  current.forEach((row) => unique.set(row.sequence, row));
  incoming.forEach((row) => unique.set(row.sequence, row));
  return [...unique.values()].sort((left, right) => left.sequence - right.sequence).slice(-MAX_PIPELINE_LOGS);
}

export function ImportPanel(props: Props) {
  const { settings, dataType, setDataType, importMode, setImportMode, filePath, setFilePath, importBatchId, setImportBatchId, batchDisplayName, setBatchDisplayName, batch, setBatch, createBatch, runAction, loadMetrics, actionStates, setAnalysisRunId } = props;
  const [mappingSummary, setMappingSummary] = useState<MetricCard[]>([]);
  const [mappingResults, setMappingResults] = useState<MetricCard[]>([]);
  const [profileMetrics, setProfileMetrics] = useState<MetricCard[]>([]);
  const [mappingCatalog, setMappingCatalog] = useState<MetricCard[]>([]);
  const [catalogHealth, setCatalogHealth] = useState<MetricCard[]>([]);
  const [csvProbe, setCsvProbe] = useState<CsvProbeResult | null>(null);
  const [rawStatus, setRawStatus] = useState<MetricCard[]>([]);
  const [qualityRows, setQualityRows] = useState<MetricCard[]>([]);
  const [etlJobs, setEtlJobs] = useState<MetricCard[]>([]);
  const [registry, setRegistry] = useState<BatchTableRegistryRow[]>([]);
  const [moduleStatus, setModuleStatus] = useState<ModuleStatusRow[]>([]);
  const [statusMessage, setStatusMessage] = useState('请选择 CSV 文件，并确认本次导入批次名称。');
  const [pipelineStatus, setPipelineStatus] = useState<ImportPipelineStatus | null>(null);
  const [pipelineLogs, setPipelineLogs] = useState<ImportPipelineLogRow[]>([]);
  const [pipelineRunId, setPipelineRunId] = useState(() => readStoredPipelineRunId(settings));
  const [pipelinePolling, setPipelinePolling] = useState(false);
  const [pipelinePollingError, setPipelinePollingError] = useState('');
  const [pipelineAutoRefresh, setPipelineAutoRefresh] = useState(false);
  const [pipelineLastPollAt, setPipelineLastPollAt] = useState<number | null>(null);
  const [pipelineLastLogAt, setPipelineLastLogAt] = useState<number | null>(null);
  const [publishedRuleSets, setPublishedRuleSets] = useState<AccessRuleSetRow[]>([]);
  const [historyBatches, setHistoryBatches] = useState<BatchListItem[]>([]);
  const [historyStatus, setHistoryStatus] = useState('尚未读取历史批次；请点击“刷新批次列表”。');
  const [selectedRuleSetId, setSelectedRuleSetId] = useState('');
  const [accessRuleConfirmed, setAccessRuleConfirmed] = useState(false);
  const [staleTakeoverConfirmed, setStaleTakeoverConfirmed] = useState(false);
  const [accessRuleMessage, setAccessRuleMessage] = useState('TCP / Game 导入前必须手动选择并确认一个已发布 IP 规则版本。');
  const [workspace, setWorkspace] = useState<ImportWorkspace>('new');
  const lastLogSeqRef = useRef(0);
  const pipelinePollInFlightRef = useRef(new Set<string>());
  const pipelineGenerationRef = useRef(0);
  const requiresAccessRules = dataType === 'tcp' || dataType === 'game';
  const selectedRuleSet = publishedRuleSets.find((item) => item.rule_set_id === selectedRuleSetId);
  const selectedHistoryBatch = historyBatches.find((item) => item.import_batch_id === importBatchId) ?? null;

  const mappingCounts = useMemo(() => {
    const counts = { required: 0, optional: 0, exact: 0, alias: 0, missingRequired: 0, missingOptional: 0 };
    for (const item of mappingResults) {
      const parsed = parseHint(item.hint);
      const required = parsed.required === '1';
      if (required) counts.required += 1;
      else counts.optional += 1;
      if (item.value === 'matched') {
        const source = (parsed.source ?? '').trim().toLowerCase();
        const target = item.label.trim().toLowerCase();
        if (source && source === target) counts.exact += 1;
        else counts.alias += 1;
      }
      if (item.value === 'missing_required') counts.missingRequired += 1;
      if (item.value === 'missing_optional') counts.missingOptional += 1;
    }
    return counts;
  }, [mappingResults]);

  const mappingSummaryText = mappingSummary.length
    ? mappingSummary.map((item) => `${item.label}: ${item.value}`).join(' · ')
    : '未跑映射汇总';
  const missingTotal = mappingCounts.missingRequired + mappingCounts.missingOptional;
  const accessRuleReady = !requiresAccessRules || (Boolean(selectedRuleSetId) && accessRuleConfirmed);
  const canImport = Boolean(filePath.trim()) && Boolean(batchDisplayName.trim()) && accessRuleReady;
  const analysisRunId = props.analysisRunId.trim() || 'RUN_DEFAULT';
  const pipelineRunning = pipelineStatus?.status === 'running' || pipelineStatus?.status === 'pending';
  const selectedPipelineStatus = String(selectedHistoryBatch?.pipeline_status ?? '').toLowerCase();
  const selectedBatchReady = ['success', 'degraded'].includes(selectedPipelineStatus) && Boolean(selectedHistoryBatch?.analysis_run_id);
  const selectedBatchRunning = ['pending', 'running'].includes(selectedPipelineStatus);
  const selectedBatchFailed = selectedPipelineStatus === 'failed';
  const selectedBatchInterrupted = selectedPipelineStatus === 'interrupted';
  const selectedOrLoadedBatchRunning = Boolean(pipelineRunning || selectedBatchRunning);
  const importBlockReason = !filePath
    ? '请先选择 CSV 文件'
    : !batchDisplayName.trim()
      ? '请先设置批次名称'
      : requiresAccessRules && !selectedRuleSetId
        ? '请先选择本次导入使用的已发布 IP 规则版本'
        : requiresAccessRules && !accessRuleConfirmed
          ? '请勾选确认本次导入的 IP 规则版本'
          : undefined;

  async function refreshPublishedRuleSets() {
    if (!requiresAccessRules) {
      setPublishedRuleSets([]);
      setAccessRuleMessage('当前数据类型不参与 Cable / FTTH IP 规则匹配。');
      return;
    }
    setAccessRuleMessage('正在加载已发布 IP 规则版本…');
    try {
      const sets = await workbenchApi.accessRuleSets(settings);
      const published = sets.filter((item) => item.status === 'published');
      setPublishedRuleSets(published);
      if (selectedRuleSetId && !published.some((item) => item.rule_set_id === selectedRuleSetId)) {
        setSelectedRuleSetId('');
        setAccessRuleConfirmed(false);
      }
      setAccessRuleMessage(published.length
        ? `找到 ${published.length} 个已发布版本；每次导入都必须手动选择并确认。`
        : '没有已发布规则。请先进入接入识别配置，新增、验证并发布 IP 网段规则。');
    } catch (error) {
      setPublishedRuleSets([]);
      setSelectedRuleSetId('');
      setAccessRuleConfirmed(false);
      setAccessRuleMessage(`IP 规则加载失败：${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async function refreshHistoryBatches() {
    const batches = await workbenchApi.listBatches(settings);
    setHistoryBatches(batches);
    setHistoryStatus(batches.length ? `已加载 ${batches.length} 个历史批次。` : '当前没有历史批次。');
    return batches;
  }

  async function deleteHistoryBatches(batchIds: string[]) {
    let deleted = 0;
    const failures: string[] = [];
    await runAction('import_delete_batches', async () => {
      for (const batchId of batchIds) {
        try {
          await workbenchApi.deleteBatch(settings, batchId);
          deleted += 1;
        } catch (error) {
          failures.push(`${batchId}: ${error instanceof Error ? error.message : String(error)}`);
        }
      }
      if (failures.length) throw new Error(`已删除 ${deleted} 个，失败 ${failures.length} 个：${failures.join('；')}`);
      return { deleted, batch_ids: batchIds };
    });
    const remaining = await refreshHistoryBatches();
    if (importBatchId && !remaining.some((item) => item.import_batch_id === importBatchId)) {
      setImportBatchId('');
      setBatchDisplayName('');
      setBatch(null);
      setPipelineRunId('');
      setPipelineStatus(null);
      setPipelineLogs([]);
      setPipelineAutoRefresh(false);
      clearStoredPipelineRunId(settings);
      setRawStatus([]);
      setQualityRows([]);
      setEtlJobs([]);
      setRegistry([]);
      setModuleStatus([]);
    }
    setHistoryStatus(failures.length ? `已删除 ${deleted} 个批次，${failures.length} 个失败；详情见执行日志。` : `已删除 ${deleted} 个批次。`);
  }

  function requiredAccessRuleSetId() {
    if (!requiresAccessRules) return undefined;
    if (!selectedRuleSetId) throw new Error('请先选择本次导入使用的已发布 IP 规则版本。');
    if (!accessRuleConfirmed) throw new Error('请勾选确认本次导入的 IP 规则版本。');
    return selectedRuleSetId;
  }

  function formatMs(ms?: number) {
    const safe = Number(ms ?? 0);
    if (!Number.isFinite(safe) || safe <= 0) return '-';
    const seconds = Math.floor(safe / 1000);
    const minutes = Math.floor(seconds / 60);
    const rest = seconds % 60;
    return minutes > 0 ? `${minutes}m ${rest}s` : `${rest}s`;
  }

  function stepTone(status: string) {
    const normalized = status.toLowerCase();
    if (normalized === 'success') return 'status-success';
    if (normalized === 'failed') return 'status-failure';
    if (normalized === 'degraded' || normalized === 'skipped' || normalized === 'interrupted') return 'status-warning';
    if (normalized === 'running') return 'status-running';
    return '';
  }

  async function refreshPipeline(runId = pipelineRunId, generation = pipelineGenerationRef.current) {
    if (!runId) return null;
    if (pipelinePollInFlightRef.current.has(runId)) return null;
    pipelinePollInFlightRef.current.add(runId);
    setPipelinePolling(true);
    try {
      const [status, firstPage] = await Promise.all([
        workbenchApi.pipelineStatus(settings, runId),
        workbenchApi.pipelineLogs(settings, runId, lastLogSeqRef.current),
      ]);
      const collected: ImportPipelineLogRow[] = [];
      let cursor = lastLogSeqRef.current;
      if (firstPage.length) {
        collected.push(...firstPage);
        cursor = Math.max(cursor, ...firstPage.map((row) => row.sequence));
      }
      for (let pageIndex = 1; firstPage.length === 100 && pageIndex < 20; pageIndex += 1) {
        const page = await workbenchApi.pipelineLogs(settings, runId, cursor);
        if (!page.length) break;
        collected.push(...page);
        cursor = Math.max(cursor, ...page.map((row) => row.sequence));
        if (page.length < 100) break;
      }
      const normalizedStatus = String(status.status).toLowerCase();
      if (['success', 'degraded', 'failed', 'canceled', 'interrupted'].includes(normalizedStatus)) {
        for (let pageIndex = 0; pageIndex < 20; pageIndex += 1) {
          const page = await workbenchApi.pipelineLogs(settings, runId, cursor);
          if (!page.length) break;
          collected.push(...page);
          cursor = Math.max(cursor, ...page.map((row) => row.sequence));
          if (page.length < 100) break;
        }
      }
      if (generation !== pipelineGenerationRef.current) return null;
      setPipelineStatus(status);
      if (status.import_batch_id) setImportBatchId(status.import_batch_id);
      if (status.analysis_run_id) setAnalysisRunId(status.analysis_run_id);
      if (collected.length) {
        lastLogSeqRef.current = cursor;
        setPipelineLogs((items) => mergePipelineLogs(items, collected));
        setPipelineLastLogAt(Date.now());
      }
      setPipelineLastPollAt(Date.now());
      setPipelinePollingError('');
      setStatusMessage(status.message ?? `Pipeline ${status.status}`);
      if (status.import_batch_id && ['success', 'degraded'].includes(normalizedStatus)) {
        const [quality, jobs] = await Promise.allSettled([
          qualityApi.allResults(settings, status.import_batch_id),
          workbenchApi.jobs(settings, status.import_batch_id),
        ]);
        if (quality.status === 'fulfilled') setQualityRows(quality.value);
        if (jobs.status === 'fulfilled') setEtlJobs(jobs.value);
      } else if (status.import_batch_id && ['failed', 'interrupted'].includes(normalizedStatus)) {
        const [failedQuality, nextRawStatus] = await Promise.allSettled([
          qualityApi.failedResults(settings, status.import_batch_id),
          workbenchApi.importStatus(settings, status.import_batch_id),
        ]);
        setQualityRows(failedQuality.status === 'fulfilled' ? failedQuality.value : []);
        setRawStatus(nextRawStatus.status === 'fulfilled' ? nextRawStatus.value : []);
      }
      return status;
    } catch (error) {
      if (generation !== pipelineGenerationRef.current) return null;
      const message = error instanceof Error ? error.message : String(error);
      setPipelinePollingError(message);
      setStatusMessage(`Pipeline 状态刷新失败：${message}`);
      return null;
    } finally {
      pipelinePollInFlightRef.current.delete(runId);
      setPipelinePolling(pipelinePollInFlightRef.current.size > 0);
    }
  }

  async function restorePipelineForBatch(selected: BatchListItem) {
    const generation = pipelineGenerationRef.current + 1;
    pipelineGenerationRef.current = generation;
    setAnalysisRunId(selected.analysis_run_id ?? '');
    if (!selected.pipeline_run_id) {
      setPipelineRunId('');
      clearStoredPipelineRunId(settings);
      setPipelineStatus(null);
      setPipelineLogs([]);
      setPipelinePollingError('');
      lastLogSeqRef.current = 0;
      setStatusMessage('该批次没有关联的自动导入流水线日志；可能是旧版或手工创建的 RAW 批次。');
      return null;
    }
    setPipelineRunId(selected.pipeline_run_id);
    storePipelineRunId(settings, selected.pipeline_run_id);
    setPipelineStatus(null);
    setPipelineLogs([]);
    setPipelinePollingError('');
    setPipelineLastPollAt(null);
    setPipelineLastLogAt(null);
    setPipelineAutoRefresh(true);
    setWorkspace('tasks');
    lastLogSeqRef.current = 0;
    setStatusMessage(`正在恢复批次 ${selected.import_batch_id} 的流水线状态与日志…`);
    return refreshPipeline(selected.pipeline_run_id, generation);
  }

  async function startPipeline() {
    await runAction('import_pipeline_start', async () => {
      if (!filePath.trim()) throw new Error('请先通过文件选择框选择 CSV 文件。');
      if (!batchDisplayName.trim()) throw new Error('请先为本次导入设置批次名称。');
      const accessRuleSetId = requiredAccessRuleSetId();
      pipelineGenerationRef.current += 1;
      setPipelineLogs([]);
      setPipelinePollingError('');
      setPipelineLastPollAt(null);
      setPipelineLastLogAt(null);
      setPipelineAutoRefresh(true);
      setQualityRows([]);
      setRawStatus([]);
      setProfileMetrics([]);
      setRegistry([]);
      setModuleStatus([]);
      lastLogSeqRef.current = 0;
      const started = await workbenchApi.pipelineStart(settings, dataType, filePath, batchDisplayName, importMode, analysisRunId, accessRuleSetId);
      setPipelineRunId(started.pipeline_run_id);
      storePipelineRunId(settings, started.pipeline_run_id);
      setPipelineStatus({
        pipeline_run_id: started.pipeline_run_id,
        status: started.status,
        current_step: 'prepare_environment',
        percent: 0,
        elapsed_ms: 0,
        import_batch_id: started.import_batch_id,
        analysis_run_id: started.analysis_run_id,
        final_fusion_status: 'pending',
        message: '后台执行计划已启动，前台将每秒刷新。',
        steps: [],
      });
      setWorkspace('tasks');
      setAccessRuleConfirmed(false);
      setStatusMessage(`后台执行计划已启动：${started.pipeline_run_id}${selectedRuleSet ? `；已绑定 IP 规则 v${selectedRuleSet.version}` : ''}`);
      return started;
    });
  }

  async function resumeCurrentBatch() {
    if (!importBatchId.trim()) {
      setStatusMessage('请先在历史批次中选择要复用的批次。');
      return;
    }
    if (selectedOrLoadedBatchRunning && !staleTakeoverConfirmed) {
      setStatusMessage('该批次仍显示有运行中任务。请等待完成；只有原 EXE 已退出时才能确认接管。');
      return;
    }
    if (staleTakeoverConfirmed) {
      const confirmed = window.confirm(
        '仅当原 EXE 已退出、且 MySQL 中该批次活动 SQL 已结束时才能接管。后端仍会检查并拒绝并发执行。确认继续？',
      );
      if (!confirmed) {
        setStatusMessage('已取消批次接管。');
        return;
      }
    }
    await runAction('import_pipeline_resume_batch', async () => {
      const started = await workbenchApi.pipelineResume(
        settings,
        importBatchId,
        props.analysisRunId.trim() || undefined,
        staleTakeoverConfirmed,
      );
      pipelineGenerationRef.current += 1;
      setPipelineLogs([]);
      setPipelinePollingError('');
      setPipelineLastPollAt(null);
      setPipelineLastLogAt(null);
      setPipelineAutoRefresh(true);
      setRegistry([]);
      setModuleStatus([]);
      lastLogSeqRef.current = 0;
      setPipelineRunId(started.pipeline_run_id);
      storePipelineRunId(settings, started.pipeline_run_id);
      setAnalysisRunId(started.analysis_run_id);
      setPipelineStatus({
        pipeline_run_id: started.pipeline_run_id,
        status: started.status,
        current_step: 'prepare_resume',
        percent: 0,
        elapsed_ms: 0,
        import_batch_id: started.import_batch_id ?? importBatchId,
        analysis_run_id: started.analysis_run_id,
        final_fusion_status: 'pending',
        message: '已启动批次复用；跳过 CSV、RAW、Quality Gate 与 CLEAN，从完整 DWS/ADS 续跑。',
        steps: [],
      });
      setWorkspace('tasks');
      setStaleTakeoverConfirmed(false);
      setStatusMessage(`批次复用任务已启动：${started.pipeline_run_id}`);
      return started;
    });
  }

  async function rebuildCurrentBatchFromRaw() {
    if (!importBatchId.trim()) {
      setStatusMessage('请先在历史批次中选择要从 RAW 重建的批次。');
      return;
    }
    if (selectedOrLoadedBatchRunning && !staleTakeoverConfirmed) {
      setStatusMessage('该批次仍显示有运行中任务。请等待完成；只有原 EXE 已退出时才能确认接管。');
      return;
    }
    const confirmed = window.confirm(
      '将保留现有 RAW 和旧 analysis run，但会从 RAW 重新执行 Quality Gate、覆盖当前批次 CLEAN/DWD 与 DWS，并生成新的 ADS/V2 analysis run。大批次可能持续较长时间，期间不要关闭应用或并发启动其他任务。确认继续？',
    );
    if (!confirmed) {
      setStatusMessage('已取消从 RAW 重建。');
      return;
    }
    if (staleTakeoverConfirmed) {
      const takeoverConfirmed = window.confirm(
        '当前批次存在遗留 running 状态。仅当原 EXE 已退出且 MySQL 已无该批次活动 SQL 时才能接管；后端仍会再次检查。确认继续？',
      );
      if (!takeoverConfirmed) {
        setStatusMessage('已取消遗留任务接管。');
        return;
      }
    }
    await runAction('import_pipeline_rebuild_batch_from_raw', async () => {
      const started = await workbenchApi.pipelineRebuildFromRaw(
        settings,
        importBatchId,
        staleTakeoverConfirmed,
      );
      pipelineGenerationRef.current += 1;
      setPipelineLogs([]);
      setPipelinePollingError('');
      setPipelineLastPollAt(null);
      setPipelineLastLogAt(null);
      setPipelineAutoRefresh(true);
      setQualityRows([]);
      setRawStatus([]);
      setProfileMetrics([]);
      setRegistry([]);
      setModuleStatus([]);
      lastLogSeqRef.current = 0;
      setPipelineRunId(started.pipeline_run_id);
      storePipelineRunId(settings, started.pipeline_run_id);
      setAnalysisRunId(started.analysis_run_id);
      setPipelineStatus({
        pipeline_run_id: started.pipeline_run_id,
        status: started.status,
        current_step: 'prepare_rebuild',
        percent: 0,
        elapsed_ms: 0,
        import_batch_id: started.import_batch_id ?? importBatchId,
        analysis_run_id: started.analysis_run_id,
        final_fusion_status: 'pending',
        message: '已启动 RAW 重建；跳过 CSV 和 RAW 导入，重跑 Quality Gate、CLEAN/DWS/ADS/V2。',
        steps: [],
      });
      setWorkspace('tasks');
      setStaleTakeoverConfirmed(false);
      setStatusMessage(`RAW 重建任务已启动：${started.pipeline_run_id}；新运行 ${started.analysis_run_id}`);
      return started;
    });
  }

  useEffect(() => {
    const storedRunId = readStoredPipelineRunId(settings);
    if (storedRunId === pipelineRunId) return;
    pipelineGenerationRef.current += 1;
    setPipelineRunId(storedRunId);
    setPipelineStatus(null);
    setPipelineLogs([]);
    setPipelinePollingError('');
    setPipelineLastPollAt(null);
    setPipelineLastLogAt(null);
    lastLogSeqRef.current = 0;
  }, [settings.host, settings.port, settings.database, settings.user]);

  useEffect(() => {
    if (!pipelineRunId || !pipelineAutoRefresh) return;
    const status = String(pipelineStatus?.status ?? 'running').toLowerCase();
    if (['success', 'degraded', 'failed', 'canceled', 'interrupted'].includes(status)) return;
    let disposed = false;
    const poll = async () => {
      if (disposed) return;
      await refreshPipeline(pipelineRunId, pipelineGenerationRef.current);
    };
    const timer = window.setInterval(poll, 1000);
    void poll();
    return () => {
      disposed = true;
      window.clearInterval(timer);
    };
  }, [pipelineAutoRefresh, pipelineRunId, pipelineStatus?.status, settings]);

  async function chooseFile() {
    const result = await runAction('select_csv_file', () => selectCsvFile());
    if (result === null) {
      setStatusMessage('无法打开系统文件选择器，请查看执行日志中的具体错误。');
      return;
    }
    const selected = typeof result === 'string' ? result : '';
    if (!selected) {
      setStatusMessage('文件选择已取消。');
      return;
    }
    setFilePath(selected);
    setAccessRuleConfirmed(false);
    if (!batchDisplayName.trim()) setBatchDisplayName(defaultBatchName(dataType, selected));
    setStatusMessage(`已选择文件：${fileName(selected)}；请确认本次导入的 IP 规则版本。`);
  }

  async function createSelectedBatch() {
    const accessRuleSetId = requiredAccessRuleSetId();
    const result = await createBatch(accessRuleSetId);
    if (result) setAccessRuleConfirmed(false);
    return result;
  }

  async function refreshMappingSummary(batchId = importBatchId) {
    const result = await loadMetrics('import_get_mapping_summary', () => mappingApi.summary(settings, batchId, dataType));
    setMappingSummary(result);
    setStatusMessage(result.length ? '映射汇总已刷新' : '映射汇总为空');
    return result;
  }

  async function refreshMappingResults(batchId = importBatchId) {
    const result = await loadMetrics('import_get_mapping_results', () => mappingApi.results(settings, batchId, dataType));
    setMappingResults(result);
    setStatusMessage(result.length ? '映射结果已刷新' : '映射结果为空');
    return result;
  }

  async function refreshProfile(batchId = importBatchId) {
    const result = await loadMetrics('dataset_profile_get', () => profileApi.get(settings, batchId, dataType));
    setProfileMetrics(result);
    setStatusMessage(result.length ? '数据画像已刷新' : '当前无 profile');
    return result;
  }

  async function refreshMappingCatalog() {
    const result = await loadMetrics('config_get_import_mappings', () => workbenchApi.importMappings(settings, dataType));
    setMappingCatalog(result);
    setStatusMessage(result.length ? '字段映射目录已刷新' : '字段映射目录为空');
  }

  async function refreshCatalogHealth() {
    const result = await loadMetrics('config_check_import_catalog', () => workbenchApi.checkImportCatalog(settings));
    setCatalogHealth(result);
    return result;
  }

  async function prepareImportEnvironment() {
    await runAction('import_prepare_environment', async () => {
      await workbenchApi.testDb(settings);
      await workbenchApi.initDb(settings);
      await workbenchApi.seedConfig(settings);
      const health = await workbenchApi.checkImportCatalog(settings);
      setCatalogHealth(health);
      return health;
    });
  }

  async function probeCurrentFile() {
    const result = await runAction('import_probe_csv', () => workbenchApi.probeCsv(filePath)) as CsvProbeResult;
    setCsvProbe(result);
    setStatusMessage(`Probe 完成：${result.file_name}`);
    return result;
  }

  async function validateMapping() {
    await runAction('import_validate_mapping', () => workbenchApi.validateMapping(settings, importBatchId, dataType, filePath));
    await refreshMappingSummary();
    await refreshMappingResults();
  }

  async function refreshRawStatus(batchId = importBatchId) {
    const result = await loadMetrics('import_get_batch_status', () => workbenchApi.importStatus(settings, batchId));
    setRawStatus(result);
    setStatusMessage(result.length ? '导入状态已刷新' : '当前没有导入状态');
    return result;
  }

  async function refreshRawLoad() {
    await runAction('import_start_raw_load', () => workbenchApi.loadRaw(settings, importBatchId, dataType, filePath, importMode));
    await refreshRawStatus();
    await refreshProfile();
  }

  async function importCurrentFile() {
    await runAction('import_current_file', async () => {
      if (!filePath.trim()) throw new Error('请先通过文件选择框选择 CSV 文件。');
      if (!batchDisplayName.trim()) throw new Error('请先为本次导入设置一个正常人可读的批次名称。');
      const accessRuleSetId = requiredAccessRuleSetId();
      const health = await workbenchApi.checkImportCatalog(settings);
      setCatalogHealth(health);
      const stale = health.some((item) => item.label === 'stale_catalog' && item.value === 'yes');
      if (stale) {
        await workbenchApi.seedConfig(settings);
        const repaired = await workbenchApi.checkImportCatalog(settings);
        setCatalogHealth(repaired);
      }
      const result = await workbenchApi.importCurrentFile(settings, dataType, filePath, batchDisplayName, importMode, accessRuleSetId);
      setBatch(result.batch);
      setImportBatchId(result.batch.import_batch_id);
      setMappingSummary(result.mapping_summary);
      setMappingResults(result.mapping_results);
      setRawStatus(result.raw_status);
      setProfileMetrics(result.profile);
      setAccessRuleConfirmed(false);
      setStatusMessage(`导入完成：${batchDisplayName} / ${result.batch.import_batch_id}`);
      return result;
    });
  }

  async function runQualityGate() {
    await runAction('quality_run_gate', () => workbenchApi.qualityGate(settings, importBatchId));
    const result = await loadMetrics('quality_get_gate_results', () => qualityApi.allResults(settings, importBatchId));
    setQualityRows(result);
    return result;
  }

  async function runCleanDwd() {
    await runAction('etl_start_clean_job', () => workbenchApi.clean(settings, importBatchId));
    const jobs = await loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId));
    setEtlJobs(jobs);
    const nextRegistry = await workbenchApi.cachedBatchTableRegistry(settings, importBatchId);
    setRegistry(nextRegistry);
    return jobs;
  }

  async function runDwsAds() {
    await runAction('import_generate_dws_ads', async () => {
      await workbenchApi.aggregate(settings, importBatchId, analysisRunId);
      await workbenchApi.completeAggregates(settings, importBatchId, analysisRunId);
      await workbenchApi.completeDashboards(settings, importBatchId, analysisRunId);
      await materializeStructuredAds();
      try {
        await workbenchApi.fuse(settings, importBatchId, analysisRunId);
        await analyticsStructuredApi.materializeLead(settings, importBatchId, analysisRunId);
      } catch (error) {
        return { status: 'basic_dashboards_ready_final_fusion_degraded', final_fusion: error instanceof Error ? error.message : String(error) };
      }
      return { status: 'dws_ads_ready', analysis_run_id: analysisRunId };
    });
    const jobs = await loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId));
    setEtlJobs(jobs);
    const nextRegistry = await workbenchApi.cachedBatchTableRegistry(settings, importBatchId);
    setRegistry(nextRegistry);
    return jobs;
  }

  async function refreshModuleReady() {
    const prepared = await workbenchApi.prepareBatchTables(settings, importBatchId);
    const nextRegistry = await workbenchApi.cachedBatchTableRegistry(settings, importBatchId);
    const status = await workbenchApi.moduleStatus(settings, importBatchId, analysisRunId);
    setRegistry(nextRegistry);
    setModuleStatus(status);
    setStatusMessage(`模块可用性已刷新：enabled=${status.filter((item) => item.enabled).length}`);
    return { prepared, registry: nextRegistry, status };
  }

  async function materializeStructuredAds() {
    await analyticsStructuredApi.materializeAppRank(settings, importBatchId, analysisRunId);
    await analyticsStructuredApi.materializeHourly(settings, importBatchId, analysisRunId);
    await analyticsStructuredApi.materializeNetwork(settings, importBatchId, analysisRunId);
    await analyticsStructuredApi.materializeUser(settings, importBatchId, analysisRunId);
    await analyticsStructuredApi.materializeLead(settings, importBatchId, analysisRunId);
  }

  async function generateAnalyzableBatch() {
    await runAction('import_generate_analyzable_batch', async () => {
      await workbenchApi.qualityGate(settings, importBatchId);
      await workbenchApi.clean(settings, importBatchId);
      await workbenchApi.aggregate(settings, importBatchId, analysisRunId);
      await workbenchApi.completeAggregates(settings, importBatchId, analysisRunId);
      await workbenchApi.completeDashboards(settings, importBatchId, analysisRunId);
      await materializeStructuredAds();
      let finalFusion = 'success';
      try {
        await workbenchApi.fuse(settings, importBatchId, analysisRunId);
        await analyticsStructuredApi.materializeLead(settings, importBatchId, analysisRunId);
      } catch (error) {
        finalFusion = `degraded: ${error instanceof Error ? error.message : String(error)}`;
      }
      const quality = await qualityApi.allResults(settings, importBatchId);
      const jobs = await workbenchApi.jobs(settings, importBatchId);
      const nextRegistry = await workbenchApi.cachedBatchTableRegistry(settings, importBatchId);
      const status = await workbenchApi.moduleStatus(settings, importBatchId, analysisRunId);
      setQualityRows(quality);
      setEtlJobs(jobs);
      setRegistry(nextRegistry);
      setModuleStatus(status);
      setStatusMessage('可分析批次生成流程已完成。');
      return { status: 'analyzable_batch_ready', final_fusion: finalFusion, module_status: status };
    });
  }

  function selectHistoryBatch(selected: BatchListItem | null) {
    pipelineGenerationRef.current += 1;
    setStaleTakeoverConfirmed(false);
    setPipelineAutoRefresh(false);
    setPipelineStatus(null);
    setPipelineLogs([]);
    setPipelinePollingError('');
    setPipelineLastPollAt(null);
    setPipelineLastLogAt(null);
    lastLogSeqRef.current = 0;
    if (!selected) {
      setImportBatchId('');
      setBatchDisplayName('');
      setBatch(null);
      setPipelineRunId('');
      setHistoryStatus('未选择批次。');
      return;
    }
    setImportBatchId(selected.import_batch_id);
    setBatchDisplayName(selected.batch_display_name ?? selected.source_file_name);
    setDataType(selected.data_type as ImportDataType);
    setAnalysisRunId(selected.analysis_run_id ?? '');
    setPipelineRunId(selected.pipeline_run_id ?? '');
    if (selected.pipeline_run_id) storePipelineRunId(settings, selected.pipeline_run_id);
    else clearStoredPipelineRunId(settings);
    setBatch({
      import_batch_id: selected.import_batch_id,
      batch_display_name: selected.batch_display_name,
      data_type: selected.data_type,
      source_file_name: selected.source_file_name,
      status: selected.status,
    });
    setHistoryStatus(`已选择 ${selected.batch_display_name?.trim() || selected.source_file_name}；尚未查询任务详情。`);
  }

  const recoveryRecommendation = selectedBatchRunning
    ? '先加载任务状态，确认它仍在执行；不要启动第二个任务。'
    : selectedBatchInterrupted
      ? '原任务已因MySQL或应用中断。确认当前没有活动SQL后，可从已完成的小时分片继续聚合。'
    : selectedBatchFailed
      ? '先查看失败步骤：CLEAN 已成功时选择“继续聚合”，否则选择“从 RAW 重建”。'
      : selectedBatchReady
        ? '结果已就绪，可以直接进入数据分析；只有规则或口径变化时才需要重建。'
        : selectedHistoryBatch?.status.toLowerCase() === 'success'
          ? 'RAW 已就绪但完整结果未确认。建议先加载任务状态，再选择恢复方式。'
          : '该批次尚未具备可靠的 RAW 数据，不建议继续聚合。';

  const importSteps = pipelineStatus?.steps.length ? pipelineStatus.steps.map((step) => ({
    title: `Step ${step.step_index}. ${step.step_label}`,
    detail: step.message || step.error_message || step.step_name,
    status: step.status,
    elapsed: step.elapsed_ms,
  })) : [
    { title: 'Step 1. 导入准备', detail: '连接、schema、mapping catalog self-heal 与版本健康。', status: 'pending', elapsed: 0 },
    { title: 'Step 2. CSV 探测', detail: '读取文件名、大小、headers 和预览。', status: 'pending', elapsed: 0 },
    { title: 'Step 3. 字段映射与 RAW 入库', detail: '后端 atomic import：catalog repair、batch、mapping、RAW load、profile。', status: 'pending', elapsed: 0 },
    { title: 'Step 4. RAW 质量检查', detail: 'Quality Gate 按 data_type 路由。', status: 'pending', elapsed: 0 },
    { title: 'Step 5. CLEAN/DWD 生成', detail: `${dataType} 批次只运行适用 RAW→CLEAN step。`, status: 'pending', elapsed: 0 },
    { title: 'Step 6. DWS/ADS 聚合', detail: '生成基础聚合、看板 ADS 与 SA Lead。', status: 'pending', elapsed: 0 },
    { title: 'Step 7. Final Lead 融合', detail: '缺 CRM/coverage/reachability 时 degraded，不阻断基础结果。', status: 'pending', elapsed: 0 },
    { title: 'Step 8. Module Ready', detail: '刷新 registry 和 module status。', status: 'pending', elapsed: 0 },
  ];

  return (
    <article className="panel form-panel step-card">
      <div className="step-card-head">
        <div>
          <h2>数据作业中心</h2>
          <p className="hero-text">按任务进入对应工作区。打开页面和切换工作区不会自动查询数据库，只有明确点击刷新或执行按钮才会访问 MySQL。</p>
        </div>
        <span className="step-badge">Data jobs</span>
      </div>
      <nav className="import-workspace-switcher" aria-label="数据作业工作区">
        <button type="button" className={workspace === 'new' ? 'is-active' : ''} onClick={() => setWorkspace('new')}>
          <strong>新建导入</strong><small>选择 CSV、规则并启动完整流水线</small>
        </button>
        <button type="button" className={workspace === 'batches' ? 'is-active' : ''} onClick={() => setWorkspace('batches')}>
          <strong>批次库</strong><small>选择已有数据并决定下一步</small>
        </button>
        <button type="button" className={workspace === 'tasks' ? 'is-active' : ''} onClick={() => setWorkspace('tasks')}>
          <strong>运行任务</strong><small>{pipelineStatus ? `${pipelineStatus.status} · ${(pipelineStatus.percent ?? 0).toFixed(0)}%` : pipelineRunId ? '有历史任务可加载' : '查看进度、日志与排错'}</small>
        </button>
      </nav>
      {workspace === 'tasks' && pipelineRunId && <div className="table-like pipeline-plan-table" style={{ marginBottom: 12 }}>
        <div className="table-row table-head"><span>执行计划</span><span>状态 / 信息</span><span>耗时</span></div>
        {importSteps.map((step) => <div key={step.title} className={`table-row ${stepTone(step.status)}`}><span>{step.title}</span><span>{step.status} · {step.detail}</span><span>{formatMs(step.elapsed)}</span></div>)}
      </div>}
      {workspace === 'new' && (
      <section className="panel form-panel">
        <div className="step-card-head">
          <div>
            <h3>新建导入</h3>
            <p className="muted-row">完成文件、批次名称和规则确认后，一次启动 CSV → RAW → CLEAN → DWS/ADS。运行后自动进入任务监控。</p>
          </div>
          <span className="step-badge">1 个主操作</span>
        </div>
        <div className="form-grid import-form-grid">
          <label>
            批次名称
            <input value={batchDisplayName} onChange={(e) => setBatchDisplayName(e.target.value)} placeholder="例如：TCP 视频体验｜Claro｜2026-07-05 晚高峰" />
          </label>
          <label>
            数据类型
            <select value={dataType} onChange={(e) => {
              const next = e.target.value as ImportDataType;
              setDataType(next);
              setAccessRuleConfirmed(false);
              if (filePath && !batchDisplayName.trim()) setBatchDisplayName(defaultBatchName(next, filePath));
            }}>
              <option value="tcp">TCP / Universal Video</option><option value="game">Game</option><option value="crm">CRM Users</option><option value="coverage">FTTH Coverage</option><option value="reachability">Reachability</option>
            </select>
          </label>
          <label>
            导入方式
            <select value={importMode} onChange={(e) => setImportMode(e.target.value as 'load_data' | 'streaming_insert')}>
              <option value="load_data">LOAD DATA LOCAL INFILE</option><option value="streaming_insert">Streaming INSERT fallback</option>
            </select>
          </label>
        </div>
        <section className="file-picker-card">
          <div>
            <span>CSV 文件</span>
            <strong title={filePath}>{filePath ? fileName(filePath) : '未选择文件'}</strong>
            <small>{filePath || '请使用系统弹框选择文件。'}</small>
          </div>
          <button type="button" disabled={pipelineRunning} onClick={chooseFile}>选择 CSV 文件</button>
        </section>
        {requiresAccessRules && (
          <section className={`access-rule-confirmation-card ${accessRuleReady ? 'is-ready' : ''}`}>
            <div className="access-rule-confirmation-head">
              <div>
                <span>本次导入的接入识别规则</span>
                <strong>{selectedRuleSet ? `v${selectedRuleSet.version} · ${selectedRuleSet.rule_set_name} · 未命中→${selectedRuleSet.default_access_type}` : '尚未选择'}</strong>
                <small>{accessRuleMessage}</small>
              </div>
              <div className="action-row">
                <button type="button" disabled={pipelineRunning} onClick={() => { void refreshPublishedRuleSets(); }}>刷新版本</button>
                <button type="button" disabled={pipelineRunning} onClick={props.onOpenAccessRules}>配置 IP 网段</button>
              </div>
            </div>
            <div className="access-rule-confirmation-grid">
              <label>
                已发布规则版本
                <select value={selectedRuleSetId} disabled={pipelineRunning} onChange={(event) => {
                  setSelectedRuleSetId(event.target.value);
                  setAccessRuleConfirmed(false);
                }}>
                  <option value="">请选择，不自动使用最新版本</option>
                  {publishedRuleSets.map((item) => <option key={item.rule_set_id} value={item.rule_set_id}>v{item.version} · {item.rule_set_name} · {item.rule_count} 条 · 未命中→{item.default_access_type}</option>)}
                </select>
              </label>
              <label className="access-rule-confirmation-check">
                <input type="checkbox" disabled={!selectedRuleSetId || pipelineRunning} checked={accessRuleConfirmed} onChange={(event) => setAccessRuleConfirmed(event.target.checked)} />
                <span>我已检查该版本的 Cable / FTTH 网段，并确认用于本次 CSV 导入</span>
              </label>
            </div>
          </section>
        )}
        <div className="primary-action-row">
          <ActionButton actionKey="import_pipeline_start" actionStates={actionStates} primary label={pipelineRunning ? '执行计划运行中' : '启动导入分析计划'} disabled={!canImport || pipelineRunning} onClick={startPipeline} title={importBlockReason} />
          {pipelineRunId && <button type="button" onClick={() => setWorkspace('tasks')}>查看当前任务</button>}
        </div>
      </section>
      )}
      {workspace === 'batches' && (
      <>
      <BatchSelector
        variant="library"
        batches={historyBatches}
        selectedBatchId={importBatchId}
        statusText={historyStatus}
        onRefresh={refreshHistoryBatches}
        onDeleteBatches={deleteHistoryBatches}
        onSelectBatch={selectHistoryBatch}
      />
      {selectedHistoryBatch ? (
        <section className="panel form-panel selected-batch-action-panel">
          <div className="step-card-head">
            <div>
              <span className="panel-kicker">当前选择</span>
              <h3>{selectedHistoryBatch.batch_display_name?.trim() || selectedHistoryBatch.source_file_name}</h3>
              <p className="muted-row">{selectedHistoryBatch.import_batch_id}</p>
            </div>
            <span className={`step-badge ${selectedBatchFailed ? 'status-failure' : selectedBatchRunning ? 'status-running' : selectedBatchReady ? 'status-success' : 'status-warning'}`}>
              {selectedBatchReady ? '可分析' : selectedBatchRunning ? '运行中' : selectedBatchInterrupted ? '任务已中断' : selectedBatchFailed ? '任务失败' : '待确认'}
            </span>
          </div>
          <div className="summary-pills">
            <span className="status-pill">RAW {selectedHistoryBatch.status.toUpperCase()}</span>
            <span className="status-pill">PIPELINE {selectedHistoryBatch.pipeline_status?.toUpperCase() ?? 'NONE'}</span>
            <span className="status-pill">{(selectedHistoryBatch.imported_rows ?? 0).toLocaleString()} rows</span>
            <span className="status-pill">analysis {selectedHistoryBatch.analysis_run_id ?? '-'}</span>
          </div>
          <div className={`next-action-hint ${selectedBatchFailed ? 'next-action-blocked' : selectedBatchReady ? 'next-action-success' : 'next-action-warning'}`}>
            <span>建议下一步</span>
            <strong>{recoveryRecommendation}</strong>
          </div>
          <div className="primary-action-row">
            {selectedHistoryBatch.pipeline_run_id && <button type="button" className="action-button-primary" onClick={() => { void restorePipelineForBatch(selectedHistoryBatch); }}>加载任务状态与日志</button>}
            {selectedBatchReady && <button type="button" className="action-button-done" onClick={props.onOpenAnalysis}>进入数据分析</button>}
          </div>
          {!selectedHistoryBatch.pipeline_run_id && <p className="muted-row">该批次没有关联的自动流水线日志，可能来自旧版本或手工导入。仍可按 RAW 状态选择下方恢复操作。</p>}
        </section>
      ) : <section className="panel empty-panel"><strong>先选择一个批次</strong><p>刷新列表并点击某行“选择批次”，系统只设置工作上下文，不会立即执行查询或聚合。</p></section>}
      {selectedHistoryBatch && <>
      <section className="panel form-panel raw-rebuild-panel">
        <div className="step-card-head">
          <div>
            <h3>从现有 RAW 重建全部分析结果</h3>
            <p className="muted-row">适用于规则、清洗口径或体验策略变化后重新计算。不会读取 CSV，也不会重新导入 RAW；会重跑 Quality Gate、CLEAN/DWD、DWS、ADS、V2 和 Findings。</p>
          </div>
          <span className="step-badge">RAW rebuild</span>
        </div>
        <div className="summary-pills">
          <span className="status-pill">batch {importBatchId || '-'}</span>
          <span className="status-pill">保留 RAW</span>
          <span className="status-pill">新建 analysis run</span>
          <span className="status-pill">SQL 逐条计时</span>
        </div>
        <p className="analytics-warning-banner">重建会覆盖当前批次共享的 CLEAN/DWD 与 DWS 内容，但保留 RAW 和旧 analysis run 的 ADS 结果。执行失败会将新运行标记为 failed，不会伪装成成功。</p>
        {selectedOrLoadedBatchRunning && (
          <label className="access-rule-confirmation-check" style={{ marginTop: 12 }}>
            <input
              type="checkbox"
              checked={staleTakeoverConfirmed}
              onChange={(event) => setStaleTakeoverConfirmed(event.target.checked)}
            />
            <span>原 EXE 已退出，我需要接管遗留的 running 状态。后端将检查 MySQL 活动 SQL，仍在执行时必定拒绝。</span>
          </label>
        )}
        <div className="primary-action-row" style={{ marginTop: 12 }}>
          <ActionButton
            actionKey="import_pipeline_rebuild_batch_from_raw"
            actionStates={actionStates}
            primary
            label="从 RAW 重建 CLEAN / DWS / ADS / V2"
            disabled={!importBatchId || (selectedOrLoadedBatchRunning && !staleTakeoverConfirmed)}
            onClick={rebuildCurrentBatchFromRaw}
            title={selectedOrLoadedBatchRunning && !staleTakeoverConfirmed ? '当前任务仍在运行；不要并发启动' : undefined}
          />
        </div>
        <p className="muted-row">实时日志会为核心脚本拆分出的每条 SQL 写入 RUNNING、SUCCESS 或 FAILED，并记录执行耗时、影响行数和语句摘要。</p>
      </section>
      <section className="panel form-panel">
        <div className="step-card-head">
          <div>
            <h3>复用当前批次继续分析</h3>
            <p className="muted-row">适用于 RAW、Quality Gate 和 CLEAN 已成功，但聚合或看板产物未完整的批次。不会再读 CSV，也不会重新导入 RAW。</p>
          </div>
          <span className="step-badge">Resume</span>
        </div>
        <div className="summary-pills">
          <span className="status-pill">batch {importBatchId || '-'}</span>
          <span className="status-pill">analysis {props.analysisRunId || '自动复用最新'}</span>
          <span className="status-pill">skip CSV / RAW / CLEAN</span>
          <span className="status-pill">8 个聚合子阶段</span>
        </div>
        {selectedOrLoadedBatchRunning && (
          <label className="access-rule-confirmation-check" style={{ marginTop: 12 }}>
            <input
              type="checkbox"
              checked={staleTakeoverConfirmed}
              onChange={(event) => setStaleTakeoverConfirmed(event.target.checked)}
            />
            <span>原 EXE 已退出，我需要接管遗留的 running 状态。后端将检查 MySQL 活动 SQL，仍在执行时必定拒绝。</span>
          </label>
        )}
        <div className="primary-action-row" style={{ marginTop: 12 }}>
          <ActionButton
            actionKey="import_pipeline_resume_batch"
            actionStates={actionStates}
            primary
            label="复用当前批次继续生成完整看板"
            disabled={!importBatchId || (selectedOrLoadedBatchRunning && !staleTakeoverConfirmed)}
            onClick={resumeCurrentBatch}
            title={selectedOrLoadedBatchRunning && !staleTakeoverConfirmed ? '当前任务仍在运行；不要并发启动' : undefined}
          />
        </div>
        <p className="muted-row">执行范围：用户日聚合 → 完整 DWS → 基础 ADS → App Rank → 小时趋势 → 网络热点 → 用户画像 → Lead Evidence → Final Lead（可降级）→ Module Ready。</p>
      </section>
      </>}
      </>
      )}
      {workspace === 'tasks' && (
      <>
      <section className="panel form-panel task-monitor-summary">
        <div className="step-card-head">
          <div>
            <h3>任务监控</h3>
            <p className="muted-row">任务状态和日志只在你点击加载或开启自动刷新后查询。离开本页不会启动新的数据处理任务。</p>
          </div>
          <span className={`step-badge ${stepTone(String(pipelineStatus?.status ?? 'pending'))}`}>{pipelineStatus?.status?.toUpperCase() ?? '未加载'}</span>
        </div>
        <div className="summary-pills">
          <span className="status-pill">task {pipelineRunId || '-'}</span>
          <span className="status-pill">batch {pipelineStatus?.import_batch_id ?? (importBatchId || '-')}</span>
          <span className="status-pill">step {pipelineStatus?.current_step ?? '-'}</span>
          <span className="status-pill">progress {(pipelineStatus?.percent ?? 0).toFixed(0)}%</span>
          <span className="status-pill">elapsed {formatMs(pipelineStatus?.elapsed_ms)}</span>
        </div>
        <div className="primary-action-row">
          <button type="button" className="action-button-primary" disabled={!pipelineRunId || pipelinePolling} onClick={() => { setPipelineAutoRefresh(true); void refreshPipeline(); }}>
            {pipelineStatus ? '立即刷新任务状态' : '加载最近任务状态'}
          </button>
          {!pipelineRunId && <button type="button" onClick={() => setWorkspace('batches')}>从批次库选择任务</button>}
          {['success', 'degraded'].includes(String(pipelineStatus?.status ?? '').toLowerCase()) && <button type="button" className="action-button-done" onClick={props.onOpenAnalysis}>进入数据分析</button>}
          {String(pipelineStatus?.status ?? '').toLowerCase() === 'failed' && <button type="button" onClick={() => setWorkspace('batches')}>去批次库选择恢复方式</button>}
        </div>
        {pipelineStatus?.status === 'failed' && (
          <div className="diagnostic-row-failed" style={{ padding: 12, borderRadius: 12 }}>
            <strong>失败步骤：{pipelineStatus.failed_step ?? pipelineStatus.current_step ?? '-'}</strong>
            <p className="muted-row">{pipelineStatus.error_message ?? pipelineStatus.message ?? '未返回错误详情'}</p>
            <button type="button" onClick={() => navigator.clipboard?.writeText(`${pipelineStatus.failed_step ?? ''}\n${pipelineStatus.error_message ?? ''}`)}>复制错误</button>
          </div>
        )}
        {!pipelineStatus && pipelineRunId && <p className="next-action-hint"><span>等待你的操作</span><strong>检测到本机保存的任务 ID，但尚未访问数据库。点击“加载最近任务状态”后才会读取进度和日志。</strong></p>}
      </section>
      <PipelineLogMonitor
        logs={pipelineLogs}
        status={pipelineStatus}
        polling={pipelinePolling}
        pollingError={pipelinePollingError}
        lastPollAt={pipelineLastPollAt}
        lastLogReceivedAt={pipelineLastLogAt}
        autoRefreshEnabled={pipelineAutoRefresh}
        onAutoRefreshChange={setPipelineAutoRefresh}
        onRefresh={() => refreshPipeline()}
      />
      <details className="advanced-actions">
        <summary>开发者诊断模式：手工执行流水线步骤</summary>
      <section className="panel form-panel">
        <h3>1. 导入准备</h3>
        <div className="action-row">
          <ActionButton actionKey="import_prepare_environment" actionStates={actionStates} label="测试/初始化/刷新 Catalog" onClick={prepareImportEnvironment} />
          <ActionButton actionKey="config_check_import_catalog" actionStates={actionStates} label="Catalog 健康" onClick={refreshCatalogHealth} />
        </div>
      </section>
      <section className="panel form-panel">
        <h3>2. 选择文件与批次</h3>
      <div className="form-grid import-form-grid">
        <label>
          批次名称
          <input value={batchDisplayName} onChange={(e) => setBatchDisplayName(e.target.value)} placeholder="例如：TCP 视频体验｜Claro｜2026-07-05 晚高峰" />
        </label>
        <label>
          数据类型
          <select value={dataType} onChange={(e) => {
            const next = e.target.value as ImportDataType;
            setDataType(next);
            setAccessRuleConfirmed(false);
            if (filePath && !batchDisplayName.trim()) setBatchDisplayName(defaultBatchName(next, filePath));
          }}>
            <option value="tcp">TCP / Universal Video</option><option value="game">Game</option><option value="crm">CRM Users</option><option value="coverage">FTTH Coverage</option><option value="reachability">Reachability</option>
          </select>
        </label>
        <label>
          导入方式
          <select value={importMode} onChange={(e) => setImportMode(e.target.value as 'load_data' | 'streaming_insert')}>
            <option value="load_data">LOAD DATA LOCAL INFILE</option><option value="streaming_insert">Streaming INSERT fallback</option>
          </select>
        </label>
      </div>
      <section className="file-picker-card">
        <div>
          <span>CSV 文件</span>
          <strong title={filePath}>{filePath ? fileName(filePath) : '未选择文件'}</strong>
          <small>{filePath || '请使用系统弹框选择文件。'}</small>
        </div>
        <button type="button" onClick={chooseFile}>选择 CSV 文件</button>
      </section>
      <div className="action-row">
        <ActionButton actionKey="import_probe_csv" actionStates={actionStates} label="Probe CSV" disabled={!filePath} onClick={probeCurrentFile} />
        <ActionButton actionKey="import_create_batch" actionStates={actionStates} label="创建批次" disabled={!canImport} onClick={createSelectedBatch} title={importBlockReason} />
      </div>
      {csvProbe && (
        <div className="table-like" style={{ marginTop: 12 }}>
          <div className="table-row table-head"><span>Probe</span><span>Value</span><span>Preview</span></div>
          <div className="table-row"><span>file</span><span>{csvProbe.file_name}</span><span>{csvProbe.file_size_bytes} bytes</span></div>
          <div className="table-row"><span>headers</span><span>{csvProbe.headers.length}</span><span>{csvProbe.headers.slice(0, 12).join(' | ')}</span></div>
          <div className="table-row"><span>preview</span><span>{csvProbe.preview_rows.length} rows</span><span>{csvProbe.preview_rows[0]?.slice(0, 6).join(' | ') ?? '-'}</span></div>
        </div>
      )}
      </section>
      <section className="panel form-panel">
        <h3>3. 字段映射校验</h3>
        <div className="action-row">
          <ActionButton actionKey="import_validate_mapping" actionStates={actionStates} label="映射校验" disabled={!importBatchId || !filePath} onClick={validateMapping} />
          <ActionButton actionKey="config_get_import_mappings" actionStates={actionStates} label="字段映射目录" onClick={refreshMappingCatalog} />
        </div>
      </section>
      <section className="panel form-panel">
        <h3>4. RAW 入库</h3>
      <div className="primary-action-row">
        <ActionButton actionKey="import_current_file" actionStates={actionStates} primary label="导入当前文件" disabled={!canImport} onClick={importCurrentFile} title={importBlockReason} />
      </div>
      <div className="action-row">
        <ActionButton actionKey="import_get_batch_status" actionStates={actionStates} label="刷新 RAW 状态" disabled={!importBatchId} onClick={() => refreshRawStatus()} />
      </div>
      </section>
      <section className="panel form-panel">
        <h3>5. RAW 质量检查</h3>
        <div className="action-row">
          <ActionButton actionKey="quality_run_gate" actionStates={actionStates} label="运行 Quality Gate" disabled={!importBatchId} onClick={runQualityGate} />
        </div>
      </section>
      <section className="panel form-panel">
        <h3>6. CLEAN / DWD 生成</h3>
        <div className="action-row">
          <ActionButton actionKey="etl_start_clean_job" actionStates={actionStates} label="RAW → CLEAN" disabled={!importBatchId} onClick={runCleanDwd} />
        </div>
      </section>
      <section className="panel form-panel">
        <h3>7. DWS / ADS 聚合</h3>
        <div className="primary-action-row">
          <ActionButton actionKey="import_generate_analyzable_batch" actionStates={actionStates} primary label="一键生成可分析结果" disabled={!importBatchId} onClick={generateAnalyzableBatch} />
        </div>
        <div className="action-row">
          <ActionButton actionKey="import_generate_dws_ads" actionStates={actionStates} label="单独生成 DWS/ADS" disabled={!importBatchId} onClick={runDwsAds} />
        </div>
      </section>
      <section className="panel form-panel">
        <h3>8. 模块可用性检查</h3>
        <div className="action-row">
          <ActionButton actionKey="analysis_get_module_status" actionStates={actionStates} label="刷新 Module Ready" disabled={!importBatchId} onClick={() => runAction('analysis_get_module_status', refreshModuleReady)} />
          <button type="button" onClick={props.onOpenAnalysis}>进入数据分析</button>
        </div>
      </section>
      </details>
      <details className="advanced-actions diagnostic-output-drawer">
        <summary>开发者诊断输出：映射、RAW、Quality、ETL 与表注册</summary>
      <div className="summary-pills">
        <span className="status-pill">required {mappingCounts.required}</span>
        <span className="status-pill">optional {mappingCounts.optional}</span>
        <span className="status-pill">exact {mappingCounts.exact}</span>
        <span className="status-pill">alias {mappingCounts.alias}</span>
        <span className={`status-pill ${mappingCounts.missingRequired ? 'status-failure' : 'status-success'}`}>missing_required {mappingCounts.missingRequired}</span>
        <span className="status-pill">missing_optional {mappingCounts.missingOptional}</span>
      </div>
      <p className={missingTotal ? 'muted-row status-failure-text' : 'muted-row'}>{mappingSummaryText}</p>
      <p className="muted-row">{statusMessage}</p>
      {catalogHealth.length > 0 && (
        <div className="table-like">
          <div className="table-row table-head"><span>Catalog</span><span>Value</span><span>Hint</span></div>
          {catalogHealth.map((item) => <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row"><span>{item.label}</span><span>{item.value}</span><span>{item.hint}</span></div>)}
        </div>
      )}
      <div className="table-like">
        <div className="table-row table-head"><span>Status</span><span>Count</span><span>Scope</span></div>
        {mappingSummary.map((item) => (
          <div key={`${item.label}-${item.value}`} className="table-row"><span>{item.label}</span><span>{item.value}</span><span>{item.hint}</span></div>
        ))}
        {!mappingSummary.length && <div className="table-row muted-row">未跑映射汇总。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Target</span><span>Match</span><span>Source / Required</span></div>
        {mappingResults.map((item) => {
          const parsed = parseHint(item.hint);
          const source = parsed.source ?? 'UNKNOWN';
          const required = parsed.required ?? '?';
          const matchLabel = item.value === 'matched' ? (source.trim().toLowerCase() === item.label.trim().toLowerCase() ? 'exact matched' : 'alias matched') : item.value;
          return <div key={`${item.label}-${item.value}-${item.hint}`} className={`table-row ${item.value === 'missing_required' ? 'diagnostic-row-failed' : ''}`}><span>{item.label}</span><span>{matchLabel}</span><span>{`source=${source} / required=${required}`}</span></div>;
        })}
        {!mappingResults.length && <div className="table-row muted-row">未跑映射结果。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>RAW Status</span><span>Value</span><span>Hint</span></div>
        {rawStatus.map((item) => <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row"><span>{item.label}</span><span>{item.value}</span><span>{item.hint}</span></div>)}
        {!rawStatus.length && <div className="table-row muted-row">未刷新 RAW 状态。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Quality Gate</span><span>Status</span><span>Hint</span></div>
        {qualityRows.map((item) => <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row"><span>{item.label}</span><span>{item.value}</span><span>{item.hint}</span></div>)}
        {!qualityRows.length && <div className="table-row muted-row">未运行 Quality Gate。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>ETL Jobs</span><span>Status</span><span>Hint</span></div>
        {etlJobs.map((item) => <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row"><span>{item.label}</span><span>{item.value}</span><span>{item.hint}</span></div>)}
        {!etlJobs.length && <div className="table-row muted-row">未运行 CLEAN/DWS/ADS。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Batch Tables</span><span>Rows</span><span>Status</span></div>
        {registry.map((item) => <div key={`${item.layer}-${item.logical_table_name}`} className="table-row"><span>{item.physical_table_name}</span><span>{item.row_count}</span><span>{item.status}</span></div>)}
        {!registry.length && <div className="table-row muted-row">未刷新 batch table registry。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Module Ready</span><span>Enabled</span><span>Reason</span></div>
        {moduleStatus.map((item) => <div key={`${item.module_id}-${item.enabled}`} className="table-row"><span>{item.module_name}</span><span>{item.enabled ? '可用' : '不可用'}</span><span>{item.status_text ?? '-'}</span></div>)}
        {!moduleStatus.length && <div className="table-row muted-row">未刷新模块可用性。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Profile</span><span>Value</span><span>Hint</span></div>
        {profileMetrics.map((item) => <div key={`${item.label}-${item.value}`} className="table-row"><span>{item.label}</span><span>{item.value}</span><span>{item.hint}</span></div>)}
        {!profileMetrics.length && <div className="table-row muted-row">未跑 dataset profile。</div>}
      </div>
      <details className="advanced-actions">
        <summary>字段映射目录</summary>
        <div className="table-like" style={{ marginTop: 12 }}>
          <div className="table-row table-head"><span>Mapping Catalog</span><span>Value</span><span>Hint</span></div>
          {mappingCatalog.map((item) => <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row"><span>{item.label}</span><span>{item.value}</span><span>{item.hint}</span></div>)}
          {!mappingCatalog.length && <div className="table-row muted-row">未加载字段映射目录。</div>}
        </div>
      </details>
      <small>{batch ? `current batch: ${batchDisplayName || batch.batch_display_name || batch.source_file_name} / ${batch.import_batch_id}` : 'no batch created'}</small>
      </details>
      </>
      )}
    </article>
  );
}

import { useEffect, useMemo, useState, type Dispatch, type SetStateAction } from 'react';
import type { ActionState, AnalysisContext, AnalysisContextKey, DashboardChartGroup, DashboardOverview, EtlJobStepRow, ExecutionLogEntry, FinalLeadUserRow, ImportBatchResult, ImportDataType, LeadUserRow, MetricCard, MySqlSettings, RuntimeEngine } from '../../shared/types';
import type { UiLanguage } from '../../shared/i18n';
import { workbenchApi } from './workbenchApi';

const defaultSettings: MySqlSettings = { host: '127.0.0.1', port: 3306, database: 'sa_vbp', user: 'root', secret: '123456', local_infile: true };
const PERSISTENCE_KEY = 'sa-fbb-experience-workbench.context.v3';
const dataTypes: ImportDataType[] = ['tcp', 'game', 'crm', 'coverage', 'reachability'];
const importModes = ['load_data', 'streaming_insert'] as const;

type ImportMode = typeof importModes[number];

type PersistedWorkbenchContext = {
  contextEngine?: unknown;
  settings?: Partial<Omit<MySqlSettings, 'secret'>>;
  dataType?: unknown;
  importMode?: unknown;
  filePath?: unknown;
  importBatchId?: unknown;
  batchDisplayName?: unknown;
  analysisRunId?: unknown;
  outputPath?: unknown;
  exportFinalActions?: unknown;
  language?: unknown;
  analysisContext?: unknown;
};

export type WorkbenchController = {
  runtimeEngine: RuntimeEngine;
  setRuntimeEngine: (engine: RuntimeEngine) => void;
  duckDbWorkspaceDir: string;
  duckDbWorkspaceError: string;
  duckDbSessionAuthorized: boolean;
  authorizeDuckDbSession: () => void;
  settings: MySqlSettings;
  setSettings: Dispatch<SetStateAction<MySqlSettings>>;
  dataType: ImportDataType;
  setDataType: Dispatch<SetStateAction<ImportDataType>>;
  importMode: ImportMode;
  setImportMode: Dispatch<SetStateAction<ImportMode>>;
  filePath: string;
  setFilePath: Dispatch<SetStateAction<string>>;
  importBatchId: string;
  setImportBatchId: Dispatch<SetStateAction<string>>;
  batchDisplayName: string;
  setBatchDisplayName: Dispatch<SetStateAction<string>>;
  analysisRunId: string;
  setAnalysisRunId: Dispatch<SetStateAction<string>>;
  outputPath: string;
  setOutputPath: Dispatch<SetStateAction<string>>;
  exportFinalActions: string[];
  setExportFinalActions: Dispatch<SetStateAction<string[]>>;
  log: ExecutionLogEntry[];
  batch: ImportBatchResult | null;
  setBatch: Dispatch<SetStateAction<ImportBatchResult | null>>;
  allMetrics: MetricCard[];
  dashboardCharts: DashboardChartGroup[];
  setDashboardCharts: Dispatch<SetStateAction<DashboardChartGroup[]>>;
  etlSteps: EtlJobStepRow[];
  setEtlSteps: Dispatch<SetStateAction<EtlJobStepRow[]>>;
  leads: LeadUserRow[];
  setLeads: Dispatch<SetStateAction<LeadUserRow[]>>;
  finalLeads: FinalLeadUserRow[];
  setFinalLeads: Dispatch<SetStateAction<FinalLeadUserRow[]>>;
  effectiveSettings: MySqlSettings;
  actionStates: Record<string, ActionState>;
  currentAction: string;
  lastActionMessage: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
  createBatch: (accessRuleSetId?: string) => Promise<ImportBatchResult | null>;
  clearPersistedContext: () => void;
  setOverview: Dispatch<SetStateAction<DashboardOverview | null>>;
  language: UiLanguage;
  setLanguage: Dispatch<SetStateAction<UiLanguage>>;
  analysisContext: AnalysisContext;
  analysisContextHistory: AnalysisContext[];
  applyAnalysisContext: (patch: Partial<AnalysisContext>) => void;
  removeAnalysisContext: (key: AnalysisContextKey) => void;
  clearAnalysisContext: () => void;
  backAnalysisContext: () => void;
};

function stringifyPreview(value: unknown) {
  try {
    const text = JSON.stringify(value, null, 2);
    if (typeof text !== 'string') return String(value);
    return text.length > 5000 ? `${text.slice(0, 5000)}…` : text;
  } catch {
    return String(value);
  }
}

function isBrowserStorageAvailable() {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}

function readPersistedContext(): PersistedWorkbenchContext {
  if (!isBrowserStorageAvailable()) return {};
  try {
    const raw = window.localStorage.getItem(PERSISTENCE_KEY);
    return raw ? JSON.parse(raw) as PersistedWorkbenchContext : {};
  } catch {
    return {};
  }
}

function writePersistedContext(context: PersistedWorkbenchContext) {
  if (!isBrowserStorageAvailable()) return;
  try {
    window.localStorage.setItem(PERSISTENCE_KEY, JSON.stringify(context));
  } catch {
    // Ignore quota / privacy mode failures; runtime state remains available.
  }
}

function removePersistedContext() {
  if (!isBrowserStorageAvailable()) return;
  try {
    window.localStorage.removeItem(PERSISTENCE_KEY);
  } catch {
    // Ignore privacy mode failures; runtime state can still be reset.
  }
}

function safeString(value: unknown, fallback = '') {
  return typeof value === 'string' ? value : fallback;
}

function safePort(value: unknown) {
  const parsed = typeof value === 'number' ? value : Number(value);
  return Number.isFinite(parsed) && parsed > 0 && parsed <= 65535 ? parsed : defaultSettings.port;
}

function safeDataType(value: unknown): ImportDataType {
  return dataTypes.includes(value as ImportDataType) ? value as ImportDataType : 'tcp';
}

function safeImportMode(value: unknown): ImportMode {
  return importModes.includes(value as ImportMode) ? value as ImportMode : 'load_data';
}

function safeStringArray(value: unknown) {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : [];
}

function safeLanguage(value: unknown): UiLanguage {
  return value === 'en-US' ? 'en-US' : 'zh-CN';
}

function safeAnalysisContext(value: unknown): AnalysisContext {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return {};
  const candidate = value as Record<string, unknown>;
  const result: AnalysisContext = {};
  const stringKeys: AnalysisContextKey[] = ['data_type', 'app_category', 'app_name', 'access_type', 'date_from', 'date_to', 'issue_metric', 'issue_side', 'user_key', 'server_ip', 'bras', 'network_object', 'baseline_type', 'finding_id'];
  for (const key of stringKeys) {
    if (typeof candidate[key] === 'string' && candidate[key]) {
      (result as Record<string, unknown>)[key] = candidate[key];
    }
  }
  for (const key of ['hour_from', 'hour_to'] as const) {
    if (typeof candidate[key] === 'number' && candidate[key] >= 0 && candidate[key] <= 23) result[key] = candidate[key];
  }
  return result;
}

function safeSettings(value: PersistedWorkbenchContext['settings']): MySqlSettings {
  return {
    host: safeString(value?.host, defaultSettings.host),
    port: safePort(value?.port),
    database: safeString(value?.database, defaultSettings.database),
    user: safeString(value?.user, defaultSettings.user),
    secret: defaultSettings.secret,
    local_infile: typeof value?.local_infile === 'boolean' ? value.local_infile : defaultSettings.local_infile,
  };
}

const persisted = readPersistedContext();
const persistedDuckDbContext = persisted.contextEngine === 'duckdb';

export function useWorkbenchController(): WorkbenchController {
  const [runtimeEngineState, setRuntimeEngineState] = useState<RuntimeEngine>('duckdb');
  const [duckDbWorkspaceDir, setDuckDbWorkspaceDir] = useState('');
  const [duckDbWorkspaceError, setDuckDbWorkspaceError] = useState('');
  const [duckDbSessionAuthorized, setDuckDbSessionAuthorized] = useState(false);
  const [settings, setSettings] = useState<MySqlSettings>(safeSettings(persisted.settings));
  const [dataType, setDataType] = useState<ImportDataType>(safeDataType(persisted.dataType));
  const [importMode, setImportMode] = useState<ImportMode>(safeImportMode(persisted.importMode));
  const [filePath, setFilePath] = useState('');
  const [importBatchId, setImportBatchId] = useState(persistedDuckDbContext ? safeString(persisted.importBatchId) : '');
  const [batchDisplayName, setBatchDisplayName] = useState(persistedDuckDbContext ? safeString(persisted.batchDisplayName) : '');
  const [analysisRunId, setAnalysisRunId] = useState(persistedDuckDbContext ? safeString(persisted.analysisRunId) : '');
  const [outputPath, setOutputPath] = useState(safeString(persisted.outputPath, 'leads_export.csv'));
  const [exportFinalActions, setExportFinalActions] = useState<string[]>(safeStringArray(persisted.exportFinalActions));
  const [language, setLanguage] = useState<UiLanguage>(safeLanguage(persisted.language));
  const [analysisContext, setAnalysisContext] = useState<AnalysisContext>(persistedDuckDbContext ? safeAnalysisContext(persisted.analysisContext) : {});
  const [analysisContextHistory, setAnalysisContextHistory] = useState<AnalysisContext[]>([]);
  const [log, setLog] = useState<ExecutionLogEntry[]>([]);
  const [batch, setBatch] = useState<ImportBatchResult | null>(null);
  const [metrics, setMetrics] = useState<MetricCard[]>([]);
  const [overview, setOverview] = useState<DashboardOverview | null>(null);
  const [dashboardCharts, setDashboardCharts] = useState<DashboardChartGroup[]>([]);
  const [etlSteps, setEtlSteps] = useState<EtlJobStepRow[]>([]);
  const [leads, setLeads] = useState<LeadUserRow[]>([]);
  const [finalLeads, setFinalLeads] = useState<FinalLeadUserRow[]>([]);
  const [actionStates, setActionStates] = useState<Record<string, ActionState>>({});
  const [currentAction, setCurrentAction] = useState('');
  const [lastActionMessage, setLastActionMessage] = useState('等待操作。');
  const effectiveSettings = useMemo(() => ({ ...settings, local_infile: importMode === 'load_data' }), [settings, importMode]);
  const allMetrics = useMemo(() => overview?.metrics ?? metrics, [overview, metrics]);

  useEffect(() => {
    let cancelled = false;
    workbenchApi.defaultDuckDbWorkspace()
      .then((next) => {
        if (cancelled) return;
        setDuckDbWorkspaceDir(next.workspace_dir);
        setDuckDbWorkspaceError('');
      })
      .catch((error) => {
        if (cancelled) return;
        setDuckDbWorkspaceDir('');
        setDuckDbWorkspaceError(error instanceof Error ? error.message : String(error));
      });
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    const { secret: _secret, ...persistableSettings } = settings;
    writePersistedContext({ contextEngine: runtimeEngineState, settings: persistableSettings, dataType, importMode, filePath, importBatchId, batchDisplayName, analysisRunId, outputPath, exportFinalActions, language, analysisContext });
  }, [runtimeEngineState, settings, dataType, importMode, filePath, importBatchId, batchDisplayName, analysisRunId, outputPath, exportFinalActions, language, analysisContext]);

  function setRuntimeEngine(engine: RuntimeEngine) {
    if (engine === runtimeEngineState) return;
    setRuntimeEngineState(engine);
    setImportBatchId('');
    setBatchDisplayName('');
    setAnalysisRunId('');
    setBatch(null);
    setDuckDbSessionAuthorized(false);
    setAnalysisContext({});
    setAnalysisContextHistory([]);
    setLastActionMessage(engine === 'duckdb'
      ? '已切换到 DuckDB 本地运行时；不会读取 MySQL 批次。'
      : '已显式进入 MySQL 兼容模式。');
  }

  function updateAnalysisContext(next: AnalysisContext) {
    setAnalysisContextHistory((history) => [...history, analysisContext].slice(-30));
    setAnalysisContext(next);
  }

  function applyAnalysisContext(patch: Partial<AnalysisContext>) {
    const next = { ...analysisContext, ...patch };
    for (const key of Object.keys(next) as AnalysisContextKey[]) {
      const value = next[key];
      if (value === '' || value === undefined || value === null) delete next[key];
    }
    if (JSON.stringify(next) !== JSON.stringify(analysisContext)) updateAnalysisContext(next);
  }

  function removeAnalysisContext(key: AnalysisContextKey) {
    if (!(key in analysisContext)) return;
    const next = { ...analysisContext };
    delete next[key];
    updateAnalysisContext(next);
  }

  function clearAnalysisContext() {
    if (Object.keys(analysisContext).length) updateAnalysisContext({});
  }

  function backAnalysisContext() {
    setAnalysisContextHistory((history) => {
      const previous = history[history.length - 1];
      if (previous) setAnalysisContext(previous);
      return previous ? history.slice(0, -1) : history;
    });
  }

  function appendLog(entry: ExecutionLogEntry) { setLog((items) => [entry, ...items].slice(0, 400)); }
  function setActionState(label: string, state: ActionState) {
    setActionStates((items) => ({ ...items, [label]: state }));
  }
  function clearPersistedContext() {
    const startedAt = new Date();
    removePersistedContext();
    setRuntimeEngineState('duckdb');
    setSettings(defaultSettings);
    setDataType('tcp');
    setImportMode('load_data');
    setFilePath('');
    setImportBatchId('');
    setBatchDisplayName('');
    setAnalysisRunId('');
    setOutputPath('leads_export.csv');
    setExportFinalActions([]);
    setLanguage('zh-CN');
    setAnalysisContext({});
    setAnalysisContextHistory([]);
    setBatch(null);
    setDuckDbSessionAuthorized(false);
    setMetrics([]);
    setOverview(null);
    setDashboardCharts([]);
    setEtlSteps([]);
    setLeads([]);
    setFinalLeads([]);
    setActionStates({});
    setCurrentAction('');
    setLastActionMessage('本地上下文已清除。');
    appendLog({
      id: `${Date.now()}-clear-local-context`,
      command: 'clear_local_context',
      status: 'success',
      started_at: startedAt.toISOString(),
      finished_at: new Date().toISOString(),
      duration_ms: 0,
      message: 'Local workbench context was cleared and reset to defaults. MySQL password returned to the built-in default and was not persisted.',
    });
  }
  async function runAction(label: string, action: () => Promise<unknown>) {
    const startedAt = new Date();
    const startedMs = Date.now();
    setCurrentAction(label);
    setLastActionMessage(`正在执行：${label}`);
    setActionState(label, { status: 'running', started_at: startedAt.toISOString(), message: 'Running...' });
    try {
      const result = await action();
      const finishedAt = new Date();
      const duration = finishedAt.getTime() - startedMs;
      const message = 'Command completed successfully.';
      setActionState(label, { status: 'success', started_at: startedAt.toISOString(), finished_at: finishedAt.toISOString(), duration_ms: duration, message });
      setLastActionMessage(`${label} 已完成。`);
      appendLog({
        id: `${startedMs}-${label}`,
        command: label,
        status: 'success',
        started_at: startedAt.toISOString(),
        finished_at: finishedAt.toISOString(),
        duration_ms: duration,
        message,
        result_preview: stringifyPreview(result),
      });
      return result;
    } catch (error) {
      const finishedAt = new Date();
      const duration = finishedAt.getTime() - startedMs;
      const message = error instanceof Error ? error.message : String(error);
      setActionState(label, { status: 'failure', started_at: startedAt.toISOString(), finished_at: finishedAt.toISOString(), duration_ms: duration, message });
      setLastActionMessage(`${label} 失败：${message}`);
      appendLog({
        id: `${startedMs}-${label}`,
        command: label,
        status: 'failure',
        started_at: startedAt.toISOString(),
        finished_at: finishedAt.toISOString(),
        duration_ms: duration,
        message,
      });
      return null;
    } finally {
      setCurrentAction('');
    }
  }
  async function loadMetrics(label: string, action: () => Promise<MetricCard[]>) {
    const result = await runAction(label, action);
    if (Array.isArray(result)) setMetrics(result as MetricCard[]);
    return Array.isArray(result) ? (result as MetricCard[]) : [];
  }

  async function createBatch(accessRuleSetId?: string) {
    const result = await runAction('import_create_batch', () => workbenchApi.createBatch(effectiveSettings, dataType, filePath, batchDisplayName, accessRuleSetId));
    if (result && typeof result === 'object' && 'import_batch_id' in result) {
      const next = result as ImportBatchResult;
      setBatch(next);
      setImportBatchId(next.import_batch_id);
      return next;
    }
    return null;
  }

  return { runtimeEngine: runtimeEngineState, setRuntimeEngine, duckDbWorkspaceDir, duckDbWorkspaceError, duckDbSessionAuthorized, authorizeDuckDbSession: () => setDuckDbSessionAuthorized(true), settings, setSettings, dataType, setDataType, importMode, setImportMode, filePath, setFilePath, importBatchId, setImportBatchId, batchDisplayName, setBatchDisplayName, analysisRunId, setAnalysisRunId, outputPath, setOutputPath, exportFinalActions, setExportFinalActions, log, batch, setBatch, allMetrics, dashboardCharts, setDashboardCharts, etlSteps, setEtlSteps, leads, setLeads, finalLeads, setFinalLeads, effectiveSettings, actionStates, currentAction, lastActionMessage, runAction, loadMetrics, createBatch, clearPersistedContext, setOverview, language, setLanguage, analysisContext, analysisContextHistory, applyAnalysisContext, removeAnalysisContext, clearAnalysisContext, backAnalysisContext };
}

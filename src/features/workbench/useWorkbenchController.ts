import { useEffect, useMemo, useState, type Dispatch, type SetStateAction } from 'react';
import type { DashboardChartGroup, DashboardOverview, EtlJobStepRow, ExecutionLogEntry, FinalLeadUserRow, ImportBatchResult, ImportDataType, LeadUserRow, MetricCard, MySqlSettings } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

const defaultSettings: MySqlSettings = { host: '127.0.0.1', port: 3306, database: 'sa_vbp', user: 'root', secret: '', local_infile: true };
const PERSISTENCE_KEY = 'sa-fbb-experience-workbench.context.v1';
const dataTypes: ImportDataType[] = ['tcp', 'game', 'crm', 'coverage', 'reachability'];
const importModes = ['load_data', 'streaming_insert'] as const;

type ImportMode = typeof importModes[number];

type PersistedWorkbenchContext = {
  settings?: Partial<Omit<MySqlSettings, 'secret'>>;
  dataType?: unknown;
  importMode?: unknown;
  filePath?: unknown;
  importBatchId?: unknown;
  analysisRunId?: unknown;
  outputPath?: unknown;
  exportFinalActions?: unknown;
};

export type WorkbenchController = {
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
  analysisRunId: string;
  setAnalysisRunId: Dispatch<SetStateAction<string>>;
  outputPath: string;
  setOutputPath: Dispatch<SetStateAction<string>>;
  exportFinalActions: string[];
  setExportFinalActions: Dispatch<SetStateAction<string[]>>;
  log: ExecutionLogEntry[];
  batch: ImportBatchResult | null;
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
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
  createBatch: () => Promise<void>;
  clearPersistedContext: () => void;
  setOverview: Dispatch<SetStateAction<DashboardOverview | null>>;
};

function stringifyPreview(value: unknown) {
  try {
    const text = JSON.stringify(value);
    if (typeof text !== 'string') return String(value);
    return text.length > 1200 ? `${text.slice(0, 1200)}…` : text;
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

function safeSettings(value: PersistedWorkbenchContext['settings']): MySqlSettings {
  return {
    host: safeString(value?.host, defaultSettings.host),
    port: safePort(value?.port),
    database: safeString(value?.database, defaultSettings.database),
    user: safeString(value?.user, defaultSettings.user),
    secret: '',
    local_infile: typeof value?.local_infile === 'boolean' ? value.local_infile : defaultSettings.local_infile,
  };
}

const persisted = readPersistedContext();

export function useWorkbenchController(): WorkbenchController {
  const [settings, setSettings] = useState<MySqlSettings>(safeSettings(persisted.settings));
  const [dataType, setDataType] = useState<ImportDataType>(safeDataType(persisted.dataType));
  const [importMode, setImportMode] = useState<ImportMode>(safeImportMode(persisted.importMode));
  const [filePath, setFilePath] = useState(safeString(persisted.filePath));
  const [importBatchId, setImportBatchId] = useState(safeString(persisted.importBatchId));
  const [analysisRunId, setAnalysisRunId] = useState(safeString(persisted.analysisRunId, 'RUN_MANUAL_001'));
  const [outputPath, setOutputPath] = useState(safeString(persisted.outputPath, 'leads_export.csv'));
  const [exportFinalActions, setExportFinalActions] = useState<string[]>(safeStringArray(persisted.exportFinalActions));
  const [log, setLog] = useState<ExecutionLogEntry[]>([]);
  const [batch, setBatch] = useState<ImportBatchResult | null>(null);
  const [metrics, setMetrics] = useState<MetricCard[]>([]);
  const [overview, setOverview] = useState<DashboardOverview | null>(null);
  const [dashboardCharts, setDashboardCharts] = useState<DashboardChartGroup[]>([]);
  const [etlSteps, setEtlSteps] = useState<EtlJobStepRow[]>([]);
  const [leads, setLeads] = useState<LeadUserRow[]>([]);
  const [finalLeads, setFinalLeads] = useState<FinalLeadUserRow[]>([]);
  const effectiveSettings = useMemo(() => ({ ...settings, local_infile: importMode === 'load_data' }), [settings, importMode]);
  const allMetrics = useMemo(() => overview?.metrics ?? metrics, [overview, metrics]);

  useEffect(() => {
    const { secret: _secret, ...persistableSettings } = settings;
    writePersistedContext({ settings: persistableSettings, dataType, importMode, filePath, importBatchId, analysisRunId, outputPath, exportFinalActions });
  }, [settings, dataType, importMode, filePath, importBatchId, analysisRunId, outputPath, exportFinalActions]);

  function appendLog(entry: ExecutionLogEntry) { setLog((items) => [entry, ...items].slice(0, 120)); }
  function clearPersistedContext() {
    const startedAt = new Date();
    removePersistedContext();
    setSettings(defaultSettings);
    setDataType('tcp');
    setImportMode('load_data');
    setFilePath('');
    setImportBatchId('');
    setAnalysisRunId('RUN_MANUAL_001');
    setOutputPath('leads_export.csv');
    setExportFinalActions([]);
    setBatch(null);
    appendLog({
      id: `${Date.now()}-clear-local-context`,
      command: 'clear_local_context',
      status: 'success',
      started_at: startedAt.toISOString(),
      finished_at: new Date().toISOString(),
      duration_ms: 0,
      message: 'Local workbench context was cleared and reset to defaults. MySQL password was never persisted.',
    });
  }
  async function runAction(label: string, action: () => Promise<unknown>) {
    const startedAt = new Date();
    const startedMs = Date.now();
    try {
      const result = await action();
      const finishedAt = new Date();
      appendLog({
        id: `${startedMs}-${label}`,
        command: label,
        status: 'success',
        started_at: startedAt.toISOString(),
        finished_at: finishedAt.toISOString(),
        duration_ms: finishedAt.getTime() - startedMs,
        message: 'Command completed successfully.',
        result_preview: stringifyPreview(result),
      });
      return result;
    } catch (error) {
      const finishedAt = new Date();
      appendLog({
        id: `${startedMs}-${label}`,
        command: label,
        status: 'failure',
        started_at: startedAt.toISOString(),
        finished_at: finishedAt.toISOString(),
        duration_ms: finishedAt.getTime() - startedMs,
        message: error instanceof Error ? error.message : String(error),
      });
      return null;
    }
  }
  async function loadMetrics(label: string, action: () => Promise<MetricCard[]>) {
    const result = await runAction(label, action);
    if (Array.isArray(result)) setMetrics(result as MetricCard[]);
  }

  async function createBatch() {
    const result = await runAction('import_create_batch', () => workbenchApi.createBatch(effectiveSettings, dataType, filePath));
    if (result && typeof result === 'object' && 'import_batch_id' in result) {
      const next = result as ImportBatchResult;
      setBatch(next);
      setImportBatchId(next.import_batch_id);
    }
  }

  return { settings, setSettings, dataType, setDataType, importMode, setImportMode, filePath, setFilePath, importBatchId, setImportBatchId, analysisRunId, setAnalysisRunId, outputPath, setOutputPath, exportFinalActions, setExportFinalActions, log, batch, allMetrics, dashboardCharts, setDashboardCharts, etlSteps, setEtlSteps, leads, setLeads, finalLeads, setFinalLeads, effectiveSettings, runAction, loadMetrics, createBatch, clearPersistedContext, setOverview };
}

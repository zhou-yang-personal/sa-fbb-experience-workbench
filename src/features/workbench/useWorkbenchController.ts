import { useMemo, useState } from 'react';
import type { DashboardChartGroup, DashboardOverview, EtlJobStepRow, ExecutionLogEntry, FinalLeadUserRow, ImportBatchResult, ImportDataType, LeadUserRow, MetricCard, MySqlSettings } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

const defaultSettings: MySqlSettings = { host: '127.0.0.1', port: 3306, database: 'sa_vbp', user: 'root', secret: '', local_infile: true };

function stringifyPreview(value: unknown) {
  try {
    const text = JSON.stringify(value);
    return text.length > 1200 ? `${text.slice(0, 1200)}…` : text;
  } catch {
    return String(value);
  }
}

export function useWorkbenchController() {
  const [settings, setSettings] = useState<MySqlSettings>(defaultSettings);
  const [dataType, setDataType] = useState<ImportDataType>('tcp');
  const [importMode, setImportMode] = useState<'load_data' | 'streaming_insert'>('load_data');
  const [filePath, setFilePath] = useState('');
  const [importBatchId, setImportBatchId] = useState('');
  const [analysisRunId, setAnalysisRunId] = useState('RUN_MANUAL_001');
  const [outputPath, setOutputPath] = useState('leads_export.csv');
  const [exportFinalActions, setExportFinalActions] = useState<string[]>([]);
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

  function appendLog(entry: ExecutionLogEntry) { setLog((items) => [entry, ...items].slice(0, 120)); }
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

  return { settings, setSettings, dataType, setDataType, importMode, setImportMode, filePath, setFilePath, importBatchId, setImportBatchId, analysisRunId, setAnalysisRunId, outputPath, setOutputPath, exportFinalActions, setExportFinalActions, log, batch, allMetrics, dashboardCharts, setDashboardCharts, etlSteps, setEtlSteps, leads, setLeads, finalLeads, setFinalLeads, effectiveSettings, runAction, loadMetrics, createBatch, setOverview };
}

import { useMemo, useState } from 'react';
import type { DashboardOverview, FinalLeadUserRow, ImportBatchResult, ImportDataType, LeadUserRow, MetricCard, MySqlSettings } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

const defaultSettings: MySqlSettings = { host: '127.0.0.1', port: 3306, database: 'sa_vbp', user: 'root', secret: '', local_infile: true };

export function useWorkbenchController() {
  const [settings, setSettings] = useState<MySqlSettings>(defaultSettings);
  const [dataType, setDataType] = useState<ImportDataType>('tcp');
  const [importMode, setImportMode] = useState<'load_data' | 'streaming_insert'>('load_data');
  const [filePath, setFilePath] = useState('');
  const [importBatchId, setImportBatchId] = useState('');
  const [analysisRunId, setAnalysisRunId] = useState('RUN_MANUAL_001');
  const [outputPath, setOutputPath] = useState('leads_export.csv');
  const [exportFinalActions, setExportFinalActions] = useState<string[]>([]);
  const [log, setLog] = useState<string[]>([]);
  const [batch, setBatch] = useState<ImportBatchResult | null>(null);
  const [metrics, setMetrics] = useState<MetricCard[]>([]);
  const [overview, setOverview] = useState<DashboardOverview | null>(null);
  const [leads, setLeads] = useState<LeadUserRow[]>([]);
  const [finalLeads, setFinalLeads] = useState<FinalLeadUserRow[]>([]);
  const effectiveSettings = useMemo(() => ({ ...settings, local_infile: importMode === 'load_data' }), [settings, importMode]);
  const allMetrics = useMemo(() => overview?.metrics ?? metrics, [overview, metrics]);

  function appendLog(message: string) { setLog((items) => [`${new Date().toLocaleTimeString()} ${message}`, ...items].slice(0, 80)); }
  async function runAction(label: string, action: () => Promise<unknown>) {
    try { const result = await action(); appendLog(`${label}: ${JSON.stringify(result)}`); return result; }
    catch (error) { appendLog(`${label} failed: ${error instanceof Error ? error.message : String(error)}`); return null; }
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

  return { settings, setSettings, dataType, setDataType, importMode, setImportMode, filePath, setFilePath, importBatchId, setImportBatchId, analysisRunId, setAnalysisRunId, outputPath, setOutputPath, exportFinalActions, setExportFinalActions, log, batch, allMetrics, leads, setLeads, finalLeads, setFinalLeads, effectiveSettings, runAction, loadMetrics, createBatch, setOverview };
}

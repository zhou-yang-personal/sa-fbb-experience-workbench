import type { ImportBatchResult, ImportDataType, MySqlSettings } from '../../shared/types';

type Props = {
  settings: MySqlSettings;
  dataType: ImportDataType;
  importMode: 'load_data' | 'streaming_insert';
  filePath: string;
  importBatchId: string;
  analysisRunId: string;
  outputPath: string;
  batch: ImportBatchResult | null;
};

function shortValue(value: string, fallback = '-') {
  if (!value.trim()) return fallback;
  return value.length > 46 ? `${value.slice(0, 18)}…${value.slice(-22)}` : value;
}

function dbLabel(settings: MySqlSettings) {
  const host = settings.host || 'host?';
  const database = settings.database || 'database?';
  const port = settings.port || 3306;
  return `${host}:${port}/${database}`;
}

export function WorkbenchContextBar({ settings, dataType, importMode, filePath, importBatchId, analysisRunId, outputPath, batch }: Props) {
  const batchStatus = batch?.status ?? (importBatchId ? 'manual' : 'not created');
  return (
    <section className="context-bar" aria-label="Current execution context">
      <div className="context-item context-primary">
        <span>Batch</span>
        <strong>{shortValue(importBatchId, 'no batch')}</strong>
        <small>{batchStatus}</small>
      </div>
      <div className="context-item">
        <span>Run</span>
        <strong>{shortValue(analysisRunId, 'no run')}</strong>
        <small>analysis_run_id</small>
      </div>
      <div className="context-item">
        <span>Data</span>
        <strong>{dataType.toUpperCase()}</strong>
        <small>{importMode === 'load_data' ? 'LOAD DATA' : 'Streaming INSERT'}</small>
      </div>
      <div className="context-item context-wide">
        <span>Source</span>
        <strong title={filePath}>{shortValue(filePath, 'no file selected')}</strong>
        <small>{dbLabel(settings)}</small>
      </div>
      <div className="context-item context-wide">
        <span>Export</span>
        <strong title={outputPath}>{shortValue(outputPath, 'no output path')}</strong>
        <small>CSV output target</small>
      </div>
    </section>
  );
}

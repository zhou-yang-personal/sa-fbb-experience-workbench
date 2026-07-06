import { useEffect, useState } from 'react';
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

export function SystemDiagnostics({ settings, importBatchId, analysisRunId, dataType }: Props) {
  const [registry, setRegistry] = useState<BatchTableRegistryRow[]>([]);
  const [statusRows, setStatusRows] = useState<ModuleStatusRow[]>([]);
  const [mappingIssues, setMappingIssues] = useState<MetricCard[]>([]);
  const [qualityFailed, setQualityFailed] = useState<MetricCard[]>([]);
  const [etlFailed, setEtlFailed] = useState<MetricCard[]>([]);
  const [message, setMessage] = useState('等待刷新诊断数据。');

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
      `registry=${registry.length}`,
      `modules=${statusRows.length}`,
      ...registry.map((row) => `${row.logical_table_name} => ${row.physical_table_name} (${row.row_count})`),
      ...statusRows.map((row) => `${row.module_name}: enabled=${row.enabled}; ${row.status_text ?? '-'}`),
    ];
    return lines.join('\n');
  }

  async function refresh() {
    if (!importBatchId.trim()) {
      setRegistry([]);
      setStatusRows([]);
      setMappingIssues([]);
      setQualityFailed([]);
      setEtlFailed([]);
      setMessage('请先选择批次。');
      return;
    }
    const [registryRows, status, mapping, quality, etl] = await Promise.all([
      workbenchApi.batchTableRegistry(settings, importBatchId),
      workbenchApi.moduleStatus(settings, importBatchId, analysisRunId || undefined),
      mappingApi.results(settings, importBatchId, dataType),
      qualityApi.failedResults(settings, importBatchId),
      jobApi.failedSteps(settings, importBatchId),
    ]);
    setRegistry(registryRows);
    setStatusRows(status);
    setMappingIssues(mapping.filter((item) => item.value === 'missing_required'));
    setQualityFailed(quality);
    setEtlFailed(etl);
    setMessage(`registry=${registryRows.length}, modules=${status.length}`);
  }

  useEffect(() => {
    void refresh().catch((error) => {
      setMessage(error instanceof Error ? error.message : String(error));
    });
  }, [settings.host, settings.port, settings.database, settings.user, importBatchId, analysisRunId, dataType]);

  return (
    <section className="panel form-panel">
      <div className="step-card-head">
        <div>
          <h2>系统诊断</h2>
          <p className="hero-text">查看 batch table registry、module disabled reason、row count 和批次可用性。</p>
        </div>
        <div className="action-row">
          <button type="button" onClick={() => void refresh()}>刷新诊断</button>
          <button type="button" onClick={() => copyText(buildDiagnosticText())}>复制诊断包</button>
        </div>
      </div>
      <p className="muted-row">{message}</p>
      <div className="summary-pills">
        <span className="status-pill">registry {registry.length}</span>
        <span className="status-pill">modules {statusRows.length}</span>
        <span className="status-pill">batch {importBatchId || 'missing'}</span>
      </div>
      <div className="table-like">
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

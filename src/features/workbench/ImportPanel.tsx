import { useMemo, useState } from 'react';
import type { ImportBatchResult, ImportDataType, MetricCard, MySqlSettings } from '../../shared/types';
import { mappingApi } from './mappingApi';
import { profileApi } from './profileApi';
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
  batch: ImportBatchResult | null;
  createBatch: () => Promise<void>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
};

function parseHint(hint: string) {
  return hint.split(',').reduce<Record<string, string>>((acc, part) => {
    const [key, value] = part.split('=').map((item) => item.trim());
    if (key) acc[key] = value ?? '';
    return acc;
  }, {});
}

export function ImportPanel(props: Props) {
  const { settings, effectiveSettings, dataType, setDataType, importMode, setImportMode, filePath, setFilePath, importBatchId, batch, createBatch, runAction, loadMetrics } = props;
  const [mappingSummary, setMappingSummary] = useState<MetricCard[]>([]);
  const [mappingResults, setMappingResults] = useState<MetricCard[]>([]);
  const [profileMetrics, setProfileMetrics] = useState<MetricCard[]>([]);
  const [mappingCatalog, setMappingCatalog] = useState<MetricCard[]>([]);
  const [statusMessage, setStatusMessage] = useState('未选择文件');

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

  async function refreshMappingSummary() {
    const result = await loadMetrics('import_get_mapping_summary', () => mappingApi.summary(settings, importBatchId, dataType));
    setMappingSummary(result);
    setStatusMessage(result.length ? '映射汇总已刷新' : '映射汇总为空');
  }

  async function refreshMappingResults() {
    const result = await loadMetrics('import_get_mapping_results', () => mappingApi.results(settings, importBatchId, dataType));
    setMappingResults(result);
    setStatusMessage(result.length ? '映射结果已刷新' : '映射结果为空');
  }

  async function refreshProfile() {
    const result = await loadMetrics('dataset_profile_get', () => profileApi.get(settings, importBatchId, dataType));
    setProfileMetrics(result);
    setStatusMessage(result.length ? '数据画像已刷新' : '当前无 profile');
  }

  async function refreshMappingCatalog() {
    const result = await loadMetrics('config_get_import_mappings', () => workbenchApi.importMappings(settings, dataType));
    setMappingCatalog(result);
    setStatusMessage(result.length ? '字段映射目录已刷新' : '字段映射目录为空');
  }

  async function validateMapping() {
    await runAction('import_validate_mapping', () => workbenchApi.validateMapping(settings, importBatchId, dataType, filePath));
    await refreshMappingSummary();
    await refreshMappingResults();
  }

  async function refreshRawStatus() {
    const result = await loadMetrics('import_get_batch_status', () => workbenchApi.importStatus(settings, importBatchId));
    setStatusMessage(result.length ? '导入状态已刷新' : '当前没有导入状态');
  }

  async function refreshRawLoad() {
    await runAction('import_start_raw_load', () => workbenchApi.loadRaw(effectiveSettings, importBatchId, dataType, filePath, importMode));
    await refreshRawStatus();
    await refreshProfile();
  }

  return (
    <article className="panel form-panel">
      <h2>CSV 导入中心</h2>
      <p className="hero-text">先看字段映射和画像，再决定是否进入 RAW 入库。</p>
      <select value={dataType} onChange={(e) => setDataType(e.target.value as ImportDataType)}>
        <option value="tcp">TCP</option><option value="game">Game</option><option value="crm">CRM Users</option><option value="coverage">FTTH Coverage</option><option value="reachability">Reachability</option>
      </select>
      <select value={importMode} onChange={(e) => setImportMode(e.target.value as 'load_data' | 'streaming_insert')}>
        <option value="load_data">LOAD DATA LOCAL INFILE</option><option value="streaming_insert">Streaming INSERT fallback</option>
      </select>
      <input value={filePath} onChange={(e) => setFilePath(e.target.value)} placeholder="CSV absolute path" />
      <div className="action-row">
        <button onClick={() => runAction('import_probe_csv', () => workbenchApi.probeCsv(filePath))}>Probe</button>
        <button onClick={createBatch}>创建批次</button>
        <button onClick={validateMapping}>映射校验</button>
        <button onClick={refreshMappingSummary}>映射汇总</button>
        <button onClick={refreshMappingResults}>映射结果</button>
        <button onClick={() => runAction('dataset_profile_refresh', () => profileApi.refresh(settings, importBatchId, dataType)).then(refreshProfile)}>刷新画像</button>
        <button onClick={refreshProfile}>查看画像</button>
        <button onClick={refreshRawLoad}>RAW 入库</button>
        <button onClick={refreshRawStatus}>刷新导入状态</button>
        <button onClick={refreshMappingCatalog}>字段映射</button>
      </div>
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
      <div className="table-like">
        <div className="table-row table-head"><span>Status</span><span>Count</span><span>Scope</span></div>
        {mappingSummary.map((item) => (
          <div key={`${item.label}-${item.value}`} className="table-row">
            <span>{item.label}</span>
            <span>{item.value}</span>
            <span>{item.hint}</span>
          </div>
        ))}
        {!mappingSummary.length && <div className="table-row muted-row">未跑映射汇总。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Target</span><span>Match</span><span>Source / Required</span></div>
        {mappingResults.map((item) => {
          const parsed = parseHint(item.hint);
          const source = parsed.source ?? 'UNKNOWN';
          const required = parsed.required ?? '?';
          const matchLabel = item.value === 'matched'
            ? (source.trim().toLowerCase() === item.label.trim().toLowerCase() ? 'exact matched' : 'alias matched')
            : item.value;
          return (
            <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row">
              <span>{item.label}</span>
              <span>{matchLabel}</span>
              <span>{`source=${source} / required=${required}`}</span>
            </div>
          );
        })}
        {!mappingResults.length && <div className="table-row muted-row">未跑映射结果。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Profile</span><span>Value</span><span>Hint</span></div>
        {profileMetrics.map((item) => (
          <div key={`${item.label}-${item.value}`} className="table-row">
            <span>{item.label}</span>
            <span>{item.value}</span>
            <span>{item.hint}</span>
          </div>
        ))}
        {!profileMetrics.length && <div className="table-row muted-row">未跑 dataset profile。</div>}
      </div>
      <div className="table-like" style={{ marginTop: 12 }}>
        <div className="table-row table-head"><span>Mapping Catalog</span><span>Value</span><span>Hint</span></div>
        {mappingCatalog.map((item) => (
          <div key={`${item.label}-${item.value}-${item.hint}`} className="table-row">
            <span>{item.label}</span>
            <span>{item.value}</span>
            <span>{item.hint}</span>
          </div>
        ))}
        {!mappingCatalog.length && <div className="table-row muted-row">未加载字段映射目录。</div>}
      </div>
      <small>{batch ? `current batch: ${batch.import_batch_id}` : 'no batch created'}</small>
    </article>
  );
}

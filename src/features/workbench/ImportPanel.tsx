import { useMemo, useState } from 'react';
import type { ActionState, ImportBatchResult, ImportDataType, MetricCard, MySqlSettings } from '../../shared/types';
import { ActionButton } from './ActionButton';
import { selectCsvFile } from './fileDialogs';
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
  setImportBatchId: (value: string) => void;
  batchDisplayName: string;
  setBatchDisplayName: (value: string) => void;
  batch: ImportBatchResult | null;
  setBatch: (value: ImportBatchResult | null) => void;
  createBatch: () => Promise<ImportBatchResult | null>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
  actionStates: Record<string, ActionState>;
};

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

export function ImportPanel(props: Props) {
  const { settings, dataType, setDataType, importMode, setImportMode, filePath, setFilePath, importBatchId, setImportBatchId, batchDisplayName, setBatchDisplayName, batch, setBatch, createBatch, runAction, loadMetrics, actionStates } = props;
  const [mappingSummary, setMappingSummary] = useState<MetricCard[]>([]);
  const [mappingResults, setMappingResults] = useState<MetricCard[]>([]);
  const [profileMetrics, setProfileMetrics] = useState<MetricCard[]>([]);
  const [mappingCatalog, setMappingCatalog] = useState<MetricCard[]>([]);
  const [catalogHealth, setCatalogHealth] = useState<MetricCard[]>([]);
  const [statusMessage, setStatusMessage] = useState('请选择 CSV 文件，并确认本次导入批次名称。');

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
  const canImport = Boolean(filePath.trim()) && Boolean(batchDisplayName.trim());

  async function chooseFile() {
    const selected = await selectCsvFile();
    if (selected) {
      setFilePath(selected);
      if (!batchDisplayName.trim()) setBatchDisplayName(defaultBatchName(dataType, selected));
      setStatusMessage(`已选择文件：${fileName(selected)}`);
    }
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

  async function validateMapping() {
    await runAction('import_validate_mapping', () => workbenchApi.validateMapping(settings, importBatchId, dataType, filePath));
    await refreshMappingSummary();
    await refreshMappingResults();
  }

  async function refreshRawStatus(batchId = importBatchId) {
    const result = await loadMetrics('import_get_batch_status', () => workbenchApi.importStatus(settings, batchId));
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
      const health = await workbenchApi.checkImportCatalog(settings);
      setCatalogHealth(health);
      const stale = health.some((item) => item.label === 'stale_catalog' && item.value === 'yes');
      if (stale) {
        await workbenchApi.seedConfig(settings);
        const repaired = await workbenchApi.checkImportCatalog(settings);
        setCatalogHealth(repaired);
      }
      const result = await workbenchApi.importCurrentFile(settings, dataType, filePath, batchDisplayName, importMode);
      setBatch(result.batch);
      setImportBatchId(result.batch.import_batch_id);
      setMappingSummary(result.mapping_summary);
      setMappingResults(result.mapping_results);
      setProfileMetrics(result.profile);
      setStatusMessage(`导入完成：${batchDisplayName} / ${result.batch.import_batch_id}`);
      return result;
    });
  }

  return (
    <article className="panel form-panel step-card">
      <div className="step-card-head">
        <div>
          <h2>数据导入：导入新数据</h2>
          <p className="hero-text">导入前必须确认批次名称。后续所有看板都以该批次为分析边界。</p>
        </div>
        <span className="step-badge">Import</span>
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
      <div className="primary-action-row">
        <ActionButton actionKey="import_current_file" actionStates={actionStates} primary label="导入当前文件" disabled={!canImport} onClick={importCurrentFile} title={!filePath ? '请先选择 CSV 文件' : !batchDisplayName ? '请先设置批次名称' : undefined} />
      </div>
      <details className="advanced-actions">
        <summary>高级操作：逐步执行 / 排错</summary>
        <input value={filePath} onChange={(e) => setFilePath(e.target.value)} placeholder="高级：CSV absolute path" />
        <div className="action-row">
          <ActionButton actionKey="import_probe_csv" actionStates={actionStates} label="Probe" disabled={!filePath} onClick={() => runAction('import_probe_csv', () => workbenchApi.probeCsv(filePath))} />
          <ActionButton actionKey="import_create_batch" actionStates={actionStates} label="创建批次" disabled={!filePath || !batchDisplayName.trim()} onClick={createBatch} />
          <ActionButton actionKey="import_validate_mapping" actionStates={actionStates} label="映射校验" disabled={!importBatchId || !filePath} onClick={validateMapping} />
          <ActionButton actionKey="import_get_mapping_summary" actionStates={actionStates} label="映射汇总" disabled={!importBatchId} onClick={() => refreshMappingSummary()} />
          <ActionButton actionKey="import_get_mapping_results" actionStates={actionStates} label="映射结果" disabled={!importBatchId} onClick={() => refreshMappingResults()} />
          <ActionButton actionKey="dataset_profile_refresh" actionStates={actionStates} label="刷新画像" disabled={!importBatchId} onClick={() => runAction('dataset_profile_refresh', () => profileApi.refresh(settings, importBatchId, dataType)).then(() => refreshProfile())} />
          <ActionButton actionKey="dataset_profile_get" actionStates={actionStates} label="查看画像" disabled={!importBatchId} onClick={() => refreshProfile()} />
          <ActionButton actionKey="import_start_raw_load" actionStates={actionStates} label="RAW 入库" disabled={!importBatchId || !filePath} onClick={refreshRawLoad} />
          <ActionButton actionKey="import_get_batch_status" actionStates={actionStates} label="刷新导入状态" disabled={!importBatchId} onClick={() => refreshRawStatus()} />
          <ActionButton actionKey="config_check_import_catalog" actionStates={actionStates} label="Catalog 健康" onClick={refreshCatalogHealth} />
          <ActionButton actionKey="config_get_import_mappings" actionStates={actionStates} label="字段映射" onClick={refreshMappingCatalog} />
        </div>
      </details>
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
    </article>
  );
}

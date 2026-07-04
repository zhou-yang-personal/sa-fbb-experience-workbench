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
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
};

export function ImportPanel(props: Props) {
  const { settings, effectiveSettings, dataType, setDataType, importMode, setImportMode, filePath, setFilePath, importBatchId, batch, createBatch, runAction, loadMetrics } = props;
  return (
    <article className="panel form-panel">
      <h2>CSV 导入中心</h2>
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
        <button onClick={() => runAction('import_validate_mapping', () => workbenchApi.validateMapping(settings, importBatchId, dataType, filePath))}>映射校验</button>
        <button onClick={() => loadMetrics('import_get_mapping_summary', () => mappingApi.summary(settings, importBatchId, dataType))}>映射汇总</button>
        <button onClick={() => loadMetrics('import_get_mapping_results', () => mappingApi.results(settings, importBatchId, dataType))}>映射结果</button>
        <button onClick={() => runAction('dataset_profile_refresh', () => profileApi.refresh(settings, importBatchId, dataType))}>刷新画像</button>
        <button onClick={() => loadMetrics('dataset_profile_get', () => profileApi.get(settings, importBatchId, dataType))}>查看画像</button>
        <button onClick={() => runAction('import_start_raw_load', () => workbenchApi.loadRaw(effectiveSettings, importBatchId, dataType, filePath, importMode))}>RAW 入库</button>
        <button onClick={() => loadMetrics('import_get_batch_status', () => workbenchApi.importStatus(settings, importBatchId))}>刷新导入状态</button>
        <button onClick={() => loadMetrics('config_get_import_mappings', () => workbenchApi.importMappings(settings, dataType))}>字段映射</button>
      </div>
      <small>{batch ? `current batch: ${batch.import_batch_id}` : 'no batch created'}</small>
    </article>
  );
}

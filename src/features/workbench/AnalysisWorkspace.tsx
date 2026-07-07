import { useEffect, useState } from 'react';
import type { BatchListItem, BatchTableRegistryRow, ModuleStatusRow } from '../../shared/types';
import { AnalyticsDashboard } from './AnalyticsDashboard';
import { AnalyticsStructuredDeepDivePanel } from './AnalyticsStructuredDeepDivePanel';
import { AnalyticsStructuredKpiPanel } from './AnalyticsStructuredKpiPanel';
import { AnalyticsStructuredPagedPanel } from './AnalyticsStructuredPagedPanel';
import { BatchSelector } from './BatchSelector';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';

export function AnalysisWorkspace({ c }: { c: WorkbenchController }) {
  const [batches, setBatches] = useState<BatchListItem[]>([]);
  const [tableRegistry, setTableRegistry] = useState<BatchTableRegistryRow[]>([]);
  const [moduleStatus, setModuleStatus] = useState<ModuleStatusRow[]>([]);
  const [statusMessage, setStatusMessage] = useState('请选择批次，系统会自动刷新分析上下文。');

  async function refreshBatchList() {
    const result = await workbenchApi.listBatches(c.settings);
    setBatches(result);
    setStatusMessage(result.length ? `已加载 ${result.length} 个批次。` : '当前没有可用批次。');
  }

  async function refreshBatchContext(batchId = c.importBatchId, analysisRunId = c.analysisRunId) {
    if (!batchId.trim()) {
      setTableRegistry([]);
      setModuleStatus([]);
      setStatusMessage('请先选择批次。');
      return;
    }
    await workbenchApi.prepareBatchTables(c.settings, batchId);
    const registry = await workbenchApi.batchTableRegistry(c.settings, batchId);
    const status = await workbenchApi.moduleStatus(c.settings, batchId, analysisRunId.trim() || undefined);
    setTableRegistry(registry);
    setModuleStatus(status);
    setStatusMessage(status.some((item) => item.enabled) ? '分析上下文已刷新。' : '当前批次多数组件未就绪，请查看诊断详情。');
  }

  useEffect(() => {
    void refreshBatchList().catch((error) => {
      setStatusMessage(error instanceof Error ? error.message : String(error));
    });
  }, [c.settings.host, c.settings.port, c.settings.database, c.settings.user]);

  useEffect(() => {
    if (!c.importBatchId.trim()) return;
    void refreshBatchContext().catch((error) => {
      setStatusMessage(error instanceof Error ? error.message : String(error));
    });
  }, [c.importBatchId, c.analysisRunId]);

  const resultsNotGenerated = moduleStatus.some((item) => (item.status_text ?? '').includes('尚未生成分析结果'));

  return (
    <section className="workbench-section-stack analysis-workspace analytics-workspace-v2">
      <BatchSelector
        batches={batches}
        selectedBatchId={c.importBatchId}
        onRefresh={refreshBatchList}
        statusText={statusMessage}
        onSelectBatch={(batch) => {
          if (!batch) {
            c.setImportBatchId('');
            c.setBatch(null);
            c.setOverview(null);
            c.setDashboardCharts([]);
            setTableRegistry([]);
            setModuleStatus([]);
            setStatusMessage('请先选择批次。');
            return;
          }
          c.setImportBatchId(batch.import_batch_id);
          c.setDataType(batch.data_type as WorkbenchController['dataType']);
          c.setOverview(null);
          c.setDashboardCharts([]);
          c.setBatch({
            import_batch_id: batch.import_batch_id,
            batch_display_name: batch.batch_display_name,
            data_type: batch.data_type,
            source_file_name: batch.source_file_name,
            status: batch.status,
          });
          c.setBatchDisplayName(batch.batch_display_name ?? batch.source_file_name);
        }}
      />

      <article className="panel form-panel analytics-context-card">
        <div className="step-card-head">
          <div>
            <h2>分析上下文</h2>
            <p className="hero-text">所有图表和表格都以当前 import_batch_id / analysis_run_id 为边界，并只消费 DWS / ADS / 分页聚合结果。</p>
          </div>
          <button type="button" onClick={() => refreshBatchContext()}>刷新上下文</button>
        </div>
        <div className="form-grid batch-context-form">
          <label>
            analysis_run_id
            <input value={c.analysisRunId} onChange={(event) => c.setAnalysisRunId(event.target.value)} placeholder="例如 RUN_20260705_VIDEO_PEAK" />
          </label>
        </div>
        <div className="summary-pills">
          <span className={c.importBatchId ? 'status-pill status-success' : 'status-pill status-failure'}>batch {c.importBatchId ? 'selected' : 'missing'}</span>
          <span className={c.analysisRunId ? 'status-pill status-success' : 'status-pill status-failure'}>analysis run {c.analysisRunId ? 'ready' : 'missing'}</span>
          <span className="status-pill">data {c.dataType.toUpperCase()}</span>
          <span className="status-pill">tables {tableRegistry.length}</span>
          <span className="status-pill">modules {moduleStatus.filter((item) => item.enabled).length}/{moduleStatus.length}</span>
        </div>
        {resultsNotGenerated && <p className="muted-row status-failure-text">当前批次尚未完成分析结果生成，请回到数据导入，完成 CLEAN/DWS/ADS 后再查看。</p>}
      </article>

      <AnalyticsStructuredKpiPanel c={c} />
      <AnalyticsStructuredDeepDivePanel c={c} />
      <AnalyticsStructuredPagedPanel c={c} />
      <AnalyticsDashboard c={c} />

      <details className="advanced-actions analytics-diagnostics">
        <summary>诊断：模块可用性 / Batch 表注册</summary>
        <div className="table-like module-readiness-table">
          <div className="table-row module-readiness-row table-head"><span>模块</span><span>Rows</span><span>状态</span></div>
          {moduleStatus.map((item) => (
            <div key={item.module_id} className="table-row module-readiness-row">
              <span>{item.module_name}</span>
              <span>{item.row_count}</span>
              <span className={item.enabled ? 'status-success' : 'status-failure-text'}>{item.status_text ?? (item.enabled ? 'enabled' : 'disabled')}</span>
            </div>
          ))}
          {!moduleStatus.length && <div className="table-row muted-row">暂无模块状态。</div>}
        </div>
        <div className="table-like module-readiness-table">
          <div className="table-row module-readiness-row table-head"><span>Table</span><span>Rows</span><span>Status</span></div>
          {tableRegistry.map((item) => (
            <div key={`${item.layer}-${item.logical_table_name}`} className="table-row module-readiness-row">
              <span>{item.physical_table_name}</span>
              <span>{item.row_count}</span>
              <span>{item.status}</span>
            </div>
          ))}
          {!tableRegistry.length && <div className="table-row muted-row">暂无 batch table registry。</div>}
        </div>
      </details>
    </section>
  );
}

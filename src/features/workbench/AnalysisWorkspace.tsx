import { useEffect, useState } from 'react';
import type { BatchListItem, BatchTableRegistryRow, ModuleStatusRow } from '../../shared/types';
import { AnalyticsAdsActions } from './AnalyticsAdsActions';
import { AnalyticsDashboard } from './AnalyticsDashboard';
import { AnalyticsStructuredDeepDivePanel } from './AnalyticsStructuredDeepDivePanel';
import { AnalyticsStructuredKpiPanel } from './AnalyticsStructuredKpiPanel';
import { AnalyticsStructuredPagedPanel } from './AnalyticsStructuredPagedPanel';
import { BatchSelector } from './BatchSelector';
import { ExperienceInvestigationHub, type InvestigationHubView } from './ExperienceInvestigationHub';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';

type LegacyAnalyticsView = 'apps' | 'quality' | 'cable' | 'users' | 'leads';
type AnalyticsView = LegacyAnalyticsView | InvestigationHubView;

export function AnalysisWorkspace({ c, activeView, onOpenImport, onNavigate }: { c: WorkbenchController; activeView: AnalyticsView; onOpenImport: () => void; onNavigate: (view: InvestigationHubView) => void }) {
  const [batches, setBatches] = useState<BatchListItem[]>([]);
  const [tableRegistry, setTableRegistry] = useState<BatchTableRegistryRow[]>([]);
  const [moduleStatus, setModuleStatus] = useState<ModuleStatusRow[]>([]);
  const [statusMessage, setStatusMessage] = useState('请选择批次；选择后不会自动执行分析。');
  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const [contextCheckRunning, setContextCheckRunning] = useState(false);
  const selectedBatch = batches.find((item) => item.import_batch_id === c.importBatchId);

  useEffect(() => {
    if (selectedBatch && (selectedBatch.analysis_run_id ?? '') !== c.analysisRunId) {
      c.setAnalysisRunId(selectedBatch.analysis_run_id ?? '');
    }
  }, [selectedBatch?.import_batch_id, selectedBatch?.analysis_run_id]);

  async function refreshBatchList() {
    const result = await workbenchApi.listBatches(c.settings);
    setBatches(result);
    setStatusMessage(result.length ? `已加载 ${result.length} 个批次。` : '当前没有可用批次。');
    return result;
  }

  async function deleteBatches(batchIds: string[]) {
    let deleted = 0;
    const failures: string[] = [];
    await c.runAction('import_delete_batches', async () => {
      for (const batchId of batchIds) {
        try {
          await workbenchApi.deleteBatch(c.settings, batchId);
          deleted += 1;
        } catch (error) {
          failures.push(`${batchId}: ${error instanceof Error ? error.message : String(error)}`);
        }
      }
      if (failures.length) throw new Error(`已删除 ${deleted} 个，失败 ${failures.length} 个：${failures.join('；')}`);
      return { deleted, batch_ids: batchIds };
    });
    const remaining = await refreshBatchList();
    if (c.importBatchId && !remaining.some((batch) => batch.import_batch_id === c.importBatchId)) {
      c.setImportBatchId('');
      c.setBatchDisplayName('');
      c.setBatch(null);
      c.setOverview(null);
      c.setDashboardCharts([]);
      setTableRegistry([]);
      setModuleStatus([]);
    }
    setStatusMessage(failures.length ? `已删除 ${deleted} 个批次，${failures.length} 个失败；详情见执行日志。` : `已删除 ${deleted} 个批次。`);
  }

  async function refreshBatchContext(batchId = c.importBatchId, analysisRunId = c.analysisRunId) {
    if (!batchId.trim()) {
      setTableRegistry([]);
      setModuleStatus([]);
      setStatusMessage('请先选择批次。');
      return;
    }
    setContextCheckRunning(true);
    setStatusMessage('正在执行结构检查；该操作可能统计批次表，请耐心等待。');
    try {
      const result = await c.runAction('analysis_context_check', async () => {
        await workbenchApi.prepareBatchTables(c.settings, batchId);
        const registry = await workbenchApi.batchTableRegistry(c.settings, batchId);
        const status = await workbenchApi.moduleStatus(c.settings, batchId, analysisRunId.trim() || undefined);
        return { registry, status };
      }) as { registry: BatchTableRegistryRow[]; status: ModuleStatusRow[] } | null;
      if (!result) {
        setStatusMessage('结构检查失败，请查看执行日志后重试。');
        return;
      }
      setTableRegistry(result.registry);
      setModuleStatus(result.status);
      setStatusMessage(result.status.some((item) => item.enabled) ? '结构检查完成。' : '当前批次多数组件未就绪，请查看诊断详情。');
    } finally {
      setContextCheckRunning(false);
    }
  }

  useEffect(() => {
    void refreshBatchList().catch((error) => {
      setStatusMessage(error instanceof Error ? error.message : String(error));
    });
  }, [c.settings.host, c.settings.port, c.settings.database, c.settings.user, c.settings.secret]);

  const resultsNotGenerated = moduleStatus.some((item) => (item.status_text ?? '').includes('尚未生成分析结果'));

  return (
    <section className="workbench-section-stack analysis-workspace analytics-workspace-v2">
      <BatchSelector
        batches={batches}
        selectedBatchId={c.importBatchId}
        onRefresh={refreshBatchList}
        onDeleteBatches={deleteBatches}
        statusText={statusMessage}
        onSelectBatch={(batch) => {
          if (!batch) {
            c.setImportBatchId('');
            c.setAnalysisRunId('');
            c.setBatch(null);
            c.setOverview(null);
            c.setDashboardCharts([]);
            setTableRegistry([]);
            setModuleStatus([]);
            setStatusMessage('请选择批次；选择后不会自动执行分析。');
            c.clearAnalysisContext();
            return;
          }
          if (batch.import_batch_id !== c.importBatchId) c.clearAnalysisContext();
          c.setImportBatchId(batch.import_batch_id);
          c.setAnalysisRunId(batch.analysis_run_id ?? '');
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
          setTableRegistry([]);
          setModuleStatus([]);
          const pipelineStatus = String(batch.pipeline_status ?? '').toLowerCase();
          setStatusMessage(pipelineStatus === 'failed'
            ? `RAW 已导入，但自动分析流水线失败：${batch.pipeline_message ?? '请回到数据导入查看失败步骤和日志。'}`
            : !batch.analysis_run_id
              ? '批次只有 RAW 导入记录，尚未发现 analysis_run_id；请回到数据导入完成 CLEAN/DWS/ADS。'
              : `批次与 analysis_run_id=${batch.analysis_run_id} 已同步，请加载当前看板。`);
        }}
      />

      <article className="panel form-panel analytics-context-card">
        <div className="step-card-head">
          <div>
            <h2>分析上下文</h2>
            <p className="hero-text">这里只保存当前批次和分析运行。打开应用、选择批次或切换看板都不会自动执行大查询。</p>
          </div>
          <button type="button" disabled={!c.importBatchId.trim() || contextCheckRunning} onClick={() => refreshBatchContext()}>{contextCheckRunning ? '检查中…' : '检查表与模块（较重）'}</button>
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
          <span className={`status-pill ${String(selectedBatch?.pipeline_status ?? '').toLowerCase() === 'failed' ? 'status-failure' : selectedBatch?.pipeline_status ? 'status-success' : 'status-warning'}`}>pipeline {selectedBatch?.pipeline_status ?? 'unknown'}</span>
          <span className="status-pill">tables {tableRegistry.length}</span>
          <span className="status-pill">modules {moduleStatus.filter((item) => item.enabled).length}/{moduleStatus.length}</span>
        </div>
        {resultsNotGenerated && <p className="muted-row status-failure-text">当前批次尚未完成分析结果生成，请回到数据导入，完成 CLEAN/DWS/ADS 后再查看。</p>}
      </article>

      {(['overview', 'findings', 'investigation', 'investigations'] as AnalyticsView[]).includes(activeView)
        ? <ExperienceInvestigationHub c={c} view={activeView as InvestigationHubView} onNavigate={onNavigate} />
        : <AnalyticsDashboard c={c} activeView={activeView as LegacyAnalyticsView} batchContext={selectedBatch} onOpenImport={onOpenImport} />}

      <details className="advanced-actions analytics-diagnostics" onToggle={(event) => setDiagnosticsOpen(event.currentTarget.open)}>
        <summary>高级分析与诊断（展开后才加载）</summary>
        {diagnosticsOpen && <>
        <AnalyticsStructuredKpiPanel c={c} />
        <AnalyticsStructuredDeepDivePanel c={c} />
        <AnalyticsAdsActions c={c} />
        <AnalyticsStructuredPagedPanel c={c} />
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
        </>}
      </details>
    </section>
  );
}

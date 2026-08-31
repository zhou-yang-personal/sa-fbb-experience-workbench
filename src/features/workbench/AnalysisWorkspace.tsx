import { useState } from 'react';
import type { AnalysisRunOption, BatchListItem } from '../../shared/types';
import { BatchSelector } from './BatchSelector';
import { DecisionWorkspaceV3, type DecisionView } from './DecisionWorkspaceV3';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';

export function AnalysisWorkspace({ c, activeView, onOpenImport }: { c: WorkbenchController; activeView: DecisionView; onOpenImport: () => void }) {
  const [batches, setBatches] = useState<BatchListItem[]>([]);
  const [analysisRuns, setAnalysisRuns] = useState<AnalysisRunOption[]>([]);
  const [status, setStatus] = useState('安全启动：尚未访问 MySQL。点击“选择数据批次”开始。');
  const [showSelector, setShowSelector] = useState(!c.importBatchId);
  const selectedBatch = batches.find((item) => item.import_batch_id === c.importBatchId);

  async function refreshBatches() {
    setStatus('正在读取批次元数据…');
    try {
      const result = await workbenchApi.listBatches(c.effectiveSettings);
      setBatches(result);
      setStatus(result.length ? `找到 ${result.length} 个批次，请选择一个。` : '没有批次；请到数据中心导入 CSV。');
      return result;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
      return [];
    }
  }

  async function loadRuns(batch: BatchListItem) {
    try {
      const result = await workbenchApi.listAnalysisRuns(c.effectiveSettings, batch.import_batch_id);
      setAnalysisRuns(result);
      const ready = result.find((run) => ['success', 'degraded'].includes(run.status.toLowerCase()) && run.v2_period_ready)
        ?? result.find((run) => run.v2_period_ready)
        ?? result[0];
      c.setAnalysisRunId(ready?.analysis_run_id ?? batch.analysis_run_id ?? '');
      setStatus(ready ? `已选择可分析运行 ${ready.analysis_run_id}。页面尚未查询看板。` : '该批次没有可分析运行，请到数据中心生成聚合。');
    } catch (error) {
      setAnalysisRuns([]);
      c.setAnalysisRunId(batch.analysis_run_id ?? '');
      setStatus(`批次已选择；运行元数据读取失败：${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async function selectBatch(batch: BatchListItem | null) {
    if (!batch) {
      c.setImportBatchId(''); c.setAnalysisRunId(''); c.setBatch(null); c.clearAnalysisContext(); setAnalysisRuns([]); setShowSelector(true); return;
    }
    if (batch.import_batch_id !== c.importBatchId) c.clearAnalysisContext();
    c.setImportBatchId(batch.import_batch_id);
    c.setBatchDisplayName(batch.batch_display_name ?? batch.source_file_name);
    c.setDataType(batch.data_type as WorkbenchController['dataType']);
    c.setBatch({ import_batch_id: batch.import_batch_id, batch_display_name: batch.batch_display_name, data_type: batch.data_type, source_file_name: batch.source_file_name, status: batch.status });
    setShowSelector(false);
    await loadRuns(batch);
  }

  async function deleteBatches(batchIds: string[]) {
    for (const batchId of batchIds) await c.runAction(`delete_${batchId}`, () => workbenchApi.deleteBatch(c.effectiveSettings, batchId));
    if (batchIds.includes(c.importBatchId)) await selectBatch(null);
    await refreshBatches();
  }

  return <section className="workbench-section-stack analysis-workspace decision-analysis-shell">
    <article className="analysis-context-compact">
      <div><strong title={c.batchDisplayName || c.importBatchId}>{c.batchDisplayName || '未选择数据批次'}</strong>{c.importBatchId && <small>{c.dataType.toUpperCase()}</small>}</div>
      <div className="hub-actions"><button type="button" onClick={() => setShowSelector((value) => !value)}>{showSelector ? '收起批次' : '切换数据批次'}</button>{!c.importBatchId && <button type="button" className="primary" onClick={onOpenImport}>去数据中心</button>}</div>
    </article>
    {showSelector && <BatchSelector batches={batches} selectedBatchId={c.importBatchId} onRefresh={refreshBatches} onDeleteBatches={deleteBatches} statusText={status} onSelectBatch={(batch) => { void selectBatch(batch); }} />}
    {analysisRuns.length > 1 && <details className="run-switcher"><summary>切换分析运行（通常无需操作）</summary><select value={c.analysisRunId} onChange={(event) => c.setAnalysisRunId(event.target.value)}>{analysisRuns.map((run) => <option key={run.analysis_run_id} value={run.analysis_run_id}>{run.analysis_run_id} · {run.status} · {run.v2_period_ready ? 'READY' : 'NOT READY'}</option>)}</select></details>}
    {selectedBatch?.pipeline_status === 'failed' && <p className="decision-status status-failure-text">该批次最近流水线失败：{selectedBatch.pipeline_message || '请到数据中心查看任务日志并从断点继续。'}</p>}
    <DecisionWorkspaceV3 c={c} view={activeView} />
  </section>;
}

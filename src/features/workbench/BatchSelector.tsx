import { useEffect, useMemo, useState } from 'react';
import type { BatchListItem } from '../../shared/types';

type Props = {
  batches: BatchListItem[];
  selectedBatchId: string;
  onSelectBatch: (batch: BatchListItem | null) => void;
  onRefresh: () => Promise<unknown> | void;
  onDeleteBatches: (batchIds: string[]) => Promise<void> | void;
  statusText?: string;
};

function batchLabel(batch: BatchListItem) {
  const displayName = batch.batch_display_name?.trim() || batch.source_file_name;
  return `${displayName} · ${batch.data_type.toUpperCase()} · ${batch.import_batch_id}`;
}

function isTestBatch(batch: BatchListItem) {
  const searchable = [batch.batch_display_name, batch.source_file_name, batch.import_batch_id]
    .filter(Boolean)
    .join(' ')
    .toLowerCase();
  return batch.status.toLowerCase() === 'failed' || /(^|[^a-z])(test|demo|sample)([^a-z]|$)/.test(searchable) || searchable.includes('测试');
}

export function BatchSelector({ batches, selectedBatchId, onSelectBatch, onRefresh, onDeleteBatches, statusText }: Props) {
  const [selectedForDeletion, setSelectedForDeletion] = useState<string[]>([]);
  const [deleting, setDeleting] = useState(false);
  const deletableBatches = useMemo(() => batches.filter((batch) => batch.status.toLowerCase() !== 'running'), [batches]);

  useEffect(() => {
    const available = new Set(deletableBatches.map((batch) => batch.import_batch_id));
    setSelectedForDeletion((items) => items.filter((id) => available.has(id)));
  }, [deletableBatches]);

  function toggleBatch(batchId: string) {
    setSelectedForDeletion((items) => items.includes(batchId) ? items.filter((id) => id !== batchId) : [...items, batchId]);
  }

  async function deleteSelected() {
    if (!selectedForDeletion.length || deleting) return;
    const targets = batches.filter((batch) => selectedForDeletion.includes(batch.import_batch_id));
    const preview = targets.slice(0, 5).map((batch) => batch.batch_display_name?.trim() || batch.source_file_name).join('\n- ');
    const more = targets.length > 5 ? `\n……另有 ${targets.length - 5} 个批次` : '';
    if (!window.confirm(`永久删除以下 ${targets.length} 个导入批次及其 RAW / DWD / DWS / ADS 数据？此操作无法撤销。\n\n- ${preview}${more}`)) return;
    setDeleting(true);
    try {
      await onDeleteBatches(selectedForDeletion);
    } finally {
      setDeleting(false);
    }
  }

  return (
    <section className="panel form-panel batch-selector-panel">
      <div className="step-card-head">
        <div>
          <h2>批次选择</h2>
          <p className="hero-text">选择已有批次作为当前工作上下文；历史测试和失败批次可在下方批量清理。</p>
        </div>
        <button type="button" onClick={onRefresh}>刷新批次列表</button>
      </div>
      <div className="form-grid">
        <label>
          当前批次
          <select
            value={selectedBatchId}
            onChange={(event) => {
              const batch = batches.find((item) => item.import_batch_id === event.target.value) ?? null;
              onSelectBatch(batch);
            }}
          >
            <option value="">请选择批次</option>
            {batches.map((batch) => <option key={batch.import_batch_id} value={batch.import_batch_id}>{batchLabel(batch)}</option>)}
          </select>
        </label>
      </div>
      <div className="summary-pills">
        <span className="status-pill">{batches.length} batches</span>
        {statusText && <span className="status-pill">{statusText}</span>}
      </div>
      <details className="batch-delete-manager">
        <summary>历史批次管理与删除</summary>
        <p className="muted-row">可批量清理测试或失败批次。正在运行的批次受保护，无法勾选；删除会同时清理该批次的物理表、分析结果和任务日志。</p>
        <div className="batch-delete-toolbar">
          <button type="button" onClick={() => setSelectedForDeletion(deletableBatches.filter(isTestBatch).map((batch) => batch.import_batch_id))}>选择测试/失败批次</button>
          <button type="button" onClick={() => setSelectedForDeletion(deletableBatches.map((batch) => batch.import_batch_id))}>选择全部可删除</button>
          <button type="button" onClick={() => setSelectedForDeletion([])}>清空选择</button>
          <button type="button" className="danger-button" disabled={!selectedForDeletion.length || deleting} onClick={deleteSelected}>
            {deleting ? '正在删除…' : `删除选中批次（${selectedForDeletion.length}）`}
          </button>
        </div>
        <div className="batch-delete-list">
          {batches.map((batch) => {
            const running = batch.status.toLowerCase() === 'running';
            const checked = selectedForDeletion.includes(batch.import_batch_id);
            return (
              <label className={`batch-delete-row ${running ? 'is-protected' : ''}`} key={batch.import_batch_id}>
                <input type="checkbox" checked={checked} disabled={running || deleting} onChange={() => toggleBatch(batch.import_batch_id)} />
                <span>
                  <strong>{batch.batch_display_name?.trim() || batch.source_file_name}</strong>
                  <small>{batch.import_batch_id}</small>
                </span>
                <span>{batch.data_type.toUpperCase()}</span>
                <span className={batch.status.toLowerCase() === 'failed' ? 'status-failure-text' : ''}>{running ? 'RUNNING · 已保护' : batch.status.toUpperCase()}</span>
                <span>{batch.imported_rows ?? 0} rows</span>
              </label>
            );
          })}
          {!batches.length && <p className="muted-row">暂无历史批次。</p>}
        </div>
      </details>
    </section>
  );
}

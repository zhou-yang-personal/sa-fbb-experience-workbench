import { useEffect, useMemo, useState } from 'react';
import type { BatchListItem } from '../../shared/types';

type Props = {
  batches: BatchListItem[];
  selectedBatchId: string;
  onSelectBatch: (batch: BatchListItem | null) => void;
  onRefresh: () => Promise<unknown> | void;
  onDeleteBatches: (batchIds: string[]) => Promise<void> | void;
  statusText?: string;
  variant?: 'compact' | 'library';
};

function batchLabel(batch: BatchListItem) {
  const displayName = batch.batch_display_name?.trim() || batch.source_file_name;
  const pipeline = batch.pipeline_status?.toUpperCase() ?? 'NO PIPELINE';
  return `${displayName} · ${batch.data_type.toUpperCase()} · PIPELINE ${pipeline} · ${batch.import_batch_id}`;
}

function isTestBatch(batch: BatchListItem) {
  const searchable = [batch.batch_display_name, batch.source_file_name, batch.import_batch_id]
    .filter(Boolean)
    .join(' ')
    .toLowerCase();
  return batch.status.toLowerCase() === 'failed' || /(^|[^a-z])(test|demo|sample)([^a-z]|$)/.test(searchable) || searchable.includes('测试');
}

function isBatchRunning(batch: BatchListItem) {
  return batch.status.toLowerCase() === 'running' || ['pending', 'running'].includes(String(batch.pipeline_status ?? '').toLowerCase());
}

function pipelineLabel(batch: BatchListItem) {
  const status = String(batch.pipeline_status ?? '').toLowerCase();
  if (status === 'success' || status === 'degraded') return '可分析';
  if (status === 'running' || status === 'pending') return '运行中';
  if (status === 'failed') return '失败';
  if (batch.status.toLowerCase() === 'success') return '仅 RAW 就绪';
  return '未就绪';
}

export function BatchSelector({ batches, selectedBatchId, onSelectBatch, onRefresh, onDeleteBatches, statusText, variant = 'compact' }: Props) {
  const [selectedForDeletion, setSelectedForDeletion] = useState<string[]>([]);
  const [deleting, setDeleting] = useState(false);
  const [keyword, setKeyword] = useState('');
  const deletableBatches = useMemo(() => batches.filter((batch) => !isBatchRunning(batch)), [batches]);
  const visibleBatches = useMemo(() => {
    const normalized = keyword.trim().toLowerCase();
    if (!normalized) return batches;
    return batches.filter((batch) => [batch.batch_display_name, batch.source_file_name, batch.import_batch_id, batch.data_type, batch.pipeline_status, batch.status]
      .filter(Boolean)
      .join(' ')
      .toLowerCase()
      .includes(normalized));
  }, [batches, keyword]);

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
          <h2>{variant === 'library' ? '批次库' : '批次选择'}</h2>
          <p className="hero-text">{variant === 'library' ? '查看每个批次的数据状态，选择后再决定查看任务、继续处理或进入分析。选择本身不会发起数据库查询。' : '选择已有批次作为当前工作上下文；历史测试和失败批次可在下方批量清理。'}</p>
        </div>
        <button type="button" onClick={onRefresh}>刷新批次列表</button>
      </div>
      {variant === 'compact' ? (
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
      ) : (
        <>
          <div className="batch-library-toolbar">
            <input value={keyword} onChange={(event) => setKeyword(event.target.value)} placeholder="搜索批次名称、文件名、批次 ID 或状态" />
            <span>{visibleBatches.length} / {batches.length} 个批次</span>
          </div>
          <div className="batch-library-table">
            <div className="batch-library-row batch-library-head"><span>批次</span><span>类型</span><span>数据状态</span><span>导入行数</span><span>操作</span></div>
            {visibleBatches.map((batch) => {
              const selected = selectedBatchId === batch.import_batch_id;
              return (
                <div className={`batch-library-row ${selected ? 'is-selected' : ''}`} key={batch.import_batch_id}>
                  <span><strong>{batch.batch_display_name?.trim() || batch.source_file_name}</strong><small>{batch.import_batch_id}</small></span>
                  <span>{batch.data_type.toUpperCase()}</span>
                  <span><strong>{pipelineLabel(batch)}</strong><small>RAW {batch.status.toUpperCase()} · PIPELINE {batch.pipeline_status?.toUpperCase() ?? 'NONE'}</small></span>
                  <span>{(batch.imported_rows ?? 0).toLocaleString()}</span>
                  <span><button type="button" className={selected ? 'action-button-done' : ''} onClick={() => onSelectBatch(batch)}>{selected ? '已选择' : '选择批次'}</button></span>
                </div>
              );
            })}
            {!visibleBatches.length && <p className="muted-row">{batches.length ? '没有匹配的批次。' : '尚未加载批次，请点击“刷新批次列表”。'}</p>}
          </div>
        </>
      )}
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
            const running = isBatchRunning(batch);
            const checked = selectedForDeletion.includes(batch.import_batch_id);
            return (
              <label className={`batch-delete-row ${running ? 'is-protected' : ''}`} key={batch.import_batch_id}>
                <input type="checkbox" checked={checked} disabled={running || deleting} onChange={() => toggleBatch(batch.import_batch_id)} />
                <span>
                  <strong>{batch.batch_display_name?.trim() || batch.source_file_name}</strong>
                  <small>{batch.import_batch_id}</small>
                </span>
                <span>{batch.data_type.toUpperCase()}</span>
                <span className={(batch.pipeline_status ?? batch.status).toLowerCase() === 'failed' ? 'status-failure-text' : ''}>{running ? '任务运行中 · 已保护' : `RAW ${batch.status.toUpperCase()} · PIPELINE ${batch.pipeline_status?.toUpperCase() ?? 'NONE'}`}</span>
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

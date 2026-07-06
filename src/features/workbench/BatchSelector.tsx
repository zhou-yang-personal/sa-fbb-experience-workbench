import type { BatchListItem } from '../../shared/types';

type Props = {
  batches: BatchListItem[];
  selectedBatchId: string;
  onSelectBatch: (batch: BatchListItem | null) => void;
  onRefresh: () => Promise<void> | void;
  statusText?: string;
};

function batchLabel(batch: BatchListItem) {
  const displayName = batch.batch_display_name?.trim() || batch.source_file_name;
  return `${displayName} · ${batch.data_type.toUpperCase()} · ${batch.import_batch_id}`;
}

export function BatchSelector({ batches, selectedBatchId, onSelectBatch, onRefresh, statusText }: Props) {
  return (
    <section className="panel form-panel batch-selector-panel">
      <div className="step-card-head">
        <div>
          <h2>批次选择</h2>
          <p className="hero-text">先选批次，再刷新模块状态。分析空间边界就是这个批次，避免手输 ID 导致串批。</p>
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
    </section>
  );
}

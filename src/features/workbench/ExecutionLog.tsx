import { useMemo, useState } from 'react';
import { formatLocalDateTime, localTimeZone } from '../../shared/localDateTime';
import type { ExecutionLogEntry, ExecutionLogStatus } from '../../shared/types';

type LogFilter = 'all' | ExecutionLogStatus;

interface ExecutionLogProps {
  log: ExecutionLogEntry[];
  compact?: boolean;
}

function copyText(text: string) {
  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    void navigator.clipboard.writeText(text);
  }
}

function formatEntry(entry: ExecutionLogEntry) {
  const lines = [
    `command: ${entry.command}`,
    `status: ${entry.status}`,
    `time_zone: ${localTimeZone()}`,
    `started_at: ${formatLocalDateTime(entry.started_at)}`,
    `finished_at: ${formatLocalDateTime(entry.finished_at)}`,
    `duration_ms: ${entry.duration_ms}`,
    `message: ${entry.message}`,
  ];
  if (entry.result_preview) lines.push(`result_preview: ${entry.result_preview}`);
  return lines.join('\n');
}

export function ExecutionLog({ log, compact = false }: ExecutionLogProps) {
  const [filter, setFilter] = useState<LogFilter>('all');
  const [keyword, setKeyword] = useState('');

  const filtered = useMemo(() => {
    const term = keyword.trim().toLowerCase();
    return log.filter((entry) => {
      if (filter !== 'all' && entry.status !== filter) return false;
      if (!term) return true;
      return [entry.command, entry.status, entry.message, entry.result_preview ?? ''].some((value) => value.toLowerCase().includes(term));
    });
  }, [filter, keyword, log]);

  const failedRows = log.filter((entry) => entry.status === 'failure');
  const successRows = log.filter((entry) => entry.status === 'success');
  const failedText = failedRows.map(formatEntry).join('\n\n---\n\n');
  const allText = log.map(formatEntry).join('\n\n---\n\n');

  return (
    <section className="panel execution-log-panel">
      {!compact && <div className="log-header">
        <div>
          <h2>诊断日志</h2>
          <p className="muted-row">记录命令、错误、耗时和返回预览。字段映射、质量门禁、ETL 失败都应在这里看到可复制诊断信息。时间按本地 PC 时区 {localTimeZone()} 显示。</p>
        </div>
        <div className="log-summary">
          <span>{localTimeZone()}</span>
          <span>{log.length} total</span>
          <span>{successRows.length} success</span>
          <span>{failedRows.length} failed</span>
        </div>
      </div>}
      {compact && <div className="log-summary log-summary-compact">
        <span>{localTimeZone()}</span>
        <span>{log.length} 条</span>
        <span className="log-count-success">成功 {successRows.length}</span>
        <span className="log-count-failure">失败 {failedRows.length}</span>
      </div>}

      <div className="table-toolbar log-toolbar">
        <div className="log-toolbar-fields">
          <input value={keyword} onChange={(e) => setKeyword(e.target.value)} placeholder="Search command / field / error / result" />
          <select value={filter} onChange={(e) => setFilter(e.target.value as LogFilter)}>
            <option value="all">All status</option>
            <option value="success">Success only</option>
            <option value="failure">Failure only</option>
          </select>
        </div>
        <div className="log-toolbar-actions">
          <button type="button" onClick={() => { setFilter('all'); setKeyword(''); }}>清空筛选</button>
          <button type="button" disabled={!failedRows.length} onClick={() => copyText(failedText)}>复制失败信息</button>
          <button type="button" disabled={!log.length} onClick={() => copyText(allText)}>复制全部日志</button>
        </div>
      </div>

      <div className="log-list structured-log-list">
        {filtered.map((entry) => (
          <article key={entry.id} className={`log-entry log-entry-${entry.status}`}>
            <div className="log-entry-head">
              <span className={`status-pill status-${entry.status}`}>{entry.status}</span>
              <strong>{entry.command}</strong>
              <button type="button" onClick={() => copyText(formatEntry(entry))}>复制</button>
            </div>
            <div className="log-meta">
              <span title={`UTC: ${entry.started_at}`}>{formatLocalDateTime(entry.started_at)}</span>
              <span>{entry.duration_ms} ms</span>
            </div>
            <pre>{entry.message}</pre>
            {entry.result_preview && <pre>{entry.result_preview}</pre>}
          </article>
        ))}
        {!filtered.length && <pre>No matching operation log.</pre>}
      </div>
    </section>
  );
}

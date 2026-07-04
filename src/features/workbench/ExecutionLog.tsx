import { useMemo, useState } from 'react';
import type { ExecutionLogEntry, ExecutionLogStatus } from '../../shared/types';

type LogFilter = 'all' | ExecutionLogStatus;

interface ExecutionLogProps {
  log: ExecutionLogEntry[];
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
    `started_at: ${entry.started_at}`,
    `finished_at: ${entry.finished_at}`,
    `duration_ms: ${entry.duration_ms}`,
    `message: ${entry.message}`,
  ];
  if (entry.result_preview) lines.push(`result_preview: ${entry.result_preview}`);
  return lines.join('\n');
}

export function ExecutionLog({ log }: ExecutionLogProps) {
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

  return (
    <section className="panel execution-log-panel">
      <div className="log-header">
        <div>
          <h2>执行日志</h2>
          <p className="muted-row">按结构化执行结果过滤操作记录，失败项可一键复制。</p>
        </div>
        <div className="log-summary">
          <span>{log.length} total</span>
          <span>{successRows.length} success</span>
          <span>{failedRows.length} failed</span>
        </div>
      </div>

      <div className="table-toolbar log-toolbar">
        <input value={keyword} onChange={(e) => setKeyword(e.target.value)} placeholder="Search command / message / result" />
        <select value={filter} onChange={(e) => setFilter(e.target.value as LogFilter)}>
          <option value="all">All status</option>
          <option value="success">Success only</option>
          <option value="failure">Failure only</option>
        </select>
        <button type="button" onClick={() => { setFilter('all'); setKeyword(''); }}>清空筛选</button>
        <button type="button" disabled={!failedRows.length} onClick={() => copyText(failedText)}>复制失败信息</button>
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
              <span>{entry.started_at}</span>
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

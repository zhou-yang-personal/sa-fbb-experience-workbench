import { useMemo, useState } from 'react';

type LogFilter = 'all' | 'success' | 'failure';

interface ExecutionLogProps {
  log: string[];
}

function getLogStatus(line: string): LogFilter {
  return line.toLowerCase().includes('failed') ? 'failure' : 'success';
}

function extractCommand(line: string) {
  const withoutTime = line.replace(/^\d{1,2}:\d{2}:\d{2}\s*/, '');
  const idx = withoutTime.indexOf(':');
  return idx > 0 ? withoutTime.slice(0, idx) : withoutTime;
}

function copyText(text: string) {
  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    void navigator.clipboard.writeText(text);
  }
}

export function ExecutionLog({ log }: ExecutionLogProps) {
  const [filter, setFilter] = useState<LogFilter>('all');
  const [keyword, setKeyword] = useState('');

  const rows = useMemo(() => log.map((line, index) => ({
    id: `${index}-${line}`,
    line,
    status: getLogStatus(line),
    command: extractCommand(line),
  })), [log]);

  const filtered = useMemo(() => {
    const term = keyword.trim().toLowerCase();
    return rows.filter((row) => {
      if (filter !== 'all' && row.status !== filter) return false;
      if (term && !row.line.toLowerCase().includes(term) && !row.command.toLowerCase().includes(term)) return false;
      return true;
    });
  }, [filter, keyword, rows]);

  const failedRows = rows.filter((row) => row.status === 'failure');
  const successRows = rows.filter((row) => row.status === 'success');
  const failedText = failedRows.map((row) => row.line).join('\n');

  return (
    <section className="panel execution-log-panel">
      <div className="log-header">
        <div>
          <h2>执行日志</h2>
          <p className="muted-row">按命令结果过滤本次前端操作记录，失败项可一键复制。</p>
        </div>
        <div className="log-summary">
          <span>{rows.length} total</span>
          <span>{successRows.length} success</span>
          <span>{failedRows.length} failed</span>
        </div>
      </div>

      <div className="table-toolbar log-toolbar">
        <input value={keyword} onChange={(e) => setKeyword(e.target.value)} placeholder="Search command / message" />
        <select value={filter} onChange={(e) => setFilter(e.target.value as LogFilter)}>
          <option value="all">All status</option>
          <option value="success">Success only</option>
          <option value="failure">Failure only</option>
        </select>
        <button type="button" onClick={() => { setFilter('all'); setKeyword(''); }}>清空筛选</button>
        <button type="button" disabled={!failedRows.length} onClick={() => copyText(failedText)}>复制失败信息</button>
      </div>

      <div className="log-list structured-log-list">
        {filtered.map((row) => (
          <article key={row.id} className={`log-entry log-entry-${row.status}`}>
            <div className="log-entry-head">
              <span className={`status-pill status-${row.status}`}>{row.status}</span>
              <strong>{row.command}</strong>
              <button type="button" onClick={() => copyText(row.line)}>复制</button>
            </div>
            <pre>{row.line}</pre>
          </article>
        ))}
        {!filtered.length && <pre>No matching operation log.</pre>}
      </div>
    </section>
  );
}

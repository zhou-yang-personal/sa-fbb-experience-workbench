import { useMemo, useState } from 'react';
import type { MetricCard } from '../../shared/types';

type SortMode = 'value_desc' | 'value_asc' | 'label_asc';

type EvidenceRow = MetricCard & {
  numeric: number;
  detail: Record<string, string>;
};

function toNumber(value: string | number | undefined) {
  if (typeof value === 'number') return Number.isFinite(value) ? value : 0;
  const parsed = Number.parseFloat(String(value ?? '').replace(/,/g, '').replace(/%/g, '').trim());
  return Number.isFinite(parsed) ? parsed : 0;
}

function parseHint(hint: string) {
  return hint.split(/,\s+|\s+\|\s+/).reduce<Record<string, string>>((acc, part) => {
    const pos = part.indexOf('=');
    if (pos < 0) return acc;
    const key = part.slice(0, pos).trim();
    const value = part.slice(pos + 1).trim();
    if (key) acc[key] = value;
    return acc;
  }, {});
}

function csvEscape(value: unknown) {
  const text = String(value ?? '');
  return /[",\n\r]/.test(text) ? `"${text.replace(/"/g, '""')}"` : text;
}

function downloadCsv(fileName: string, rows: EvidenceRow[]) {
  const dynamicKeys = Array.from(new Set(rows.flatMap((row) => Object.keys(row.detail)))).slice(0, 24);
  const header = ['rank', 'label', 'value', 'hint', ...dynamicKeys];
  const body = rows.map((row, index) => [index + 1, row.label, row.value, row.hint, ...dynamicKeys.map((key) => row.detail[key] ?? '')]);
  const csv = [header, ...body].map((line) => line.map(csvEscape).join(',')).join('\n');
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = fileName;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

function normalizeFileName(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9\u4e00-\u9fa5]+/g, '_').replace(/^_+|_+$/g, '') || 'analytics_table';
}

export function AnalyticsEvidenceTable({ title, rows, limit = 80 }: { title: string; rows: MetricCard[]; limit?: number }) {
  const [keyword, setKeyword] = useState('');
  const [sortMode, setSortMode] = useState<SortMode>('value_desc');
  const [minAbsValue, setMinAbsValue] = useState('');
  const [selected, setSelected] = useState<EvidenceRow | null>(null);

  const evidenceRows = useMemo<EvidenceRow[]>(() => rows.map((row) => ({ ...row, numeric: toNumber(row.value), detail: parseHint(row.hint) })), [rows]);
  const filtered = useMemo(() => {
    const min = Number.parseFloat(minAbsValue);
    const hasMin = Number.isFinite(min);
    const q = keyword.trim().toLowerCase();
    return evidenceRows
      .filter((row) => !q || `${row.label} ${row.value} ${row.hint}`.toLowerCase().includes(q))
      .filter((row) => !hasMin || Math.abs(row.numeric) >= min)
      .sort((a, b) => {
        if (sortMode === 'value_asc') return Math.abs(a.numeric) - Math.abs(b.numeric);
        if (sortMode === 'label_asc') return a.label.localeCompare(b.label);
        return Math.abs(b.numeric) - Math.abs(a.numeric);
      })
      .slice(0, limit);
  }, [evidenceRows, keyword, minAbsValue, sortMode, limit]);

  return (
    <article className="analytics-card analytics-table-card analytics-evidence-table-card">
      <div className="analytics-section-head">
        <div>
          <h3>{title}</h3>
          <p>可搜索、排序、阈值过滤、导出和点击查看证据详情。</p>
        </div>
        <span>{filtered.length}/{rows.length} rows</span>
      </div>
      <div className="analytics-table-toolbar">
        <input value={keyword} onChange={(event) => setKeyword(event.target.value)} placeholder="搜索 label / value / hint" />
        <input value={minAbsValue} onChange={(event) => setMinAbsValue(event.target.value)} inputMode="decimal" placeholder="最小绝对值" />
        <select value={sortMode} onChange={(event) => setSortMode(event.target.value as SortMode)}>
          <option value="value_desc">按绝对值降序</option>
          <option value="value_asc">按绝对值升序</option>
          <option value="label_asc">按名称排序</option>
        </select>
        <button type="button" disabled={!filtered.length} onClick={() => downloadCsv(`${normalizeFileName(title)}.csv`, filtered)}>导出当前表</button>
      </div>
      <div className="analytics-table-wrap">
        <table className="analytics-table">
          <thead><tr><th>Rank</th><th>Name</th><th>Value</th><th>Evidence</th><th>Action</th></tr></thead>
          <tbody>
            {filtered.map((row, index) => {
              const detail = Object.keys(row.detail).length ? Object.entries(row.detail).slice(0, 6).map(([key, value]) => `${key}=${value}`).join(' · ') : row.hint;
              return (
                <tr key={`${row.label}-${index}`}>
                  <td>{index + 1}</td>
                  <td>{row.label}</td>
                  <td>{row.value}</td>
                  <td title={row.hint}>{detail || '-'}</td>
                  <td><button type="button" onClick={() => setSelected(row)}>详情</button></td>
                </tr>
              );
            })}
            {!filtered.length && <tr><td colSpan={5}>No data. Run analysis pipeline and refresh dashboard.</td></tr>}
          </tbody>
        </table>
      </div>
      {selected && (
        <div className="analytics-evidence-drawer-backdrop" onClick={() => setSelected(null)}>
          <aside className="analytics-evidence-drawer" onClick={(event) => event.stopPropagation()}>
            <div className="analytics-section-head">
              <div>
                <h3>{selected.label}</h3>
                <p>Value: {selected.value}</p>
              </div>
              <button type="button" onClick={() => setSelected(null)}>关闭</button>
            </div>
            <div className="analytics-evidence-summary">
              <strong>Raw evidence text</strong>
              <pre>{selected.hint || '-'}</pre>
            </div>
            <div className="analytics-evidence-kv">
              {Object.entries(selected.detail).map(([key, value]) => <div key={key}><span>{key}</span><strong>{value}</strong></div>)}
              {!Object.keys(selected.detail).length && <p className="muted-row">当前行没有 key=value 结构化字段。</p>}
            </div>
          </aside>
        </div>
      )}
    </article>
  );
}

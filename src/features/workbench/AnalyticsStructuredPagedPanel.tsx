import { useEffect, useState } from 'react';
import type { MetricCard } from '../../shared/types';
import { AnalyticsEvidenceTable } from './AnalyticsEvidenceTable';
import { analyticsStructuredApi, type StructuredAnalyticsQuery } from './analyticsStructuredApi';
import type { WorkbenchController } from './useWorkbenchController';

function num(value: string) {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

export function AnalyticsStructuredPagedPanel({ c }: { c: WorkbenchController }) {
  const [query, setQuery] = useState<StructuredAnalyticsQuery>({ page: 1, pageSize: 200, sortBy: 'default' });
  const [rows, setRows] = useState<Record<string, MetricCard[]>>({ app: [], hourly: [], network: [], user: [], lead: [] });
  const [status, setStatus] = useState('waiting for batch and run');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function refresh(next = query) {
    if (disabled) return;
    setStatus('loading paged structured data');
    try {
      const [app, hourly, network, user, lead] = await Promise.all([
        analyticsStructuredApi.appRank(c.effectiveSettings, c.importBatchId, c.analysisRunId, next).catch(() => []),
        analyticsStructuredApi.hourlyTrend(c.effectiveSettings, c.importBatchId, c.analysisRunId, next).catch(() => []),
        analyticsStructuredApi.networkHotspots(c.effectiveSettings, c.importBatchId, c.analysisRunId, next).catch(() => []),
        analyticsStructuredApi.userProfiles(c.effectiveSettings, c.importBatchId, c.analysisRunId, next).catch(() => []),
        analyticsStructuredApi.leadEvidence(c.effectiveSettings, c.importBatchId, c.analysisRunId, next).catch(() => []),
      ]);
      setRows({ app, hourly, network, user, lead });
      setStatus(`page=${next.page ?? 1}, pageSize=${next.pageSize ?? 200}, rows=${app.length + hourly.length + network.length + user.length + lead.length}`);
    } catch (error) {
      setRows({ app: [], hourly: [], network: [], user: [], lead: [] });
      setStatus(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => { void refresh(); }, [c.importBatchId, c.analysisRunId]);

  function apply(next: StructuredAnalyticsQuery) {
    setQuery(next);
    void refresh(next);
  }

  return (
    <article className="analytics-card analytics-structured-kpi-panel">
      <div className="analytics-section-head">
        <div>
          <h3>Paged Structured Analytics</h3>
          <p>Backend paging and filters for App, Hourly, Network, User and Lead evidence.</p>
        </div>
        <button type="button" disabled={disabled} onClick={() => refresh()}>Refresh</button>
      </div>
      <div className="analytics-table-toolbar">
        <input value={query.keyword ?? ''} onChange={(event) => setQuery({ ...query, page: 1, keyword: event.target.value })} placeholder="keyword" />
        <input value={query.minValue ?? ''} onChange={(event) => setQuery({ ...query, page: 1, minValue: num(event.target.value) })} placeholder="min value" />
        <select value={query.sortBy ?? 'default'} onChange={(event) => setQuery({ ...query, page: 1, sortBy: event.target.value })}>
          <option value="default">default</option>
          <option value="users">users</option>
          <option value="traffic_gb">traffic</option>
          <option value="duration_hours">duration</option>
          <option value="subscriber_rtt_ms">rtt</option>
          <option value="demand_score">demand</option>
          <option value="migration_motive_score">motive</option>
          <option value="label">label</option>
        </select>
        <select value={query.pageSize ?? 200} onChange={(event) => setQuery({ ...query, page: 1, pageSize: Number.parseInt(event.target.value, 10) })}>
          <option value={80}>80</option>
          <option value={200}>200</option>
          <option value={500}>500</option>
        </select>
        <button type="button" onClick={() => apply({ ...query, page: Math.max(1, (query.page ?? 1) - 1) })}>Prev</button>
        <span className="status-pill">Page {query.page ?? 1}</span>
        <button type="button" onClick={() => apply({ ...query, page: (query.page ?? 1) + 1 })}>Next</button>
        <button type="button" onClick={() => apply({ ...query, page: 1 })}>Apply</button>
      </div>
      <div className="analytics-context-line"><span>{status}</span></div>
      <div className="analytics-structured-evidence-grid analytics-structured-deep-grid">
        <AnalyticsEvidenceTable title="Paged App Rank" rows={rows.app} limit={query.pageSize ?? 200} />
        <AnalyticsEvidenceTable title="Paged Hourly Trend" rows={rows.hourly} limit={query.pageSize ?? 200} />
        <AnalyticsEvidenceTable title="Paged Network Hotspots" rows={rows.network} limit={query.pageSize ?? 200} />
        <AnalyticsEvidenceTable title="Paged User Profiles" rows={rows.user} limit={query.pageSize ?? 200} />
        <AnalyticsEvidenceTable title="Paged Lead Evidence" rows={rows.lead} limit={query.pageSize ?? 200} />
      </div>
    </article>
  );
}

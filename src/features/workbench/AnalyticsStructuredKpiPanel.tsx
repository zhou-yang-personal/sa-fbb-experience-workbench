import { useEffect, useState } from 'react';
import type { MetricCard } from '../../shared/types';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import type { WorkbenchController } from './useWorkbenchController';

export function AnalyticsStructuredKpiPanel({ c }: { c: WorkbenchController }) {
  const [items, setItems] = useState<MetricCard[]>([]);
  const [appRows, setAppRows] = useState<MetricCard[]>([]);
  const [hourlyRows, setHourlyRows] = useState<MetricCard[]>([]);
  const [status, setStatus] = useState('等待当前批次和 analysis_run_id。');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function refresh() {
    if (disabled) return;
    setStatus('正在读取结构化 KPI / App Rank / Hourly Trend API...');
    try {
      const [kpis, apps, hourly] = await Promise.all([
        analyticsStructuredApi.kpis(c.effectiveSettings, c.importBatchId, c.analysisRunId),
        analyticsStructuredApi.appRank(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
        analyticsStructuredApi.hourlyTrend(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
      ]);
      setItems(kpis);
      setAppRows(apps.slice(0, 8));
      setHourlyRows(hourly.slice(0, 8));
      setStatus(`结构化 API 已刷新：KPI=${kpis.length} 项，App Rank=${apps.length} 行，Hourly=${hourly.length} 行。`);
    } catch (error) {
      setItems([]);
      setAppRows([]);
      setHourlyRows([]);
      setStatus(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => {
    void refresh();
  }, [c.importBatchId, c.analysisRunId]);

  return (
    <article className="analytics-card analytics-structured-kpi-panel">
      <div className="analytics-section-head">
        <div>
          <h3>结构化 Analytics API</h3>
          <p>读取 KPI、App Rank、Hourly Trend 三个结构化命令，验证看板从 MetricCard 代理表达向结构化 DWS/ADS API 迁移。</p>
        </div>
        <button type="button" disabled={disabled} onClick={refresh}>刷新结构化 API</button>
      </div>
      <div className="analytics-context-line"><span>{status}</span></div>
      <div className="analytics-kpi-strip analytics-structured-kpi-strip">
        {items.map((item) => (
          <article key={item.label} className="analytics-kpi-card">
            <span>{item.label}</span>
            <strong>{item.value}</strong>
            <small>{item.hint}</small>
          </article>
        ))}
        {!items.length && <p className="muted-row">暂无结构化 KPI。旧驾驶舱仍可继续使用既有 DWS/ADS API。</p>}
      </div>
      <div className="analytics-structured-preview-grid">
        <div className="analytics-table-wrap analytics-structured-preview-table">
          <table className="analytics-table">
            <thead><tr><th>Rank</th><th>App / User Type</th><th>Users</th><th>Evidence</th></tr></thead>
            <tbody>
              {appRows.map((row, index) => <tr key={`${row.label}-${index}`}><td>{index + 1}</td><td>{row.label}</td><td>{row.value}</td><td>{row.hint}</td></tr>)}
              {!appRows.length && <tr><td colSpan={4}>暂无结构化 App Rank 预览。</td></tr>}
            </tbody>
          </table>
        </div>
        <div className="analytics-table-wrap analytics-structured-preview-table">
          <table className="analytics-table">
            <thead><tr><th>Hour</th><th>Segment</th><th>Users</th><th>Evidence</th></tr></thead>
            <tbody>
              {hourlyRows.map((row, index) => <tr key={`${row.label}-${index}`}><td>{index + 1}</td><td>{row.label}</td><td>{row.value}</td><td>{row.hint}</td></tr>)}
              {!hourlyRows.length && <tr><td colSpan={4}>暂无结构化 Hourly Trend 预览。</td></tr>}
            </tbody>
          </table>
        </div>
      </div>
    </article>
  );
}

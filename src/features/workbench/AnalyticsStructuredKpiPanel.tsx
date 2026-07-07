import { useEffect, useState } from 'react';
import type { MetricCard } from '../../shared/types';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';

export function AnalyticsStructuredKpiPanel({ c }: { c: WorkbenchController }) {
  const [items, setItems] = useState<MetricCard[]>([]);
  const [status, setStatus] = useState('等待当前批次和 analysis_run_id。');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function refresh() {
    if (disabled) return;
    setStatus('正在读取结构化 KPI API...');
    try {
      const result = await workbenchApi.analyticsKpis(c.effectiveSettings, c.importBatchId, c.analysisRunId);
      setItems(result);
      setStatus(result.length ? `结构化 KPI 已刷新：${result.length} 项。` : '结构化 KPI API 返回空结果。');
    } catch (error) {
      setItems([]);
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
          <h3>结构化 KPI API</h3>
          <p>优先读取 `analytics_get_kpi_summary`，用于验证看板从 MetricCard 代理表达向结构化 DWS/ADS API 迁移。</p>
        </div>
        <button type="button" disabled={disabled} onClick={refresh}>刷新 KPI API</button>
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
    </article>
  );
}

import { useEffect, useState } from 'react';
import type { MetricCard } from '../../shared/types';
import { AnalyticsEvidenceTable } from './AnalyticsEvidenceTable';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import type { WorkbenchController } from './useWorkbenchController';

export function AnalyticsStructuredDeepDivePanel({ c }: { c: WorkbenchController }) {
  const [networkRows, setNetworkRows] = useState<MetricCard[]>([]);
  const [userRows, setUserRows] = useState<MetricCard[]>([]);
  const [leadRows, setLeadRows] = useState<MetricCard[]>([]);
  const [status, setStatus] = useState('等待当前批次和 analysis_run_id。');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function refresh() {
    if (disabled) return;
    setStatus('正在读取 Network / User / Lead 结构化 API...');
    try {
      const [network, users, leads] = await Promise.all([
        analyticsStructuredApi.networkHotspots(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
        analyticsStructuredApi.userProfiles(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
        analyticsStructuredApi.leadEvidence(c.effectiveSettings, c.importBatchId, c.analysisRunId).catch(() => []),
      ]);
      setNetworkRows(network);
      setUserRows(users);
      setLeadRows(leads);
      setStatus(`结构化深钻已刷新：Network=${network.length} 行，User=${users.length} 行，Lead=${leads.length} 行。`);
    } catch (error) {
      setNetworkRows([]);
      setUserRows([]);
      setLeadRows([]);
      setStatus(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => { void refresh(); }, [c.importBatchId, c.analysisRunId]);

  return (
    <article className="analytics-card analytics-structured-kpi-panel">
      <div className="analytics-section-head">
        <div>
          <h3>结构化深钻 API</h3>
          <p>读取 Network Hotspot、User Profile、Lead Evidence 三个结构化命令；所有深钻结果均进入可搜索、可导出、可查看详情的证据表。</p>
        </div>
        <button type="button" disabled={disabled} onClick={refresh}>刷新深钻 API</button>
      </div>
      <div className="analytics-context-line"><span>{status}</span></div>
      <div className="analytics-structured-evidence-grid analytics-structured-deep-grid">
        <AnalyticsEvidenceTable title="Structured Network Hotspot Evidence" rows={networkRows} limit={160} />
        <AnalyticsEvidenceTable title="Structured User Profile Evidence" rows={userRows} limit={240} />
        <AnalyticsEvidenceTable title="Structured Lead Evidence" rows={leadRows} limit={240} />
      </div>
    </article>
  );
}

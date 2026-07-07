import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import type { MetricCard } from '../../shared/types';
import type { WorkbenchController } from './useWorkbenchController';

type SectionProps = {
  title: string;
  rows: MetricCard[];
  valueLabel: string;
  emptyText: string;
};

function PreviewSection({ title, rows, valueLabel, emptyText }: SectionProps) {
  return (
    <div className="analytics-table-wrap analytics-structured-preview-table">
      <table className="analytics-table">
        <thead><tr><th>{title}</th><th>{valueLabel}</th><th>Evidence</th></tr></thead>
        <tbody>
          {rows.map((row, index) => <tr key={`${title}-${row.label}-${index}`}><td>{row.label}</td><td>{row.value}</td><td>{row.hint}</td></tr>)}
          {!rows.length && <tr><td colSpan={3}>{emptyText}</td></tr>}
        </tbody>
      </table>
    </div>
  );
}

export function AnalyticsStructuredDeepDivePanel({ c }: { c: WorkbenchController }) {
  const [networkRows, setNetworkRows] = useState<MetricCard[]>([]);
  const [userRows, setUserRows] = useState<MetricCard[]>([]);
  const [leadRows, setLeadRows] = useState<MetricCard[]>([]);
  const [status, setStatus] = useState('等待当前批次和 analysis_run_id。');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function refresh() {
    if (disabled) return;
    setStatus('正在读取 Network / User / Lead 结构化 API...');
    const req = { settings: c.effectiveSettings, import_batch_id: c.importBatchId, analysis_run_id: c.analysisRunId };
    try {
      const [network, users, leads] = await Promise.all([
        invoke<MetricCard[]>('analytics_get_network_hotspots', { req }).catch(() => []),
        invoke<MetricCard[]>('analytics_get_user_profiles', { req }).catch(() => []),
        invoke<MetricCard[]>('analytics_get_lead_evidence', { req }).catch(() => []),
      ]);
      setNetworkRows(network.slice(0, 10));
      setUserRows(users.slice(0, 10));
      setLeadRows(leads.slice(0, 10));
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
          <p>读取 Network Hotspot、User Profile、Lead Evidence 三个结构化命令，继续把看板从图表展示推进到可复核的业务证据链。</p>
        </div>
        <button type="button" disabled={disabled} onClick={refresh}>刷新深钻 API</button>
      </div>
      <div className="analytics-context-line"><span>{status}</span></div>
      <div className="analytics-structured-preview-grid analytics-structured-deep-grid">
        <PreviewSection title="Network Hotspot" rows={networkRows} valueLabel="Severity" emptyText="暂无结构化 Network Hotspot 预览。" />
        <PreviewSection title="User Profile" rows={userRows} valueLabel="Traffic GB" emptyText="暂无结构化 User Profile 预览。" />
        <PreviewSection title="Lead Evidence" rows={leadRows} valueLabel="Demand" emptyText="暂无结构化 Lead Evidence 预览。" />
      </div>
    </article>
  );
}

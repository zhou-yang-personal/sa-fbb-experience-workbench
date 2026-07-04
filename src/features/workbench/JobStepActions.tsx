import { useState } from 'react';
import type { EtlJobStepRow, MetricCard, MySqlSettings } from '../../shared/types';
import { jobApi } from './jobApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  setEtlSteps: (value: EtlJobStepRow[]) => void;
};

export function JobStepActions({ settings, importBatchId, loadMetrics, runAction, setEtlSteps }: Props) {
  const [status, setStatus] = useState('ALL');
  const [limit, setLimit] = useState('100');
  const parsedLimit = Math.max(1, Math.min(500, Number.parseInt(limit, 10) || 100));
  async function loadStepDetails(nextStatus = status) {
    const result = await runAction('etl_get_job_steps', () => jobApi.jobSteps(settings, importBatchId, { status: nextStatus, limit: parsedLimit }));
    if (Array.isArray(result)) setEtlSteps(result as EtlJobStepRow[]);
  }
  return (
    <>
      <button onClick={() => loadMetrics('etl_get_recent_steps', () => jobApi.recentSteps(settings, importBatchId))}>ETL Steps</button>
      <button onClick={() => loadMetrics('etl_get_failed_steps', () => jobApi.failedSteps(settings, importBatchId))}>Failed Steps</button>
      <select value={status} onChange={(e) => setStatus(e.target.value)}>
        <option value="ALL">All step status</option>
        <option value="pending">pending</option>
        <option value="running">running</option>
        <option value="success">success</option>
        <option value="failed">failed</option>
      </select>
      <input value={limit} onChange={(e) => setLimit(e.target.value)} inputMode="numeric" placeholder="limit" />
      <button onClick={() => loadStepDetails()}>Step Detail</button>
      <button onClick={() => { setStatus('failed'); void loadStepDetails('failed'); }}>Failed Detail</button>
    </>
  );
}

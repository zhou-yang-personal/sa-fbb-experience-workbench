import type { EtlJobStepRow, MetricCard, MySqlSettings } from '../../shared/types';
import { EtlActions } from './EtlActions';
import { JobStepActions } from './JobStepActions';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  etlSteps: EtlJobStepRow[];
  setEtlSteps: (value: EtlJobStepRow[]) => void;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
};

const etlFlow = [
  'RAW → CLEAN: standardize date, numeric fields, identity and access type',
  'CLEAN → DWS: user-day, app-category, access-hour and bottleneck aggregates',
  'DWS → ADS: overview, app category, quality and Cable/FTTH dashboard tables',
  'ADS → Final Lead: CRM / coverage / reachability fusion and action split',
  'Inspection: recent jobs, recent steps and failed steps',
];

function shortJobId(jobId: string) {
  return jobId.length > 12 ? `${jobId.slice(0, 12)}…` : jobId;
}

export function EtlJobCenter(props: Props) {
  const { settings, importBatchId, loadMetrics, etlSteps } = props;
  return (
    <section className="panel form-panel">
      <h2>ETL Job Center</h2>
      <p className="hero-text">以 import_batch_id 为主入口管理清洗、聚合、看板 ADS 和 Final Lead Fusion，并可回看 Job / Step 级状态。</p>
      <div className="action-row etl-step-actions">
        <button onClick={() => loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId))}>ETL Jobs</button>
        <JobStepActions {...props} />
        <EtlActions {...props} />
      </div>
      <ol className="pipeline-list">
        {etlFlow.map((item) => <li key={item}>{item}</li>)}
      </ol>
      <div className="table-like etl-step-table">
        <div className="table-row etl-step-row table-head"><span>Job</span><span>Step</span><span>Status</span><span>Target / Rows</span><span>Time</span><span>Message</span></div>
        {etlSteps.map((item, index) => (
          <div key={`${item.job_id}-${item.step_name}-${index}`} className="table-row etl-step-row">
            <span title={`${item.job_type} / ${item.job_id}`}>{item.job_type}<br /><small>{shortJobId(item.job_id)}</small></span>
            <span>{item.step_name}</span>
            <span>{item.status}</span>
            <span>{item.target_table ?? '-'}<br /><small>rows={item.affected_rows ?? 0}</small></span>
            <span><small>{item.started_at ?? '-'}</small><br /><small>{item.finished_at ?? '-'}</small></span>
            <span>{item.message ?? '-'}</span>
          </div>
        ))}
        {!etlSteps.length && <div className="table-row muted-row">No ETL step detail loaded. Click Step Detail or Failed Detail.</div>}
      </div>
    </section>
  );
}

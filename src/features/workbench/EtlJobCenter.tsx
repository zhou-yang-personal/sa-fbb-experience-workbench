import type { MetricCard, MySqlSettings } from '../../shared/types';
import { EtlActions } from './EtlActions';
import { JobStepActions } from './JobStepActions';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
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

export function EtlJobCenter(props: Props) {
  const { settings, importBatchId, loadMetrics } = props;
  return (
    <section className="panel form-panel">
      <h2>ETL Job Center</h2>
      <p className="hero-text">以 import_batch_id 为主入口管理清洗、聚合、看板 ADS 和 Final Lead Fusion，并可回看 Job / Step 级状态。</p>
      <div className="action-row">
        <button onClick={() => loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId))}>ETL Jobs</button>
        <JobStepActions {...props} />
        <EtlActions {...props} />
      </div>
      <ol className="pipeline-list">
        {etlFlow.map((item) => <li key={item}>{item}</li>)}
      </ol>
    </section>
  );
}

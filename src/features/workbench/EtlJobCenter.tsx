import type { ActionState, EtlJobStepRow, MetricCard, MySqlSettings } from '../../shared/types';
import { ActionButton } from './ActionButton';
import { EtlActions } from './EtlActions';
import { JobStepActions } from './JobStepActions';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  etlSteps: EtlJobStepRow[];
  setEtlSteps: (value: EtlJobStepRow[]) => void;
  actionStates: Record<string, ActionState>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
};

const etlFlow = [
  'RAW → CLEAN: standardize date, numeric fields, identity and access type',
  'CLEAN → DWS / ADS: user-day, app-category, quality and Cable/FTTH aggregates',
  'ADS → Final Lead: CRM / coverage / reachability fusion and action split',
  'Inspection: recent jobs, recent steps and failed steps',
];

function shortJobId(jobId: string) { return jobId.length > 12 ? `${jobId.slice(0, 12)}…` : jobId; }

export function EtlJobCenter(props: Props) {
  const { settings, importBatchId, analysisRunId, actionStates, runAction, loadMetrics, etlSteps } = props;
  const disabled = !importBatchId.trim();

  async function generateAnalysis() {
    await runAction('analyze_generate_results', async () => {
      await workbenchApi.clean(settings, importBatchId);
      await workbenchApi.aggregate(settings, importBatchId, analysisRunId);
      await workbenchApi.fuse(settings, importBatchId, analysisRunId);
      const jobs = await workbenchApi.jobs(settings, importBatchId);
      return { status: 'analysis_generated', jobs };
    });
    await loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId));
  }

  return (
    <section className="panel form-panel step-card">
      <div className="step-card-head">
        <div>
          <h2>Analyze：生成分析结果</h2>
          <p className="hero-text">一键执行 RAW→CLEAN、DWS/ADS 和 Final Lead Fusion。高级区保留单步排错入口。</p>
        </div>
        <span className="step-badge">4 / 5</span>
      </div>
      <div className="primary-action-row">
        <ActionButton actionKey="analyze_generate_results" actionStates={actionStates} primary label="生成分析结果" disabled={disabled} onClick={generateAnalysis} title={disabled ? '请先完成导入并生成 import_batch_id' : undefined} />
      </div>
      <details className="advanced-actions">
        <summary>高级操作：单步 ETL / 查看步骤</summary>
        <div className="action-row etl-step-actions">
          <ActionButton actionKey="etl_get_recent_jobs" actionStates={actionStates} label="ETL Jobs" disabled={disabled} onClick={() => loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId))} />
          <JobStepActions {...props} />
          <EtlActions {...props} />
        </div>
      </details>
      <ol className="pipeline-list">{etlFlow.map((item) => <li key={item}>{item}</li>)}</ol>
      <div className="table-like etl-step-table">
        <div className="table-row etl-step-row table-head"><span>Job</span><span>Step</span><span>Status</span><span>Target / Rows</span><span>Time</span><span>Message</span></div>
        {etlSteps.map((item, index) => (
          <div key={`${item.job_id}-${item.step_name}-${index}`} className="table-row etl-step-row">
            <span title={`${item.job_type} / ${item.job_id}`}>{item.job_type}<br /><small>{shortJobId(item.job_id)}</small></span>
            <span>{item.step_name}</span><span>{item.status}</span><span>{item.target_table ?? '-'}<br /><small>rows={item.affected_rows ?? 0}</small></span>
            <span><small>{item.started_at ?? '-'}</small><br /><small>{item.finished_at ?? '-'}</small></span><span>{item.message ?? '-'}</span>
          </div>
        ))}
        {!etlSteps.length && <div className="table-row muted-row">No ETL step detail loaded. Click Step Detail or Failed Detail.</div>}
      </div>
    </section>
  );
}

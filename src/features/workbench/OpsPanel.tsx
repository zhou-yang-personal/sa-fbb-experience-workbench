import type { DashboardOverview, FinalLeadUserRow, LeadUserRow, MetricCard, MySqlSettings } from '../../shared/types';
import { DashboardActions } from './DashboardActions';
import { EtlActions } from './EtlActions';
import { JobStepActions } from './JobStepActions';
import { LeadActions } from './LeadActions';
import { QualityActions } from './QualityActions';
import { RunFields } from './RunFields';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  setImportBatchId: (value: string) => void;
  analysisRunId: string;
  setAnalysisRunId: (value: string) => void;
  outputPath: string;
  setOutputPath: (value: string) => void;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
  setOverview: (value: DashboardOverview) => void;
  setLeads: (value: LeadUserRow[]) => void;
  setFinalLeads: (value: FinalLeadUserRow[]) => void;
};

export function OpsPanel(props: Props) {
  return (
    <section className="panel form-panel">
      <h2>Operations</h2>
      <RunFields {...props} />
      <div className="action-row">
        <QualityActions {...props} />
        <JobStepActions {...props} />
        <EtlActions {...props} />
        <DashboardActions {...props} />
        <LeadActions {...props} />
      </div>
    </section>
  );
}

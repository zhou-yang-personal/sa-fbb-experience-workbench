import type { FinalLeadUserRow, LeadUserRow, MetricCard, MySqlSettings } from '../../shared/types';
import { ExportPresetActions } from './ExportPresetActions';
import { LeadActions } from './LeadActions';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  outputPath: string;
  setOutputPath: (value: string) => void;
  exportFinalActions: string[];
  setExportFinalActions: (value: string[]) => void;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
  setLeads: (value: LeadUserRow[]) => void;
  setFinalLeads: (value: FinalLeadUserRow[]) => void;
};

const leadActions = [
  'IDENTITY_MAPPING_REQUIRED',
  'NETWORK_OPTIMIZATION_FIRST',
  'MARKET_FIBER_UPSELL',
  'REACHABILITY_FIX_FIRST',
  'BUILD_OR_COVERAGE_CHECK',
  'FTTH_SPEED_UPSELL',
];

export function FinalLeadCenter(props: Props) {
  const { settings, importBatchId, analysisRunId, loadMetrics, setOutputPath, setExportFinalActions } = props;
  return (
    <section className="panel form-panel">
      <h2>Final Lead Center</h2>
      <p className="hero-text">围绕 analysis_run_id 查询 SA Lead、Final Lead、Final Action Mix，并按 action 导出可交付 CSV。</p>
      <ExportPresetActions analysisRunId={analysisRunId} setOutputPath={setOutputPath} setExportFinalActions={setExportFinalActions} />
      <div className="action-row">
        <button onClick={() => loadMetrics('final_lead_summary', () => workbenchApi.leadSummary(settings, importBatchId, analysisRunId))}>Final Action Mix</button>
      </div>
      <LeadActions {...props} />
      <ol className="pipeline-list">
        {leadActions.map((item) => <li key={item}>{item}</li>)}
      </ol>
    </section>
  );
}

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
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
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
      <h2>迁转升套机会</h2>
      <p className="hero-text">围绕当前批次和 analysis_run_id 查询 SA Lead、Final Lead、Final Action Mix；导出按钮直接保留在本看板内。</p>
      <ExportPresetActions analysisRunId={analysisRunId} setOutputPath={setOutputPath} setExportFinalActions={setExportFinalActions} />
      <div className="action-row">
        <button onClick={() => loadMetrics('final_lead_summary', () => workbenchApi.leadSummary(settings, importBatchId, analysisRunId))}>刷新 Final Action Mix</button>
      </div>
      <LeadActions {...props} />
      <ol className="pipeline-list">
        {leadActions.map((item) => <li key={item}>{item}</li>)}
      </ol>
    </section>
  );
}

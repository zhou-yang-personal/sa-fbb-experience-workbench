import type { MySqlSettings } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
};

export function EtlActions({ settings, importBatchId, analysisRunId, runAction }: Props) {
  return (
    <>
      <button onClick={() => runAction('etl_start_clean_job', () => workbenchApi.clean(settings, importBatchId))}>RAW→CLEAN</button>
      <button onClick={() => runAction('etl_start_aggregate_job', () => workbenchApi.aggregate(settings, importBatchId, analysisRunId))}>DWS/ADS</button>
      <button onClick={() => runAction('leads_run_final_fusion', () => workbenchApi.fuse(settings, importBatchId, analysisRunId))}>配置化融合</button>
    </>
  );
}

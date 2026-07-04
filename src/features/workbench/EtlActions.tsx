import type { ActionState, MySqlSettings } from '../../shared/types';
import { ActionButton } from './ActionButton';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  analysisRunId: string;
  actionStates: Record<string, ActionState>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
};

export function EtlActions({ settings, importBatchId, analysisRunId, actionStates, runAction }: Props) {
  const disabled = !importBatchId.trim();
  return (
    <>
      <ActionButton actionKey="etl_start_clean_job" actionStates={actionStates} label="RAW→CLEAN" disabled={disabled} onClick={() => runAction('etl_start_clean_job', () => workbenchApi.clean(settings, importBatchId))} />
      <ActionButton actionKey="etl_start_aggregate_job" actionStates={actionStates} label="DWS/ADS" disabled={disabled} onClick={() => runAction('etl_start_aggregate_job', () => workbenchApi.aggregate(settings, importBatchId, analysisRunId))} />
      <ActionButton actionKey="leads_run_final_fusion" actionStates={actionStates} label="配置化融合" disabled={disabled} onClick={() => runAction('leads_run_final_fusion', () => workbenchApi.fuse(settings, importBatchId, analysisRunId))} />
    </>
  );
}

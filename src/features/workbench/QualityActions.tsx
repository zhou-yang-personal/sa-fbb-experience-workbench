import type { MetricCard, MySqlSettings } from '../../shared/types';
import { qualityApi } from './qualityApi';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
};

export function QualityActions({ settings, importBatchId, runAction, loadMetrics }: Props) {
  return (
    <>
      <button onClick={() => loadMetrics('quality_get_batch_report', () => workbenchApi.qualityBasic(settings, importBatchId))}>基础质量</button>
      <button onClick={() => runAction('quality_run_gate', () => workbenchApi.qualityGate(settings, importBatchId))}>完整质量门禁</button>
      <button onClick={() => loadMetrics('quality_get_gate_results', () => qualityApi.allResults(settings, importBatchId))}>质量结果</button>
      <button onClick={() => loadMetrics('quality_get_failed_results', () => qualityApi.failedResults(settings, importBatchId))}>失败质量项</button>
      <button onClick={() => loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId))}>ETL状态</button>
    </>
  );
}

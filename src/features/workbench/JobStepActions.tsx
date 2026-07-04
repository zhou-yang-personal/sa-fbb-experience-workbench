import type { MetricCard, MySqlSettings } from '../../shared/types';
import { jobApi } from './jobApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
};

export function JobStepActions({ settings, importBatchId, loadMetrics }: Props) {
  return (
    <>
      <button onClick={() => loadMetrics('etl_get_recent_steps', () => jobApi.recentSteps(settings, importBatchId))}>ETL Steps</button>
      <button onClick={() => loadMetrics('etl_get_failed_steps', () => jobApi.failedSteps(settings, importBatchId))}>Failed Steps</button>
    </>
  );
}

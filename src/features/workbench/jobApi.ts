import { invoke } from '@tauri-apps/api/core';
import type { EtlJobStepRow, EtlJobStepsQuery, MetricCard, MySqlSettings } from '../../shared/types';

function normalizeFilter(value?: string) {
  const normalized = value?.trim();
  return normalized && normalized !== 'ALL' ? normalized : undefined;
}

export const jobApi = {
  recentSteps: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('etl_get_recent_steps', { settings, importBatchId }),
  failedSteps: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('etl_get_failed_steps', { settings, importBatchId }),
  jobSteps: (settings: MySqlSettings, importBatchId: string, query?: EtlJobStepsQuery) => invoke<EtlJobStepRow[]>('etl_get_job_steps', {
    req: {
      settings,
      import_batch_id: importBatchId,
      job_id: normalizeFilter(query?.jobId),
      status: normalizeFilter(query?.status),
      limit: query?.limit ?? 100,
    },
  }),
};

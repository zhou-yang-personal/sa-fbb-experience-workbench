import { invoke } from '@tauri-apps/api/core';
import type { MetricCard, MySqlSettings } from '../../shared/types';

export const jobApi = {
  recentSteps: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('etl_get_recent_steps', { settings, importBatchId }),
  failedSteps: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('etl_get_failed_steps', { settings, importBatchId }),
};

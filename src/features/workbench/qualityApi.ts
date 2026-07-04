import { invoke } from '@tauri-apps/api/core';
import type { MetricCard, MySqlSettings } from '../../shared/types';

export const qualityApi = {
  allResults: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('quality_get_gate_results', { settings, importBatchId }),
  failedResults: (settings: MySqlSettings, importBatchId: string) => invoke<MetricCard[]>('quality_get_failed_results', { settings, importBatchId }),
};

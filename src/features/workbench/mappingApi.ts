import { invoke } from '@tauri-apps/api/core';
import type { MetricCard, MySqlSettings } from '../../shared/types';

export const mappingApi = {
  summary: (settings: MySqlSettings, importBatchId: string, dataType: string) =>
    invoke<MetricCard[]>('import_get_mapping_summary', { settings, importBatchId, dataType }),
  results: (settings: MySqlSettings, importBatchId: string, dataType: string) =>
    invoke<MetricCard[]>('import_get_mapping_results', { settings, importBatchId, dataType }),
};

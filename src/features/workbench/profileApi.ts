import { invoke } from '@tauri-apps/api/core';
import type { CommandAck, MySqlSettings } from '../../shared/types';

export const profileApi = {
  refresh: (settings: MySqlSettings, importBatchId: string, dataType: string) =>
    invoke<CommandAck>('dataset_profile_refresh', { settings, importBatchId, dataType }),
};

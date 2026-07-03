import { invoke } from '@tauri-apps/api/core';

export async function dbTestConnection(): Promise<string> {
  return invoke<string>('db_test_connection');
}

export async function importProbeCsv(path: string): Promise<string> {
  return invoke<string>('import_probe_csv', { path });
}

export async function importCreateBatch(dataType: string, sourceFileName: string): Promise<string> {
  return invoke<string>('import_create_batch', { dataType, sourceFileName });
}

export async function importStartRawLoad(importBatchId: string): Promise<string> {
  return invoke<string>('import_start_raw_load', { importBatchId });
}

export async function etlStartCleanJob(importBatchId: string): Promise<string> {
  return invoke<string>('etl_start_clean_job', { importBatchId });
}

export async function etlStartAggregateJob(importBatchId: string): Promise<string> {
  return invoke<string>('etl_start_aggregate_job', { importBatchId });
}

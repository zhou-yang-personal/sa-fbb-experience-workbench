import { open, save } from '@tauri-apps/plugin-dialog';

export async function selectCsvFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'CSV files', extensions: ['csv'] }],
  });
  if (Array.isArray(selected)) return selected[0] ?? '';
  return typeof selected === 'string' ? selected : '';
}

export async function selectCsvSavePath(defaultPath: string) {
  const selected = await save({
    defaultPath,
    filters: [{ name: 'CSV files', extensions: ['csv'] }],
  });
  return typeof selected === 'string' ? selected : '';
}

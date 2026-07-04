import { selectCsvSavePath } from './fileDialogs';

type Props = {
  analysisRunId: string;
  setOutputPath: (value: string) => void;
  setExportFinalActions: (value: string[]) => void;
};

function safeRunId(value: string) { return value.trim().replace(/[^a-zA-Z0-9_-]/g, '_') || 'RUN_MANUAL'; }

export function buildExportFileName(prefix: string, analysisRunId: string) {
  const runId = safeRunId(analysisRunId);
  const stamp = new Date().toISOString().slice(0, 16).replace(/[-:T]/g, '');
  return `${prefix}_${runId}_${stamp}.csv`;
}

export function ExportPresetActions({ analysisRunId, setOutputPath, setExportFinalActions }: Props) {
  const presets = [
    { label: 'SA Lead CSV', prefix: 'sa_leads', finalActions: [] },
    { label: 'Final Lead CSV', prefix: 'final_leads', finalActions: [] },
    { label: 'Market Upsell CSV', prefix: 'market_upsell', finalActions: ['MARKET_FIBER_UPSELL', 'FTTH_SPEED_UPSELL'] },
    { label: 'Reachability CSV', prefix: 'reachability_coverage', finalActions: ['REACHABILITY_FIX_FIRST', 'BUILD_OR_COVERAGE_CHECK'] },
  ];
  async function choosePreset(prefix: string, finalActions: string[]) {
    const selected = await selectCsvSavePath(buildExportFileName(prefix, analysisRunId));
    if (selected) {
      setOutputPath(selected);
      setExportFinalActions(finalActions);
    }
  }
  return (
    <div className="summary-pills export-preset-pills">
      {presets.map((preset) => (
        <button key={preset.label} type="button" className="status-pill" onClick={() => { void choosePreset(preset.prefix, preset.finalActions); }}>{preset.label}</button>
      ))}
    </div>
  );
}

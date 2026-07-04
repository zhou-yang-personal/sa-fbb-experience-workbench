type Props = {
  analysisRunId: string;
  setOutputPath: (value: string) => void;
  setExportFinalActions: (value: string[]) => void;
};

function safeRunId(value: string) {
  return value.trim().replace(/[^a-zA-Z0-9_-]/g, '_') || 'RUN_MANUAL';
}

export function ExportPresetActions({ analysisRunId, setOutputPath, setExportFinalActions }: Props) {
  const runId = safeRunId(analysisRunId);
  const date = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  const presets = [
    { label: 'SA Lead CSV', outputPath: `sa_leads_${runId}_${date}.csv`, finalActions: [] },
    { label: 'Final Lead CSV', outputPath: `final_leads_${runId}_${date}.csv`, finalActions: [] },
    { label: 'Market Upsell CSV', outputPath: `market_upsell_${runId}_${date}.csv`, finalActions: ['MARKET_FIBER_UPSELL', 'FTTH_SPEED_UPSELL'] },
    { label: 'Reachability CSV', outputPath: `reachability_coverage_${runId}_${date}.csv`, finalActions: ['REACHABILITY_FIX_FIRST', 'BUILD_OR_COVERAGE_CHECK'] },
  ];
  return (
    <div className="summary-pills">
      {presets.map((preset) => (
        <button key={preset.label} type="button" className="status-pill" onClick={() => { setOutputPath(preset.outputPath); setExportFinalActions(preset.finalActions); }}>
          {preset.label}
        </button>
      ))}
    </div>
  );
}

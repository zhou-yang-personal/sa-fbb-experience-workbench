type Props = {
  analysisRunId: string;
  setOutputPath: (value: string) => void;
};

function safeRunId(value: string) {
  return value.trim().replace(/[^a-zA-Z0-9_-]/g, '_') || 'RUN_MANUAL';
}

export function ExportPresetActions({ analysisRunId, setOutputPath }: Props) {
  const runId = safeRunId(analysisRunId);
  const date = new Date().toISOString().slice(0, 10).replace(/-/g, '');
  return (
    <div className="summary-pills">
      <button type="button" className="status-pill" onClick={() => setOutputPath(`sa_leads_${runId}_${date}.csv`)}>SA Lead CSV</button>
      <button type="button" className="status-pill" onClick={() => setOutputPath(`final_leads_${runId}_${date}.csv`)}>Final Lead CSV</button>
      <button type="button" className="status-pill" onClick={() => setOutputPath(`market_fiber_upsell_${runId}_${date}.csv`)}>Market Upsell CSV</button>
      <button type="button" className="status-pill" onClick={() => setOutputPath(`reachability_fix_${runId}_${date}.csv`)}>Reachability CSV</button>
    </div>
  );
}

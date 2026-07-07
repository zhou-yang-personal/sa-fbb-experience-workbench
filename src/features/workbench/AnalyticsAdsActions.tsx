import { useState } from 'react';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import type { WorkbenchController } from './useWorkbenchController';

export function AnalyticsAdsActions({ c }: { c: WorkbenchController }) {
  const [message, setMessage] = useState('Ready to materialize structured ADS.');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function runAppRank() {
    if (disabled) return;
    setMessage('Running App Rank ADS materialization...');
    try {
      const result = await analyticsStructuredApi.materializeAppRank(c.effectiveSettings, c.importBatchId, c.analysisRunId);
      setMessage(result.message);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <article className="analytics-card analytics-structured-kpi-panel">
      <div className="analytics-section-head">
        <div>
          <h3>Structured ADS Materialization</h3>
          <p>Run materialization jobs for structured analytics ADS tables.</p>
        </div>
        <button type="button" disabled={disabled} onClick={runAppRank}>Materialize App Rank ADS</button>
      </div>
      <div className="analytics-context-line"><span>{message}</span></div>
    </article>
  );
}

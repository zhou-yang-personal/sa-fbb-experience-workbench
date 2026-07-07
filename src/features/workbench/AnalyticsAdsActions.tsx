import { useState } from 'react';
import { analyticsStructuredApi } from './analyticsStructuredApi';
import type { WorkbenchController } from './useWorkbenchController';

type ActionKey = 'app' | 'hour' | 'net' | 'user' | 'lead';

const ACTIONS: { key: ActionKey; label: string }[] = [
  { key: 'app', label: 'App Rank' },
  { key: 'hour', label: 'Hourly Trend' },
  { key: 'net', label: 'Network Hotspot' },
  { key: 'user', label: 'User Profile' },
  { key: 'lead', label: 'Lead Evidence' },
];

export function AnalyticsAdsActions({ c }: { c: WorkbenchController }) {
  const [message, setMessage] = useState('Ready to materialize structured ADS.');
  const disabled = !c.importBatchId.trim() || !c.analysisRunId.trim();

  async function run(key: ActionKey) {
    if (disabled) return;
    setMessage(`Running ${key} ADS materialization...`);
    try {
      const api = analyticsStructuredApi;
      const result = key === 'app'
        ? await api.materializeAppRank(c.effectiveSettings, c.importBatchId, c.analysisRunId)
        : key === 'hour'
          ? await api.materializeHourly(c.effectiveSettings, c.importBatchId, c.analysisRunId)
          : key === 'net'
            ? await api.materializeNetwork(c.effectiveSettings, c.importBatchId, c.analysisRunId)
            : key === 'user'
              ? await api.materializeUser(c.effectiveSettings, c.importBatchId, c.analysisRunId)
              : await api.materializeLead(c.effectiveSettings, c.importBatchId, c.analysisRunId);
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
        <div className="button-row">
          {ACTIONS.map((item) => <button key={item.key} type="button" disabled={disabled} onClick={() => run(item.key)}>{item.label}</button>)}
        </div>
      </div>
      <div className="analytics-context-line"><span>{message}</span></div>
    </article>
  );
}

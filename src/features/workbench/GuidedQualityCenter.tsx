import { useMemo, useState } from 'react';
import type { ActionState, MetricCard, MySqlSettings } from '../../shared/types';
import { ActionButton } from './ActionButton';
import { qualityApi } from './qualityApi';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  actionStates: Record<string, ActionState>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
};

function parseQualityValue(value: string) {
  const parts = value.split(' · ');
  const passed = parts[0]?.startsWith('passed=') ? parts[0].replace('passed=', '') : 'unknown';
  const severity = parts[0]?.startsWith('passed=') ? parts[1] ?? 'info' : parts[0] ?? 'info';
  const metricPart = parts[0]?.startsWith('passed=') ? parts[2] ?? 'value=' : parts[1] ?? 'value=';
  const [metricName, metricValue] = metricPart.split('=');
  return { passed, severity, metricName: metricName || 'value', metricValue: metricValue ?? value };
}

export function GuidedQualityCenter({ settings, importBatchId, actionStates, runAction, loadMetrics }: Props) {
  const [qualityRows, setQualityRows] = useState<MetricCard[]>([]);
  const [mode, setMode] = useState('未加载 quality gate');
  const parsedRows = useMemo(() => qualityRows.map((item) => ({ ...item, ...parseQualityValue(item.value) })), [qualityRows]);
  const counts = useMemo(() => {
    const result = { failed: 0, warning: 0, info: 0, passed: 0 };
    for (const item of parsedRows) {
      const severity = item.severity.toLowerCase();
      if (item.passed === '1') result.passed += 1;
      if (item.passed === '0' || severity.includes('fail')) result.failed += 1;
      else if (severity.includes('warn')) result.warning += 1;
      else result.info += 1;
    }
    return result;
  }, [parsedRows]);

  async function loadAllResults() {
    const result = await loadMetrics('quality_get_gate_results', () => qualityApi.allResults(settings, importBatchId));
    setQualityRows(result);
    setMode(result.length ? '完整质量门禁结果' : '未跑 quality gate');
    return result;
  }

  async function runGateAndRefresh() {
    await runAction('quality_run_gate', () => workbenchApi.qualityGate(settings, importBatchId));
    await loadAllResults();
  }

  async function loadFailedResults() {
    const result = await loadMetrics('quality_get_failed_results', () => qualityApi.failedResults(settings, importBatchId));
    setQualityRows(result);
    setMode(result.length ? '失败质量项' : '当前 batch 无失败项');
  }

  async function loadBasicQuality() {
    const result = await loadMetrics('quality_get_batch_report', () => workbenchApi.qualityBasic(settings, importBatchId));
    setQualityRows(result);
    setMode(result.length ? '基础质量' : '基础质量为空');
  }

  const canRun = Boolean(importBatchId.trim());
  const conclusion = !parsedRows.length ? '未运行质量检查。' : counts.failed > 0 ? `存在 ${counts.failed} 个阻断项，建议先处理。` : counts.warning > 0 ? `无阻断项，但存在 ${counts.warning} 个 warning，可继续但需标记风险。` : '质量检查通过，可以进入分析。';

  return (
    <section className="panel form-panel step-card">
      <div className="step-card-head">
        <div>
          <h2>Validate：质量检查</h2>
          <p className="hero-text">检查 RAW 完整性、用户字段、接入类型、时间范围、应用数量和拓扑有效性。</p>
        </div>
        <span className="step-badge">3 / 5</span>
      </div>
      <div className="primary-action-row">
        <ActionButton actionKey="quality_run_gate" actionStates={actionStates} primary label="运行质量检查" disabled={!canRun} onClick={runGateAndRefresh} title={!canRun ? '请先完成导入并生成 import_batch_id' : undefined} />
      </div>
      <section className={`quality-conclusion ${counts.failed ? 'status-failure-text' : ''}`}>
        <strong>{conclusion}</strong>
        <small>{mode}</small>
      </section>
      <details className="advanced-actions">
        <summary>高级操作</summary>
        <div className="action-row">
          <ActionButton actionKey="quality_get_batch_report" actionStates={actionStates} label="基础质量" disabled={!canRun} onClick={loadBasicQuality} />
          <ActionButton actionKey="quality_get_gate_results" actionStates={actionStates} label="全部结果" disabled={!canRun} onClick={loadAllResults} />
          <ActionButton actionKey="quality_get_failed_results" actionStates={actionStates} label="失败项" disabled={!canRun} onClick={loadFailedResults} />
          <ActionButton actionKey="etl_get_recent_jobs" actionStates={actionStates} label="ETL 状态" disabled={!canRun} onClick={() => loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId)).then(() => setMode('ETL 状态已刷新'))} />
        </div>
      </details>
      <div className="summary-pills">
        <span className="status-pill">passed {counts.passed}</span>
        <span className="status-pill status-failure">failed {counts.failed}</span>
        <span className="status-pill">warning {counts.warning}</span>
        <span className="status-pill">info {counts.info}</span>
      </div>
      <div className="table-like">
        <div className="table-row table-head quality-row"><span>Level</span><span>Check</span><span>Metric</span><span>Value</span><span>Passed</span><span>Text</span></div>
        {parsedRows.map((item, index) => (
          <div key={`${item.label}-${item.metricName}-${index}`} className="table-row quality-row"><span>{item.severity}</span><span>{item.label}</span><span>{item.metricName}</span><span>{item.metricValue}</span><span>{item.passed}</span><span>{item.hint}</span></div>
        ))}
        {!parsedRows.length && <div className="table-row muted-row">未跑 quality gate，或当前 batch 无数据。</div>}
      </div>
    </section>
  );
}

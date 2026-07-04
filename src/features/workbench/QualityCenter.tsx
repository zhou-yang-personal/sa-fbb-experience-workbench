import { useMemo, useState } from 'react';
import type { MetricCard, MySqlSettings } from '../../shared/types';
import { qualityApi } from './qualityApi';
import { QualityActions } from './QualityActions';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<MetricCard[]>;
};

const qualityChecks = [
  'CSV vs RAW row reconciliation',
  'User identity usability',
  'Cable / FTTH access mix',
  'Time range and active-hour distribution',
  'App count and topology UNKNOWN ratio',
  'Failed quality item drill-down',
];

function parseQualityValue(value: string) {
  const rich = value.match(/^passed=(\d)\s+·\s+(.+?)\s+·\s+([^=]+)=(.*)$/);
  if (rich) {
    const [, passed, severity, metricName, metricValue] = rich;
    return { passed, severity, metricName, metricValue };
  }
  const legacy = value.match(/^(.+?)\s+·\s+([^=]+)=(.*)$/);
  if (legacy) {
    const [, severity, metricName, metricValue] = legacy;
    return { passed: 'unknown', severity, metricName, metricValue };
  }
  return { passed: 'unknown', severity: 'info', metricName: 'value', metricValue: value };
}

export function QualityCenter(props: Props) {
  const { settings, importBatchId, runAction, loadMetrics } = props;
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

  async function loadBasicQuality() {
    const result = await loadMetrics('quality_get_batch_report', () => workbenchApi.qualityBasic(settings, importBatchId));
    setQualityRows(result);
    setMode(result.length ? '基础质量' : '基础质量为空');
  }

  async function loadAllResults() {
    const result = await loadMetrics('quality_get_gate_results', () => qualityApi.allResults(settings, importBatchId));
    setQualityRows(result);
    setMode(result.length ? '完整质量门禁结果' : '未跑 quality gate');
  }

  async function loadFailedResults() {
    const result = await loadMetrics('quality_get_failed_results', () => qualityApi.failedResults(settings, importBatchId));
    setQualityRows(result);
    setMode(result.length ? '失败质量项' : '当前 batch 无失败项');
  }

  async function loadEtlStatus() {
    const result = await loadMetrics('etl_get_recent_jobs', () => workbenchApi.jobs(settings, importBatchId));
    setMode(result.length ? 'ETL 状态' : '当前无 ETL 状态');
  }

  async function runGateAndRefresh() {
    await runAction('quality_run_gate', () => workbenchApi.qualityGate(settings, importBatchId));
    await loadAllResults();
  }

  return (
    <section className="panel form-panel">
      <h2>Quality Center</h2>
      <p className="hero-text">围绕 import_batch_id 做 RAW 入库完整性、字段可用性、接入类型、时间范围、应用数量和拓扑有效性的集中检查。</p>
      <div className="action-row">
        <QualityActions
          onLoadBasicQuality={loadBasicQuality}
          onRunGate={runGateAndRefresh}
          onLoadResults={loadAllResults}
          onLoadFailedResults={loadFailedResults}
          onLoadEtlStatus={loadEtlStatus}
        />
      </div>
      <div className="summary-pills">
        <span className="status-pill">passed {counts.passed}</span>
        <span className="status-pill status-failure">failed {counts.failed}</span>
        <span className="status-pill">warning {counts.warning}</span>
        <span className="status-pill">info {counts.info}</span>
      </div>
      <p className="muted-row">{mode}</p>
      <div className="table-like">
        <div className="table-row table-head quality-row"><span>Level</span><span>Check</span><span>Metric</span><span>Value</span><span>Passed</span><span>Text</span></div>
        {parsedRows.map((item, index) => (
          <div key={`${item.label}-${item.metricName}-${index}`} className="table-row quality-row">
            <span>{item.severity}</span>
            <span>{item.label}</span>
            <span>{item.metricName}</span>
            <span>{item.metricValue}</span>
            <span>{item.passed}</span>
            <span>{item.hint}</span>
          </div>
        ))}
        {!parsedRows.length && <div className="table-row muted-row">未跑 quality gate，或当前 batch 无数据。</div>}
      </div>
      <ol className="pipeline-list">
        {qualityChecks.map((item) => <li key={item}>{item}</li>)}
      </ol>
    </section>
  );
}

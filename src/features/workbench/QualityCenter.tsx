import type { MetricCard, MySqlSettings } from '../../shared/types';
import { QualityActions } from './QualityActions';

type Props = {
  settings: MySqlSettings;
  importBatchId: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  loadMetrics: (label: string, action: () => Promise<MetricCard[]>) => Promise<void>;
};

const qualityChecks = [
  'CSV vs RAW row reconciliation',
  'User identity usability',
  'Cable / FTTH access mix',
  'Time range and active-hour distribution',
  'App count and topology UNKNOWN ratio',
  'Failed quality item drill-down',
];

export function QualityCenter(props: Props) {
  return (
    <section className="panel form-panel">
      <h2>Quality Center</h2>
      <p className="hero-text">围绕 import_batch_id 做 RAW 入库完整性、字段可用性、接入类型、时间范围、应用数量和拓扑有效性的集中检查。</p>
      <div className="action-row">
        <QualityActions {...props} />
      </div>
      <ol className="pipeline-list">
        {qualityChecks.map((item) => <li key={item}>{item}</li>)}
      </ol>
    </section>
  );
}

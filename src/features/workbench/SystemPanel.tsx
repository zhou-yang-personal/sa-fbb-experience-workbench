import { useState } from 'react';
import { ConnectionPanel } from './ConnectionPanel';
import { EtlJobCenter } from './EtlJobCenter';
import { ExecutionLog } from './ExecutionLog';
import { QualityCenter } from './QualityCenter';
import { SystemDiagnostics } from './SystemDiagnostics';
import type { WorkbenchController } from './useWorkbenchController';

export function SystemPanel({ c }: { c: WorkbenchController }) {
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [executionLogOpen, setExecutionLogOpen] = useState(false);

  return (
    <section className="workbench-section-stack system-workspace">
      <article className="panel form-panel">
        <h2>系统管理</h2>
        <p className="hero-text">这里只保留数据库连接、数据可用性检查、后台任务和诊断日志。它们支撑看板，但不再作为产品主入口。</p>
      </article>
      <ConnectionPanel settings={c.settings} setSettings={c.setSettings} runAction={c.runAction} clearPersistedContext={c.clearPersistedContext} actionStates={c.actionStates} />
      <SystemDiagnostics settings={c.settings} importBatchId={c.importBatchId} analysisRunId={c.analysisRunId} dataType={c.dataType} />
      <details className="advanced-actions" onToggle={(event) => setAdvancedOpen(event.currentTarget.open)}>
        <summary>高级排错：Quality Gate / ETL 单步入口（展开后加载）</summary>
        {advancedOpen && <>
          <QualityCenter {...c} />
          <EtlJobCenter {...c} />
        </>}
      </details>
      <details className="advanced-actions" onToggle={(event) => setExecutionLogOpen(event.currentTarget.open)}>
        <summary>本次执行日志（{c.log.length} 条，展开后渲染）</summary>
        {executionLogOpen && <ExecutionLog log={c.log} />}
      </details>
    </section>
  );
}

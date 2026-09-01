import { useState } from 'react';
import { ConnectionPanel } from './ConnectionPanel';
import { DuckDbWorkspacePanel } from './DuckDbWorkspacePanel';
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
        <p className="hero-text">管理程序自动保存的 DuckDB/Parquet 数据、后台任务和诊断日志；MySQL 仅作为迁移期兼容入口。</p>
      </article>
      <DuckDbWorkspacePanel c={c} />
      <details className="advanced-actions">
        <summary>兼容模式：旧版 MySQL 工作流</summary>
        <ConnectionPanel settings={c.settings} setSettings={c.setSettings} runAction={c.runAction} clearPersistedContext={c.clearPersistedContext} actionStates={c.actionStates} />
      </details>
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

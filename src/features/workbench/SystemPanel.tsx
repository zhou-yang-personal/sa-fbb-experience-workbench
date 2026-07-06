import { ConnectionPanel } from './ConnectionPanel';
import { EtlJobCenter } from './EtlJobCenter';
import { ExecutionLog } from './ExecutionLog';
import { QualityCenter } from './QualityCenter';
import type { WorkbenchController } from './useWorkbenchController';

export function SystemPanel({ c }: { c: WorkbenchController }) {
  return (
    <section className="workbench-section-stack system-workspace">
      <article className="panel form-panel">
        <h2>系统管理</h2>
        <p className="hero-text">这里只保留数据库连接、数据可用性检查、后台任务和诊断日志。它们支撑看板，但不再作为产品主入口。</p>
      </article>
      <ConnectionPanel settings={c.settings} setSettings={c.setSettings} runAction={c.runAction} clearPersistedContext={c.clearPersistedContext} actionStates={c.actionStates} />
      <QualityCenter {...c} />
      <EtlJobCenter {...c} />
      <ExecutionLog log={c.log} />
    </section>
  );
}

import { ConnectionPanel } from './ConnectionPanel';
import { DashboardCenter } from './DashboardCenter';
import { EtlJobCenter } from './EtlJobCenter';
import { ExecutionLog } from './ExecutionLog';
import { FinalLeadCenter } from './FinalLeadCenter';
import { ImportPanel } from './ImportPanel';
import { MetricGrid } from './MetricGrid';
import { QualityCenter } from './QualityCenter';
import { ResultTables } from './ResultTables';
import { WorkbenchHeader } from './WorkbenchHeader';
import { useWorkbenchController } from './useWorkbenchController';
import './extra.css';

const pipelineSteps = ['Import', 'Mapping', 'Quality', 'ETL', 'Dashboard', 'Final Leads', 'Export'];

export function WorkbenchAppV2() {
  const c = useWorkbenchController();
  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">SA FBB Experience Workbench</div>
        <nav>{['Settings', 'Import', 'Mapping', 'Quality', 'ETL', 'Dashboard', 'Leads'].map((item) => <button key={item} type="button" className="nav-item">{item}</button>)}</nav>
      </aside>
      <section className="content">
        <WorkbenchHeader />
        <ConnectionPanel settings={c.settings} setSettings={c.setSettings} runAction={c.runAction} />
        <section className="two-column">
          <ImportPanel {...c} />
          <article className="panel"><h2>Pipeline</h2><ol className="pipeline-list">{pipelineSteps.map((step) => <li key={step}>{step}</li>)}</ol></article>
        </section>
        <section className="two-column">
          <QualityCenter {...c} />
          <EtlJobCenter {...c} />
        </section>
        <section className="two-column">
          <DashboardCenter {...c} />
          <FinalLeadCenter {...c} />
        </section>
        <MetricGrid metrics={c.allMetrics} />
        <ResultTables leads={c.leads} finalLeads={c.finalLeads} />
        <ExecutionLog log={c.log} />
      </section>
    </main>
  );
}

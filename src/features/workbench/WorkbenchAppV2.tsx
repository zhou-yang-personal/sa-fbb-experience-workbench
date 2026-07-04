import { useState } from 'react';
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

type WorkbenchSection = 'settings' | 'import' | 'quality' | 'etl' | 'dashboard' | 'leads' | 'results' | 'log';

const pipelineSteps = ['Import', 'Mapping', 'Quality', 'ETL', 'Dashboard', 'Final Leads', 'Export'];
const navItems: Array<{ id: WorkbenchSection; label: string; hint: string }> = [
  { id: 'settings', label: 'Settings', hint: 'MySQL connection' },
  { id: 'import', label: 'Import', hint: 'CSV, mapping, RAW load' },
  { id: 'quality', label: 'Quality', hint: 'RAW gate and profile' },
  { id: 'etl', label: 'ETL', hint: 'Jobs and step details' },
  { id: 'dashboard', label: 'Dashboard', hint: 'ADS / DWS charts' },
  { id: 'leads', label: 'Leads', hint: 'Lead and export actions' },
  { id: 'results', label: 'Results', hint: 'Metrics and user tables' },
  { id: 'log', label: 'Run Log', hint: 'Execution trace' },
];

export function WorkbenchAppV2() {
  const c = useWorkbenchController();
  const [activeSection, setActiveSection] = useState<WorkbenchSection>('settings');

  function renderSection() {
    if (activeSection === 'settings') {
      return <ConnectionPanel settings={c.settings} setSettings={c.setSettings} runAction={c.runAction} />;
    }
    if (activeSection === 'import') {
      return (
        <section className="two-column workbench-section-grid">
          <ImportPanel {...c} />
          <article className="panel">
            <h2>Pipeline</h2>
            <ol className="pipeline-list">{pipelineSteps.map((step) => <li key={step}>{step}</li>)}</ol>
          </article>
        </section>
      );
    }
    if (activeSection === 'quality') {
      return (
        <section className="workbench-section-stack">
          <QualityCenter {...c} />
          <MetricGrid metrics={c.allMetrics} />
        </section>
      );
    }
    if (activeSection === 'etl') {
      return <EtlJobCenter {...c} />;
    }
    if (activeSection === 'dashboard') {
      return (
        <section className="workbench-section-stack">
          <DashboardCenter {...c} />
          <MetricGrid metrics={c.allMetrics} />
        </section>
      );
    }
    if (activeSection === 'leads') {
      return <FinalLeadCenter {...c} />;
    }
    if (activeSection === 'results') {
      return (
        <section className="workbench-section-stack">
          <MetricGrid metrics={c.allMetrics} />
          <ResultTables leads={c.leads} finalLeads={c.finalLeads} />
        </section>
      );
    }
    return <ExecutionLog log={c.log} />;
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">SA FBB Experience Workbench</div>
        <nav>
          {navItems.map((item) => (
            <button
              key={item.id}
              type="button"
              className={`nav-item ${activeSection === item.id ? 'is-active' : ''}`}
              onClick={() => setActiveSection(item.id)}
            >
              <span>{item.label}</span>
              <small>{item.hint}</small>
            </button>
          ))}
        </nav>
      </aside>
      <section className="content">
        <WorkbenchHeader />
        <section className="section-shell">
          {renderSection()}
        </section>
      </section>
    </main>
  );
}

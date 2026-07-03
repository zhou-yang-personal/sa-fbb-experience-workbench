import { dashboardMetrics, importBatches, migrationLeadSummary } from './shared/mockData';

const modules = [
  'Import Center',
  'Data Quality',
  'ETL Jobs',
  'Overview',
  'App Category',
  'Experience Quality',
  'Cable vs FTTH',
  'Migration Leads',
];

const pipelineSteps = [
  'CSV file probe',
  'RAW load to MySQL',
  'RAW quality gate',
  'RAW → CLEAN/DWD',
  'CLEAN → DWS',
  'DWS → ADS',
  'Dashboard query',
  'Lead export',
];

function App() {
  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">SA FBB Experience Workbench</div>
        <nav>
          {modules.map((item) => (
            <button key={item} type="button" className="nav-item">
              {item}
            </button>
          ))}
        </nav>
      </aside>

      <section className="content">
        <header className="hero-card">
          <div>
            <p className="eyebrow">Raw First · MySQL in-database cleaning · Cable-to-Fiber</p>
            <h1>SA 家宽应用体验本地分析工作台</h1>
            <p className="hero-text">
              第一阶段聚焦 CSV 原样入库、RAW 质量门禁、库内清洗、聚合看板和迁转升套机会识别。
              前端只查询 DWS / ADS，不直接扫 RAW 大表。
            </p>
          </div>
          <div className="version-card">
            <span>Version</span>
            <strong>0.1.0</strong>
          </div>
        </header>

        <section className="metric-grid">
          {dashboardMetrics.map((metric) => (
            <article key={metric.label} className="metric-card">
              <span>{metric.label}</span>
              <strong>{metric.value}</strong>
              <small>{metric.hint}</small>
            </article>
          ))}
        </section>

        <section className="two-column">
          <article className="panel">
            <h2>工程落地链路</h2>
            <ol className="pipeline-list">
              {pipelineSteps.map((step) => (
                <li key={step}>{step}</li>
              ))}
            </ol>
          </article>

          <article className="panel">
            <h2>导入批次</h2>
            <div className="table-like">
              {importBatches.map((batch) => (
                <div key={batch.importBatchId} className="table-row">
                  <span>{batch.dataType.toUpperCase()}</span>
                  <span>{batch.sourceFileName}</span>
                  <span>{batch.status}</span>
                </div>
              ))}
            </div>
          </article>
        </section>

        <section className="panel">
          <h2>迁转升套机会分层</h2>
          <div className="lead-grid">
            {migrationLeadSummary.map((lead) => (
              <article key={lead.leadType} className="lead-card">
                <strong>{lead.leadType}</strong>
                <span>{lead.recommendedAction}</span>
                <small>
                  users={lead.userCount}, demand={lead.avgDemandScore}, motive={lead.avgMigrationMotiveScore}
                </small>
              </article>
            ))}
          </div>
        </section>
      </section>
    </main>
  );
}

export default App;

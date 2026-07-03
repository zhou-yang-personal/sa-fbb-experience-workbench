import { useMemo, useState } from 'react';
import { api } from './shared/tauriApi';
import type { DashboardOverview, ImportBatchResult, LeadUserRow, MetricCard, MySqlSettings } from './shared/types';

const defaultSettings: MySqlSettings = {
  host: '127.0.0.1',
  port: 3306,
  database: 'sa_vbp',
  user: 'root',
  secret: '',
  local_infile: true,
};

const pipelineSteps = [
  '1. 本地工程骨架',
  '2. MySQL 连接与 migration',
  '3. CSV probe 与 import batch',
  '4. TCP/Game RAW 入库',
  '5. RAW 质量门禁',
  '6. RAW → CLEAN → DWS → ADS',
  '7. 看板 / Lead / 导出',
  '8. 稳定化与打包验证入口',
];

function App() {
  const [settings, setSettings] = useState<MySqlSettings>(defaultSettings);
  const [dataType, setDataType] = useState<'tcp' | 'game'>('tcp');
  const [filePath, setFilePath] = useState('');
  const [importBatchId, setImportBatchId] = useState('');
  const [analysisRunId, setAnalysisRunId] = useState('RUN_MANUAL_001');
  const [outputPath, setOutputPath] = useState('leads_export.csv');
  const [log, setLog] = useState<string[]>([]);
  const [batch, setBatch] = useState<ImportBatchResult | null>(null);
  const [qualityMetrics, setQualityMetrics] = useState<MetricCard[]>([]);
  const [overview, setOverview] = useState<DashboardOverview | null>(null);
  const [leads, setLeads] = useState<LeadUserRow[]>([]);

  const allMetrics = useMemo(() => overview?.metrics ?? qualityMetrics, [overview, qualityMetrics]);

  function appendLog(message: string) {
    setLog((items) => [`${new Date().toLocaleTimeString()} ${message}`, ...items].slice(0, 30));
  }

  async function runAction(label: string, action: () => Promise<unknown>) {
    try {
      const result = await action();
      appendLog(`${label}: ${JSON.stringify(result)}`);
      return result;
    } catch (error) {
      appendLog(`${label} failed: ${error instanceof Error ? error.message : String(error)}`);
      return null;
    }
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">SA FBB Experience Workbench</div>
        <nav>
          {['Settings', 'Import', 'Quality', 'ETL', 'Dashboard', 'Leads', 'Export'].map((item) => (
            <button key={item} type="button" className="nav-item">{item}</button>
          ))}
        </nav>
      </aside>

      <section className="content">
        <header className="hero-card">
          <div>
            <p className="eyebrow">Raw First · MySQL in-database cleaning · Cable-to-Fiber</p>
            <h1>SA 家宽应用体验本地分析工作台</h1>
            <p className="hero-text">已按 1-8 步把初步可用链路落地到 dev：连接、初始化、批次、入库、质量检查、ETL、看板、导出。</p>
          </div>
          <div className="version-card"><span>Version</span><strong>0.2.1</strong></div>
        </header>

        <section className="panel form-panel">
          <h2>MySQL 连接与初始化</h2>
          <div className="form-grid">
            <input value={settings.host} onChange={(e) => setSettings({ ...settings, host: e.target.value })} placeholder="host" />
            <input value={settings.port} onChange={(e) => setSettings({ ...settings, port: Number(e.target.value) })} placeholder="port" />
            <input value={settings.database} onChange={(e) => setSettings({ ...settings, database: e.target.value })} placeholder="database" />
            <input value={settings.user} onChange={(e) => setSettings({ ...settings, user: e.target.value })} placeholder="user" />
            <input type="password" value={settings.secret} onChange={(e) => setSettings({ ...settings, secret: e.target.value })} placeholder="password" />
          </div>
          <div className="action-row">
            <button onClick={() => runAction('db_test_connection', () => api.dbTestConnection(settings))}>测试连接</button>
            <button onClick={() => runAction('db_initialize', () => api.dbInitialize(settings))}>初始化数据库</button>
          </div>
        </section>

        <section className="two-column">
          <article className="panel form-panel">
            <h2>CSV 导入</h2>
            <select value={dataType} onChange={(e) => setDataType(e.target.value as 'tcp' | 'game')}><option value="tcp">TCP</option><option value="game">Game</option></select>
            <input value={filePath} onChange={(e) => setFilePath(e.target.value)} placeholder="CSV absolute path" />
            <div className="action-row">
              <button onClick={() => runAction('import_probe_csv', () => api.importProbeCsv(filePath))}>Probe</button>
              <button onClick={async () => {
                const result = await runAction('import_create_batch', () => api.importCreateBatch(settings, dataType, filePath));
                if (result && typeof result === 'object' && 'import_batch_id' in result) {
                  const next = result as ImportBatchResult;
                  setBatch(next);
                  setImportBatchId(next.import_batch_id);
                }
              }}>创建批次</button>
              <button onClick={() => runAction('import_start_raw_load', () => api.importStartRawLoad(settings, importBatchId, dataType, filePath))}>RAW 入库</button>
            </div>
            <small>{batch ? `current batch: ${batch.import_batch_id}` : 'no batch created'}</small>
          </article>

          <article className="panel">
            <h2>工程落地链路</h2>
            <ol className="pipeline-list">{pipelineSteps.map((step) => <li key={step}>{step}</li>)}</ol>
          </article>
        </section>

        <section className="panel form-panel">
          <h2>质量检查 / ETL / 看板 / 导出</h2>
          <div className="form-grid">
            <input value={importBatchId} onChange={(e) => setImportBatchId(e.target.value)} placeholder="import_batch_id" />
            <input value={analysisRunId} onChange={(e) => setAnalysisRunId(e.target.value)} placeholder="analysis_run_id" />
            <input value={outputPath} onChange={(e) => setOutputPath(e.target.value)} placeholder="export path" />
          </div>
          <div className="action-row">
            <button onClick={async () => {
              const result = await runAction('quality_get_batch_report', () => api.qualityGetBatchReport(settings, importBatchId));
              if (Array.isArray(result)) setQualityMetrics(result as MetricCard[]);
            }}>质量检查</button>
            <button onClick={() => runAction('etl_start_clean_job', () => api.etlStartCleanJob(settings, importBatchId))}>RAW → CLEAN</button>
            <button onClick={() => runAction('etl_start_aggregate_job', () => api.etlStartAggregateJob(settings, importBatchId, analysisRunId))}>DWS / ADS</button>
            <button onClick={async () => {
              const result = await runAction('dashboard_get_overview', () => api.dashboardGetOverview(settings, importBatchId, analysisRunId));
              if (result && typeof result === 'object' && 'metrics' in result) setOverview(result as DashboardOverview);
            }}>刷新看板</button>
            <button onClick={async () => {
              const result = await runAction('leads_query_users', () => api.leadsQueryUsers(settings, analysisRunId));
              if (Array.isArray(result)) setLeads(result as LeadUserRow[]);
            }}>查询 Lead</button>
            <button onClick={() => runAction('export_leads_csv', () => api.exportLeadsCsv(settings, analysisRunId, outputPath))}>导出</button>
          </div>
        </section>

        <section className="metric-grid">
          {allMetrics.map((metric) => <article key={metric.label} className="metric-card"><span>{metric.label}</span><strong>{metric.value}</strong><small>{metric.hint}</small></article>)}
        </section>

        <section className="two-column">
          <article className="panel"><h2>Lead 用户</h2><div className="table-like">{leads.map((lead) => <div key={lead.user_key} className="table-row lead-row"><span>{lead.demand_score}/{lead.migration_motive_score}</span><span>{lead.user_key}</span><span>{lead.lead_type}</span></div>)}</div></article>
          <article className="panel"><h2>执行日志</h2><div className="log-list">{log.map((line) => <pre key={line}>{line}</pre>)}</div></article>
        </section>
      </section>
    </main>
  );
}

export default App;

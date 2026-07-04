import { useMemo, useState } from 'react';
import { api } from '../../shared/tauriApi';
import { phaseApi } from '../../shared/phaseApi';
import type { DashboardOverview, FinalLeadUserRow, ImportBatchResult, LeadUserRow, MetricCard, MySqlSettings } from '../../shared/types';
import { ExecutionLog } from './ExecutionLog';
import { MetricGrid } from './MetricGrid';
import { ResultTables } from './ResultTables';

type ImportDataType = 'tcp' | 'game' | 'crm' | 'coverage' | 'reachability';

const defaultSettings: MySqlSettings = { host: '127.0.0.1', port: 3306, database: 'sa_vbp', user: 'root', secret: '', local_infile: true };
const pipelineSteps = ['Import', 'Quality Gate', 'RAW to CLEAN', 'DWS/ADS', 'Final Fusion', 'Export'];

export function WorkbenchApp() {
  const [settings, setSettings] = useState<MySqlSettings>(defaultSettings);
  const [dataType, setDataType] = useState<ImportDataType>('tcp');
  const [importMode, setImportMode] = useState<'load_data' | 'streaming_insert'>('load_data');
  const [filePath, setFilePath] = useState('');
  const [importBatchId, setImportBatchId] = useState('');
  const [analysisRunId, setAnalysisRunId] = useState('RUN_MANUAL_001');
  const [outputPath, setOutputPath] = useState('leads_export.csv');
  const [log, setLog] = useState<string[]>([]);
  const [batch, setBatch] = useState<ImportBatchResult | null>(null);
  const [metrics, setMetrics] = useState<MetricCard[]>([]);
  const [overview, setOverview] = useState<DashboardOverview | null>(null);
  const [leads, setLeads] = useState<LeadUserRow[]>([]);
  const [finalLeads, setFinalLeads] = useState<FinalLeadUserRow[]>([]);
  const effectiveSettings = { ...settings, local_infile: importMode === 'load_data' };
  const allMetrics = useMemo(() => overview?.metrics ?? metrics, [overview, metrics]);

  function appendLog(message: string) { setLog((items) => [`${new Date().toLocaleTimeString()} ${message}`, ...items].slice(0, 60)); }
  async function runAction(label: string, action: () => Promise<unknown>) {
    try { const result = await action(); appendLog(`${label}: ${JSON.stringify(result)}`); return result; }
    catch (error) { appendLog(`${label} failed: ${error instanceof Error ? error.message : String(error)}`); return null; }
  }
  async function loadMetrics(label: string, action: () => Promise<MetricCard[]>) {
    const result = await runAction(label, action);
    if (Array.isArray(result)) setMetrics(result as MetricCard[]);
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">SA FBB Experience Workbench</div>
        <nav>{['Settings', 'Import', 'Mapping', 'Quality', 'ETL', 'Dashboard', 'Leads', 'Final Leads'].map((item) => <button key={item} type="button" className="nav-item">{item}</button>)}</nav>
      </aside>
      <section className="content">
        <header className="hero-card">
          <div><p className="eyebrow">Raw First · MySQL in-database cleaning · Cable-to-Fiber</p><h1>SA 家宽应用体验本地分析工作台</h1><p className="hero-text">导入、字段映射、质量门禁、ETL、ADS 看板、Final Lead 和导出闭环。</p></div>
          <div className="version-card"><span>Version</span><strong>1.0.6</strong></div>
        </header>

        <section className="panel form-panel">
          <h2>MySQL 连接与配置初始化</h2>
          <div className="form-grid">
            <input value={settings.host} onChange={(e) => setSettings({ ...settings, host: e.target.value })} placeholder="host" />
            <input value={settings.port} onChange={(e) => setSettings({ ...settings, port: Number(e.target.value) })} placeholder="port" />
            <input value={settings.database} onChange={(e) => setSettings({ ...settings, database: e.target.value })} placeholder="database" />
            <input value={settings.user} onChange={(e) => setSettings({ ...settings, user: e.target.value })} placeholder="user" />
            <input type="password" value={settings.secret} onChange={(e) => setSettings({ ...settings, secret: e.target.value })} placeholder="password" />
          </div>
          <div className="action-row"><button onClick={() => runAction('db_test_connection', () => api.dbTestConnection(settings))}>测试连接</button><button onClick={() => runAction('db_initialize', () => api.dbInitialize(settings))}>初始化数据库</button><button onClick={() => runAction('config_seed_defaults', () => api.configSeedDefaults(settings))}>初始化映射配置</button></div>
        </section>

        <section className="two-column">
          <article className="panel form-panel">
            <h2>CSV 导入中心</h2>
            <select value={dataType} onChange={(e) => setDataType(e.target.value as ImportDataType)}><option value="tcp">TCP</option><option value="game">Game</option><option value="crm">CRM Users</option><option value="coverage">FTTH Coverage</option><option value="reachability">Reachability</option></select>
            <select value={importMode} onChange={(e) => setImportMode(e.target.value as 'load_data' | 'streaming_insert')}><option value="load_data">LOAD DATA LOCAL INFILE</option><option value="streaming_insert">Streaming INSERT fallback</option></select>
            <input value={filePath} onChange={(e) => setFilePath(e.target.value)} placeholder="CSV absolute path" />
            <div className="action-row"><button onClick={() => runAction('import_probe_csv', () => api.importProbeCsv(filePath))}>Probe</button><button onClick={async () => { const result = await runAction('import_create_batch', () => api.importCreateBatch(effectiveSettings, dataType, filePath)); if (result && typeof result === 'object' && 'import_batch_id' in result) { const next = result as ImportBatchResult; setBatch(next); setImportBatchId(next.import_batch_id); } }}>创建批次</button><button onClick={() => runAction('import_start_raw_load', () => api.importStartRawLoad(effectiveSettings, importBatchId, dataType, filePath, importMode))}>RAW 入库</button><button onClick={() => loadMetrics('import_get_batch_status', () => api.importGetBatchStatus(settings, importBatchId))}>刷新导入状态</button><button onClick={() => loadMetrics('config_get_import_mappings', () => api.configGetImportMappings(settings, dataType))}>字段映射</button></div>
            <small>{batch ? `current batch: ${batch.import_batch_id}` : 'no batch created'}</small>
          </article>
          <article className="panel"><h2>Pipeline</h2><ol className="pipeline-list">{pipelineSteps.map((step) => <li key={step}>{step}</li>)}</ol></article>
        </section>

        <section className="panel form-panel">
          <h2>任务 / 看板 / Lead / 导出</h2>
          <div className="form-grid"><input value={importBatchId} onChange={(e) => setImportBatchId(e.target.value)} placeholder="import_batch_id" /><input value={analysisRunId} onChange={(e) => setAnalysisRunId(e.target.value)} placeholder="analysis_run_id" /><input value={outputPath} onChange={(e) => setOutputPath(e.target.value)} placeholder="export path" /></div>
          <div className="action-row"><button onClick={() => loadMetrics('quality_get_batch_report', () => api.qualityGetBatchReport(settings, importBatchId))}>基础质量</button><button onClick={() => runAction('quality_run_gate', () => phaseApi.qualityRunGate(settings, importBatchId))}>完整质量门禁</button><button onClick={() => loadMetrics('etl_get_recent_jobs', () => api.etlGetRecentJobs(settings, importBatchId))}>ETL状态</button><button onClick={() => runAction('etl_start_clean_job', () => api.etlStartCleanJob(settings, importBatchId))}>RAW→CLEAN</button><button onClick={() => runAction('etl_start_aggregate_job', () => api.etlStartAggregateJob(settings, importBatchId, analysisRunId))}>DWS/ADS</button><button onClick={() => runAction('leads_run_final_fusion', () => phaseApi.leadsRunFinalFusion(settings, importBatchId, analysisRunId))}>配置化融合</button><button onClick={() => loadMetrics('config_get_join_rules', () => api.configGetJoinRules(settings))}>Join规则</button><button onClick={async () => { const result = await runAction('dashboard_get_overview', () => api.dashboardGetOverview(settings, importBatchId, analysisRunId)); if (result && typeof result === 'object' && 'metrics' in result) setOverview(result as DashboardOverview); }}>Overview</button><button onClick={() => loadMetrics('app_category', () => phaseApi.dashboardGetAppCategory(settings, importBatchId, analysisRunId))}>应用看板</button><button onClick={() => loadMetrics('experience_quality', () => phaseApi.dashboardGetExperienceQuality(settings, importBatchId, analysisRunId))}>体验看板</button><button onClick={() => loadMetrics('cable_fiber_compare', () => phaseApi.dashboardGetCableFiberCompare(settings, importBatchId, analysisRunId))}>Cable/FTTH</button><button onClick={async () => { const result = await runAction('leads_query_users', () => api.leadsQueryUsers(settings, analysisRunId)); if (Array.isArray(result)) setLeads(result as LeadUserRow[]); }}>SA Lead</button><button onClick={async () => { const result = await runAction('final_leads_query_users', () => api.finalLeadsQueryUsers(settings, analysisRunId)); if (Array.isArray(result)) setFinalLeads(result as FinalLeadUserRow[]); }}>Final Lead</button><button onClick={() => runAction('export_leads_csv', () => api.exportLeadsCsv(settings, analysisRunId, outputPath))}>导出SA</button><button onClick={() => runAction('export_final_leads_csv', () => api.exportFinalLeadsCsv(settings, analysisRunId, outputPath))}>导出Final</button></div>
        </section>

        <MetricGrid metrics={allMetrics} />
        <ResultTables leads={leads} finalLeads={finalLeads} />
        <ExecutionLog log={log} />
      </section>
    </main>
  );
}

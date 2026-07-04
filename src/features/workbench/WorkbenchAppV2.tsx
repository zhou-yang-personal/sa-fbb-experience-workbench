import { useMemo, useState } from 'react';
import type { PipelineStepStatus } from '../../shared/types';
import { ConnectionPanel } from './ConnectionPanel';
import { DashboardCenter } from './DashboardCenter';
import { EtlJobCenter } from './EtlJobCenter';
import { FinalLeadCenter } from './FinalLeadCenter';
import { ImportPanel } from './ImportPanel';
import { MetricGrid } from './MetricGrid';
import { NextActionHint } from './NextActionHint';
import { PipelineStatusBar, type PipelineStep, type PipelineStepId } from './PipelineStatusBar';
import { ResultTables } from './ResultTables';
import { RunLogDrawer } from './RunLogDrawer';
import { WorkbenchContextBar } from './WorkbenchContextBar';
import { WorkbenchHeader } from './WorkbenchHeader';
import { useWorkbenchController } from './useWorkbenchController';
import './extra.css';

type WorkbenchSection = PipelineStepId;

function hasSuccess(state?: { status?: string }) {
  return state?.status === 'success';
}

function hasFailure(state?: { status?: string }) {
  return state?.status === 'failure';
}

function hasRunning(states: Record<string, { status?: string }>, keys: string[]) {
  return keys.some((key) => states[key]?.status === 'running');
}

function stepStatus(done: boolean, running: boolean, failed: boolean): PipelineStepStatus {
  if (running) return 'running';
  if (failed) return 'failed';
  if (done) return 'success';
  return 'not_started';
}

export function WorkbenchAppV2() {
  const c = useWorkbenchController();
  const [activeSection, setActiveSection] = useState<WorkbenchSection>('start');
  const [logOpen, setLogOpen] = useState(false);

  const pipeline = useMemo<PipelineStep[]>(() => {
    const startDone = hasSuccess(c.actionStates.db_initialize) || hasSuccess(c.actionStates.config_seed_defaults) || hasSuccess(c.actionStates.db_test_connection);
    const importDone = Boolean(c.batch || c.importBatchId || hasSuccess(c.actionStates.import_current_file) || hasSuccess(c.actionStates.import_start_raw_load));
    const validateDone = hasSuccess(c.actionStates.quality_run_gate) || hasSuccess(c.actionStates.quality_get_gate_results);
    const analyzeDone = hasSuccess(c.actionStates.analyze_generate_results) || hasSuccess(c.actionStates.leads_run_final_fusion) || c.dashboardCharts.length > 0;
    const resultsDone = c.dashboardCharts.length > 0 || c.leads.length > 0 || c.finalLeads.length > 0;
    return [
      { id: 'start', label: 'Start', hint: 'DB ready', status: stepStatus(startDone, hasRunning(c.actionStates, ['db_test_connection', 'db_initialize', 'config_seed_defaults', 'start_prepare_database']), hasFailure(c.actionStates.start_prepare_database)) },
      { id: 'import', label: 'Import', hint: 'CSV to RAW', status: stepStatus(importDone, hasRunning(c.actionStates, ['import_current_file', 'import_start_raw_load']), hasFailure(c.actionStates.import_current_file)) },
      { id: 'validate', label: 'Validate', hint: 'Quality gate', status: stepStatus(validateDone, hasRunning(c.actionStates, ['quality_run_gate', 'quality_get_gate_results']), hasFailure(c.actionStates.quality_run_gate)) },
      { id: 'analyze', label: 'Analyze', hint: 'DWS / ADS / Leads', status: stepStatus(analyzeDone, hasRunning(c.actionStates, ['analyze_generate_results', 'etl_start_clean_job', 'etl_start_aggregate_job', 'leads_run_final_fusion']), hasFailure(c.actionStates.analyze_generate_results)) },
      { id: 'results', label: 'Results', hint: 'Review and export', status: stepStatus(resultsDone, hasRunning(c.actionStates, ['dashboard_refresh_all', 'export_final_leads_csv', 'export_leads_csv']), hasFailure(c.actionStates.export_final_leads_csv)) },
    ];
  }, [c.actionStates, c.batch, c.dashboardCharts.length, c.finalLeads.length, c.importBatchId, c.leads.length]);

  const hint = useMemo(() => {
    if (activeSection === 'start') return { title: '测试并初始化数据库', detail: '连接成功后进入 Import。密码不会保存到本地。' };
    if (activeSection === 'import') return { title: c.filePath ? '导入当前 CSV 文件' : '先选择 CSV 文件', detail: c.filePath ? '系统会自动执行 probe、创建批次、映射校验、RAW 入库和画像刷新。' : '使用系统弹框选择文件，不要手输路径。', tone: c.filePath ? 'normal' as const : 'warning' as const };
    if (activeSection === 'validate') return { title: c.importBatchId ? '运行质量检查' : '先完成导入', detail: c.importBatchId ? '检查 RAW 完整性、字段可用性、时间范围、接入类型和应用有效性。' : '没有 import_batch_id 时不能做质量检查。', tone: c.importBatchId ? 'normal' as const : 'blocked' as const };
    if (activeSection === 'analyze') return { title: c.importBatchId ? '生成分析结果' : '先完成质量检查', detail: '系统会依次执行 RAW→CLEAN、DWS/ADS 和 Final Lead Fusion。' };
    return { title: c.analysisRunId ? '刷新结果并导出' : '先生成 analysis_run_id', detail: '在一个页面查看 Dashboard、Lead、Final Lead，并用保存弹框选择导出位置。' };
  }, [activeSection, c.analysisRunId, c.filePath, c.importBatchId]);

  function renderSection() {
    if (activeSection === 'start') {
      return <ConnectionPanel settings={c.settings} setSettings={c.setSettings} runAction={c.runAction} clearPersistedContext={c.clearPersistedContext} actionStates={c.actionStates} />;
    }
    if (activeSection === 'import') {
      return <ImportPanel {...c} />;
    }
    if (activeSection === 'validate') {
      return (
        <section className="workbench-section-stack">
          <QualityCenterBridge controller={c} />
          <MetricGrid metrics={c.allMetrics} />
        </section>
      );
    }
    if (activeSection === 'analyze') {
      return <EtlJobCenter {...c} />;
    }
    return (
      <section className="workbench-section-stack">
        <DashboardCenter {...c} />
        <FinalLeadCenter {...c} />
        <MetricGrid metrics={c.allMetrics} />
        <ResultTables leads={c.leads} finalLeads={c.finalLeads} />
      </section>
    );
  }

  return (
    <main className="app-shell guided-shell">
      <aside className="sidebar guided-sidebar">
        <div className="brand">SA FBB Experience Workbench</div>
        <PipelineStatusBar steps={pipeline} activeStep={activeSection} onSelect={setActiveSection} />
        <div className="sidebar-log-card">
          <strong>当前动作</strong>
          <small>{c.currentAction || '无运行中动作'}</small>
          <button type="button" onClick={() => setLogOpen(true)}>查看执行日志</button>
        </div>
      </aside>
      <section className="content">
        <WorkbenchHeader />
        <WorkbenchContextBar
          settings={c.settings}
          dataType={c.dataType}
          importMode={c.importMode}
          filePath={c.filePath}
          importBatchId={c.importBatchId}
          analysisRunId={c.analysisRunId}
          outputPath={c.outputPath}
          batch={c.batch}
        />
        <NextActionHint title={hint.title} detail={hint.detail} tone={hint.tone} />
        <section className="action-feedback-bar">
          <span>{c.lastActionMessage}</span>
          {c.currentAction && <strong>Running: {c.currentAction}</strong>}
        </section>
        <section className="section-shell guided-section-shell">
          {renderSection()}
        </section>
      </section>
      <RunLogDrawer open={logOpen} log={c.log} onClose={() => setLogOpen(false)} />
    </main>
  );
}

function QualityCenterBridge({ controller }: { controller: ReturnType<typeof useWorkbenchController> }) {
  const { QualityCenter } = require('./QualityCenter') as typeof import('./QualityCenter');
  return <QualityCenter {...controller} />;
}

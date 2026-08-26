import { useState } from 'react';
import { AnalysisWorkspace } from './AnalysisWorkspace';
import { ConfigurationPanel } from './ConfigurationPanel';
import { ImportPanel } from './ImportPanel';
import { NextActionHint } from './NextActionHint';
import { RunLogDrawer } from './RunLogDrawer';
import { SystemPanel } from './SystemPanel';
import { WorkbenchContextBar } from './WorkbenchContextBar';
import { WorkbenchHeader } from './WorkbenchHeader';
import { useWorkbenchController } from './useWorkbenchController';
import './extra.css';
import './AnalyticsDashboard.css';
import './ProductShellV3.css';

type WorkbenchSection = 'overview' | 'findings' | 'investigation' | 'investigations' | 'apps' | 'quality' | 'cable' | 'users' | 'leads' | 'import' | 'config' | 'system';

type ProductNavItem = {
  id: WorkbenchSection;
  label: [string, string];
  hint: [string, string];
  group: 'insight' | 'data';
};

const productNav: ProductNavItem[] = [
  { id: 'overview', label: ['体验健康总览', 'Experience overview'], hint: ['状态、发现与数据覆盖', 'Status, findings and coverage'], group: 'insight' },
  { id: 'findings', label: ['自动发现', 'Auto findings'], hint: ['满足规则与样本的问题', 'Rule and sample qualified issues'], group: 'insight' },
  { id: 'investigations', label: ['问题调查', 'Investigations'], hint: ['保存并继续分析路径', 'Save and continue analysis paths'], group: 'insight' },
  { id: 'apps', label: ['应用体验', 'App experience'], hint: ['具体 App、用户与问题证据', 'App, user and issue evidence'], group: 'insight' },
  { id: 'cable', label: ['Cable vs FTTH', 'Cable vs FTTH'], hint: ['同口径接入体验对比', 'Comparable access experience'], group: 'insight' },
  { id: 'quality', label: ['网络 / 路径证据', 'Network / path evidence'], hint: ['仅展示数据支持的定位能力', 'Localization supported by data'], group: 'insight' },
  { id: 'users', label: ['用户洞察', 'User insights'], hint: ['需求、体验与分类证据', 'Demand and experience evidence'], group: 'insight' },
  { id: 'leads', label: ['体验驱动机会', 'Experience opportunities'], hint: ['与体验 Finding 明确分离', 'Separate from experience findings'], group: 'insight' },
  { id: 'import', label: ['数据导入与批次', 'Import & batches'], hint: ['CSV → RAW → 可分析批次', 'CSV to analysis-ready batch'], group: 'data' },
  { id: 'config', label: ['规则与配置', 'Rules & configuration'], hint: ['接入识别与分析策略', 'Access and analysis policies'], group: 'data' },
  { id: 'system', label: ['系统诊断', 'Diagnostics'], hint: ['连接、任务和执行日志', 'Connection, tasks and logs'], group: 'data' },
];

export function WorkbenchAppV2() {
  const c = useWorkbenchController();
  const [activeSection, setActiveSection] = useState<WorkbenchSection>('overview');
  const [logOpen, setLogOpen] = useState(false);

  const hasBatch = Boolean(c.importBatchId.trim());
  const hasBatchName = Boolean(c.batchDisplayName.trim());
  const languageIndex = c.language === 'zh-CN' ? 0 : 1;

  function hint() {
    if (['overview', 'findings', 'investigation', 'investigations', 'apps', 'quality', 'cable', 'users', 'leads'].includes(activeSection)) {
      if (!hasBatch) return { title: '先选择导入批次', detail: '数据分析以 import_batch_id 为边界。可以先去“数据导入”创建批次，或在分析页下拉选择已有批次。', tone: 'warning' as const };
      return { title: '按需加载当前分析页', detail: '应用不会在启动或切换页面时自动执行大查询。确认批次和 analysis_run_id 后，点击“加载当前看板”再开始任务。', tone: 'normal' as const };
    }
    if (activeSection === 'import') {
      if (!hasBatchName) return { title: '导入前先命名批次', detail: '批次名称必须是正常人能读懂的业务名称，后续所有看板先按这个批次进入。', tone: 'warning' as const };
      return { title: c.filePath ? '启动自动导入分析计划' : '先选择 CSV 文件', detail: c.filePath ? '点击“启动导入分析计划”，系统会自动完成 RAW、Quality Gate、CLEAN/DWS/ADS 和 Module Ready。' : '使用系统弹框选择文件，不要手输路径。', tone: c.filePath ? 'normal' as const : 'warning' as const };
    }
    if (activeSection === 'config') {
      return { title: '配置并发布接入识别规则', detail: '维护 Cable / FTTH IPv4 网段，先验证和预览覆盖率，再发布不可变版本并应用到批次。', tone: 'normal' as const };
    }
    return { title: '系统诊断与后台能力', detail: '数据库连接、数据可用性、ETL 任务和执行日志只作为支撑能力，不再占用主分析入口。', tone: 'normal' as const };
  }

  function renderSection() {
    if (['overview', 'findings', 'investigation', 'investigations', 'apps', 'quality', 'cable', 'users', 'leads'].includes(activeSection)) return <AnalysisWorkspace c={c} activeView={activeSection as 'overview' | 'findings' | 'investigation' | 'investigations' | 'apps' | 'quality' | 'cable' | 'users' | 'leads'} onOpenImport={() => setActiveSection('import')} onNavigate={(view) => setActiveSection(view)} />;
    if (activeSection === 'import') return <ImportPanel {...c} onOpenAnalysis={() => setActiveSection('overview')} onOpenAccessRules={() => setActiveSection('config')} />;
    if (activeSection === 'config') return <ConfigurationPanel c={c} />;
    return <SystemPanel c={c} />;
  }

  const nextHint = hint();

  return (
    <main className="app-shell guided-shell product-shell">
      <aside className="sidebar guided-sidebar product-sidebar">
        <div className="brand">SA FBB Experience Workbench</div>
        <nav className="product-nav" aria-label="Product navigation">
          {(['insight', 'data'] as const).map((group) => <div className="nav-group" key={group}>
            <p>{group === 'insight' ? (languageIndex === 0 ? '洞察与调查' : 'Insights & investigation') : (languageIndex === 0 ? '数据与配置' : 'Data & configuration')}</p>
            {productNav.filter((item) => item.group === group).map((item) => (
              <button key={item.id} type="button" className={`nav-item ${activeSection === item.id ? 'is-active' : ''}`} onClick={() => setActiveSection(item.id)}>
                <span>{item.label[languageIndex]}</span>
                <small>{item.hint[languageIndex]}</small>
              </button>
            ))}
          </div>)}
        </nav>
        <div className="sidebar-log-card">
          <strong>诊断日志</strong>
          <small>{c.currentAction || c.lastActionMessage || '无运行中动作'}</small>
          <button type="button" onClick={() => setLogOpen(true)}>查看执行日志</button>
        </div>
      </aside>
      <section className="content">
        <WorkbenchHeader language={c.language} />
        <WorkbenchContextBar
          settings={c.settings}
          dataType={c.dataType}
          importMode={c.importMode}
          filePath={c.filePath}
          importBatchId={c.importBatchId}
          batchDisplayName={c.batchDisplayName}
          analysisRunId={c.analysisRunId}
          outputPath={c.outputPath}
          batch={c.batch}
          language={c.language}
          analysisContext={c.analysisContext}
          canGoBack={c.analysisContextHistory.length > 0}
          onLanguageChange={c.setLanguage}
          onRemoveFilter={c.removeAnalysisContext}
          onClearFilters={c.clearAnalysisContext}
          onBack={c.backAnalysisContext}
        />
        <NextActionHint title={nextHint.title} detail={nextHint.detail} tone={nextHint.tone} />
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

import { useState } from 'react';
import { AnalysisErrorBoundary } from './AnalysisErrorBoundary';
import { AnalysisWorkspace } from './AnalysisWorkspace';
import { DuckDbAnalysisWorkspace } from './DuckDbAnalysisWorkspace';
import { DuckDbWorkspacePanel } from './DuckDbWorkspacePanel';
import { ConfigurationPanel } from './ConfigurationPanel';
import { ImportPanel } from './ImportPanel';
import { RunLogDrawer } from './RunLogDrawer';
import { SystemPanel } from './SystemPanel';
import { WorkbenchContextBar } from './WorkbenchContextBar';
import { useWorkbenchController } from './useWorkbenchController';
import './extra.css';
import './AnalyticsDashboard.css';
import './ProductShellV3.css';
import './DecisionWorkspaceV3.css';

type WorkbenchSection = 'panorama' | 'quality' | 'access' | 'opportunities' | 'data' | 'config' | 'system';

type ProductNavItem = {
  id: WorkbenchSection;
  label: [string, string];
  hint: [string, string];
  group: 'primary' | 'secondary';
};

const productNav: ProductNavItem[] = [
  { id: 'panorama', label: ['全景洞察', 'Panorama'], hint: ['指标 · App', 'Metrics · Apps'], group: 'primary' },
  { id: 'quality', label: ['质差分析', 'Poor quality'], hint: ['规模 → 证据侧 → 下钻', 'Scale → evidence → drilldown'], group: 'primary' },
  { id: 'access', label: ['Cable / FTTH', 'Cable / FTTH'], hint: ['独立接入制式专项', 'Access comparison'], group: 'primary' },
  { id: 'opportunities', label: ['潜客机会', 'Opportunities'], hint: ['迁转 · 升套 · 组网 · Bundle', 'Migration · upgrade · mesh · bundle'], group: 'primary' },
  { id: 'data', label: ['数据中心', 'Data center'], hint: ['导入 · 聚合 · 任务 · 日志', 'Import · jobs · logs'], group: 'primary' },
  { id: 'config', label: ['规则配置', 'Rules'], hint: ['IP Others 与分析阈值', 'IP Others & thresholds'], group: 'secondary' },
  { id: 'system', label: ['系统诊断', 'Diagnostics'], hint: ['连接与专家工具', 'Connection & expert tools'], group: 'secondary' },
];

export function WorkbenchAppV2() {
  const c = useWorkbenchController();
  const [activeSection, setActiveSection] = useState<WorkbenchSection>('panorama');
  const [logOpen, setLogOpen] = useState(false);

  const hasBatch = Boolean(c.importBatchId.trim());
  const hasBatchName = Boolean(c.batchDisplayName.trim());
  const languageIndex = c.language === 'zh-CN' ? 0 : 1;

  function hint() {
    if (c.runtimeEngine === 'duckdb') {
      if (activeSection === 'data') return { title: 'DuckDB 本地数据中心', detail: 'CSV 转换、Parquet 明细与聚合结果均写入所选工作区，不连接 MySQL。', tone: 'normal' as const };
      return { title: 'DuckDB 隔离模式', detail: '仅查询 workspace.duckdb。尚未迁移的模块明确不可用，不会静默回退到 MySQL。', tone: 'normal' as const };
    }
    if (['panorama', 'quality', 'access', 'opportunities'].includes(activeSection)) {
      if (!hasBatch) return { title: '先选择导入批次', detail: '数据分析以 import_batch_id 为边界。可以先去“数据导入”创建批次，或在分析页下拉选择已有批次。', tone: 'warning' as const };
      return { title: '按需加载当前分析页', detail: '应用不会在启动或切换页面时自动执行大查询。确认批次和 analysis_run_id 后，点击“加载当前看板”再开始任务。', tone: 'normal' as const };
    }
    if (activeSection === 'data') {
      if (!hasBatchName) return { title: '导入前先命名批次', detail: '批次名称必须是正常人能读懂的业务名称，后续所有看板先按这个批次进入。', tone: 'warning' as const };
      return { title: c.filePath ? '启动自动导入分析计划' : '先选择 CSV 文件', detail: c.filePath ? '点击“启动导入分析计划”，系统会自动完成 RAW、Quality Gate、CLEAN/DWS/ADS 和 Module Ready。' : '使用系统弹框选择文件，不要手输路径。', tone: c.filePath ? 'normal' as const : 'warning' as const };
    }
    if (activeSection === 'config') {
      return { title: '配置并发布接入识别规则', detail: '维护 Cable / FTTH IPv4 网段，先验证和预览覆盖率，再发布不可变版本并应用到批次。', tone: 'normal' as const };
    }
    return { title: '系统诊断与后台能力', detail: '数据库连接、数据可用性、ETL 任务和执行日志只作为支撑能力，不再占用主分析入口。', tone: 'normal' as const };
  }

  function renderSection() {
    if (c.runtimeEngine === 'duckdb') {
      if (['panorama', 'quality', 'access', 'opportunities'].includes(activeSection)) return <DuckDbAnalysisWorkspace c={c} activeView={activeSection as 'panorama' | 'quality' | 'access' | 'opportunities'} onOpenData={() => setActiveSection('data')} />;
      if (activeSection === 'data') return <DuckDbWorkspacePanel c={c} onOpenAnalysis={() => setActiveSection('access')} />;
      if (activeSection === 'config') return <article className="panel form-panel"><span className="step-badge">DuckDB</span><h2>规则配置迁移中</h2><p className="hero-text">当前可在数据中心输入 FTTH IP 范围；完整版本化规则编辑器尚未迁移。本页不会访问 MySQL。</p></article>;
      return <section className="workbench-section-stack"><article className="panel form-panel"><span className="step-badge">DuckDB</span><h2>本地运行时诊断</h2><p className="hero-text">默认运行时不加载 MySQL 诊断组件。工作区状态可在数据中心检查。</p></article><DuckDbWorkspacePanel c={c} /></section>;
    }
    if (['panorama', 'quality', 'access', 'opportunities'].includes(activeSection)) return <AnalysisWorkspace c={c} activeView={activeSection as 'panorama' | 'quality' | 'access' | 'opportunities'} onOpenImport={() => setActiveSection('data')} />;
    if (activeSection === 'data') return <ImportPanel {...c} onOpenAnalysis={() => setActiveSection('panorama')} onOpenAccessRules={() => setActiveSection('config')} />;
    if (activeSection === 'config') return <ConfigurationPanel c={c} />;
    return <SystemPanel c={c} />;
  }

  const nextHint = hint();

  return (
    <main className="app-shell guided-shell product-shell">
      <aside className="sidebar guided-sidebar product-sidebar">
        <div className="brand"><span>SA FBB Experience Workbench</span><small>v2.0.0-2</small></div>
        <label className="sidebar-language runtime-selector">
          <span>运行时</span>
          <select aria-label="Runtime engine" value={c.runtimeEngine} onChange={(event) => c.setRuntimeEngine(event.target.value as 'duckdb' | 'mysql_compat')}>
            <option value="duckdb">DuckDB（默认）</option>
            <option value="mysql_compat">MySQL（兼容）</option>
          </select>
          <small>{c.runtimeEngine === 'duckdb' ? '本地工作区 · 不查询 MySQL' : '显式兼容模式 · 可查询旧批次'}</small>
        </label>
        <nav className="product-nav" aria-label="Product navigation">
          {(['primary', 'secondary'] as const).map((group) => <div className="nav-group" key={group}>
            <p>{group === 'primary' ? (languageIndex === 0 ? '分析工作台' : 'Workspace') : (languageIndex === 0 ? '设置与支持' : 'Settings & support')}</p>
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
        <label className="sidebar-language">
          <span>{languageIndex === 0 ? '语言' : 'Language'}</span>
          <select aria-label="Language" value={c.language} onChange={(event) => c.setLanguage(event.target.value as 'zh-CN' | 'en-US')}>
            <option value="zh-CN">中文</option>
            <option value="en-US">English</option>
          </select>
        </label>
      </aside>
      <section className="content">
        <WorkbenchContextBar
          language={c.language}
          analysisContext={c.analysisContext}
          canGoBack={c.analysisContextHistory.length > 0}
          onRemoveFilter={c.removeAnalysisContext}
          onClearFilters={c.clearAnalysisContext}
          onBack={c.backAnalysisContext}
        />
        {c.currentAction && <section className="action-feedback-bar"><span>{nextHint.title}</span><strong>Running: {c.currentAction}</strong></section>}
        <section className="section-shell guided-section-shell">
          <AnalysisErrorBoundary language={c.language} resetKey={`${c.runtimeEngine}-${activeSection}`} onReset={() => setActiveSection('panorama')}>
            {renderSection()}
          </AnalysisErrorBoundary>
        </section>
      </section>
      <RunLogDrawer open={logOpen} log={c.log} onClose={() => setLogOpen(false)} />
    </main>
  );
}

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

type WorkbenchSection = 'overview' | 'apps' | 'quality' | 'cable' | 'users' | 'leads' | 'import' | 'config' | 'system';

type ProductNavItem = {
  id: WorkbenchSection;
  label: string;
  hint: string;
  group: '洞察' | '数据与配置';
};

const productNav: ProductNavItem[] = [
  { id: 'overview', label: '经营与体验总览', hint: '风险、机会与数据可信度', group: '洞察' },
  { id: 'apps', label: '应用体验', hint: '具体 App、用户与问题归因', group: '洞察' },
  { id: 'cable', label: 'Cable vs FTTH', hint: '同口径接入体验对比', group: '洞察' },
  { id: 'quality', label: '网络问题定位', hint: 'BRAS / OLT / PON 与问题侧', group: '洞察' },
  { id: 'users', label: '用户洞察', hint: '需求、体验与分类证据', group: '洞察' },
  { id: 'leads', label: '迁转升套机会', hint: '修障排除后的候选用户', group: '洞察' },
  { id: 'import', label: '数据导入与批次', hint: 'CSV → RAW → 可分析批次', group: '数据与配置' },
  { id: 'config', label: '接入识别配置', hint: 'IP 网段与规则版本', group: '数据与配置' },
  { id: 'system', label: '系统诊断', hint: '连接、任务和执行日志', group: '数据与配置' },
];

export function WorkbenchAppV2() {
  const c = useWorkbenchController();
  const [activeSection, setActiveSection] = useState<WorkbenchSection>('overview');
  const [logOpen, setLogOpen] = useState(false);

  const hasBatch = Boolean(c.importBatchId.trim());
  const hasBatchName = Boolean(c.batchDisplayName.trim());

  function hint() {
    if (['overview', 'apps', 'quality', 'cable', 'users', 'leads'].includes(activeSection)) {
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
    if (['overview', 'apps', 'quality', 'cable', 'users', 'leads'].includes(activeSection)) return <AnalysisWorkspace c={c} activeView={activeSection as 'overview' | 'apps' | 'quality' | 'cable' | 'users' | 'leads'} />;
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
          {(['洞察', '数据与配置'] as const).map((group) => <div className="nav-group" key={group}>
            <p>{group}</p>
            {productNav.filter((item) => item.group === group).map((item) => (
              <button key={item.id} type="button" className={`nav-item ${activeSection === item.id ? 'is-active' : ''}`} onClick={() => setActiveSection(item.id)}>
                <span>{item.label}</span>
                <small>{item.hint}</small>
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
        <WorkbenchHeader />
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

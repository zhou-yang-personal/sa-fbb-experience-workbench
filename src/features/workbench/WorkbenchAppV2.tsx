import { useState } from 'react';
import { AnalysisWorkspace } from './AnalysisWorkspace';
import { ImportPanel } from './ImportPanel';
import { NextActionHint } from './NextActionHint';
import { RunLogDrawer } from './RunLogDrawer';
import { SystemPanel } from './SystemPanel';
import { WorkbenchContextBar } from './WorkbenchContextBar';
import { WorkbenchHeader } from './WorkbenchHeader';
import { useWorkbenchController } from './useWorkbenchController';
import './extra.css';

type WorkbenchSection = 'analysis' | 'import' | 'system';

type ProductNavItem = {
  id: WorkbenchSection;
  label: string;
  hint: string;
};

const productNav: ProductNavItem[] = [
  { id: 'analysis', label: '数据分析', hint: '默认入口：先选批次，再看看板' },
  { id: 'import', label: '数据导入', hint: 'CSV → RAW → 可分析批次' },
  { id: 'system', label: '系统管理', hint: '连接、诊断日志、后台任务' },
];

export function WorkbenchAppV2() {
  const c = useWorkbenchController();
  const [activeSection, setActiveSection] = useState<WorkbenchSection>('analysis');
  const [logOpen, setLogOpen] = useState(false);

  const hasBatch = Boolean(c.importBatchId.trim());
  const hasBatchName = Boolean(c.batchDisplayName.trim());

  function hint() {
    if (activeSection === 'analysis') {
      if (!hasBatch) return { title: '先选择导入批次', detail: '数据分析以 import_batch_id 为边界。可以先去“数据导入”创建批次，或在分析页下拉选择已有批次。', tone: 'warning' as const };
      return { title: '查看当前批次看板', detail: '模块会根据批次类型、必填字段和所需聚合表判断是否可用；不可用模块会置灰。', tone: 'normal' as const };
    }
    if (activeSection === 'import') {
      if (!hasBatchName) return { title: '导入前先命名批次', detail: '批次名称必须是正常人能读懂的业务名称，后续所有看板先按这个批次进入。', tone: 'warning' as const };
      return { title: c.filePath ? '导入当前 CSV 文件' : '先选择 CSV 文件', detail: c.filePath ? '系统会自动执行 probe、创建批次、映射校验、RAW 入库和画像刷新。' : '使用系统弹框选择文件，不要手输路径。', tone: c.filePath ? 'normal' as const : 'warning' as const };
    }
    return { title: '系统诊断与后台能力', detail: '数据库连接、数据可用性、ETL 任务和执行日志只作为支撑能力，不再占用主分析入口。', tone: 'normal' as const };
  }

  function renderSection() {
    if (activeSection === 'analysis') return <AnalysisWorkspace c={c} />;
    if (activeSection === 'import') return <ImportPanel {...c} />;
    return <SystemPanel c={c} />;
  }

  const nextHint = hint();

  return (
    <main className="app-shell guided-shell product-shell">
      <aside className="sidebar guided-sidebar product-sidebar">
        <div className="brand">SA FBB Experience Workbench</div>
        <nav className="product-nav" aria-label="Product navigation">
          {productNav.map((item) => (
            <button key={item.id} type="button" className={`nav-item ${activeSection === item.id ? 'is-active' : ''}`} onClick={() => setActiveSection(item.id)}>
              <span>{item.label}</span>
              <small>{item.hint}</small>
            </button>
          ))}
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

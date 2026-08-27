import type { UiLanguage } from '../../shared/i18n';

export function WorkbenchHeader({ language = 'zh-CN' }: { language?: UiLanguage }) {
  const zh = language === 'zh-CN';
  return (
    <header className="hero-card">
      <div>
        <p className="eyebrow">Experience intelligence · Network action · Qualified growth</p>
        <h1>{zh ? 'SA 家宽应用体验本地分析工作台' : 'SA FBB Experience Analysis Workbench'}</h1>
        <p className="hero-text">{zh ? '从体验状态和自动发现进入可持续下钻的问题调查；所有结论可回溯到批次、规则版本和聚合证据。' : 'Move from experience status and findings into persistent drill-down investigations; every conclusion traces to a batch, rule version and aggregate evidence.'}</p>
      </div>
      <div className="version-card"><span>Version</span><strong>1.0.56</strong></div>
    </header>
  );
}

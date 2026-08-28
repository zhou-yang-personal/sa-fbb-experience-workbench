import type { UiLanguage } from '../../shared/i18n';

export function WorkbenchHeader({ language = 'zh-CN' }: { language?: UiLanguage }) {
  const zh = language === 'zh-CN';
  return (
    <header className="hero-card">
      <div>
        <p className="eyebrow">Experience intelligence · Network action · Qualified growth</p>
        <h1>{zh ? 'SA 家宽应用体验本地分析工作台' : 'SA FBB Experience Analysis Workbench'}</h1>
        <p className="hero-text">{zh ? '先看完整业务全景，再进入质差、接入制式和潜客专项；所有结论可回溯到批次、规则版本和聚合证据。' : 'Start with the complete business panorama, then move into quality, access and opportunity analysis; every conclusion traces to a batch, rule version and aggregate evidence.'}</p>
      </div>
      <div className="version-card"><span>Version</span><strong>1.0.60</strong></div>
    </header>
  );
}

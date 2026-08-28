import type { AnalysisContext, AnalysisContextKey } from '../../shared/types';
import type { UiLanguage } from '../../shared/i18n';

type Props = {
  language: UiLanguage;
  analysisContext: AnalysisContext;
  canGoBack: boolean;
  onRemoveFilter: (key: AnalysisContextKey) => void;
  onClearFilters: () => void;
  onBack: () => void;
};

const contextLabels: Record<AnalysisContextKey, { zh: string; en: string }> = {
  data_type: { zh: '数据类型', en: 'Data type' },
  app_category: { zh: '应用类别', en: 'App category' },
  app_name: { zh: '应用', en: 'App' },
  access_type: { zh: '接入制式', en: 'Access' },
  date_from: { zh: '开始日期', en: 'Date from' },
  date_to: { zh: '结束日期', en: 'Date to' },
  hour_from: { zh: '开始小时', en: 'Hour from' },
  hour_to: { zh: '结束小时', en: 'Hour to' },
  issue_metric: { zh: '问题指标', en: 'Issue metric' },
  issue_side: { zh: '问题侧', en: 'Issue side' },
  user_key: { zh: '用户', en: 'User' },
  server_ip: { zh: '服务端 IP', en: 'Server IP' },
  bras: { zh: 'BRAS', en: 'BRAS' },
  network_object: { zh: '网络对象', en: 'Network object' },
  baseline_type: { zh: '基线', en: 'Baseline' },
  finding_id: { zh: '发现', en: 'Finding' },
};

export function WorkbenchContextBar({ language, analysisContext, canGoBack, onRemoveFilter, onClearFilters, onBack }: Props) {
  const entries = Object.entries(analysisContext) as [AnalysisContextKey, AnalysisContext[AnalysisContextKey]][];
  if (!entries.length && !canGoBack) return null;

  return (
    <section className="context-bar-shell" aria-label="Current analysis context">
      <div className="analysis-context-strip">
      <div className="analysis-context-chips">
        <span className="analysis-path-label">{language === 'zh-CN' ? '分析路径' : 'Path'}</span>
        {entries.map(([key, value]) => (
          <button type="button" className="analysis-context-chip" key={key} title={language === 'zh-CN' ? '点击移除此条件' : 'Remove this filter'} onClick={() => onRemoveFilter(key)}>
            <span>{contextLabels[key][language === 'zh-CN' ? 'zh' : 'en']}</span>
            <strong>{String(value)}</strong>
            <b aria-hidden="true">×</b>
          </button>
        ))}
      </div>
      <div className="analysis-context-actions">
        {canGoBack && <button type="button" onClick={onBack}>{language === 'zh-CN' ? '返回' : 'Back'}</button>}
        {entries.length > 0 && <button type="button" onClick={onClearFilters}>{language === 'zh-CN' ? '清空' : 'Clear'}</button>}
      </div>
      </div>
    </section>
  );
}

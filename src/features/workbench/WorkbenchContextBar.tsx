import type { AnalysisContext, AnalysisContextKey, ImportBatchResult, ImportDataType, MySqlSettings } from '../../shared/types';
import type { UiLanguage } from '../../shared/i18n';

type Props = {
  settings: MySqlSettings;
  dataType: ImportDataType;
  importMode: 'load_data' | 'streaming_insert';
  filePath: string;
  importBatchId: string;
  batchDisplayName: string;
  analysisRunId: string;
  outputPath: string;
  batch: ImportBatchResult | null;
  language: UiLanguage;
  analysisContext: AnalysisContext;
  canGoBack: boolean;
  onLanguageChange: (language: UiLanguage) => void;
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

function shortValue(value: string, fallback = '-') {
  if (!value.trim()) return fallback;
  return value.length > 46 ? `${value.slice(0, 18)}…${value.slice(-22)}` : value;
}

function dbLabel(settings: MySqlSettings) {
  const host = settings.host || 'host?';
  const database = settings.database || 'database?';
  const port = settings.port || 3306;
  return `${host}:${port}/${database}`;
}

export function WorkbenchContextBar({ settings, dataType, importMode, filePath, importBatchId, batchDisplayName, analysisRunId, outputPath, batch, language, analysisContext, canGoBack, onLanguageChange, onRemoveFilter, onClearFilters, onBack }: Props) {
  const batchStatus = batch?.status ?? (importBatchId ? 'manual selected' : 'not selected');
  const displayName = batchDisplayName || batch?.batch_display_name || batch?.source_file_name || '';
  return (
    <section className="context-bar-shell" aria-label="Current analysis context">
    <div className="context-bar">
      <div className="context-item context-primary">
        <span>Batch</span>
        <strong title={displayName}>{shortValue(displayName, 'no batch name')}</strong>
        <small>{shortValue(importBatchId, batchStatus)}</small>
      </div>
      <div className="context-item">
        <span>Run</span>
        <strong>{shortValue(analysisRunId, 'no run')}</strong>
        <small>analysis_run_id</small>
      </div>
      <div className="context-item">
        <span>Data</span>
        <strong>{dataType.toUpperCase()}</strong>
        <small>{importMode === 'load_data' ? 'LOAD DATA' : 'Streaming INSERT'}</small>
      </div>
      <div className="context-item context-wide">
        <span>Source</span>
        <strong title={filePath}>{shortValue(filePath, 'no file selected')}</strong>
        <small>{dbLabel(settings)}</small>
      </div>
      <div className="context-item context-wide">
        <span>Export target</span>
        <strong title={outputPath}>{shortValue(outputPath, 'no output path')}</strong>
        <small>看板内按钮直接导出</small>
      </div>
    </div>
    <div className="analysis-context-strip">
      <div className="analysis-context-label">
        <strong>{language === 'zh-CN' ? '分析路径' : 'Analysis path'}</strong>
        <small>{language === 'zh-CN' ? '切换页面时保留以下条件' : 'These filters persist across pages'}</small>
      </div>
      <div className="analysis-context-chips">
        {(Object.entries(analysisContext) as [AnalysisContextKey, AnalysisContext[AnalysisContextKey]][]).map(([key, value]) => (
          <button type="button" className="analysis-context-chip" key={key} title={language === 'zh-CN' ? '点击移除此条件' : 'Remove this filter'} onClick={() => onRemoveFilter(key)}>
            <span>{contextLabels[key][language === 'zh-CN' ? 'zh' : 'en']}</span>
            <strong>{String(value)}</strong>
            <b aria-hidden="true">×</b>
          </button>
        ))}
        {!Object.keys(analysisContext).length && <span className="analysis-context-empty">{language === 'zh-CN' ? '当前为全局范围；点击 Finding 或图表可逐层追加条件。' : 'Global scope; click a finding or chart to add filters.'}</span>}
      </div>
      <div className="analysis-context-actions">
        <button type="button" disabled={!canGoBack} onClick={onBack}>{language === 'zh-CN' ? '返回上层' : 'Back'}</button>
        <button type="button" disabled={!Object.keys(analysisContext).length} onClick={onClearFilters}>{language === 'zh-CN' ? '清空条件' : 'Clear'}</button>
        <select aria-label="Language" value={language} onChange={(event) => onLanguageChange(event.target.value as UiLanguage)}>
          <option value="zh-CN">中文</option>
          <option value="en-US">English</option>
        </select>
      </div>
    </div>
    </section>
  );
}

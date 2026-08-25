import type { LocalizedText, UiLanguage } from '../../shared/i18n';
import { DEFAULT_UI_LANGUAGE, localized } from '../../shared/i18n';
import './ChartExplanation.css';

export type ChartExplanationContent = {
  question: LocalizedText;
  calculation: LocalizedText;
  interpretation: LocalizedText;
  limitation: LocalizedText;
};

type ChartExplanationProps = ChartExplanationContent & {
  language?: UiLanguage;
};

const labels: Record<UiLanguage, Record<'question' | 'calculation' | 'interpretation' | 'limitation', string>> = {
  'zh-CN': {
    question: '回答的问题',
    calculation: '计算口径',
    interpretation: '如何解读',
    limitation: '数据限制',
  },
  'en-US': {
    question: 'Question',
    calculation: 'Calculation',
    interpretation: 'How to read',
    limitation: 'Data limitation',
  },
};

/**
 * Shared, bilingual explanation block for charts. New dashboards can provide
 * metric-specific definitions without coupling copy to ECharts rendering.
 */
export function ChartExplanation({
  question,
  calculation,
  interpretation,
  limitation,
  language = DEFAULT_UI_LANGUAGE,
}: ChartExplanationProps) {
  const copy = labels[language];
  return (
    <dl className="chart-explanation">
      <div><dt>{copy.question}</dt><dd>{localized(question, language)}</dd></div>
      <div><dt>{copy.calculation}</dt><dd>{localized(calculation, language)}</dd></div>
      <div><dt>{copy.interpretation}</dt><dd>{localized(interpretation, language)}</dd></div>
      <div><dt>{copy.limitation}</dt><dd>{localized(limitation, language)}</dd></div>
    </dl>
  );
}

export type UiLanguage = 'zh-CN' | 'en-US';

export type LocalizedText = {
  'zh-CN': string;
  'en-US': string;
};

export const DEFAULT_UI_LANGUAGE: UiLanguage = 'zh-CN';

export function localized(text: LocalizedText, language: UiLanguage = DEFAULT_UI_LANGUAGE) {
  return text[language];
}

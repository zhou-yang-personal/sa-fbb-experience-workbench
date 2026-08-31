import { Component, type ErrorInfo, type ReactNode } from 'react';
import type { UiLanguage } from '../../shared/i18n';

type Props = {
  children: ReactNode;
  language: UiLanguage;
  resetKey: string;
  onReset: () => void;
};

type State = { error: Error | null };

export class AnalysisErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('analysis_workspace_render_failed', error, info.componentStack);
  }

  componentDidUpdate(previous: Props) {
    if (previous.resetKey !== this.props.resetKey && this.state.error) {
      this.setState({ error: null });
    }
  }

  render() {
    if (!this.state.error) return this.props.children;
    const zh = this.props.language === 'zh-CN';
    return <section className="analysis-error-boundary" role="alert">
      <span>{zh ? '页面渲染已安全隔离' : 'Page rendering safely isolated'}</span>
      <h2>{zh ? '调查页面遇到不兼容的数据，没有退出应用。' : 'The investigation page received incompatible data; the app remains open.'}</h2>
      <p>{this.state.error.message || (zh ? '未知前端渲染错误' : 'Unknown rendering error')}</p>
      <button type="button" className="primary-button" onClick={this.props.onReset}>{zh ? '返回体验总览' : 'Return to overview'}</button>
    </section>;
  }
}

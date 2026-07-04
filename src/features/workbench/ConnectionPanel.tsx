import type { ActionState, MySqlSettings } from '../../shared/types';
import { ActionButton } from './ActionButton';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  setSettings: (settings: MySqlSettings) => void;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  clearPersistedContext: () => void;
  actionStates: Record<string, ActionState>;
};

export function ConnectionPanel({ settings, setSettings, runAction, clearPersistedContext, actionStates }: Props) {
  async function prepareDatabase() {
    await runAction('start_prepare_database', async () => {
      await workbenchApi.testDb(settings);
      await workbenchApi.initDb(settings);
      await workbenchApi.seedConfig(settings);
      return { status: 'ready', database: settings.database };
    });
  }

  return (
    <section className="workbench-section-stack">
      <section className="panel form-panel step-card">
        <div className="step-card-head">
          <div>
            <h2>Start：数据库连接与初始化</h2>
            <p className="hero-text">第一步只做一件事：让 MySQL schema 和默认映射配置就绪。</p>
          </div>
          <span className="step-badge">1 / 5</span>
        </div>
        <div className="form-grid">
          <input value={settings.host} onChange={(e) => setSettings({ ...settings, host: e.target.value })} placeholder="host" />
          <input value={settings.port} onChange={(e) => setSettings({ ...settings, port: Number(e.target.value) })} placeholder="port" />
          <input value={settings.database} onChange={(e) => setSettings({ ...settings, database: e.target.value })} placeholder="database" />
          <input value={settings.user} onChange={(e) => setSettings({ ...settings, user: e.target.value })} placeholder="user" />
          <input type="password" value={settings.secret} onChange={(e) => setSettings({ ...settings, secret: e.target.value })} placeholder="password" />
        </div>
        <div className="primary-action-row">
          <ActionButton actionKey="start_prepare_database" actionStates={actionStates} primary label="测试并初始化数据库" onClick={prepareDatabase} />
        </div>
        <details className="advanced-actions">
          <summary>高级操作</summary>
          <div className="action-row">
            <ActionButton actionKey="db_test_connection" actionStates={actionStates} label="仅测试连接" onClick={() => runAction('db_test_connection', () => workbenchApi.testDb(settings))} />
            <ActionButton actionKey="db_initialize" actionStates={actionStates} label="仅初始化数据库" onClick={() => runAction('db_initialize', () => workbenchApi.initDb(settings))} />
            <ActionButton actionKey="config_seed_defaults" actionStates={actionStates} label="仅初始化映射配置" onClick={() => runAction('config_seed_defaults', () => workbenchApi.seedConfig(settings))} />
          </div>
        </details>
      </section>

      <section className="panel persistence-panel">
        <div>
          <h2>本地上下文</h2>
          <p className="muted-row">刷新页面后会自动恢复常用工作上下文；MySQL 密码不会写入 localStorage，需要重新输入。</p>
        </div>
        <div className="persistence-grid">
          <span>保存：host / port / database / user / data type / import mode</span>
          <span>保存：file path / import batch / analysis run / output path</span>
          <span>不保存：password / command result / runtime logs / customer data</span>
        </div>
        <div className="action-row">
          <button type="button" onClick={clearPersistedContext}>清除本地上下文并恢复默认</button>
        </div>
      </section>
    </section>
  );
}

import type { MySqlSettings } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  setSettings: (settings: MySqlSettings) => void;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
};

export function ConnectionPanel({ settings, setSettings, runAction }: Props) {
  return (
    <section className="panel form-panel">
      <h2>MySQL 连接与配置初始化</h2>
      <div className="form-grid">
        <input value={settings.host} onChange={(e) => setSettings({ ...settings, host: e.target.value })} placeholder="host" />
        <input value={settings.port} onChange={(e) => setSettings({ ...settings, port: Number(e.target.value) })} placeholder="port" />
        <input value={settings.database} onChange={(e) => setSettings({ ...settings, database: e.target.value })} placeholder="database" />
        <input value={settings.user} onChange={(e) => setSettings({ ...settings, user: e.target.value })} placeholder="user" />
        <input type="password" value={settings.secret} onChange={(e) => setSettings({ ...settings, secret: e.target.value })} placeholder="password" />
      </div>
      <div className="action-row">
        <button onClick={() => runAction('db_test_connection', () => workbenchApi.testDb(settings))}>测试连接</button>
        <button onClick={() => runAction('db_initialize', () => workbenchApi.initDb(settings))}>初始化数据库</button>
        <button onClick={() => runAction('config_seed_defaults', () => workbenchApi.seedConfig(settings))}>初始化映射配置</button>
      </div>
    </section>
  );
}

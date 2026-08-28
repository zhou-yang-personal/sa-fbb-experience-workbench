import { useState } from 'react';
import type { DecisionRuleProfileRow } from '../../shared/types';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';

type NumberKey = Exclude<keyof DecisionRuleProfileRow, 'rule_profile_id' | 'profile_name' | 'status' | 'notes' | 'updated_at'>;

const fields: Array<{ key: NumberKey; zh: string; en: string; unit: string }> = [
  { key: 'minimum_user_observations', zh: '用户最低观测数', en: 'Minimum user observations', unit: '条' },
  { key: 'minimum_app_users', zh: 'App 最低用户数', en: 'Minimum App users', unit: '人' },
  { key: 'minimum_app_observations', zh: 'App 最低有效观测数', en: 'Minimum App observations', unit: '条' },
  { key: 'sufficient_app_users', zh: 'App 充分样本用户数', en: 'Sufficient App users', unit: '人' },
  { key: 'sufficient_app_observations', zh: 'App 充分样本观测数', en: 'Sufficient App observations', unit: '条' },
  { key: 'persistent_poor_rate_pct', zh: '持续质差用户门槛', en: 'Persistent poor rate', unit: '%' },
  { key: 'attention_app_poor_rate_pct', zh: '关注 App 差观测门槛', en: 'Attention App poor rate', unit: '%' },
  { key: 'attention_app_persistent_user_rate_pct', zh: '关注 App 持续用户门槛', en: 'Attention App persistent users', unit: '%' },
  { key: 'problem_app_poor_rate_pct', zh: '问题 App 差观测门槛', en: 'Problem App poor rate', unit: '%' },
  { key: 'problem_app_persistent_user_rate_pct', zh: '问题 App 持续用户门槛', en: 'Problem App persistent users', unit: '%' },
  { key: 'severe_app_poor_rate_pct', zh: '严重 App 差观测门槛', en: 'Severe App poor rate', unit: '%' },
  { key: 'severe_app_persistent_user_rate_pct', zh: '严重 App 持续用户门槛', en: 'Severe App persistent users', unit: '%' },
  { key: 'severe_app_severe_user_rate_pct', zh: '严重 App 严重用户门槛', en: 'Severe App severe users', unit: '%' },
  { key: 'heavy_traffic_gb', zh: '重流量门槛', en: 'Heavy traffic', unit: 'GB' },
  { key: 'heavy_usage_hours', zh: '长时用户门槛', en: 'Heavy usage', unit: '小时' },
  { key: 'peak_hour_start', zh: '高峰开始', en: 'Peak start', unit: '时' },
  { key: 'peak_hour_end', zh: '高峰结束', en: 'Peak end', unit: '时' },
  { key: 'migration_min_traffic_gb', zh: '迁转潜客最低流量', en: 'Migration minimum traffic', unit: 'GB' },
  { key: 'speed_upgrade_min_traffic_gb', zh: '升套最低流量', en: 'Upgrade minimum traffic', unit: 'GB' },
  { key: 'speed_upgrade_max_effective_mbps', zh: '升套有效速率上限', en: 'Upgrade effective-rate ceiling', unit: 'Mbps' },
  { key: 'mesh_min_wifi_delay_ms', zh: 'AP 组网 Wi-Fi 时延门槛', en: 'Mesh Wi-Fi delay', unit: 'ms' },
  { key: 'app_bundle_min_observations', zh: 'App Bundle 最低使用次数', en: 'App bundle observations', unit: '条' },
];

export function DecisionRuleCenter({ c }: { c: WorkbenchController }) {
  const [rules, setRules] = useState<DecisionRuleProfileRow[]>([]);
  const [draft, setDraft] = useState<DecisionRuleProfileRow | null>(null);
  const [status, setStatus] = useState('尚未读取决策规则。');
  const zh = c.language === 'zh-CN';

  async function load() {
    const rows = await workbenchApi.decisionRules(c.effectiveSettings);
    setRules(rows);
    setDraft(rows.find((item) => item.status === 'draft') ?? null);
    setStatus(`已读取 ${rows.length} 个版本；分析运行固定绑定当时已发布版本。`);
  }

  async function create() {
    await c.runAction('decision_rule_create_draft', () => workbenchApi.createDecisionRuleDraft(c.effectiveSettings));
    await load();
  }

  async function save() {
    if (!draft) return;
    await c.runAction('decision_rule_update', () => workbenchApi.updateDecisionRule(c.effectiveSettings, draft));
    await load();
  }

  async function publish() {
    if (!draft) return;
    await c.runAction('decision_rule_publish', () => workbenchApi.publishDecisionRule(c.effectiveSettings, draft.rule_profile_id));
    await load();
  }

  return <article className="panel form-panel decision-rule-center">
    <div className="step-card-head">
      <div><h2>{zh ? '分析与潜客规则' : 'Analysis & opportunity rules'}</h2><p>{zh ? '所有样本、质差、分档和潜客门槛都在这里显式配置并版本化；修改只影响新分析运行。' : 'Sample, quality, segmentation and opportunity thresholds are explicit and versioned. Changes only affect new runs.'}</p></div>
      <div className="hub-actions"><button type="button" onClick={load}>{zh ? '读取规则' : 'Load'}</button><button type="button" onClick={create}>{zh ? '创建草稿' : 'Create draft'}</button></div>
    </div>
    <p className="analytics-context-line">{status}</p>
    {draft ? <>
      <div className="form-grid decision-rule-grid">
        <label>{zh ? '规则名称' : 'Profile name'}<input value={draft.profile_name} onChange={(event) => setDraft({ ...draft, profile_name: event.target.value })} /></label>
        {fields.map((field) => <label key={field.key}>{zh ? field.zh : field.en}<span className="field-with-unit"><input type="number" step="any" value={draft[field.key]} onChange={(event) => setDraft({ ...draft, [field.key]: Number(event.target.value) })} /><small>{field.unit}</small></span></label>)}
        <label>{zh ? '备注' : 'Notes'}<textarea value={draft.notes ?? ''} onChange={(event) => setDraft({ ...draft, notes: event.target.value })} /></label>
      </div>
      <div className="hub-actions"><button type="button" onClick={save}>{zh ? '保存草稿' : 'Save draft'}</button><button type="button" className="primary" onClick={publish}>{zh ? '校验并发布新版本' : 'Validate & publish'}</button></div>
    </> : <p className="muted-row">{zh ? '没有草稿。创建草稿会复制最新已发布版本。' : 'No draft. Create one from the latest published version.'}</p>}
    <div className="rule-version-list">{rules.map((rule) => <span key={rule.rule_profile_id} className={`status-pill ${rule.status === 'published' ? 'status-success' : 'status-warning'}`}>v{rule.version} · {rule.status} · {rule.profile_name}</span>)}</div>
  </article>;
}

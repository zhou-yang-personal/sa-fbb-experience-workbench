import { useEffect, useMemo, useState, type FormEvent } from 'react';
import type { AccessIpRangeRow, AccessRulePreviewResult, AccessRuleSetRow, AccessRuleValidationResult } from '../../shared/types';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';
import './AccessRuleCenter.css';

type RuleForm = {
  ruleId?: string;
  ruleName: string;
  cidr: string;
  startIp: string;
  endIp: string;
  accessType: 'CABLE' | 'FTTH' | 'OTHER';
  priority: number;
  enabled: boolean;
  notes: string;
};

const emptyForm: RuleForm = {
  ruleName: '',
  cidr: '',
  startIp: '',
  endIp: '',
  accessType: 'CABLE',
  priority: 100,
  enabled: true,
  notes: '',
};

function countLabel(value: number) {
  return new Intl.NumberFormat('en-US').format(value);
}

export function AccessRuleCenter({ c }: { c: WorkbenchController }) {
  const [ruleSets, setRuleSets] = useState<AccessRuleSetRow[]>([]);
  const [draft, setDraft] = useState<AccessRuleSetRow | null>(null);
  const [rules, setRules] = useState<AccessIpRangeRow[]>([]);
  const [form, setForm] = useState<RuleForm>(emptyForm);
  const [validation, setValidation] = useState<AccessRuleValidationResult | null>(null);
  const [preview, setPreview] = useState<AccessRulePreviewResult | null>(null);
  const [message, setMessage] = useState('连接 MySQL 后加载规则配置。');

  const publishedSets = useMemo(() => ruleSets.filter((item) => item.status === 'published'), [ruleSets]);
  const activePublished = publishedSets[0];

  async function loadDraft() {
    setMessage('正在读取 IP 接入规则…');
    try {
      const nextDraft = await workbenchApi.accessRuleDraft(c.settings);
      const [sets, nextRules] = await Promise.all([
        workbenchApi.accessRuleSets(c.settings),
        workbenchApi.accessRules(c.settings, nextDraft.rule_set_id),
      ]);
      setDraft(nextDraft);
      setRuleSets(sets);
      setRules(nextRules);
      setValidation(null);
      setMessage(`草稿 v${nextDraft.version} 已加载，共 ${nextRules.length} 条规则。`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => {
    void loadDraft();
  }, [c.settings.host, c.settings.port, c.settings.database, c.settings.user]);

  function editRule(rule: AccessIpRangeRow) {
    setForm({
      ruleId: rule.rule_id,
      ruleName: rule.rule_name,
      cidr: rule.cidr ?? '',
      startIp: rule.cidr ? '' : rule.start_ip,
      endIp: rule.cidr ? '' : rule.end_ip,
      accessType: rule.access_type as RuleForm['accessType'],
      priority: rule.priority,
      enabled: rule.enabled,
      notes: rule.notes ?? '',
    });
  }

  async function saveRule(event: FormEvent) {
    event.preventDefault();
    if (!draft) return;
    const result = await c.runAction('access_rule_upsert', () => workbenchApi.saveAccessRule(c.settings, {
      ruleSetId: draft.rule_set_id,
      ruleId: form.ruleId,
      ruleName: form.ruleName,
      cidr: form.cidr,
      startIp: form.startIp,
      endIp: form.endIp,
      accessType: form.accessType,
      priority: form.priority,
      enabled: form.enabled,
      notes: form.notes,
    }));
    if (result) {
      setForm(emptyForm);
      await loadDraft();
    }
  }

  async function toggleRule(rule: AccessIpRangeRow) {
    await c.runAction('access_rule_toggle', () => workbenchApi.saveAccessRule(c.settings, {
      ruleSetId: rule.rule_set_id,
      ruleId: rule.rule_id,
      ruleName: rule.rule_name,
      cidr: rule.cidr,
      startIp: rule.cidr ? undefined : rule.start_ip,
      endIp: rule.cidr ? undefined : rule.end_ip,
      accessType: rule.access_type as RuleForm['accessType'],
      priority: rule.priority,
      enabled: !rule.enabled,
      notes: rule.notes,
    }));
    await loadDraft();
  }

  async function removeRule(rule: AccessIpRangeRow) {
    if (!window.confirm(`从草稿中删除规则“${rule.rule_name}”？`)) return;
    const result = await c.runAction('access_rule_delete', () => workbenchApi.deleteAccessRule(c.settings, rule.rule_set_id, rule.rule_id));
    if (result) await loadDraft();
  }

  async function validateDraft() {
    if (!draft) return;
    const result = await c.runAction('access_rule_validate', () => workbenchApi.validateAccessRules(c.settings, draft.rule_set_id));
    if (result) setValidation(result as AccessRuleValidationResult);
  }

  async function updateDefaultAccessType(defaultAccessType: string) {
    if (!draft) return;
    const result = await c.runAction('access_rule_set_default_update', () => workbenchApi.updateAccessRuleDefault(c.settings, draft.rule_set_id, defaultAccessType));
    if (result) {
      setDraft(result as AccessRuleSetRow);
      setMessage(`未命中 IP 已设置为 ${(result as AccessRuleSetRow).default_access_type}。`);
      setValidation(null);
      setPreview(null);
    }
  }

  async function previewDraft() {
    if (!draft || !c.importBatchId.trim()) return;
    const result = await c.runAction('access_rule_preview', () => workbenchApi.previewAccessRules(c.settings, draft.rule_set_id, c.importBatchId));
    if (result) setPreview(result as AccessRulePreviewResult);
  }

  async function publishDraft() {
    if (!draft) return;
    const checked = await c.runAction('access_rule_validate_before_publish', () => workbenchApi.validateAccessRules(c.settings, draft.rule_set_id));
    if (!checked) return;
    setValidation(checked as AccessRuleValidationResult);
    if (!(checked as AccessRuleValidationResult).valid) return;
    if (!window.confirm(`发布接入规则 v${draft.version}？发布后该版本不可继续编辑。`)) return;
    const result = await c.runAction('access_rule_publish', () => workbenchApi.publishAccessRules(c.settings, draft.rule_set_id));
    if (result) await loadDraft();
  }

  async function applyPublishedToBatch() {
    if (!activePublished || !c.importBatchId.trim()) return;
    const applied = await c.runAction('access_rule_apply_to_batch', () => workbenchApi.applyAccessRulesToBatch(c.settings, activePublished.rule_set_id, c.importBatchId));
    if (applied) setMessage(`已将规则 v${activePublished.version} 应用到当前批次；重新运行 CLEAN/DWS/ADS 后看板生效。`);
  }

  return (
    <section className="access-rules-page">
      <header className="workspace-page-header">
        <div>
          <p className="eyebrow">Configuration · Access classification</p>
          <h2>IP 段与接入类型</h2>
          <p>Cable / FTTH 先由已发布 IP 规则识别，再回退到 CSV 原始字段与规则集默认类型。规则按版本绑定批次，不修改 RAW 数据。</p>
        </div>
        <button type="button" className="quiet-button" onClick={loadDraft}>刷新</button>
      </header>

      <section className="access-rule-summary">
        <article><span>编辑版本</span><strong>{draft ? `v${draft.version}` : '-'}</strong><small>{draft?.status ?? '未加载'}</small></article>
        <article><span>规则数量</span><strong>{rules.length}</strong><small>{rules.filter((rule) => rule.enabled).length} enabled</small></article>
        <article><span>当前发布</span><strong>{activePublished ? `v${activePublished.version}` : '-'}</strong><small>{activePublished?.published_at ?? '尚未发布'}</small></article>
        <article><span>当前批次</span><strong>{c.importBatchId ? '已选择' : '未选择'}</strong><small>{c.batchDisplayName || '预览和应用需要批次'}</small></article>
        <article><span>未命中默认</span><strong>{draft?.default_access_type ?? '-'}</strong><small>没有命中任何 IP 段时使用</small></article>
      </section>

      <section className="access-rule-workspace">
        <form className="access-rule-editor" onSubmit={saveRule}>
          <div className="section-heading">
            <div><h3>{form.ruleId ? '编辑规则' : '新增规则'}</h3><p>填写 CIDR，或填写起止 IPv4 地址。启用规则之间不得重叠。</p></div>
            {form.ruleId && <button type="button" className="text-button" onClick={() => setForm(emptyForm)}>取消编辑</button>}
          </div>
          <label>规则名称<input required value={form.ruleName} onChange={(event) => setForm({ ...form, ruleName: event.target.value })} placeholder="例如 Johannesburg Cable pool" /></label>
          <div className="form-grid access-rule-grid">
            <label>CIDR<input value={form.cidr} onChange={(event) => setForm({ ...form, cidr: event.target.value })} placeholder="10.20.0.0/16" /></label>
            <label>起始 IP<input disabled={Boolean(form.cidr.trim())} value={form.startIp} onChange={(event) => setForm({ ...form, startIp: event.target.value })} placeholder="10.20.0.0" /></label>
            <label>结束 IP<input disabled={Boolean(form.cidr.trim())} value={form.endIp} onChange={(event) => setForm({ ...form, endIp: event.target.value })} placeholder="10.20.255.255" /></label>
          </div>
          <div className="form-grid access-rule-grid compact">
            <label>接入类型<select value={form.accessType} onChange={(event) => setForm({ ...form, accessType: event.target.value as RuleForm['accessType'] })}><option value="CABLE">CABLE</option><option value="FTTH">FTTH</option><option value="OTHER">OTHER</option></select></label>
            <label>优先级<input type="number" value={form.priority} onChange={(event) => setForm({ ...form, priority: Number(event.target.value) })} /></label>
            <label className="check-label"><input type="checkbox" checked={form.enabled} onChange={(event) => setForm({ ...form, enabled: event.target.checked })} />立即启用</label>
          </div>
          <label>备注<textarea rows={3} value={form.notes} onChange={(event) => setForm({ ...form, notes: event.target.value })} placeholder="记录网段来源、负责人或变更原因" /></label>
          <button type="submit" className="primary-button" disabled={!draft}>{form.ruleId ? '保存修改' : '添加到草稿'}</button>
        </form>

        <aside className="access-rule-publish-panel">
          <div className="section-heading"><div><h3>验证与发布</h3><p>发布版本是批次分析的可追溯分类依据。</p></div></div>
          <label>未命中 IP 默认归类
            <select value={draft?.default_access_type ?? 'CABLE'} disabled={!draft} onChange={(event) => void updateDefaultAccessType(event.target.value)}>
              <option value="CABLE">CABLE（仅配置 FTTH 网段时推荐）</option>
              <option value="FTTH">FTTH（仅配置 Cable 网段时）</option>
              <option value="OTHER">OTHER</option>
              <option value="UNKNOWN">UNKNOWN（不自动归类）</option>
            </select>
          </label>
          <small>优先级：命中 IP 规则 → CSV 接入字段 → 此默认值。你的场景只需录入 FTTH 网段，并将默认值保持为 CABLE。</small>
          <div className={`validation-callout ${validation?.valid ? 'is-valid' : validation ? 'is-invalid' : ''}`}>
            <strong>{validation ? (validation.valid ? '可发布' : '需要修正') : '尚未验证'}</strong>
            <span>{validation?.message ?? '先完成规则配置，再运行重叠和格式检查。'}</span>
          </div>
          <div className="stacked-actions">
            <button type="button" onClick={validateDraft} disabled={!draft}>验证草稿</button>
            <button type="button" onClick={previewDraft} disabled={!draft || !c.importBatchId}>用当前批次预览</button>
            <button type="button" className="primary-button" onClick={publishDraft} disabled={!draft}>发布规则版本</button>
            <button type="button" onClick={applyPublishedToBatch} disabled={!activePublished || !c.importBatchId}>将已发布版本应用到当前批次</button>
          </div>
          {preview && <div className="preview-metrics">
            <div><span>样本 IP</span><strong>{countLabel(preview.sample_ip_count)}</strong></div>
            <div><span>识别覆盖率</span><strong>{preview.coverage_pct.toFixed(1)}%</strong></div>
            <div><span>Cable</span><strong>{countLabel(preview.cable_ip_count)}</strong></div>
            <div><span>FTTH</span><strong>{countLabel(preview.ftth_ip_count)}</strong></div>
            <div><span>默认归类</span><strong>{countLabel(preview.fallback_ip_count)}</strong></div>
            <div><span>未匹配</span><strong>{countLabel(preview.unmatched_ip_count)}</strong></div>
          </div>}
        </aside>
      </section>

      <section className="access-rule-table-card">
        <div className="section-heading"><div><h3>草稿规则</h3><p>{message}</p></div><span>{draft?.rule_set_name ?? '-'}</span></div>
        <div className="access-rule-table-wrap">
          <table className="access-rule-table">
            <thead><tr><th>规则</th><th>范围</th><th>接入</th><th>优先级</th><th>状态</th><th>操作</th></tr></thead>
            <tbody>
              {rules.map((rule) => <tr key={rule.rule_id} className={rule.enabled ? '' : 'is-disabled'}>
                <td><strong>{rule.rule_name}</strong><small>{rule.notes || rule.updated_at}</small></td>
                <td><code>{rule.cidr || `${rule.start_ip} — ${rule.end_ip}`}</code></td>
                <td><span className={`access-badge access-${rule.access_type.toLowerCase()}`}>{rule.access_type}</span></td>
                <td>{rule.priority}</td>
                <td>{rule.enabled ? '启用' : '停用'}</td>
                <td><div className="table-actions"><button type="button" onClick={() => editRule(rule)}>编辑</button><button type="button" onClick={() => toggleRule(rule)}>{rule.enabled ? '停用' : '启用'}</button><button type="button" className="danger-text" onClick={() => removeRule(rule)}>删除</button></div></td>
              </tr>)}
              {!rules.length && <tr><td colSpan={6} className="empty-cell">当前草稿没有规则。先添加 Cable 或 FTTH IPv4 网段。</td></tr>}
            </tbody>
          </table>
        </div>
      </section>
    </section>
  );
}

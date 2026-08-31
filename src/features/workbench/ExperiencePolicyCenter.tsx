import { useState } from 'react';
import type { AppExperienceProfileRow, ExperiencePolicyRow } from '../../shared/types';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';
import './ExperiencePolicyCenter.css';

function numeric(value: string) { const result = Number(value); return Number.isFinite(result) ? result : 0; }
function optionalNumeric(value: string) { return value.trim() === '' ? undefined : numeric(value); }

export function ExperiencePolicyCenter({ c }: { c: WorkbenchController }) {
  const zh = c.language === 'zh-CN';
  const [policies, setPolicies] = useState<ExperiencePolicyRow[]>([]);
  const [profiles, setProfiles] = useState<AppExperienceProfileRow[]>([]);
  const [draft, setDraft] = useState<ExperiencePolicyRow | null>(null);
  const [profile, setProfile] = useState<AppExperienceProfileRow | null>(null);
  const [message, setMessage] = useState(zh ? '点击加载，不会自动访问数据库。' : 'Load explicitly; no automatic database request.');

  async function load() {
    const result = await c.runAction('experience_policy_list', () => workbenchApi.experiencePolicies(c.effectiveSettings));
    if (!Array.isArray(result)) return;
    const rows = result as ExperiencePolicyRow[];
    setPolicies(rows);
    const selected = rows.find((item) => item.status === 'draft') ?? rows.find((item) => item.status === 'published') ?? null;
    setDraft(selected ? { ...selected } : null);
    if (selected) {
      const nextProfiles = await workbenchApi.experienceProfiles(c.effectiveSettings, selected.policy_id);
      setProfiles(nextProfiles); setProfile(nextProfiles[0] ? { ...nextProfiles[0] } : null);
    }
    setMessage(zh ? `已加载 ${rows.length} 个策略版本。` : `${rows.length} policy versions loaded.`);
  }

  async function selectPolicy(item: ExperiencePolicyRow) {
    setDraft({ ...item });
    const result = await c.runAction('experience_profile_list', () => workbenchApi.experienceProfiles(c.effectiveSettings, item.policy_id));
    if (Array.isArray(result)) { setProfiles(result as AppExperienceProfileRow[]); setProfile((result as AppExperienceProfileRow[])[0] ?? null); }
  }

  async function createDraft() { await c.runAction('experience_policy_create_draft', () => workbenchApi.createExperiencePolicyDraft(c.effectiveSettings)); await load(); }
  async function savePolicy() { if (!draft || draft.status !== 'draft') return; await c.runAction('experience_policy_update', () => workbenchApi.updateExperiencePolicy(c.effectiveSettings, draft)); await load(); }
  async function publish() { if (!draft || draft.status !== 'draft') return; await c.runAction('experience_policy_publish', () => workbenchApi.publishExperiencePolicy(c.effectiveSettings, draft.policy_id)); await load(); }
  async function saveProfile() { if (!profile || draft?.status !== 'draft') return; await c.runAction('experience_profile_update', () => workbenchApi.updateExperienceProfile(c.effectiveSettings, profile)); await selectPolicy(draft); }
  async function cloneProfile() { if (!profile || draft?.status !== 'draft') return; await c.runAction('experience_profile_clone', () => workbenchApi.cloneExperienceProfile(c.effectiveSettings, draft.policy_id, profile.profile_id)); await selectPolicy(draft); }

  function countField(key: keyof ExperiencePolicyRow, labelZh: string, labelEn: string) {
    if (!draft) return null;
    return <label>{zh ? labelZh : labelEn}<input type="number" min={1} value={Number(draft[key])} disabled={draft.status !== 'draft'} onChange={(event) => setDraft({ ...draft, [key]: numeric(event.target.value) })} /></label>;
  }
  function rateField(key: keyof ExperiencePolicyRow, labelZh: string, labelEn: string) {
    if (!draft) return null;
    return <label>{zh ? labelZh : labelEn}<div className="policy-rate-input"><input type="number" min={0} max={100} step="0.1" value={Number(draft[key])} disabled={draft.status !== 'draft'} onChange={(event) => setDraft({ ...draft, [key]: numeric(event.target.value) })} /><span>%</span></div></label>;
  }
  function profileField(key: keyof AppExperienceProfileRow, label: string) {
    if (!profile) return null;
    return <label>{label}<input type="number" step="0.01" value={profile[key] === undefined ? '' : Number(profile[key])} disabled={draft?.status !== 'draft'} onChange={(event) => setProfile({ ...profile, [key]: optionalNumeric(event.target.value) })} /></label>;
  }

  return <section className="panel experience-policy-center">
    <header className="step-card-head"><div><p className="eyebrow">EXPERIENCE POLICY</p><h2>{zh ? '体验口径与 Finding 规则' : 'Experience metric & finding policy'}</h2><p>{zh ? '所有比例阈值、持续性、最低样本与 App Profile 都版本化；已发布版本不可原地修改。' : 'Rates, persistence, minimum samples and App profiles are versioned; published versions are immutable.'}</p></div><div className="policy-actions"><button onClick={load}>{zh ? '加载策略' : 'Load'}</button><button onClick={createDraft} disabled={Boolean(policies.find((item) => item.status === 'draft'))}>{zh ? '从已发布版本新建草稿' : 'Clone draft'}</button></div></header>
    <p className="muted-row">{message}</p>
    <div className="policy-version-list">{policies.map((item) => <button key={item.policy_id} className={draft?.policy_id === item.policy_id ? 'is-selected' : ''} onClick={() => selectPolicy(item)}><strong>v{item.version} · {item.policy_name}</strong><span>{item.status}</span><small>{item.updated_at}</small></button>)}</div>
    {draft && <>
      <section className="policy-form-section"><header><div><h3>{zh ? '用户持续性与严重性' : 'User persistence & severity'}</h3><p>{zh ? '曾受影响只要求至少一次异常；持续和严重用户必须同时满足以下次数与比例。' : 'Ever affected requires one issue; persistent and severe users must meet both counts and rates below.'}</p></div><span className="status-pill">v{draft.version} · {draft.status}</span></header><div className="policy-form-grid">
        <label>{zh ? '策略名称' : 'Policy name'}<input value={draft.policy_name} disabled={draft.status !== 'draft'} onChange={(event) => setDraft({ ...draft, policy_name: event.target.value })} /></label>
        {countField('persistent_min_valid_obs', '持续用户最少有效观测', 'Persistent minimum valid observations')}
        {countField('persistent_min_poor_obs', '持续用户最少差观测', 'Persistent minimum poor observations')}
        {rateField('persistent_min_poor_rate_pct', '持续用户差观测率门槛', 'Persistent poor observation rate')}
        {countField('severe_user_min_valid_obs', '严重用户最少有效观测', 'Severe minimum valid observations')}
        {countField('severe_user_min_severe_obs', '严重用户最少严重观测', 'Severe minimum severe observations')}
        {rateField('severe_user_min_severe_rate_pct', '严重观测率门槛', 'Severe observation rate')}
      </div></section>
      <section className="policy-form-section"><header><div><h3>{zh ? 'App 样本与 Finding 门槛' : 'App sample & finding thresholds'}</h3><p>{zh ? '样本不足的 App 不进入问题排名，空值不会显示为 0。' : 'Insufficient apps do not enter issue rankings; missing values are not zero.'}</p></div></header><div className="policy-form-grid">
        {countField('minimum_app_eligible_users', 'App 最少合格用户', 'Minimum eligible users per app')}
        {countField('minimum_app_valid_obs', 'App 最少有效观测', 'Minimum valid observations per app')}
        {rateField('finding_attention_persistent_user_rate_pct', '关注 Finding 持续用户率', 'Attention finding persistent rate')}
        {rateField('finding_severe_user_rate_pct', '严重 Finding 严重用户率', 'Severe finding user rate')}
        <label className="policy-notes">{zh ? '版本说明' : 'Version notes'}<textarea value={draft.notes ?? ''} disabled={draft.status !== 'draft'} onChange={(event) => setDraft({ ...draft, notes: event.target.value })} /></label>
      </div><div className="policy-actions"><button disabled={draft.status !== 'draft'} onClick={savePolicy}>{zh ? '保存策略草稿' : 'Save policy draft'}</button><button className="primary-button" disabled={draft.status !== 'draft'} onClick={publish}>{zh ? '发布不可变版本' : 'Publish immutable version'}</button></div></section>
      <section className="policy-form-section"><header><div><h3>{zh ? 'App Experience Profiles' : 'App experience profiles'}</h3><p>{zh ? '不同数据类型和 App 类别可使用不同体验阈值；空字段表示该指标不参与本 Profile。' : 'Data types and app categories can use distinct thresholds; blank metrics are not evaluated.'}</p></div></header><div className="profile-tabs">{profiles.map((item) => <button className={profile?.profile_id === item.profile_id ? 'is-selected' : ''} key={item.profile_id} onClick={() => setProfile({ ...item })}>{item.profile_code}<small>{item.data_type} · {item.app_category ?? 'fallback'}</small></button>)}</div>{profile && <div className="policy-form-grid profile-editor">
        <label>{zh ? '名称' : 'Name'}<input value={profile.profile_name} disabled={draft.status !== 'draft'} onChange={(event) => setProfile({ ...profile, profile_name: event.target.value })} /></label>
        <label>{zh ? 'App 类别（空=兜底）' : 'App category (blank=fallback)'}<input value={profile.app_category ?? ''} disabled={draft.status !== 'draft'} onChange={(event) => setProfile({ ...profile, app_category: event.target.value || undefined })} /></label>
        {profileField('poor_vmos_below', 'Poor vMOS <')}{profileField('severe_vmos_below', 'Severe vMOS <')}
        {profileField('poor_mos_below', 'Poor MOS <')}{profileField('severe_mos_below', 'Severe MOS <')}
        {profileField('poor_subscriber_rtt_ms_at_least', 'Poor user RTT ≥ ms')}{profileField('severe_subscriber_rtt_ms_at_least', 'Severe user RTT ≥ ms')}
        {profileField('poor_network_rtt_ms_at_least', 'Poor network RTT ≥ ms')}{profileField('severe_network_rtt_ms_at_least', 'Severe network RTT ≥ ms')}
        {profileField('poor_user_loss_pct_at_least', 'Poor user loss ≥ %')}{profileField('severe_user_loss_pct_at_least', 'Severe user loss ≥ %')}
        {profileField('poor_network_loss_pct_at_least', 'Poor network loss ≥ %')}{profileField('severe_network_loss_pct_at_least', 'Severe network loss ≥ %')}
        {profileField('poor_jitter_ms_at_least', 'Poor jitter ≥ ms')}{profileField('severe_jitter_ms_at_least', 'Severe jitter ≥ ms')}
      </div>}<div className="policy-actions"><button disabled={!profile || draft.status !== 'draft'} onClick={cloneProfile}>{zh ? '复制为 App 类别 Profile' : 'Clone category profile'}</button><button disabled={!profile || draft.status !== 'draft'} onClick={saveProfile}>{zh ? '保存当前 Profile' : 'Save current profile'}</button></div></section>
    </>}
  </section>;
}

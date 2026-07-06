import { useState } from 'react';
import { DashboardCenter } from './DashboardCenter';
import { FinalLeadCenter } from './FinalLeadCenter';
import { MetricGrid } from './MetricGrid';
import { ResultTables } from './ResultTables';
import type { WorkbenchController } from './useWorkbenchController';

type AnalysisModuleId = 'overview' | 'app' | 'video' | 'game' | 'quality' | 'cableFiber' | 'leads' | 'profile';

type AnalysisModule = {
  id: AnalysisModuleId;
  title: string;
  description: string;
  requiredFields: string[];
  aggregateTables: string[];
  dataTypes?: string[];
};

const analysisModules: AnalysisModule[] = [
  {
    id: 'overview',
    title: '总览看板',
    description: '用户、流量、时长、接入类型和 Lead 总体结论。',
    requiredFields: ['import_batch_id', 'analysis_run_id', 'user_key', 'duration / traffic'],
    aggregateTables: ['ads_dashboard_overview'],
  },
  {
    id: 'app',
    title: '应用使用分析',
    description: '按应用分类查看活跃用户、使用时长、流量和 Top App。',
    requiredFields: ['app_name', 'app_category', 'statistics_duration', 'traffic volume'],
    aggregateTables: ['dws_app_category_hourly', 'ads_app_category_summary'],
    dataTypes: ['tcp', 'game'],
  },
  {
    id: 'video',
    title: '视频体验分析',
    description: '围绕 Universal Video / OTT / Short Video 分析 VMOS、速率和卡顿。',
    requiredFields: ['universal_video_applications', 'vmos', 'effective_download_duration_s', 'downloaded_data_volume_kb'],
    aggregateTables: ['dws_tcp_app_hourly', 'ads_video_experience_summary'],
    dataTypes: ['tcp'],
  },
  {
    id: 'game',
    title: '游戏体验分析',
    description: '围绕 Game 应用分析 MOS、RTT、jitter 和游戏时长。',
    requiredFields: ['application_protocol', 'mos', 'game_duration_s', 'upstream_rtt_jitter_ms', 'downstream_rtt_jitter_ms'],
    aggregateTables: ['dws_game_app_hourly', 'ads_game_experience_summary'],
    dataTypes: ['game'],
  },
  {
    id: 'quality',
    title: '网络体验质量',
    description: 'RTT、PLR、MOS / VMOS、Wi-Fi delay 和瓶颈侧拆分。',
    requiredFields: ['subscriber_side_rtt_ms', 'network_side_rtt_ms', 'packet_loss', 'wifi_delay_ms'],
    aggregateTables: ['dws_quality_hourly', 'ads_experience_quality'],
    dataTypes: ['tcp', 'game'],
  },
  {
    id: 'cableFiber',
    title: 'Cable vs FTTH 对比',
    description: '按接入类型比较忙时速率、时延、丢包和体验评分。',
    requiredFields: ['user_type / wan_type', 'hour_of_day', 'rtt', 'loss', 'effective_download_mbps'],
    aggregateTables: ['dws_access_hourly', 'ads_cable_fiber_compare'],
    dataTypes: ['tcp', 'game'],
  },
  {
    id: 'leads',
    title: '迁转升套机会',
    description: 'Lead Type、Final Action、推荐套餐和可导出营销清单。',
    requiredFields: ['user_key', 'lead_type', 'demand_score', 'migration_motive_score', 'final_action'],
    aggregateTables: ['ads_migration_lead_user', 'ads_final_marketing_lead'],
  },
  {
    id: 'profile',
    title: '用户明细画像',
    description: '按用户查看应用偏好、体验指标、Lead Type 和推荐动作。',
    requiredFields: ['user_key', 'app_preference', 'quality metrics', 'lead_type'],
    aggregateTables: ['dws_user_profile', 'ads_migration_lead_user'],
  },
];

function hasBatch(c: WorkbenchController) {
  return Boolean(c.importBatchId.trim());
}

function hasRun(c: WorkbenchController) {
  return Boolean(c.analysisRunId.trim());
}

function moduleStatus(module: AnalysisModule, c: WorkbenchController) {
  if (!hasBatch(c)) return { available: false, reason: '先选择或输入导入批次 import_batch_id。' };
  if (!hasRun(c)) return { available: false, reason: '先设置 analysis_run_id。' };
  if (module.dataTypes && !module.dataTypes.includes(c.dataType)) {
    return { available: false, reason: `当前批次类型为 ${c.dataType.toUpperCase()}，该模块需要 ${module.dataTypes.map((item) => item.toUpperCase()).join(' / ')} 数据。` };
  }
  return { available: true, reason: '可用：批次、运行 ID 和数据类型满足当前模块基础要求。' };
}

function renderModuleBody(activeModule: AnalysisModule, c: WorkbenchController) {
  if (['overview', 'app', 'video', 'game', 'quality', 'cableFiber'].includes(activeModule.id)) {
    return (
      <section className="workbench-section-stack">
        <DashboardCenter {...c} activeModuleTitle={activeModule.title} />
        <MetricGrid metrics={c.allMetrics} />
      </section>
    );
  }
  if (activeModule.id === 'leads') {
    return (
      <section className="workbench-section-stack">
        <FinalLeadCenter {...c} />
        <ResultTables leads={c.leads} finalLeads={c.finalLeads} />
      </section>
    );
  }
  return (
    <section className="workbench-section-stack">
      <ResultTables leads={c.leads} finalLeads={c.finalLeads} />
      <MetricGrid metrics={c.allMetrics} />
    </section>
  );
}

export function AnalysisWorkspace({ c }: { c: WorkbenchController }) {
  const [activeModuleId, setActiveModuleId] = useState<AnalysisModuleId>('overview');
  const activeModule = analysisModules.find((item) => item.id === activeModuleId) ?? analysisModules[0];
  const activeStatus = moduleStatus(activeModule, c);

  return (
    <section className="workbench-section-stack analysis-workspace">
      <article className="panel form-panel analysis-batch-panel">
        <div className="step-card-head">
          <div>
            <h2>数据分析：先选择导入批次</h2>
            <p className="hero-text">每次分析以一个导入批次为边界。看板只读取当前批次对应的 CLEAN / DWS / ADS 结果，不跨批次混算。</p>
          </div>
          <span className="step-badge">核心入口</span>
        </div>
        <div className="form-grid batch-context-form">
          <label>
            批次名称
            <input value={c.batchDisplayName} onChange={(event) => c.setBatchDisplayName(event.target.value)} placeholder="例如：TCP 视频体验｜Claro｜2026-07-05 晚高峰" />
          </label>
          <label>
            import_batch_id
            <input value={c.importBatchId} onChange={(event) => c.setImportBatchId(event.target.value)} placeholder="选择或粘贴导入批次 ID" />
          </label>
          <label>
            analysis_run_id
            <input value={c.analysisRunId} onChange={(event) => c.setAnalysisRunId(event.target.value)} placeholder="例如 RUN_20260705_VIDEO_PEAK" />
          </label>
          <label>
            批次数据类型
            <select value={c.dataType} onChange={(event) => c.setDataType(event.target.value as WorkbenchController['dataType'])}>
              <option value="tcp">TCP / Universal Video</option>
              <option value="game">Game</option>
              <option value="crm">CRM Users</option>
              <option value="coverage">FTTH Coverage</option>
              <option value="reachability">Reachability</option>
            </select>
          </label>
        </div>
        <div className="summary-pills">
          <span className={hasBatch(c) ? 'status-pill status-success' : 'status-pill status-failure'}>batch {hasBatch(c) ? 'selected' : 'missing'}</span>
          <span className={hasRun(c) ? 'status-pill status-success' : 'status-pill status-failure'}>analysis run {hasRun(c) ? 'ready' : 'missing'}</span>
          <span className="status-pill">data {c.dataType.toUpperCase()}</span>
          <span className="status-pill">tables: RAW → CLEAN → module AGG / ADS</span>
        </div>
      </article>

      <article className="panel form-panel">
        <h2>分析模块</h2>
        <p className="hero-text">模块是否可用由批次、数据类型、必填字段和所需聚合表共同决定。不可用模块保持置灰并展示原因。</p>
        <div className="analysis-module-grid">
          {analysisModules.map((module) => {
            const status = moduleStatus(module, c);
            const active = activeModuleId === module.id;
            return (
              <button
                key={module.id}
                type="button"
                className={`analysis-module-card ${active ? 'is-active' : ''} ${status.available ? '' : 'is-disabled'}`}
                disabled={!status.available}
                onClick={() => setActiveModuleId(module.id)}
                title={status.reason}
              >
                <strong>{module.title}</strong>
                <span>{module.description}</span>
                <small>{status.available ? '可用' : status.reason}</small>
              </button>
            );
          })}
        </div>
        <div className="table-like module-readiness-table">
          <div className="table-row module-readiness-row table-head"><span>模块</span><span>必填字段</span><span>聚合表</span><span>状态</span></div>
          {analysisModules.map((module) => {
            const status = moduleStatus(module, c);
            return (
              <div key={module.id} className="table-row module-readiness-row">
                <span>{module.title}</span>
                <span>{module.requiredFields.join(', ')}</span>
                <span>{module.aggregateTables.join(', ')}</span>
                <span className={status.available ? 'status-success' : 'status-failure-text'}>{status.available ? 'enabled' : status.reason}</span>
              </div>
            );
          })}
        </div>
      </article>

      <article className={`panel form-panel ${activeStatus.available ? '' : 'disabled-analysis-panel'}`}>
        <h2>{activeModule.title}</h2>
        <p className="hero-text">{activeModule.description}</p>
        {!activeStatus.available && <p className="muted-row status-failure-text">{activeStatus.reason}</p>}
        {activeStatus.available && renderModuleBody(activeModule, c)}
      </article>
    </section>
  );
}

import { useEffect, useMemo, useState } from 'react';
import type { BatchListItem, BatchTableRegistryRow, MetricCard, ModuleStatusRow } from '../../shared/types';
import { BatchSelector } from './BatchSelector';
import { MetricGrid } from './MetricGrid';
import { OverviewDashboard, AppUsageDashboard, VideoExperienceDashboard, GameExperienceDashboard, NetworkQualityDashboard, CableFiberCompareDashboard, MigrationLeadDashboard, UserProfileDashboard } from './ModuleDashboards';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';

type AnalysisModuleId = 'overview' | 'app_usage' | 'video_experience' | 'game_experience' | 'network_quality' | 'cable_fiber_compare' | 'migration_lead' | 'user_profile';

type AnalysisModule = {
  id: AnalysisModuleId;
  title: string;
  description: string;
  aggregateTables: string[];
  dataTypes?: string[];
};

const analysisModules: AnalysisModule[] = [
  {
    id: 'overview',
    title: '总览看板',
    description: '用户、流量、时长、接入类型和 Lead 总体结论。',
    aggregateTables: ['dws_user_daily_profile', 'ads_dashboard_overview'],
  },
  {
    id: 'app_usage',
    title: '应用使用分析',
    description: '按应用分类查看活跃用户、使用时长、流量和 Top App。',
    aggregateTables: ['dws_app_category_daily', 'ads_app_category_detail'],
    dataTypes: ['tcp', 'game'],
  },
  {
    id: 'video_experience',
    title: '视频体验分析',
    description: '围绕 Universal Video / OTT / Short Video 分析 VMOS、速率和卡顿。',
    aggregateTables: ['dwd_tcp_detail_clean', 'dws_user_daily_profile', 'ads_experience_quality_summary'],
    dataTypes: ['tcp'],
  },
  {
    id: 'game_experience',
    title: '游戏体验分析',
    description: '围绕 Game 应用分析 MOS、RTT、jitter 和游戏时长。',
    aggregateTables: ['dwd_game_detail_clean', 'dws_user_daily_profile'],
    dataTypes: ['game'],
  },
  {
    id: 'network_quality',
    title: '网络体验质量',
    description: 'RTT、PLR、MOS / VMOS、Wi-Fi delay 和瓶颈侧拆分。',
    aggregateTables: ['dws_access_type_hourly_compare', 'ads_experience_quality_summary'],
    dataTypes: ['tcp', 'game'],
  },
  {
    id: 'cable_fiber_compare',
    title: 'Cable vs FTTH 对比',
    description: '按接入类型比较忙时速率、时延、丢包和体验评分。',
    aggregateTables: ['dws_access_type_hourly_compare', 'ads_cable_fiber_compare'],
    dataTypes: ['tcp', 'game'],
  },
  {
    id: 'migration_lead',
    title: '迁转升套机会',
    description: 'Lead Type、Final Action、推荐套餐和可导出营销清单。',
    aggregateTables: ['ads_migration_lead_user', 'ads_final_marketing_lead_user'],
  },
  {
    id: 'user_profile',
    title: '用户明细画像',
    description: '按用户查看应用偏好、体验指标、Lead Type 和推荐动作。',
    aggregateTables: ['dws_user_daily_profile'],
  },
];

function isModuleEnabled(module: AnalysisModule, status: ModuleStatusRow | undefined, dataType: string) {
  if (status) return status.enabled;
  if (module.dataTypes && !module.dataTypes.includes(dataType)) return false;
  return false;
}

function renderModuleBody(activeModuleId: AnalysisModuleId, c: WorkbenchController) {
  switch (activeModuleId) {
    case 'overview':
      return <OverviewDashboard c={c} />;
    case 'app_usage':
      return <AppUsageDashboard c={c} />;
    case 'video_experience':
      return <VideoExperienceDashboard c={c} />;
    case 'game_experience':
      return <GameExperienceDashboard c={c} />;
    case 'network_quality':
      return <NetworkQualityDashboard c={c} />;
    case 'cable_fiber_compare':
      return <CableFiberCompareDashboard c={c} />;
    case 'migration_lead':
      return <MigrationLeadDashboard c={c} />;
    case 'user_profile':
      return <UserProfileDashboard c={c} />;
  }
}

function moduleMetricsFallback(status: ModuleStatusRow[], registry: BatchTableRegistryRow[]): MetricCard[] {
  return [
    { label: 'module status', value: String(status.length), hint: `enabled=${status.filter((item) => item.enabled).length}` },
    { label: 'batch registry', value: String(registry.length), hint: 'meta_batch_table_registry' },
  ];
}

export function AnalysisWorkspace({ c }: { c: WorkbenchController }) {
  const [activeModuleId, setActiveModuleId] = useState<AnalysisModuleId>('overview');
  const [batches, setBatches] = useState<BatchListItem[]>([]);
  const [tableRegistry, setTableRegistry] = useState<BatchTableRegistryRow[]>([]);
  const [moduleStatus, setModuleStatus] = useState<ModuleStatusRow[]>([]);
  const [statusMessage, setStatusMessage] = useState('请选择批次，系统会自动刷新模块状态。');

  const activeModule = useMemo(
    () => analysisModules.find((item) => item.id === activeModuleId) ?? analysisModules[0],
    [activeModuleId],
  );
  const activeStatus = moduleStatus.find((item) => item.module_id === activeModuleId);
  const moduleMetrics = useMemo(() => moduleMetricsFallback(moduleStatus, tableRegistry), [moduleStatus, tableRegistry]);

  async function refreshBatchList() {
    const result = await workbenchApi.listBatches(c.settings);
    setBatches(result);
    setStatusMessage(result.length ? `已加载 ${result.length} 个批次。` : '当前没有可用批次。');
  }

  async function refreshBatchContext(batchId = c.importBatchId, analysisRunId = c.analysisRunId) {
    if (!batchId.trim()) {
      setTableRegistry([]);
      setModuleStatus([]);
      setStatusMessage('请先选择批次。');
      return;
    }
    await workbenchApi.prepareBatchTables(c.settings, batchId);
    const registry = await workbenchApi.batchTableRegistry(c.settings, batchId);
    const status = await workbenchApi.moduleStatus(c.settings, batchId, analysisRunId.trim() || undefined);
    setTableRegistry(registry);
    setModuleStatus(status);
    await c.loadMetrics('analysis_get_module_metrics', () => workbenchApi.moduleMetrics(c.settings, batchId, analysisRunId.trim() || undefined));
    setStatusMessage(status.some((item) => item.enabled) ? '模块状态已刷新。' : '当前模块多为置灰，请查看原因。');
  }

  useEffect(() => {
    void refreshBatchList().catch((error) => {
      setStatusMessage(error instanceof Error ? error.message : String(error));
    });
  }, [c.settings.host, c.settings.port, c.settings.database, c.settings.user]);

  useEffect(() => {
    if (!c.importBatchId.trim()) return;
    void refreshBatchContext().catch((error) => {
      setStatusMessage(error instanceof Error ? error.message : String(error));
    });
  }, [c.importBatchId, c.analysisRunId]);

  const visibleStatus = activeStatus ?? moduleStatus.find((item) => item.module_id === activeModule.id);
  const enabled = isModuleEnabled(activeModule, visibleStatus, c.dataType);
  const resultsNotGenerated = moduleStatus.some((item) => (item.status_text ?? '').includes('尚未生成分析结果'));

  return (
    <section className="workbench-section-stack analysis-workspace">
      <BatchSelector
        batches={batches}
        selectedBatchId={c.importBatchId}
        onRefresh={refreshBatchList}
        statusText={statusMessage}
        onSelectBatch={(batch) => {
          if (!batch) {
            c.setImportBatchId('');
            c.setBatch(null);
            c.setOverview(null);
            c.setDashboardCharts([]);
            setTableRegistry([]);
            setModuleStatus([]);
            setStatusMessage('请先选择批次。');
            return;
          }
          c.setImportBatchId(batch.import_batch_id);
          c.setDataType(batch.data_type as WorkbenchController['dataType']);
          c.setOverview(null);
          c.setDashboardCharts([]);
          c.setBatch({
            import_batch_id: batch.import_batch_id,
            batch_display_name: batch.batch_display_name,
            data_type: batch.data_type,
            source_file_name: batch.source_file_name,
            status: batch.status,
          });
          c.setBatchDisplayName(batch.batch_display_name ?? batch.source_file_name);
        }}
      />

      <article className="panel form-panel">
        <h2>分析参数</h2>
        <div className="form-grid">
          <label>
            analysis_run_id
            <input value={c.analysisRunId} onChange={(event) => c.setAnalysisRunId(event.target.value)} placeholder="例如 RUN_20260705_VIDEO_PEAK" />
          </label>
        </div>
        <div className="summary-pills">
          <span className={c.importBatchId ? 'status-pill status-success' : 'status-pill status-failure'}>batch {c.importBatchId ? 'selected' : 'missing'}</span>
          <span className={c.analysisRunId ? 'status-pill status-success' : 'status-pill status-failure'}>analysis run {c.analysisRunId ? 'ready' : 'missing'}</span>
          <span className="status-pill">data {c.dataType.toUpperCase()}</span>
        </div>
      </article>

      <article className="panel form-panel">
        <h2>模块可用性</h2>
        <p className="hero-text">模块可用性来自后端真实检查。缺 data_type、缺 required table 或 batch 状态不完整时会置灰并给出原因。</p>
        {resultsNotGenerated && <p className="muted-row status-failure-text">当前批次尚未完成分析结果生成，请回到数据导入，完成 CLEAN/DWS/ADS 后再查看。</p>}
        <div className="analysis-module-grid">
          {analysisModules.map((module) => {
            const status = moduleStatus.find((item) => item.module_id === module.id);
            const active = activeModuleId === module.id;
            const disabled = !isModuleEnabled(module, status, c.dataType);
            return (
              <button
                key={module.id}
                type="button"
                className={`analysis-module-card ${active ? 'is-active' : ''} ${disabled ? 'is-disabled' : ''}`}
                disabled={disabled}
                onClick={() => setActiveModuleId(module.id)}
                title={status?.status_text ?? module.description}
              >
                <strong>{module.title}</strong>
                <span>{module.description}</span>
                <small>{disabled ? status?.status_text ?? 'disabled' : 'enabled'}</small>
              </button>
            );
          })}
        </div>
        <div className="table-like module-readiness-table">
          <div className="table-row module-readiness-row table-head"><span>模块</span><span>必需表</span><span>状态</span></div>
          {analysisModules.map((module) => {
            const status = moduleStatus.find((item) => item.module_id === module.id);
            const disabled = !isModuleEnabled(module, status, c.dataType);
            return (
              <div key={module.id} className="table-row module-readiness-row">
                <span>{module.title}</span>
                <span>{module.aggregateTables.join(', ')}</span>
                <span className={disabled ? 'status-failure-text' : 'status-success'}>{status?.status_text ?? (disabled ? 'disabled' : 'enabled')}</span>
              </div>
            );
          })}
        </div>
      </article>

      <article className={`panel form-panel ${enabled ? '' : 'disabled-analysis-panel'}`}>
        <h2>{activeModule.title}</h2>
        <p className="hero-text">{activeModule.description}</p>
        {!enabled && <p className="muted-row status-failure-text">{visibleStatus?.status_text ?? '当前模块置灰，请先选择批次或补齐依赖表。'}</p>}
        {enabled && renderModuleBody(activeModule.id, c)}
        <MetricGrid metrics={c.allMetrics.length ? c.allMetrics : moduleMetrics} />
      </article>
    </section>
  );
}

import type { DashboardChartGroup, DashboardOverview, MetricCard } from '../../shared/types';
import { selectCsvSavePath } from './fileDialogs';
import { DashboardCenter } from './DashboardCenter';
import { FinalLeadCenter } from './FinalLeadCenter';
import { workbenchApi } from './workbenchApi';
import type { WorkbenchController } from './useWorkbenchController';

type ModuleShellProps = {
  c: WorkbenchController;
  moduleId: string;
  title: string;
  description: string;
  chartTitle?: string;
  chartKind?: DashboardChartGroup['kind'];
  loadMetrics: () => Promise<MetricCard[] | DashboardOverview>;
  useFinalLeadCenter?: boolean;
};

function safeFilenamePart(value: string, fallback: string) {
  const cleaned = value.trim().replace(/[<>:"/\\|?*\x00-\x1F]/g, '_').replace(/\s+/g, '_').replace(/\.+$/g, '');
  return cleaned || fallback;
}

function timestampPart() {
  const now = new Date();
  const pad = (value: number) => String(value).padStart(2, '0');
  return `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
}

function buildExportName(moduleId: string, analysisRunId: string, batchDisplayName: string, importBatchId: string) {
  const batch = safeFilenamePart(batchDisplayName || importBatchId, 'batch');
  const runId = safeFilenamePart(analysisRunId.trim() || 'RUN_MANUAL', 'RUN_MANUAL');
  return `${batch}_${runId}_${safeFilenamePart(moduleId, 'module')}_${timestampPart()}.csv`;
}

function ModuleShell({ c, moduleId, title, description, chartTitle, chartKind = 'bar', loadMetrics, useFinalLeadCenter }: ModuleShellProps) {
  async function refresh() {
    if (moduleId === 'overview') {
      const result = await c.runAction(`${moduleId}_refresh`, loadMetrics);
      if (result && typeof result === 'object' && 'metrics' in result) {
        c.setOverview(result as DashboardOverview);
        c.setDashboardCharts([]);
      }
      return;
    }
    const result = await c.loadMetrics(`${moduleId}_refresh`, loadMetrics as () => Promise<MetricCard[]>);
    c.setDashboardCharts([{ title: chartTitle ?? title, kind: chartKind, metrics: result }]);
  }

  async function exportModule() {
    const selected = await selectCsvSavePath(buildExportName(moduleId, c.analysisRunId, c.batchDisplayName || c.batch?.batch_display_name || '', c.importBatchId));
    if (!selected) return;
    await c.runAction(`${moduleId}_export_csv`, () => workbenchApi.exportModule(c.effectiveSettings, c.importBatchId, c.analysisRunId || undefined, moduleId, selected));
  }

  return (
    <section className="panel form-panel step-card">
      <div className="step-card-head">
        <div>
          <h2>{title}</h2>
          <p className="hero-text">{description}</p>
        </div>
        <span className="step-badge">{moduleId}</span>
      </div>
      <div className="action-row">
        <button type="button" onClick={refresh}>刷新</button>
        <button type="button" onClick={exportModule}>导出模块 CSV</button>
      </div>
      {useFinalLeadCenter ? <FinalLeadCenter {...c} /> : <DashboardCenter {...c} activeModuleTitle={title} />}
    </section>
  );
}

export function OverviewDashboard({ c }: { c: WorkbenchController }) {
  return <ModuleShell c={c} moduleId="overview" title="OverviewDashboard" description="用户、流量、时长、接入类型和 Lead 总体结论。" chartTitle="Overview" loadMetrics={() => workbenchApi.overview(c.effectiveSettings, c.importBatchId, c.analysisRunId)} />;
}

export function AppUsageDashboard({ c }: { c: WorkbenchController }) {
  return <ModuleShell c={c} moduleId="app_usage" title="AppUsageDashboard" description="按应用分类查看活跃用户、使用时长、流量和 Top App。" chartTitle="App Category" loadMetrics={() => workbenchApi.appCategory(c.effectiveSettings, c.importBatchId, c.analysisRunId)} />;
}

export function VideoExperienceDashboard({ c }: { c: WorkbenchController }) {
  return <ModuleShell c={c} moduleId="video_experience" title="VideoExperienceDashboard" description="围绕 Universal Video / OTT / Short Video 分析 VMOS、速率和卡顿。" chartTitle="Video Experience Detail" chartKind="radar" loadMetrics={() => workbenchApi.videoDetail(c.effectiveSettings, c.importBatchId, c.analysisRunId)} />;
}

export function GameExperienceDashboard({ c }: { c: WorkbenchController }) {
  return <ModuleShell c={c} moduleId="game_experience" title="GameExperienceDashboard" description="围绕 Game 应用分析 MOS、RTT、jitter 和游戏时长。" chartTitle="Game Experience" loadMetrics={() => workbenchApi.gameExperience(c.effectiveSettings, c.importBatchId, c.analysisRunId)} />;
}

export function NetworkQualityDashboard({ c }: { c: WorkbenchController }) {
  return <ModuleShell c={c} moduleId="network_quality" title="NetworkQualityDashboard" description="RTT、PLR、MOS / VMOS、Wi-Fi delay 和瓶颈侧拆分。" chartTitle="Network Quality" chartKind="radar" loadMetrics={() => workbenchApi.networkQuality(c.effectiveSettings, c.importBatchId, c.analysisRunId)} />;
}

export function CableFiberCompareDashboard({ c }: { c: WorkbenchController }) {
  return <ModuleShell c={c} moduleId="cable_fiber_compare" title="CableFiberCompareDashboard" description="按接入类型比较忙时速率、时延、丢包和体验评分。" chartTitle="Cable vs FTTH" loadMetrics={() => workbenchApi.cableFiber(c.effectiveSettings, c.importBatchId, c.analysisRunId)} />;
}

export function MigrationLeadDashboard({ c }: { c: WorkbenchController }) {
  return <ModuleShell c={c} moduleId="migration_lead" title="MigrationLeadDashboard" description="Lead Type、Final Action、推荐套餐和可导出营销清单。" chartTitle="Final Action Mix" loadMetrics={() => workbenchApi.leadSummary(c.effectiveSettings, c.importBatchId, c.analysisRunId)} useFinalLeadCenter />;
}

export function UserProfileDashboard({ c }: { c: WorkbenchController }) {
  return <ModuleShell c={c} moduleId="user_profile" title="UserProfileDashboard" description="按用户查看应用偏好、体验指标、Lead Type 和推荐动作。" chartTitle="User Profile" loadMetrics={() => workbenchApi.userProfile(c.effectiveSettings, c.importBatchId, c.analysisRunId)} />;
}

import { useEffect, useMemo, useState } from 'react';
import type { DuckDbAccessHourlyRow, DuckDbAccessSummaryRow, DuckDbAnalysisRunItem, DuckDbBatchListItem } from '../../shared/types';
import type { WorkbenchController } from './useWorkbenchController';
import { workbenchApi } from './workbenchApi';

type DuckDbView = 'panorama' | 'quality' | 'access' | 'opportunities';

function value(value: number | undefined, digits = 2) {
  return value == null || !Number.isFinite(value) ? '—' : value.toLocaleString('zh-CN', { maximumFractionDigits: digits });
}

export function DuckDbAnalysisWorkspace({ c, activeView, onOpenData }: { c: WorkbenchController; activeView: DuckDbView; onOpenData: () => void }) {
  const [batches, setBatches] = useState<DuckDbBatchListItem[]>([]);
  const [runs, setRuns] = useState<DuckDbAnalysisRunItem[]>([]);
  const [summary, setSummary] = useState<DuckDbAccessSummaryRow[]>([]);
  const [hourly, setHourly] = useState<DuckDbAccessHourlyRow[]>([]);
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const workspaceDir = c.duckDbWorkspaceDir;

  async function refreshBatches() {
    if (!workspaceDir) {
      setBatches([]);
      setMessage(c.duckDbWorkspaceError || '正在准备应用本地数据目录…');
      return;
    }
    setBusy(true);
    try {
      const next = await workbenchApi.listDuckDbBatches({ workspace_dir: workspaceDir });
      setBatches(next);
      const persistedBatch = next.find((item) => item.import_batch_id === c.importBatchId);
      if (c.importBatchId && !persistedBatch) {
        c.setImportBatchId('');
        c.setAnalysisRunId('');
        c.setBatchDisplayName('');
        setRuns([]);
        setSummary([]);
        setHourly([]);
      } else if (persistedBatch) {
        const nextRuns = await workbenchApi.listDuckDbAnalysisRuns({ workspace_dir: workspaceDir }, persistedBatch.import_batch_id);
        setRuns(nextRuns);
        if (!nextRuns.some((run) => run.analysis_run_id === c.analysisRunId)) {
          const ready = nextRuns.find((run) => run.status === 'success') || nextRuns[0];
          c.setAnalysisRunId(ready?.analysis_run_id || '');
        }
      }
      setMessage(next.length ? `已找到 ${next.length} 个本地分析批次。` : '还没有本地分析结果；请前往数据中心选择 CSV。旧 MySQL 批次不会显示在这里。');
    } catch (error) {
      setBatches([]);
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    if (c.duckDbSessionAuthorized) void refreshBatches();
  }, [workspaceDir, c.duckDbSessionAuthorized]);

  function authorizeAndRefresh() {
    if (c.duckDbSessionAuthorized) {
      void refreshBatches();
    } else {
      c.authorizeDuckDbSession();
    }
  }

  async function selectBatch(batch: DuckDbBatchListItem) {
    c.setImportBatchId(batch.import_batch_id);
    c.setBatchDisplayName(batch.batch_display_name || batch.source_file_name);
    setSummary([]);
    setHourly([]);
    const nextRuns = await workbenchApi.listDuckDbAnalysisRuns({ workspace_dir: workspaceDir }, batch.import_batch_id);
    setRuns(nextRuns);
    const run = nextRuns.find((item) => item.status === 'success') || nextRuns[0];
    c.setAnalysisRunId(run?.analysis_run_id || '');
  }

  async function loadAccess() {
    if (!workspaceDir || !c.importBatchId || !c.analysisRunId) return;
    setBusy(true);
    setMessage('正在读取 DuckDB 已发布结果…');
    try {
      const [nextSummary, nextHourly] = await Promise.all([
        workbenchApi.duckDbAccessSummary({ workspace_dir: workspaceDir }, c.importBatchId, c.analysisRunId),
        workbenchApi.duckDbAccessHourly({ workspace_dir: workspaceDir }, c.importBatchId, c.analysisRunId),
      ]);
      setSummary(nextSummary);
      setHourly(nextHourly);
      setMessage(nextSummary.length ? '已读取本地 DuckDB 发布结果。' : '该运行没有 Access 发布结果。');
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  const maxTraffic = useMemo(() => Math.max(1, ...summary.map((row) => row.downloaded_gb || 0)), [summary]);

  if (activeView === 'quality' || activeView === 'opportunities') {
    return <section className="workbench-section-stack">
      <article className="panel form-panel"><span className="step-badge">DuckDB 2.0.0-3</span><h2>{activeView === 'quality' ? '质差分析' : '潜客机会'}</h2>
        <p className="hero-text">该模块尚未迁移到 DuckDB 发布契约。为防止混合数据，本页不会回退查询 MySQL。</p>
        <button type="button" onClick={onOpenData}>前往 DuckDB 数据中心</button>
      </article>
    </section>;
  }

  return <section className="workbench-section-stack analysis-workspace">
    <article className="panel form-panel">
      <div className="step-card-head"><div><span className="step-badge">DuckDB 本地运行时</span><h2>{activeView === 'access' ? 'Cable / FTTH' : '全景洞察'}</h2><p className="hero-text">显示程序自动保存的本地分析历史和发布结果，不读取 MySQL。启动时不会自动打开历史数据库。</p></div><button type="button" onClick={() => void authorizeAndRefresh()} disabled={busy}>刷新历史分析</button></div>
      <div className="form-grid">
        <select value={c.importBatchId} onChange={(event) => { const selected = batches.find((item) => item.import_batch_id === event.target.value); if (selected) void selectBatch(selected); }}>
          <option value="">{batches.length ? '选择本地分析批次' : '暂无本地分析批次'}</option>
          {batches.map((batch) => <option key={batch.import_batch_id} value={batch.import_batch_id}>{batch.batch_display_name || batch.source_file_name} · {batch.status}</option>)}
        </select>
        <select value={c.analysisRunId} disabled={!runs.length} onChange={(event) => c.setAnalysisRunId(event.target.value)}>
          <option value="">选择分析运行</option>{runs.map((run) => <option key={run.analysis_run_id} value={run.analysis_run_id}>{run.analysis_run_id} · {run.status}</option>)}
        </select>
        <button className="action-button-primary" type="button" disabled={busy || !c.analysisRunId} onClick={() => void loadAccess()}>加载 DuckDB 结果</button>
        <button type="button" onClick={onOpenData}>导入 CSV</button>
      </div>
      {message && <p className="muted-row">{message}</p>}
    </article>
    {summary.length > 0 && <>
      <div className="metric-grid compact-metric-grid">
        {summary.map((row) => <article className="metric-card" key={row.access_type}><span>{row.access_type} 活跃用户</span><strong>{value(row.active_users, 0)}</strong><small>{value(row.downloaded_gb)} GB · {value(row.avg_effective_download_mbps)} Mbps</small></article>)}
      </div>
      <article className="panel form-panel"><h2>Cable / FTTH 指标对比</h2><p className="hero-text">柱长按各接入类型流量相对值显示；数值保持真实单位。</p>
        {summary.map((row) => <div className="distribution-row" key={row.access_type}><strong>{row.access_type}</strong><div><i style={{ width: `${((row.downloaded_gb || 0) / maxTraffic) * 100}%` }} /></div><span>{value(row.downloaded_gb)} GB</span></div>)}
        <div className="decision-table-wrap"><table className="decision-table"><thead><tr><th>接入类型</th><th>用户</th><th>样本</th><th>流量 GB</th><th>有效下载 Mbps</th><th>RTT ms</th><th>用户丢包 %</th><th>网络丢包 %</th><th>vMOS</th></tr></thead><tbody>{summary.map((row) => <tr key={row.access_type}><td>{row.access_type}</td><td>{value(row.active_users, 0)}</td><td>{value(row.observation_rows, 0)}</td><td>{value(row.downloaded_gb)}</td><td>{value(row.avg_effective_download_mbps)}</td><td>{value(row.avg_rtt_ms)}</td><td>{value(row.avg_user_loss_pct)}</td><td>{value(row.avg_network_loss_pct)}</td><td>{value(row.avg_vmos)}</td></tr>)}</tbody></table></div>
      </article>
      <article className="panel form-panel"><h2>小时聚合</h2><p className="hero-text">当前运行共 {hourly.length} 条日期 × 小时 × 接入类型记录。</p><div className="decision-table-wrap"><table className="decision-table"><thead><tr><th>日期</th><th>小时</th><th>接入类型</th><th>活跃用户</th><th>流量 GB</th><th>有效下载 Mbps</th><th>RTT ms</th></tr></thead><tbody>{hourly.slice(0, 240).map((row) => <tr key={`${row.stat_date}-${row.hour_of_day}-${row.access_type}`}><td>{row.stat_date}</td><td>{String(row.hour_of_day).padStart(2, '0')}:00</td><td>{row.access_type}</td><td>{value(row.active_users, 0)}</td><td>{value(row.downloaded_gb)}</td><td>{value(row.avg_effective_download_mbps)}</td><td>{value(row.avg_rtt_ms)}</td></tr>)}</tbody></table></div></article>
    </>}
  </section>;
}

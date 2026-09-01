import { useEffect, useMemo, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import type { DuckDbPocResult, DuckDbWorkspaceStatus } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

const WORKSPACE_STORAGE_KEY = 'sa-fbb-duckdb-workspace';

function defaultWorkspace() {
  return localStorage.getItem(WORKSPACE_STORAGE_KEY) || '';
}

function formatInteger(value: number) {
  return new Intl.NumberFormat('zh-CN').format(value);
}

export function DuckDbWorkspacePanel() {
  const [workspaceDir, setWorkspaceDir] = useState(defaultWorkspace);
  const [filePath, setFilePath] = useState('');
  const [batchName, setBatchName] = useState('DuckDB 性能验证');
  const [defaultAccessType, setDefaultAccessType] = useState<'CABLE' | 'FTTH' | 'OTHER'>('CABLE');
  const [ftthRanges, setFtthRanges] = useState('');
  const [status, setStatus] = useState<DuckDbWorkspaceStatus>();
  const [result, setResult] = useState<DuckDbPocResult>();
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');

  const normalizedRanges = useMemo(() => ftthRanges
    .split(/[\n,;]/)
    .map((value) => value.trim())
    .filter(Boolean), [ftthRanges]);

  useEffect(() => {
    if (workspaceDir) localStorage.setItem(WORKSPACE_STORAGE_KEY, workspaceDir);
  }, [workspaceDir]);

  useEffect(() => {
    if (!busy || !workspaceDir.trim()) return undefined;
    const timer = window.setInterval(() => {
      workbenchApi.duckDbWorkspaceStatus({ workspace_dir: workspaceDir.trim() })
        .then(setStatus)
        .catch(() => undefined);
    }, 1500);
    return () => window.clearInterval(timer);
  }, [busy, workspaceDir]);

  async function chooseWorkspace() {
    const selected = await open({ directory: true, multiple: false, title: '选择 DuckDB 工作区目录' });
    if (typeof selected === 'string') setWorkspaceDir(selected);
  }

  async function chooseCsv() {
    const selected = await open({ directory: false, multiple: false, filters: [{ name: 'CSV', extensions: ['csv', 'txt'] }], title: '选择 TCP / 视频 CSV' });
    if (typeof selected === 'string') setFilePath(selected);
  }

  async function initializeWorkspace() {
    if (!workspaceDir.trim()) {
      setMessage('请先选择工作区目录。');
      return;
    }
    setBusy(true);
    setMessage('正在初始化 workspace.duckdb…');
    try {
      const next = await workbenchApi.initializeDuckDbWorkspace({ workspace_dir: workspaceDir.trim() });
      setStatus(next);
      setMessage(`工作区已就绪：${next.database_path}`);
    } catch (error) {
      setMessage(String(error));
    } finally {
      setBusy(false);
    }
  }

  async function analyzeCsv() {
    if (!workspaceDir.trim() || !filePath.trim()) {
      setMessage('请先选择工作区和 CSV。');
      return;
    }
    setBusy(true);
    setResult(undefined);
    setMessage('正在计算 SHA-256、生成 Parquet，并执行小时聚合；窗口可继续响应…');
    try {
      const next = await workbenchApi.analyzeCsvWithDuckDb({
        workspace_dir: workspaceDir.trim(),
        file_path: filePath.trim(),
        data_type: 'tcp',
        batch_display_name: batchName.trim() || undefined,
        default_access_type: defaultAccessType,
        ftth_ranges: normalizedRanges,
      });
      setResult(next);
      const nextStatus = await workbenchApi.duckDbWorkspaceStatus({ workspace_dir: workspaceDir.trim() });
      setStatus(nextStatus);
      setMessage(`分析完成，耗时 ${(next.elapsed_ms / 1000).toFixed(1)} 秒。`);
    } catch (error) {
      setMessage(String(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="panel form-panel step-card duckdb-workspace-panel">
      <div className="step-card-head">
        <div>
          <h2>DuckDB + Parquet 本地分析（2.0 预览）</h2>
          <p className="hero-text">CSV 只作为源证据；清洗明细写入分区 Parquet，小时聚合和发布状态保存在单个 workspace.duckdb。无需安装 MySQL。</p>
        </div>
        <span className="step-badge">推荐</span>
      </div>

      <div className="form-grid">
        <input value={workspaceDir} onChange={(event) => setWorkspaceDir(event.target.value)} placeholder="工作区目录" />
        <button type="button" onClick={chooseWorkspace} disabled={busy}>选择工作区</button>
        <input value={filePath} onChange={(event) => setFilePath(event.target.value)} placeholder="TCP / 视频 CSV 路径" />
        <button type="button" onClick={chooseCsv} disabled={busy}>选择 CSV</button>
        <input value={batchName} onChange={(event) => setBatchName(event.target.value)} placeholder="批次名称" />
        <select value={defaultAccessType} onChange={(event) => setDefaultAccessType(event.target.value as 'CABLE' | 'FTTH' | 'OTHER')}>
          <option value="CABLE">未命中规则默认 Cable</option>
          <option value="FTTH">未命中规则默认 FTTH</option>
          <option value="OTHER">未命中规则默认 Other</option>
        </select>
      </div>
      <label className="field-stack">
        <span>FTTH IP 范围（可选，每行一个 CIDR、IP 或起止范围）</span>
        <textarea rows={3} value={ftthRanges} onChange={(event) => setFtthRanges(event.target.value)} placeholder={'10.20.0.0/16\n172.16.1.1-172.16.1.254'} />
      </label>
      <div className="primary-action-row">
        <button type="button" onClick={initializeWorkspace} disabled={busy}>初始化 / 检查工作区</button>
        <button type="button" className="primary" onClick={analyzeCsv} disabled={busy}>{busy ? '处理中…' : '运行 DuckDB 分析 POC'}</button>
      </div>

      {message && <p className="muted-row">{message}</p>}
      {status && (
        <div className="persistence-grid">
          <span>DuckDB：{status.duckdb_version || '未初始化'}</span>
          <span>批次 / 运行：{status.batch_count} / {status.run_count}</span>
          <span>仍在运行：{status.running_run_count}（正常完成或失败后应为 0）</span>
          <span>最新步骤：{status.latest_run_step || '—'} · {status.latest_run_status || '—'}</span>
          {status.latest_run_message && <span>{status.latest_run_message}</span>}
        </div>
      )}
      {result && (
        <div className="metric-grid compact-metric-grid">
          {result.metrics.map((metric) => (
            <article className="metric-card" key={metric.label}>
              <span>{metric.label}</span>
              <strong>{metric.label.includes('行数') || metric.label.includes('活跃用户') ? formatInteger(Number(metric.value)) : metric.value}</strong>
              <small>{metric.hint}</small>
            </article>
          ))}
          <article className="metric-card">
            <span>运行标识</span>
            <strong>{result.analysis_run_id}</strong>
            <small>{result.parquet_path}</small>
          </article>
        </div>
      )}
    </section>
  );
}

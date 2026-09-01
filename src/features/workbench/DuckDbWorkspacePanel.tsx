import { useEffect, useMemo, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import type { DuckDbPocResult, DuckDbWorkspaceStatus } from '../../shared/types';
import { workbenchApi } from './workbenchApi';
import type { WorkbenchController } from './useWorkbenchController';

function formatInteger(value: number) {
  return new Intl.NumberFormat('zh-CN').format(value);
}

function fileName(path: string) {
  return path.split(/[\\/]/).filter(Boolean).pop() || '';
}

function batchNameFromPath(path: string) {
  return fileName(path).replace(/\.(csv|txt)$/i, '') || '本地 CSV 分析';
}

export function DuckDbWorkspacePanel({ c, onOpenAnalysis }: { c: WorkbenchController; onOpenAnalysis?: () => void }) {
  const [defaultAccessType, setDefaultAccessType] = useState<'CABLE' | 'FTTH' | 'OTHER'>('CABLE');
  const [ftthRanges, setFtthRanges] = useState('');
  const [status, setStatus] = useState<DuckDbWorkspaceStatus>();
  const [result, setResult] = useState<DuckDbPocResult>();
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const workspaceDir = c.duckDbWorkspaceDir;
  const filePath = c.filePath;

  const normalizedRanges = useMemo(() => ftthRanges
    .split(/[\n,;]/)
    .map((value) => value.trim())
    .filter(Boolean), [ftthRanges]);

  useEffect(() => {
    if (!workspaceDir) return;
    workbenchApi.duckDbWorkspaceStatus({ workspace_dir: workspaceDir })
      .then(setStatus)
      .catch((error) => setMessage(error instanceof Error ? error.message : String(error)));
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

  async function chooseCsv() {
    const selected = await open({ directory: false, multiple: false, filters: [{ name: 'CSV', extensions: ['csv', 'txt'] }], title: '选择 TCP / 视频 CSV' });
    if (typeof selected === 'string') {
      c.setFilePath(selected);
      setResult(undefined);
      setMessage(`已选择 ${fileName(selected)}；确认高级选项后点击“开始本地分析”。`);
    }
  }

  async function analyzeCsv() {
    if (!workspaceDir.trim()) {
      setMessage(c.duckDbWorkspaceError || '程序尚未准备好本地数据目录，请稍后重试。');
      return;
    }
    if (!filePath.trim()) {
      setMessage('请先选择 CSV 文件。');
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
        batch_display_name: batchNameFromPath(filePath),
        default_access_type: defaultAccessType,
        ftth_ranges: normalizedRanges,
      });
      setResult(next);
      c.authorizeDuckDbSession();
      c.setImportBatchId(next.import_batch_id);
      c.setAnalysisRunId(next.analysis_run_id);
      c.setBatchDisplayName(batchNameFromPath(filePath));
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
          <h2>选择 CSV 开始本地分析</h2>
          <p className="hero-text">只需选择 TCP / 视频 CSV。程序会自动管理 DuckDB、分区 Parquet、历史批次和分析结果，无需选择数据库目录，也无需安装 MySQL。</p>
        </div>
        <span className="step-badge">默认运行时</span>
      </div>

      <div className="file-picker-card">
        <div>
          <span>待分析文件</span>
          <strong>{filePath ? fileName(filePath) : '尚未选择 CSV'}</strong>
          <small>{filePath || '支持 .csv 和 .txt；分析不会把整个文件载入内存。'}</small>
        </div>
        <button type="button" onClick={chooseCsv} disabled={busy}>选择 CSV 文件</button>
      </div>

      <details className="advanced-actions">
        <summary>高级选项：Cable / FTTH 识别规则</summary>
        <div className="form-grid" style={{ marginTop: 12 }}>
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
      </details>
      <div className="primary-action-row">
        <button type="button" className="action-button-primary" onClick={analyzeCsv} disabled={busy || !filePath || !workspaceDir}>{busy ? '处理中…' : '开始本地分析'}</button>
        {result && onOpenAnalysis && <button type="button" onClick={onOpenAnalysis}>查看分析结果</button>}
      </div>

      {!workspaceDir && <p className="muted-row">{c.duckDbWorkspaceError || '正在准备应用本地数据目录…'}</p>}
      {message && <p className="muted-row">{message}</p>}
      {status && (
        <details className="advanced-actions">
          <summary>存储与运行详情</summary>
          <div className="persistence-grid">
            <span>本地数据库：{status.duckdb_version || '首次分析时创建'}</span>
            <span>历史批次 / 运行：{status.batch_count} / {status.run_count}</span>
            <span>仍在运行：{status.running_run_count}（正常完成或失败后应为 0）</span>
            <span>最新步骤：{status.latest_run_step || '—'} · {status.latest_run_status || '—'}</span>
            <span>内部存储：{status.database_path}</span>
            {status.latest_run_message && <span>{status.latest_run_message}</span>}
          </div>
        </details>
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

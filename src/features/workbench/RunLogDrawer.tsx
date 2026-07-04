import type { ExecutionLogEntry } from '../../shared/types';
import { ExecutionLog } from './ExecutionLog';

type Props = {
  open: boolean;
  log: ExecutionLogEntry[];
  onClose: () => void;
};

export function RunLogDrawer({ open, log, onClose }: Props) {
  if (!open) return null;
  return (
    <div className="run-log-drawer-backdrop" role="presentation">
      <aside className="run-log-drawer" aria-label="Execution log drawer">
        <div className="run-log-drawer-head">
          <div>
            <h2>执行日志</h2>
            <p className="muted-row">主流程只显示结论；这里保留完整技术明细。</p>
          </div>
          <button type="button" onClick={onClose}>关闭</button>
        </div>
        <ExecutionLog log={log} />
      </aside>
    </div>
  );
}

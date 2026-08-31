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
    <div className="run-log-drawer-backdrop" role="presentation" onClick={onClose}>
      <aside className="run-log-drawer" aria-label="Execution log drawer" onClick={(event) => event.stopPropagation()}>
        <div className="run-log-drawer-head">
          <div>
            <h2>执行日志</h2>
            <p>命令、耗时、错误和返回摘要</p>
          </div>
          <button type="button" className="run-log-drawer-close" onClick={onClose} aria-label="关闭执行日志">关闭</button>
        </div>
        <ExecutionLog log={log} compact />
      </aside>
    </div>
  );
}

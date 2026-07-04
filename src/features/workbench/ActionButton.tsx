import type { ActionState } from '../../shared/types';

type Props = {
  actionKey: string;
  actionStates: Record<string, ActionState>;
  label: string;
  onClick: () => Promise<unknown> | void;
  disabled?: boolean;
  primary?: boolean;
  title?: string;
};

function statusLabel(state?: ActionState) {
  if (!state) return 'idle';
  if (state.status === 'running') return 'running';
  if (state.status === 'success') return 'done';
  if (state.status === 'failure') return 'failed';
  return 'idle';
}

export function ActionButton({ actionKey, actionStates, label, onClick, disabled, primary, title }: Props) {
  const state = actionStates[actionKey];
  const running = state?.status === 'running';
  const blocked = Boolean(disabled || running);
  const suffix = state?.status === 'running' ? '…' : state?.status === 'success' ? ' ✓' : state?.status === 'failure' ? ' ×' : '';
  return (
    <button type="button" className={`action-button ${primary ? 'action-button-primary' : ''} action-button-${statusLabel(state)}`} disabled={blocked} title={title ?? state?.message} onClick={() => { void onClick(); }}>
      <span>{label}{suffix}</span>
      {state?.duration_ms !== undefined && state.status !== 'running' && <small>{state.duration_ms} ms</small>}
    </button>
  );
}

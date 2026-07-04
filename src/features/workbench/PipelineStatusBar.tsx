import type { PipelineStepStatus } from '../../shared/types';

export type PipelineStepId = 'start' | 'import' | 'validate' | 'analyze' | 'results';

export type PipelineStep = {
  id: PipelineStepId;
  label: string;
  hint: string;
  status: PipelineStepStatus;
};

type Props = {
  steps: PipelineStep[];
  activeStep: PipelineStepId;
  onSelect: (step: PipelineStepId) => void;
};

const statusMark: Record<PipelineStepStatus, string> = {
  not_started: '○',
  running: '…',
  success: '✓',
  warning: '!',
  failed: '×',
};

export function PipelineStatusBar({ steps, activeStep, onSelect }: Props) {
  return (
    <section className="pipeline-status-bar" aria-label="Pipeline status">
      {steps.map((step, index) => (
        <button
          key={step.id}
          type="button"
          className={`pipeline-step pipeline-step-${step.status} ${activeStep === step.id ? 'is-active' : ''}`}
          onClick={() => onSelect(step.id)}
        >
          <span className="pipeline-step-index">{index + 1}</span>
          <span className="pipeline-step-body">
            <strong>{statusMark[step.status]} {step.label}</strong>
            <small>{step.hint}</small>
          </span>
        </button>
      ))}
    </section>
  );
}

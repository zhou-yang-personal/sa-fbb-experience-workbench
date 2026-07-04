import { WorkbenchAppV2 } from './WorkbenchAppV2';

/**
 * Legacy compatibility wrapper.
 *
 * `WorkbenchAppV2` is the active workbench shell mounted by `src/main.tsx`.
 * This component remains only to avoid breaking stale imports while preventing
 * the pre-V2 duplicated UI from drifting behind current workflow behavior.
 */
export function WorkbenchApp() {
  return <WorkbenchAppV2 />;
}

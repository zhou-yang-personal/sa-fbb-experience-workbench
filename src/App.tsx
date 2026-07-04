import { WorkbenchAppV2 } from './features/workbench/WorkbenchAppV2';

/**
 * Legacy compatibility entry.
 *
 * The active application entry is `src/main.tsx`, which mounts `WorkbenchAppV2`
 * directly. Keep this default export only for tooling or tests that still import
 * `src/App.tsx`; do not add new workflow logic here.
 */
export default function App() {
  return <WorkbenchAppV2 />;
}

type LegacyOpsPanelProps = Record<string, unknown>;

/**
 * Legacy placeholder.
 *
 * The current workflow is split into `QualityCenter`, `EtlJobCenter`,
 * `DashboardCenter` and `FinalLeadCenter` inside `WorkbenchAppV2`.
 * Keep this component only for stale imports; do not wire new operations here.
 */
export function OpsPanel(_props: LegacyOpsPanelProps) {
  return (
    <section className="panel form-panel empty-panel">
      <h2>Legacy Operations Panel</h2>
      <p>Operations are now handled by the V2 workbench centers.</p>
    </section>
  );
}

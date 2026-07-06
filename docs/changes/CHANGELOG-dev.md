# CHANGELOG-dev

## 1.0.16 - 2026-07-05

### Added

- Added `AnalysisWorkspace` as the default product entry for batch-first dashboard analysis.
- Added `SystemPanel` to collect database connection, data availability checks, ETL task inspection and diagnostic logs under System Management.
- Added `docs/design/product-function-tree-v0.2.md` to document the product function tree and batch-first constraints.
- Added readable batch naming to frontend workbench context and `meta_import_batch.batch_display_name`.
- Added backend auto-column guard for `meta_import_batch.batch_display_name` when creating or checking import batches on existing local databases.
- Added module readiness definitions for Overview, App Usage, Video Experience, Game Experience, Network Quality, Cable vs FTTH, Migration Leads and User Profile.

### Changed

- Replaced the Start / Import / Validate / Analyze / Results navigation center with product navigation: Data Analysis / Data Import / System Management.
- Made Data Analysis the default landing page.
- Data Import now requires a human-readable batch name before creating or importing a batch.
- Dashboard / Lead export remains inside the relevant dashboard panels; there is no standalone Export module.
- Mapping required-field failure now reports missing target fields and source-header matching detail instead of only a missing count.
- Execution Log is renamed and positioned as Diagnostic Log, with copy-all and copy-failure actions.
- Synchronized package, Cargo, Tauri config, README and handoff version markers to `1.0.16`.

### Not fully implemented

- Per-batch physical RAW / CLEAN / DWS / ADS table creation is not fully switched in this round. 1.0.16 still uses shared physical tables plus `import_batch_id` isolation, with the target-state table naming documented for the next database-mainline refactor.

### Not verified

- ChatGPT GitHub connector cannot run `npm install`, `npm run check`, `npm run build`, `cargo check` or `tauri:build`.
- Real MySQL and CSV end-to-end flow was not executed in this round.

## 1.0.15 - 2026-07-04

### Added

- Added a 5-step guided workbench shell: Start / Import / Validate / Analyze / Results.
- Added `PipelineStatusBar`, `NextActionHint`, `ActionButton`, `RunLogDrawer` and file dialog helpers.
- Added running / success / failure / disabled feedback state for guided actions.
- Added system CSV file picker for the Import main flow.
- Added system save dialog support to export presets.
- Added one-click Import main action that runs CSV probe, batch creation, mapping validation, RAW load, status refresh and profile refresh.
- Added one-click Analyze action that runs clean, aggregate and final fusion as a business flow.
- Added Results page consolidation for dashboard, final lead, metrics and result tables.

### Changed

- Replaced the previous 8-entry module navigation with guided pipeline navigation in `WorkbenchAppV2`.
- Moved Run Log from primary navigation to a drawer entry.
- Kept manual path input only in Import advanced mode.
- Updated ETL, dashboard and lead actions to use the unified guided action button where connector changes were safe.
- Synchronized package, Cargo, Tauri config, README and handoff version markers to `1.0.15`.

### Not verified

- ChatGPT GitHub connector cannot run `npm install`, `npm run check`, `npm run build`, `cargo check` or `tauri:build`.
- Real MySQL and CSV end-to-end flow was not executed in this round.
- `QualityCenter.tsx` and `FinalLeadCenter.tsx` large rewrites were blocked by connector safety checks; partial compatibility changes were made through new components and dependent action components.

## 1.0.14 - 2026-07-04

### Changed

- Tightened Import Center with structured mapping summary, mapping result, dataset profile and mapping catalog rendering.
- Tightened Quality Center with structured quality gate result table, severity counters and empty-state messaging.
- Added `passed=` markers to quality gate result values so frontend can distinguish passed, failed, warning and info rows.
- Kept backend command shape stable while improving display readability for Import / Quality smoke.
- Synchronized package, Cargo, Tauri config, README and handoff version markers to `1.0.14`.

### Verified

- `npm install`
- `npm run check`
- `npm run build`
- `cd src-tauri && cargo check`
- Import page smoke
- Quality page smoke

### Not verified

- Real Tauri desktop interaction.
- Full MySQL import / ETL E2E in this round.

## 1.0.13 - 2026-07-04

### Changed

- Confirmed `src/main.tsx` remains the active React entry and mounts `WorkbenchAppV2` directly.
- Reduced legacy entry drift by turning `src/App.tsx` into a compatibility wrapper that forwards to `WorkbenchAppV2`.
- Reduced legacy shell drift by turning `src/features/workbench/WorkbenchApp.tsx` into a compatibility wrapper that forwards to `WorkbenchAppV2`.
- Converted `OpsPanel` into a legacy placeholder instead of a stale duplicate operations composer.
- Marked `TaskCenter2` as a legacy placeholder.
- Kept legacy files in place instead of deleting them, pending local compile confirmation for stale imports.
- Synchronized Workbench header, package, Cargo, Tauri config, README, handoff and project rules to `1.0.13`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.12 - 2026-07-04

### Added

- Added `DashboardChartKind` and `DashboardChartGroup` frontend types.
- Added reusable `DashboardCharts` component with ECharts bar and radar panels.
- Added Dashboard Center chart rendering for App Category, Experience Quality, Cable vs FTTH and Final Action Mix.
- Added DashboardActions single-chart load buttons and a combined multi-chart load action.
- Added dashboard chart grid, chart card and chart footnote styles.
- Kept chart data source on existing ADS / DWS dashboard commands and Final Lead summary command; no RAW scan path was introduced.
- Synchronized Workbench header, package, Cargo, Tauri config, README, handoff and project rules to `1.0.12`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.11 - 2026-07-04

### Added

- Added backend model `EtlJobStepsRequest` for structured ETL step detail queries.
- Added backend model `EtlJobStepRow` with job_id, job_type, step_name, target_table, status, affected_rows, started_at, finished_at and message.
- Added `etl_get_job_steps` Tauri command and registered it in `main.rs`.
- Added parameterized ETL step detail query with optional job_id, status and limit filters.
- Added frontend `EtlJobStepRow` / `EtlJobStepsQuery` types and `jobApi.jobSteps` wrapper.
- Added ETL Step Detail and Failed Detail actions in `JobStepActions`.
- Added structured ETL Step Detail table to `EtlJobCenter`.
- Synchronized Workbench header, package, Cargo, Tauri config, README, handoff and project rules to `1.0.11`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.10 - 2026-07-04

### Added

- Added optional server-side filters to `LeadsQueryRequest`: `lead_type`, `final_action` and `keyword`.
- Updated `leads_query_users` to apply server-side page / page_size / lead_type / keyword filtering.
- Updated `final_leads_query_users` to apply server-side page / page_size / final_action / keyword filtering.
- Added `final_actions` support to `ExportLeadsRequest` and `export_final_leads_csv` for action-scoped Final Lead CSV export.
- Added frontend Lead Center controls for backend keyword, SA lead_type, Final final_action, page and page size.
- Updated Final Lead export presets so Market Upsell and Reachability presets set both filename and final-action export scope.
- Synchronized Workbench header, package, Cargo, Tauri config, README, handoff and project rules to `1.0.10`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.9 - 2026-07-04

### Added

- Added reusable `PaginationControls` for frontend result tables.
- Enhanced `ResultTables` with SA Lead / Final Lead text filters, type/action filters, pagination and empty states.
- Added Final Lead action summary pills with click-to-filter behavior.
- Added `ExportPresetActions` for SA Lead, Final Lead, Market Upsell and Reachability export filename presets.
- Updated Final Lead Center to expose export presets before executing CSV export.
- Added result table toolbar, pagination, table header and action summary styles.
- Synchronized Workbench header, package, Cargo, Tauri config, README, handoff and project rules to `1.0.9`.

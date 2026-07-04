# CHANGELOG-dev

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

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.

## 1.0.8 - 2026-07-04

### Added

- Added dedicated Quality Center panel for quality action grouping and check-scope explanation.
- Added dedicated ETL Job Center panel for ETL jobs, recent steps, failed steps and execution chain.
- Added dedicated Dashboard Center panel for Overview, App Category, Experience Quality, Cable/FTTH and Final Summary entry points.
- Added dedicated Final Lead Center panel for SA Lead, Final Lead, Final Action Mix and CSV export entry points.
- Rewired `WorkbenchAppV2` from a single operations block into separated Quality / ETL / Dashboard / Final Lead workbench centers.
- Synchronized Workbench header, package, Cargo, README, handoff and project rules to `1.0.8`.

### Blocked / Not completed

- `src-tauri/tauri.conf.json` version update to `1.0.8` remains blocked by platform safety checks.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.7 - 2026-07-04

### Added

- Expanded dataset profile refresh from row count only to row count, source line range, distinct accounts, distinct MACs and type-specific dimensions.
- Added mapping validation result and summary commands.
- Added frontend mapping result API wrapper.
- Added Import Center actions for mapping summary, mapping result, dataset profile refresh and dataset profile view.
- Updated Workbench header, package, Cargo, README, handoff and project rules to `1.0.7`.

### Blocked / Not completed

- `src-tauri/tauri.conf.json` version update to `1.0.7` remains blocked by platform safety checks.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.6 - 2026-07-04

### Added

- Added mapped RAW import implementation: `src-tauri/src/raw_import_v2.rs`.
- Routed RAW import command through mapped import adapter.
- Added CSV mapping validation command and mapping validation result table.
- Added observability schema for import row errors, mapping validation and dataset profile.
- Added dataset profile refresh and query commands, plus frontend profile API and Import Center profile actions.
- Added ETL job step inspection commands and frontend job inspection API wrapper.
- Added quality gate result inspection commands and frontend quality result API wrapper.
- Added modular frontend WorkbenchAppV2 and split connection, import, operations, quality, ETL, dashboard, lead, metric and log components.
- Added ECharts metric bar component for dashboard visualization.
- Synchronized package and Cargo versions to `1.0.6`.

### Blocked / Not completed

- `src-tauri/tauri.conf.json` version update to `1.0.6` was blocked by platform safety checks.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.5 - 2026-07-04

### Added

- Added configurable import mapping schema: `database/migrations/003_mapping_schema.sql`.
- Added `cfg_import_field_mapping` for CSV header to RAW target column mapping.
- Added `cfg_final_join_rule` for configurable CRM / Reachability / Coverage join keys.
- Added default mapping and join seed: `database/seeds/002_default_mapping_seed.sql`.
- Updated database initialization to include the mapping schema and seed.
- Added `config_commands.rs` as the command handler module for mapping and join rule inspection.
- Added `final_fusion.rs` as a configurable final lead fusion SQL builder module.
- Split command handlers into import, ETL, dashboard and lead modules to keep `main.rs` small and maintainable.
- Registered configuration commands in `main.rs`.
- Rewired Final Lead fusion execution to use the configurable fusion builder.
- Exposed configuration APIs in `src/shared/tauriApi.ts`.
- Synchronized package and Cargo versions to `1.0.5`.

### Blocked / Not completed

- `src-tauri/tauri.conf.json` version update was blocked by platform safety checks.
- `src/App.tsx` configuration UI update was blocked by platform safety checks.
- `src-tauri/src/raw_import.rs` configurable alias integration was blocked by platform safety checks.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

# CHANGELOG-dev

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
- Real MySQL and CSV end-to-end flow not executed.

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
- Rewired Final Lead fusion execution to use the configurable final fusion builder.
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

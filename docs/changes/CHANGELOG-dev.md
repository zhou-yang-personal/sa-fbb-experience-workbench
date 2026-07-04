# CHANGELOG-dev

## 1.0.6 - 2026-07-04

### Added

- Added mapped RAW import implementation: `src-tauri/src/raw_import_v2.rs`.
- Routed RAW import command through mapped import adapter.
- Added CSV mapping validation command and mapping validation result table.
- Added observability schema for import row errors, mapping validation and dataset profile.
- Added ETL job step inspection commands and frontend job inspection API wrapper.
- Added quality gate result inspection commands and frontend quality result API wrapper.
- Added modular frontend WorkbenchAppV2 and split connection, import, operations, quality, ETL, dashboard, lead, metric and log components.
- Added ECharts metric bar component for dashboard visualization.
- Synchronized package and Cargo versions to `1.0.6`.

### Blocked / Not completed

- `src-tauri/tauri.conf.json` version update to `1.0.6` was blocked by platform safety checks.
- Extra style import in `src/main.tsx` was blocked by platform safety checks; base styles are still used.

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

## 1.0.4 - 2026-07-04

### Fixed

- Added Final Lead query command and frontend list entry.
- Added full paginated Final Lead CSV export command.
- Added final lead frontend type and Rust model.
- Enhanced CRM / coverage / reachability fusion SQL with commercial action separation: identity mapping, blacklist exclusion, arrears check, contract check, reachability fix, coverage/build check and market upsell.
- Synchronized package, Tauri and Cargo versions to `1.0.4`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.3 - 2026-07-04

### Fixed

- Extended Import Center data type support to TCP, Game, CRM, FTTH Coverage and Reachability.
- Added RAW import support for CRM, coverage and reachability CSV files.
- Added trace fields to commercial helper RAW tables: `source_file_name` and `source_line_no`.
- Reworked Lead CSV export from first-page-only to full paginated export.
- Synchronized package, Tauri and Cargo versions to `1.0.3`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.2 - 2026-07-04

### Fixed

- Added import progress updates for streaming INSERT fallback.
- Added `total_rows` and `imported_rows` reconciliation on RAW import completion.
- Expanded quality gate with `csv_vs_raw_rows` check.
- Added `import_get_batch_status` command and frontend action.
- Added `etl_get_recent_jobs` command and frontend action.
- Synchronized package, Tauri and Cargo versions to `1.0.2`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.1 - 2026-07-04

### Fixed

- Reworked RAW CSV import to use explicit TCP / Game field lists for `LOAD DATA LOCAL INFILE`.
- Added streaming INSERT fallback for environments where local infile is disabled.
- Added `raw_import.rs` to isolate RAW import logic from `main.rs`.
- Added `job_runner.rs` to write `meta_etl_job` and `meta_etl_job_step` during ETL execution.
- Routed RAW to CLEAN, base aggregate and Phase commands through job logging.
- Expanded RAW quality gate to cover row count, identity, access mix, time range, active hours, app count and topology UNKNOWN checks.
- Synchronized package, Tauri and Cargo versions to `1.0.1`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 1.0.0 - 2026-07-03

### Added

- Implemented Phase 1-7 complete application baseline on `dev`.
- Added complete app schema extension: metadata, CRM, coverage, reachability, DWS and ADS tables.
- Added RAW quality gate SQL.
- Added complete DWS aggregate SQL.
- Added complete dashboard ADS SQL for Overview, App Category, Experience Quality and Cable vs FTTH.
- Added final CRM / coverage / reachability lead fusion SQL.
- Added Rust phase command handlers for quality, complete DWS, complete ADS, final lead fusion and dashboard queries.
- Registered Phase 1-7 Tauri commands in `src-tauri/src/main.rs`.
- Added frontend `phaseApi` wrapper and exposed Phase 1-7 workflow buttons in the UI.
- Updated package and Tauri app version to `1.0.0`.

### Not verified

- Dependency installation not run in ChatGPT GitHub connector environment.
- Frontend build not run.
- Rust check not run.
- Tauri package build not run.
- Real MySQL and CSV end-to-end flow not executed.

## 0.2.1 - 2026-07-03

### Added

- Implemented the initial usable workflow across steps 1-8.
- Added MySQL connection command and database initialization command.
- Added Rust SQL runner for migration and ETL scripts.
- Added CSV/file probe command entry.
- Added import batch creation and RAW load command path.
- Added RAW quality report command.
- Added RAW to CLEAN and aggregate command wiring.
- Added dashboard overview query command.
- Added migration lead query and CSV export command.
- Rewired frontend from mock-only view to Tauri command workflow UI.
- Updated version files to `0.2.1`.

## 0.1.0 - 2026-07-03

### Added

- Created `dev` branch from the architecture design branch.
- Added detailed architecture design: `docs/design/current-core-design.md`.
- Initialized React + TypeScript + Vite frontend skeleton.
- Added MySQL core layered schema baseline.

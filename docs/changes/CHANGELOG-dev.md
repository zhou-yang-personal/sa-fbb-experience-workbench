# CHANGELOG-dev

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
- Initialized Tauri 2 / Rust command skeleton.
- Added MySQL core layered schema baseline.
- Added TCP and Game RAW to CLEAN SQL templates.
- Added DWS user daily aggregate SQL template.
- Added migration lead ADS SQL template.
- Added README, requirements and handoff baseline documents.

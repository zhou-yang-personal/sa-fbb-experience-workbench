# CHANGELOG-dev

## 1.0.22 - 2026-07-06

### Changed

- Extended TCP / Game CLEAN timestamp guards to accept one- or two-digit day/month slash dates such as `20/9/2025 23:58:06` and `1/9/2025 03:05:00`.
- Normalized tab, LF, CR and NBSP in CLEAN timestamp text to spaces instead of deleting them, then compressed repeated whitespace before guarded `STR_TO_DATE`.
- Made `migration_lead` module readiness depend on base SA Lead results in `ads_migration_lead_user`; Final Lead readiness is reported as ready or degraded without disabling the base module.
- Changed module-level `migration_lead` CSV export to use SA Lead summary results so Final Lead degradation does not make the module export empty by default.
- Synchronized package, Cargo, Tauri config, README, handoff, header and mapping catalog version markers to `1.0.22`.

### Fixed

- Fixed valid SA timestamps with single-digit month/day being classified as `WARN_INVALID_STAT_TIME`.
- Fixed middle tab/CR/LF in timestamp text being removed and joining date/time into an unparsable value.
- Fixed Final Lead missing/empty rows making the whole Migration Lead module appear unavailable even when SA Lead results exist.

### Verification

- `npm run check` passed.
- `npm run build` passed; Vite reported the existing large chunk warning.
- `cd src-tauri && cargo check` passed with existing dead_code warnings.
- `cd src-tauri && cargo test -- --nocapture` passed: 18 tests passed.
- `npm run tauri:build` passed and produced ignored local Linux bundles under `src-tauri/target`.
- SQL-level tests verify CLEAN templates normalize `CHAR(9)`, `CHAR(10)`, `CHAR(13)` and `CHAR(160)` to spaces, use one- or two-digit slash date guards, and do not call `STR_TO_DATE` on raw timestamp fields.
- Real MySQL / customer CSV smoke has not been executed yet.

## 1.0.21 - 2026-07-06

### Changed

- Kept the top-level product navigation to three entries and made Data Import the default workflow entry.
- Reworked Data Import into an eight-step visible workflow from CSV preparation through RAW load, Quality Gate, CLEAN/DWD, DWS/ADS and Module Ready.
- Moved Quality Gate, RAW to CLEAN, DWS/ADS and Module Ready into the Data Import main workflow while keeping system-side entries as advanced diagnostics.
- Clean jobs now route by batch `data_type`: tcp batches run only `tcp_raw_to_clean`, game batches run only `game_raw_to_clean`, and auxiliary batch types record skipped/not-applicable instead of failing.
- Module status text now distinguishes missing table, rows=0 result not generated, current analysis_run_id without rows, not-applicable data_type and missing fields.
- Synchronized package, Cargo, Tauri config, README, handoff and header version markers to `1.0.21`.

### Fixed

- Fixed MySQL ERROR 1411 during CLEAN when `statistics_duration` / `statistical_time` contains trailing tab, CR, LF or NBSP.
- Guarded `STR_TO_DATE` calls by first creating cleaned `stat_time_text`, then calling date parsing only for supported timestamp patterns; invalid values become `WARN_INVALID_STAT_TIME` instead of aborting the clean job.

### Verification

- `npm run check` passed.
- `npm run build` passed; Vite reported the existing large chunk warning.
- `cd src-tauri && cargo check` passed with existing dead_code warnings.
- `cd src-tauri && cargo test -- --nocapture` passed: 14 tests passed.
- `npm run tauri:build` passed and produced ignored local Linux bundles under `src-tauri/target`.
- SQL-level synthetic coverage checks verify CLEAN templates normalize `CHAR(9)`, `CHAR(10)`, `CHAR(13)` and `CHAR(160)` before guarded `STR_TO_DATE`.
- Real MySQL / customer CSV smoke was not executed in this environment.

## 1.0.20 - 2026-07-06

### Added

- Added mapping catalog self-heal before import validation, RAW load, atomic import and config seed.
- Added mapping catalog health command with app version, mapping seed version, applied time, stale state and critical alias gap metrics.
- Added `meta_app_config` for observable `app_version`, `mapping_seed_version` and `mapping_seed_applied_at`.
- Added backend `import_current_file_atomic` so the main import path runs probe, catalog repair, batch creation, mapping validation, RAW load and dataset profile refresh as one backend flow.

### Changed

- Moved required mapping gate into the RAW load entrypoint so direct `import_start_raw_load` cannot bypass validation.
- Mark mapping validation and RAW load failures as `meta_import_batch.status='failed'` with a failure message.
- Main ImportPanel button now calls the backend atomic command instead of composing probe / create batch / validate / load RAW in the frontend.
- Frontend missing-required errors now show only the first 20 normalized headers while preserving complete details in mapping results.
- Synchronized package, Cargo, Tauri config, README, handoff and header version markers to `1.0.20`.

### Verification

- `npm run check` passed.
- `npm run build` passed; Vite reported the existing large chunk warning.
- `cd src-tauri && cargo check` passed with existing dead_code warnings.
- `cd src-tauri && cargo test -- --nocapture` passed: 10 tests passed.
- `npm run tauri:build` passed and produced ignored local Linux bundles under `src-tauri/target`.
- Synthetic Universal Video header coverage was verified by Rust tests using `ID, Subscriber Account, User Mac, Local IP Address, Universal Video Applications, Statistics Duration, Downloaded Data Volume (KB), Effective Download Duration (s)`.
- Real MySQL / customer CSV import smoke was not executed in this environment.

## 1.0.19 - 2026-07-06

### Fixed

- Fixed Universal Video detail CSV alias matching for headers such as `Subscriber Account`, `Subscriber Account` with NBSP, `Downloaded Data Volume (KB)` and `Effective Download Duration (s)`.
- Replaced per-command header normalization with shared normalization that converts all non ASCII alphanumeric separators to `_`, trims leading/trailing `_` and compresses repeated `_`.
- Made `mapping_validation_commands.rs` and `raw_import_v2.rs` use the same normalization helper to avoid validation / raw-load drift.
- Changed `tcp.user_type` and `game.user_type` import mappings to optional so missing access type does not block RAW import.
- Removed unsafe positional fallback from mapped streaming RAW import when a column has no matching header alias.
- Expanded mapping validation diagnostics with candidate aliases, normalized aliases and normalized CSV headers.

### Changed

- Added Universal Video detail aliases for subscriber account, user MAC, application name, statistics duration, downloaded data volume and effective download duration.
- Synchronized package, Cargo, Tauri config, README, handoff and header version markers to `1.0.19`.

### Verification

- `npm run check` passed.
- `npm run build` passed; Vite reported the existing large chunk warning.
- `cd src-tauri && cargo check` passed with existing dead_code warnings.
- `cd src-tauri && cargo test -- --nocapture` passed: 5 tests passed.
- `npm run tauri:build` passed and produced local Linux bundles under ignored `src-tauri/target`.
- Real Universal Video detail CSV import smoke was not executed in this environment because no customer CSV / MySQL smoke setup was provided.

## 1.0.18 - 2026-07-06

### Added

- Added business-backed dashboard commands for Game Experience, Network Quality, User Profile, Video Experience detail and Cable / FTTH hourly detail.
- Added module business CSV export queries for Overview, App Usage, Video Experience, Game Experience, Network Quality, Cable / FTTH, Migration Lead and User Profile.
- Added `docs/validation/mysql-smoke-checklist-1.0.18.md` for MySQL / CSV / Lead / batch switching smoke validation.

### Changed

- Hardened batch SQL table binding to replace table identifiers in quoted names and supported SQL table positions instead of broad substring replacement.
- Updated complete aggregate and dashboard ETL Job Step metadata to record physical source / target table names.
- Extended `meta_etl_job_step.source_table` and `target_table` to `VARCHAR(512)` for multi-table physical diagnostics.
- Module status now checks supported data_type, physical table existence, row_count, required physical fields via `information_schema.columns`, and optional ADS rows for the selected `analysis_run_id`.
- Game, Network and User Profile module dashboards now call business DWD / DWS / ADS commands instead of generic moduleMetrics.
- Module export filename presets now include safe batch name, analysis_run_id, module_id and timestamp.
- Synchronized package, Cargo, Tauri config, README, handoff and header version markers to `1.0.18`.

### Verification

- `npm install` passed; npm reported existing audit findings: 2 moderate and 1 high.
- `npm run check` passed.
- `npm run build` passed; Vite reported the existing large chunk warning.
- `cd src-tauri && cargo check` passed with existing dead_code warnings.
- `npm run tauri:build` passed and produced local Linux bundles under ignored `src-tauri/target`.
- `cd src-tauri && cargo test replaces_identifier_positions_only -- --nocapture` passed for the SQL table replacement helper.
- MySQL / TCP CSV / Game CSV / Lead export / batch switching smoke was not executed in this environment because no `mysql` / `mysqladmin` client and no CSV samples were present.

## 1.0.17 - 2026-07-06

### Added

- Added batch list selection, batch table registry and module status commands for analysis workspace routing.
- Added batch-first analysis workspace shell with real batch selection and backend module readiness checks.
- Added per-module dashboard shell components and system diagnostics registry/status views.

### Changed

- Updated mapping validation to aggregate alias rows into target-level required checks.
- Extended default mapping aliases for Universal Video CSV headers and required import fields.
- Routed dashboard, lead and final fusion queries through batch physical table resolution where available.
- Synchronized package, Cargo, Tauri config, README, handoff and header version markers to `1.0.17`.

### Not fully implemented

- End-to-end compile, MySQL smoke and tauri build still need to be run locally after the route changes.

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

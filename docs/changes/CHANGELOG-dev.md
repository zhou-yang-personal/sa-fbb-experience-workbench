# CHANGELOG-dev

## 1.0.25 - 2026-07-06

### Added

- Added `AnalyticsDashboard.tsx`, a DWS/ADS-backed analytics cockpit for the Data Analysis area.
- Added `AnalyticsDashboard.css` for large-screen KPI cards, chart panels, tabs and sticky-header tables.
- Added six analysis tabs: overview cockpit, app experience, network quality, Cable vs FTTH, user profile and migration upsell leads.

### Changed

- Replaced the default Data Analysis module-card workflow with a large-screen analytics cockpit while keeping batch selection and diagnostics.
- Moved module readiness and batch table registry into an advanced diagnostics section instead of making it the primary analysis surface.
- Updated `WorkbenchAppV2` analysis guidance from small module charts to the current batch analytics cockpit.
- Reused existing DWS/ADS-backed dashboard commands for the first implementation; no RAW table scans were added.
- Synchronized `package.json`, `tauri.conf.json`, `WorkbenchHeader.tsx`, `mapping_catalog.rs`, README, handoff and changelog to `1.0.25`; `src-tauri/Cargo.toml` update was blocked by ChatGPT GitHub connector safety checks and remains pending.

### Verification

- GitHub connector diff confirms the new analytics cockpit component, analytics CSS, AnalysisWorkspace integration and version/documentation updates on `dev`.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

## 1.0.24 - 2026-07-06

### Changed

- Replaced all executable timestamp NBSP normalization from `CHAR(160)` to `CONVERT(0xC2A0 USING utf8mb4)` in TCP / Game Quality Gate and RAW→CLEAN SQL.
- Added CSS overflow guards for pipeline plan rows, failure cards and realtime pipeline logs so full SQL statements no longer stretch the Data Import page.
- Updated Rust SQL-template assertions to require `CONVERT(0xC2A0 USING utf8mb4)` and reject `CHAR(160)`.
- Synchronized `package.json` and README to `1.0.24`; Cargo/Tauri/Header version updates were blocked by ChatGPT GitHub connector safety checks and remain pending for local/Codex follow-up.

### Fixed

- Fixed MySQL `ERROR 3854 Cannot convert string '\xA0' from binary to utf8mb4` in Quality Gate by removing `CHAR(160)` from timestamp cleaning expressions.
- Prevented long pipeline error text and realtime logs from expanding the import page indefinitely.

### Verification

- Repository search found no remaining `CHAR(160)` occurrences after this change.
- GitHub connector diff confirms the SQL, Rust assertion, CSS and `package.json` changes on `dev`.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV smoke has not been executed yet.

## 1.0.23 - 2026-07-06

### Added

- Added persistent import pipeline metadata tables: `meta_pipeline_run`, `meta_pipeline_step` and `meta_pipeline_log`.
- Added backend pipeline commands: `import_pipeline_start`, `import_pipeline_get_status` and `import_pipeline_get_logs`.
- Added an asynchronous import-to-analysis execution plan that runs environment preparation, CSV probe, atomic import, Quality Gate, CLEAN/DWD, DWS/ADS, optional Final Lead fusion and Module Ready in order.

### Changed

- Reworked Data Import default UX from visible manual 1-8 action buttons to one primary "启动导入分析计划" button with a status timeline and realtime backend logs.
- Frontend now polls pipeline status and logs every second while a pipeline is running; manual single-step actions remain available only under advanced troubleshooting.
- Final Lead fusion is treated as an optional/degradable pipeline step. Missing auxiliary data or zero Final Lead rows marks the pipeline `degraded` while keeping base DWS/ADS and SA Lead available.
- Quality Gate now routes by batch `data_type`: tcp checks only TCP raw/clean tables, game checks only Game raw/clean tables, and CRM/coverage/reachability are recorded as skipped/not applicable.
- Synchronized package, Cargo, Tauri config, README, handoff, header and mapping catalog version markers to `1.0.23`.

### Fixed

- Fixed Quality Gate MySQL ERROR 1411 by normalizing `statistics_duration` / `statistical_time` into `stat_time_text` before guarded `STR_TO_DATE`.
- Fixed Quality Gate timestamp guards to support one- or two-digit slash day/month formats and to replace tab/LF/CR/NBSP with spaces instead of deleting middle separators.

### Verification

- `npm run check` passed.
- `npm run build` passed; Vite reported the existing large chunk warning.
- `cd src-tauri && cargo check` passed with existing dead_code warnings.
- `cd src-tauri && cargo test -- --nocapture` passed: 23 tests passed.
- `npm run tauri:build` passed and produced ignored local Linux bundles under `src-tauri/target`.
- Rust tests verify Quality Gate SQL does not parse raw timestamp fields directly, includes the same invisible-character cleanup guards as CLEAN, routes by data_type, and keeps Final Lead failures as pipeline degradation.
- Real MySQL / customer CSV smoke has not been executed yet.

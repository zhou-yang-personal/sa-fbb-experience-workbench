# CHANGELOG-dev

## 1.0.41 - 2026-08-24

### Added

- Added a five-second RAW import heartbeat containing processed bytes, source size and transfer percentage for LOAD DATA and Streaming INSERT.
- Added a 30-second no-byte-change warning that distinguishes pre-transfer stalls, mid-transfer stalls and post-transfer MySQL parse/index/commit waits.

### Changed

- Import batches are now attached to the pipeline immediately after creation instead of only after the entire RAW import returns.
- Version markers were synchronized to `1.0.41`.

### Verification

- `cd src-tauri && cargo test --offline`: passed, 41 tests.
- `npm run check`, `npm run build` and `cd src-tauri && cargo check --offline`: passed.
- Live Windows/MySQL import and representative 1 GB+ benchmark: not run in this environment.

## 1.0.40 - 2026-08-24

### Fixed

- Registered a per-import MySQL `LocalInfileHandler` so `LOAD DATA LOCAL INFILE` receives the selected CSV bytes instead of an empty client payload that produces zero imported rows without warnings.
- Restricted the handler to the canonical path of the current user-selected file and removed it immediately after the LOAD DATA statement.

### Performance

- The handler streams the CSV through a bounded 1 MiB buffer and does not load the complete file into application memory.

### Verification

- `cd src-tauri && cargo test --offline`: passed, 39 tests.
- The exact-file allowlist, alternate-file denial and missing-file rejection are covered by unit tests.
- Real MySQL import and representative 1 GB+ CSV benchmark: not run in this environment.

## 1.0.39 - 2026-08-24

### Changed

- CSV Probe, mapping validation, `LOAD DATA LOCAL INFILE` and Streaming INSERT now share bounded delimiter detection for comma, Tab and semicolon files.
- Dataset profile metrics now query the current batch physical RAW table and exclude the `--` sentinel from distinct identity counts.
- Pipeline failure cards automatically load RAW batch status, table registry evidence and failed Quality Gate items.
- Added TCP aliases for `Throughput (Average Bandwidth) (kbps)` and `Users Average Effective Download Rate (kbps)`.
- Version markers were synchronized to `1.0.39`.

### Fixed

- RAW imports no longer mark zero-row outcomes as successful. LOAD DATA verifies batch-visible rows, captures MySQL warnings and fails before Quality Gate; Streaming INSERT rejects header-only/zero-row files.
- Quality Gate fatal errors now include the failed check names and metric evidence instead of only instructing the user to call another command.
- Quality checks no longer count the `--` sentinel as a real account, MAC or application value.

### Verification

- `npm run check`: passed.
- `npm run build`: passed with the existing non-blocking Vite chunk-size warning.
- `cd src-tauri && cargo check --offline`: passed with existing warnings.
- `cd src-tauri && cargo test --offline`: passed, 36 tests.
- Real MySQL and representative 1 GB+ CSV smoke/benchmark: not run in this environment.

## 1.0.38 - 2026-08-22

### Added

- Added historical batch management with checkbox selection, quick selection of test/failed batches, and protected bulk deletion.
- Batch deletion removes owned physical tables plus associated legacy RAW/DWD/DWS/ADS rows, analysis runs, exports, quality/mapping/profile results, pipelines and ETL metadata.

### Changed

- MySQL password defaults to `123456`; a user override remains memory-only and resets to the built-in default after reload or context reset.
- Version markers were synchronized to `1.0.38`.

### Fixed

- Replaced the oversized `ads_network_hotspot_rank` utf8mb4 composite primary key with an auto-increment primary key and bounded prefix index, preventing MySQL ERROR 1071 during database initialization.

### Verification

- `npm run check`: passed.
- `npm run build`: passed with the existing non-blocking Vite chunk-size warning.
- `cd src-tauri && cargo check --offline`: passed with existing warnings.
- `cd src-tauri && cargo test --offline`: passed, 30 tests.
- Real MySQL batch deletion and migration execution remain to be smoke-tested against the target database.

## 1.0.37 - 2026-08-21

### Fixed

- Registered the Tauri Dialog plugin and `dialog:default` capability so the packaged Windows app opens the native CSV file picker.
- File-picker invocation failures now surface in the action feedback and execution log.

### Changed

- TCP / Game imports no longer silently bind the latest published IP rule version.
- Every TCP / Game import requires manual selection and confirmation of a published IP rule version; the selected ID is validated and bound by the Rust backend.
- Version markers were synchronized to `1.0.37`.

### Verification

- `npm run check`: passed.
- `npm run build`: passed with the existing non-blocking Vite chunk-size warning.
- `cd src-tauri && cargo check --offline`: passed with existing warnings.
- `cd src-tauri && cargo test --offline`: passed, 27 tests.
- Packaged Windows native-dialog interaction and real MySQL import remain to be verified by the GitHub Actions artifact smoke test.

## 1.0.36 - 2026-08-21

### Added

- Added GitHub Actions automation for Windows MSI, NSIS EXE and portable EXE builds.
- Branch builds upload 30-day workflow artifacts; version tags automatically create GitHub Releases with public binaries.

### Changed

- Version markers were synchronized to `1.0.36`.

### Verification

- Local workflow syntax and repository status checks completed.
- Remote Actions and downloadable release assets require the pushed workflow run to complete.

## 1.0.35 - 2026-08-21

### Added

- Versioned Cable / FTTH / Other IPv4 range rules with draft editing, validation, preview, atomic publishing and batch binding.
- DWS tables at actual App and user-App grain, plus topology-grain network hotspot evidence.
- Six task-oriented analysis dashboards and an evidence drawer.

### Changed

- Access classification now uses published IP rules first and CSV source fields only as fallback; DWD preserves rule provenance and confidence.
- Automatic import analysis now blocks on fatal Quality Gate errors, retains non-fatal warnings, and materializes structured App, hourly, network, user and lead ADS outputs.
- Lead logic explicitly separates A0 identity-insufficient and A2 repair-first users from A1 candidates requiring CRM, coverage and reachability checks.
- Product navigation, visual hierarchy, filtering, chart fallbacks and configuration workflow were redesigned.
- Reordered CSV headers no longer force the 1 GB+ path into batched INSERT; `LOAD DATA` now maps source positions to RAW columns with MySQL user variables.
- Version markers were synchronized to `1.0.35`.

### Fixed

- Corrected Rust closure syntax that prevented the backend from parsing.
- Corrected RAW/DWD/DWS SQL table placeholders so jobs target the current batch's physical tables.
- Replaced category-only App ranking and synthetic network labels with real App and BRAS / OLT / PON evidence.
- Aligned the per-batch physical table suffix implementation with its tested 15-character contract.

### Verification

- `npm run check`: passed.
- `npm run build`: passed with a non-blocking Vite chunk-size warning.
- `cd src-tauri && cargo check`: passed with existing warnings.
- `cd src-tauri && cargo test --offline`: passed, 26 tests.
- Real MySQL and 1 GB+ CSV benchmark: not run.

## 1.0.34 - 2026-07-07

### Changed

- App Rank, Hourly Trend, Network Hotspot, User Profile and Lead Evidence structured read commands now prefer materialized Analytics ADS rows for the current `analysis_run_id`.
- Each read command falls back to the prior DWS / Lead source when Analytics ADS rows are absent or the Analytics ADS table cannot be resolved.
- Evidence hints now include `source=...` to make ADS-first versus fallback reads visible in UI tables.
- Version markers were synchronized to `1.0.34` in package, Tauri config, Cargo, Workbench header, mapping catalog, README and handoff.

### Verification

- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV smoke has not been executed yet.

## 1.0.33 - 2026-07-07

### Added

- Added `AnalyticsAdsActions.tsx` as a visible structured ADS materialization action panel.
- Added compact materialization commands for Hourly, User, Lead and Network ADS tables.
- Exposed App, Hourly, User, Lead and Network materialization APIs in `analyticsStructuredApi.ts`.

### Changed

- `AnalysisWorkspace.tsx` now renders the structured ADS action panel.
- Version markers were synchronized to `1.0.33` in package, Tauri config, Cargo, Workbench header, mapping catalog, README and handoff.

### Verification

- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV smoke has not been executed yet.

## 1.0.32 - 2026-07-07

### Fixed

- Restored `analysis_run_batch`, `table_exists` and `table_columns` in `batch_tables.rs`.

### Added

- Added and registered `analytics_materialize_app_rank`.

## 1.0.31 - 2026-07-07

### Added

- Added `AnalyticsStructuredPagedPanel.tsx`.
- Registered structured Analytics ADS tables in the batch table registry.
- Added SQL scripts `003b` to `003f` for App, Hourly, User, Lead and Network analytics ADS materialization.

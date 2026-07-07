# CHANGELOG-dev

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

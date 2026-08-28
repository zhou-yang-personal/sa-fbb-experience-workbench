# CHANGELOG-dev

## 1.0.62 - 2026-08-28

### Fixed

- Replaced the nested page-panel execution log with a single-header, single-scroll-region drawer.
- Prevented the search, status and copy controls from stretching or wrapping into malformed layouts.
- Added bounded long-text rendering, responsive narrow-window behavior and backdrop-click closing.

### Verification

- `npm run check`: passed on 2026-08-28.
- `npm run build`: passed on 2026-08-28.
- No database, aggregation, dependency or lock-file changes.

## 1.0.61 - 2026-08-28

### Changed

- Removed duplicated product copy, version card, global-scope placeholder, technical context and batch/run metadata from the analysis landing area.
- Compacted batch selection, page title, load and PDF actions into low-height toolbars; analysis-path controls appear only when filters exist.
- Moved language selection and the application version to the sidebar so the first viewport prioritizes analysis content.

### Verification

- `npm run check`: passed on 2026-08-28.
- `npm run build`: passed on 2026-08-28.
- `cargo check --offline`: passed on 2026-08-28 with the existing 23 warnings.
- No database, aggregation, dependency or lock-file changes.

## 1.0.60 - 2026-08-28

### Added

- Added migration 011 for versioned decision rules, run bindings, four opportunity outputs and independently resumable aggregation subtask checkpoints.
- Added DWS-backed metric, unique-App, user-distribution, issue-side and Cable/FTTH queries without RAW scans.
- Added previously discarded TCP duration, average-rate, throughput, connection and fluency fields to CLEAN/DWD and the reusable user-by-App period DWS; existing RAW batches can regenerate them without re-importing CSV.
- Added migration, speed upgrade, AP/mesh and App Bundle opportunity materialization with explicit availability and limitation states.
- Opportunity candidates now enforce configurable active-day, observation and matched-condition gates and use daily traffic/duration/observation evidence instead of opaque demand scores.
- Mesh/AP candidates now require configurable home-side evidence coverage and use Wi-Fi delay, RTT delta or loss delta; insufficient coverage remains unavailable instead of becoming zero candidates.

### Changed

- Reduced the primary navigation to Panorama, Poor Quality, Cable/FTTH, Opportunities and Data Center; configuration and diagnostics are secondary.
- Reordered the product narrative so overall metrics, all Apps and all users appear before access segmentation or issue-side evidence.
- Expanded each panorama perspective with a shared overall baseline, user distributions and full unique-App coverage; the quality page now retains the affected unique-App list after its evidence summary.
- Added an explicit mutually exclusive unique-App status reconciliation so total Apps equals severe + problem + watch + normal + limited-sample + insufficient-sample.
- Changed TCP/Game analytical identity to IPv4 only, using User Account first and Local IP as fallback.
- Added all-non-empty-chart PDF printing, bilingual page copy, explicit GB/TB/Mbps/ms/% units, rule versions, numerators, denominators and sample limitations.

### Performance and recovery

- Added one checkpoint per DWS/ADS subtask. A resumed run skips successful subtasks and retries incomplete work, while the existing hourly date/hour checkpoints remain unchanged.
- All new panorama queries read the approximately user-by-App V2 DWS rather than RAW or DWD.

### Verification

- `npm run check`: passed on 2026-08-28.
- `npm run build`: passed on 2026-08-28.
- `cargo check`: passed on 2026-08-28 with existing warnings only.
- `cargo test --no-fail-fast`: 57 passed, 0 failed on 2026-08-28.
- Live Windows/WebView2 and real customer MySQL migration/SQL execution remain pending.

## 1.0.59 - 2026-08-28

### Fixed

- Replaced panic-prone App and network ADS row conversions with nullable fallible decoding and added a command-level panic guard to every structured legacy dashboard query.
- Removed schema migration work from the dashboard coverage read path; loading a dashboard is now read-only.

### Changed

- Restored Overview as the comprehensive insights entry point while retaining Findings as a dedicated highlighted exception and investigation workflow.
- Rebuilt Application Experience around the complete App-by-access portfolio, with policy-derived normal, attention, severe, insufficient-sample and legacy-unclassified states.
- Added full-population App coverage, traffic and status views plus a wide evidence table containing metric numerators, denominators, sample state, policy version and drill-down actions.
- Increased the bounded App ADS page size from 200 to 500 so the current 246-combination portfolio is not truncated; no RAW/DWD scan is introduced.

### Verification

- `npm run check`: passed on 2026-08-28.
- `npm run build`: passed on 2026-08-28; the existing ECharts chunk-size warning remains.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 RUSTFLAGS='-C debuginfo=0' cargo check --offline`: passed on 2026-08-28 with existing warnings only.
- Live Windows/WebView2/customer-MySQL validation remains pending for the 1.0.59 artifact.

## 1.0.58 - 2026-08-28

### Fixed

- Replaced panic-prone MySQL conversions in the investigation workflow with fallible nullable row decoding for legacy V2 and saved-investigation data.
- Hardened Finding, coverage, numeric and status rendering against null or malformed historical payloads.
- Added a page-level analysis error boundary with a recovery route back to the overview.

### Verification

- `npm run check`: passed on 2026-08-28.
- `npm run build`: passed on 2026-08-28; the existing ECharts chunk-size warning remains.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 RUSTFLAGS='-C debuginfo=0' cargo check --offline`: passed on 2026-08-28 with existing warnings only.
- Live Windows/WebView2/MySQL validation remains pending for the 1.0.58 artifact.

## 1.0.57 - 2026-08-27

### Fixed

- Scoped the stale-pipeline takeover confirmations to a dedicated grid layout in both RAW rebuild and DWS/ADS resume panels.
- Prevented the product-shell `input { width: 100% }` rule from stretching checkboxes and squeezing the safety explanation.

### Verification

- `npm run check`: passed on 2026-08-27.
- `npm run build`: passed on 2026-08-27; the existing ECharts chunk-size warning remains.
- Live Windows/WebView2 visual confirmation remains pending for the 1.0.57 artifact.

## 1.0.56 - 2026-08-27

### Fixed

- Replaced the hourly V2 full-batch transaction that repeatedly triggered an InnoDB semaphore assertion with independent date/hour transactions.
- Added resumable partition checkpoints and interrupted-state reconciliation after MySQL restart, preventing stale `running` metadata and partial results from appearing ready.
- Added a MySQL named aggregation lock to reject concurrent DWS/ADS rebuilds on the same local database.
- Restored statement-level SQL logging on the resume path and included MySQL connection IDs in SQL evidence.

### Performance

- Added a one-time DWD index on `(import_batch_id, stat_date, hour_of_day)` before hourly partitioning, so each partition performs a bounded range scan instead of repeatedly scanning the complete DWD table.
- Reuses already committed legacy App, Period V2 and App ADS results for the same analysis run; successful hourly partitions are never recomputed during resume.

### Observability

- Added per-partition start/success/failure/skip logs with date, hour, progress, connection ID, duration and affected rows.
- Clarified that the 15-second application heartbeat proves only that the application thread can write metadata; current SQL activity is represented by SQL/partition logs.

### Verification

- `npm run check`: passed on 2026-08-27.
- `npm run build`: passed on 2026-08-27; the existing ECharts chunk-size warning remains.
- `cargo check --offline`: passed on 2026-08-27 with existing warnings only.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS='-C debuginfo=0' cargo test --offline -q`: passed on 2026-08-27; 53 tests passed.
- Live Windows/MySQL recovery against the 13.2M-row customer batch remains pending for the 1.0.56 artifact.

## 1.0.55 - 2026-08-26

### Changed

- Reorganized import operations into New Import, Batch Library and Running Tasks workspaces around real operator goals.
- Replaced the import-page history dropdown with a searchable batch status table while retaining the compact selector in Analysis Workspace.
- Made batch selection local-only; pipeline state, log retrieval and polling now require an explicit operator action.
- Added state-specific next-step guidance and direct transitions from task failure to recovery choices and from ready batches to analysis.
- Consolidated duplicate manual actions and diagnostic tables under developer-only disclosure controls.

### Verification

- Operator journey and UX scoring are documented in `docs/design/import-job-center-operator-journey.md`.
- `npm run check`: passed on 2026-08-26.
- `npm run build`: passed on 2026-08-26; the existing ECharts chunk-size warning remains.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 cargo test --offline`: passed on 2026-08-26; 48 tests passed with existing warnings only.
- Live Windows/WebView2 interaction and customer MySQL behavior remain pending until the new artifact is tested locally.

## 1.0.54 - 2026-08-26

### Fixed

- Fixed the remaining batch-refresh crash path in analysis-run registered-table lookup and shared batch-table metadata resolution when legacy registry values are `NULL`.
- Decoupled batch-list refresh and batch selection from automatic analysis-run queries; loading run options is now a separate explicit action.
- Removed import-page mount queries for access-rule versions, history batches and stored pipeline polling, restoring the documented no-database-command startup boundary.
- Added command boundary markers for batch and analysis-run list calls to the credential-free Windows runtime log.

### Verification

- `npm run check`: passed on 2026-08-26.
- `npm run build`: passed on 2026-08-26; the existing ECharts chunk-size warning remains.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 cargo test --offline`: passed on 2026-08-26; 48 tests passed.
- Live Windows/MySQL refresh against the customer legacy metadata remains pending.

## 1.0.53 - 2026-08-26

### Fixed

- Fixed the batch-refresh Windows crash caused by legacy nullable `meta_import_batch` text values being decoded into required Rust `String` tuple fields.
- Added SQL-side null/blank fallbacks and fallible row-by-row decoding for batch ID, data type, source filename, status and row counts, so malformed legacy metadata degrades visibly instead of panicking the process.

### Verification

- `npm run check`: passed on 2026-08-26.
- `npm run build`: passed on 2026-08-26; the existing ECharts chunk-size warning remains.
- `cargo check --offline`: passed on 2026-08-26 with existing warnings only.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 cargo test --offline`: passed on 2026-08-26; 48 tests passed.
- Live Windows/MySQL refresh against the nullable legacy row remains pending.

## 1.0.52 - 2026-08-26

### Added

- Added an explicit current-batch RAW rebuild pipeline that preserves CSV/RAW, reruns Quality Gate through CLEAN/DWS/ADS/V2, creates a new analysis run and retains existing concurrency protection.
- Added statement-level SQL execution events for RAW rebuild core scripts, including RUNNING/SUCCESS/FAILED, duration, affected rows and a bounded statement preview in the existing real-time pipeline log.
- Added a credential-free Windows runtime log for application start, normal exit, Tauri runtime errors and Rust panic evidence.

### Fixed

- Removed startup batch and analysis-run queries. Opening the application now performs no MySQL command until the user explicitly refreshes or starts an action.

### Verification

- `npm run check`: passed on 2026-08-26.
- `npm run build`: passed on 2026-08-26; the existing ECharts chunk-size warning remains.
- `cargo check --offline`: passed on 2026-08-26 with existing warnings only.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 cargo test --offline`: passed on 2026-08-26; 46 tests passed.
- Windows artifact startup and live MySQL RAW rebuild remain pending.

## 1.0.51 - 2026-08-26

### Fixed

- Preferred the latest successful/degraded analysis run for each batch, so side-by-side manual V2 results are not overwritten by an older pipeline run ID.
- Added an explicit per-batch analysis-run selector with bounded Period V2, App ADS and Hourly V2 readiness checks plus policy-version context.
- Replaced all-or-nothing V2 foundation loading with isolated dataset results and clear partial-failure or wrong-run guidance.

### Verification

- `npm run check`: passed on 2026-08-26.
- `npm run build`: passed on 2026-08-26; the existing ECharts chunk-size warning remains.
- `cargo check --offline`: passed on 2026-08-26 with existing warnings only.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 cargo test --offline`: passed on 2026-08-26; 44 tests passed.
- Live Windows/MySQL selection of `RUN_REAGG_V2_20260825` remains to be verified on the customer batch.

## 1.0.50 - 2026-08-25

- Added a shared, persistent Analysis Context with filter chips, single-filter removal, clear and back navigation across analysis pages.
- Replaced the default overview entry with Experience Status, rule-qualified Auto Findings and explicit Data Coverage; large queries remain user-triggered and ADS/DWS-first.
- Added a bilingual Investigation Workspace with finding scope, cautious evidence-side judgement, affected-user evidence, user/app/hour drill-down and saved investigations.
- Added typed Tauri APIs for V2 status, findings, coverage, investigation evidence and saved investigation state.
- Added versioned Experience Policy and App Profile configuration, immutable publish flow and category-profile cloning. Published policies are snapshotted by new analysis runs.
- Added `user × app × hour` DWS and App-hour ADS for reusable time investigation without dashboard RAW scans.
- Retained source Server IP in new CLEAN results and added an App/Finding-scoped evidence drill-down capped at 200 priority users and 20,000 DWD observations; no global Server-IP explosion is created.
- Added previous comparable-run verification for poor-observation, persistent-user and severe-user rates. Comparison is suppressed unless all bound rule versions match.
- Renamed legacy network views to Network / Path Evidence and stopped presenting missing topology as confirmed hotspots or root causes.
- Clarified that commercial results are experience-driven opportunities rather than directly marketable CRM leads.
- Synchronized application version markers to `1.0.50`; `AGENTS.project.md` remains unchanged because it requires explicit user authorization.

### Verification

- `npm run check`: passed on 2026-08-25.
- `npm run build`: passed on 2026-08-25; the existing ECharts chunk-size warning remains.
- `cargo check --offline`: passed on 2026-08-25 with existing warnings.
- `cargo test --offline --quiet`: passed on 2026-08-25.
- Live MySQL execution, controlled Server-IP runtime and comparable-run values on the 13,205,379-row Windows batch remain pending.

## 1.0.49 - 2026-08-25

### Added

- Added a versioned experience-policy foundation, App experience profiles, analysis-run policy snapshots and side-by-side V2 user/App, App/access and App ADS tables.
- Added four auditable App experience rates with exact numerators, denominators, sample status and policy version: poor observation, ever affected, persistent poor and severe poor.
- Added a no-RAW-reimport manual reaggregation for `BATCH_7ae0c7d1c0a240ba833e366bf755397d`, producing `RUN_REAGG_V2_20260825` while preserving `RUN_MANUAL_001`.
- Added reusable bilingual chart explanations to all current dashboard and PDF chart positions.

### Changed

- Others is now an explicit per-rule-version choice. New drafts start unconfigured; preview, publish, import binding and batch binding reject missing or Unknown Others values.
- RAW-to-CLEAN classification now uses explicit IP ranges followed by configured Others. CSV access fields remain evidence only; missing/invalid IP is kept as unavailable instead of being forced to Cable.
- App materialization now produces V2 metrics and App queries prefer V2 ADS. Problem-App charts use persistent-poor metrics and exclude insufficient samples; legacy ADS remains a compatibility fallback.
- Analysis runs snapshot access-rule and experience-policy versions. Version markers were synchronized to `1.0.49`, except `AGENTS.project.md`, which requires explicit user authorization to edit.

### Verification

- `npm run check`: passed on 2026-08-25.
- `npm run build`: passed on 2026-08-25; the existing ECharts chunk-size warning remains.
- `cargo check --offline` with the existing isolated target cache and incremental compilation disabled: passed with 23 existing warnings.
- Rust access-rule tests: 2 passed in the parallel backend validation.
- SQL static checks and `git diff --check`: passed. Live MySQL execution against the 13,205,379-row Windows batch is pending because this Linux environment has no MySQL service.
- Dependencies and committed lock files were not changed.

## 1.0.48 - 2026-08-25

### Added

- Added a versioned unmatched-IP default to access rule sets. Existing and new sets default to Cable, so a user can configure only FTTH ranges while retaining UNKNOWN as an explicit conservative option.
- Added bounded TCP/Game dataset coverage and access-classification context to every on-demand dashboard task.
- Added full-population Lead stage and user cohort summaries from ADS instead of deriving totals from paginated evidence.

### Changed

- RAW-to-CLEAN access precedence is now IP rule, recognizable CSV source field, then rule-set default; previews include fallback-classified IPs in Cable/FTTH totals.
- Missing Game input is shown as not imported and no longer presented as zero game activity.
- Cable/FTTH charts use active-user-weighted typical-day curves; user charts use full demand, traffic and bottleneck cohorts; App labels clarify that poor users had at least one poor observation during the analysis period.
- Project and application version markers were synchronized to `1.0.48`.

### Verification

- `npm run check`: passed on 2026-08-25.
- `cargo check` with isolated target directory: passed on 2026-08-25 with existing warnings only.
- Live rerun of CLEAN/DWS/ADS against the 3.46 GiB Windows/MySQL batch and Windows PDF visual acceptance: pending.
- Dependencies and committed lock files were not changed.

## 1.0.47 - 2026-08-25

### Added

- Added “导出全部图表 PDF” as an explicit six-dataset task covering all 20 chart positions across the six decision dashboards while excluding evidence tables.
- Added dataset progress, stop-after-current-query behavior, partial/empty/failure states and a reusable report preview.
- Added an A4 landscape print report with batch, analysis run, filters, local PC time/timezone, DWS/ADS source and omitted/failed chart summaries.

### Changed

- Data loaded for PDF preparation is reused by the current dashboard session; application startup and page navigation remain query-free.
- Project and application version markers were synchronized to `1.0.47`.

### Verification

- `npm run check`: passed on 2026-08-25.
- `npm run build`: passed on 2026-08-25; the existing ECharts chunk-size warning remains.
- Targeted Rust test `critical_aliases_track_universal_video_contract`: passed; 1 test passed and 43 were filtered, with existing warnings only.
- Windows WebView2 print-dialog behavior and visual inspection of a real multi-page PDF: not run in this Linux source environment.
- Dependencies and lock files were not changed.

## 1.0.46 - 2026-08-25

### Added

- Added an explicit “reuse current batch” pipeline that skips CSV/RAW/CLEAN work and resumes the complete DWS/ADS, optional Final Lead and Module Ready tail.
- Added backend stale-run takeover validation with explicit confirmation and batch-specific MySQL active-statement detection.
- Added named start/completion/error logs for eight DWS/ADS aggregate subtasks.

### Changed

- Automatic and manual complete-result paths now materialize all five structured ADS datasets: App Rank, hourly trend, network hotspot, user profile and Lead Evidence.
- Normal Registry and module-readiness flows use cached/information-schema estimates plus bounded `EXISTS` checks instead of exact full-table `COUNT(*)` scans.
- Base user-daily aggregation leaves `meta_analysis_run` running; success is recorded only after complete DWS and structured ADS materialization.
- Project and application version markers were synchronized to `1.0.46`.

### Verification

- `npm run build`: passed on 2026-08-25; the existing ECharts chunk-size warning remains.
- `cargo check --manifest-path src-tauri/Cargo.toml` with an isolated target directory: passed; existing dead-code and compatibility warnings remain.
- `CARGO_TARGET_DIR=/home/g314vows/.cache/sa-fbb-cargo-test CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 cargo test --manifest-path src-tauri/Cargo.toml --offline`: passed; 44 Rust tests passed and only existing warnings remain. An earlier temporary-filesystem attempt exhausted its quota before linking and was cleaned before the successful low-debug retry.
- Live Windows/MySQL resume against the 3.46 GiB batch: not run in this environment.

## 1.0.45 - 2026-08-25

### Changed

- Historical batches now expose RAW status and latest pipeline status separately, and selecting one synchronizes its latest pipeline `analysis_run_id`.
- Returning to the import page restores the selected batch's pipeline status and logs from sequence zero; selecting an old/manual batch clears unrelated persisted logs.
- Terminal pipeline refreshes load only lightweight quality and job evidence instead of automatically triggering Registry and module table-count diagnostics.

### Fixed

- Dashboard queries whose datasets are empty or whose KPI structures are all zero no longer report `SUCCESS`; the UI identifies missing CLEAN/DWS/ADS evidence and links back to import diagnostics.
- Latest pipeline run metadata now takes precedence over older analysis-run records for the same batch, preventing dashboard queries from using a stale run ID.

### Verification

- `npm run check` and `npm run build`: passed on 2026-08-25; the existing ECharts chunk-size warning remains.
- Targeted Rust format check and `cd src-tauri && cargo check --offline`: passed with existing warnings.
- `cd src-tauri && CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --offline`: passed; 42 Rust tests passed. The first attempt exhausted the full root filesystem during linking, so generated Cargo caches were cleaned and the test was rerun serially.
- Live Windows/MySQL batch selection, pipeline-log recovery and dashboard evidence verification: not run in this environment.

## 1.0.44 - 2026-08-25

### Changed

- System-page navigation no longer starts MySQL diagnostics automatically; the user must explicitly start the diagnostic task.
- Catalog, mapping, quality, ETL, module and Registry checks now run serially with progress, per-step state and stop-after-current-query behavior.
- Quality/ETL troubleshooting components and the full execution log mount only after their collapsed sections are opened.

### Fixed

- Removed duplicate large-table row-count refreshes from one module/Registry diagnostic run by adding a cached Registry read path.

### Verification

- `npm run check` and `npm run build`: passed on 2026-08-25; the existing ECharts chunk-size warning remains.
- `cd src-tauri && cargo check --offline` and `cd src-tauri && cargo test --offline`: passed; 42 Rust tests passed and only existing Rust warnings remain.
- Live Windows/MySQL page-navigation timing and diagnostic query timing: not run in this environment.

## 1.0.43 - 2026-08-25

### Added

- Added 15-second liveness heartbeats with step elapsed time and phase-specific explanations for non-RAW long-running pipeline steps.
- Added a pipeline monitoring console with polling health, log-silence detection, filters, search, pause/resume, manual refresh, ordering and filtered copy.
- Persisted the latest pipeline ID per MySQL context so monitoring resumes after navigating away from and back to the import page.

### Fixed

- Prevented overlapping one-second polling requests and deduplicated frontend log rows by sequence.
- Drained multi-page log backlogs and performed a final terminal-state log fetch so completion records are not lost.
- Added bounded retries when concurrent backend heartbeat and step-transition logs contend for the same sequence.
- Removed repeated pipeline-schema DDL from one-second read-only status and log polling.
- Standardized newly generated pipeline timestamps on UTC and converted both pipeline and execution log display to the local PC timezone; copied logs include the detected IANA timezone.

### Verification

- `npm run check` and `cd src-tauri && cargo check --offline`: passed on 2026-08-25.
- `npm run build`, `cd src-tauri && cargo check --offline` and `cd src-tauri && cargo test --offline`: passed; 42 Rust tests passed and only existing Rust warnings remain.
- Live Windows/MySQL 3.46 GiB pipeline monitoring: not run in this environment.

## 1.0.42 - 2026-08-24

### Changed

- Analysis startup now restores only lightweight context. It does not automatically prepare batch tables, count physical tables or load dashboard datasets.
- Each decision page exposes an explicit, page-scoped load task with plan, progress, current step, retry and stop-after-current-query interaction.
- Overview datasets run sequentially, while other pages request only the DWS / ADS dataset they need.
- Advanced diagnostics mount only when expanded, and ECharts is loaded dynamically when charts are rendered.

### Verification

- `npm run check`: passed on 2026-08-24.
- `npm run build`, `cd src-tauri && cargo check --offline` and `cd src-tauri && cargo test --offline`: passed; 41 Rust tests passed and only existing Rust warnings remain.
- Live Windows/MySQL interaction and representative 1 GB+ benchmark: not run in this environment.

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

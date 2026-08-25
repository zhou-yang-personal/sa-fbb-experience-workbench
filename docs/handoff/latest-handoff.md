# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.48
```

## Source-of-truth branch

```text
codex/task-dashboard-ip-segmentation
```

## Current baseline

Raw First MySQL pipeline is preserved:

```text
CSV → MySQL RAW → Quality Gate → CLEAN/DWD → DWS/ADS → SA Lead / Final Lead → Analytics cockpit / export
```

## 1.0.48 update

- Access rule sets now include `default_access_type`. The classification order is explicit IP range, recognizable CSV field, then rule-set default. The migrated/default value is Cable, matching the operating assumption that configured ranges identify FTTH and all remaining valid IPs are Cable.
- Access preview counts fallback-classified IPs in Cable/FTTH totals and reports the fallback population separately. Applying the corrected semantics to an existing batch requires rerunning CLEAN/DWS/ADS, not reimporting the CSV.
- Dashboard readiness distinguishes missing Game input from zero game use and warns when existing hourly ADS still contains UNKNOWN even though the bound rule set now has a non-UNKNOWN default.
- Lead stage charts now query a full ADS group summary rather than counting the first 500 evidence rows. User distribution charts query full-population demand, traffic and bottleneck cohorts instead of the first 300 profiles.
- Cable/FTTH trend charts show an active-user-weighted typical 24-hour profile while retaining dated hourly evidence in the table.
- Version markers are synchronized to `1.0.48`.

### Current-batch next action

- Reuse `BATCH_7ae0c7d1c0a240ba833e366bf755397d`; do not reimport the 3.46 GiB CSV. With no same-batch SQL active, open the import page's advanced steps, run `RAW → CLEAN`, then `单独生成 DWS/ADS`. Do not use the ordinary resume button for this correction because that path intentionally skips CLEAN.
- Game metrics remain intentionally unavailable until the separate Game file is imported and included in an analyzable batch.

## 1.0.47 update

- Added an explicit all-dashboard PDF task that serially queries the six structured DWS/ADS datasets and prepares every non-empty chart from the six decision views; evidence tables are excluded.
- The export locks batch ID, analysis run ID and active filters, reports per-dataset progress, supports stop-after-current-query, and records local PC generation time/timezone plus omitted-chart and query-failure evidence.
- Added a print-only A4 landscape report preview. Windows/WebView2 opens the familiar print flow so the user can save through Microsoft Print to PDF without a new runtime dependency or lock-file change.
- Export-loaded datasets are reused by the current dashboard session, while navigation and application startup remain query-free.
- Version markers are synchronized to `1.0.47`.

### PDF validation boundary

- Type checking, the production frontend build and the targeted Rust version/mapping test pass in the source environment.
- The Windows WebView2 print dialog, ECharts canvas pagination and a real multi-page PDF must still be visually accepted on the target PC/EXE.

## 1.0.46 update

- Added an explicit existing-batch resume pipeline. It validates successful RAW, latest Quality Gate and CLEAN jobs, skips CSV/RAW work, and reruns the complete DWS/ADS, optional Final Lead and Module Ready tail.
- A stale takeover requires explicit user confirmation and is rejected while MySQL `PROCESSLIST` still contains SQL for any physical table in that batch. The current 1.0.41 process must not be interrupted while its query remains active.
- DWS/ADS now emits start/completion/error records for eight named aggregate subtasks and materializes App Rank, hourly trend, network hotspot, user profile and Lead Evidence in both automatic and manual full-result paths.
- Normal batch-table preparation, Registry refresh and Module Ready checks no longer execute exact `COUNT(*)` scans. Registry uses cached/estimated counts and readiness uses bounded `EXISTS` checks.
- `meta_analysis_run` stays `running` after base user-daily aggregation and becomes `success` only after complete DWS and structured ADS materialization; failures are recorded as `failed`.
- Version markers are synchronized to `1.0.46`, including the project checklist baseline.

### Live-batch handoff boundary

- Batch `BATCH_7ae0c7d1c0a240ba833e366bf755397d` was observed running under the old 1.0.41 executable at `dws_ads_aggregate`; no write or interruption was performed from this source workspace.
- Do not start a resume task until the original EXE has exited and MySQL has no statement referencing the batch physical-table suffix. Then run 1.0.48, select the same batch and use the explicit resume action.
- Windows/MySQL validation against the 3.46 GiB source batch remains outstanding.

## 1.0.45 update

- Batch history now reports RAW and pipeline readiness independently and resolves the latest pipeline ID, status, failure message and `analysis_run_id` for each batch.
- Selecting a batch synchronizes its analysis context. The import page reloads that batch's pipeline status and logs from sequence zero, while batches without a pipeline clear stale monitor state.
- Dashboard completion is evidence-aware: empty arrays and six all-zero KPI placeholders produce an explicit empty state, not `SUCCESS`, and direct the user to the selected batch's import diagnostics.
- Terminal pipeline refreshes no longer auto-run Registry and module table-count checks.
- Version markers are synchronized to `1.0.45`.

## 1.0.44 update

- Removed all automatic MySQL diagnostics from system-page navigation; diagnostics now start only after an explicit user action.
- Added a six-step serialized diagnostic task with progress, current step, partial failures and stop-after-current-query behavior.
- Added cached Registry reads and changed module inspection to refresh table counts once, avoiding duplicate large-table counts in one diagnostic run.
- Lazily mounted Quality/ETL troubleshooting and the full execution log only after their sections are expanded.
- Version markers are synchronized to `1.0.44`.

## 1.0.43 update

- Serialized frontend pipeline polling with an in-flight guard, drained paged log backlogs, deduplicated by sequence and fetched terminal logs once more before stopping.
- Added 15-second liveness heartbeats for Quality Gate, CLEAN/DWD, DWS/ADS, final fusion and module-readiness steps; each heartbeat updates both step elapsed time and pipeline status.
- Replaced the raw log list with a monitoring console for poll health, heartbeat silence, progress, level/step/search filters, pause/resume, manual refresh, ordering and filtered copy.
- Retained up to 5,000 unique rows in memory while rendering at most 600, and added backend retries for concurrent sequence allocation conflicts.
- Removed schema DDL from read-only polling and persisted the latest pipeline ID per MySQL context so monitoring resumes after page navigation.
- Stored newly generated pipeline timestamps in UTC and rendered pipeline and execution logs in the local PC timezone, including timezone metadata in copied text. Legacy timezone-less `DATETIME` rows cannot be converted reliably.
- Version markers are synchronized to `1.0.43`.

## 1.0.42 update

- Removed automatic batch-table preparation, registry counting, module inspection and dashboard loading from application startup, restored-context changes and analysis-page navigation.
- Each decision page now exposes an explicit on-demand task with its dataset plan, progress, current step, partial failure state, retry and stop-after-current-query behavior.
- The current page loads only its required DWS / ADS datasets; overview loads its five datasets sequentially instead of starting six queries concurrently.
- Advanced diagnostics mount only after expansion, and ECharts is dynamically imported only when a chart is rendered.
- Version markers are synchronized to `1.0.42`.

## 1.0.41 update

- RAW import emits a database-backed heartbeat every five seconds with processed bytes, source size and percentage for both LOAD DATA and Streaming INSERT.
- A 100% client transfer is explicitly separated from MySQL parse/index/commit time; 30 seconds without byte movement produces a phase-specific warning without falsely failing the job.
- The newly created import batch is bound to `meta_pipeline_run` before RAW loading, so the UI can expose the batch ID while the long-running statement is still active.
- Version markers are synchronized to `1.0.41`.

## 1.0.40 update

- Fixed the Rust MySQL client path that executed `LOAD DATA LOCAL INFILE` without a local-infile handler, which made the client send an empty payload and MySQL report zero rows without warnings.
- The selected CSV is now transferred with a bounded 1 MiB buffer; the application does not load the whole file into memory.
- The per-import handler canonicalizes and permits only the exact user-selected file, rejects any other server-requested local path, and is removed immediately after the statement.
- Version markers are synchronized to `1.0.40`.

## 1.0.39 update

- RAW imports now reject zero-row outcomes before Quality Gate. LOAD DATA records the detected delimiter, verifies batch-visible rows and surfaces up to 20 MySQL warnings; Streaming INSERT also rejects header-only/zero-row files.
- Probe, mapping validation, LOAD DATA and Streaming INSERT consistently use the bounded delimiter detection result for comma, Tab or semicolon input.
- Dataset profiles resolve the current batch physical RAW table and no longer report the empty shared base table; `--` is excluded from distinct identity counts.
- Pipeline failures automatically load RAW status, table registry evidence and failed Quality Gate items in the failure card.
- Added customer TCP aliases for throughput average bandwidth and users average effective download rate.
- Version markers are synchronized to `1.0.39`.

## 1.0.38 update

- Added protected single/bulk deletion for historical import batches, including per-batch tables, shared legacy rows, analysis results and job metadata.
- MySQL password now starts with the requested built-in default `123456`; user overrides remain in memory only.
- Replaced the oversized utf8mb4 network-hotspot composite primary key with an auto-increment key and bounded prefix lookup index.
- Version markers are synchronized to `1.0.38`.

## 1.0.37 update

- Registered the Tauri Dialog plugin and default dialog permission so the packaged Windows app can open the CSV picker.
- Added visible file-dialog failure feedback instead of leaving rejected dialog invocations silent.
- TCP / Game imports now require an explicit published IP rule-set selection and per-import confirmation; the backend validates and binds that exact version instead of selecting the latest version automatically.
- Version markers are synchronized to `1.0.37`.

## 1.0.36 update

- Added `.github/workflows/build-desktop.yml` for Windows MSI, NSIS EXE and portable EXE builds.
- Pushes to `dev` and `codex/**` publish 30-day Actions artifacts; `v*` tags additionally publish GitHub Releases.
- CI runs frontend type-check, Rust tests and the Tauri production build before publishing files.
- Version markers are synchronized to `1.0.36`.

## 1.0.35 update

- Added versioned IPv4 access classification with editable drafts, CIDR/range normalization, overlap validation, bounded batch preview, atomic publish and explicit batch assignment.
- TCP/Game DWD rows now retain source access type, final access type, local IP, classification source/confidence, rule ID and rule-set version. IP rules take precedence over the source field.
- Rebuilt the primary UI around six decision views: overview, app experience, network action, Cable vs FTTH, users and qualified opportunities.
- Added real App-grain and user-App DWS aggregates, topology-grain network hotspots and evidence-aware lead exclusions.
- Fixed batch physical-table placeholders across RAW quality, CLEAN, DWS and ADS scripts, and made fatal Quality Gate errors stop downstream execution while warnings remain visible.
- Reordered mapped CSV headers now stay on the `LOAD DATA LOCAL INFILE` path instead of falling back to 500-row INSERT batches; retries reset only the dedicated RAW batch table.
- The automatic pipeline now materializes all structured ADS views after DWS and refreshes lead evidence after final fusion.
- Version markers are synchronized to `1.0.35`.

## Verification

- `npm run check` and `npm run build`: passed on 2026-08-25; Vite reports the existing non-blocking JavaScript chunk-size warning.
- Targeted Rust format check and `cd src-tauri && cargo check --offline`: passed on 2026-08-25 with existing dead-code/unused warnings.
- `cd src-tauri && CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --offline`: passed on 2026-08-25, 42 tests. Generated Cargo build artifacts were cleaned once after the root filesystem filled during the first linker attempt.
- Network hotspot migration has a regression test for the bounded utf8mb4 index definition; target MySQL execution is not yet smoke-tested.
- Historical batch deletion is compiled and unit-tested for status/table targeting, but destructive execution against a real target MySQL batch was not run.
- Packaged Windows CSV picker, database initialization, batch deletion, RAW progress heartbeat and a real import with manually selected rules remain to be smoke-tested from the next Windows artifact.
- The scoped LOCAL INFILE handler, zero-row rejection, MySQL warning capture and physical-table dataset profile routing are compiled/unit-tested but not smoke-tested against a live MySQL 8.0 instance.
- Real MySQL / customer CSV smoke: not run; no MySQL service or representative 1 GB+ fixture was available.

## Performance assessment

- The main path remains bounded-memory: header-mapped `LOAD DATA LOCAL INFILE` into per-batch RAW tables, MySQL set-based DWD/DWS/ADS transforms, and paged ADS reads.
- This is architecturally suitable for 1 GB+ CSV input, but production performance is not yet proven. A representative benchmark must record import throughput, DWD/DWS/ADS duration, peak MySQL memory/disk, temporary-table spill and final dashboard latency.

## 1.0.34 update

- App / Hourly / Network / User / Lead structured analytics read commands now prefer materialized Analytics ADS tables when the current `analysis_run_id` has rows.
- If Analytics ADS tables are absent or empty, the commands fall back to the previous DWS / Lead query paths.
- Evidence hints now include `source=...` so the UI can distinguish ADS-first reads from fallback reads.
- README, package, Tauri config, Cargo, Workbench header and mapping catalog are synchronized to `1.0.34`.

## 1.0.34 verification at handoff

- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV smoke: not run.

## Next work

1. Run migration and end-to-end smoke on MySQL 8.0 with TCP and Game batches.
2. Benchmark a representative 1 GB+ CSV on target hardware and tune MySQL buffer/temp settings from evidence.
3. Add CRM, coverage and reachability fixtures to validate final A1 eligibility, not only SA-derived opportunity scoring.

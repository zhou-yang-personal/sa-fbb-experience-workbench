# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
2.0.0-alpha.1
```

## Source-of-truth branch

```text
codex/task-duckdb-runtime-poc
```

## Current baseline

The 2.0 target runtime is local DuckDB + Parquet; the 1.x MySQL path remains an explicit compatibility path during migration:

```text
CSV + SHA-256 manifest → DuckDB streaming transform → partitioned Parquet DWD → DuckDB DWS/ADS → atomic publish → dashboard/export
```

## 2.0.0-alpha.1 update

- Added an embedded DuckDB workspace with versioned batch/run/step metadata and no MySQL connection requirement.
- Added the first TCP/video vertical slice: source SHA-256 and manifest, header-alias resolution, partitioned Parquet, IP-range Cable/FTTH classification, hourly access aggregation and access summary publication.
- Added panic/error closure so active batch/run/step state is marked failed instead of remaining stale running.
- Added a default DuckDB panel in System Management; the previous MySQL connection workflow is folded under compatibility mode.
- Added the bundled DuckDB/Parquet Rust dependency and generated `src-tauri/Cargo.lock` for reproducible native builds.
- `npm run check`, `npm run build` and `cargo test --release --no-fail-fast` passed; 73 Rust tests passed with existing/example warnings only.
- The release benchmark harness completed the synthetic TCP fixture end to end: 4 source rows, 3 valid Parquet rows, 3 hourly rows and 2 Access summary rows in about 222 ms. This validates execution shape only, not 3–4 GiB production performance.
- This alpha supports TCP/video only. Game, full DWS/ADS parity, existing dashboards, opportunities, export parity, Windows package size and real 3–4 GiB performance remain pending.
- No production database query, ETL, aggregation or customer-data operation was executed; verification used only a synthetic temporary workspace under `/tmp`.

## 1.0.73 update

- Fixed leading blank PDF pages by collapsing the interactive workspace during Decision report printing instead of hiding it with `visibility:hidden` while retaining layout space.
- Reworded upstream-rate availability to reflect the actual evidence boundary: no confirmed mapped upstream-throughput field; source CSV headers/vendor dictionaries still require verification.
- Added App decision evidence with actual values, rule thresholds, numerators/denominators and the explicit distinction between affected App users and an App-server root cause.
- Added a five-dimension Cable/FTTH radar using fixed anchors, raw-value companions and missing-dimension suppression; it remains descriptive and non-causal.
- Reworked user/access bands into share-and-count bars. Cable and FTTH widths use within-cohort denominators rather than raw cross-cohort maxima.
- Added candidate-specific reasons, exact rule checks and priority derivation. The current Opportunity model is disclosed as High/Standard rules with no composite score.
- Bound hourly peak shading to the analysis rule profile and broke lines across missing hours.
- `npm run check`, `npm run build`, `cargo check --offline` and all 69 Rust tests passed. Whole-crate `cargo fmt --check` still reports the existing unformatted baseline across many untouched Rust files, so no repository-wide formatting rewrite was applied.
- No schema migration, dependency, lock-file, production database, ETL, aggregation or recovery change. Existing published results are read-only inputs; Windows/WebView2 PDF visual verification remains pending.
- `AGENTS.project.md` remains unchanged because editing it requires explicit user authorization.

## 1.0.72 update

- Fixed the stale-running root cause: nullable legacy `source_version` checkpoints no longer panic during Rust decoding, and all three background pipeline entry points now close failure state after unexpected panics.
- Checkpoint reuse now requires exact implementation and source versions at both subtask and hourly-partition levels. Upstream recomputation invalidates every downstream checkpoint.
- Advanced the shared hourly and period core contracts to V4, Access to `access_specialty_v3` and Opportunity to `opportunity_feature_v3`. This deliberately forces one compatible recomputation instead of retaining the current mixed publication.
- Added dashboard/export readiness gates. The existing Access=0 and old Opportunity=23,669 mixed state will be rejected until a complete same-version publication succeeds.
- Added null-safe Opportunity candidate/export decoding and additive average-download hourly rollups; UI unavailable copy has a non-`undefined` fallback.
- Existing RAW/CLEAN for `BATCH_7ee2e638909346fabf8396cd3660b9c0` remain reusable. Do not reimport CSV or rebuild CLEAN solely for this repair.
- `cargo test --offline` passed with 69 tests; `npm run check` and `npm run build` passed. The project has no `npm test` script.
- No production database command or long-running job was executed. Live migration, one explicit DWS/ADS resume, sample-count reconciliation and Windows export verification remain pending.
- `AGENTS.project.md` remains unchanged because editing it requires explicit user authorization.

## 1.0.71 update

- Added a separate candidate-detail CSV export beside the opportunity list. It exports all rows matching the active opportunity type and applied IP/App search, not only the visible page.
- The backend streams the published `ads_opportunity_user_v3` result directly to the selected path and includes complete evidence, limitations and rule-version columns.
- CSV is UTF-8 with BOM for Windows Excel. Chart PDF remains chart-only.
- `npm run check`, `npm run build`, `cargo check --offline` and all 64 Rust tests passed.
- Live Windows save-dialog, Excel and real-MySQL export verification remain pending. `AGENTS.project.md` remains unchanged because editing it requires explicit user authorization.

## 1.0.70 update

- Added a metric-first panorama workflow covering overall values, versioned user bands, 24-hour trends, App rankings and issue highlighting; unsupported upstream rate remains explicitly unavailable.
- Rebuilt Cable/FTTH as a four-level specialty view: overall deltas, dual hourly curves, cohort distributions and same-App sample-aware comparison.
- Added paged opportunity candidates keyed by analysis IP, filters/search and an evidence-detail modal. Dashboard queries only read published results.
- Replaced the former monolithic opportunity transaction with reusable user features, four independently checkpointed candidate stages and atomic publication. Opportunity generation no longer scans the hourly core twice.
- Aggregation implementation/source versions are now checked per subtask: existing successful V3 core/dashboard checkpoints remain reusable, while only new or incompatible access/opportunity tasks run.
- Added migration `012_opportunity_workbench_schema.sql` for access/user aggregates, hourly/band ADS, opportunity features and candidate staging.
- Added versioned distribution boundaries to the decision-rule profile. `npm run check`, `cargo check` and 64 Rust tests passed; production build and live MySQL execution are recorded separately below.
- Business pages now auto-read bounded DWS/ADS after the operator enters an analysis context; the repeated Load button was removed, while all import/rebuild/materialization actions remain explicit in Data Jobs.
- Live Windows/WebView2 and 13.2M-row MySQL runtime/output reconciliation remain pending. `AGENTS.project.md` remains unchanged because editing it requires explicit user authorization.

## 1.0.69 update

- Replaced bottom-appended App drill-down content with an in-viewport modal containing the App baseline, user distributions and explicit loading/error/empty states.
- Added Escape, backdrop and button dismissal plus request-generation protection so a late response cannot overwrite the latest selected App.
- Removed the duplicated User perspective tab; the same user distributions remain in the Metric perspective, so no analytical content or backend query capability was removed.
- `npm run check` and `npm run build` passed.
- This is a frontend interaction change; no database, aggregation, dependency or lock-file behavior changed. Live Windows/WebView2 visual verification remains pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.68 update

- Scoped the legacy Analytics PDF visibility reset to its own print-source mode so it no longer hides the V3 full-chart report during printing.
- Added a dedicated Decision Workspace print-source mode and waits for document fonts/layout before opening the Windows/WebView2 print dialog.
- `npm run check` and `npm run build` passed; production CSS inspection confirms there is no unconditional print-time body visibility reset.
- This is a frontend-only export fix; no database, aggregation, dependency or lock-file behavior changed. Live Windows/WebView2 PDF verification remains pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.67 update

- Promoted the partitioned user × App × hour V2 table to the shared analytical core. It now stores additive SUM/COUNT state for every reusable metric, quality-driver counts, traffic and duration.
- Period V2 and the four legacy compatibility DWS tables now derive from the shared core; they no longer rescan TCP/Game DWD independently.
- Added a dedicated `experience_core` stage before compatibility DWS/ADS. Hour partitions remain independently committed and resumable.
- Aggregation subtask and partition checkpoints now bind implementation/source versions. Old successful checkpoints are invalidated when their executable contract changes instead of being silently reused.
- Network/path and other incompatible-grain aggregations remain separate by design. Live 13.2M-row Windows/MySQL runtime and output reconciliation remain pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.66 update

- Replaced the DWS/ADS application-only heartbeat with a 15-second MySQL activity probe covering `PROCESSLIST`, the aggregation named-lock owner, the active aggregation subtask and hourly partition checkpoint.
- Confirmed statements report connection ID, MySQL elapsed seconds, state and SQL preview. Three consecutive samples without a batch statement become a warning and explicitly identify a suspected stall.
- Probe failures are warnings and never claim SQL liveness. The task monitor now summarizes the latest database activity result instead of repeating an application-thread disclaimer.
- No schema, aggregation result, dependency or lock-file behavior changed. Live Windows/MySQL verification remains pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.65 update

- Fixed execution-log drawer overflow by constraining every nested grid, list, entry and message block to the drawer's content width.
- Split filtering controls from copy actions so buttons wrap as a group instead of forcing a five-column toolbar beyond the frame.
- Long SQL, command names, timestamps and errors now wrap safely; each message keeps its bounded vertical scrolling behavior.
- No database, aggregation, dependency or lock-file behavior changed. Live Windows/WebView2 visual verification remains pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.64 update

- Fixed MySQL 1411 during RAW rebuild by validating User Account and Local IP with `IS_IPV4()` before every `INET_ATON()` call in TCP/Game clean, V2 access fallback, manual reaggregation and access-rule preview.
- RAW-to-CLEAN now loads bounded 500,000-row RAW primary-key ranges as separate committed ETL steps with per-SQL connection, duration and affected-row logs.
- Dedicated per-batch CLEAN tables are truncated once; large secondary indexes are removed during bulk load and rebuilt once afterward, including the hourly partition index. Index restoration is attempted even when a chunk fails.
- A database named lock serializes RAW-to-CLEAN work so two operator actions cannot compete for buffer pool, redo and disk bandwidth.
- Full quality validation now runs after CLEAN and reuses normalized DWD timestamp, identity, access and quality columns instead of reparsing the full RAW table with regular expressions.
- This release changes application SQL/schema defaults but does not edit the target machine's `my.ini`. Real 13.2M-row Windows/MySQL runtime remains pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.63 update

- Fixed the access-rule import confirmation checkbox at 16px in the product theme so generic input width and padding cannot stretch it.
- The confirmation copy now consumes the remaining column and wraps safely; stale-takeover confirmations retain the same layout contract.
- No database, aggregation, dependency or lock-file behavior changed. Live Windows/WebView2 visual verification remains pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.62 update

- Rebuilt the execution-log drawer as a single-header, single-scroll-region layout instead of nesting a full page panel inside a scrolling drawer.
- Stabilized search, status and copy controls with a responsive grid and bounded long-text rendering.
- Backdrop clicks now close the drawer while clicks inside the drawer remain isolated.
- No database, aggregation, dependency or lock-file behavior changed. Live Windows/WebView2 visual verification remains pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.61 update

- Removed the repeated hero, version card, global-scope placeholder, technical context and duplicate batch/run block from the analysis landing area.
- Reduced the selected batch and page actions to compact toolbars; analysis-path controls render only after the operator creates a drill-down filter.
- Moved language and version controls to the sidebar so the default viewport prioritizes analytical content.
- No query, aggregation, schema or dependency behavior changed. Live Windows/WebView2 visual verification remains pending.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.60 update

- Replaced the 11-entry dashboard shell with five operator entries: Panorama, Poor Quality, Cable/FTTH, Opportunities and Data Center. Rules and diagnostics are secondary.
- Added a decision workspace backed by the existing V2 DWS: metric panorama, unique-App portfolio, user distributions, issue-side evidence and access comparison. These reads do not scan RAW.
- Added migration 011 with versioned decision rules, run-level snapshots, four opportunity outputs and subtask checkpoints.
- Added configurable thresholds for sample eligibility, persistent quality, problem Apps, heavy use, peak hours and opportunity qualification.
- Existing RAW batches can regenerate TCP effective/video duration, average download rate, throughput, connection success/delay and fluency into CLEAN/DWD and the user-by-App DWS; no CSV re-import is required.
- Added migration, speed-upgrade, mesh/AP and App Bundle opportunity materialization as an independently logged pipeline subtask. Mesh/AP validates evidence coverage before generating candidates.
- DWS/ADS resume now skips completed subtasks and retries only incomplete work; hourly partitions retain their finer-grained checkpoints.
- TCP/Game user keys now use IPv4 only (User Account first, Local IP fallback). MAC and non-IP accounts no longer define analytical users.
- PDF export prepares all non-empty business chart sections for the selected batch/run and excludes the interactive App detail table by default.
- `npm run check`, `npm run build`, Linux `cargo check` and 57 Rust tests passed. Live Windows/WebView2 and customer MySQL migration/SQL execution remain to be verified.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.59 update

- Restored the Overview route as a comprehensive, explicitly loaded DWS/ADS dashboard. Findings remains a separate exception list and investigation entry rather than replacing the full insight view.
- Application Experience now retains every loaded App-by-access combination and highlights policy-derived Attention/Severe states alongside Normal, Insufficient Sample and legacy Unclassified rows.
- Added full App status composition, eligible-user coverage, traffic context and a numerator/denominator evidence table with shared-context drill-down.
- Added dashboard command panic isolation and nullable App/network ADS decoding. The coverage query no longer performs migrations during a read action.
- Frontend type check/build and low-disk Rust check passed on Linux. Live Windows/WebView2 and customer MySQL validation is still required.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.58 update

- Replaced panicking `mysql_common` conversions across Finding, investigation evidence, hourly evidence, Server IP and saved-investigation reads with nullable fallible row decoding.
- Normalized legacy Finding and coverage payloads before rendering and made number/status formatting tolerant of null or malformed values.
- Added an analysis-workspace error boundary so an unexpected WebView render error shows a recoverable message instead of taking down the product UI.
- Frontend type check, production build and low-disk `cargo check --offline` passed on Linux; the existing Rust warnings remain. Live Windows/MySQL validation is still required.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.57 update

- Fixed the two stale-takeover confirmations in RAW rebuild and DWS/ADS resume. Product-shell input defaults no longer stretch their checkboxes to full width.
- The checkbox is fixed at 16px while the safety explanation receives the remaining width and wraps naturally.
- The change is scoped to stale-takeover controls and does not alter text/select inputs or access-rule confirmation behavior.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.56 update

- Replaced the full-batch hourly V2 `INSERT ... SELECT` with date/hour partitions. Every partition runs in its own transaction and is committed independently.
- Added `meta_aggregation_partition_checkpoint` with connection ID, attempt count, duration, affected rows and error evidence. A resumed run skips successful partitions and retries only incomplete ones.
- Added a one-time `(import_batch_id, stat_date, hour_of_day)` index to each batch TCP/Game DWD table before hourly partitioning, avoiding a full DWD scan for every hour.
- Added a MySQL named lock so only one DWS/ADS aggregation may run per local database instance.
- Pipeline status reconciliation marks work interrupted by a MySQL restart as `interrupted`; analysis readiness now requires a successful/degraded run and cannot expose partial V2 rows.
- Full import, RAW rebuild and DWS/ADS resume now share statement-level logs. SQL entries include connection ID, duration, affected rows and preview; partition entries provide progress and recovery checkpoints.
- MySQL 4 GiB buffer pool / 1 GiB redo configuration was applied separately on the target Windows database by the operator; this repository does not edit `my.ini`.
- Frontend type check/build, Rust check and 53 Rust tests passed on Linux. This does not prove the customer 13.2M-row runtime; the next Windows artifact must be tested by resuming the same batch without reimporting CSV.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.55 update

- Import and batch operations are split into three operator workspaces: New Import, Batch Library and Running Tasks.
- Batch selection is now a local context action only. Pipeline status, logs and polling are loaded explicitly, preserving the zero-database-command startup boundary.
- Batch Library is a searchable status table with next-action guidance for running, failed, analysis-ready and RAW-only batches.
- Starting a new import, RAW rebuild or DWS/ADS resume routes directly to the shared task monitor; failed tasks link back to recovery choices.
- Duplicate advanced actions were removed from the primary flow. Manual ETL steps and diagnostic output remain available inside developer diagnostics.
- Operator simulation, before/after scoring and Windows acceptance steps are recorded in `docs/design/import-job-center-operator-journey.md`.
- Frontend type check/build and 48 Rust tests passed; live Windows/WebView2 and customer MySQL interaction remain unverified here.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.54 update

- Batch refresh no longer chains an automatic analysis-run query when a batch ID is restored from local context; selecting a batch also leaves run-list loading as an explicit user action.
- Nullable text decoding is now guarded in registered-table resolution, shared batch-table resolution, analysis-run batch lookup, batch data type and registry listing, covering the second `mysql_common` panic path observed after 1.0.53.
- Import page mount no longer automatically queries published access rules, history batches or a stored pipeline. Runtime logs now identify starts and successful completion of the two refresh commands.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.53 update

- Fixed a Windows process crash when refreshing batches if legacy `meta_import_batch` rows contain `NULL` in text metadata that current schema declares non-null.
- Batch listing now applies SQL-side fallbacks and fallible row-by-row Rust decoding instead of an infallible typed tuple conversion; malformed metadata is shown with compatibility labels rather than triggering a `mysql_common` panic.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.52 update

- Application startup no longer invokes MySQL. Batch and analysis-run lists load only after an explicit refresh action.
- Added an explicit existing-RAW rebuild pipeline that preserves CSV/RAW and old run-scoped ADS, regenerates Quality Gate/CLEAN/DWS/ADS/V2 under a new `RUN_REBUILD_*`, and rejects concurrent batch SQL.
- Core SQL scripts now emit statement-level RUNNING/SUCCESS/FAILED entries with duration, affected rows and a bounded statement preview into `meta_pipeline_log` during RAW rebuild.
- Windows writes credential-free runtime lifecycle and Rust panic evidence to `%LOCALAPPDATA%\\SA FBB Experience Workbench\\runtime.log`.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.51 update

- Batch selection now prefers the latest successful/degraded analysis run. A completed manual V2 run such as `RUN_REAGG_V2_20260825` is no longer hidden by an older pipeline run.
- The analysis context includes a metadata-only run selector with Period V2, App ADS, Hourly V2 and bound policy readiness. It does not scan RAW or DWD.
- V2 foundation loading isolates status, findings, coverage and comparison failures. Available datasets remain visible, while missing V2 rows and failed datasets receive explicit messages.
- `AGENTS.project.md` still records 1.0.48 and remains unchanged because editing it requires explicit user authorization.

## 1.0.50 update

- New database migration: `009_investigation_workspace_schema.sql`.
- New reusable tables: `dws_user_app_hourly_experience_v2`, `ads_app_hourly_experience_v2`; batch table registry now creates per-batch physical copies.
- New commands: V2 status/findings/coverage, previous comparable-run verification, investigation user/hour/controlled Server-IP evidence, investigation save/list, experience policy/profile list/draft/update/clone/publish.
- The user may still be running `database/sql/manual/reaggregate_current_batch_experience_v2.sql`. That script creates the period V2 tables only; do not interrupt it. Hourly V2 is materialized by the 1.0.50 App materialization path after the new binary/database initialization is used.
- Linux validation cannot verify customer MySQL values or 13.2M-row runtime. Do not claim the hourly scan performance is proven until tested on the Windows dataset.
- New CLEAN runs retain `server_ip`. Investigation parses it only after an App is selected, first limits the scope to 200 priority affected users through DWS, then reads at most 20,000 indexed DWD observations. Existing CLEAN tables gain the nullable column but do not retroactively recover values; rerun CLEAN before expecting Server-IP evidence.
- Overview V2 compares against the latest earlier successful run only when access rule, Others, App mapping and experience-policy bindings are identical. If no such run exists, it reports not comparable instead of manufacturing a trend.

- Access-rule drafts no longer inherit or default Others. Users explicitly map unmatched valid IPv4 addresses to Cable, FTTH or Other; missing/invalid IP remains unavailable, and CSV access fields are evidence only.
- Analysis runs bind the access rule and published experience policy. V2 distinguishes poor observations, ever-affected users, persistent-poor users and severe-poor users with exact numerators, denominators and minimum-sample status.
- App materialization produces V2 DWS/ADS and App queries prefer it. The existing App dashboard now exposes persistent, observation, ever-affected and severe views and excludes insufficient samples from problem charts.
- Every current chart and PDF chart position has a reusable bilingual explanation block. The 1.0.50 primary navigation, Analysis Context and Investigation Workspace use the global Chinese/English switch; some legacy compatibility-page operational copy remains Chinese-first.
- Version markers are synchronized to `1.0.50`, except `AGENTS.project.md`; project governance requires explicit user authorization before editing that file.

### Current-batch SQL path

- Run `database/migrations/008_experience_analysis_policy_schema.sql`, then `database/sql/manual/reaggregate_current_batch_experience_v2.sql` on the Windows MySQL database.
- The manual job reads the existing DWD tables for `BATCH_7ae0c7d1c0a240ba833e366bf755397d`, expects the bound rule version to explicitly contain Others → Cable, writes `RUN_REAGG_V2_20260825`, and does not reimport RAW or overwrite `RUN_MANUAL_001`.
- Linux source checks passed, but the 13,205,379-row MySQL run is not verified here because no database service is listening on `127.0.0.1:3306`. Save the command output as `experience-v2-reaggregate-result.txt` for result review.

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

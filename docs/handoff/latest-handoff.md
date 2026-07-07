# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.24
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project includes the Phase 1-7 complete application baseline plus the 1.0.23 asynchronous import pipeline workflow and the 1.0.24 NBSP/log overflow hotfix.

Core path remains:

```text
CSV file selection
→ MySQL RAW import
→ RAW Quality Gate
→ RAW to CLEAN / DWD
→ DWS / ADS aggregation
→ SA Lead / optional Final Lead fusion
→ Dashboard and export
```

## 1.0.24 update

- TCP / Game Quality Gate SQL no longer uses executable `CHAR(160)` for NBSP replacement.
- TCP / Game RAW→CLEAN SQL no longer uses executable `CHAR(160)` for NBSP replacement.
- Timestamp NBSP replacement now uses `CONVERT(0xC2A0 USING utf8mb4)`, preventing MySQL `ERROR 3854 Cannot convert string '\xA0' from binary to utf8mb4`.
- Timestamp cleanup still replaces tab / LF / CR / NBSP with spaces, compresses repeated whitespace, and parses only guarded `stat_time_text`.
- Rust SQL-template tests were updated to require `CONVERT(0xC2A0 USING utf8mb4)` and reject `CHAR(160)`.
- Pipeline plan rows, failure card text, and realtime log entries now have CSS overflow guards so long SQL statements no longer stretch the Import page indefinitely.
- `package.json`, README, changelog and this handoff were updated to `1.0.24`.
- `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `WorkbenchHeader.tsx` and `mapping_catalog.rs` version updates were attempted but blocked by ChatGPT GitHub connector safety checks; these remain pending for local/Codex follow-up.

## Important rules

1. Always read `AGENTS.md`, `AGENTS.common.md`, `AGENTS.project.md` and `docs/design/current-core-design.md` before design or code changes.
2. CSV files must first be imported into MySQL RAW tables.
3. Do not perform full in-memory cleaning of large CSV files.
4. Dashboard pages must query DWS / ADS tables instead of RAW tables.
5. Do not submit customer CSV files, database exports, local logs, build outputs or installers.
6. Current 1.0.24 preserves the Raw First MySQL import path and only hotfixes NBSP parsing plus long-log display overflow.

## Not verified

- Real MySQL and TCP / Game CSV end-to-end smoke must be checked from the latest delivery report.
- Customer real CSV validation has not been recorded in this document.
- Lead query/export and batch switching non-contamination must be checked with the 1.0.18 checklist when a MySQL test schema and sample CSVs are available.
- Full version synchronization is incomplete because several config/source version files were blocked by the connector safety layer.

## Latest connector-side verification

- Repository search found no remaining `CHAR(160)` occurrences after the 1.0.24 SQL/test changes.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV smoke has not been executed yet.

## Next recommended work

1. Locally update the blocked version files to `1.0.24`: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src/features/workbench/WorkbenchHeader.tsx`, `src-tauri/src/mapping_catalog.rs`.
2. Run `npm run check`, `npm run build`, `cd src-tauri && cargo check`, `cd src-tauri && cargo test -- --nocapture`, and `npm run tauri:build` locally.
3. Re-run the TCP CSV pipeline that previously failed at `tcp_raw_quality_gate` and verify the NBSP / `0xA0` error is gone.
4. If full modal-based log details are still required, implement that locally or via Codex; current connector-safe UI fix is CSS overflow containment, not a new TSX modal.

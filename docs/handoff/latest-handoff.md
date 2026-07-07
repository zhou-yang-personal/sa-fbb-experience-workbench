# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.26
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project includes the Phase 1-7 complete application baseline, the 1.0.23 asynchronous import pipeline workflow, the 1.0.24 NBSP/log overflow hotfix, the 1.0.25 analytics cockpit UI, and the 1.0.26 analytics evidence-table enhancement.

Core path remains:

```text
CSV file selection
→ MySQL RAW import
→ RAW Quality Gate
→ RAW to CLEAN / DWD
→ DWS / ADS aggregation
→ SA Lead / optional Final Lead fusion
→ Analytics cockpit / evidence table / export
```

## 1.0.26 update

- Added `src/features/workbench/AnalyticsEvidenceTable.tsx`.
- Evidence tables support keyword search, absolute-value threshold filtering, sorting, CSV export and row-level detail drawer.
- `AnalyticsDashboard.tsx` now supports additional chart kinds: scatter, heatmap and funnel.
- Overview adds an Insight Strip for demand entry, experience risk entry, Cable/FTTH evidence entry and lead entry.
- App, quality, Cable vs FTTH, user and lead tabs now use evidence tables instead of static metric rows.
- `AnalyticsDashboard.css` was extended for evidence toolbar, sticky table, row detail drawer and larger table viewport.
- The implementation still reuses existing DWS / ADS-backed commands and does not add RAW scans, dependencies, lock changes or database schema changes.
- `package.json`, `src-tauri/tauri.conf.json`, `mapping_catalog.rs`, README, changelog and this handoff were updated to `1.0.26`.
- `WorkbenchHeader.tsx` and `src-tauri/Cargo.toml` version updates were blocked by ChatGPT GitHub connector safety checks and remain pending for local/Codex follow-up.

## Important rules

1. Always read `AGENTS.md`, `AGENTS.common.md`, `AGENTS.project.md` and `docs/design/current-core-design.md` before design or code changes.
2. CSV files must first be imported into MySQL RAW tables.
3. Do not perform full in-memory cleaning of large CSV files.
4. Dashboard pages must query DWS / ADS tables instead of RAW tables.
5. Do not submit customer CSV files, database exports, local logs, build outputs or installers.
6. Current 1.0.26 preserves the Raw First MySQL import path and strengthens the Data Analysis product surface using existing DWS / ADS APIs.

## Not verified

- Real MySQL and TCP / Game CSV end-to-end dashboard smoke must be checked from the latest delivery report.
- Customer real CSV validation has not been recorded in this document.
- Lead query/export and batch switching non-contamination must be checked with the 1.0.18 checklist when a MySQL test schema and sample CSVs are available.
- Full version synchronization is incomplete because `WorkbenchHeader.tsx` and `src-tauri/Cargo.toml` were blocked by the connector safety layer.

## Latest connector-side verification

- GitHub connector diff confirms the evidence table component, enriched analytics dashboard, analytics CSS and documentation/version updates on `dev`.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

## Next recommended work

1. Locally update blocked `src-tauri/Cargo.toml` and `WorkbenchHeader.tsx` version markers to `1.0.26`.
2. Continue fast development with structured chart APIs and dedicated ADS tables for KPI summary, app rank, hourly trend, network hotspot, user profile and lead evidence.
3. Final validation can run later in one pass: `npm run check`, `npm run build`, `cd src-tauri && cargo check`, `cd src-tauri && cargo test -- --nocapture`, and `npm run tauri:build`.

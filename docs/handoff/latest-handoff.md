# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.25
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project includes the Phase 1-7 complete application baseline, the 1.0.23 asynchronous import pipeline workflow, the 1.0.24 NBSP/log overflow hotfix, and the 1.0.25 analytics cockpit UI.

Core path remains:

```text
CSV file selection
→ MySQL RAW import
→ RAW Quality Gate
→ RAW to CLEAN / DWD
→ DWS / ADS aggregation
→ SA Lead / optional Final Lead fusion
→ Analytics cockpit and export
```

## 1.0.25 update

- Data Analysis now defaults to a large-screen analytics cockpit instead of module cards and small charts.
- Added `src/features/workbench/AnalyticsDashboard.tsx`.
- Added `src/features/workbench/AnalyticsDashboard.css`.
- `AnalysisWorkspace` keeps batch selection and analysis context at the top, renders the cockpit as the main product surface, and moves module readiness / batch table registry to an advanced diagnostics section.
- `WorkbenchAppV2` imports the new analytics CSS and updates analysis guidance copy.
- The cockpit includes six tabs: overview, app experience, network quality, Cable vs FTTH, user profile and migration upsell leads.
- The first implementation intentionally reuses existing DWS / ADS-backed commands: overview, app category, experience quality, network quality, video detail, game experience, cable-fiber compare, cable-fiber hourly, user profile and lead summary.
- No RAW table scan, dependency change, lock update, database schema change or new ADS table was introduced in this pass.
- `package.json`, `src-tauri/tauri.conf.json`, `WorkbenchHeader.tsx`, `mapping_catalog.rs`, README, changelog and this handoff were updated to `1.0.25`.
- `src-tauri/Cargo.toml` version update was blocked by ChatGPT GitHub connector safety checks and remains pending for local/Codex follow-up.

## Important rules

1. Always read `AGENTS.md`, `AGENTS.common.md`, `AGENTS.project.md` and `docs/design/current-core-design.md` before design or code changes.
2. CSV files must first be imported into MySQL RAW tables.
3. Do not perform full in-memory cleaning of large CSV files.
4. Dashboard pages must query DWS / ADS tables instead of RAW tables.
5. Do not submit customer CSV files, database exports, local logs, build outputs or installers.
6. Current 1.0.25 preserves the Raw First MySQL import path and only strengthens the Data Analysis product surface using existing DWS / ADS APIs.

## Not verified

- Real MySQL and TCP / Game CSV end-to-end dashboard smoke must be checked from the latest delivery report.
- Customer real CSV validation has not been recorded in this document.
- Lead query/export and batch switching non-contamination must be checked with the 1.0.18 checklist when a MySQL test schema and sample CSVs are available.
- Full version synchronization is incomplete because `src-tauri/Cargo.toml` was blocked by the connector safety layer.

## Latest connector-side verification

- GitHub connector diff confirms the new analytics cockpit component, analytics CSS, AnalysisWorkspace integration and documentation/version updates on `dev`.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

## Next recommended work

1. Locally update blocked `src-tauri/Cargo.toml` to `1.0.25`.
2. Run `npm run check`, `npm run build`, `cd src-tauri && cargo check`, `cd src-tauri && cargo test -- --nocapture`, and `npm run tauri:build` locally.
3. Open Data Analysis after importing a real TCP/Game batch and verify the analytics cockpit loads KPI cards, large charts and tables.
4. Next engineering phase should add structured chart APIs and dedicated ADS tables for KPI summary, app rank, hourly trend, network hotspot, user profile and lead evidence.

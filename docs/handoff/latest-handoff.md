# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.29
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project includes the Phase 1-7 complete application baseline, the 1.0.23 asynchronous import pipeline workflow, the 1.0.24 NBSP/log overflow hotfix, the 1.0.25 analytics cockpit UI, the 1.0.26 analytics evidence-table enhancement, the 1.0.27 structured analytics API foundation, the 1.0.28 structured evidence-table upgrade, and the 1.0.29 structured-first cockpit integration.

Core path remains:

```text
CSV file selection
→ MySQL RAW import
→ RAW Quality Gate
→ RAW to CLEAN / DWD
→ DWS / ADS aggregation
→ SA Lead / optional Final Lead fusion
→ Structured analytics API / analytics cockpit / evidence table / export
```

## 1.0.29 update

- Added `src/features/workbench/analyticsStructuredCharts.ts` as the structured chart dataset adapter layer.
- `AnalyticsDashboard.tsx` now loads structured KPI, App Rank, Hourly Trend, Network Hotspot, User Profile and Lead Evidence through `analyticsStructuredApi` during refresh.
- The main analytics cockpit is now Structured First: chart datasets and evidence surfaces prefer structured Analytics ADS results, with legacy `workbenchApi` dashboard commands retained as fallback.
- Overview, Apps, Quality, Cable, Users and Leads tabs now expose structured evidence-table entries inside the cockpit instead of leaving structured evidence only in the panels above the dashboard.
- The cockpit hero, tab hints and chart footnotes now explicitly say the source strategy is structured DWS/ADS first and legacy DWS/ADS fallback.
- Version markers were synchronized to `1.0.29` in `package.json`, `src-tauri/tauri.conf.json`, `WorkbenchHeader.tsx`, `mapping_catalog.rs`, README, changelog and this handoff.
- `src-tauri/Cargo.toml` remains at `1.0.28` because ChatGPT GitHub connector safety checks blocked the TOML update payload. Update it locally or through Codex before treating version synchronization as fully complete.
- No dependency or lock-file changes were made.

## 1.0.28 update

- `AnalyticsStructuredKpiPanel.tsx` now keeps KPI cards and upgrades structured App Rank / Hourly Trend to `AnalyticsEvidenceTable`.
- `AnalyticsStructuredDeepDivePanel.tsx` now upgrades Network Hotspot / User Profile / Lead Evidence to `AnalyticsEvidenceTable`.
- Structured App / Hourly / Network / User / Lead results now support search, sorting, threshold filtering, CSV export and row detail drawers.
- `AnalyticsDashboard.css` adds structured evidence grid layouts for wide-screen two-column and three-column evidence tables, with responsive single-column fallback.
- Version markers were synchronized to `1.0.28` in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `WorkbenchHeader.tsx`, `mapping_catalog.rs`, README, changelog and this handoff.
- No dependency or lock-file changes were made.

## 1.0.27 update

- Added `database/migrations/006_analytics_dashboard_schema.sql` with dedicated analytics ADS tables for KPI summary, app rank, hourly trend, network hotspot, user profile and lead evidence.
- Wired `006_analytics_dashboard_schema.sql` into `src-tauri/src/migrations.rs`.
- Added structured analytics backend commands:
  - `analytics_get_kpi_summary`
  - `analytics_get_app_rank`
  - `analytics_get_hourly_trend`
  - `analytics_get_network_hotspots`
  - `analytics_get_user_profiles`
  - `analytics_get_lead_evidence`
- Registered the above commands in `src-tauri/src/main.rs`.
- Added `src/features/workbench/analyticsStructuredApi.ts` to centralize structured frontend Tauri invokes.
- `AnalysisWorkspace` renders structured analytics panels before the legacy analytics cockpit.
- All new analytics command reads stay on DWS / ADS tables and do not scan RAW.

## Important rules

1. Always read `AGENTS.md`, `AGENTS.common.md`, `AGENTS.project.md` and `docs/design/current-core-design.md` before design or code changes.
2. CSV files must first be imported into MySQL RAW tables.
3. Do not perform full in-memory cleaning of large CSV files.
4. Dashboard pages must query DWS / ADS tables instead of RAW tables.
5. Do not submit customer CSV files, database exports, local logs, build outputs or installers.
6. Current 1.0.29 preserves the Raw First MySQL import path and makes the main Data Analysis cockpit consume structured Analytics API outputs first.

## Not verified

- Real MySQL and TCP / Game CSV end-to-end dashboard smoke has not been executed in this connector session.
- Customer real CSV validation has not been recorded in this document.
- Build/test/check commands were intentionally not run in the ChatGPT GitHub connector environment.

## Latest connector-side verification

- GitHub connector writes succeeded for `analyticsStructuredCharts.ts`, `AnalyticsDashboard.tsx`, non-Cargo version files, README, changelog and this handoff on `dev`.
- `src-tauri/Cargo.toml` update was attempted once and blocked by ChatGPT GitHub connector safety checks; it was not retried with the same payload.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

## Next recommended work

1. Update `src-tauri/Cargo.toml` from `1.0.28` to `1.0.29` locally or through Codex to complete version synchronization.
2. Run the full local validation pass after the fast implementation phase.
3. Inspect the structured cockpit in a real MySQL batch and tune chart field priority if hint keys differ from the adapter assumptions.
4. Add backend pagination/filtering for structured analytics commands if real customer result size exceeds current command limits.
5. Continue DWS→ADS SQL generation for App/Hourly/Network/User/Lead tables using small SQL scripts to avoid connector safety blocks.

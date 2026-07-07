# CHANGELOG-dev

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

## 1.0.30 - 2026-07-07

### Added

- Added backend query controls to `DashboardRequest`: `page`, `page_size`, `keyword`, `sort_by` and `min_value` for structured analytics commands.
- Added paginated and filterable structured command behavior for App Rank, Hourly Trend, Network Hotspots and User Profiles.
- Added `analytics_get_lead_evidence_page` as a small dedicated paged Lead Evidence command.
- Registered `analytics_get_lead_evidence_page` in `src-tauri/src/main.rs` and routed frontend `analyticsStructuredApi.leadEvidence` to the paged command.

### Changed

- `analyticsStructuredApi.ts` now accepts optional structured query parameters and forwards them as snake_case Tauri request fields.
- `AnalyticsStructuredKpiPanel.tsx` requests backend `pageSize=500` for App Rank and Hourly Trend evidence tables.

### Verification

- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

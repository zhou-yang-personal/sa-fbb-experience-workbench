# CHANGELOG-dev

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

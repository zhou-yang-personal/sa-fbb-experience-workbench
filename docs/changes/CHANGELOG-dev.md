# CHANGELOG-dev

## 1.0.28 - 2026-07-07

### Changed

- Upgraded structured App Rank and Hourly Trend from compact preview tables to full `AnalyticsEvidenceTable` surfaces.
- Upgraded structured Network Hotspot, User Profile and Lead Evidence from compact preview tables to full `AnalyticsEvidenceTable` surfaces.
- Kept structured KPI cards in `AnalyticsStructuredKpiPanel`, while making all non-KPI structured results searchable, sortable, threshold-filterable, CSV-exportable and inspectable through row detail drawers.
- Expanded `AnalyticsDashboard.css` with structured evidence grid layouts for wide-screen two-column and three-column evidence-table panels.
- Synchronized version markers to `1.0.28` in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `WorkbenchHeader.tsx`, `mapping_catalog.rs`, README and changelog.

### Verification

- GitHub connector diff confirms structured evidence table upgrades, CSS layout updates and version/documentation updates on `dev`.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

## 1.0.27 - 2026-07-06

### Added

- Added `database/migrations/006_analytics_dashboard_schema.sql` with dedicated analytics ADS tables for KPI summary, app rank, hourly trend, network hotspot, user profile and lead evidence.
- Added `database/sql/dws_to_ads/003a_analytics_kpi_summary.sql` as the first compact analytics KPI generation script.
- Added structured analytics backend commands: `analytics_get_kpi_summary`, `analytics_get_app_rank`, `analytics_get_hourly_trend`, `analytics_get_network_hotspots`, `analytics_get_user_profiles`, and `analytics_get_lead_evidence`.
- Registered all structured analytics commands in Tauri `main.rs`.
- Added `AnalyticsStructuredKpiPanel.tsx` for KPI / App Rank / Hourly Trend previews.
- Added `AnalyticsStructuredDeepDivePanel.tsx` for Network Hotspot / User Profile / Lead Evidence previews.
- Added `analyticsStructuredApi.ts` to centralize structured frontend Tauri invokes outside component bodies.

### Changed

- Wired `006_analytics_dashboard_schema.sql` into `src-tauri/src/migrations.rs`.
- Synchronized version markers to `1.0.27` in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `WorkbenchHeader.tsx`, `mapping_catalog.rs`, README, changelog and handoff.
- Continued the dashboard hardening direction from MetricCard-only cockpit toward structured DWS/ADS-backed analytics APIs.
- Kept all new analytics reads on DWS/ADS physical tables; no RAW table scans were introduced.
- README and handoff now document the completed 1.0.27 structured analytics foundation.

### Blocked / Deferred

- `database/sql/dws_to_ads/003_analytics_enhanced_dashboards.sql` and later split App/Hourly/User/Lead SQL scripts triggered ChatGPT GitHub connector safety block, so only the compact KPI SQL script was added.
- `workbenchApi.ts` non-KPI wrapper update was avoided after safety block; structured analytics now uses `analyticsStructuredApi.ts` instead.
- Full structured chart datasets and full evidence-table UX for structured Network/User/Lead are deferred to the next fast development pass.

### Verification

- GitHub connector diff confirms schema, migration wiring, KPI SQL, analytics KPI/App/Hourly/Network/User/Lead commands, Tauri registration, structured frontend API, structured preview panels and documentation/version updates on `dev`.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

## 1.0.26 - 2026-07-06

### Added

- Added `AnalyticsEvidenceTable.tsx`, a reusable evidence table for analytics pages with search, sorting, minimum absolute value filtering, CSV export and row detail drawer.
- Added scatter, heatmap and funnel chart support inside `AnalyticsDashboard.tsx` for demand-vs-QoE proxy views, application heatmap views and lead funnel views.
- Added Insight Strip on the overview cockpit to expose business demand, experience risk, Cable/FTTH evidence and lead entry points.

### Changed

- Replaced static metric tables in the analytics cockpit with evidence tables across overview, app experience, network quality, Cable vs FTTH, user profile and migration lead tabs.
- Strengthened analytics CSS for evidence table toolbar, sticky table, details drawer, insight strip and larger table viewport.
- Reused existing DWS/ADS-backed dashboard commands for this fast development pass; no RAW table scans were added.
- Synchronized `package.json`, `tauri.conf.json`, `mapping_catalog.rs`, README, handoff and changelog to `1.0.26`; `WorkbenchHeader.tsx` and `src-tauri/Cargo.toml` updates were blocked by ChatGPT GitHub connector safety checks and remain pending.

### Verification

- GitHub connector diff confirms the evidence table component, enriched analytics dashboard, analytics CSS and version/documentation updates on `dev`.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

## 1.0.25 - 2026-07-06

### Added

- Added `AnalyticsDashboard.tsx`, a DWS/ADS-backed analytics cockpit for the Data Analysis area.
- Added `AnalyticsDashboard.css` for large-screen KPI cards, chart panels, tabs and sticky-header tables.
- Added six analysis tabs: overview cockpit, app experience, network quality, Cable vs FTTH, user profile and migration upsell leads.

### Changed

- Replaced the default Data Analysis module-card workflow with a large-screen analytics cockpit while keeping batch selection and diagnostics.
- Moved module readiness and batch table registry into an advanced diagnostics section instead of making it the primary analysis surface.
- Updated `WorkbenchAppV2` analysis guidance from small module charts to the current batch analytics cockpit.
- Reused existing DWS/ADS-backed dashboard commands for the first implementation; no RAW table scans were added.
- Synchronized `package.json`, `tauri.conf.json`, `WorkbenchHeader.tsx`, `mapping_catalog.rs`, README, handoff and changelog to `1.0.25`; `src-tauri/Cargo.toml` update was blocked by ChatGPT GitHub connector safety checks and remains pending.

### Verification

- GitHub connector diff confirms the new analytics cockpit component, analytics CSS, AnalysisWorkspace integration and version/documentation updates on `dev`.
- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo test -- --nocapture`: not run in ChatGPT GitHub connector environment.
- `npm run tauri:build`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

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

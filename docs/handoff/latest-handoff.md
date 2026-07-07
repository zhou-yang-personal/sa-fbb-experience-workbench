# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.31
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project keeps the Raw First MySQL pipeline:

```text
CSV file selection
→ MySQL RAW import
→ RAW Quality Gate
→ RAW to CLEAN / DWD
→ DWS / ADS aggregation
→ SA Lead / optional Final Lead fusion
→ Structured analytics API / analytics cockpit / evidence table / export
```

## 1.0.31 update

- Added `AnalyticsStructuredPagedPanel.tsx`.
- `AnalysisWorkspace.tsx` renders the paged structured panel before the main analytics cockpit.
- The new panel exposes backend page, pageSize, keyword, sort and minValue controls for App, Hourly, Network, User and Lead evidence.
- `batch_tables.rs` now registers the structured Analytics ADS tables from migration 006 as per-batch ADS tables.
- Added SQL materialization scripts:
  - `003b_analytics_app_rank.sql`
  - `003c_analytics_hourly_trend.sql`
  - `003d_analytics_user_profile.sql`
  - `003e_analytics_lead_evidence.sql`
  - `003f_analytics_network_hotspot.sql`
- `README.md`, `package.json` and `mapping_catalog.rs` are synchronized to `1.0.31`.

## Known connector blocks

- The Rust command to run the new 003b-003f SQL scripts was blocked by connector safety checks and is not wired yet.
- `CHANGELOG-dev.md` update to 1.0.31 was blocked.
- `src-tauri/tauri.conf.json`, `WorkbenchHeader.tsx` and `src-tauri/Cargo.toml` are not fully synchronized because prior connector writes were blocked.

## Not verified

- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV dashboard smoke has not been executed yet.

## Next work

1. Run local TypeScript and Rust checks.
2. Wire a safe Rust command for the 003b-003f materialization SQL.
3. Move structured commands from DWS fallback reads to materialized Analytics ADS reads after SQL smoke is confirmed.
4. Align blocked version files locally if connector continues to block them.

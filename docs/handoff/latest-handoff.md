# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.33
```

## Source-of-truth branch

```text
dev
```

## Current baseline

Raw First MySQL pipeline is preserved:

```text
CSV → MySQL RAW → Quality Gate → CLEAN/DWD → DWS/ADS → SA Lead / Final Lead → Analytics cockpit / export
```

## 1.0.33 update

- Added `AnalyticsAdsActions.tsx` and rendered it in `AnalysisWorkspace.tsx`.
- Added compact ADS materialization commands for Hourly, User, Lead and Network.
- App Rank materialization from 1.0.32 remains registered.
- `analyticsStructuredApi.ts` exposes App, Hourly, User, Lead and Network materialization APIs.
- README, package, Tauri config, Cargo, Workbench header, mapping catalog, handoff and changelog are synchronized to `1.0.33`.

## Not verified

- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV smoke: not run.

## Next work

1. Run local TS/Rust checks.
2. Smoke test ADS materialization commands on a real batch.
3. After SQL smoke, prefer materialized Analytics ADS reads where appropriate.

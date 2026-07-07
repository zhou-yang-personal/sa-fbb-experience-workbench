# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.34
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

## 1.0.34 update

- App / Hourly / Network / User / Lead structured analytics read commands now prefer materialized Analytics ADS tables when the current `analysis_run_id` has rows.
- If Analytics ADS tables are absent or empty, the commands fall back to the previous DWS / Lead query paths.
- Evidence hints now include `source=...` so the UI can distinguish ADS-first reads from fallback reads.
- README, package, Tauri config, Cargo, Workbench header and mapping catalog are synchronized to `1.0.34`.

## Not verified

- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV smoke: not run.

## Next work

1. Run local TS/Rust checks.
2. Smoke test ADS materialization commands on a real batch.
3. Confirm the ADS-first commands return ADS rows after materialization and fallback rows before materialization.

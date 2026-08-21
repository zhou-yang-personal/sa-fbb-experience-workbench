# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.37
```

## Source-of-truth branch

```text
codex/task-dashboard-ip-segmentation
```

## Current baseline

Raw First MySQL pipeline is preserved:

```text
CSV → MySQL RAW → Quality Gate → CLEAN/DWD → DWS/ADS → SA Lead / Final Lead → Analytics cockpit / export
```

## 1.0.37 update

- Registered the Tauri Dialog plugin and default dialog permission so the packaged Windows app can open the CSV picker.
- Added visible file-dialog failure feedback instead of leaving rejected dialog invocations silent.
- TCP / Game imports now require an explicit published IP rule-set selection and per-import confirmation; the backend validates and binds that exact version instead of selecting the latest version automatically.
- Version markers are synchronized to `1.0.37`.

## 1.0.36 update

- Added `.github/workflows/build-desktop.yml` for Windows MSI, NSIS EXE and portable EXE builds.
- Pushes to `dev` and `codex/**` publish 30-day Actions artifacts; `v*` tags additionally publish GitHub Releases.
- CI runs frontend type-check, Rust tests and the Tauri production build before publishing files.
- Version markers are synchronized to `1.0.36`.

## 1.0.35 update

- Added versioned IPv4 access classification with editable drafts, CIDR/range normalization, overlap validation, bounded batch preview, atomic publish and explicit batch assignment.
- TCP/Game DWD rows now retain source access type, final access type, local IP, classification source/confidence, rule ID and rule-set version. IP rules take precedence over the source field.
- Rebuilt the primary UI around six decision views: overview, app experience, network action, Cable vs FTTH, users and qualified opportunities.
- Added real App-grain and user-App DWS aggregates, topology-grain network hotspots and evidence-aware lead exclusions.
- Fixed batch physical-table placeholders across RAW quality, CLEAN, DWS and ADS scripts, and made fatal Quality Gate errors stop downstream execution while warnings remain visible.
- Reordered mapped CSV headers now stay on the `LOAD DATA LOCAL INFILE` path instead of falling back to 500-row INSERT batches; retries reset only the dedicated RAW batch table.
- The automatic pipeline now materializes all structured ADS views after DWS and refreshes lead evidence after final fusion.
- Version markers are synchronized to `1.0.35`.

## Verification

- `npm run check`: passed on 2026-08-21.
- `npm run build`: passed on 2026-08-21; Vite reports a non-blocking JavaScript chunk-size warning.
- `cd src-tauri && cargo check`: passed on 2026-08-21 with existing dead-code/unused warnings.
- `cd src-tauri && cargo test --offline`: passed, 27 tests.
- Packaged Windows CSV picker interaction and a real MySQL import with manually selected rules remain to be smoke-tested from the 1.0.37 Windows artifact.
- Real MySQL / customer CSV smoke: not run; no MySQL service or representative 1 GB+ fixture was available.

## Performance assessment

- The main path remains bounded-memory: header-mapped `LOAD DATA LOCAL INFILE` into per-batch RAW tables, MySQL set-based DWD/DWS/ADS transforms, and paged ADS reads.
- This is architecturally suitable for 1 GB+ CSV input, but production performance is not yet proven. A representative benchmark must record import throughput, DWD/DWS/ADS duration, peak MySQL memory/disk, temporary-table spill and final dashboard latency.

## 1.0.34 update

- App / Hourly / Network / User / Lead structured analytics read commands now prefer materialized Analytics ADS tables when the current `analysis_run_id` has rows.
- If Analytics ADS tables are absent or empty, the commands fall back to the previous DWS / Lead query paths.
- Evidence hints now include `source=...` so the UI can distinguish ADS-first reads from fallback reads.
- README, package, Tauri config, Cargo, Workbench header and mapping catalog are synchronized to `1.0.34`.

## 1.0.34 verification at handoff

- `npm run check`: not run in ChatGPT GitHub connector environment.
- `npm run build`: not run in ChatGPT GitHub connector environment.
- `cd src-tauri && cargo check`: not run in ChatGPT GitHub connector environment.
- Real MySQL / customer CSV smoke: not run.

## Next work

1. Run migration and end-to-end smoke on MySQL 8.0 with TCP and Game batches.
2. Benchmark a representative 1 GB+ CSV on target hardware and tune MySQL buffer/temp settings from evidence.
3. Add CRM, coverage and reachability fixtures to validate final A1 eligibility, not only SA-derived opportunity scoring.

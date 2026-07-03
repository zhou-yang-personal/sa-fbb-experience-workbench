# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
0.1.0
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project has been initialized with:

- Governance files.
- Detailed architecture design.
- React + TypeScript + Vite frontend skeleton.
- Tauri 2 / Rust command skeleton.
- MySQL metadata / dim / raw / dwd / dws / ads schema baseline.
- RAW to CLEAN SQL templates for TCP and Game data.
- DWS and ADS SQL templates for user profile and migration leads.

## Important rules

1. Always read `AGENTS.md`, `AGENTS.common.md`, `AGENTS.project.md` and `docs/design/current-core-design.md` before design or code changes.
2. CSV files must first be imported into MySQL RAW tables.
3. Do not perform full in-memory cleaning of large CSV files.
4. Dashboard pages must query DWS / ADS tables instead of RAW tables.
5. Do not submit customer CSV files, database exports, local logs, build outputs or installers.

## Next recommended work

1. Run `npm install` locally.
2. Run `npm run build`.
3. Run `cd src-tauri && cargo check`.
4. Replace Rust command stubs with real MySQL connection and job orchestration.
5. Implement `LOAD DATA LOCAL INFILE` import path.
6. Implement streaming INSERT fallback.
7. Connect frontend pages to Tauri commands.

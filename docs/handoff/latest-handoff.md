# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
0.2.1
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project now includes the initial usable workflow implementation:

- Governance files.
- Detailed architecture design.
- React + TypeScript + Vite frontend workflow UI.
- Tauri 2 / Rust command implementation.
- MySQL metadata / dim / raw / dwd / dws / ads schema baseline.
- Database initialization command.
- MySQL connection command.
- Import batch command.
- RAW load command path.
- RAW quality report command.
- RAW to CLEAN SQL runner.
- CLEAN to DWS and DWS to ADS SQL runner.
- Dashboard overview command.
- Migration lead query and CSV export command.

## Important rules

1. Always read `AGENTS.md`, `AGENTS.common.md`, `AGENTS.project.md` and `docs/design/current-core-design.md` before design or code changes.
2. CSV files must first be imported into MySQL RAW tables.
3. Do not perform full in-memory cleaning of large CSV files.
4. Dashboard pages must query DWS / ADS tables instead of RAW tables.
5. Do not submit customer CSV files, database exports, local logs, build outputs or installers.

## Not verified

- Dependency installation was not run.
- Frontend build was not run.
- Rust check was not run.
- Tauri package build was not run.
- Real MySQL and CSV end-to-end flow was not executed.

## Next recommended work

1. Run local dependency installation and build checks.
2. Fix compile errors if any.
3. Validate MySQL bulk import settings.
4. Validate CSV column order against RAW tables.
5. Replace fast probe with full header and preview parsing after local compile check.
6. Harden RAW import fallback and SQL quality gates.

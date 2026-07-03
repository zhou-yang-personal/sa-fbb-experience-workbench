# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.0
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project now includes the Phase 1-7 complete application baseline:

- Governance files and detailed architecture design.
- React + TypeScript + Vite workflow UI.
- Tauri 2 / Rust command implementation.
- MySQL metadata / dim / raw / dwd / dws / ads schema baseline.
- Extended schema for CRM, FTTH coverage, reachability, dashboard ADS and final marketing leads.
- Database initialization command with both core and extended schema.
- CSV probe, import batch and RAW load command path.
- RAW quality gate SQL and command.
- RAW to CLEAN SQL runner.
- Complete DWS aggregate SQL and command.
- Complete ADS dashboard SQL and command.
- Migration lead scoring and final CRM / coverage / reachability fusion SQL.
- Dashboard commands for Overview, App Category, Experience Quality and Cable vs FTTH.
- Lead query, final lead summary and CSV export command.

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
- `src-tauri/Cargo.toml` version remains `0.2.1` because connector update was blocked by platform safety checks.

## Next recommended work

1. Run local dependency installation and build checks.
2. Fix compile errors if any.
3. Manually synchronize `src-tauri/Cargo.toml` package version to `1.0.0` if needed.
4. Validate MySQL bulk import settings and CSV column order.
5. Validate Phase 1-7 SQL chain on sample data.
6. Replace any remaining baseline SQL with measured production SQL after local test.

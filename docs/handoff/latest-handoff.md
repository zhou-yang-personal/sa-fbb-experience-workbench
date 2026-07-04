# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.3
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project now includes the Phase 1-7 complete application baseline plus deviation-fix rounds:

- Governance files and detailed architecture design.
- React + TypeScript + Vite workflow UI.
- Tauri 2 / Rust command implementation.
- MySQL metadata / dim / raw / dwd / dws / ads schema baseline.
- Extended schema for CRM, FTTH coverage, reachability, dashboard ADS and final marketing leads.
- CRM, coverage and reachability RAW tables now include source trace fields.
- Database initialization command with both core and extended schema.
- CSV probe, import batch and RAW load command path.
- RAW import supports TCP, Game, CRM, FTTH Coverage and Reachability data types.
- RAW import uses explicit column lists for all supported data types.
- RAW import has streaming INSERT fallback selected by `mode=streaming_insert` or `local_infile=false`.
- Streaming fallback updates import progress while inserting chunks.
- RAW import writes `total_rows` and `imported_rows` for row reconciliation.
- RAW quality gate checks row count, CSV vs RAW row diff, identity, access mix, time range, active hours, app count and topology UNKNOWN.
- RAW to CLEAN SQL runner.
- ETL job commands write `meta_etl_job` and `meta_etl_job_step` for step status and failure diagnostics.
- Import status and recent ETL job status commands are available from the UI.
- Complete DWS aggregate SQL and command.
- Complete ADS dashboard SQL and command.
- Migration lead scoring and final CRM / coverage / reachability fusion SQL.
- Dashboard commands for Overview, App Category, Experience Quality and Cable vs FTTH.
- Lead query, final lead summary and full paginated CSV export command.

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
3. Validate all five import data types on small samples.
4. Validate final CRM / coverage / reachability lead fusion with real mapping keys.
5. Add final lead table query and export UI if required.
6. Replace remaining baseline SQL with measured production SQL after local test.

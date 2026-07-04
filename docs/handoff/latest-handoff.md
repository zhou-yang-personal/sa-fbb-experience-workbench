# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.7
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
- CRM, coverage and reachability RAW tables include source trace fields.
- Database initialization command with core, extended, mapping and observability schema.
- CSV probe, import batch and RAW load command path.
- RAW import supports TCP, Game, CRM, FTTH Coverage and Reachability data types.
- RAW import has original implementation plus `raw_import_v2.rs` mapped implementation.
- `raw_import_v2.rs` reads `cfg_import_field_mapping` and uses header aliases for mapped streaming INSERT.
- RAW import command is routed through mapped import adapter.
- RAW import has streaming INSERT fallback selected by `mode=streaming_insert` or `local_infile=false`.
- Streaming fallback updates import progress while inserting chunks.
- RAW import writes `total_rows` and `imported_rows` for row reconciliation.
- RAW quality gate checks row count, CSV vs RAW row diff, identity, access mix, time range, active hours, app count and topology UNKNOWN.
- Mapping validation command writes `meta_mapping_validation_result`.
- Mapping validation summary and detail result commands are registered.
- Dataset profile refresh now writes row count, source line range, distinct accounts, distinct MACs and data-type-specific dimensions.
- Dataset profile query reads `meta_dataset_profile` into frontend metric cards.
- Observability schema includes row errors, mapping validation and dataset profile tables.
- RAW to CLEAN SQL runner.
- ETL job commands write `meta_etl_job` and `meta_etl_job_step` for step status and failure diagnostics.
- ETL job step inspection commands are registered.
- Quality gate result inspection commands are registered.
- Complete DWS aggregate SQL and command.
- Complete ADS dashboard SQL and command.
- Migration lead scoring and final CRM / coverage / reachability fusion SQL.
- Final fusion adds commercial action separation: identity mapping, blacklist, arrears, contract, reachability, coverage/build and market actions.
- Configurable import mapping schema: `cfg_import_field_mapping`.
- Configurable final join rule schema: `cfg_final_join_rule`.
- Default mapping and join rule seed: `database/seeds/002_default_mapping_seed.sql`.
- Command handlers are split into import, ETL, dashboard, lead, config and phase modules.
- `main.rs` now registers split command modules, config commands, mapped import, mapping validation, mapping result, dataset profile, ETL step inspection and quality result inspection.
- Final Lead fusion command now uses the configurable fusion builder.
- Modular frontend shell exists in `src/features/workbench/WorkbenchAppV2.tsx` and is mounted from `src/main.tsx`.
- Reusable frontend components exist for connection, import, operations, quality, ETL, dashboard, lead, metric grid, result tables, execution log and ECharts metric bar.
- Import Center exposes mapping validation, mapping summary, mapping results, dataset profile refresh and dataset profile view.
- Dashboard commands for Overview, App Category, Experience Quality and Cable vs FTTH.
- Lead query, final lead query, final lead summary, SA Lead CSV export and Final Lead CSV export commands.

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
- `src-tauri/tauri.conf.json` remains at the previous version because connector JSON update was blocked by platform safety checks.

## Next recommended work

1. Complete remaining blocked wiring locally/Codex: Tauri version.
2. Run local dependency installation and build checks.
3. Fix compile errors if any.
4. Validate all five import data types on small samples.
5. Validate final CRM / coverage / reachability lead fusion with real mapping keys.
6. Continue UI componentization for Quality Center, ETL Job Center and Dashboard Center after local compile feedback.

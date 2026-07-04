# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.11
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
- Dataset profile refresh writes row count, source line range, distinct accounts, distinct MACs and data-type-specific dimensions.
- Dataset profile query reads `meta_dataset_profile` into frontend metric cards.
- Observability schema includes row errors, mapping validation and dataset profile tables.
- RAW to CLEAN SQL runner.
- ETL job commands write `meta_etl_job` and `meta_etl_job_step` for step status and failure diagnostics.
- ETL job step inspection commands are registered.
- Structured ETL step inspection command `etl_get_job_steps` is registered.
- `EtlJobStepRow` exposes job_id, job_type, step_name, target_table, status, affected_rows, started_at, finished_at and message.
- ETL Job Center renders a structured Step Detail table and Failed Detail shortcut.
- Quality gate result inspection commands are registered.
- Complete DWS aggregate SQL and command.
- Complete ADS dashboard SQL and command.
- Migration lead scoring and final CRM / coverage / reachability fusion SQL.
- Final fusion adds commercial action separation: identity mapping, blacklist, arrears, contract, reachability, coverage/build and market actions.
- Configurable import mapping schema: `cfg_import_field_mapping`.
- Configurable final join rule schema: `cfg_final_join_rule`.
- Default mapping and join rule seed: `database/seeds/002_default_mapping_seed.sql`.
- Command handlers are split into import, ETL, dashboard, lead, config and phase modules.
- `main.rs` registers split command modules, config commands, mapped import, mapping validation, mapping result, dataset profile, ETL step inspection, structured ETL step detail and quality result inspection.
- Final Lead fusion command uses the configurable fusion builder.
- Modular frontend shell exists in `src/features/workbench/WorkbenchAppV2.tsx` and is mounted from `src/main.tsx`.
- Import Center exposes mapping validation, mapping summary, mapping results, dataset profile refresh and dataset profile view.
- Quality Center, ETL Job Center, Dashboard Center and Final Lead Center are separated frontend panels.
- ResultTables supports SA Lead / Final Lead text filters, type/action filters, pagination and empty states.
- Final Lead results support final_action summary pills with click-to-filter behavior.
- Final Lead Center exposes export filename presets for SA Lead, Final Lead, Market Upsell and Reachability lists.
- Lead query backend supports page / page_size / lead_type / final_action / keyword request fields.
- SA Lead query applies server-side pagination plus optional lead_type and keyword filters.
- Final Lead query applies server-side pagination plus optional final_action and keyword filters.
- Final Lead CSV export supports `final_actions` filtering for action-specific delivery files.
- Export presets now set both output filename and Final Action export scope.
- Market Upsell preset exports `MARKET_FIBER_UPSELL` and `FTTH_SPEED_UPSELL`.
- Reachability preset exports `REACHABILITY_FIX_FIRST` and `BUILD_OR_COVERAGE_CHECK`.
- Reusable frontend components exist for connection, import, quality, ETL, dashboard, lead, metric grid, result tables, pagination, execution log and ECharts metric bar.
- Dashboard commands for Overview, App Category, Experience Quality and Cable vs FTTH.
- Lead query, final lead query, final lead summary, SA Lead CSV export and Final Lead CSV export commands.
- Package, Cargo and Tauri app config are synchronized to `1.0.11`.

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
3. Validate server-side Lead query filtering, final-action export filters and ETL Step Detail against real MySQL data.
4. Validate all five import data types on small samples.
5. Continue deeper UI refinement after local compile feedback: richer Dashboard Center charts and old-entry legacy cleanup.

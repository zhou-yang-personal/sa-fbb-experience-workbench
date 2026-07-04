# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.15
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project now includes the Phase 1-7 complete application baseline plus guided interaction fixes:

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
- `src/main.tsx` is the active React entry and mounts `WorkbenchAppV2` directly.
- `src/App.tsx` is a compatibility wrapper that forwards to `WorkbenchAppV2`.
- `src/features/workbench/WorkbenchApp.tsx` is a compatibility wrapper that forwards to `WorkbenchAppV2`.
- `OpsPanel` and `TaskCenter2` are legacy placeholders retained for stale imports only.
- Import Center exposes mapping validation, mapping summary, mapping results, dataset profile refresh and dataset profile view.
- Quality Center, ETL Job Center, Dashboard Center and Final Lead Center are separated frontend panels.
- Dashboard Center supports multi-chart panels for App Category, Experience Quality, Cable vs FTTH and Final Action Mix.
- `DashboardCharts` renders reusable ECharts bar / radar panels from ADS / DWS metric results.
- DashboardActions can load individual chart groups or all dashboard chart groups in one action.
- ResultTables supports SA Lead / Final Lead text filters, type/action filters, pagination and empty states.
- Final Lead results support final_action summary pills with click-to-filter behavior.
- Final Lead Center exposes export filename presets for SA Lead, Final Lead, Market Upsell and Reachability lists.
- Lead query backend supports page / page_size / lead_type / final_action / keyword request fields.
- SA Lead query applies server-side pagination plus optional lead_type and keyword filters.
- Final Lead query applies server-side pagination plus optional final_action and keyword filters.
- Final Lead CSV export supports `final_actions` filtering for action-specific delivery files.
- Export presets now use system save dialogs and set Final Action export scope.
- Guided UI adds 5-step navigation: Start / Import / Validate / Analyze / Results.
- Guided UI adds pipeline status bar, next-action hint, action feedback bar and Run Log drawer.
- Guided UI adds `ActionButton` for running / success / failure / disabled button states.
- Import uses a system CSV file picker in the main flow; manual path entry is kept only in advanced mode.
- Import main action runs probe, create batch, mapping validation, RAW load, status refresh and dataset profile refresh.
- Analyze main action runs RAW→CLEAN, aggregate and Final Fusion as one business action.
- Package, Cargo, Tauri app config, README and handoff version markers are synchronized to `1.0.15`.

## Important rules

1. Always read `AGENTS.md`, `AGENTS.common.md`, `AGENTS.project.md` and `docs/design/current-core-design.md` before design or code changes.
2. CSV files must first be imported into MySQL RAW tables.
3. Do not perform full in-memory cleaning of large CSV files.
4. Dashboard pages must query DWS / ADS tables instead of RAW tables.
5. Do not submit customer CSV files, database exports, local logs, build outputs or installers.

## Not verified

- Dependency installation was not run in ChatGPT GitHub connector environment.
- Frontend build was not run in ChatGPT GitHub connector environment.
- Rust check was not run in ChatGPT GitHub connector environment.
- Tauri package build was not run in ChatGPT GitHub connector environment.
- Real MySQL and CSV end-to-end flow was not executed in this round.
- `QualityCenter.tsx` was not replaced because connector safety checks blocked large-file rewrite; guided quality component exists but the main route still uses the existing Quality Center.

## Next recommended work

1. Run local dependency installation and build checks.
2. Run `npm run check`, `npm run build`, `cd src-tauri && cargo check`, and `npm run tauri:build`.
3. If build passes, finish the remaining Validate-page route switch locally or through Codex.
4. Run a real CSV smoke test: Start → Import → Validate → Analyze → Results & Export.

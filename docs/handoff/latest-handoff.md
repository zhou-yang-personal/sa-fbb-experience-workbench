# SA FBB Experience Workbench｜Latest Handoff

## Current version

```text
1.0.22
```

## Source-of-truth branch

```text
dev
```

## Current baseline

The project now includes the Phase 1-7 complete application baseline plus 1.0.22 import-to-analysis workflow closure and CLEAN timestamp compatibility hardening:

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
- Export presets use system save dialogs and set Final Action export scope.
- 1.0.17 replaced the 5-step guided navigation with the product function tree: Data Analysis / Data Import / System Management.
- Data Analysis is the default entry and requires an import batch before dashboard use.
- `AnalysisWorkspace` declares module required fields, applicable data types and required aggregate tables, then disables unavailable modules.
- `ImportPanel` requires a readable batch name before creating an import batch.
- `meta_import_batch.batch_display_name` is added to schema and auto-added for existing local databases when creating or checking a batch.
- Mapping required-field failure messages now list missing target fields instead of only a count.
- `ExecutionLog` is promoted to diagnostic log and can copy all logs or failure logs.
- 1.0.18 hardens batch physical table SQL replacement to only replace table identifiers in supported SQL positions and quoted table names.
- 1.0.18 records physical table names in ETL Job Step source / target metadata for clean, aggregate, dashboard and final fusion jobs.
- 1.0.18 module status checks data_type support, physical table existence, physical row_count, required field presence via `information_schema.columns`, and optional ADS `analysis_run_id` rows.
- 1.0.18 adds business-backed module dashboard commands for Game, Network, User Profile, Video detail and Cable/FTTH hourly detail.
- 1.0.18 changes module CSV export to query business ADS / DWS / DWD physical tables instead of exporting module status / registry metadata.
- 1.0.18 extends `meta_etl_job_step.source_table` and `target_table` to `VARCHAR(512)` for multi-table physical diagnostics.
- 1.0.19 fixes Universal Video detail CSV header normalization so NBSP, tabs, repeated whitespace, parentheses, `%`, slash and other non ASCII alphanumeric separators normalize to `_`.
- 1.0.19 makes mapping validation and mapped RAW import share the same header normalization helper.
- 1.0.19 changes `tcp.user_type` and `game.user_type` mapping required flags to optional; missing user_type no longer blocks RAW import.
- 1.0.19 adds Universal Video detail aliases for subscriber account, user MAC, application, statistics duration, downloaded data volume and effective download duration.
- 1.0.19 mapping validation diagnostics include candidate aliases, normalized aliases and normalized CSV headers.
- 1.0.20 adds mapping catalog self-heal before validation, RAW load, atomic import and config seed.
- 1.0.20 adds mapping catalog health metrics for app version, mapping seed version, applied time, stale status and critical alias gaps.
- 1.0.20 moves the main import button to backend `import_current_file_atomic`.
- 1.0.20 enforces required mapping gate inside RAW load so direct command invocation cannot bypass validation.
- 1.0.20 marks batches `failed` on mapping validation or RAW load failure instead of leaving pending batches.
- 1.0.20 truncates frontend missing-required normalized header previews to the first 20 headers while preserving full details in mapping results.
- 1.0.21 keeps the left navigation to three entries: Data Import, Data Analysis and System Management.
- 1.0.21 moves Quality Gate, RAW to CLEAN, DWS/ADS and Module Ready into the Data Import main workflow.
- 1.0.21 fixes CLEAN time parsing by normalizing tab, LF, CR and NBSP before guarded `STR_TO_DATE`.
- 1.0.21 routes clean jobs by batch data_type so tcp batches run only tcp clean and game batches run only game clean.
- 1.0.21 improves module status text for missing table, rows=0 result not generated, missing analysis_run_id rows, not applicable data_type and missing fields.
- 1.0.22 extends CLEAN timestamp parsing to support one- or two-digit slash day/month formats such as `20/9/2025 23:58:06`.
- 1.0.22 replaces tab, LF, CR and NBSP with spaces before timestamp parsing and compresses repeated whitespace, so middle invisible separators do not join date/time text.
- 1.0.22 keeps SA Lead / `ads_migration_lead_user` as the availability basis for `migration_lead`; Final Lead missing or empty results now produce a degraded readiness note instead of disabling the module.
- Package, Cargo, Tauri app config, README, handoff, header and mapping catalog version markers are synchronized to `1.0.22`.

## Important rules

1. Always read `AGENTS.md`, `AGENTS.common.md`, `AGENTS.project.md` and `docs/design/current-core-design.md` before design or code changes.
2. CSV files must first be imported into MySQL RAW tables.
3. Do not perform full in-memory cleaning of large CSV files.
4. Dashboard pages must query DWS / ADS tables instead of RAW tables.
5. Do not submit customer CSV files, database exports, local logs, build outputs or installers.
6. Current 1.0.22 closes CLEAN timestamp compatibility and Final Lead degradation behavior while preserving the Raw First MySQL import path.

## Not verified

- Real MySQL and TCP / Game CSV end-to-end smoke must be checked from the latest delivery report.
- Customer real CSV validation has not been recorded in this document.
- Lead query/export and batch switching non-contamination must be checked with the 1.0.18 checklist when a MySQL test schema and sample CSVs are available.

## Latest local verification

- `npm run check`: passed.
- `npm run build`: passed; Vite reported the existing large chunk warning.
- `cd src-tauri && cargo check`: passed with existing dead_code warnings.
- `cd src-tauri && cargo test -- --nocapture`: passed, 18 tests passed.
- `npm run tauri:build`: passed and produced ignored local Linux bundles under `src-tauri/target`.
- Synthetic Universal Video header coverage was verified by Rust tests.
- CLEAN SQL invisible-character handling and clean data_type routing were verified by Rust tests.
- Real Universal Video detail CSV import smoke must be checked from the latest delivery report; customer CSV / MySQL smoke status is not recorded as passed here unless the delivery report says so.

## Next recommended work

1. Run `docs/validation/mysql-smoke-checklist-1.0.18.md` against a clean MySQL test schema.
2. Use real customer-shaped TCP and Game CSVs when available; if synthetic CSVs are used, mark the smoke as synthetic-only.
3. Validate Lead query/export and batch switching non-contamination before merge if MySQL access is available.

# MySQL Smoke Checklist 1.0.18

This checklist validates the 1.0.18 batch-first physical table path. Mark results with the exact date, schema name, CSV source and whether data is real customer-shaped data or synthetic data.

## 1. Environment

- MySQL 8.0 is reachable from the desktop app.
- Test schema is new or intentionally cleared.
- App version shows `1.0.18-dev`.
- `npm run check`, `npm run build`, `cd src-tauri && cargo check`, and `npm run tauri:build` have been run or their failure is recorded.

Suggested setup:

```sql
DROP DATABASE IF EXISTS sa_fbb_smoke_1018;
CREATE DATABASE sa_fbb_smoke_1018 DEFAULT CHARACTER SET utf8mb4;
```

## 2. Initialize And Seed

1. Connect to `sa_fbb_smoke_1018`.
2. Run `db_initialize`.
3. Run `config_seed_defaults`.
4. Confirm core tables exist:

```sql
SELECT table_name
FROM information_schema.tables
WHERE table_schema = DATABASE()
  AND table_name IN (
    'meta_import_batch',
    'meta_batch_table_registry',
    'meta_batch_module_status',
    'raw_tcp_detail_import',
    'raw_game_detail_import',
    'dwd_tcp_detail_clean',
    'dwd_game_detail_clean',
    'dws_user_daily_profile',
    'ads_dashboard_overview',
    'ads_final_marketing_lead_user'
  )
ORDER BY table_name;
```

## 3. TCP CSV Batch

1. Import a named TCP CSV batch.
2. Run mapping validation.
3. Run RAW load.
4. Run RAW quality gate.
5. Run RAW to CLEAN.
6. Run base aggregate.
7. Run complete aggregates.
8. Run complete dashboards with a known `analysis_run_id`.

Physical table checks:

```sql
SELECT logical_table_name, physical_table_name, row_count, status
FROM meta_batch_table_registry
WHERE import_batch_id = '<TCP_BATCH_ID>'
ORDER BY layer, logical_table_name;

SELECT COUNT(*) FROM dwd_tcp_detail_clean__<tcp_batch_short_id>;
SELECT COUNT(*) FROM dws_user_daily_profile__<tcp_batch_short_id>;
SELECT COUNT(*) FROM ads_dashboard_overview__<tcp_batch_short_id> WHERE analysis_run_id = '<RUN_ID>';
SELECT COUNT(*) FROM ads_experience_quality_summary__<tcp_batch_short_id> WHERE analysis_run_id = '<RUN_ID>';
```

## 4. Game CSV Batch

1. Import a named Game CSV batch.
2. Run mapping validation.
3. Run RAW load.
4. Run RAW quality gate.
5. Run RAW to CLEAN.
6. Run base aggregate.
7. Run complete aggregates.
8. Run complete dashboards with a known `analysis_run_id`.

Physical table checks:

```sql
SELECT logical_table_name, physical_table_name, row_count, status
FROM meta_batch_table_registry
WHERE import_batch_id = '<GAME_BATCH_ID>'
ORDER BY layer, logical_table_name;

SELECT COUNT(*) FROM dwd_game_detail_clean__<game_batch_short_id>;
SELECT COUNT(*) FROM dws_user_daily_profile__<game_batch_short_id>;
```

## 5. Module Status

Run `analysis_get_module_status` for both batches. Confirm:

- Unsupported `data_type` modules are disabled with a data_type reason.
- Empty or missing physical tables are listed in `missing_tables`.
- `missing_required_fields` lists only genuinely missing physical fields.
- Logical fields are not reported as missing physical columns.
- If `analysis_run_id` is supplied, ADS modules require rows for that run.

SQL inspection:

```sql
SELECT module_id, enabled, data_type, missing_required_fields, missing_tables, row_count, status_text
FROM meta_batch_module_status
WHERE import_batch_id = '<BATCH_ID>'
ORDER BY module_id;
```

## 6. Business Dashboards

Refresh these modules for the selected batch:

- Overview
- App Usage
- Video Experience Detail
- Game Experience
- Network Quality
- Cable / FTTH Compare
- User Profile
- Migration Lead

Confirm Game, Network and User Profile do not return generic module status cards.

## 7. Lead Query And Export

1. Run migration lead generation.
2. Run final fusion.
3. Query SA Lead and Final Lead tables.
4. Export SA Lead CSV.
5. Export Final Lead CSV.
6. Export module CSV for `migration_lead`.

Checks:

```sql
SELECT lead_type, COUNT(*)
FROM ads_migration_lead_user__<batch_short_id>
WHERE analysis_run_id = '<RUN_ID>'
GROUP BY lead_type;

SELECT COALESCE(final_action, 'UNKNOWN') AS final_action, COUNT(*)
FROM ads_final_marketing_lead_user__<batch_short_id>
WHERE analysis_run_id = '<RUN_ID>'
GROUP BY COALESCE(final_action, 'UNKNOWN');
```

## 8. Batch Switching Non-Contamination

1. Select TCP batch and refresh dashboards.
2. Select Game batch and refresh dashboards.
3. Switch back to TCP.
4. Confirm displayed metrics, module status and exports use the selected batch physical tables.

SQL checks:

```sql
SELECT import_batch_id, physical_table_name
FROM meta_batch_table_registry
WHERE physical_table_name LIKE '%__<tcp_batch_short_id>'
   OR physical_table_name LIKE '%__<game_batch_short_id>'
ORDER BY import_batch_id, physical_table_name;

SELECT COUNT(*) AS cross_batch_rows
FROM dws_user_daily_profile__<tcp_batch_short_id>
WHERE import_batch_id <> '<TCP_BATCH_ID>';
```

Expected `cross_batch_rows = 0`.

## 9. Result Record

Record:

- Date:
- Operator:
- Schema:
- TCP CSV source:
- Game CSV source:
- Synthetic data used: yes / no
- Build commands result:
- MySQL smoke result:
- Lead export result:
- Batch switching result:
- Known failures:

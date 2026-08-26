# Manual V2 reaggregation

This directory contains a side-by-side V2 reaggregation for the verified batch
`BATCH_7ae0c7d1c0a240ba833e366bf755397d`. It reuses the existing DWD tables and
does not import or scan RAW. It writes a new analysis run
`RUN_REAGG_V2_20260825`, so `RUN_MANUAL_001` remains intact.

## Execution order

Run from the repository root. The MySQL client prompts for the password; do not
put the password in shell history.

```bash
mysql --protocol=TCP -h 127.0.0.1 -P 3306 -u app_root -p sa_vbp \
  < database/migrations/008_experience_analysis_policy_schema.sql

mysql --protocol=TCP -h 127.0.0.1 -P 3306 -u app_root -p sa_vbp \
  < database/sql/manual/reaggregate_current_batch_experience_v2.sql \
  | tee experience-v2-reaggregate-result.txt
```

Do not execute the reaggregation while another SQL task for the same batch is
running. The prerequisite guard intentionally raises a CHECK error and prevents
a successful run status if the exact batch, rule version, published policy or
explicit `Others = CABLE` mapping is missing. The first result set must show one
policy binding row. The final result sets must show:

- `unknown_access_rows = 0`;
- `one_of_one_hundred_persistent = 0`;
- nonzero user/App rows and App ADS rows;
- `RUN_REAGG_V2_20260825` in `success` status.

## Scope and limitations

- The current batch contains TCP data only. Game remains not imported; no Game
  metric is converted to zero.
- The baseline policy values are provisional configuration data. They must be
  reviewed against representative distributions before production use.
- The script does not split `server_ip` and does not build a Server-IP fact table.
- App-level average QoE values in the V2 DWS are user-weighted averages of the
  user/App period means; the four rate metrics retain exact observation/user
  numerators and denominators.

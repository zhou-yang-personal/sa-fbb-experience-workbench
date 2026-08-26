-- SA FBB Experience Workbench dashboard review export
-- Output columns: 说明, 内容
-- Scope: BATCH_7ae0c7d1c0a240ba833e366bf755397d / RUN_MANUAL_001
-- Compatibility: MySQL 8.0; static SELECT only; no variables, procedure, PREPARE, temp table or write operation.
--
-- Before execution:
--   1. Select the sa_vbp database in the SQL client.
--   2. Replace BOTH occurrences of CHANGE_THIS_PRIVATE_SALT with the same private random string.
--   3. Disable the SQL client's Limit Rows setting so every result row can be exported.
--   4. Export the single result grid as UTF-8 CSV or tab-delimited TXT.

USE sa_vbp;

SELECT export_data.description AS `说明`, export_data.content AS `内容`
FROM (
    SELECT
        0 AS sort_order,
        '00_批次上下文' AS description,
        CAST(JSON_OBJECT(
            'import_batch_id', import_batch_id,
            'batch_display_name', batch_display_name,
            'data_type', data_type,
            'source_file_name', source_file_name,
            'source_file_size_bytes', source_file_size_bytes,
            'access_rule_set_id', access_rule_set_id,
            'access_rule_set_version', access_rule_set_version,
            'status', status,
            'total_rows', total_rows,
            'imported_rows', imported_rows,
            'created_at', created_at,
            'started_at', started_at,
            'finished_at', finished_at,
            'message', message
        ) AS CHAR) AS content
    FROM meta_import_batch
    WHERE import_batch_id = 'BATCH_7ae0c7d1c0a240ba833e366bf755397d'

    UNION ALL

    SELECT
        10 AS sort_order,
        CONCAT('01_物理表/', base_table_name) AS description,
        CAST(JSON_OBJECT(
            'logical_table_name', logical_table_name,
            'base_table_name', base_table_name,
            'physical_table_name', physical_table_name,
            'layer', layer,
            'data_type', data_type,
            'cached_row_count', row_count,
            'status', status,
            'updated_at', updated_at
        ) AS CHAR) AS content
    FROM meta_batch_table_registry
    WHERE import_batch_id = 'BATCH_7ae0c7d1c0a240ba833e366bf755397d'
      AND base_table_name IN (
          'dws_user_daily_profile',
          'ads_migration_lead_user',
          'ads_dashboard_kpi_summary',
          'ads_app_experience_rank',
          'ads_hourly_experience_trend',
          'ads_network_hotspot_rank',
          'ads_user_experience_profile',
          'ads_lead_evidence_detail'
      )

    UNION ALL

    SELECT
        20 AS sort_order,
        '02_当前看板KPI' AS description,
        CAST(JSON_OBJECT(
            'total_users', COUNT(DISTINCT u.user_key),
            'cable_users', COUNT(DISTINCT CASE WHEN u.user_type = 'CABLE' THEN u.user_key END),
            'ftth_users', COUNT(DISTINCT CASE WHEN u.user_type = 'FTTH' THEN u.user_key END),
            'unknown_users', COUNT(DISTINCT CASE WHEN COALESCE(u.user_type, 'UNKNOWN') NOT IN ('CABLE', 'FTTH') THEN u.user_key END),
            'total_traffic_gb', ROUND(COALESCE(SUM(u.total_download_gb), 0), 6),
            'total_game_hours', ROUND(COALESCE(SUM(u.total_game_hours), 0), 6),
            'sa_lead_users', (
                SELECT COUNT(DISTINCT l.user_key)
                FROM ads_migration_lead_user__33e366bf755397d l
                WHERE l.analysis_run_id = 'RUN_MANUAL_001'
            ),
            'a1_users', (
                SELECT COUNT(DISTINCT l.user_key)
                FROM ads_migration_lead_user__33e366bf755397d l
                WHERE l.analysis_run_id = 'RUN_MANUAL_001'
                  AND l.lead_type LIKE 'A1_%'
            ),
            'a2_users', (
                SELECT COUNT(DISTINCT l.user_key)
                FROM ads_migration_lead_user__33e366bf755397d l
                WHERE l.analysis_run_id = 'RUN_MANUAL_001'
                  AND l.lead_type LIKE 'A2_%'
            )
        ) AS CHAR) AS content
    FROM dws_user_daily_profile__33e366bf755397d u
    WHERE u.import_batch_id = 'BATCH_7ae0c7d1c0a240ba833e366bf755397d'

    UNION ALL

    SELECT
        30 AS sort_order,
        CONCAT('03_结构化KPI/', kpi_group, '/', kpi_key) AS description,
        CAST(JSON_OBJECT(
            'analysis_run_id', analysis_run_id,
            'import_batch_id', import_batch_id,
            'kpi_group', kpi_group,
            'kpi_key', kpi_key,
            'kpi_label', kpi_label,
            'kpi_value', kpi_value,
            'kpi_unit', kpi_unit,
            'kpi_hint', kpi_hint,
            'display_order', display_order,
            'created_at', created_at
        ) AS CHAR) AS content
    FROM ads_dashboard_kpi_summary__33e366bf755397d
    WHERE analysis_run_id = 'RUN_MANUAL_001'

    UNION ALL

    SELECT
        40 AS sort_order,
        CONCAT('04_App体验/', COALESCE(app_name, 'UNKNOWN'), '/', COALESCE(user_type, 'UNKNOWN')) AS description,
        CAST(JSON_OBJECT(
            'analysis_run_id', analysis_run_id,
            'import_batch_id', import_batch_id,
            'app_category', app_category,
            'app_name', app_name,
            'user_type', user_type,
            'active_users', active_users,
            'traffic_gb', traffic_gb,
            'duration_hours', duration_hours,
            'avg_effective_mbps', avg_effective_mbps,
            'avg_vmos', avg_vmos,
            'avg_mos', avg_mos,
            'avg_subscriber_rtt_ms', avg_subscriber_rtt_ms,
            'avg_network_rtt_ms', avg_network_rtt_ms,
            'avg_user_loss_pct', avg_user_loss_pct,
            'avg_network_loss_pct', avg_network_loss_pct,
            'poor_experience_users', poor_experience_users,
            'poor_experience_user_pct', poor_experience_user_pct,
            'main_issue_driver', main_issue_driver,
            'evidence_summary', evidence_summary,
            'created_at', created_at
        ) AS CHAR) AS content
    FROM ads_app_experience_rank__33e366bf755397d
    WHERE analysis_run_id = 'RUN_MANUAL_001'

    UNION ALL

    SELECT
        50 AS sort_order,
        CONCAT(
            '05_小时趋势/', stat_date, '/', LPAD(hour_of_day, 2, '0'), ':00/',
            COALESCE(user_type, 'UNKNOWN'), '/', COALESCE(app_category, 'ALL')
        ) AS description,
        CAST(JSON_OBJECT(
            'analysis_run_id', analysis_run_id,
            'import_batch_id', import_batch_id,
            'stat_date', stat_date,
            'hour_of_day', hour_of_day,
            'user_type', user_type,
            'app_category', app_category,
            'active_users', active_users,
            'traffic_gb', traffic_gb,
            'duration_hours', duration_hours,
            'avg_effective_mbps', avg_effective_mbps,
            'avg_vmos', avg_vmos,
            'avg_mos', avg_mos,
            'avg_subscriber_rtt_ms', avg_subscriber_rtt_ms,
            'avg_network_rtt_ms', avg_network_rtt_ms,
            'avg_user_loss_pct', avg_user_loss_pct,
            'avg_network_loss_pct', avg_network_loss_pct,
            'poor_experience_users', poor_experience_users,
            'created_at', created_at
        ) AS CHAR) AS content
    FROM ads_hourly_experience_trend__33e366bf755397d
    WHERE analysis_run_id = 'RUN_MANUAL_001'

    UNION ALL

    SELECT
        60 AS sort_order,
        CONCAT(
            '06_网络热点/', COALESCE(bras, 'UNKNOWN'), '/', COALESCE(olt, 'UNKNOWN'), '/',
            COALESCE(pon, 'UNKNOWN'), '/', COALESCE(user_type, 'UNKNOWN')
        ) AS description,
        CAST(JSON_OBJECT(
            'analysis_run_id', analysis_run_id,
            'import_batch_id', import_batch_id,
            'bras', bras,
            'olt', olt,
            'pon', pon,
            'user_type', user_type,
            'affected_users', affected_users,
            'traffic_gb', traffic_gb,
            'avg_subscriber_rtt_ms', avg_subscriber_rtt_ms,
            'avg_network_rtt_ms', avg_network_rtt_ms,
            'avg_user_loss_pct', avg_user_loss_pct,
            'avg_network_loss_pct', avg_network_loss_pct,
            'avg_wifi_delay_ms', avg_wifi_delay_ms,
            'poor_experience_user_pct', poor_experience_user_pct,
            'main_issue_driver', main_issue_driver,
            'suggested_action', suggested_action,
            'evidence_summary', evidence_summary,
            'created_at', created_at
        ) AS CHAR) AS content
    FROM ads_network_hotspot_rank__33e366bf755397d
    WHERE analysis_run_id = 'RUN_MANUAL_001'

    UNION ALL

    SELECT
        70 AS sort_order,
        '07_用户画像' AS description,
        CAST(JSON_OBJECT(
            'analysis_run_id', analysis_run_id,
            'import_batch_id', import_batch_id,
            'user_key_hash', SHA2(CONCAT('CHANGE_THIS_PRIVATE_SALT', '|', COALESCE(user_key, '')), 256),
            'user_type', user_type,
            'active_days', active_days,
            'active_hours', active_hours,
            'total_traffic_gb', total_traffic_gb,
            'video_traffic_gb', video_traffic_gb,
            'game_hours', game_hours,
            'top_app_category', top_app_category,
            'avg_effective_mbps', avg_effective_mbps,
            'avg_vmos', avg_vmos,
            'avg_mos', avg_mos,
            'avg_subscriber_rtt_ms', avg_subscriber_rtt_ms,
            'avg_network_rtt_ms', avg_network_rtt_ms,
            'avg_user_loss_pct', avg_user_loss_pct,
            'avg_network_loss_pct', avg_network_loss_pct,
            'bottleneck_side', bottleneck_side,
            'issue_driver', issue_driver,
            'lead_type', lead_type,
            'demand_score', demand_score,
            'migration_motive_score', migration_motive_score,
            'recommended_offer', recommended_offer,
            'evidence_summary', evidence_summary,
            'created_at', created_at
        ) AS CHAR) AS content
    FROM ads_user_experience_profile__33e366bf755397d
    WHERE analysis_run_id = 'RUN_MANUAL_001'

    UNION ALL

    SELECT
        80 AS sort_order,
        '08_迁转机会证据' AS description,
        CAST(JSON_OBJECT(
            'analysis_run_id', analysis_run_id,
            'import_batch_id', import_batch_id,
            'user_key_hash', SHA2(CONCAT('CHANGE_THIS_PRIVATE_SALT', '|', COALESCE(user_key, '')), 256),
            'user_type', user_type,
            'lead_type', lead_type,
            'demand_score', demand_score,
            'migration_motive_score', migration_motive_score,
            'recommended_offer', recommended_offer,
            'top_app_category', top_app_category,
            'total_traffic_gb', total_traffic_gb,
            'video_traffic_gb', video_traffic_gb,
            'game_hours', game_hours,
            'avg_effective_mbps', avg_effective_mbps,
            'avg_vmos', avg_vmos,
            'avg_mos', avg_mos,
            'bottleneck_side', bottleneck_side,
            'issue_driver', issue_driver,
            'final_action', final_action,
            'ftth_available_flag', ftth_available_flag,
            'reachable_flag', reachable_flag,
            'evidence_summary', evidence_summary,
            'created_at', created_at
        ) AS CHAR) AS content
    FROM ads_lead_evidence_detail__33e366bf755397d
    WHERE analysis_run_id = 'RUN_MANUAL_001'

    UNION ALL

    SELECT
        90 AS sort_order,
        CONCAT('09_Lead分层/', lead_summary.user_type, '/', lead_summary.lead_type) AS description,
        CAST(JSON_OBJECT(
            'user_type', lead_summary.user_type,
            'lead_type', lead_summary.lead_type,
            'users', lead_summary.users,
            'avg_demand_score', lead_summary.avg_demand_score,
            'avg_migration_motive_score', lead_summary.avg_migration_motive_score,
            'total_traffic_gb', lead_summary.total_traffic_gb,
            'avg_effective_mbps', lead_summary.avg_effective_mbps,
            'avg_vmos', lead_summary.avg_vmos,
            'avg_mos', lead_summary.avg_mos
        ) AS CHAR) AS content
    FROM (
        SELECT
            COALESCE(user_type, 'UNKNOWN') AS user_type,
            COALESCE(lead_type, 'UNKNOWN') AS lead_type,
            COUNT(DISTINCT user_key) AS users,
            ROUND(AVG(demand_score), 4) AS avg_demand_score,
            ROUND(AVG(migration_motive_score), 4) AS avg_migration_motive_score,
            ROUND(SUM(total_traffic_gb), 4) AS total_traffic_gb,
            ROUND(AVG(avg_effective_mbps), 4) AS avg_effective_mbps,
            ROUND(AVG(avg_vmos), 4) AS avg_vmos,
            ROUND(AVG(avg_mos), 4) AS avg_mos
        FROM ads_lead_evidence_detail__33e366bf755397d lead_source
        WHERE lead_source.analysis_run_id = 'RUN_MANUAL_001'
        GROUP BY lead_source.user_type, lead_source.lead_type
    ) lead_summary

    UNION ALL

    SELECT
        100 AS sort_order,
        CONCAT('10_质量检查/', check_section, '/', check_item) AS description,
        CAST(JSON_OBJECT(
            'check_section', check_section,
            'check_item', check_item,
            'metric_name', metric_name,
            'metric_value', metric_value,
            'metric_text', metric_text,
            'severity', severity,
            'passed', passed,
            'created_at', created_at
        ) AS CHAR) AS content
    FROM meta_quality_check_result
    WHERE import_batch_id = 'BATCH_7ae0c7d1c0a240ba833e366bf755397d'

    UNION ALL

    SELECT
        110 AS sort_order,
        CONCAT('11_IP规则/', COALESCE(r.rule_name, 'NO_RULE')) AS description,
        CAST(JSON_OBJECT(
            'import_batch_id', b.import_batch_id,
            'rule_set_id', b.access_rule_set_id,
            'rule_set_version', b.access_rule_set_version,
            'rule_set_name', s.rule_set_name,
            'default_access_type', s.default_access_type,
            'rule_set_status', s.status,
            'published_at', s.published_at,
            'rule_id', r.rule_id,
            'rule_name', r.rule_name,
            'cidr', r.cidr,
            'start_ip', r.start_ip,
            'end_ip', r.end_ip,
            'access_type', r.access_type,
            'priority', r.priority,
            'enabled', r.enabled,
            'notes', r.notes
        ) AS CHAR) AS content
    FROM meta_import_batch b
    LEFT JOIN meta_access_rule_set s ON s.rule_set_id = b.access_rule_set_id
    LEFT JOIN dim_access_ip_range r ON r.rule_set_id = b.access_rule_set_id
    WHERE b.import_batch_id = 'BATCH_7ae0c7d1c0a240ba833e366bf755397d'

    UNION ALL

    SELECT
        120 AS sort_order,
        CONCAT('12_接入类型/', access_summary.user_type) AS description,
        CAST(JSON_OBJECT(
            'user_type', access_summary.user_type,
            'unique_users', access_summary.unique_users,
            'user_day_rows', access_summary.user_day_rows,
            'active_dates', access_summary.active_dates,
            'total_download_gb', access_summary.total_download_gb,
            'total_game_hours', access_summary.total_game_hours,
            'avg_subscriber_rtt_ms', access_summary.avg_subscriber_rtt_ms,
            'avg_network_rtt_ms', access_summary.avg_network_rtt_ms,
            'avg_user_down_loss', access_summary.avg_user_down_loss,
            'avg_network_down_loss', access_summary.avg_network_down_loss
        ) AS CHAR) AS content
    FROM (
        SELECT
            COALESCE(access_source.user_type, 'UNKNOWN') AS user_type,
            COUNT(DISTINCT access_source.user_key) AS unique_users,
            COUNT(*) AS user_day_rows,
            COUNT(DISTINCT access_source.stat_date) AS active_dates,
            ROUND(SUM(access_source.total_download_gb), 6) AS total_download_gb,
            ROUND(SUM(access_source.total_game_hours), 6) AS total_game_hours,
            ROUND(AVG(access_source.avg_subscriber_rtt_ms), 6) AS avg_subscriber_rtt_ms,
            ROUND(AVG(access_source.avg_network_rtt_ms), 6) AS avg_network_rtt_ms,
            ROUND(AVG(access_source.avg_user_down_loss), 6) AS avg_user_down_loss,
            ROUND(AVG(access_source.avg_network_down_loss), 6) AS avg_network_down_loss
        FROM dws_user_daily_profile__33e366bf755397d access_source
        WHERE access_source.import_batch_id = 'BATCH_7ae0c7d1c0a240ba833e366bf755397d'
        GROUP BY access_source.user_type
    ) access_summary

    UNION ALL

    SELECT
        130 AS sort_order,
        CONCAT('13_App映射/', COALESCE(raw_app_name, 'UNKNOWN')) AS description,
        CAST(JSON_OBJECT(
            'raw_app_name', raw_app_name,
            'standard_app_name', standard_app_name,
            'app_category', app_category,
            'invalid_app_flag', invalid_app_flag,
            'updated_at', updated_at
        ) AS CHAR) AS content
    FROM dim_app_mapping

    UNION ALL

    SELECT
        140 AS sort_order,
        CONCAT('14_阈值/', config_key) AS description,
        CAST(JSON_OBJECT(
            'config_key', config_key,
            'config_value', config_value,
            'value_type', value_type,
            'description', description,
            'updated_at', updated_at
        ) AS CHAR) AS content
    FROM dim_threshold_config
    WHERE config_key LIKE 'lead.%'
       OR config_key LIKE 'quality.%'
       OR config_key LIKE 'experience.%'
) export_data
ORDER BY export_data.sort_order, export_data.description;

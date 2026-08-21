-- DWS → ADS migration lead user baseline
-- This baseline avoids SET @var. Runtime parameters are represented by CTE params.

DELETE FROM :ads_migration_lead_user WHERE analysis_run_id = :analysis_run_id;

INSERT INTO :ads_migration_lead_user (
  analysis_run_id,
  import_batch_id,
  user_key,
  user_type,
  lead_type,
  demand_score,
  migration_motive_score,
  recommended_offer
)
WITH params AS (
  SELECT
    :analysis_run_id AS analysis_run_id,
    :import_batch_id AS import_batch_id,
    CAST(COALESCE((SELECT config_value FROM dim_threshold_config WHERE config_key='lead.demand_threshold'), '60') AS UNSIGNED) AS demand_threshold,
    CAST(COALESCE((SELECT config_value FROM dim_threshold_config WHERE config_key='lead.motive_threshold'), '40') AS UNSIGNED) AS motive_threshold
), profile AS (
  SELECT
    import_batch_id,
    user_key,
    CASE
      WHEN SUM(CASE WHEN user_type = 'CABLE' THEN 1 ELSE 0 END) > 0 THEN 'CABLE'
      WHEN SUM(CASE WHEN user_type = 'FTTH' THEN 1 ELSE 0 END) > 0 THEN 'FTTH'
      ELSE COALESCE(MAX(user_type), 'UNKNOWN')
    END AS user_type,
    SUM(COALESCE(video_rows, 0)) AS video_rows,
    SUM(COALESCE(game_rows, 0)) AS game_rows,
    SUM(COALESCE(total_download_gb, 0)) AS total_download_gb,
    SUM(COALESCE(total_game_hours, 0)) AS total_game_hours,
    AVG(avg_vmos) AS avg_vmos,
    AVG(avg_mos) AS avg_mos,
    AVG(avg_subscriber_rtt_ms) AS avg_subscriber_rtt_ms,
    AVG(avg_network_rtt_ms) AS avg_network_rtt_ms,
    AVG(avg_user_down_loss) AS avg_user_down_loss,
    AVG(avg_network_down_loss) AS avg_network_down_loss,
    MAX(COALESCE(peak_row_pct, 0)) AS peak_row_pct
  FROM :dws_user_daily_profile
  WHERE import_batch_id = (SELECT import_batch_id FROM params)
    AND user_key IS NOT NULL
    AND TRIM(user_key) <> ''
    AND user_key <> 'UNKNOWN'
  GROUP BY import_batch_id, user_key
), confidence AS (
  SELECT user_key, MAX(confidence_score) AS confidence_score
  FROM (
    SELECT user_key, CASE key_confidence WHEN 'HIGH_ACCOUNT_KEY' THEN 3 WHEN 'MEDIUM_MAC_USER_KEY' THEN 2 WHEN 'LOW_IP_ONLY_KEY' THEN 1 ELSE 0 END AS confidence_score
    FROM :dwd_tcp_detail_clean WHERE import_batch_id = (SELECT import_batch_id FROM params)
    UNION ALL
    SELECT user_key, CASE key_confidence WHEN 'HIGH_ACCOUNT_KEY' THEN 3 WHEN 'MEDIUM_MAC_USER_KEY' THEN 2 WHEN 'LOW_IP_ONLY_KEY' THEN 1 ELSE 0 END
    FROM :dwd_game_detail_clean WHERE import_batch_id = (SELECT import_batch_id FROM params)
  ) c
  GROUP BY user_key
), scored AS (
  SELECT
    p.analysis_run_id,
    d.import_batch_id,
    d.user_key,
    d.user_type,
    COALESCE(c.confidence_score, 0) AS confidence_score,
    COALESCE(b.bottleneck_type, 'UNKNOWN') AS bottleneck_type,
    LEAST(100,
      CASE WHEN d.total_download_gb >= 20 THEN 35 WHEN d.total_download_gb >= 8 THEN 25 WHEN d.total_download_gb >= 2 THEN 15 ELSE 0 END +
      CASE WHEN d.total_game_hours >= 8 THEN 25 WHEN d.total_game_hours >= 3 THEN 15 WHEN d.total_game_hours > 0 THEN 8 ELSE 0 END +
      CASE WHEN d.video_rows >= 500 THEN 20 WHEN d.video_rows >= 200 THEN 12 WHEN d.video_rows > 0 THEN 6 ELSE 0 END +
      CASE WHEN d.peak_row_pct >= 35 THEN 20 WHEN d.peak_row_pct >= 20 THEN 10 ELSE 0 END
    ) AS demand_score,
    CASE
      WHEN d.user_type = 'CABLE' THEN LEAST(100,
        30 +
        CASE WHEN d.avg_subscriber_rtt_ms >= 50 THEN 20 ELSE 0 END +
        CASE WHEN d.avg_user_down_loss >= 1 THEN 20 ELSE 0 END +
        CASE WHEN d.peak_row_pct >= 30 THEN 20 ELSE 0 END +
        CASE WHEN d.total_download_gb >= 8 OR d.total_game_hours >= 3 THEN 10 ELSE 0 END
      )
      ELSE 0
    END AS migration_motive_score
  FROM profile d
  JOIN params p ON p.import_batch_id = d.import_batch_id
  LEFT JOIN confidence c ON c.user_key = d.user_key
  LEFT JOIN :dws_user_experience_bottleneck b ON b.import_batch_id = d.import_batch_id AND b.user_key = d.user_key
), typed AS (
  SELECT
    s.*,
    CASE
      WHEN s.confidence_score < 2 THEN 'A0_身份可信度不足_不得营销'
      WHEN s.bottleneck_type = 'NETWORK_SIDE_SEVERE' THEN 'A2_网络严重异常_先修障'
      WHEN s.user_type = 'CABLE' AND s.demand_score >= (SELECT demand_threshold FROM params) AND s.migration_motive_score >= (SELECT motive_threshold FROM params) THEN 'A1_Cable高需求且有迁转动力_候选'
      WHEN s.user_type = 'CABLE' AND s.demand_score >= (SELECT demand_threshold FROM params) THEN 'B_Cable高需求但迁转动力不足_培育池'
      WHEN s.user_type = 'FTTH' AND s.demand_score >= (SELECT demand_threshold FROM params) THEN 'C_FTTH存量高速升套用户'
      ELSE 'D_普通观察用户'
    END AS lead_type
  FROM scored s
)
SELECT
  analysis_run_id,
  import_batch_id,
  user_key,
  user_type,
  lead_type,
  demand_score,
  migration_motive_score,
  CASE
    WHEN lead_type LIKE 'A0_%' THEN '补齐用户身份与触达资格，禁止进入营销名单'
    WHEN lead_type LIKE 'A2_%' THEN '转网络运维修障，修复并复测后再评估机会'
    WHEN lead_type LIKE 'A1_%' THEN '进入CRM/FTTH覆盖/可触达资格校验，暂不直接营销'
    WHEN lead_type LIKE 'B_%' THEN '进入培育池，等待CRM/覆盖/可触达验证'
    WHEN lead_type LIKE 'C_%' THEN '推荐FTTH存量500M+/900M高速升套包'
    ELSE '普通观察，不进入本轮营销'
  END AS recommended_offer
FROM typed;

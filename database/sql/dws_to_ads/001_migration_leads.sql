-- DWS → ADS migration lead user baseline
-- This baseline avoids SET @var. Runtime parameters are represented by CTE params.

INSERT INTO ads_migration_lead_user (
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
    60 AS demand_threshold,
    40 AS motive_threshold
), scored AS (
  SELECT
    p.analysis_run_id,
    d.import_batch_id,
    d.user_key,
    d.user_type,
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
  FROM dws_user_daily_profile d
  JOIN params p ON p.import_batch_id = d.import_batch_id
), typed AS (
  SELECT
    s.*,
    CASE
      WHEN s.user_type = 'CABLE' AND s.demand_score >= 60 AND s.migration_motive_score >= 40 THEN 'A1_Cable高需求且有迁转动力_可优先营销'
      WHEN s.user_type = 'CABLE' AND s.demand_score >= 60 THEN 'B_Cable高需求但迁转动力不足_培育池'
      WHEN s.user_type = 'FTTH' AND s.demand_score >= 60 THEN 'C_FTTH存量高速升套用户'
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
    WHEN lead_type LIKE 'A1_%' THEN '推荐Fiber 500M+/600M/900M + 对称速率 + Wi-Fi 6/Mesh + 视频/OTT/世界杯权益'
    WHEN lead_type LIKE 'B_%' THEN '进入培育池，等待CRM/覆盖/可触达验证'
    WHEN lead_type LIKE 'C_%' THEN '推荐FTTH存量500M+/900M高速升套包'
    ELSE '普通观察，不进入本轮营销'
  END AS recommended_offer
FROM typed;

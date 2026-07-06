# SA FBB Experience Workbench｜产品功能树 v0.2

## 1. 产品定位

本产品的主入口是数据分析看板，不是 ETL 工具。ETL、质量检查、日志和数据库连接是支撑能力，应降级到数据导入或系统管理中。

## 2. 一级功能树

```text
SA FBB Experience Workbench
├─ 1. 数据分析
│  ├─ 批次选择
│  ├─ 总览看板
│  ├─ 应用使用分析
│  ├─ 视频体验分析
│  ├─ 游戏体验分析
│  ├─ 网络体验质量
│  ├─ Cable vs FTTH 对比
│  ├─ 迁转升套机会
│  └─ 用户明细画像
│
├─ 2. 数据导入
│  ├─ 导入新数据
│  ├─ 批次命名
│  ├─ 字段识别与映射检查
│  ├─ 数据可用性检查
│  └─ 数据准备状态
│
└─ 3. 系统管理
   ├─ 数据库连接
   ├─ 诊断日志
   ├─ 高级任务记录
   ├─ 参数配置
   └─ 版本与诊断
```

独立“结果导出”模块取消。导出按钮必须放在对应看板内部。

## 3. 批次边界规则

每次分析必须先选择导入批次。导入批次是分析空间，不只是筛选条件。

第一阶段落地策略：

1. 前端进入数据分析前必须有 `import_batch_id`。
2. 导入时必须设置可读 `batch_display_name`。
3. `meta_import_batch` 保存 `batch_display_name`。
4. 当前代码仍以共享 RAW / CLEAN / DWS / ADS 表加 `import_batch_id` 隔离为主。
5. 每批次独立物理表命名与 SQL 编排作为后续数据库主链路改造，不在本轮一次性切换，避免破坏现有导入和 ETL。

目标态表命名建议：

```text
raw_tcp_detail_import__{batch_short_id}
dwd_tcp_detail_clean__{batch_short_id}
dws_app_category_hourly__{batch_short_id}
ads_video_experience_summary__{batch_short_id}
ads_final_marketing_lead__{batch_short_id}
```

## 4. 模块可用性规则

每个分析模块必须声明：

1. 必填字段。
2. 需要的数据类型。
3. 需要的聚合表。
4. 不可用时的置灰原因。

当前前端已落地基础置灰逻辑：

| 模块 | 关键依赖 |
|---|---|
| 总览看板 | import_batch_id、analysis_run_id、用户、流量、时长 |
| 应用使用分析 | app_name、app_category、duration、traffic |
| 视频体验分析 | universal_video_applications、vmos、downloaded_data_volume_kb |
| 游戏体验分析 | application_protocol、mos、game_duration_s、jitter |
| 网络体验质量 | RTT、PLR、MOS / VMOS、Wi-Fi delay |
| Cable vs FTTH 对比 | user_type / wan_type、hour_of_day、RTT、loss、speed |
| 迁转升套机会 | lead_type、demand_score、migration_motive_score、final_action |
| 用户明细画像 | user_key、app_preference、quality metrics、lead_type |

## 5. 诊断日志规则

系统日志必须用于定位问题，而不是只显示成败。

导入映射失败时，日志至少应包含：

1. 缺失 required 字段数量。
2. 缺失 target column。
3. 当前匹配到的 source header 或未匹配说明。
4. required 标记。
5. 建议用户检查字段映射目录和 CSV header alias。

本轮已把 `import_current_file` 的字段映射错误从单纯数量改为明细列表。

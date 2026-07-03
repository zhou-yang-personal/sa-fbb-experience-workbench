# SA FBB Experience Workbench｜当前核心架构设计

## 1. 设计结论

本项目定位为 **SA 家宽应用体验数据本地分析工作台**，不是普通 BI。第一阶段目标是把 SA TCP / Game CSV 大文件在本地稳定入库、库内清洗、聚合分析，并输出 Cable-to-Fiber 迁转升套机会名单和统计看板。

核心链路必须采用：

```text
CSV 文件选择
→ 文件元信息登记
→ MySQL RAW 表原样高速导入
→ RAW 入库完整性校验
→ MySQL 库内清洗 / 标准化 / 衍生字段
→ CLEAN / DWD 明细层
→ DWS / ADS 聚合与看板结果层
→ 前端看板查询 DWS / ADS
→ 用户名单 / 汇总结果导出
```

约束：

1. CSV 大文件可能达到几千万行，不允许应用内全量读入内存做清洗。
2. CSV 只允许读取少量样本用于预览、字段识别和字段映射。
3. 数据清洗、字段标准化、应用分类、用户画像、体验评分和迁转分层优先在 MySQL 内完成。
4. 看板不得直接扫 RAW 明细大表，必须查询 DWS / ADS 聚合结果。
5. 当前 SA 数据只能形成“应用体验驱动的迁转升套机会”，正式营销名单需要后续 JOIN CRM、套餐、FTTH 覆盖、合约、欠费、黑名单和可触达状态。

## 2. 参考基线与差异

整体工程框架参考 `latam-fbb-desktop` 的桌面端本地分析架构：

```text
React / Vite UI
→ Tauri invoke
→ Rust ETL / Query Backend
→ MySQL Raw / Clean / Aggregate
→ Dashboard 查询与展示
```

可复用的设计原则：

- Raw First：源文件先进入专属 Raw 宽表。
- Clean / Aggregate 分层：清洗、聚合、运行查询分离。
- Dashboard 不直接操作大表源数据。
- 大表处理必须可观测、可续跑、可解释。
- loading table + quality gate + atomic swap 优先。

本项目与 `latam-fbb-desktop` 的主要差异：

| 维度 | latam-fbb-desktop | SA FBB Experience Workbench |
|---|---|---|
| 数据来源 | 政府公开 FBB 市场数据 | SA 单板 TCP / Game 应用体验数据 |
| 核心目标 | 市场快照、竞争分析、国家维度分析 | Cable-to-Fiber 迁转升套、体验质量、用户机会识别 |
| 主数据库 | MySQL + DuckDB Runtime | 第一阶段以本地 MySQL 为主 |
| 数据粒度 | 国家 / 城市 / 运营商 / 技术 / 速率档 | 用户 / 应用 / 小时 / 接入类型 / 网络侧字段 |
| 看板重点 | 市场份额、速率、城市、竞争 | 应用使用、RTT、PLR、MOS/VMOS、Cable vs FTTH、Lead 分层 |
| 导入主链路 | Raw → Clean → Aggregate → Runtime | CSV → RAW → CLEAN/DWD → DWS/ADS |

第一阶段不强制引入 DuckDB Runtime。只有当后续出现“脱离 MySQL 运行看板”“客户演示便携包”“离线发包分析”等需求时，再评估 MySQL → DuckDB Runtime 发布能力。

## 3. 技术选型

推荐第一阶段技术栈：

```text
Desktop: Tauri 2
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Backend: Rust Tauri commands
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Config: JSON / TOML
Logs: local rolling log + MySQL job tables
Package: Windows EXE / MSI
```

选型理由：

1. **Tauri + Rust** 适合做轻量 Windows EXE，本地文件读取、导入调度、日志和系统配置能力强。
2. **React + TypeScript** 适合快速构建导入工作台、数据任务状态页和多看板交互。
3. **ECharts** 更适合本项目需要的 Treemap、环形图、堆叠柱、小时折线、TopN 横向柱和深色大屏风格。
4. **MySQL 8.0** 作为本地计算与存储主引擎，承接几千万行 RAW、清洗 SQL、聚合表和看板查询。
5. **LOAD DATA LOCAL INFILE** 作为大 CSV 主导入能力，应用层不做大文件内存清洗。

## 4. 前端模块架构

建议目录结构：

```text
src/
├─ app/
│  ├─ App.tsx
│  ├─ routes.tsx
│  └─ appConfig.ts
├─ features/
│  ├─ import-center/
│  ├─ data-quality/
│  ├─ etl-jobs/
│  ├─ dashboard-overview/
│  ├─ app-category-detail/
│  ├─ experience-quality/
│  ├─ cable-fiber-compare/
│  ├─ migration-leads/
│  ├─ user-profile/
│  └─ settings/
├─ shared/
│  ├─ api/
│  ├─ charts/
│  ├─ components/
│  ├─ formatters/
│  ├─ i18n/
│  └─ types/
└─ styles/
```

### 4.1 Import Center

职责：

- 选择 CSV 文件。
- 读取前 100 行预览。
- 识别编码、分隔符、header、文件大小、文件 hash。
- 选择数据类型：TCP / Game / CRM / Coverage / Reachability。
- 配置字段映射。
- 创建 import batch。
- 触发 RAW 导入任务。
- 展示导入进度、速度、剩余时间、失败原因。

### 4.2 Data Quality

职责：

- RAW 入库行数校验。
- CSV 文件行数与 RAW 行数对比。
- 字段缺失率、空值率、0 值率、UNKNOWN 率。
- user_account / user_mac 可用性检查。
- Cable / FTTH 分布检查。
- 时间范围、小时分布和应用数量检查。
- 拓扑字段 BRAS / OLT / PON / WAN 有效率检查。

### 4.3 ETL Jobs

职责：

- 管理 RAW → CLEAN 任务。
- 管理 CLEAN → DWS 任务。
- 管理 DWS → ADS 任务。
- 支持失败重跑。
- 支持分步骤查看 SQL、耗时、状态、message。
- 支持基于 import_batch_id 重跑。

### 4.4 Dashboard Overview

对应总览页：

- 用户数。
- 总流量。
- 总使用时长。
- Heavy User 数。
- Cable / FTTH 用户占比。
- 应用分类使用时长排行。
- 应用分类总流量排行。
- Heavy Users by Traffic Category。
- Internet Usage Distribution。
- Usage / Total Traffic by Category Treemap。

### 4.5 App Category Detail

按应用分类钻取：

- OTT Video。
- Short Video。
- Live Video。
- Game。
- Video Conference。
- Office。
- Web / Others。

指标：

- 用户数。
- 应用数。
- 使用时长。
- 下载量。
- 有效下载速率。
- 忙时活跃度。
- Top App。
- Top User。
- Cable / FTTH 对比。

### 4.6 Experience Quality

体验质量页：

- VMOS / MOS 分布。
- 用户侧 RTT。
- 网络侧 RTT。
- 用户侧下行丢包。
- 用户侧上行丢包。
- 网络侧下行丢包。
- 网络侧上行丢包。
- Game jitter。
- Wi-Fi delay。
- 用户侧 / 网络侧 / Wi-Fi 侧瓶颈拆分。

### 4.7 Cable Fiber Compare

Cable vs FTTH 对比页：

- 每小时用户数。
- 每小时平均 RTT。
- 每小时平均丢包。
- 每小时平均有效下载速率。
- 每小时 VMOS / MOS。
- 忙时与非忙时对比。
- 每用户粒度明细。
- Cable 相比 FTTH 的体验差异。

### 4.8 Migration Leads

迁转升套机会页：

- Lead Type 漏斗。
- A1 / A0 / A2 / B / C / D 用户数量。
- 需求分、迁转动力分。
- 推荐套餐。
- 用户级明细表。
- 一键导出 CSV / Excel。
- 后续 CRM JOIN 状态。

### 4.9 User Profile

单用户画像：

- user_key / user_account / user_mac。
- key_confidence。
- user_type。
- 应用偏好。
- 忙时活跃。
- 视频、短视频、直播、游戏指标。
- 用户侧 RTT / 网络侧 RTT / 丢包 / Wi-Fi delay。
- Lead Type。
- 推荐动作。

## 5. Rust / Tauri 后端模块架构

建议目录结构：

```text
src-tauri/src/
├─ main.rs
├─ commands/
│  ├─ db_commands.rs
│  ├─ file_commands.rs
│  ├─ import_commands.rs
│  ├─ etl_commands.rs
│  ├─ dashboard_commands.rs
│  ├─ export_commands.rs
│  └─ settings_commands.rs
├─ db/
│  ├─ mysql_pool.rs
│  ├─ migrations.rs
│  ├─ query_runner.rs
│  └─ transaction.rs
├─ import/
│  ├─ csv_probe.rs
│  ├─ csv_manifest.rs
│  ├─ load_data.rs
│  ├─ streaming_insert.rs
│  └─ import_batch.rs
├─ etl/
│  ├─ raw_to_clean.rs
│  ├─ clean_to_dws.rs
│  ├─ dws_to_ads.rs
│  ├─ job_runner.rs
│  └─ quality_gate.rs
├─ export/
│  ├─ csv_export.rs
│  └─ excel_export.rs
├─ settings/
│  ├─ app_settings.rs
│  └─ mysql_settings.rs
└─ logging/
   └─ app_logger.rs
```

### 5.1 后端职责边界

Rust 后端负责：

- 本地文件选择与文件信息读取。
- CSV 样本读取，不做全量清洗。
- MySQL 连接管理。
- LOAD DATA LOCAL INFILE 调度。
- 流式分块 INSERT fallback。
- ETL SQL 编排。
- 任务状态记录。
- 失败重跑。
- 导出文件。
- 本地配置与日志。

Rust 后端不负责：

- 几千万行 CSV 的业务清洗。
- 大表全量内存聚合。
- 看板口径硬编码在前端。
- 用应用内循环替代 MySQL 聚合 SQL。

## 6. MySQL 数据分层设计

### 6.1 元数据层

```text
meta_import_batch
meta_import_file
meta_import_field_mapping
meta_etl_job
meta_etl_job_step
meta_quality_check_result
meta_app_config
```

职责：

- 管理导入批次。
- 记录文件元信息。
- 记录字段映射模板。
- 记录 ETL 任务和步骤。
- 记录质量检查结果。
- 记录全局配置和阈值版本。

### 6.2 DIM 配置层

```text
dim_app_mapping
dim_app_category
dim_threshold_config
dim_offer_rule
dim_access_type_mapping
dim_key_confidence_rule
dim_lead_type_rule
```

职责：

- 应用名称标准化。
- 应用分类映射。
- Cable / FTTH 标准化。
- 阈值配置。
- 推荐套餐规则。
- Lead Type 规则。

### 6.3 RAW 原始入库层

```text
raw_tcp_detail_import
raw_game_detail_import
raw_crm_user_import
raw_ftth_coverage_import
raw_reachability_import
```

设计原则：

- 原样承接 CSV。
- 每条记录带 `import_batch_id`。
- 每条记录带 `source_file_name`。
- 必要时带 `source_line_no`。
- 字段优先使用 VARCHAR / TEXT，日期和数字先不强制转换。
- RAW 表不建立过多索引，避免拖慢导入。
- RAW 表不做复杂生成列。

### 6.4 CLEAN / DWD 标准明细层

```text
dwd_tcp_detail_clean
dwd_game_detail_clean
dwd_user_identity_clean
dwd_user_access_type_daily
```

处理内容：

- 时间字段标准化。
- 数值字段标准化。
- 速率 Kbps / Mbps 转换。
- 流量 KB / MB / GB 转换。
- 丢包率单位确认与统一。
- RTT / Jitter / Wi-Fi delay 数值化。
- 应用名称标准化。
- 应用分类映射。
- Cable / FTTH 标准化。
- user_key 生成。
- key_confidence 生成。
- worst_latency / worst_loss / worst_jitter 衍生。
- invalid_app_flag / data_quality_flag 生成。

### 6.5 DWS 聚合层

```text
dws_user_hourly_metrics
dws_user_daily_profile
dws_user_app_category_daily
dws_app_category_daily
dws_access_type_hourly_compare
dws_network_cluster_daily
dws_user_experience_bottleneck
```

聚合粒度：

- import_batch_id。
- stat_date。
- hour_of_day。
- user_key。
- user_type。
- app_category。
- app_name。
- bras / olt / pon。

### 6.6 ADS 看板结果层

```text
ads_dashboard_overview
ads_app_category_detail
ads_experience_quality_summary
ads_cable_fiber_compare
ads_migration_lead_funnel
ads_migration_lead_summary
ads_migration_lead_user
ads_build_priority_cluster
```

职责：

- 面向前端查询。
- 查询稳定、轻量、可分页。
- 不做重型业务计算。
- 以 import_batch_id / analysis_run_id 隔离不同批次结果。

## 7. 大 CSV 导入设计

### 7.1 主方案：LOAD DATA LOCAL INFILE

流程：

```text
1. 前端选择文件
2. Rust 读取文件元信息和前 100 行
3. 用户确认数据类型和字段映射
4. 写入 meta_import_batch / meta_import_file
5. 生成 RAW 表导入 SQL
6. 执行 LOAD DATA LOCAL INFILE
7. 记录导入开始、结束、耗时、行数、错误
8. 执行 RAW 层质量检查
```

适用：

- 千万级 CSV。
- 本地 MySQL 允许 `local_infile`。
- 文件在本机可访问。

### 7.2 Fallback：流式分块 INSERT

当客户环境禁用 `LOAD DATA LOCAL INFILE` 时使用：

- Rust 流式读取 CSV。
- 每 5,000 / 10,000 行批量 INSERT。
- 只做最小转义和字段数量对齐。
- 不做业务清洗。
- 支持失败行记录。
- 支持断点续跑。

### 7.3 质量门禁

RAW 导入后必须执行：

- 文件行数 vs RAW 行数。
- 字段数量检查。
- 空行检查。
- user_account / user_mac 空值率。
- user_type 分布。
- 时间范围。
- 应用数量。
- Cable / FTTH 分布。
- UNKNOWN 拓扑字段比例。

质量门禁失败时：

- 阻断 RAW → CLEAN。
- 展示失败项。
- 允许用户选择修正映射后重跑。

## 8. 清洗与聚合任务设计

### 8.1 RAW → CLEAN

使用 SQL 执行清洗，不在应用内循环清洗。

关键动作：

- 字段类型转换。
- 日期解析。
- 数值解析。
- 应用分类 JOIN dim_app_mapping。
- user_key 生成。
- key_confidence 标记。
- 体验字段标准化。
- 衍生指标生成。

### 8.2 CLEAN → DWS

关键聚合：

- 用户小时级使用。
- 用户日级画像。
- 应用分类日级聚合。
- Cable / FTTH 小时对比。
- 用户侧 / 网络侧 / Wi-Fi 侧瓶颈拆分。
- 网络侧聚类。

### 8.3 DWS → ADS

关键结果：

- 看板总览。
- 应用分类详情。
- 体验质量摘要。
- Cable vs FTTH 对比。
- 迁转升套漏斗。
- 用户级 Lead 明细。

### 8.4 SQL 变量策略

MySQL SQL 不依赖如下写法作为主实现：

```sql
SET @run_time = NOW();
SET @min_valid_user_rows = 3;
SET @min_cluster_users = 10;
```

统一优先使用：

- CTE 参数块。
- 临时参数表。
- `dim_threshold_config` 配置表。
- Rust 将参数写入 `meta_etl_job` / `dim_threshold_config` 后再执行 SQL。

示例：

```sql
WITH params AS (
  SELECT
    NOW() AS run_time,
    3 AS min_valid_user_rows,
    10 AS min_cluster_users
)
SELECT *
FROM params;
```

## 9. 用户识别与主键策略

当前 SA 数据中 `user_account` 可能出现：

- IPv4。
- MAC。
- masked MAC / masked account。
- 其他账号格式。

因此设计上不能假设 `user_account` 一定可直接 JOIN CRM。

建议生成统一 `user_key`：

```text
优先级 1：可靠 user_account
优先级 2：可靠 user_mac
优先级 3：user_account 看起来是 MAC
优先级 4：local_ip + 时间窗口，仅用于分析，不作为营销主键
```

同步生成 `key_confidence`：

```text
HIGH_ACCOUNT_KEY
MEDIUM_MAC_USER_KEY
LOW_MASKED_ACCOUNT_KEY
LOW_IP_ONLY_KEY
UNKNOWN_KEY
```

Lead 输出时：

- `HIGH_ACCOUNT_KEY` 可进入正式 CRM JOIN 候选。
- `MEDIUM_MAC_USER_KEY` 需要 CRM / 装机系统补充映射。
- `LOW_IP_ONLY_KEY` 只能做体验分析或区域聚类，不直接营销。

## 10. 应用分类设计

建议一级分类：

```text
long_video_ott
short_video
live_video
game
cloud_gaming
video_conference
office
web
social
other
invalid_app
```

映射逻辑：

- `universal_video_applications` → TCP / Video 应用名。
- `application_protocol` → Game 应用名。
- 通过 `dim_app_mapping` 做标准化。
- 明显识别错误或设备/更新类应用可标记 `invalid_app`，不进入 TOP 应用分析。

## 11. 体验指标设计

### 11.1 TCP / Video 指标

核心字段：

- VMOS。
- connection establishment success rate。
- connection establishment delay。
- upstream / downstream RTT。
- network side RTT。
- subscriber side RTT。
- user avg download rate。
- effective download rate。
- downloaded data volume。
- effective download duration。
- video download duration。
- network / user side packet loss。
- Wi-Fi delay。

### 11.2 Game 指标

核心字段：

- MOS。
- connection establishment success rate。
- upstream / downstream RTT。
- network side RTT。
- subscriber side RTT。
- network / user side packet loss。
- upstream / downstream jitter。
- heartbeat latency。
- game duration。
- single flow rate。
- Wi-Fi delay。
- worst latency / worst loss / worst jitter。

### 11.3 体验瓶颈归因

建议拆分：

```text
NETWORK_SIDE_SEVERE
USER_SIDE_OR_WIFI_PRESSURE
APP_DEMAND_HIGH_BUT_EXPERIENCE_OK
PEAK_HOUR_PRESSURE
DATA_INSUFFICIENT
```

原则：

- 网络侧严重异常用户不直接进入营销升套名单，应进入网络优化或建网评估。
- 用户侧 / Wi-Fi 侧压力用户适合 Wi-Fi 6 ONT / Mesh / Fiber + Wi-Fi 组合包。
- 高应用需求且体验轻度承压的 Cable 用户是 A1 迁转营销重点。

## 12. 迁转升套评分模型

### 12.1 Demand Score

需求分来自：

- OTT 使用强度。
- 短视频使用强度。
- 直播使用强度。
- 游戏使用强度。
- 忙时活跃度。
- 多场景并发。
- 下载量。
- 有效时长。

建议输出 0–100 分。

### 12.2 Migration Motive Score

迁转动力分来自：

- Cable 用户。
- Cable 相比 FTTH RTT 更高。
- Cable 相比 FTTH 丢包更高。
- Cable VMOS / MOS 更低。
- 忙时体验承压。
- 适合 Fiber 产品承接。
- 非网络侧严重异常。

建议输出 0–100 分。

### 12.3 Lead Type

```text
A1_Cable高需求且有迁转动力_可优先营销
A0_高价值但CRM主键待确认
A2_Cable高需求但网络侧异常_先优化或建网
B_Cable高需求但迁转动力不足_培育池
C_FTTH存量高速升套用户
D_普通观察用户
```

### 12.4 推荐套餐

推荐三档：

1. Fiber 300M / 500M：基础迁转包，主打稳定、对称体验。
2. Fiber 500M / 600M + Wi-Fi 6 / OTT：家庭娱乐包，主打高清视频、直播、世界杯、家庭多设备。
3. Fiber 900M / 1G + Mesh + 游戏 / 直播权益：高价值包，主打游戏低时延、多设备并发、家庭娱乐中心。

## 13. 看板查询接口设计

前端只通过 Tauri command 访问后端，不直连 MySQL。

建议 command：

```text
db_test_connection
import_probe_csv
import_create_batch
import_start_raw_load
import_get_progress
quality_get_batch_report
etl_start_clean_job
etl_start_aggregate_job
etl_get_job_detail
dashboard_get_overview
dashboard_get_app_category_detail
dashboard_get_experience_quality
dashboard_get_cable_fiber_compare
leads_get_funnel
leads_get_summary
leads_query_users
export_leads_csv
settings_get
settings_update
```

接口约束：

- 所有列表接口必须支持分页。
- 所有看板接口必须带 `import_batch_id` 或 `analysis_run_id`。
- 大结果导出走后端文件流，不把全量数据传给前端内存。
- 错误信息必须带 job_id / step_id / SQL stage，便于定位。

## 14. 可观测性设计

任务表：

```text
meta_etl_job
meta_etl_job_step
meta_quality_check_result
```

每个任务记录：

- job_id。
- import_batch_id。
- job_type。
- status。
- started_at。
- finished_at。
- duration_ms。
- current_step。
- total_steps。
- affected_rows。
- error_code。
- error_message。

每个步骤记录：

- step_name。
- source_table。
- target_table。
- SQL template name。
- started_at。
- finished_at。
- status。
- affected_rows。
- message。

## 15. 性能策略

### 15.1 导入性能

- RAW 导入优先 LOAD DATA。
- RAW 表导入阶段不建过多索引。
- CLEAN / DWS / ADS 可以在写入完成后建立必要索引。
- 大批次任务分步骤提交，避免长事务压死本地 MySQL。

### 15.2 查询性能

- 前端只查 DWS / ADS。
- 高频筛选字段建联合索引。
- 用户明细必须分页。
- TopN 查询预聚合。
- Cable / FTTH 小时对比预计算。

### 15.3 存储治理

- 支持按 batch 删除 RAW / CLEAN / DWS / ADS。
- 支持归档旧 batch。
- 不提交本地数据库文件和导出文件到 Git。

## 16. 目录结构建议

```text
.
├─ AGENTS.md
├─ AGENTS.common.md
├─ AGENTS.project.md
├─ README.md
├─ package.json
├─ package-lock.json
├─ index.html
├─ vite.config.ts
├─ tsconfig.json
├─ src/
├─ src-tauri/
├─ database/
│  ├─ migrations/
│  ├─ seeds/
│  └─ sql/
│     ├─ raw_to_clean/
│     ├─ clean_to_dws/
│     └─ dws_to_ads/
└─ docs/
   ├─ design/current-core-design.md
   ├─ requirements/current-requirements.md
   ├─ handoff/latest-handoff.md
   ├─ changes/CHANGELOG-dev.md
   └─ development/chatgpt-github-connector-guide.md
```

## 17. MVP 范围

### V0.1：架构骨架与数据导入闭环

- Tauri + React + TypeScript 工程初始化。
- MySQL 连接配置与连接测试。
- CSV 文件探测。
- import batch 管理。
- TCP RAW 导入。
- Game RAW 导入。
- RAW 质量检查。
- RAW → CLEAN 基础清洗。
- CLEAN → DWS 基础聚合。
- 导入与 ETL 日志页面。

### V0.2：核心看板

- 总览页。
- 应用分类详情页。
- 体验质量页。
- Cable vs FTTH 对比页。
- 用户明细查询。

### V0.3：迁转升套机会

- Demand Score。
- Migration Motive Score。
- Lead Type 分层。
- 推荐套餐。
- Lead 漏斗。
- 用户名单导出。

### V0.4：客户营销闭环增强

- CRM 套餐表导入。
- FTTH 覆盖表导入。
- 可触达状态导入。
- 正式营销名单过滤。
- 转化结果回填。
- ROI 复盘。

## 18. 风险边界

1. **主键风险**：user_account 可能不是 CRM 账号，必须保留 key_confidence。
2. **口径风险**：丢包率、成功率、VMOS/MOS、download_fluency 的单位和含义必须通过样本和客户口径校准。
3. **拓扑风险**：BRAS / OLT / PON 可能 UNKNOWN 比例高，区域建网分析不能强行依赖无效字段。
4. **营销风险**：网络侧严重异常用户不应直接营销升套。
5. **性能风险**：前端不得加载全量用户明细；必须分页和后端导出。
6. **交付风险**：第一阶段不提交客户 CSV、数据库、导出名单、日志、安装包。

## 19. 下一步工程落地顺序

1. 初始化 Tauri + React + TypeScript + Vite 工程。
2. 建立 README、requirements、handoff、changelog。
3. 建立 MySQL metadata / dim / raw / dwd / dws / ads migration。
4. 实现 MySQL 连接配置。
5. 实现 CSV probe。
6. 实现 import batch。
7. 实现 TCP / Game RAW 导入。
8. 实现 RAW 质量检查页。
9. 实现 RAW → CLEAN SQL。
10. 实现 CLEAN → DWS / ADS SQL。
11. 实现 Overview / Experience / Cable-Fiber / Leads 四类核心看板。

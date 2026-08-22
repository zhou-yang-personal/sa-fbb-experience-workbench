# SA FBB Experience Workbench｜当前需求基线

## 1. 产品目标

构建一个本地 EXE 应用，用于处理 SA 家宽 TCP / Game 应用体验 CSV 数据，并支撑 Cable-to-Fiber 迁转升套机会识别。

## 2. 必须做

1. CSV 大文件先原样进入 MySQL RAW 表，不在应用内做全量内存清洗。
2. 提供 TCP / Game 两类 CSV 数据导入入口。
3. 提供 import batch、导入任务、质量检查、ETL 任务状态管理。
4. 通过 MySQL SQL 完成 RAW → CLEAN/DWD → DWS/ADS。
5. 提供经营总览、应用体验、网络问题定位、Cable vs FTTH、用户洞察、迁转升套机会六类任务看板。
6. 输出用户级 Lead Type 和推荐套餐字段。
7. 保留后续 CRM、FTTH 覆盖、可触达状态 JOIN 的扩展点。
8. 提供版本化 IPv4 网段配置，用于优先识别 Cable / FTTH；必须支持草稿、重叠校验、抽样预览、发布、批次绑定和规则证据追溯。每次 TCP / Game 导入必须由用户手动选择并确认一个已发布规则版本，不得静默使用最新版本。
9. App 看板必须使用真实 App 粒度，网络热点必须保留 BRAS / OLT / PON，图表必须能回到聚合证据。
10. A0 身份不足和 A2 网络严重异常用户不得表述为可直接营销名单；A1 仍需 CRM、覆盖和可触达资格校验。
11. 历史导入批次必须支持单个或批量删除，并清理该批次的 RAW、DWD、DWS、ADS、分析运行和任务日志；运行中的导入、ETL 或流水线必须禁止删除。
12. 本地 MySQL 连接密码默认值为 `123456`，允许用户覆盖；覆盖值不得写入 localStorage 或执行日志。

## 3. 明确不做

1. 不把体验差用户直接等同于升套潜客。
2. 不在第一阶段提交客户真实 CSV、数据库导出、安装包或运行日志。
3. 不让前端直接查询 RAW 大表。
4. 不在应用内对几千万行 CSV 做全量清洗。
5. 不在第一阶段强制引入 DuckDB Runtime。

## 4. MVP 验收口径

- 工程可按 `npm install`、`npm run dev`、`npm run build`、`npm run tauri:dev` 路径继续验证。
- 数据库存在 metadata / dim / raw / dwd / dws / ads 分层 DDL。
- 前端存在导入、ETL、看板、Lead 分层的入口骨架。
- Tauri command 已提供后续接入 MySQL / CSV / ETL 的接口名称。
- SQL 模板不使用 `SET @var`，优先使用 CTE 参数块。
- 乱序但可映射的 CSV 表头仍应走 `LOAD DATA LOCAL INFILE`；1 GB+ 性能必须以目标硬件真实基准测试为准。
- `ads_network_hotspot_rank` 在 utf8mb4 下必须使用不超过 InnoDB 3072 字节限制的主键和索引设计。

# SA FBB Experience Workbench｜当前需求基线

## 1. 产品目标

构建一个本地 EXE 应用，用于处理 SA 家宽 TCP / Game 应用体验 CSV 数据，并支撑 Cable-to-Fiber 迁转升套机会识别。

## 2. 必须做

1. CSV 大文件先原样进入 MySQL RAW 表，不在应用内做全量内存清洗。
2. 提供 TCP / Game 两类 CSV 数据导入入口。
3. 提供 import batch、导入任务、质量检查、ETL 任务状态管理。
4. 通过 MySQL SQL 完成 RAW → CLEAN/DWD → DWS/ADS。
5. 提供总览、体验质量、Cable vs FTTH、迁转升套机会四类核心页面骨架。
6. 输出用户级 Lead Type 和推荐套餐字段。
7. 保留后续 CRM、FTTH 覆盖、可触达状态 JOIN 的扩展点。

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

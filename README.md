# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.24
```

## 1. 核心目标

1. 支持千万级 CSV 文件不经应用内内存清洗，优先原样导入 MySQL RAW 表。
2. 基于 MySQL 完成 RAW → CLEAN/DWD → DWS/ADS 分层清洗和聚合。
3. 支撑总览、应用分类、体验质量、Cable vs FTTH、迁转升套机会等看板。
4. 输出可复核、可重跑、可导出的用户级机会名单。
5. 通过 CRM、FTTH 覆盖、可触达状态完成最终营销名单融合。

## 2. 架构原则

核心链路：

```text
CSV 文件选择
→ 文件元信息登记
→ MySQL RAW 表原样高速导入
→ RAW 入库完整性校验
→ MySQL 库内清洗 / 标准化 / 衍生字段
→ CLEAN / DWD 明细层
→ DWS / ADS 聚合与看板结果层
→ CRM / 覆盖 / 触达融合
→ 用户名单 / 汇总结果导出
```

详细设计见：`docs/design/current-core-design.md`。

产品功能树见：`docs/design/product-function-tree-v0.2.md`。

## 3. Phase 1-7 完整应用基线

1. Phase 1：本地工程骨架、版本文件、MySQL 连接、数据库初始化。
2. Phase 2：大 CSV 导入入口、批次管理、RAW 入库主路径。
3. Phase 3：RAW 质量门禁、RAW → CLEAN、完整 DWS 聚合。
4. Phase 4：Overview、应用分类、体验质量、Cable vs FTTH 看板 ADS。
5. Phase 5：Lead scoring、瓶颈归因、迁转升套分层。
6. Phase 6：CRM、FTTH 覆盖、可触达状态融合，生成最终营销动作。
7. Phase 7：导出、handoff、changelog、交付检查入口。

## 4. 1.0.24 收口重点

- 修复 Quality Gate / CLEAN 时间清洗中的 NBSP 处理：不再使用 `CHAR(160)`，改为 `CONVERT(0xC2A0 USING utf8mb4)`，避免 MySQL `ERROR 3854 Cannot convert string '\xA0' from binary to utf8mb4`。
- TCP / Game 的 Quality Gate 和 RAW→CLEAN SQL 已同步修复，继续保留 tab / CR / LF / NBSP 转空格、连续空白压缩、`stat_time_text` guarded `STR_TO_DATE` 和单数字日/月支持。
- Pipeline 实时日志和执行计划错误显示增加 CSS 高度兜底：日志区域内部滚动，执行计划和失败卡片不再被完整 SQL statement 撑开。
- Rust SQL 片段测试同步为 `CONVERT(0xC2A0 USING utf8mb4)`，并断言不再包含 `CHAR(160)`。
- `package.json` 已同步到 `1.0.24`；部分 Tauri/Rust 配置文件通过 ChatGPT GitHub connector 写入时被 safety block 拦截，需在本地或后续 Codex 中补齐。

## 5. 1.0.23 收口重点

- 数据导入页主流程从 8 个手工按钮改为“启动导入分析计划”：用户选择 data type、CSV、批次名和导入方式后，后端异步串行执行完整 pipeline。
- 新增持久化 pipeline 元数据表：`meta_pipeline_run`、`meta_pipeline_step`、`meta_pipeline_log`，用于状态、步骤耗时和实时日志轮询。
- 新增后端命令 `import_pipeline_start`、`import_pipeline_get_status`、`import_pipeline_get_logs`；前端每秒刷新当前步骤、总进度、运行时间和日志。
- Final Lead 融合作为可降级步骤：缺 CRM / coverage / reachability 或 final rows=0 时 pipeline 标记 `degraded`，基础 DWS/ADS 和 SA Lead 仍可用。
- Quality Gate 时间解析同步 CLEAN 逻辑：tab、CR、LF、NBSP 先转空格并压缩，支持单数字日/月，不再直接 `STR_TO_DATE` 原始字段。
- Quality Gate 按 batch `data_type` 路由：tcp 只扫 TCP，game 只扫 Game，辅助数据类型记录 skipped/not applicable。
- 版本标记统一同步到 `1.0.23`。

## 6. 1.0.22 收口重点

- TCP / Game CLEAN 时间解析继续收口：支持 `d/M/yyyy HH:mm:ss`、`dd/M/yyyy HH:mm:ss`、`d/MM/yyyy HH:mm:ss`、`dd/MM/yyyy HH:mm:ss` 以及对应无秒格式。
- `statistics_duration` / `statistical_time` 中 tab、CR、LF、NBSP 统一替换为空格并压缩连续空白，避免中间分隔符被删除后拼坏时间文本。
- `STR_TO_DATE` 继续只处理已清洗的 `stat_time_text`，无效时间进入 `WARN_INVALID_STAT_TIME`，不应中断 CLEAN。
- `migration_lead` 模块可用性以基础 `ads_migration_lead_user` 为准；Final Lead 缺失或 rows=0 时显示降级提示，不再让 SA Lead 机会模块整体置灰。
- 模块级迁转导出回到 SA Lead 汇总；Final Lead 查询和专用导出保留在 Lead / Final Lead 能力中。
- 版本标记统一同步到 `1.0.22`。

## 7. 1.0.21 收口重点

- 左侧一级入口保持 3 个：数据导入、数据分析、系统管理。
- 数据导入内部重构为 8 步闭环：导入准备、选择文件与批次、字段映射校验、RAW 入库、RAW 质量检查、CLEAN/DWD、DWS/ADS、模块可用性检查。
- Quality Gate、RAW→CLEAN、DWS/ADS 和 Module Ready 已归入数据导入主流程，不再藏在系统管理。
- 修复 `statistics_duration` / `statistical_time` 中 tab、CR、LF、NBSP 等不可见字符导致 `STR_TO_DATE` 抛 ERROR 1411 的问题。
- CLEAN job 按 batch `data_type` 路由：tcp 只跑 `tcp_raw_to_clean`，game 只跑 `game_raw_to_clean`，辅助数据类型记录 skipped。
- 模块状态文案区分 missing table、rows=0 结果未生成、analysis_run_id 无结果、不适用和缺字段。
- 版本标记统一同步到 `1.0.21`。

## 8. 1.0.20 收口重点

- 1.0.20 不再逐条修 alias，而是收口 Import / Mapping / RAW Load 契约。
- 旧库 mapping catalog 会在导入前自动 self-heal：validation、RAW load、atomic import 和 config seed 都会确保默认 catalog 已刷新。
- 新增 mapping catalog health，可观测 `app_version`、`mapping_seed_version`、`mapping_seed_applied_at`、stale 状态和 critical alias 缺失。
- Import 主按钮改为后端 `import_current_file_atomic`：probe、catalog repair、create batch、mapping validation、RAW load、dataset profile refresh 在后端串联。
- Mapping required gate 下沉到 RAW load 入口，外部直接 invoke `import_start_raw_load` 也不能绕过。
- Mapping fail / RAW fail 会把 batch 标记为 `failed` 并写入 message，不再留下 pending batch。
- 前端 required 缺失错误只展示前 20 个 normalized headers，完整信息保留在 mapping result。
- 版本标记统一同步到 `1.0.20`。

## 9. 1.0.19 修复重点

- 修复 Universal Video detail CSV header normalization：NBSP、tab、多空格、括号、百分号、斜杠等非 ASCII 字母数字分隔符统一归一为 `_`，并压缩重复 `_`。
- `mapping_validation_commands.rs` 和 `raw_import_v2.rs` 共用同一套 header normalize helper，避免映射校验通过但 RAW 入库失败。
- `tcp.user_type` 和 `game.user_type` 不再作为 RAW 导入 required 字段；缺失时允许入库，后续 CLEAN/DWS 使用 UNKNOWN 或通过其他数据补齐。
- 补充 Universal Video detail 常见 alias，包括 `Subscriber Account`、`Downloaded Data Volume (KB)`、`Effective Download Duration (s)` 等。
- 映射错误信息增加 target、required、候选 alias、normalized alias 和当前 CSV normalized headers，便于定位真实 CSV header 差异。
- 版本标记统一同步到 `1.0.19`。

## 10. 1.0.18 收口重点

- 在 1.0.17 批次选择、batch registry、module status 和批次物理表路由基础上，收口为 `1.0.18`。
- 修复 SQL 物理表替换逻辑，避免 base table 子串误替换和二次替换。
- ETL Job Step 的 source / target 表记录改为物理表名，complete aggregates / dashboards 可直接定位批次表。
- Module status 增加 data_type、物理表存在性、row_count、字段级 information_schema 和 analysis_run_id 数据检查。
- Game、Network、User Profile、Video detail 等模块改为查询业务 DWD / DWS / ADS 物理表，不再只返回通用 moduleMetrics。
- `analysis_export_module_csv` 改为导出模块业务结果，不再把 module status / registry 冒充业务导出。
- 新增 MySQL smoke checklist：`docs/validation/mysql-smoke-checklist-1.0.18.md`。

## 11. 1.0.15 修复重点

- 主界面从功能模块入口收口为 5 步向导：Start / Import / Validate / Analyze / Results。
- 增加全局 Pipeline 状态条、下一步提示和当前动作反馈条。
- 增加统一 ActionButton，支持 running / success / failure / disabled 状态反馈。
- Import 主流程改为“选择 CSV 文件 + 导入当前文件”，主流程使用系统文件选择框，不再要求手输路径。
- Export 预设改为系统保存弹框选择输出路径。
- Run Log 从一级入口改为抽屉式查看，主流程只保留结论反馈。
- Analyze 增加一键生成分析结果入口，保留单步 ETL 高级排错。
- 本轮未改 SQL、数据库结构、业务口径或依赖。

## 12. 技术栈

```text
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Desktop: Tauri 2
Backend: Rust
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Package manager: npm
```

## 13. 开发命令

```bash
npm install
npm run dev
npm run build
npm run tauri:dev
npm run tauri:build
cd src-tauri && cargo check
```

## 14. 当前状态

1.0.24 在 1.0.23 pipeline runner 基础上修复 Quality Gate / CLEAN 的 NBSP 字符集处理，并用 CSS 兜底限制长错误和实时日志高度。编译验证和 smoke 结果以本轮交付报告与 changelog 的实际执行记录为准；真实 MySQL / customer CSV smoke 如未执行，不得视为通过。

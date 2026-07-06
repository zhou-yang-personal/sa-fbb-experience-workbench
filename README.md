# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.19
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

## 4. 1.0.19 修复重点

- 修复 Universal Video detail CSV header normalization：NBSP、tab、多空格、括号、百分号、斜杠等非 ASCII 字母数字分隔符统一归一为 `_`，并压缩重复 `_`。
- `mapping_validation_commands.rs` 和 `raw_import_v2.rs` 共用同一套 header normalize helper，避免映射校验通过但 RAW 入库失败。
- `tcp.user_type` 和 `game.user_type` 不再作为 RAW 导入 required 字段；缺失时允许入库，后续 CLEAN/DWS 使用 UNKNOWN 或通过其他数据补齐。
- 补充 Universal Video detail 常见 alias，包括 `Subscriber Account`、`Downloaded Data Volume (KB)`、`Effective Download Duration (s)` 等。
- 映射错误信息增加 target、required、候选 alias、normalized alias 和当前 CSV normalized headers，便于定位真实 CSV header 差异。
- 版本标记统一同步到 `1.0.19`。

## 5. 1.0.18 收口重点

- 在 1.0.17 批次选择、batch registry、module status 和批次物理表路由基础上，收口为 `1.0.18`。
- 修复 SQL 物理表替换逻辑，避免 base table 子串误替换和二次替换。
- ETL Job Step 的 source / target 表记录改为物理表名，complete aggregates / dashboards 可直接定位批次表。
- Module status 增加 data_type、物理表存在性、row_count、字段级 information_schema 和 analysis_run_id 数据检查。
- Game、Network、User Profile、Video detail 等模块改为查询业务 DWD / DWS / ADS 物理表，不再只返回通用 moduleMetrics。
- `analysis_export_module_csv` 改为导出模块业务结果，不再把 module status / registry 冒充业务导出。
- 新增 MySQL smoke checklist：`docs/validation/mysql-smoke-checklist-1.0.18.md`。

## 6. 1.0.15 修复重点

- 主界面从功能模块入口收口为 5 步向导：Start / Import / Validate / Analyze / Results。
- 增加全局 Pipeline 状态条、下一步提示和当前动作反馈条。
- 增加统一 ActionButton，支持 running / success / failure / disabled 状态反馈。
- Import 主流程改为“选择 CSV 文件 + 导入当前文件”，主流程使用系统文件选择框，不再要求手输路径。
- Export 预设改为系统保存弹框选择输出路径。
- Run Log 从一级入口改为抽屉式查看，主流程只保留结论反馈。
- Analyze 增加一键生成分析结果入口，保留单步 ETL 高级排错。
- 本轮未改 SQL、数据库结构、业务口径或依赖。

## 7. 技术栈

```text
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Desktop: Tauri 2
Backend: Rust
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Package manager: npm
```

## 8. 开发命令

```bash
npm install
npm run dev
npm run build
npm run tauri:dev
npm run tauri:build
cd src-tauri && cargo check
```

## 9. 当前状态

1.0.19 已修复 Universal Video detail CSV header alias / normalize 导致的 required mapping 误拦截问题。编译验证和 smoke 结果以本轮交付报告与 changelog 的实际执行记录为准；真实 MySQL / TCP / Game CSV smoke 如未执行，不得视为通过。

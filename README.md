# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.32
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

## 4. 1.0.32 收口重点

- 修复 `batch_tables.rs` 中被上一轮覆盖掉的公共 helper：`analysis_run_batch`、`table_exists`、`table_columns`，避免 Lead 查询和模块状态检查编译失败。
- 新增并注册 `analytics_materialize_app_rank` 命令，可执行 `003b_analytics_app_rank.sql`，把 App Rank 从 DWS 物化到结构化 Analytics ADS 表。
- `analyticsStructuredApi.ts` 已暴露 `materializeAppRank` 前端 API；按钮接入 TSX 时被 connector safety block，暂未渲染到 UI。
- 版本标记已同步到 `package.json` 和 `mapping_catalog.rs`；`tauri.conf.json`、`WorkbenchHeader.tsx`、`Cargo.toml` 仍受 connector safety block 限制。

## 5. 1.0.31 收口重点

- 新增 `AnalyticsStructuredPagedPanel.tsx`，把 1.0.30 后端分页/过滤能力暴露为独立可操作面板。
- `AnalysisWorkspace.tsx` 已接入分页结构化面板，用户可对 App、Hourly、Network、User、Lead 五类证据做后端分页、关键词、排序和阈值查询。
- 本轮继续保持前端只通过 Tauri command 访问 DWS/ADS 聚合结果，不直连 MySQL，不扫描 RAW。

## 6. 1.0.30 收口重点

- `DashboardRequest` 增加 `page`、`page_size`、`keyword`、`sort_by`、`min_value`。
- App Rank、Hourly Trend、Network Hotspot、User Profile 结构化命令支持后端分页/过滤/排序。
- 新增 `analytics_get_lead_evidence_page`，为 Lead Evidence 提供小型分页命令。
- `analyticsStructuredApi.ts` 支持 query options，并把 Lead Evidence 路由到分页命令。

## 7. 1.0.29 收口重点

- 新增 `analyticsStructuredCharts.ts`，把结构化 `MetricCard[]` 的 key=value hint 转换为 ECharts 友好的 chart rows。
- `AnalyticsDashboard.tsx` 现在刷新时并行调用 `analyticsStructuredApi` 的 KPI、App Rank、Hourly Trend、Network Hotspot、User Profile、Lead Evidence 六类接口。
- 主驾驶舱改为 Structured First：图表和证据区优先使用结构化 Analytics ADS 结果；结构化接口失败或无数据时保留旧 `workbenchApi` dashboard commands fallback。
- Overview、Apps、Quality、Cable、Users、Leads 各 Tab 均嵌入对应结构化 Evidence Table 入口。
- 不新增依赖，不修改 lock 文件，不改变 RAW → CLEAN/DWD → DWS/ADS 主链路。

## 8. 技术栈

```text
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Desktop: Tauri 2
Backend: Rust
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Package manager: npm
```

## 9. 开发命令

```bash
npm install
npm run dev
npm run build
npm run tauri:dev
npm run tauri:build
cd src-tauri && cargo check
```

## 10. 当前状态

1.0.32 在 1.0.31 分页结构化面板基础上，补回 batch table 公共 helper，并完成 App Rank Analytics ADS 物化命令的后端注册。编译验证和 smoke 结果以最终统一验证为准；真实 MySQL / customer CSV smoke 如未执行，不得视为通过。

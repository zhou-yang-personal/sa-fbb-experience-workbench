# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.28
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

## 4. 1.0.28 收口重点

- 结构化 App Rank、Hourly Trend、Network Hotspot、User Profile、Lead Evidence 从简单预览表升级为 `AnalyticsEvidenceTable`。
- 结构化结果现在支持关键词搜索、最小绝对值阈值过滤、排序、CSV 导出和 row-level 详情抽屉。
- `AnalyticsStructuredKpiPanel.tsx` 保留 KPI 卡片，同时把 App Rank / Hourly Trend 接入完整证据表。
- `AnalyticsStructuredDeepDivePanel.tsx` 把 Network / User / Lead 三类深钻结果接入完整证据表。
- `AnalyticsDashboard.css` 增加结构化 evidence grid 布局，支持两列表和三列表在宽屏下展开，在窄屏下自动收敛为单列。
- 继续保持 DWS / ADS 查询，不扫描 RAW，不新增依赖，不修改 lock。

## 5. 1.0.27 收口重点

- 新增 `database/migrations/006_analytics_dashboard_schema.sql`，定义看板专用 ADS 表：KPI Summary、App Rank、Hourly Trend、Network Hotspot、User Profile、Lead Evidence。
- `src-tauri/src/migrations.rs` 已接入 `006_analytics_dashboard_schema.sql`，数据库初始化会创建 analytics ADS schema。
- 新增结构化 Analytics 后端命令：`analytics_get_kpi_summary`、`analytics_get_app_rank`、`analytics_get_hourly_trend`、`analytics_get_network_hotspots`、`analytics_get_user_profiles`、`analytics_get_lead_evidence`。
- `src-tauri/src/main.rs` 已注册上述结构化命令，且所有新增命令只查询 DWS / ADS 物理表，不扫描 RAW。
- 新增 `analyticsStructuredApi.ts`，集中封装结构化 Analytics 的前端 Tauri invoke。
- 新增 `AnalyticsStructuredKpiPanel.tsx`，在分析页展示 KPI / App Rank / Hourly Trend 结构化预览。
- 新增 `AnalyticsStructuredDeepDivePanel.tsx`，在分析页展示 Network Hotspot / User Profile / Lead Evidence 结构化深钻预览。
- 版本标记已同步到 `package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`、`WorkbenchHeader.tsx`、`mapping_catalog.rs`、README、handoff、changelog。
- `workbenchApi.ts` 仍保留原有工作台 API；结构化 Analytics 命令统一通过 `analyticsStructuredApi.ts` 调用。

## 6. 1.0.26 收口重点

- 新增 `AnalyticsEvidenceTable.tsx`，把各核心看板表格升级为可搜索、可排序、可设置最小绝对值阈值、可导出 CSV、可点击查看详情的证据表。
- `AnalyticsDashboard.tsx` 增强图表类型：在原有 bar / donut / radar / line 基础上增加 scatter / heatmap / funnel，用于需求-体验散点、应用热力矩阵、Lead funnel 等产品化表达。
- 总览页增加 Insight Strip，直接给出业务需求入口、体验风险入口、转光证据入口、机会名单入口。
- 应用体验、网络质量、Cable vs FTTH、用户画像、迁转升套机会 Tab 均接入证据表，减少只能看小图、不能复核的问题。
- 本轮继续复用既有 DWS / ADS 后端 commands，不直接扫 RAW，不新增依赖，不修改 lock，不改数据库结构。

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

1.0.28 在 1.0.27 结构化 ADS / API 基础上，把结构化结果从预览表升级为可复核、可过滤、可导出、可查看详情的证据表。当前已具备 KPI、App Rank、Hourly Trend、Network Hotspot、User Profile、Lead Evidence 六类结构化命令和完整证据表入口。编译验证和 smoke 结果以最终统一验证为准；真实 MySQL / customer CSV smoke 如未执行，不得视为通过。

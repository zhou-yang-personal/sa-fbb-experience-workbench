# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.26
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

## 4. 1.0.26 收口重点

- 新增 `AnalyticsEvidenceTable.tsx`，把各核心看板表格升级为可搜索、可排序、可设置最小绝对值阈值、可导出 CSV、可点击查看详情的证据表。
- `AnalyticsDashboard.tsx` 增强图表类型：在原有 bar / donut / radar / line 基础上增加 scatter / heatmap / funnel，用于需求-体验散点、应用热力矩阵、Lead funnel 等产品化表达。
- 总览页增加 Insight Strip，直接给出业务需求入口、体验风险入口、转光证据入口、机会名单入口。
- 应用体验、网络质量、Cable vs FTTH、用户画像、迁转升套机会 Tab 均接入证据表，减少只能看小图、不能复核的问题。
- 本轮继续复用既有 DWS / ADS 后端 commands，不直接扫 RAW，不新增依赖，不修改 lock，不改数据库结构。
- 版本标记已同步到 `package.json`、`tauri.conf.json`、`mapping_catalog.rs`、README、handoff、changelog；`WorkbenchHeader.tsx` 和 `Cargo.toml` 版本写入被 ChatGPT GitHub connector safety block 拦截，需本地或 Codex 补齐。

## 5. 1.0.25 收口重点

- 数据分析页由模块卡片 + 小图骨架升级为大屏分析驾驶舱：总览、应用体验、网络质量、Cable vs FTTH、用户画像、迁转升套机会 6 个 Tab。
- 新增 `AnalyticsDashboard.tsx` 和 `AnalyticsDashboard.css`，提供 KPI 条、大图卡片、环图、雷达图、横向 TopN 条形图、趋势代理图和聚合结果表格。
- `AnalysisWorkspace` 默认渲染大屏驾驶舱，批次选择和分析上下文保留在顶部，模块可用性 / batch table registry 下沉到折叠诊断区。
- 看板仍只通过既有 DWS / ADS 后端 commands 获取聚合结果，不直接扫 RAW，不新增依赖，不改数据库结构。
- `WorkbenchAppV2` 更新分析页提示文案，不再说“按页面 8 步”，而是提示查看当前批次分析驾驶舱。

## 6. 1.0.24 收口重点

- 修复 Quality Gate / CLEAN 时间清洗中的 NBSP 处理：不再使用 `CHAR(160)`，改为 `CONVERT(0xC2A0 USING utf8mb4)`，避免 MySQL `ERROR 3854 Cannot convert string '\xA0' from binary to utf8mb4`。
- TCP / Game 的 Quality Gate 和 RAW→CLEAN SQL 已同步修复，继续保留 tab / CR / LF / NBSP 转空格、连续空白压缩、`stat_time_text` guarded `STR_TO_DATE` 和单数字日/月支持。
- Pipeline 实时日志和执行计划错误显示增加 CSS 高度兜底：日志区域内部滚动，执行计划和失败卡片不再被完整 SQL statement 撑开。
- Rust SQL 片段测试同步为 `CONVERT(0xC2A0 USING utf8mb4)`，并断言不再包含 `CHAR(160)`。

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

1.0.26 在 1.0.25 大屏驾驶舱基础上继续强化证据表、导出、详情抽屉和更多图表表达。当前版本仍未新增 ADS 表结构，继续复用既有 DWS / ADS commands；后续应继续落地结构化图表 API、分页用户明细、Lead 证据链和更完整的业务 ADS 表。编译验证和 smoke 结果以本轮交付报告与 changelog 的实际执行记录为准；真实 MySQL / customer CSV smoke 如未执行，不得视为通过。

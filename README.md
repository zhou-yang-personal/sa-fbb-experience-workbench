# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.6
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

详细设计见：`docs/design/current-core-design.md`

## 3. Phase 1-7 完整应用基线

1. Phase 1：本地工程骨架、版本文件、MySQL 连接、数据库初始化。
2. Phase 2：大 CSV 导入入口、批次管理、RAW 入库主路径。
3. Phase 3：RAW 质量门禁、RAW → CLEAN、完整 DWS 聚合。
4. Phase 4：Overview、应用分类、体验质量、Cable vs FTTH 看板 ADS。
5. Phase 5：Lead scoring、瓶颈归因、迁转升套分层。
6. Phase 6：CRM、FTTH 覆盖、可触达状态融合，生成最终营销动作。
7. Phase 7：导出、handoff、changelog、交付检查入口。

## 4. 1.0.6 修复重点

- 新增映射化 RAW 导入实现：`src-tauri/src/raw_import_v2.rs`。
- RAW 导入 command 已切到 mapped import adapter。
- 新增 mapping validation command，支持把 CSV header 与 `cfg_import_field_mapping` 校验结果写入 `meta_mapping_validation_result`。
- 新增 observability schema：导入行错误、字段映射校验、数据集 profile。
- 新增 ETL job step inspection 和 quality result inspection command。
- 前端入口已切到模块化 `WorkbenchAppV2`，并拆分 Connection / Import / Operations / Metric / Result / Log 组件。
- 新增 ECharts metric bar 图表组件。

## 5. 技术栈

```text
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Desktop: Tauri 2
Backend: Rust
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Package manager: npm
```

## 6. 开发命令

```bash
npm install
npm run dev
npm run build
npm run tauri:dev
npm run tauri:build
cd src-tauri && cargo check
```

## 7. 当前状态

1.0.6 继续补齐“极致完整代码基线”：mapped RAW import、mapping validation、observability schema、job/quality inspection、模块化前端和图表组件已落地。没有在本地执行依赖安装、前端构建、Rust 检查、Tauri 打包和真实 MySQL / CSV 端到端验证。`src-tauri/tauri.conf.json` 仍因 connector safety block 未同步到 1.0.6。

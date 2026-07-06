# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.16
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

## 4. 1.0.16 修复重点

- 主界面从 Start / Import / Validate / Analyze / Results 流程向导调整为产品功能树：数据分析 / 数据导入 / 系统管理。
- 默认入口改为“数据分析”，进入分析前先选择或输入导入批次。
- 导入前强制设置可读批次名称，后端保存 `meta_import_batch.batch_display_name`。
- 分析模块声明必填字段、适用数据类型和所需聚合表；基础条件不满足时置灰并显示原因。
- 取消独立结果导出模块，导出按钮保留在迁转升套机会等看板内部。
- 诊断日志增强：失败日志保留更长结果预览，映射 required 缺失时列出具体 target 字段。
- 本轮未把所有 RAW / CLEAN / DWS / ADS 改为每批次独立物理表；当前仍以共享表 + `import_batch_id` 隔离为主，物理独立表是下一轮数据库主链路改造。

## 5. 1.0.15 修复重点

- 主界面从功能模块入口收口为 5 步向导：Start / Import / Validate / Analyze / Results。
- 增加全局 Pipeline 状态条、下一步提示和当前动作反馈条。
- 增加统一 ActionButton，支持 running / success / failure / disabled 状态反馈。
- Import 主流程改为“选择 CSV 文件 + 导入当前文件”，主流程使用系统文件选择框，不再要求手输路径。
- Export 预设改为系统保存弹框选择输出路径。
- Run Log 从一级入口改为抽屉式查看，主流程只保留结论反馈。
- Analyze 增加一键生成分析结果入口，保留单步 ETL 高级排错。
- 本轮未改 SQL、数据库结构、业务口径或依赖。

## 6. 技术栈

```text
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Desktop: Tauri 2
Backend: Rust
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Package manager: npm
```

## 7. 开发命令

```bash
npm install
npm run dev
npm run build
npm run tauri:dev
npm run tauri:build
cd src-tauri && cargo check
```

## 8. 当前状态

1.0.16 已在任务分支上完成产品功能树落地改造。ChatGPT GitHub connector 环境不能执行本地构建命令，需本地继续运行 `npm run check`、`npm run build`、`cd src-tauri && cargo check` 和 `npm run tauri:build`。

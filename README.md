# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.5
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

## 4. 1.0.5 修复重点

- 新增 `cfg_import_field_mapping`，用于沉淀不同 CSV header 到 RAW target column 的映射关系。
- 新增 `cfg_final_join_rule`，用于沉淀 CRM / Reachability / Coverage 与 SA Lead 的 join key 规则。
- 新增默认配置种子：`database/seeds/002_default_mapping_seed.sql`。
- 数据库初始化已纳入 mapping schema 和默认 mapping seed。
- 新增配置查询 command 模块和配置化 Final Fusion 构建模块，供后续 UI / command 注册使用。

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

1.0.5 继续补齐配置化能力：导入字段映射表、商业融合 join key 规则表和默认种子已经落地。由于 ChatGPT GitHub connector 对部分 Rust/JSON 整文件更新触发 safety block，`src-tauri/tauri.conf.json` 仍需人工同步版本并继续完成 command 注册。没有在本地执行依赖安装、前端构建、Rust 检查、Tauri 打包和真实 MySQL / CSV 端到端验证。

# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
0.1.0
```

## 1. 核心目标

1. 支持千万级 CSV 文件不经应用内内存清洗，优先原样导入 MySQL RAW 表。
2. 基于 MySQL 完成 RAW → CLEAN/DWD → DWS/ADS 分层清洗和聚合。
3. 支撑总览、应用分类、体验质量、Cable vs FTTH、迁转升套机会等看板。
4. 输出可复核、可重跑、可导出的用户级机会名单。

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
→ 前端看板查询 DWS / ADS
→ 用户名单 / 汇总结果导出
```

详细设计见：

```text
docs/design/current-core-design.md
```

## 3. 技术栈

```text
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Desktop: Tauri 2
Backend: Rust
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Package manager: npm
```

## 4. 目录结构

```text
.
├─ AGENTS.md
├─ AGENTS.common.md
├─ AGENTS.project.md
├─ README.md
├─ package.json
├─ index.html
├─ vite.config.ts
├─ tsconfig.json
├─ src/
├─ src-tauri/
├─ database/
│  ├─ migrations/
│  └─ sql/
└─ docs/
   ├─ design/current-core-design.md
   ├─ requirements/current-requirements.md
   ├─ handoff/latest-handoff.md
   ├─ changes/CHANGELOG-dev.md
   └─ development/chatgpt-github-connector-guide.md
```

## 5. 开发命令

安装依赖：

```bash
npm install
```

启动前端：

```bash
npm run dev
```

启动 Tauri：

```bash
npm run tauri:dev
```

前端构建：

```bash
npm run build
```

Tauri 打包：

```bash
npm run tauri:build
```

Rust 检查：

```bash
cd src-tauri && cargo check
```

## 6. 当前状态

0.1.0 已完成工程骨架、架构文档、基础前端页面、Tauri command stub、MySQL 分层 DDL 与核心 SQL 模板。实际数据库连接、LOAD DATA 执行和端到端构建验证仍需后续在本地环境继续验证。

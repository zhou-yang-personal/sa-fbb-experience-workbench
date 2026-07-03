# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
0.2.1
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

## 4. 初步可用链路

0.2.1 已把 1-8 步落地到 `dev`：

1. 本地工程骨架和版本文件。
2. MySQL 连接、数据库初始化和 migration runner。
3. CSV probe、import batch 创建。
4. TCP / Game RAW 入库命令入口。
5. RAW 质量检查指标查询。
6. RAW → CLEAN、CLEAN → DWS、DWS → ADS SQL runner。
7. 前端真实调用 Tauri command，支持 dashboard、lead 查询和导出入口。
8. README、handoff、changelog、AGENTS.project.md 同步到 0.2.1。

## 5. 目录结构

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
│  ├─ seeds/
│  └─ sql/
└─ docs/
   ├─ design/current-core-design.md
   ├─ requirements/current-requirements.md
   ├─ handoff/latest-handoff.md
   ├─ changes/CHANGELOG-dev.md
   └─ development/chatgpt-github-connector-guide.md
```

## 6. 开发命令

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

## 7. 当前状态

0.2.1 已完成初步可用链路的代码落地，但尚未在本地执行依赖安装、前端构建、Rust 检查、Tauri 打包和真实 MySQL / CSV 端到端验证。

# AGENTS.project.md｜SA FBB Experience Workbench 项目定制检查清单

本文件只承载本项目定制规则。每日治理同步任务不得自动覆盖本文件；只有用户在当前会话中明确授权时才可以修改。

## B0. 项目身份

- [ ] 项目名称已确认：`SA FBB Experience Workbench`。
- [ ] 仓库已确认：`zhou-yang-personal/sa-fbb-experience-workbench`。
- [ ] 产品定位已确认：基于 SA 家宽应用体验数据的本地 EXE 分析工作台。
- [ ] 核心用户场景已确认：CSV 大文件原样入库、MySQL 库内清洗、Cable/FTTH 体验对比、迁转升套机会识别、统计分析看板。
- [ ] 推荐技术栈已确认：`Tauri 2 + React + TypeScript + Vite + ECharts + Rust + MySQL 8.0`。

## B1. 当前开发基线检查

- [ ] 默认分支已确认：`main`。
- [ ] 当前 source-of-truth 开发分支已确认：`dev`。
- [ ] 修改前已从目标分支读取最新目标文件。
- [ ] 常规任务分支命名建议使用：`chatgpt/task-xxx`。
- [ ] Codex 任务分支命名建议使用：`codex/task-xxx`。
- [ ] PR 目标分支默认已确认：`dev`。
- [ ] 只有用户明确要求直接修改目标分支时，才允许跳过任务分支。

## B2. 本项目必读文件检查

开始任何需求设计、代码修改、UI 调整或 PR 验收前，必须读取：

- [ ] `AGENTS.md`
- [ ] `AGENTS.common.md`
- [ ] `AGENTS.project.md`
- [ ] `README.md`
- [ ] `docs/design/current-core-design.md`
- [ ] `docs/requirements/current-requirements.md`，如存在。
- [ ] `docs/handoff/latest-handoff.md`，如存在。
- [ ] `docs/changes/CHANGELOG-dev.md`，如存在。

按任务类型追加读取：

- [ ] 涉及 ChatGPT GitHub connector 操作时，已读取项目内 connector guide。
- [ ] 涉及版本时，已读取所有项目版本文件。
- [ ] 涉及依赖时，已读取 package / lock / dependency manifest。
- [ ] 涉及后端、数据库、ETL、UI、构建或发布时，已读取对应模块文件。

## B3. 本项目产品方向一致性检查

- [ ] 本项目不是普通 BI，而是 SA 家宽应用体验数据本地分析工作台。
- [ ] 核心链路必须保持：`CSV 文件选择 → 文件元信息登记 → MySQL RAW 表原样高速导入 → RAW 入库完整性校验 → MySQL 库内清洗/标准化/衍生字段 → CLEAN/DWD → DWS/ADS 聚合 → 看板查询聚合结果`。
- [ ] CSV 大文件不得在应用内做全量内存清洗；只允许读取少量样本用于预览、字段识别和映射。
- [ ] 大文件导入优先使用 `LOAD DATA LOCAL INFILE`；客户环境禁用时，使用流式分块 INSERT fallback。
- [ ] 看板不得直接扫 RAW 明细表；应查询 DWS / ADS 聚合结果。
- [ ] 迁转升套逻辑不得把“体验差用户”直接等同于“升套潜客”；必须区分高应用需求、轻度体验承压、网络侧严重异常、家庭侧/Wi-Fi 侧问题。
- [ ] 当前 SA 数据只能输出“应用体验驱动的迁转升套机会”；正式营销名单需要后续 JOIN CRM、套餐、FTTH 覆盖、可触达状态等数据。

## B4. 数据处理架构约束

- [ ] RAW 层用于原样承接 CSV，字段可优先用 `VARCHAR / TEXT`，不要在 RAW 阶段强制业务转换。
- [ ] 每条 RAW 记录必须能追溯 `import_batch_id`、`source_file_name`、必要时的 `source_line_no`。
- [ ] CLEAN / DWD 层用于日期、数值、单位、应用分类、接入类型、用户主键、质量标记和衍生字段标准化。
- [ ] DWS 层用于用户级、小时级、应用级、接入类型级、网络侧聚类级聚合。
- [ ] ADS 层用于看板结果和名单结果，前端优先查询 ADS。
- [ ] 必须保留导入日志、清洗日志、聚合日志和失败重跑能力。
- [ ] SQL 参数不使用 `SET @var` 作为主链路，优先使用 CTE 参数块、临时参数表或配置表。
- [ ] 不提交客户 CSV、数据库导出、运行日志、构建产物或安装包。

## B5. 看板与业务模块约束

第一阶段功能优先级：

1. CSV 原样导入与批次管理。
2. RAW → CLEAN 清洗任务。
3. CLEAN → DWS / ADS 聚合任务。
4. 总览看板。
5. 应用分类详情看板。
6. RTT / PLR / MOS / VMOS 体验质量看板。
7. Cable vs FTTH 对比看板。
8. 迁转升套机会用户列表。
9. 用户明细导出。

## B6. 本项目版本检查

- [ ] 当前版本已确认：`1.0.7`。
- [ ] 前端版本文件已同步：`package.json`。
- [ ] Tauri 版本文件当前需人工确认：`src-tauri/tauri.conf.json` 更新到 `1.0.7` 被 connector safety block 拦截。
- [ ] Rust package 版本文件已同步：`src-tauri/Cargo.toml`。
- [ ] README 当前版本已同步：`README.md`。
- [ ] 最新交接版本已同步：`docs/handoff/latest-handoff.md`。
- [ ] 变更记录已同步：`docs/changes/CHANGELOG-dev.md`。
- [ ] 不改依赖时，未修改 lock 文件。

## B7. 本项目构建与 CI 检查

本项目可用命令：

- [ ] 安装依赖：`npm install`。
- [ ] 前端开发：`npm run dev`。
- [ ] 前端构建：`npm run build`。
- [ ] 前端类型检查：`npm run check`。
- [ ] Tauri 开发启动：`npm run tauri:dev`。
- [ ] Tauri 打包：`npm run tauri:build`。
- [ ] Rust 检查：`cd src-tauri && cargo check`。

CI 状态：

- [ ] 当前未建立 CI；未发现 CI 时，不得声称 CI 通过。

## B8. 本项目禁止事项检查

- [ ] 不提交凭据、运行数据、缓存、数据库导出、客户 CSV、安装包或构建产物。
- [ ] 不在应用内对几千万行 CSV 做全量内存清洗。
- [ ] 不让前端看板直接查询 RAW 大表。
- [ ] 不把临时 SQL 阈值写死成不可配置规则。
- [ ] 不做无关 UI 风格重写。
- [ ] 不删除用途未确认的旧模块。
- [ ] 不修改用户未授权的发布、依赖或数据库结构。

## B9. 本项目交付附加要求

每次修改完成后必须汇报：目标分支、任务分支、commit hash、最新版本号、修改文件清单、做了什么、没做什么、是否修改依赖 / lock 文件、是否执行 build / test / check、未验证项和原因。

## B10. ChatGPT GitHub Connector 操作检查

- [ ] 已读取项目内 connector guide。
- [ ] 使用 `compare_commits` 检查分支差异；不得用 `update_ref` 做分支状态探测。
- [ ] 使用 `fetch_file` 获取文件内容和 sha。
- [ ] 使用 `update_file` 更新已有 UTF-8 文本文件。
- [ ] 使用 `create_file` 新增小型 UTF-8 文本文件。
- [ ] 每次写入后已回读关键文件确认。
- [ ] 遇到 safety block、not fast-forward、sha 冲突时，已停止说明或重新读取后再判断，未盲目重试。
- [ ] 操作结束前已复盘是否出现新的 connector 问题或更优流程。
- [ ] 如出现新经验，已更新项目内 connector guide 或公共治理仓。

# AGENTS.md｜项目开发协作执行入口

本文件是本仓库进行需求设计、代码修改、UI 调整、PR 验收和人工开发时的最高优先级入口。所有执行者必须先读取本文件；无法读取时必须暂停并要求用户提供文件内容。

## 1. 必须继续读取

在执行任何需求设计、代码修改、UI 调整或 PR 验收前，必须继续读取：

```text
AGENTS.common.md
AGENTS.project.md
docs/requirements/manual-feedback-p0.md
```

如果本次涉及 GitHub connector 操作，还必须读取：

```text
docs/development/chatgpt-github-connector-guide.md
```

## 2. 文件职责

```text
AGENTS.common.md                          # 跨项目通用规则，可由公共治理仓同步
AGENTS.project.md                         # 本项目定制规则，不得被公共同步任务覆盖
AGENTS.md                                 # 固定入口，只负责引导读取和声明优先级
docs/requirements/manual-feedback-p0.md   # 人工反馈问题 / 需求清单；自动 Review P0 优先入口
```

## 3. 冲突优先级

```text
用户明确指令 > docs/requirements/manual-feedback-p0.md 未完成项 > AGENTS.project.md > AGENTS.common.md > 公共治理仓模板 > 一般经验
```

## 4. 自动同步边界

每日治理同步任务只允许自动更新：

```text
AGENTS.common.md
docs/development/chatgpt-github-connector-guide.md
```

每日治理同步任务不得自动覆盖：

```text
AGENTS.project.md
docs/requirements/manual-feedback-p0.md
AGENTS.md
.github/pull_request_template.md
项目 handoff
项目 changelog
项目业务代码
项目业务文档
数据库 / ETL / UI / 构建逻辑
依赖文件和 lock 文件
```

除非用户在当前会话中明确授权。

## 5. 执行要求

- 修改前必须读取对应文件。
- 文件缺失时必须说明“未发现 / 未确认”。
- 自动 Review 任务开始后，必须读取 `docs/requirements/manual-feedback-p0.md`；其中未勾选项作为 P0 最高优先级处理。
- 自动 Review 任务必须一次性尽力处理完 `docs/requirements/manual-feedback-p0.md` 中所有未勾选项；不得用自动发现的 P0 覆盖手动反馈 P0。
- 如果无法一次处理完，必须逐项说明阻塞原因、已完成部分和下一步，不得只输出笼统结论。
- 自动 Review 任务结束前必须更新 `docs/requirements/manual-feedback-p0.md`：已修复项打勾 `[x]`，并简要记录修复方案、涉及文件、验证状态、commit / PR；未修复项保留 `[ ]` 并更新进展。
- 涉及版本号时必须按 `AGENTS.project.md` 的项目版本规则同步。
- 不改依赖时不得修改 lock 文件。
- 不得虚构 build / test / CI 通过。
- GitHub connector 操作结束前，必须检查是否出现新问题或更优流程；如有，更新 connector guide 或公共治理仓。

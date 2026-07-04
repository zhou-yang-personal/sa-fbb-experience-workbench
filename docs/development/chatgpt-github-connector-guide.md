# ChatGPT GitHub Connector 操作手册

本文件记录 ChatGPT 使用 GitHub connector 处理远端仓库文件、分支、提交和 PR 的能力边界、推荐流程、失败处理和经验回写机制。涉及 GitHub 远端提交、分支、PR、文件写入或版本同步时，应先读取本文件。

本文件是从公共治理仓 `zhou-yang-personal/ai-dev-governance-kit/guides/chatgpt-github-connector-guide.md` 同步而来，可在本地保留项目特有补充。

本文件说明 connector 可如何操作 PR 模板、版本文件等文本文件，不代表公共仓必须把 PR 模板作为每日治理同步的默认写入对象。

## 1. 适用范围

- 适用于 ChatGPT 通过 GitHub connector 直接读取或修改 GitHub 仓库的任务。
- 适用于文档、源码、配置、PR 模板、版本文件等 UTF-8 文本文件操作。
- 不替代具体项目的 `AGENTS.md`、`AGENTS.common.md`、`AGENTS.project.md`、产品设计文档、代码规范和用户明确指令。
- 与具体项目规则冲突时，以用户明确指令和项目仓 `AGENTS.project.md` 为准。

## 2. 常用能力

只读能力：

- `get_repo`：读取仓库元信息、默认分支、权限和状态。
- `fetch_file`：按仓库路径读取 UTF-8 文本文件，并获取当前文件 sha。
- `search`：在仓库内搜索文件、函数、错误文本或关键字。
- `compare_commits`：比较两个 ref / commit 的差异，用于检查分支 ahead / behind、变更文件清单。
- `fetch_commit`：读取单个 commit 的 diff、文件清单和链接。
- `get_pr_info` / `fetch_pr_patch`：读取 PR 元信息和 patch。

写入能力：

- `create_file`：新增小型 UTF-8 文本文件。
- `update_file`：基于当前 sha 替换已有 UTF-8 文本文件。
- `delete_file`：基于当前 sha 删除已有文件。
- `create_branch`：从指定 base ref 或 commit 创建任务分支。
- `create_pull_request`：创建 PR。

不适合作为常规手段的能力：

- `update_ref`：只用于明确移动分支指针；不得用于分支状态探测、noop 检查或试错。
- `search_branches`：只可作为辅助搜索；不得用其空结果判断带 `/` 的分支不存在。
- `create_blob` / `create_tree` / `create_commit`：仅在需要组合多文件 commit 且已有完整 tree / parent 信息时使用；常规修改优先用 `create_file` / `update_file`。

## 3. 推荐流程

1. 用 `get_repo` 确认仓库、默认分支、权限和 archived / fork 状态。
2. 用 `compare_commits` 检查目标分支差异。
3. 如需判断带 `/` 的分支是否存在，优先直接用 `fetch_file(ref=<branch>)` 或 `compare_commits` 指定 ref 验证，不依赖 `search_branches` 空结果。
4. 用 `fetch_file` 读取 `AGENTS.md`、`AGENTS.common.md`、`AGENTS.project.md`、README、handoff、设计文档、目标文件和 sha。
5. 更新已有文本文件：先 `fetch_file` 获取 sha，再 `update_file`。
6. 新增小文本文件：使用 `create_file`。
7. 删除文件：先确认文件职责和当前 sha，再用 `delete_file`。
8. 每次写入后，必须 `fetch_file` 回读关键片段。
9. 多文件修改必须串行执行，不并发写同一路径。
10. sha 冲突时，重新 `fetch_file` 获取最新内容，重新判断是否继续；不得盲目覆盖。
11. 创建 PR 后，不自动 merge，除非用户明确要求。

## 4. 文件类型规则

| 文件类型 | 推荐操作 | 注意事项 |
|---|---|---|
| Markdown | `fetch_file` + `update_file` / `create_file` | 适合 connector 直接操作；长文改动优先小步提交 |
| TypeScript / TSX | `fetch_file` + `update_file` | 修改后提示运行前端构建 |
| Rust / TOML | `fetch_file` + `update_file` | 修改后提示运行 Rust 检查 |
| Python | `fetch_file` + `update_file` | 修改后提示运行 py_compile / compileall |
| JSON 配置 | 优先最小修改 | 整文件写入可能触发平台拦截；失败后停止说明，不盲试 |
| PR 模板 | `create_file` / `update_file` | 只放确认清单，不复制 AGENTS 全文；不是每日治理同步默认写入对象 |
| lock 文件 | 默认不改 | 只有新增、删除、升级依赖时才改 |
| 大文件 | 优先交给 Codex / 本地 | connector 不适合反复整文件重写 |
| 二进制、图片、DB、安装包、构建产物 | 默认不通过 connector 操作 | 不提交运行数据和构建产物 |

## 5. 常见失败与处理方式

| 失败现象 | 正确处理 |
|---|---|
| `fetch_file` 返回 404 | 记录文件未发现；如需新增，用 `create_file` |
| `create_file` 提示文件已存在 | 改用 `fetch_file` + `update_file` |
| `update_file` sha 冲突 | 重新读取文件和 sha 后再判断 |
| `update_ref` 返回 not fast-forward | 不 force；除非用户明确要求且已说明风险 |
| `search_branches` 对带 `/` 的分支返回空 | 不据此判断分支不存在；直接用 `fetch_file(ref=<branch>)` 或 `compare_commits` 指定 ref 验证 |
| 平台 safety block | 停止说明被拦截的文件和操作类型；不得多轮盲试 |
| 构建命令无法执行 | 明确写 `not run in ChatGPT GitHub connector environment` |
| CI 未配置或无法读取 | 不声称 CI 通过 |

## 6. 经验回写机制

每次 GitHub connector 操作结束前，执行者必须检查：

- 本次是否出现新的接口限制。
- 本次是否出现新的平台写入拦截。
- 本次是否出现 sha 冲突、路径错误、文件不存在或文件类型不适配。
- 本次是否出现接口误用，例如用 `update_ref` 做分支状态探测。
- 本次是否发现更优的读取、比较、写入、回读、交付流程。

如新经验具备跨项目复用价值，优先回写本文件；如只属于某个项目，写入项目仓自己的 connector guide 或 `AGENTS.project.md`。

不得新增独立 `feedback-loop` 文件或新的每日必读治理入口。

## 7. 已知踩坑记录

- 2026-07-02：不要使用 `update_ref` 做分支状态探测；使用 `compare_commits` 检查分支差异。
- 2026-07-02：配置类 JSON 整文件更新可能触发平台安全拦截；后续优先最小修改，失败后停止说明。
- 2026-07-02：README 长目录树整文件恢复可能触发拦截；如只是版本号修改，应避免重写整个 README。
- 2026-07-04：较长 Rust 整文件重写也可能触发平台安全拦截；遇到 `src-tauri/src/main.rs`、`raw_import.rs` 这类大入口文件时，优先在本地或 Codex 分小补丁处理。

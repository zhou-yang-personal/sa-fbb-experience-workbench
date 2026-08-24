# SA FBB Experience Workbench

SA FBB Experience Workbench 是一个本地 EXE 数据分析工作台，用于处理 SA 单板输出的家宽 TCP / Game 应用体验 CSV 数据，完成大文件原样入库、MySQL 库内清洗、聚合分析、Cable/FTTH 体验对比和 Cable-to-Fiber 迁转升套机会识别。

当前版本：

```text
1.0.40
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

## 4. 1.0.40 收口重点

- 修复 Rust MySQL 客户端未注册 `LOCAL INFILE` 数据处理器、导致 MySQL 接收到空文件并返回 0 行且无 warning 的问题。
- `LOAD DATA LOCAL INFILE` 现在以 1 MiB 缓冲流式传输用户当前选择的 CSV，不会把 1 GB+ 文件整体载入应用内存。
- MySQL 发起的本地文件请求必须解析为当前选择文件的同一规范路径；其他路径会被拒绝，处理器在单次导入后立即移除。

## 5. 1.0.39 收口重点

- RAW 导入不再接受“0 行成功”：LOAD DATA 与 Streaming INSERT 均验证批次物理表可见数据，0 行时在 RAW 步骤直接失败并返回分隔符、文件和 MySQL warning 证据。
- CSV Probe、字段映射、LOAD DATA 和 Streaming INSERT 统一使用有界探测得到的逗号、Tab 或分号分隔符。
- 数据画像改为查询批次物理 RAW 表；流水线失败后自动展示 RAW 状态和 Quality Gate 失败项。
- 补齐客户 TCP 样例中的平均带宽和用户有效下载速率字段别名。

## 6. 1.0.38 收口重点

- 历史批次支持按批次或批量删除，级联清理物理表、RAW/DWD/DWS/ADS 数据、分析结果和任务元数据；运行中任务禁止删除。
- MySQL 密码默认填入 `123456`，允许用户覆盖且不把覆盖值持久化到 localStorage。
- 修复 `ads_network_hotspot_rank` 的 utf8mb4 组合主键超过 InnoDB 3072 字节限制导致数据库初始化失败的问题。

## 7. 1.0.37 收口重点

- 修复 Windows 桌面端 CSV 文件选择器：补齐 Tauri Dialog 后端插件和最小权限配置，文件选择失败时显示可诊断错误。
- TCP / Game 每次导入必须手动选择并确认一个已发布 IP 规则版本；后端不再静默绑定最新版本。

## 8. 1.0.36 收口重点

- 新增 GitHub Actions Windows 自动构建：任务分支和 `dev` 推送生成 30 天 Artifact，`v*` 标签自动发布 GitHub Release。
- 自动构建依次执行前端类型检查、Rust 测试和 Tauri Windows MSI / NSIS EXE / portable EXE 打包。

## 9. 1.0.35 收口重点

- 新增版本化 IPv4 网段配置，支持 Cable / FTTH / Other 草稿编辑、重叠校验、批次抽样预览、发布和批次绑定。
- RAW → DWD 接入类型按“已发布 IP 规则优先、源字段回退、未匹配标记”分类，并保留规则版本和证据来源。
- 分析入口重组为经营总览、应用体验、网络问题定位、Cable vs FTTH、用户洞察和迁转机会六类任务看板。
- App、网络拓扑和 Lead 看板改为真实业务粒度；A0 身份不足与 A2 先修障不会被表述为可直接营销名单。
- 批次 SQL 统一绑定物理表；乱序 CSV 表头通过 MySQL 用户变量映射后仍走 `LOAD DATA LOCAL INFILE`，看板只读 DWS / ADS。

## 10. 1.0.34 收口重点

- App / Hourly / Network / User / Lead 五类结构化查询命令已支持 materialized Analytics ADS 优先读取。
- 如果目标 `analysis_run_id` 暂无物化 ADS 数据，或老库缺少 Analytics ADS base 表，查询会安全回退到原 DWS / Lead 表路径。
- Evidence hint 中增加 `source=...`，便于区分当前结果来自物化 ADS 还是 fallback 聚合表。
- 本轮继续保持前端只通过 Tauri command 访问后端，不直连 MySQL，不扫描 RAW。

## 11. 1.0.33 收口重点

- 新增 `AnalyticsAdsActions.tsx`，在 Analysis Workspace 中提供结构化 ADS 物化操作入口。
- App / Hourly / Network / User / Lead 五类结构化 ADS 物化命令均已注册到 Tauri。
- `analyticsStructuredApi.ts` 已暴露五类物化 API，前端动作面板可触发对应命令。

## 12. 1.0.32 收口重点

- 修复 `batch_tables.rs` 中被上一轮覆盖掉的公共 helper：`analysis_run_batch`、`table_exists`、`table_columns`，避免 Lead 查询和模块状态检查编译失败。
- 新增并注册 `analytics_materialize_app_rank` 命令，可执行 `003b_analytics_app_rank.sql`，把 App Rank 从 DWS 物化到结构化 Analytics ADS 表。

## 13. 1.0.31 收口重点

- 新增 `AnalyticsStructuredPagedPanel.tsx`，把 1.0.30 后端分页/过滤能力暴露为独立可操作面板。
- `AnalysisWorkspace.tsx` 已接入分页结构化面板，用户可对 App、Hourly、Network、User、Lead 五类证据做后端分页、关键词、排序和阈值查询。

## 14. 技术栈

```text
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Desktop: Tauri 2
Backend: Rust
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Package manager: npm
```

## 15. 开发命令

```bash
npm install
npm run dev
npm run build
npm run tauri:dev
npm run tauri:build
cd src-tauri && cargo check
```

## 16. 当前状态

1.0.40 修复 `LOAD DATA LOCAL INFILE` 未实际传输本地 CSV 的根因，并以受限路径、固定缓冲流式传输。真实 MySQL 导入和 1 GB+ customer CSV 基准测试未执行前，不得声称生产性能已验证。

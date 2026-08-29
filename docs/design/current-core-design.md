# SA FBB Experience Workbench｜当前核心架构设计

## 1.0.67 公共聚合核心

体验分析的公共事实粒度固定为 `analysis run × 日期 × 小时 × 分析用户 IP × App × 接入类型`。`dws_user_app_hourly_experience_v2` 按实际日期/小时读取 DWD，每片独立提交，并为流量、时长、速率、连接、RTT、丢包、Wi-Fi、vMOS/MOS 和 jitter 保存可累加的 `SUM + 非空 COUNT`。均值字段只用于兼容读取，不作为后续再聚合的计算输入。

`dws_user_app_period_experience_v2`、App/接入周期表以及旧版 App 日、App 用户、分类日、接入小时兼容表只能从公共核心派生，不再分别扫描 DWD。网络位置、Server IP 和其他无法由公共粒度无损表达的专项证据继续独立计算。公共核心先于兼容 DWS/ADS 执行；分片与子任务 checkpoint 同时绑定实现版本和来源版本，代码口径变化会使旧成功断点失效，进程或 MySQL 中断则只重跑未成功分片。

## 1.0.59 全量洞察与问题高亮

体验总览是完整洞察入口，不以 Finding 替代全量分布。总览显式加载 ADS/DWS 后同时展示 App 组合状态、用户覆盖、业务规模、持续问题、用户需求、接入趋势、网络/路径证据和机会分层；Findings 保留为独立的异常清单和 Investigation 入口。

应用体验的展示粒度为 `App × Access Type`。所有已物化组合必须保留，前端直接使用当前 analysis run 绑定策略生成的 `sample_status` 和 `attention_level` 高亮 Normal、Attention、Severe、Insufficient Sample 与 Legacy/Unclassified，禁止在 React 中另设隐藏问题阈值。样本不足与空指标不得按 0 或正常展示。

结构化看板查询必须为只读操作并使用有界 ADS/DWS 结果。命令边界捕获历史数据类型转换 panic，单个数据集失败只能降级为可见错误，不能终止桌面进程；schema migration 只能出现在初始化或显式维护流程，不能由“加载当前看板”触发。

## 1. 设计结论

本项目定位为 **SA 家宽应用体验数据本地分析工作台**，不是普通 BI。第一阶段目标是把 SA TCP / Game CSV 大文件在本地稳定入库、库内清洗、聚合分析，并输出 Cable-to-Fiber 迁转升套机会名单和统计看板。

核心链路必须采用：

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

约束：

1. CSV 大文件可能达到几千万行，不允许应用内全量读入内存做清洗。
2. CSV 只允许读取少量样本用于预览、字段识别和字段映射。
3. 数据清洗、字段标准化、应用分类、用户画像、体验评分和迁转分层优先在 MySQL 内完成。
4. 看板不得直接扫 RAW 明细大表，必须查询 DWS / ADS 聚合结果。
5. 当前 SA 数据只能形成“应用体验驱动的迁转升套机会”，正式营销名单需要后续 JOIN CRM、套餐、FTTH 覆盖、合约、欠费、黑名单和可触达状态。

### 1.1 1.0.35 产品决策面

主分析入口按用户决策任务组织，而不是按 ETL 表组织：

```text
经营与体验总览
├─ 应用体验：真实 App → 受影响用户 → 体验与网络证据
├─ 网络问题定位：BRAS / OLT / PON → 问题侧 → 建议行动
├─ Cable vs FTTH：同日期小时、同单位、同分母对比
├─ 用户洞察：需求、使用、体验和接入识别证据
└─ 迁转升套机会：A0 身份不足 / A2 先修障 / A1 待资格校验
```

所有图表必须显示单位或分母，点击后可查看证据字段。少于 8 个时间点时，趋势图降级为柱图，避免暗示不存在的趋势。

### 1.2 数据作业中心交互边界

数据导入与批次管理按操作者任务拆分为三个工作区：

```text
新建导入：CSV + 批次名称 + 规则确认 → 启动完整流水线
批次库：刷新/搜索/选择批次 → 查看建议 → 进入分析或选择恢复方式
运行任务：显式加载状态 → 阶段计划 → 实时日志 → 成功进入分析或失败恢复
```

约束：

1. 打开页面、切换工作区和选择批次只更新本地上下文，不自动访问 MySQL。
2. 批次列表、流水线状态、日志和自动刷新必须由用户显式启动。
3. 新建、RAW 重建和 DWS/ADS 续跑成功启动后进入同一个任务监控工作区。
4. RAW 重建与 DWS/ADS 续跑必须解释适用条件，不把两个高风险动作当成等价主按钮。
5. 手工 1-8 步、映射目录、Registry 和诊断表仅在开发者诊断模式展示。
6. 操作者场景评分与验收路径以 `docs/design/import-job-center-operator-journey.md` 为准。

### 1.3 按需分析与任务交互

分析工作区采用显式任务模型：应用启动和恢复本地上下文只展示批次信息，不自动准备批次表、统计大表或读取看板。用户进入具体决策页后点击“加载当前看板”，系统只查询该页所需的 DWS / ADS 数据集；总览所需的多个数据集按步骤顺序执行，并展示计划、进度、当前步骤和失败信息。停止操作不宣称中断 MySQL 已开始的语句，而是在当前语句返回后跳过尚未开始的后续步骤。高级诊断组件只有在用户展开后才挂载。

批次状态分为两个独立层次：`meta_import_batch.status` 只表示 RAW 入库状态，`meta_pipeline_run.status` 才表示 Quality Gate、CLEAN/DWD、DWS/ADS 和模块就绪的整体流水线状态。批次选择器必须解析该批次最新流水线，并同步它的 `analysis_run_id`；不得沿用上一个批次或更早分析运行的上下文。看板只有在至少一个必需数据集含有有效聚合证据时才可标记成功，空数组与全 0 KPI 结构属于“无分析结果”，需要引导用户返回导入页查看该批次流水线。

系统诊断遵循同一任务模型：切换到诊断页只渲染轻量配置与空状态，不调用 MySQL。用户启动后，Catalog、映射失败项、质量失败项、ETL 失败项、模块深度检查和 Registry 快照按顺序执行；停止只跳过尚未开始的检查。常规模块深度检查使用批次表的有界 `EXISTS` 查询判断可用性，Registry 使用已缓存数值或 `information_schema.tables.table_rows` 估算，不在页面导航、批次准备或 Module Ready 时执行隐式大表精确 `COUNT(*)`。Quality / ETL 单步工具和完整执行日志仅在对应折叠区展开后挂载。

1.0.52 将上述边界收紧为零数据库访问安全启动：应用打开和本地上下文恢复只读取 WebView 本地状态，不调用批次、analysis run、流水线或就绪度 API；批次列表也必须由用户显式刷新。Windows 运行时另写入不含业务数据的本地 `runtime.log`，用于区分正常退出、Tauri 运行错误和 Rust panic。1.0.53/1.0.54 进一步要求所有批次及批次表注册元数据读取兼容历史 `NULL`/空值：Rust 使用可失败的逐列解码，单条脏元数据不得导致桌面进程退出；刷新批次、加载运行列表、加载规则和恢复流水线必须是相互独立的显式操作。

历史批次同时提供两个不同恢复动作：已有 CLEAN 可用时从 DWS/ADS 续跑；规则或清洗口径变化时从既有 RAW 全量重建。RAW 重建保留 CSV/RAW 和旧 analysis run，以新 run 执行分块 CLEAN/DWD、CLEAN 后质量验证、DWS/ADS/V2、可选 Final Lead 与 Module Ready。CLEAN/DWD 与不按 run 隔离的 DWS 是批次共享结果，会被重建覆盖；ADS/V2 使用新 run 隔离。核心 SQL 脚本经 SQL runner 拆分后，为每条语句写入开始、成功或失败、耗时、影响行数与有界摘要；pipeline 元数据写入不参与自递归日志。

RAW→CLEAN 使用批次物理 RAW 表的自增主键分片，每 50 万行独立提交。批次物理 CLEAN 表在重建开始时使用 `TRUNCATE`，批量写入期间暂卸用户/应用/小时二级索引，全部分片结束后一次性恢复；失败路径同样尝试恢复索引。完整质量验证复用 CLEAN 中已解析的时间、身份、接入类型和质量标记，避免对千万行 RAW 再做一次正则日期解析。该策略降低单事务 undo、锁和索引写放大，但真实总耗时必须在目标 MySQL 与客户数据上验收。

hourly V2 禁止整批单事务。任务先确保批次 DWD 物理表具有 `(import_batch_id, stat_date, hour_of_day)` 索引，再枚举实际日期/小时分片；每片使用 `DELETE 当前 run/date/hour + INSERT 当前 run/date/hour` 的独立事务。`meta_aggregation_partition_checkpoint` 保存每片的连接 ID、尝试次数、耗时、影响行数与错误，成功片在恢复时直接跳过。MySQL 命名锁保证同一实例只有一个 DWS/ADS 聚合，MySQL 重启后显式状态查询会把无活动 SQL 的遗留运行标记为 `interrupted`。只有完整运行进入 `success/degraded` 后，前端才允许把 run 标记为可分析。

### 1.4 全部图表 PDF 导出

PDF 导出是独立的显式分析任务，不跟随页面切换自动运行。任务启动时锁定当前 `import_batch_id`、`analysis_run_id`、接入类型、关键词和最小用户数，随后顺序读取 KPI、App Rank、小时趋势、网络热点、用户画像和 Lead Evidence 六个 DWS/ADS 数据集。已开始的数据库语句不做虚假中断；停止操作只跳过后续数据集。

报告按经营总览、应用体验、网络问题定位、Cable vs FTTH、用户洞察和迁转升套机会六个章节保留看板原有图表口径，共覆盖 20 个图表位置。当前筛选下为空的图不生成空白页，而是在封面列为已跳过；查询失败同样显式记录。报告只包含图表和必要的上下文元数据，不包含证据明细表。首版复用 Windows WebView2 打印预览保存 PDF，以 CSS 打印版式生成 A4 横向报告，不引入额外 PDF 运行库或修改依赖锁；无对话框的原生 `PrintToPdf` 可作为后续增强。

### 1.2 接入类型识别

Cable / FTTH 识别采用可追溯的版本规则：

```text
已发布 IPv4 网段规则
→ 绑定 import_batch_id 和规则版本
→ RAW → DWD 时用 INET_ATON(local_ip_address) 匹配
→ 命中规则：IP_RULE / HIGH
→ 有效 IPv4 未命中：使用该规则版本显式配置的 Others / RULE_SET_OTHERS / HIGH
→ 缺失或非法 IPv4：UNKNOWN / UNAVAILABLE_IP / LOW
```

`Others` 是未命中任何显式 IP 网段的剩余有效 IPv4 集合，不是写死的第三种接入技术。规则草稿必须由用户明确选择 Others 最终归为 Cable、FTTH 或 Other；未配置或配置为 Unknown 的草稿不得预览、发布或绑定新批次。当前业务版本选择 Others → Cable，但这个选择只保存在规则版本中，不得成为 React、Rust、SQL 或数据库默认值。CSV `user_type` / `wan_type` 只保留为来源证据，不参与最终分类优先级。规则支持 CIDR 或起止 IPv4、启停、优先级、重叠阻断、最多 100,000 个不同 IP 的有界预览及原子发布。规则应用不修改 RAW；应用到历史批次后必须重跑 CLEAN / DWS / ADS。

### 1.6 V2 体验指标与旁路聚合

V2 App 分析采用版本化 `Experience Policy`，每个 `analysis_run` 同时绑定接入规则版本、Others 归类和体验策略版本。首轮公共粒度为用户 × App × 分析周期，并继续汇总为 App × 接入制式；看板只读对应 ADS，不扫描 RAW。

必须区分：

1. `Poor Observation Rate = 差体验观测 / 有效体验观测`。
2. `Ever Affected User Rate = 至少一次差体验的合格用户 / 合格用户`。
3. `Persistent Poor User Rate = 满足最低观测数、最低异常次数和用户差体验率门槛的用户 / 合格用户`。
4. `Severe Poor User Rate = 满足独立严重阈值、最低严重次数和严重率门槛的用户 / 合格用户`。

V2 表必须保留分子、分母、样本状态、策略版本和主要证据指标；`INSUFFICIENT_SAMPLE` 不得进入问题排名。缺失、未导入、样本不足和数值 0 是不同状态。当前手工脚本复用既有 DWD，为指定批次并排生成 V2 DWS/ADS，不覆盖旧分析运行，也不进行 Server IP 全量拆分。

### 1.5 看板总体口径与数据覆盖

看板额外读取一个有界的数据覆盖状态：TCP / Game RAW 是否存在、CLEAN 是否就绪，以及绑定规则集的未命中默认类型。独立 Game 文件不存在时，游戏时长与 MOS 标记为不可用，不生成伪 0 图表或把 0 加入 Lead 判断解释。

机会阶段和用户分布使用完整 ADS 表的 `GROUP BY` 聚合，明细分页只用于证据列表。Cable / FTTH 多日小时数据在图表层按 `active_users` 加权形成典型 24 小时曲线，避免 7 天 145 个时点拥挤；原始日期小时仍保留在证据表与导出数据中。

## 2. 参考基线与差异

整体工程框架参考 `latam-fbb-desktop` 的桌面端本地分析架构：

```text
React / Vite UI
→ Tauri invoke
→ Rust ETL / Query Backend
→ MySQL Raw / Clean / Aggregate
→ Dashboard 查询与展示
```

可复用的设计原则：

- Raw First：源文件先进入专属 Raw 宽表。
- Clean / Aggregate 分层：清洗、聚合、运行查询分离。
- Dashboard 不直接操作大表源数据。
- 大表处理必须可观测、可续跑、可解释。
- loading table + quality gate + atomic swap 优先。

本项目与 `latam-fbb-desktop` 的主要差异：

| 维度 | latam-fbb-desktop | SA FBB Experience Workbench |
|---|---|---|
| 数据来源 | 政府公开 FBB 市场数据 | SA 单板 TCP / Game 应用体验数据 |
| 核心目标 | 市场快照、竞争分析、国家维度分析 | Cable-to-Fiber 迁转升套、体验质量、用户机会识别 |
| 主数据库 | MySQL + DuckDB Runtime | 第一阶段以本地 MySQL 为主 |
| 数据粒度 | 国家 / 城市 / 运营商 / 技术 / 速率档 | 用户 / 应用 / 小时 / 接入类型 / 网络侧字段 |
| 看板重点 | 市场份额、速率、城市、竞争 | 应用使用、RTT、PLR、MOS/VMOS、Cable vs FTTH、Lead 分层 |
| 导入主链路 | Raw → Clean → Aggregate → Runtime | CSV → RAW → CLEAN/DWD → DWS/ADS |

第一阶段不强制引入 DuckDB Runtime。只有当后续出现“脱离 MySQL 运行看板”“客户演示便携包”“离线发包分析”等需求时，再评估 MySQL → DuckDB Runtime 发布能力。

## 3. 技术选型

推荐第一阶段技术栈：

```text
Desktop: Tauri 2
Frontend: React + TypeScript + Vite
Chart: Apache ECharts
Backend: Rust Tauri commands
Database: MySQL 8.0
CSV Import: LOAD DATA LOCAL INFILE + streaming INSERT fallback
Config: JSON / TOML
Logs: local rolling log + MySQL job tables
Package: Windows EXE / MSI
```

选型理由：

1. **Tauri + Rust** 适合做轻量 Windows EXE，本地文件读取、导入调度、日志和系统配置能力强。
2. **React + TypeScript** 适合快速构建导入工作台、数据任务状态页和多看板交互。
3. **ECharts** 更适合本项目需要的 Treemap、环形图、堆叠柱、小时折线、TopN 横向柱和深色大屏风格。
4. **MySQL 8.0** 作为本地计算与存储主引擎，承接几千万行 RAW、清洗 SQL、聚合表和看板查询。
5. **LOAD DATA LOCAL INFILE** 作为大 CSV 主导入能力，应用层不做大文件内存清洗。

1.0.35 的导入器先读取表头并生成 `@csv_N → RAW target column` 映射，因此列顺序变化不会再自动降级为批量 INSERT。只有显式关闭 `local_infile` 或选择 fallback 模式时才走有界的 500 行 streaming INSERT。

## 4. 前端模块架构

建议目录结构：

```text
src/
├─ app/
│  ├─ App.tsx
│  ├─ routes.tsx
│  └─ appConfig.ts
├─ features/
│  ├─ import-center/
│  ├─ data-quality/
│  ├─ etl-jobs/
│  ├─ dashboard-overview/
│  ├─ app-category-detail/
│  ├─ experience-quality/
│  ├─ cable-fiber-compare/
│  ├─ migration-leads/
│  ├─ user-profile/
│  └─ settings/
├─ shared/
│  ├─ api/
│  ├─ charts/
│  ├─ components/
│  ├─ formatters/
│  ├─ i18n/
│  └─ types/
└─ styles/
```

### 4.1 Import Center

职责：

- 选择 CSV 文件。
- 读取前 100 行预览。
- 识别编码、分隔符、header、文件大小、文件 hash。
- 选择数据类型：TCP / Game / CRM / Coverage / Reachability。
- 配置字段映射。
- 创建 import batch。
- 触发 RAW 导入任务。
- 展示导入进度、速度、剩余时间、失败原因。

流水线日志采用 `pipeline_run_id + sequence` 作为唯一有序游标。前台同一时刻只允许一个增量请求，按 sequence 去重并在终态后补拉一次最终日志；如果积压超过单页限制，连续分页追平而不是跳过。只读状态/日志轮询不执行 schema DDL，最近一次 pipeline ID 按 MySQL 上下文保存在本地，以便离开导入页后恢复监控。RAW 文件传输每 5 秒报告字节进度，质量检查、CLEAN/DWD、DWS/ADS、融合和模块检查等无法获得 MySQL 内部百分比的步骤每 15 秒写入存活心跳、步骤耗时和当前阶段说明。监控台分别展示轮询健康、日志静默时长、计划进度和筛选结果，45 秒无心跳时提示数据库繁忙或连接受阻，但不把“无百分比”误判为失败。新产生的 pipeline run、step 和 log 时间使用 `UTC_TIMESTAMP()` 写入 MySQL，API 返回带 `Z` 的 UTC 时间；前端统一通过浏览器 `Intl.DateTimeFormat` 转换为本地 PC 时区，并在复制文本中写入 IANA 时区名称。旧版 `DATETIME` 历史数据不携带时区元数据，因此仅从本版本新任务开始保证转换准确。

恢复日志以当前选中批次为准，而不是只读取“本机最后一次 pipeline ID”。历史列表查询返回该批次最新的 `pipeline_run_id`、流水线状态、失败原因和 `analysis_run_id`；进入导入页或重新选择批次时从 sequence 0 分页恢复日志。若批次来自旧版或手工 RAW 导入且没有关联流水线，则清空上一批次的监控状态并明确提示没有可恢复日志。

对于 RAW、Quality Gate 和 CLEAN 已有成功证据、但 DWS/ADS 不完整的批次，Import Center 提供“复用当前批次”显式任务。该任务不重读 CSV、不重建 RAW/CLEAN，从基础用户日聚合开始依次完成完整 DWS、基础 ADS、App Rank、小时趋势、网络热点、用户画像和 Lead Evidence，然后尝试 Final Lead 并刷新 Module Ready。后端在创建续跑任务前检查同批次流水线状态和 MySQL `PROCESSLIST`；只有用户明确确认原进程已退出且该批次无活动 SQL 时，才能接管遗留的 running 元数据。

DWS/ADS 的可观测性细化为 10 个命名子阶段，其中公共体验核心独立于兼容 DWS/ADS；每个子阶段写入开始、完成或失败日志，外层长步骤仍保持 15 秒数据库事实探测。`meta_analysis_run` 在基础用户日聚合完成后保持 `running`，只有公共核心、完整 DWS 与结构化 ADS 均成功后才转为 `success`。

### 4.2 Data Quality

职责：

- RAW 入库行数校验。
- CSV 文件行数与 RAW 行数对比。
- 字段缺失率、空值率、0 值率、UNKNOWN 率。
- user_account / user_mac 可用性检查。
- Cable / FTTH 分布检查。
- 时间范围、小时分布和应用数量检查。
- 拓扑字段 BRAS / OLT / PON / WAN 有效率检查。

### 4.3 ETL Jobs

职责：

- 管理 RAW → CLEAN 任务。
- 管理 CLEAN → DWS 任务。
- 管理 DWS → ADS 任务。
- 支持失败重跑。
- 支持分步骤查看 SQL、耗时、状态、message。
- 支持基于 import_batch_id 重跑。

### 4.4 Dashboard Overview

对应总览页：

- 用户数。
- 总流量。
- 总使用时长。
- Heavy User 数。
- Cable / FTTH 用户占比。
- 应用分类使用时长排行。
- 应用分类总流量排行。
- Heavy Users by Traffic Category。
- Internet Usage Distribution。
- Usage / Total Traffic by Category Treemap。

### 4.5 App Category Detail

按应用分类钻取：

- OTT Video。
- Short Video。
- Live Video。
- Game。
- Video Conference。
- Office。
- Web / Others。

指标：

- 用户数。
- 应用数。
- 使用时长。
- 下载量。
- 有效下载速率。
- 忙时活跃度。
- Top App。
- Top User。
- Cable / FTTH 对比。

### 4.6 Experience Quality

体验质量页：

- VMOS / MOS 分布。
- 用户侧 RTT。
- 网络侧 RTT。
- 用户侧下行丢包。
- 用户侧上行丢包。
- 网络侧下行丢包。
- 网络侧上行丢包。
- Game jitter。
- Wi-Fi delay。
- 用户侧 / 网络侧 / Wi-Fi 侧瓶颈拆分。

### 4.7 Cable Fiber Compare

Cable vs FTTH 对比页：

- 每小时用户数。
- 每小时平均 RTT。
- 每小时平均丢包。
- 每小时平均有效下载速率。
- 每小时 VMOS / MOS。
- 忙时与非忙时对比。
- 每用户粒度明细。
- Cable 相比 FTTH 的体验差异。

### 4.8 Migration Leads

迁转升套机会页：

- Lead Type 漏斗。
- A1 / A0 / A2 / B / C / D 用户数量。
- 需求分、迁转动力分。
- 推荐套餐。
- 用户级明细表。
- 一键导出 CSV / Excel。
- 后续 CRM JOIN 状态。

### 4.9 User Profile

单用户画像：

- user_key / user_account / user_mac。
- key_confidence。
- user_type。
- 应用偏好。
- 忙时活跃。
- 视频、短视频、直播、游戏指标。
- 用户侧 RTT / 网络侧 RTT / 丢包 / Wi-Fi delay。
- Lead Type。
- 推荐动作。

## 5. Rust / Tauri 后端模块架构

建议目录结构：

```text
src-tauri/src/
├─ main.rs
├─ commands/
│  ├─ db_commands.rs
│  ├─ file_commands.rs
│  ├─ import_commands.rs
│  ├─ etl_commands.rs
│  ├─ dashboard_commands.rs
│  ├─ export_commands.rs
│  └─ settings_commands.rs
├─ db/
│  ├─ mysql_pool.rs
│  ├─ migrations.rs
│  ├─ query_runner.rs
│  └─ transaction.rs
├─ import/
│  ├─ csv_probe.rs
│  ├─ csv_manifest.rs
│  ├─ load_data.rs
│  ├─ streaming_insert.rs
│  └─ import_batch.rs
├─ etl/
│  ├─ raw_to_clean.rs
│  ├─ clean_to_dws.rs
│  ├─ dws_to_ads.rs
│  ├─ job_runner.rs
│  └─ quality_gate.rs
├─ export/
│  ├─ csv_export.rs
│  └─ excel_export.rs
├─ settings/
│  ├─ app_settings.rs
│  └─ mysql_settings.rs
└─ logging/
   └─ app_logger.rs
```

### 5.1 后端职责边界

Rust 后端负责：

- 本地文件选择与文件信息读取。
- CSV 样本读取，不做全量清洗。
- MySQL 连接管理。
- LOAD DATA LOCAL INFILE 调度。
- 流式分块 INSERT fallback。
- ETL SQL 编排。
- 任务状态记录。
- 失败重跑。
- 导出文件。
- 本地配置与日志。

Rust 后端不负责：

- 几千万行 CSV 的业务清洗。
- 大表全量内存聚合。
- 看板口径硬编码在前端。
- 用应用内循环替代 MySQL 聚合 SQL。

## 6. MySQL 数据分层设计

### 6.1 元数据层

```text
meta_import_batch
meta_import_file
meta_import_field_mapping
meta_etl_job
meta_etl_job_step
meta_quality_check_result
meta_app_config
```

职责：

- 管理导入批次。
- 记录文件元信息。
- 记录字段映射模板。
- 记录 ETL 任务和步骤。
- 记录质量检查结果。
- 记录全局配置和阈值版本。

### 6.2 DIM 配置层

```text
dim_app_mapping
dim_app_category
dim_threshold_config
dim_offer_rule
dim_access_type_mapping
dim_key_confidence_rule
dim_lead_type_rule
```

职责：

- 应用名称标准化。
- 应用分类映射。
- Cable / FTTH 标准化。
- 阈值配置。
- 推荐套餐规则。
- Lead Type 规则。

### 6.3 RAW 原始入库层

```text
raw_tcp_detail_import
raw_game_detail_import
raw_crm_user_import
raw_ftth_coverage_import
raw_reachability_import
```

设计原则：

- 原样承接 CSV。
- 每条记录带 `import_batch_id`。
- 每条记录带 `source_file_name`。
- 必要时带 `source_line_no`。
- 字段优先使用 VARCHAR / TEXT，日期和数字先不强制转换。
- RAW 表不建立过多索引，避免拖慢导入。
- RAW 表不做复杂生成列。

### 6.4 CLEAN / DWD 标准明细层

```text
dwd_tcp_detail_clean
dwd_game_detail_clean
dwd_user_identity_clean
dwd_user_access_type_daily
```

处理内容：

- 时间字段标准化。
- 数值字段标准化。
- 速率 Kbps / Mbps 转换。
- 流量 KB / MB / GB 转换。
- 丢包率单位确认与统一。
- RTT / Jitter / Wi-Fi delay 数值化。
- 应用名称标准化。
- 应用分类映射。
- Cable / FTTH 标准化。
- user_key 生成。
- key_confidence 生成。
- worst_latency / worst_loss / worst_jitter 衍生。
- invalid_app_flag / data_quality_flag 生成。

### 6.5 DWS 聚合层

```text
dws_user_hourly_metrics
dws_user_daily_profile
dws_user_app_category_daily
dws_app_category_daily
dws_access_type_hourly_compare
dws_network_cluster_daily
dws_user_experience_bottleneck
```

聚合粒度：

- import_batch_id。
- stat_date。
- hour_of_day。
- user_key。
- user_type。
- app_category。
- app_name。
- bras / olt / pon。

### 6.6 ADS 看板结果层

```text
ads_dashboard_overview
ads_app_category_detail
ads_experience_quality_summary
ads_cable_fiber_compare
ads_migration_lead_funnel
ads_migration_lead_summary
ads_migration_lead_user
ads_build_priority_cluster
```

职责：

- 面向前端查询。
- 查询稳定、轻量、可分页。
- 不做重型业务计算。
- 以 import_batch_id / analysis_run_id 隔离不同批次结果。

## 7. 大 CSV 导入设计

### 7.1 主方案：LOAD DATA LOCAL INFILE

流程：

```text
1. 前端选择文件
2. Rust 读取文件元信息和前 100 行
3. 用户确认数据类型和字段映射
4. 写入 meta_import_batch / meta_import_file
5. 生成 RAW 表导入 SQL
6. 执行 LOAD DATA LOCAL INFILE
7. 记录导入开始、结束、耗时、行数、错误
8. 执行 RAW 层质量检查
```

Probe 只读取有界样本识别逗号、Tab 或分号，字段映射、LOAD DATA 和 Streaming INSERT 必须复用同一分隔符。Rust MySQL 客户端在每次 LOAD DATA 前注册仅允许当前所选 CSV 规范路径的 `LocalInfileHandler`，通过 1 MiB 缓冲流式传输文件，拒绝服务端请求其他本地路径，并在语句结束后立即移除处理器。RAW 长步骤以共享原子计数器记录客户端已处理字节，每 5 秒通过独立 MySQL 连接写入 pipeline heartbeat；达到 100% 后显示 MySQL 解析/索引/提交等待，连续 30 秒无变化则输出分阶段 warning。批次创建后立即绑定到 pipeline run。LOAD DATA 返回后先验证当前 `import_batch_id` 在批次物理 RAW 表中可见；0 行必须在 RAW 步骤直接失败并保留 MySQL warning，不得进入 Quality Gate 后才以四项通用错误暴露。

适用：

- 千万级 CSV。
- 本地 MySQL 允许 `local_infile`。
- 文件在本机可访问。

### 7.2 Fallback：流式分块 INSERT

当客户环境禁用 `LOAD DATA LOCAL INFILE` 时使用：

- Rust 流式读取 CSV。
- 每 5,000 / 10,000 行批量 INSERT。
- 只做最小转义和字段数量对齐。
- 不做业务清洗。
- 支持失败行记录。
- 支持断点续跑。

### 7.3 质量门禁

RAW 导入后必须执行：

- 文件行数 vs RAW 行数。
- 字段数量检查。
- 空行检查。
- user_account / user_mac 空值率。
- user_type 分布。
- 时间范围。
- 应用数量。
- Cable / FTTH 分布。
- UNKNOWN 拓扑字段比例。

质量门禁失败时：

- 阻断 RAW → CLEAN。
- 在流水线失败卡片直接展示失败项、RAW 批次状态和物理表行数。
- 允许用户选择修正映射后重跑。

数据画像必须通过批次表注册表解析物理 RAW 表；共享 RAW 基表仅作为 `CREATE TABLE ... LIKE` 模板，不作为已分批数据的画像来源。

## 8. 清洗与聚合任务设计

### 8.1 RAW → CLEAN

使用 SQL 执行清洗，不在应用内循环清洗。

关键动作：

- 字段类型转换。
- 日期解析。
- 数值解析。
- 应用分类 JOIN dim_app_mapping。
- user_key 生成。
- key_confidence 标记。
- 体验字段标准化。
- 衍生指标生成。

### 8.2 CLEAN → DWS

关键聚合：

- 用户小时级使用。
- 用户日级画像。
- 应用分类日级聚合。
- Cable / FTTH 小时对比。
- 用户侧 / 网络侧 / Wi-Fi 侧瓶颈拆分。
- 网络侧聚类。

### 8.3 DWS → ADS

关键结果：

- 看板总览。
- 应用分类详情。
- 体验质量摘要。
- Cable vs FTTH 对比。
- 迁转升套漏斗。
- 用户级 Lead 明细。

### 8.4 SQL 变量策略

MySQL SQL 不依赖如下写法作为主实现：

```sql
SET @run_time = NOW();
SET @min_valid_user_rows = 3;
SET @min_cluster_users = 10;
```

统一优先使用：

- CTE 参数块。
- 临时参数表。
- `dim_threshold_config` 配置表。
- Rust 将参数写入 `meta_etl_job` / `dim_threshold_config` 后再执行 SQL。

示例：

```sql
WITH params AS (
  SELECT
    NOW() AS run_time,
    3 AS min_valid_user_rows,
    10 AS min_cluster_users
)
SELECT *
FROM params;
```

## 9. 用户识别与主键策略

当前 SA 数据中 `user_account` 可能出现：

- IPv4。
- MAC。
- masked MAC / masked account。
- 其他账号格式。

因此设计上不能假设 `user_account` 一定可直接 JOIN CRM。

建议生成统一 `user_key`：

```text
优先级 1：可靠 user_account
优先级 2：可靠 user_mac
优先级 3：user_account 看起来是 MAC
优先级 4：local_ip + 时间窗口，仅用于分析，不作为营销主键
```

同步生成 `key_confidence`：

```text
HIGH_ACCOUNT_KEY
MEDIUM_MAC_USER_KEY
LOW_MASKED_ACCOUNT_KEY
LOW_IP_ONLY_KEY
UNKNOWN_KEY
```

Lead 输出时：

- `HIGH_ACCOUNT_KEY` 可进入正式 CRM JOIN 候选。
- `MEDIUM_MAC_USER_KEY` 需要 CRM / 装机系统补充映射。
- `LOW_IP_ONLY_KEY` 只能做体验分析或区域聚类，不直接营销。

## 10. 应用分类设计

建议一级分类：

```text
long_video_ott
short_video
live_video
game
cloud_gaming
video_conference
office
web
social
other
invalid_app
```

映射逻辑：

- `universal_video_applications` → TCP / Video 应用名。
- `application_protocol` → Game 应用名。
- 通过 `dim_app_mapping` 做标准化。
- 明显识别错误或设备/更新类应用可标记 `invalid_app`，不进入 TOP 应用分析。

## 11. 体验指标设计

### 11.1 TCP / Video 指标

核心字段：

- VMOS。
- connection establishment success rate。
- connection establishment delay。
- upstream / downstream RTT。
- network side RTT。
- subscriber side RTT。
- user avg download rate。
- effective download rate。
- downloaded data volume。
- effective download duration。
- video download duration。
- network / user side packet loss。
- Wi-Fi delay。

### 11.2 Game 指标

核心字段：

- MOS。
- connection establishment success rate。
- upstream / downstream RTT。
- network side RTT。
- subscriber side RTT。
- network / user side packet loss。
- upstream / downstream jitter。
- heartbeat latency。
- game duration。
- single flow rate。
- Wi-Fi delay。
- worst latency / worst loss / worst jitter。

### 11.3 体验瓶颈归因

建议拆分：

```text
NETWORK_SIDE_SEVERE
USER_SIDE_OR_WIFI_PRESSURE
APP_DEMAND_HIGH_BUT_EXPERIENCE_OK
PEAK_HOUR_PRESSURE
DATA_INSUFFICIENT
```

原则：

- 网络侧严重异常用户不直接进入营销升套名单，应进入网络优化或建网评估。
- 用户侧 / Wi-Fi 侧压力用户适合 Wi-Fi 6 ONT / Mesh / Fiber + Wi-Fi 组合包。
- 高应用需求且体验轻度承压的 Cable 用户是 A1 迁转营销重点。

## 12. 迁转升套评分模型

### 12.1 Demand Score

需求分来自：

- OTT 使用强度。
- 短视频使用强度。
- 直播使用强度。
- 游戏使用强度。
- 忙时活跃度。
- 多场景并发。
- 下载量。
- 有效时长。

建议输出 0–100 分。

### 12.2 Migration Motive Score

迁转动力分来自：

- Cable 用户。
- Cable 相比 FTTH RTT 更高。
- Cable 相比 FTTH 丢包更高。
- Cable VMOS / MOS 更低。
- 忙时体验承压。
- 适合 Fiber 产品承接。
- 非网络侧严重异常。

建议输出 0–100 分。

### 12.3 Lead Type

```text
A1_Cable高需求且有迁转动力_可优先营销
A0_高价值但CRM主键待确认
A2_Cable高需求但网络侧异常_先优化或建网
B_Cable高需求但迁转动力不足_培育池
C_FTTH存量高速升套用户
D_普通观察用户
```

### 12.4 推荐套餐

推荐三档：

1. Fiber 300M / 500M：基础迁转包，主打稳定、对称体验。
2. Fiber 500M / 600M + Wi-Fi 6 / OTT：家庭娱乐包，主打高清视频、直播、世界杯、家庭多设备。
3. Fiber 900M / 1G + Mesh + 游戏 / 直播权益：高价值包，主打游戏低时延、多设备并发、家庭娱乐中心。

## 13. 看板查询接口设计

前端只通过 Tauri command 访问后端，不直连 MySQL。

建议 command：

```text
db_test_connection
import_probe_csv
import_create_batch
import_start_raw_load
import_get_progress
quality_get_batch_report
etl_start_clean_job
etl_start_aggregate_job
etl_get_job_detail
dashboard_get_overview
dashboard_get_app_category_detail
dashboard_get_experience_quality
dashboard_get_cable_fiber_compare
leads_get_funnel
leads_get_summary
leads_query_users
export_leads_csv
settings_get
settings_update
```

接口约束：

- 所有列表接口必须支持分页。
- 所有看板接口必须带 `import_batch_id` 或 `analysis_run_id`。
- 大结果导出走后端文件流，不把全量数据传给前端内存。
- 错误信息必须带 job_id / step_id / SQL stage，便于定位。

## 14. 可观测性设计

任务表：

```text
meta_etl_job
meta_etl_job_step
meta_quality_check_result
```

每个任务记录：

- job_id。
- import_batch_id。
- job_type。
- status。
- started_at。
- finished_at。
- duration_ms。
- current_step。
- total_steps。
- affected_rows。
- error_code。
- error_message。

每个步骤记录：

- step_name。
- source_table。
- target_table。
- SQL template name。
- started_at。
- finished_at。
- status。
- affected_rows。
- message。

## 15. 性能策略

### 15.1 导入性能

- RAW 导入优先 LOAD DATA。
- RAW 表导入阶段不建过多索引。
- CLEAN / DWS / ADS 可以在写入完成后建立必要索引。
- 大批次任务分步骤提交，避免长事务压死本地 MySQL。

### 15.2 查询性能

- 前端只查 DWS / ADS。
- 高频筛选字段建联合索引。
- 用户明细必须分页。
- TopN 查询预聚合。
- Cable / FTTH 小时对比预计算。

### 15.3 存储治理

- 支持按 batch 删除 RAW / CLEAN / DWS / ADS。
- 批次删除同时清理关联分析运行、导出记录、质量结果、映射结果和任务日志；运行中的 RAW / ETL / pipeline 必须阻断删除。
- 每批物理表删除前必须校验注册表归属和安全表名，删除失败后允许对同一批次重试收口。
- 支持归档旧 batch。
- 不提交本地数据库文件和导出文件到 Git。

`ads_network_hotspot_rank` 保留 BRAS / OLT / PON 的 `VARCHAR(255)` 原值，但使用自增主键和 128 字符前缀的查询索引，避免 utf8mb4 组合键超过 InnoDB 3072 字节限制。

## 16. 目录结构建议

```text
.
├─ AGENTS.md
├─ AGENTS.common.md
├─ AGENTS.project.md
├─ README.md
├─ package.json
├─ package-lock.json
├─ index.html
├─ vite.config.ts
├─ tsconfig.json
├─ src/
├─ src-tauri/
├─ database/
│  ├─ migrations/
│  ├─ seeds/
│  └─ sql/
│     ├─ raw_to_clean/
│     ├─ clean_to_dws/
│     └─ dws_to_ads/
└─ docs/
   ├─ design/current-core-design.md
   ├─ requirements/current-requirements.md
   ├─ handoff/latest-handoff.md
   ├─ changes/CHANGELOG-dev.md
   └─ development/chatgpt-github-connector-guide.md
```

## 17. MVP 范围

### V0.1：架构骨架与数据导入闭环

- Tauri + React + TypeScript 工程初始化。
- MySQL 连接配置与连接测试。
- CSV 文件探测。
- import batch 管理。
- TCP RAW 导入。
- Game RAW 导入。
- RAW 质量检查。
- RAW → CLEAN 基础清洗。
- CLEAN → DWS 基础聚合。
- 导入与 ETL 日志页面。

### V0.2：核心看板

- 总览页。
- 应用分类详情页。
- 体验质量页。
- Cable vs FTTH 对比页。
- 用户明细查询。

### V0.3：迁转升套机会

- Demand Score。
- Migration Motive Score。
- Lead Type 分层。
- 推荐套餐。
- Lead 漏斗。
- 用户名单导出。

### V0.4：客户营销闭环增强

- CRM 套餐表导入。
- FTTH 覆盖表导入。
- 可触达状态导入。
- 正式营销名单过滤。
- 转化结果回填。
- ROI 复盘。

## 18. 风险边界

1. **主键风险**：user_account 可能不是 CRM 账号，必须保留 key_confidence。
2. **口径风险**：丢包率、成功率、VMOS/MOS、download_fluency 的单位和含义必须通过样本和客户口径校准。
3. **拓扑风险**：BRAS / OLT / PON 可能 UNKNOWN 比例高，区域建网分析不能强行依赖无效字段。
4. **营销风险**：网络侧严重异常用户不应直接营销升套。
5. **性能风险**：前端不得加载全量用户明细；必须分页和后端导出。
6. **交付风险**：第一阶段不提交客户 CSV、数据库、导出名单、日志、安装包。

## 19. 下一步工程落地顺序

1. 初始化 Tauri + React + TypeScript + Vite 工程。
2. 建立 README、requirements、handoff、changelog。
3. 建立 MySQL metadata / dim / raw / dwd / dws / ads migration。
4. 实现 MySQL 连接配置。
5. 实现 CSV probe。
6. 实现 import batch。
7. 实现 TCP / Game RAW 导入。
8. 实现 RAW 质量检查页。
9. 实现 RAW → CLEAN SQL。
10. 实现 CLEAN → DWS / ADS SQL。
11. 实现 Overview / Experience / Cable-Fiber / Leads 四类核心看板。
## 1.0.50 investigation architecture

`WorkbenchController` owns a single persistent Analysis Context shared by the V2 overview, findings, legacy evidence pages and Investigation Workspace. Chart selections append dimensions to this context instead of creating isolated local filters.

The V2 query path is ADS/DWS-first:

1. `dws_user_app_period_experience_v2` supplies the four experience metrics and affected-user evidence.
2. `ads_app_experience_v2` supplies explainable, sample-qualified App findings.
3. `dws_user_app_hourly_experience_v2` and `ads_app_hourly_experience_v2` supply time drill-down.
4. `meta_analysis_run_policy_binding` preserves the access-rule, Others mapping and experience-policy version used by the run.
5. `meta_saved_investigation` stores only context and references, not copied fact data.

Server IP remains controlled/on-demand and is not exploded globally. New CLEAN results retain the source field; a drill-down requires an explicit App, selects at most 200 priority affected users from DWS and parses at most 20,000 matching DWD observations. Network objects are only treated as topology when BRAS/OLT/PON contain real values; otherwise the UI reports limited localization capability.

Run verification uses the latest earlier successful analysis only when access-rule ID/version, configured Others, App-mapping version and experience-policy ID/version are all identical. Missing compatible baselines and missing V2 observations are presented as non-comparable rather than zero change.

The analysis context resolves runs independently from pipeline history. Batch defaults prefer the latest successful/degraded `meta_analysis_run`, while a metadata-only selector exposes Period V2, App ADS and Hourly V2 readiness for every run. V2 foundation queries use partial-result semantics: status, findings, coverage and verification report their own failures, so one optional query cannot hide the other valid evidence.

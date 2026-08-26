# SA FBB Experience Workbench｜当前需求基线

## 1. 产品目标

构建一个本地 EXE 应用，用于处理 SA 家宽 TCP / Game 应用体验 CSV 数据，并支撑 Cable-to-Fiber 迁转升套机会识别。

## 2. 必须做

1. CSV 大文件先原样进入 MySQL RAW 表，不在应用内做全量内存清洗。
2. 提供 TCP / Game 两类 CSV 数据导入入口。
3. 提供 import batch、导入任务、质量检查、ETL 任务状态管理。
4. 通过 MySQL SQL 完成 RAW → CLEAN/DWD → DWS/ADS。
5. 提供经营总览、应用体验、网络问题定位、Cable vs FTTH、用户洞察、迁转升套机会六类任务看板。
6. 输出用户级 Lead Type 和推荐套餐字段。
7. 保留后续 CRM、FTTH 覆盖、可触达状态 JOIN 的扩展点。
8. 提供版本化 IPv4 网段配置，用于优先识别 Cable / FTTH；必须支持草稿、重叠校验、抽样预览、发布、批次绑定和规则证据追溯。每次 TCP / Game 导入必须由用户手动选择并确认一个已发布规则版本，不得静默使用最新版本。
9. App 看板必须使用真实 App 粒度，网络热点必须保留 BRAS / OLT / PON，图表必须能回到聚合证据。
10. A0 身份不足和 A2 网络严重异常用户不得表述为可直接营销名单；A1 仍需 CRM、覆盖和可触达资格校验。
11. 历史导入批次必须支持单个或批量删除，并清理该批次的 RAW、DWD、DWS、ADS、分析运行和任务日志；运行中的导入、ETL 或流水线必须禁止删除。
12. 本地 MySQL 连接密码默认值为 `123456`，允许用户覆盖；覆盖值不得写入 localStorage 或执行日志。
13. RAW 导入不得把 0 行结果标记为成功；Probe、字段映射和两种导入模式必须使用同一分隔符判断，失败信息必须直接展示 RAW 行数、MySQL warning 或 Quality Gate 失败项。
14. `LOAD DATA LOCAL INFILE` 必须由客户端流式提供当前所选 CSV 的内容；本地文件处理器只允许该文件的规范路径，拒绝服务端请求其他路径，并在单次语句后移除。
15. RAW 长时间导入必须持续提供可观测进度：至少包含客户端已处理字节、文件总大小、百分比、批次 ID，并区分文件传输与 MySQL 解析/提交等待；连续无变化不得静默。
16. 应用启动、恢复或选择历史批次、切换分析页时不得自动执行批次表准备、RAW 全表计数或批量看板查询；分析任务必须由用户明确启动，展示执行范围、进度、当前步骤、失败和重试，并允许停止尚未开始的后续步骤。
17. 流水线日志必须按 sequence 可靠增量读取并去重，不得因重叠轮询重复展示；RAW 导入以外的长步骤也必须定期写入存活心跳和步骤耗时。前台必须能区分“任务仍运行但无内部百分比”“日志刷新失败”“用户暂停刷新”和“心跳延迟”。日志时间必须以 UTC 持久化并按本地 PC 时区显示，复制日志时必须带出时区标识。
18. 进入系统诊断页不得自动执行 Catalog、批次表计数、模块、映射、质量或 ETL 查询；诊断必须由用户显式启动并展示串行步骤、进度、失败和停止后续检查入口。一次诊断不得为 Registry 和模块状态重复刷新大表计数，高级排错和执行日志必须展开后才挂载。
19. 批次的 RAW 导入成功不得等同于完整分析成功。历史批次必须分别展示 RAW 与流水线状态，选择批次时同步其最新流水线和 `analysis_run_id`，并可恢复该流水线的完整日志；看板空数组或全 0 指标必须显示“无可用 CLEAN/DWS/ADS 证据”，不得显示 `SUCCESS`。
20. 已完成 RAW、Quality Gate 和 CLEAN 的历史批次必须可显式从 DWS/ADS 续跑，不重读 CSV 或重建 RAW。续跑必须覆盖完整 DWS、基础 ADS、App Rank、小时趋势、网络热点、用户画像、Lead Evidence、可选 Final Lead 和 Module Ready，并展示子阶段日志。同批次存在活动 MySQL SQL 时必须拒绝并发接管；批次准备、Registry 和 Module Ready 常规路径不得隐式扫描大表精确 `COUNT(*)`。
21. 六类决策看板必须提供显式的“导出全部图表 PDF”任务。默认导出当前批次、`analysis_run_id` 和筛选条件下的全部非空图表，不包含明细表；任务必须顺序加载聚合数据、显示进度与失败、允许停止尚未开始的步骤，并在报告中记录本地生成时间、时区、DWS/ADS 来源和被跳过的空图。
22. IP 规则集必须把 `Others` 作为规则版本中的显式组成部分：`Others` 指未命中上方任何已配置 IP 网段的剩余 IP 集合，用户必须在规则页明确配置这部分最终归属的制式（当前业务选择 Cable），不得由前端、后端、数据库默认值或 SQL 隐式写死。该选择必须参与草稿、验证、预览、发布、批次绑定和证据追溯；接入制式分析结果不再使用 UNKNOWN 承接正常的未命中人群。未导入独立 Game 文件时，游戏时长与 MOS 必须显示为“未导入/不可用”，不得用数值 0 参与结论。
23. Lead 分层和用户分布图必须来自完整 ADS 总体聚合，不得用分页明细或 Top N 代替总体；多日小时趋势应默认压缩为按活跃用户加权的典型 24 小时曲线，同时保留小时证据明细。

## 2.1 已确认的下一版产品需求

1. 接入制式只把明确命中的 FTTH IP 网段识别为 FTTH；未命中集合由规则页中的 `Others` 显式决定当前规则版本应归为哪种制式。`Others` 是“剩余集合”，不是固定写死的第三种接入制式。
2. 当前业务规则将 `Others` 配置为 Cable，因此重新生成 CLEAN/DWS/ADS 后，正常分析结果中不应再出现接入制式 UNKNOWN。
3. 身份可信度在当前阶段弱化：不得作为体验总览的核心结论，不得阻断 App、时间、网络和接入制式分析；只在用户明细或最终营销资格阶段作为辅助提示。
4. 所有看板、图表、筛选器、状态提示、证据说明和 PDF 报告必须支持中英文切换。数据库字段和内部枚举可保留英文，但展示层不得直接暴露难以理解的内部代码。
5. 每张图表下方必须提供简短解释，至少说明图表回答的问题、指标口径与分母、正确解读方式、样本或数据覆盖限制。
6. 百分比必须能查看分子、分母和样本量；样本不足的对象不得进入严重问题排名。缺失值不得自动转换为 0，缺失拓扑不得包装成可定位的网络热点。
7. “问题 App”不能继续采用“差体验用户占比大于 0”或“周期内任意一次异常”作为唯一判定。必须区分差体验观测占比、曾受影响用户占比、持续差体验用户占比和严重差体验用户占比。
8. 体验指标阈值、持续异常判定阈值、App 最低样本量和问题 App 门槛必须可配置并版本化；每个分析运行必须记录使用的阈值版本，以便复核和重跑。
9. 新分析流程不能只覆盖“总览 → 问题 App → App 详情”一条路径，必须支持从 App、用户、Cable/FTTH、时间、网络位置、家庭/用户侧、Server IP/内容源、容量与使用、数据可信度等维度交叉发现和下钻。
10. 任意分析入口应遵循统一深度：整体状态 → 比较与异常 → 影响对象 → 原因与证据 → 行动与复验。跨页面下钻必须保留当前 App、接入制式、时间、问题指标等上下文。
11. 首页应优先提供可解释的自动发现和调查入口，不要求用户从大量孤立图表中自行拼结论；所有自动发现必须包含影响范围、比较基线、主要驱动、数据可信度和可下钻证据。
12. 新版多维分析需要复用已导入 RAW 批次，重新设计标准 DWD、多维 DWS 和调查型 ADS；不得要求用户重新上传已经成功进入 RAW 的 3.46 GiB CSV，也不得让看板直接扫描 RAW。
13. 新聚合至少应评估 App×时间、App×接入制式、App×问题指标、用户×App持续性、网络节点×App、网络节点×时间、Server IP×App、批次间对比和数据覆盖等关系；最终范围需在分析流程评审后确认。
14. 聚合性能设计必须避免为每张看板重复扫描 RAW/DWD。应复用标准中间层，一轮任务生成相关 DWS/ADS，并继续提供子阶段、心跳、耗时、失败和安全重跑能力。
15. 当前是离线批次分析工作台，不得把批次趋势包装成实时监控、预测告警或闭环自动化；这些能力只有在新增连续数据源和验证机制后才能进入范围。

## 2.2 待评审的分析路径草案（尚未冻结）

候选产品结构为：体验健康总览、多维探索、问题调查、行动中心、数据与规则。候选分析主线包括整体体验、App、用户、Cable/FTTH、时间、网络位置、家庭/用户侧、Server IP/内容源、容量与使用、数据可信度。下一步应先完成外部方案评审，明确用户角色、核心决策、入口优先级、指标口径、跨维交互、聚合模型和分阶段范围，再进入代码实现。

## 3. 明确不做

1. 不把体验差用户直接等同于升套潜客。
2. 不在第一阶段提交客户真实 CSV、数据库导出、安装包或运行日志。
3. 不让前端直接查询 RAW 大表。
4. 不在应用内对几千万行 CSV 做全量清洗。
5. 不在第一阶段强制引入 DuckDB Runtime。

## 4. MVP 验收口径

- 工程可按 `npm install`、`npm run dev`、`npm run build`、`npm run tauri:dev` 路径继续验证。
- 数据库存在 metadata / dim / raw / dwd / dws / ads 分层 DDL。
- 前端存在导入、ETL、看板、Lead 分层的入口骨架。
- Tauri command 已提供后续接入 MySQL / CSV / ETL 的接口名称。
- SQL 模板不使用 `SET @var`，优先使用 CTE 参数块。
- 乱序但可映射的 CSV 表头仍应走 `LOAD DATA LOCAL INFILE`；1 GB+ 性能必须以目标硬件真实基准测试为准。
- `ads_network_hotspot_rank` 在 utf8mb4 下必须使用不超过 InnoDB 3072 字节限制的主键和索引设计。
- 数据画像和质量检查必须读取当前批次物理表，不得误读空的共享 RAW 基表。
- `LOAD DATA LOCAL INFILE` 必须实际传输非空客户端数据；路径白名单必须覆盖“允许当前文件、拒绝其他文件”的测试。
- 大文件 RAW 导入期间至少每 5 秒产生一次后端心跳，30 秒无字节变化时必须给出阶段性诊断提示。
- RAW 质量检查、CLEAN/DWD、DWS/ADS 等长步骤至少每 15 秒产生一次存活心跳；前台不得并发执行多个日志轮询请求，也不得重复展示同一 sequence。
- 选择历史批次后必须恢复该批次最新流水线的状态与日志，并以对应 `analysis_run_id` 查询看板；空结果和全 0 KPI 不得通过看板成功验收。
- 复用批次必须在后端验证 RAW/Quality/CLEAN 前置状态，同批次活动 SQL 存在时拒绝续跑，且完整生成五类结构化 ADS。
- 常规批次准备、Registry 和 Module Ready 不执行批次 RAW/DWD 大表精确全表计数；可用性验证使用有界查询。
- 全部图表导出不得在进入看板或切换页面时自动查询；只有用户启动后才顺序读取 6 个聚合数据集。报告应包含六类看板的全部非空图表、批次/运行/筛选元数据和空图/失败摘要，且不包含证据明细表。
- 仅配置 FTTH 网段且规则版本将 `Others` 显式配置为 Cable 时，CLEAN 结果不得继续产生因未命中规则导致的 UNKNOWN 接入类型；新规则草稿未配置 `Others` 时不得发布，旧聚合存在 UNKNOWN 时必须提示重跑 CLEAN/DWS/ADS。
- Game RAW 不存在时，Game Hours / MOS 不得显示为有效的 0；Lead 分层计数必须与完整 ADS 表按用户聚合结果一致。
## 1.0.50 implemented analysis workflow

- The primary path is `Experience Status → Auto Findings → Investigation Workspace → Saved Investigation`; legacy dashboards remain compatibility entries.
- Analysis Context persists App, access type, date/hour, issue metric/side, user, Server IP, BRAS, baseline and Finding across pages, with chips, remove, clear and back.
- Findings require sufficient samples and versioned policy thresholds. Every displayed rate retains numerator, denominator, sample size and policy version; missing and insufficient values are not zero.
- Investigation reads period and hourly DWS/ADS only. Network-side and user-side labels are evidence judgements, never confirmed root causes.
- Server-IP investigation is App/Finding-scoped and bounded to 200 priority users plus 20,000 DWD observations; it never scans RAW or creates an unbounded exploded IP fact table.
- Verification compares an analysis run only with an earlier successful run carrying identical access, Others, App-mapping and experience-policy versions; otherwise the UI reports that no comparable baseline exists.
- Batch selection must prefer the latest successful/degraded analyzable run and expose an explicit run selector with Period V2, App ADS and Hourly readiness. A manually generated run must not be hidden by an older pipeline run.
- V2 status, findings, coverage and verification load independently; one failed auxiliary dataset must not erase successful results, and a run without V2 rows must be identified as a context mismatch rather than an empty business result.
- Experience Policy and App Experience Profile drafts are editable and published as immutable versions. New runs bind and snapshot the published version.
- Game not imported, topology unavailable, identity limited and Server-IP controlled drill-down are distinct coverage states.
- Experience Findings and Cable-to-Fiber experience opportunities are separate; formal marketing eligibility requires external CRM/coverage/plan/arrears/blacklist/reachability data.

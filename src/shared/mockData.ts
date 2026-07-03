import type { DashboardMetric, ImportBatchSummary, MigrationLeadSummary } from './types';

export const dashboardMetrics: DashboardMetric[] = [
  { label: 'RAW batches', value: '0', hint: 'CSV 原样导入批次' },
  { label: 'Clean users', value: '0', hint: '完成清洗的用户数' },
  { label: 'A1 leads', value: '0', hint: '高需求且可优先营销' },
  { label: 'Quality gates', value: '0', hint: '已执行数据质量检查' },
];

export const importBatches: ImportBatchSummary[] = [
  {
    importBatchId: 'demo-tcp-raw',
    dataType: 'tcp',
    sourceFileName: 'tcp_detail_*.csv',
    totalRows: 0,
    importedRows: 0,
    status: 'pending',
  },
  {
    importBatchId: 'demo-game-raw',
    dataType: 'game',
    sourceFileName: 'game_tcp_detail_*.csv',
    totalRows: 0,
    importedRows: 0,
    status: 'pending',
  },
];

export const migrationLeadSummary: MigrationLeadSummary[] = [
  {
    leadType: 'A1_Cable高需求且有迁转动力_可优先营销',
    userCount: 0,
    avgDemandScore: 0,
    avgMigrationMotiveScore: 0,
    recommendedAction: 'Fiber 500M+/600M/900M + Wi-Fi 6/Mesh',
  },
  {
    leadType: 'A0_高价值但CRM主键待确认',
    userCount: 0,
    avgDemandScore: 0,
    avgMigrationMotiveScore: 0,
    recommendedAction: '先做 CRM / 装机系统映射确认',
  },
  {
    leadType: 'A2_Cable高需求但网络侧异常_先优化或建网',
    userCount: 0,
    avgDemandScore: 0,
    avgMigrationMotiveScore: 0,
    recommendedAction: '进入网络优化或建网评估',
  },
];

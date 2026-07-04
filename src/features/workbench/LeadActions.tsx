import { useMemo, useState } from 'react';
import type { ActionState, FinalLeadUserRow, LeadQueryParams, LeadUserRow, MySqlSettings } from '../../shared/types';
import { ActionButton } from './ActionButton';
import { workbenchApi } from './workbenchApi';

const finalActionOptions = ['ALL', 'IDENTITY_MAPPING_REQUIRED', 'EXCLUDE_BLACKLIST', 'ARREARS_CHECK_FIRST', 'CONTRACT_CHECK_FIRST', 'NETWORK_OPTIMIZATION_FIRST', 'MARKET_FIBER_UPSELL', 'REACHABILITY_FIX_FIRST', 'BUILD_OR_COVERAGE_CHECK', 'NURTURE_POOL', 'FTTH_SPEED_UPSELL', 'OBSERVE', 'UNKNOWN'];

type Props = {
  settings: MySqlSettings;
  analysisRunId: string;
  outputPath: string;
  exportFinalActions: string[];
  actionStates: Record<string, ActionState>;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  setLeads: (value: LeadUserRow[]) => void;
  setFinalLeads: (value: FinalLeadUserRow[]) => void;
};

function numericValue(value: string, fallback: number, min: number, max: number) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.max(min, Math.min(max, parsed));
}

export function LeadActions({ settings, analysisRunId, outputPath, exportFinalActions, actionStates, runAction, setLeads, setFinalLeads }: Props) {
  const [keyword, setKeyword] = useState('');
  const [leadType, setLeadType] = useState('');
  const [finalAction, setFinalAction] = useState('ALL');
  const [page, setPage] = useState('1');
  const [pageSize, setPageSize] = useState('100');
  const queryParams = useMemo<LeadQueryParams>(() => ({ page: numericValue(page, 1, 1, 100000), pageSize: numericValue(pageSize, 100, 1, 1000), keyword, leadType, finalAction }), [finalAction, keyword, leadType, page, pageSize]);
  const exportScope = exportFinalActions.length ? exportFinalActions.join(', ') : 'ALL final actions';
  const canQuery = Boolean(analysisRunId.trim());
  const canExport = canQuery && Boolean(outputPath.trim());

  async function queryFinalLeads() {
    const result = await runAction('final_leads_query_users', () => workbenchApi.finalLeads(settings, analysisRunId, queryParams));
    if (Array.isArray(result)) setFinalLeads(result as FinalLeadUserRow[]);
  }

  return (
    <>
      <div className="table-toolbar lead-query-toolbar">
        <input value={keyword} onChange={(e) => setKeyword(e.target.value)} placeholder="backend keyword: user / crm / action / offer" />
        <input value={leadType} onChange={(e) => setLeadType(e.target.value)} placeholder="SA lead_type exact filter" />
        <select value={finalAction} onChange={(e) => setFinalAction(e.target.value)}>{finalActionOptions.map((item) => <option key={item} value={item}>{item}</option>)}</select>
        <input value={page} onChange={(e) => setPage(e.target.value)} inputMode="numeric" placeholder="page" />
        <input value={pageSize} onChange={(e) => setPageSize(e.target.value)} inputMode="numeric" placeholder="page size" />
      </div>
      <div className="primary-action-row">
        <ActionButton actionKey="final_leads_query_users" actionStates={actionStates} primary label="查询 Final Lead" disabled={!canQuery} onClick={queryFinalLeads} />
      </div>
      <details className="advanced-actions">
        <summary>高级操作：SA Lead / 导出</summary>
        <div className="action-row">
          <ActionButton actionKey="leads_query_users" actionStates={actionStates} label="查询 SA Lead" disabled={!canQuery} onClick={async () => { const result = await runAction('leads_query_users', () => workbenchApi.leads(settings, analysisRunId, queryParams)); if (Array.isArray(result)) setLeads(result as LeadUserRow[]); }} />
          <ActionButton actionKey="export_leads_csv" actionStates={actionStates} label="导出 SA" disabled={!canExport} onClick={() => runAction('export_leads_csv', () => workbenchApi.exportLeads(settings, analysisRunId, outputPath))} />
          <ActionButton actionKey="export_final_leads_csv" actionStates={actionStates} label="导出 Final" disabled={!canExport} onClick={() => runAction('export_final_leads_csv', () => workbenchApi.exportFinal(settings, analysisRunId, outputPath, { finalActions: exportFinalActions }))} />
        </div>
      </details>
      <p className="muted-row">Backend query page={queryParams.page}, page_size={queryParams.pageSize}; Final export scope: {exportScope}</p>
    </>
  );
}

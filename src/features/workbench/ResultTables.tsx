import { useMemo, useState } from 'react';
import type { FinalLeadUserRow, LeadUserRow } from '../../shared/types';
import { PaginationControls } from './PaginationControls';

type ResultTablesProps = {
  leads: LeadUserRow[];
  finalLeads: FinalLeadUserRow[];
};

function uniqueSorted(values: Array<string | undefined>) {
  return Array.from(new Set(values.filter((value): value is string => Boolean(value)))).sort();
}

function pageRows<T>(rows: T[], page: number, pageSize: number) {
  const start = (page - 1) * pageSize;
  return rows.slice(start, start + pageSize);
}

function pageCount(total: number, pageSize: number) {
  return Math.max(1, Math.ceil(total / pageSize));
}

export function ResultTables({ leads, finalLeads }: ResultTablesProps) {
  const [leadQuery, setLeadQuery] = useState('');
  const [leadType, setLeadType] = useState('ALL');
  const [leadPage, setLeadPage] = useState(1);
  const [leadPageSize, setLeadPageSize] = useState(20);
  const [finalQuery, setFinalQuery] = useState('');
  const [finalAction, setFinalAction] = useState('ALL');
  const [finalPage, setFinalPage] = useState(1);
  const [finalPageSize, setFinalPageSize] = useState(20);

  const leadTypes = useMemo(() => uniqueSorted(leads.map((item) => item.lead_type)), [leads]);
  const finalActions = useMemo(() => uniqueSorted(finalLeads.map((item) => item.final_action ?? 'UNKNOWN')), [finalLeads]);

  const filteredLeads = useMemo(() => {
    const query = leadQuery.trim().toLowerCase();
    return leads.filter((item) => {
      const matchesType = leadType === 'ALL' || item.lead_type === leadType;
      const matchesQuery = !query || [item.user_key, item.user_type, item.lead_type, item.recommended_offer].some((value) => (value ?? '').toLowerCase().includes(query));
      return matchesType && matchesQuery;
    });
  }, [leads, leadQuery, leadType]);

  const filteredFinalLeads = useMemo(() => {
    const query = finalQuery.trim().toLowerCase();
    return finalLeads.filter((item) => {
      const action = item.final_action ?? 'UNKNOWN';
      const matchesAction = finalAction === 'ALL' || action === finalAction;
      const matchesQuery = !query || [item.user_key, item.crm_user_id, item.lead_type, item.final_action, item.recommended_offer, item.current_plan_name].some((value) => (value ?? '').toLowerCase().includes(query));
      return matchesAction && matchesQuery;
    });
  }, [finalLeads, finalAction, finalQuery]);

  const finalActionSummary = useMemo(() => {
    const counts = new Map<string, number>();
    for (const item of finalLeads) {
      const action = item.final_action ?? 'UNKNOWN';
      counts.set(action, (counts.get(action) ?? 0) + 1);
    }
    return Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
  }, [finalLeads]);

  const leadPageCount = pageCount(filteredLeads.length, leadPageSize);
  const finalPageCount = pageCount(filteredFinalLeads.length, finalPageSize);
  const visibleLeads = pageRows(filteredLeads, Math.min(leadPage, leadPageCount), leadPageSize);
  const visibleFinalLeads = pageRows(filteredFinalLeads, Math.min(finalPage, finalPageCount), finalPageSize);

  return (
    <section className="two-column">
      <article className="panel">
        <h2>SA Lead</h2>
        <div className="table-toolbar">
          <input value={leadQuery} onChange={(e) => { setLeadQuery(e.target.value); setLeadPage(1); }} placeholder="filter by user / lead / offer" />
          <select value={leadType} onChange={(e) => { setLeadType(e.target.value); setLeadPage(1); }}>
            <option value="ALL">All lead types</option>
            {leadTypes.map((item) => <option key={item} value={item}>{item}</option>)}
          </select>
        </div>
        <PaginationControls page={Math.min(leadPage, leadPageCount)} pageCount={leadPageCount} total={filteredLeads.length} pageSize={leadPageSize} onPageChange={setLeadPage} onPageSizeChange={(size) => { setLeadPageSize(size); setLeadPage(1); }} />
        <div className="table-like">
          <div className="table-row lead-row table-head"><span>Score</span><span>User</span><span>Lead Type</span><span>Offer</span></div>
          {visibleLeads.map((item, index) => (
            <div key={`${item.user_key}-${index}`} className="table-row lead-row">
              <span>{item.demand_score}/{item.migration_motive_score}</span>
              <span>{item.user_key}</span>
              <span>{item.lead_type}</span>
              <span>{item.recommended_offer ?? '-'}</span>
            </div>
          ))}
          {!visibleLeads.length && <div className="table-row muted-row">No SA Lead matched.</div>}
        </div>
      </article>
      <article className="panel">
        <h2>Final Lead</h2>
        <div className="summary-pills">
          {finalActionSummary.slice(0, 8).map(([action, count]) => <button key={action} type="button" className="status-pill" onClick={() => { setFinalAction(action); setFinalPage(1); }}>{action}: {count}</button>)}
        </div>
        <div className="table-toolbar">
          <input value={finalQuery} onChange={(e) => { setFinalQuery(e.target.value); setFinalPage(1); }} placeholder="filter by user / crm / action / offer" />
          <select value={finalAction} onChange={(e) => { setFinalAction(e.target.value); setFinalPage(1); }}>
            <option value="ALL">All final actions</option>
            {finalActions.map((item) => <option key={item} value={item}>{item}</option>)}
          </select>
        </div>
        <PaginationControls page={Math.min(finalPage, finalPageCount)} pageCount={finalPageCount} total={filteredFinalLeads.length} pageSize={finalPageSize} onPageChange={setFinalPage} onPageSizeChange={(size) => { setFinalPageSize(size); setFinalPage(1); }} />
        <div className="table-like">
          <div className="table-row lead-row table-head"><span>Action</span><span>User</span><span>CRM</span><span>Network</span></div>
          {visibleFinalLeads.map((item, index) => (
            <div key={`${item.user_key}-${index}`} className="table-row lead-row">
              <span>{item.final_action ?? 'UNKNOWN'}</span>
              <span>{item.user_key}</span>
              <span>{item.crm_user_id ?? '-'}</span>
              <span>{item.ftth_available_flag ?? 'UNKNOWN'}/{item.reachable_flag ?? 'N'}</span>
            </div>
          ))}
          {!visibleFinalLeads.length && <div className="table-row muted-row">No Final Lead matched.</div>}
        </div>
      </article>
    </section>
  );
}

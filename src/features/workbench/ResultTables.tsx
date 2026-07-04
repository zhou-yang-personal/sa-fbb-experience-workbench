import type { FinalLeadUserRow, LeadUserRow } from '../../shared/types';

export function ResultTables({ leads, finalLeads }: { leads: LeadUserRow[]; finalLeads: FinalLeadUserRow[] }) {
  return (
    <section className="two-column">
      <article className="panel">
        <h2>SA Lead</h2>
        <div className="table-like">
          {leads.map((item) => (
            <div key={item.user_key} className="table-row lead-row">
              <span>{item.demand_score}/{item.migration_motive_score}</span>
              <span>{item.user_key}</span>
              <span>{item.lead_type}</span>
            </div>
          ))}
        </div>
      </article>
      <article className="panel">
        <h2>Final Lead</h2>
        <div className="table-like">
          {finalLeads.map((item) => (
            <div key={item.user_key} className="table-row lead-row">
              <span>{item.final_action}</span>
              <span>{item.user_key}</span>
              <span>{item.crm_user_id ?? '-'}</span>
            </div>
          ))}
        </div>
      </article>
    </section>
  );
}

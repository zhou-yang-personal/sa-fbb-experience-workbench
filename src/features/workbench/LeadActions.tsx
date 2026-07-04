import type { FinalLeadUserRow, LeadUserRow, MySqlSettings } from '../../shared/types';
import { workbenchApi } from './workbenchApi';

type Props = {
  settings: MySqlSettings;
  analysisRunId: string;
  outputPath: string;
  runAction: (label: string, action: () => Promise<unknown>) => Promise<unknown>;
  setLeads: (value: LeadUserRow[]) => void;
  setFinalLeads: (value: FinalLeadUserRow[]) => void;
};

export function LeadActions({ settings, analysisRunId, outputPath, runAction, setLeads, setFinalLeads }: Props) {
  return (
    <>
      <button onClick={async () => { const result = await runAction('leads_query_users', () => workbenchApi.leads(settings, analysisRunId)); if (Array.isArray(result)) setLeads(result as LeadUserRow[]); }}>SA Lead</button>
      <button onClick={async () => { const result = await runAction('final_leads_query_users', () => workbenchApi.finalLeads(settings, analysisRunId)); if (Array.isArray(result)) setFinalLeads(result as FinalLeadUserRow[]); }}>Final Lead</button>
      <button onClick={() => runAction('export_leads_csv', () => workbenchApi.exportLeads(settings, analysisRunId, outputPath))}>导出SA</button>
      <button onClick={() => runAction('export_final_leads_csv', () => workbenchApi.exportFinal(settings, analysisRunId, outputPath))}>导出Final</button>
    </>
  );
}

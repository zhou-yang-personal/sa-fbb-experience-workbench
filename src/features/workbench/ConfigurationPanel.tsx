import { AccessRuleCenter } from './AccessRuleCenter';
import { ExperiencePolicyCenter } from './ExperiencePolicyCenter';
import type { WorkbenchController } from './useWorkbenchController';

export function ConfigurationPanel({ c }: { c: WorkbenchController }) {
  return <section className="workbench-section-stack"><AccessRuleCenter c={c} /><ExperiencePolicyCenter c={c} /></section>;
}

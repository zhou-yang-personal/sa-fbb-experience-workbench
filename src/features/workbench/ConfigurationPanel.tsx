import { AccessRuleCenter } from './AccessRuleCenter';
import type { WorkbenchController } from './useWorkbenchController';

export function ConfigurationPanel({ c }: { c: WorkbenchController }) {
  return <AccessRuleCenter c={c} />;
}

import { AccessRuleCenter } from './AccessRuleCenter';
import { ExperiencePolicyCenter } from './ExperiencePolicyCenter';
import { DecisionRuleCenter } from './DecisionRuleCenter';
import type { WorkbenchController } from './useWorkbenchController';

export function ConfigurationPanel({ c }: { c: WorkbenchController }) {
  return <section className="workbench-section-stack"><AccessRuleCenter c={c} /><DecisionRuleCenter c={c} /><details><summary>专家配置：观测级 App 体验画像</summary><ExperiencePolicyCenter c={c} /></details></section>;
}

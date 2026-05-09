import { callCommand } from "./tauri";

export type SkillTemplateKind = "basic" | "playbook";

export type SkillTemplate = {
  kind: SkillTemplateKind;
  name: string;
  description: string;
  tags: string[];
  skillMd: string;
};

export function listSkillTemplates(): Promise<SkillTemplate[]> {
  return callCommand<SkillTemplate[]>("list_skill_templates");
}

export function getSkillTemplate(kind: SkillTemplateKind): Promise<SkillTemplate> {
  return callCommand<SkillTemplate>("get_skill_template", { kind });
}

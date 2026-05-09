use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillTemplateKind {
    Basic,
    Playbook,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillTemplate {
    pub kind: SkillTemplateKind,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub skill_md: String,
}

pub fn list_skill_templates() -> Vec<SkillTemplate> {
    vec![
        get_skill_template(SkillTemplateKind::Basic),
        get_skill_template(SkillTemplateKind::Playbook),
    ]
}

pub fn get_skill_template(kind: SkillTemplateKind) -> SkillTemplate {
    match kind {
        SkillTemplateKind::Basic => SkillTemplate {
            kind,
            name: "Basic Skill".to_string(),
            description: "Focused instruction package for a reusable AI workflow.".to_string(),
            tags: vec!["skill".to_string()],
            skill_md: BASIC_SKILL_TEMPLATE.to_string(),
        },
        SkillTemplateKind::Playbook => SkillTemplate {
            kind,
            name: "Playbook Skill".to_string(),
            description: "Structured steps for a repeatable workflow.".to_string(),
            tags: vec!["playbook".to_string()],
            skill_md: PLAYBOOK_SKILL_TEMPLATE.to_string(),
        },
    }
}

const BASIC_SKILL_TEMPLATE: &str = "# Basic Skill

Use this skill when a reusable AI workflow needs a focused instruction package.

## Instructions

Describe the trigger, expected inputs, steps, and completion criteria.
";

const PLAYBOOK_SKILL_TEMPLATE: &str = "# Playbook Skill

Use this skill when a repeatable workflow should be represented as structured steps.

## Steps

1. Define the entry condition.
2. Execute each workflow step.
3. Verify the expected outcome.
";

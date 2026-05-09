use serde::{Deserialize, Serialize};

use crate::asset::id::is_safe_asset_id;
use crate::asset::model::{PlaybookAsset, PromptAsset, RuleAsset, RuleKind, SkillAsset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationStatus {
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub status: ValidationStatus,
    pub messages: Vec<String>,
}

impl ValidationReport {
    pub fn valid() -> Self {
        Self {
            status: ValidationStatus::Valid,
            messages: Vec::new(),
        }
    }

    pub fn push_error(&mut self, message: impl Into<String>) {
        self.status = ValidationStatus::Invalid;
        self.messages.push(message.into());
    }
}

pub fn validate_prompt(prompt: &PromptAsset) -> ValidationReport {
    let mut report = ValidationReport::valid();

    if !is_safe_asset_id(&prompt.id) {
        report.push_error("id must use only a-z, 0-9, hyphen, or underscore");
    }

    if prompt.name.trim().is_empty() {
        report.push_error("name is required");
    }

    if prompt.body.trim().is_empty() {
        report.push_error("body is required");
    }

    report
}

pub fn validate_skill(skill: &SkillAsset) -> ValidationReport {
    let mut report = ValidationReport::valid();

    if !is_safe_asset_id(&skill.id) {
        report.push_error("id must use only a-z, 0-9, hyphen, or underscore");
    }

    if skill.skill_md.trim().is_empty() {
        report.push_error("SKILL.md must not be empty");
    }

    report
}

pub fn validate_rule(rule: &RuleAsset) -> ValidationReport {
    let mut report = ValidationReport::valid();

    if !is_safe_asset_id(&rule.id) {
        report.push_error("id must use only a-z, 0-9, hyphen, or underscore");
    }

    if rule.name.trim().is_empty() {
        report.push_error("name is required");
    }

    if rule.body.trim().is_empty() {
        report.push_error("body is required");
    }

    match rule.rule_kind {
        RuleKind::Instruction => {}
        RuleKind::Command => {
            let Some(command_rule) = &rule.command_rule else {
                report.push_error("command rule requires a command spec");
                return report;
            };
            if command_rule.prefix.is_empty()
                || command_rule
                    .prefix
                    .iter()
                    .any(|part| part.trim().is_empty())
            {
                report.push_error("command rule prefix must not be empty");
            }
        }
    }

    report
}

pub fn validate_playbook(playbook: &PlaybookAsset) -> ValidationReport {
    let mut report = ValidationReport::valid();

    if !is_safe_asset_id(&playbook.id) {
        report.push_error("id must use only a-z, 0-9, hyphen, or underscore");
    }

    if playbook.name.trim().is_empty() {
        report.push_error("name is required");
    }

    if playbook.trigger.trim().is_empty() {
        report.push_error("trigger is required");
    }

    if playbook.steps.is_empty() {
        report.push_error("playbook requires at least one step");
    }

    for step in &playbook.steps {
        if step.title.trim().is_empty() {
            report.push_error("playbook step title is required");
        }
        if step.body.trim().is_empty() {
            report.push_error("playbook step body is required");
        }
    }

    report
}

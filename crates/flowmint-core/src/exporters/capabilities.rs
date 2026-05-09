use serde::{Deserialize, Serialize};

use crate::sync::plan::SyncScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportAssetKind {
    Prompt,
    Skill,
    Playbook,
    InstructionRule,
    CommandRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportSupport {
    Supported,
    Unsupported,
    RequiresValidation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetCapability {
    pub asset_kind: ExportAssetKind,
    pub scope: SyncScope,
    pub support: ExportSupport,
    pub output_hint: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetCapabilities {
    pub target_id: String,
    pub display_name: String,
    pub capabilities: Vec<TargetCapability>,
}

pub fn list_target_capabilities() -> Vec<TargetCapabilities> {
    vec![
        TargetCapabilities {
            target_id: "claude-code".to_string(),
            display_name: "Claude Code".to_string(),
            capabilities: scoped_capabilities(&[
                supported(
                    ExportAssetKind::Prompt,
                    ".claude/commands/<id>.md or ~/.claude/commands/<id>.md",
                ),
                supported(
                    ExportAssetKind::Skill,
                    ".claude/skills/<id>/ or ~/.claude/skills/<id>/",
                ),
                supported(
                    ExportAssetKind::Playbook,
                    ".claude/skills/<id>/ or ~/.claude/skills/<id>/",
                ),
                supported(
                    ExportAssetKind::InstructionRule,
                    ".claude/rules/<id>.md or ~/.claude/rules/<id>.md",
                ),
                unsupported(
                    ExportAssetKind::CommandRule,
                    "Use Claude permissions/settings support in a later phase.",
                ),
            ]),
        },
        TargetCapabilities {
            target_id: "codex".to_string(),
            display_name: "Codex".to_string(),
            capabilities: scoped_capabilities(&[
                unsupported(
                    ExportAssetKind::Prompt,
                    "Codex has no confirmed Claude-style prompt command path; use explicit Prompt-as-Skill conversion.",
                ),
                supported(
                    ExportAssetKind::Skill,
                    ".agents/skills/<id>/ or ~/.agents/skills/<id>/",
                ),
                supported(
                    ExportAssetKind::Playbook,
                    ".agents/skills/<id>/ or ~/.agents/skills/<id>/",
                ),
                supported(
                    ExportAssetKind::InstructionRule,
                    "AGENTS.md or ~/.codex/AGENTS.md managed block",
                ),
                supported(
                    ExportAssetKind::CommandRule,
                    ".codex/rules/<id>.rules or ~/.codex/rules/<id>.rules",
                ),
            ]),
        },
        TargetCapabilities {
            target_id: "gemini-cli".to_string(),
            display_name: "Gemini CLI".to_string(),
            capabilities: scoped_capabilities(&[
                supported(
                    ExportAssetKind::Prompt,
                    ".gemini/commands/<id>.toml or ~/.gemini/commands/<id>.toml",
                ),
                requires_validation(
                    ExportAssetKind::Skill,
                    ".gemini/skills/<id>/ or ~/.gemini/skills/<id>/",
                    "Skill discovery must be validated against the installed Gemini CLI before enabling writes.",
                ),
                requires_validation(
                    ExportAssetKind::Playbook,
                    ".gemini/skills/<id>/ or ~/.gemini/skills/<id>/",
                    "Skill discovery must be validated against the installed Gemini CLI before enabling writes.",
                ),
                supported(
                    ExportAssetKind::InstructionRule,
                    "GEMINI.md or ~/.gemini/GEMINI.md managed block/import",
                ),
                unsupported(
                    ExportAssetKind::CommandRule,
                    "Gemini command permission rule export is deferred.",
                ),
            ]),
        },
    ]
}

pub fn capability_for(
    target_id: &str,
    asset_kind: ExportAssetKind,
    scope: SyncScope,
) -> Option<TargetCapability> {
    list_target_capabilities()
        .into_iter()
        .find(|target| target.target_id == target_id)?
        .capabilities
        .into_iter()
        .find(|capability| capability.asset_kind == asset_kind && capability.scope == scope)
}

fn scoped_capabilities(entries: &[CapabilityTemplate]) -> Vec<TargetCapability> {
    let mut capabilities = Vec::new();
    for entry in entries {
        for scope in [SyncScope::Project, SyncScope::GlobalUser] {
            capabilities.push(TargetCapability {
                asset_kind: entry.asset_kind,
                scope,
                support: entry.support,
                output_hint: entry.output_hint.to_string(),
                reason: entry.reason.to_string(),
            });
        }
    }
    capabilities
}

#[derive(Debug, Clone, Copy)]
struct CapabilityTemplate {
    asset_kind: ExportAssetKind,
    support: ExportSupport,
    output_hint: &'static str,
    reason: &'static str,
}

fn supported(asset_kind: ExportAssetKind, output_hint: &'static str) -> CapabilityTemplate {
    CapabilityTemplate {
        asset_kind,
        support: ExportSupport::Supported,
        output_hint,
        reason: "",
    }
}

fn unsupported(asset_kind: ExportAssetKind, reason: &'static str) -> CapabilityTemplate {
    CapabilityTemplate {
        asset_kind,
        support: ExportSupport::Unsupported,
        output_hint: "",
        reason,
    }
}

fn requires_validation(
    asset_kind: ExportAssetKind,
    output_hint: &'static str,
    reason: &'static str,
) -> CapabilityTemplate {
    CapabilityTemplate {
        asset_kind,
        support: ExportSupport::RequiresValidation,
        output_hint,
        reason,
    }
}

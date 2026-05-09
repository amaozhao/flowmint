use flowmint_core::exporters::capabilities::{
    ExportAssetKind, ExportSupport, capability_for, list_target_capabilities,
};
use flowmint_core::sync::plan::SyncScope;

#[test]
fn capability_registry_lists_supported_targets() {
    let targets = list_target_capabilities();
    let ids = targets
        .iter()
        .map(|target| target.target_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["claude-code", "codex", "gemini-cli"]);
}

#[test]
fn capability_registry_blocks_codex_prompt_commands_by_default() {
    let capability = capability_for("codex", ExportAssetKind::Prompt, SyncScope::Project)
        .expect("codex target should be known");

    assert_eq!(capability.support, ExportSupport::Unsupported);
    assert!(capability.reason.contains("Prompt-as-Skill"));
}

#[test]
fn capability_registry_separates_instruction_and_command_rules() {
    let codex_instruction = capability_for(
        "codex",
        ExportAssetKind::InstructionRule,
        SyncScope::Project,
    )
    .expect("codex target should be known");
    let codex_command = capability_for("codex", ExportAssetKind::CommandRule, SyncScope::Project)
        .expect("codex target should be known");
    let claude_command = capability_for(
        "claude-code",
        ExportAssetKind::CommandRule,
        SyncScope::Project,
    )
    .expect("claude target should be known");

    assert_eq!(codex_instruction.support, ExportSupport::Supported);
    assert_eq!(codex_command.support, ExportSupport::Supported);
    assert_eq!(claude_command.support, ExportSupport::Unsupported);
    assert!(codex_command.output_hint.contains(".codex/rules"));
}

#[test]
fn capability_registry_marks_gemini_skills_as_requires_validation() {
    let skill = capability_for("gemini-cli", ExportAssetKind::Skill, SyncScope::Project)
        .expect("gemini target should be known");
    let command_rule = capability_for(
        "gemini-cli",
        ExportAssetKind::CommandRule,
        SyncScope::Project,
    )
    .expect("gemini target should be known");

    assert_eq!(skill.support, ExportSupport::RequiresValidation);
    assert!(skill.reason.contains("installed Gemini CLI"));
    assert_eq!(command_rule.support, ExportSupport::Unsupported);
}

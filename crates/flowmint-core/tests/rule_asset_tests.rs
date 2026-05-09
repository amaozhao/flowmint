use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{
    AssetDetail, AssetFilter, AssetType, CommandRule, CommandRuleDecision, CreateAssetInput,
    RuleAsset, RuleKind,
};
use flowmint_core::asset::rule::{create_rule, get_rule, list_rules};
use flowmint_core::asset::store::{create_asset, get_asset, list_assets};

fn test_home(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-rule-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn instruction_rule(id: &str) -> RuleAsset {
    RuleAsset {
        id: id.to_string(),
        name: "TypeScript Style".to_string(),
        description: Some("Project style rule".to_string()),
        tags: vec!["typescript".to_string()],
        rule_kind: RuleKind::Instruction,
        path_globs: vec!["src/**/*.ts".to_string()],
        command_rule: None,
        target_compatibility: vec!["claude-code".to_string(), "codex".to_string()],
        body: "Prefer explicit return types for exported functions.".to_string(),
    }
}

fn command_rule(id: &str) -> RuleAsset {
    RuleAsset {
        id: id.to_string(),
        name: "Safe Git Status".to_string(),
        description: None,
        tags: vec!["codex".to_string()],
        rule_kind: RuleKind::Command,
        path_globs: Vec::new(),
        command_rule: Some(CommandRule {
            prefix: vec!["git".to_string(), "status".to_string()],
            decision: CommandRuleDecision::Allow,
        }),
        target_compatibility: vec!["codex".to_string()],
        body: "Allow read-only repository status checks.".to_string(),
    }
}

#[test]
fn create_list_and_get_instruction_rule() {
    let home = test_home("instruction");

    create_rule(&home, instruction_rule("typescript-style")).expect("rule should create");
    let rules = list_rules(&home, Some(RuleKind::Instruction)).expect("rules should list");
    let loaded = get_rule(&home, "typescript-style").expect("rule should load");

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].asset_type, AssetType::InstructionRule);
    assert_eq!(loaded.path_globs, vec!["src/**/*.ts"]);
    assert_eq!(
        loaded.body,
        "Prefer explicit return types for exported functions."
    );

    cleanup(&home);
}

#[test]
fn create_command_rule_requires_command_spec() {
    let home = test_home("command-validation");
    let invalid = RuleAsset {
        command_rule: None,
        ..command_rule("safe-git-status")
    };

    let result = create_rule(&home, invalid);

    assert!(result.is_err());
    assert!(!home.join("rules/safe-git-status.md").exists());

    cleanup(&home);
}

#[test]
fn asset_store_handles_rule_assets() {
    let home = test_home("asset-store");

    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::InstructionRule {
                asset: instruction_rule("typescript-style"),
            },
        },
    )
    .expect("instruction rule should create through asset store");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::CommandRule {
                asset: command_rule("safe-git-status"),
            },
        },
    )
    .expect("command rule should create through asset store");

    let instruction = get_asset(&home, "instruction-rule:typescript-style")
        .expect("instruction rule should load");
    let command =
        get_asset(&home, "command-rule:safe-git-status").expect("command rule should load");
    let command_summaries = list_assets(
        &home,
        AssetFilter {
            asset_type: Some(AssetType::CommandRule),
            query: None,
        },
    )
    .expect("command rules should list");

    assert!(matches!(instruction, AssetDetail::InstructionRule { .. }));
    assert!(matches!(command, AssetDetail::CommandRule { .. }));
    assert_eq!(command_summaries.len(), 1);
    assert_eq!(command_summaries[0].id, "safe-git-status");

    cleanup(&home);
}

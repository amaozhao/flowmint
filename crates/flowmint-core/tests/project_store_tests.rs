use std::fs;
use std::path::PathBuf;

use flowmint_core::asset::model::{
    AssetDetail, CommandRule, CommandRuleDecision, CreateAssetInput, PlaybookAsset,
    PlaybookInvocation, PlaybookSideEffectLevel, PlaybookStep, PromptAsset, RuleAsset, RuleKind,
};
use flowmint_core::asset::store::create_asset;
use flowmint_core::project::manifest::attach_skill;
use flowmint_core::project::model::{AttachedAssetState, ProjectAssetType};
use flowmint_core::project::store::{
    add_project, attach_asset, attach_asset_to_profile, get_project, list_projects,
};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::plan::SyncScope;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-project-store-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn prompt(id: &str) -> PromptAsset {
    PromptAsset {
        id: id.to_string(),
        name: "Daily Plan".to_string(),
        description: Some("Turn notes into a plan".to_string()),
        tags: vec!["planning".to_string()],
        variables: Vec::new(),
        body: "Write a plan.".to_string(),
    }
}

fn playbook(id: &str) -> PlaybookAsset {
    PlaybookAsset {
        id: id.to_string(),
        name: "Release Check".to_string(),
        description: Some("Repeatable release review".to_string()),
        tags: vec!["release".to_string()],
        trigger: "Before publishing".to_string(),
        inputs: Vec::new(),
        steps: vec![PlaybookStep {
            title: "Run checks".to_string(),
            body: "Run the verification suite.".to_string(),
        }],
        verification: "All checks pass.".to_string(),
        failure_handling: "Stop on failure.".to_string(),
        side_effect_level: PlaybookSideEffectLevel::RunsCommands,
        recommended_invocation: PlaybookInvocation::Manual,
        target_compatibility: vec!["claude-code".to_string(), "codex".to_string()],
    }
}

fn instruction_rule(id: &str) -> RuleAsset {
    RuleAsset {
        id: id.to_string(),
        name: "TypeScript Style".to_string(),
        description: Some("Project style".to_string()),
        tags: vec!["typescript".to_string()],
        rule_kind: RuleKind::Instruction,
        path_globs: vec!["src/**/*.ts".to_string()],
        command_rule: None,
        target_compatibility: vec!["claude-code".to_string(), "codex".to_string()],
        body: "Prefer explicit return types.".to_string(),
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
            decision: CommandRuleDecision::Prompt,
        }),
        target_compatibility: vec!["codex".to_string()],
        body: "Prompt before running git status.".to_string(),
    }
}

#[test]
fn add_project_initializes_manifest_and_lists_recent_project() {
    let home = test_path("home");
    let project_dir = test_path("project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");

    let detail = add_project(&home, &project_dir).expect("project should add");
    assert!(detail.initialized);
    assert_eq!(detail.manifest.export.target, "claude-code");

    let projects = list_projects(&home).expect("projects should list");
    assert_eq!(projects.len(), 1);
    let expected_path = project_dir
        .canonicalize()
        .expect("project path should canonicalize");
    assert_eq!(projects[0].path, expected_path);
    assert!(projects[0].initialized);

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

#[test]
fn project_detail_marks_missing_attached_assets() {
    let home = test_path("missing-home");
    let project_dir = test_path("missing-project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    add_project(&home, &project_dir).expect("project should add");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("daily-plan"),
            },
        },
    )
    .expect("prompt should create");

    attach_asset(&home, &project_dir, "prompt:daily-plan").expect("prompt should attach");
    attach_skill(&project_dir, "missing-skill").expect("missing skill should be represented");

    let detail = get_project(&home, &project_dir).expect("project should load");
    assert_eq!(detail.attached_assets.len(), 2);

    let prompt = detail
        .attached_assets
        .iter()
        .find(|asset| asset.asset_ref == "prompt:daily-plan")
        .expect("prompt attachment should be present");
    assert_eq!(prompt.asset_type, ProjectAssetType::Prompt);
    assert_eq!(prompt.state, AttachedAssetState::Available);
    assert_eq!(
        prompt.summary.as_ref().map(|summary| summary.name.as_str()),
        Some("Daily Plan")
    );

    let skill = detail
        .attached_assets
        .iter()
        .find(|asset| asset.asset_ref == "skill:missing-skill")
        .expect("skill attachment should be present");
    assert_eq!(skill.asset_type, ProjectAssetType::Skill);
    assert_eq!(skill.state, AttachedAssetState::Missing);
    assert!(skill.summary.is_none());

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

#[test]
fn project_can_attach_v2_asset_types() {
    let home = test_path("v2-home");
    let project_dir = test_path("v2-project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    add_project(&home, &project_dir).expect("project should add");

    for asset in [
        AssetDetail::Playbook {
            asset: playbook("release-check"),
        },
        AssetDetail::InstructionRule {
            asset: instruction_rule("typescript-style"),
        },
        AssetDetail::CommandRule {
            asset: command_rule("safe-git-status"),
        },
    ] {
        create_asset(&home, CreateAssetInput { asset }).expect("asset should create");
    }

    attach_asset(&home, &project_dir, "playbook:release-check").expect("playbook should attach");
    attach_asset(&home, &project_dir, "instruction-rule:typescript-style")
        .expect("instruction rule should attach");
    let detail = attach_asset(&home, &project_dir, "command-rule:safe-git-status")
        .expect("command rule should attach");

    assert_eq!(detail.manifest.exports[0].playbooks, vec!["release-check"]);
    assert_eq!(
        detail.manifest.exports[0].instruction_rules,
        vec!["typescript-style"]
    );
    assert_eq!(
        detail.manifest.exports[0].command_rules,
        vec!["safe-git-status"]
    );
    assert_eq!(detail.attached_assets.len(), 3);
    assert!(
        detail
            .attached_assets
            .iter()
            .any(|asset| asset.asset_type == ProjectAssetType::Playbook
                && asset.asset_ref == "playbook:release-check"
                && asset.state == AttachedAssetState::Available)
    );
    assert!(detail.attached_assets.iter().any(|asset| asset.asset_type
        == ProjectAssetType::InstructionRule
        && asset.asset_ref == "instruction-rule:typescript-style"
        && asset.state == AttachedAssetState::Available));
    assert!(
        detail
            .attached_assets
            .iter()
            .any(|asset| asset.asset_type == ProjectAssetType::CommandRule
                && asset.asset_ref == "command-rule:safe-git-status"
                && asset.state == AttachedAssetState::Available)
    );

    let manifest =
        fs::read_to_string(project_dir.join(".flowmint.toml")).expect("manifest should read");
    assert!(manifest.contains("[[exports]]"));
    assert!(manifest.contains("playbooks = [\"release-check\"]"));
    assert!(manifest.contains("instruction_rules = [\"typescript-style\"]"));
    assert!(manifest.contains("command_rules = [\"safe-git-status\"]"));

    let projects = list_projects(&home).expect("projects should list");
    assert_eq!(projects[0].attached_assets, 3);

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

#[test]
fn project_can_attach_assets_to_target_scope_profile() {
    let home = test_path("profile-home");
    let project_dir = test_path("profile-project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    add_project(&home, &project_dir).expect("project should add");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::CommandRule {
                asset: command_rule("safe-git-status"),
            },
        },
    )
    .expect("command rule should create");

    let detail = attach_asset_to_profile(
        &home,
        &project_dir,
        "codex",
        SyncScope::Project,
        "command-rule:safe-git-status",
    )
    .expect("command rule should attach to codex profile");

    assert_eq!(detail.manifest.exports.len(), 2);
    let codex_profile = detail
        .manifest
        .exports
        .iter()
        .find(|profile| profile.target == "codex" && profile.scope == SyncScope::Project)
        .expect("codex profile should exist");
    assert_eq!(codex_profile.command_rules, vec!["safe-git-status"]);
    assert_eq!(detail.manifest.export.target, "claude-code");

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

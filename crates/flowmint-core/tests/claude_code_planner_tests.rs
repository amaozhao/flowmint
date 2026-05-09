use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{
    AssetDetail, CommandRule, CommandRuleDecision, CreateAssetInput, PlaybookAsset, PlaybookInput,
    PlaybookInvocation, PlaybookSideEffectLevel, PlaybookStep, PromptAsset, RuleAsset, RuleKind,
    SkillAsset, SkillMetadata,
};
use flowmint_core::asset::store::create_asset;
use flowmint_core::exporters::claude_code::preview_claude_code_sync;
use flowmint_core::project::manifest::{attach_prompt, attach_skill, init_project_manifest};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::conflict::SyncConflictKind;
use flowmint_core::sync::plan::SyncOperation;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-claude-planner-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn prompt(id: &str) -> PromptAsset {
    PromptAsset {
        id: id.to_string(),
        name: "Daily Plan".to_string(),
        description: None,
        tags: Vec::new(),
        variables: Vec::new(),
        body: "Write a daily plan.".to_string(),
    }
}

fn skill(id: &str) -> SkillAsset {
    SkillAsset {
        id: id.to_string(),
        name: "Research Helper".to_string(),
        description: Some("Research with primary sources".to_string()),
        tags: vec!["research".to_string()],
        root_dir: PathBuf::new(),
        skill_md: "# Research Helper\n\nUse primary sources.".to_string(),
        metadata: Some(SkillMetadata {
            raw_toml: "name = \"Research Helper\"\n".to_string(),
        }),
        files: Vec::new(),
    }
}

fn playbook(id: &str) -> PlaybookAsset {
    PlaybookAsset {
        id: id.to_string(),
        name: "Release Check".to_string(),
        description: Some("Repeatable release check".to_string()),
        tags: vec!["release".to_string()],
        trigger: "Before release".to_string(),
        inputs: vec![PlaybookInput {
            name: "version".to_string(),
            description: None,
            required: true,
        }],
        steps: vec![PlaybookStep {
            title: "Run checks".to_string(),
            body: "Run the full verification suite.".to_string(),
        }],
        verification: "All checks pass.".to_string(),
        failure_handling: "Stop and report failures.".to_string(),
        side_effect_level: PlaybookSideEffectLevel::RunsCommands,
        recommended_invocation: PlaybookInvocation::Manual,
        target_compatibility: vec!["claude-code".to_string()],
    }
}

fn instruction_rule(id: &str) -> RuleAsset {
    RuleAsset {
        id: id.to_string(),
        name: "TypeScript Style".to_string(),
        description: Some("Project rule".to_string()),
        tags: vec!["typescript".to_string()],
        rule_kind: RuleKind::Instruction,
        path_globs: vec!["src/**/*.ts".to_string()],
        command_rule: None,
        target_compatibility: vec!["claude-code".to_string()],
        body: "Prefer explicit return types.".to_string(),
    }
}

fn command_rule(id: &str) -> RuleAsset {
    RuleAsset {
        id: id.to_string(),
        name: "Unsafe Command".to_string(),
        description: None,
        tags: vec!["claude-code".to_string()],
        rule_kind: RuleKind::Command,
        path_globs: Vec::new(),
        command_rule: Some(CommandRule {
            prefix: vec!["rm".to_string(), "-rf".to_string()],
            decision: CommandRuleDecision::Forbid,
        }),
        target_compatibility: vec!["claude-code".to_string()],
        body: "Forbid destructive removal.".to_string(),
    }
}

#[test]
fn planner_creates_prompt_skill_directory_and_managed_block_without_writing() {
    let home = test_path("home");
    let project_dir = test_path("project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    init_project_manifest(&project_dir).expect("project should initialize");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("daily-plan"),
            },
        },
    )
    .expect("prompt should create");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Skill {
                asset: skill("research-helper"),
            },
        },
    )
    .expect("skill should create");
    fs::create_dir_all(home.join("skills/research-helper/examples"))
        .expect("examples should create");
    fs::write(
        home.join("skills/research-helper/examples/example.md"),
        "Example content",
    )
    .expect("example should write");
    fs::create_dir_all(home.join("skills/research-helper/resources"))
        .expect("resources should create");
    fs::write(
        home.join("skills/research-helper/resources/data.txt"),
        "data",
    )
    .expect("resource should write");
    attach_prompt(&project_dir, "daily-plan").expect("prompt should attach");
    attach_skill(&project_dir, "research-helper").expect("skill should attach");

    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");

    assert!(plan.conflicts.is_empty());
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".claude/commands/daily-plan.md"),
    );
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".claude/skills/research-helper/SKILL.md"),
    );
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".claude/skills/research-helper/metadata.toml"),
    );
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".claude/skills/research-helper/examples/example.md"),
    );
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".claude/skills/research-helper/resources/data.txt"),
    );
    assert_has_create_file(&plan.operations, &project_dir.join("CLAUDE.md"));
    assert!(!project_dir.join(".claude/commands/daily-plan.md").exists());
    assert!(!project_dir.join("CLAUDE.md").exists());

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

#[test]
fn planner_reports_existing_unmanaged_generated_file_as_conflict() {
    let home = test_path("unmanaged-home");
    let project_dir = test_path("unmanaged-project");
    fs::create_dir_all(project_dir.join(".claude/commands")).expect("commands dir should create");
    fs::write(
        project_dir.join(".claude/commands/daily-plan.md"),
        "user-owned content",
    )
    .expect("target should write");
    init_library_at(&home).expect("library should initialize");
    init_project_manifest(&project_dir).expect("project should initialize");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("daily-plan"),
            },
        },
    )
    .expect("prompt should create");
    attach_prompt(&project_dir, "daily-plan").expect("prompt should attach");

    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");

    assert_eq!(plan.conflicts[0].kind, SyncConflictKind::UnmanagedTarget);
    assert!(!has_operation_for(
        &plan.operations,
        &project_dir.join(".claude/commands/daily-plan.md")
    ));

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

#[test]
fn planner_reports_broken_claude_managed_marker_as_conflict() {
    let home = test_path("broken-home");
    let project_dir = test_path("broken-project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    fs::write(project_dir.join("CLAUDE.md"), "<!-- FLOWMINT:BEGIN -->\n")
        .expect("claude file should write");
    init_library_at(&home).expect("library should initialize");
    init_project_manifest(&project_dir).expect("project should initialize");

    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");

    assert_eq!(
        plan.conflicts[0].kind,
        SyncConflictKind::IncompleteManagedBlock
    );
    assert!(!has_operation_for(
        &plan.operations,
        &project_dir.join("CLAUDE.md")
    ));

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

#[test]
fn planner_blocks_unsafe_asset_ids_before_generating_paths() {
    let home = test_path("unsafe-home");
    let project_dir = test_path("unsafe-project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    fs::write(
        project_dir.join(".flowmint.toml"),
        "[project]\nname = \"unsafe\"\n\n[export]\ntarget = \"claude-code\"\n\n[attach]\nprompts = [\"../escape\"]\nskills = []\n",
    )
    .expect("manifest should write");

    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");

    assert_eq!(plan.conflicts[0].kind, SyncConflictKind::UnsafeAssetId);
    assert!(!plan.operations.iter().any(|operation| {
        operation_target(operation)
            .to_string_lossy()
            .contains("escape")
    }));

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

#[test]
fn planner_creates_playbook_skill_and_instruction_rule_from_v2_manifest() {
    let home = test_path("playbook-rule-home");
    let project_dir = test_path("playbook-rule-project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Playbook {
                asset: playbook("release-check"),
            },
        },
    )
    .expect("playbook should create");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::InstructionRule {
                asset: instruction_rule("typescript-style"),
            },
        },
    )
    .expect("rule should create");
    fs::write(
        project_dir.join(".flowmint.toml"),
        r#"[project]
name = "project"

[[exports]]
target = "claude-code"
scope = "project"
prompts = []
skills = []
playbooks = ["release-check"]
instruction_rules = ["typescript-style"]
command_rules = []
"#,
    )
    .expect("manifest should write");

    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");

    assert!(plan.conflicts.is_empty());
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".claude/skills/release-check/SKILL.md"),
    );
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".claude/rules/typescript-style.md"),
    );
    assert_has_create_file(&plan.operations, &project_dir.join("CLAUDE.md"));

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

#[test]
fn planner_reports_claude_command_rules_as_unsupported_mapping() {
    let home = test_path("command-rule-home");
    let project_dir = test_path("command-rule-project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::CommandRule {
                asset: command_rule("dangerous-rm"),
            },
        },
    )
    .expect("command rule should create");
    fs::write(
        project_dir.join(".flowmint.toml"),
        r#"[project]
name = "project"

[[exports]]
target = "claude-code"
scope = "project"
prompts = []
skills = []
playbooks = []
instruction_rules = []
command_rules = ["dangerous-rm"]
"#,
    )
    .expect("manifest should write");

    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");

    assert_eq!(plan.conflicts[0].kind, SyncConflictKind::UnsupportedMapping);

    fs::remove_dir_all(home).expect("home should remove");
    fs::remove_dir_all(project_dir).expect("project should remove");
}

fn assert_has_create_file(operations: &[SyncOperation], target_path: &Path) {
    assert!(
        operations.iter().any(|operation| matches!(
            operation,
            SyncOperation::CreateFile { target_path: path, .. } if path == target_path
        )),
        "expected create-file operation for {}",
        target_path.display()
    );
}

fn has_operation_for(operations: &[SyncOperation], target_path: &Path) -> bool {
    operations
        .iter()
        .any(|operation| operation_target(operation) == target_path)
}

fn operation_target(operation: &SyncOperation) -> &Path {
    match operation {
        SyncOperation::CreateFile { target_path, .. }
        | SyncOperation::UpdateFile { target_path, .. }
        | SyncOperation::CreateDir { target_path }
        | SyncOperation::DeleteGeneratedFile { target_path, .. }
        | SyncOperation::Noop { target_path, .. } => target_path,
    }
}

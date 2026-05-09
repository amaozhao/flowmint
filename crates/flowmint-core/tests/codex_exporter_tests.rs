use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{
    AssetDetail, CommandRule, CommandRuleDecision, CreateAssetInput, PlaybookAsset,
    PlaybookInvocation, PlaybookSideEffectLevel, PlaybookStep, PromptAsset, RuleAsset, RuleKind,
    SkillAsset,
};
use flowmint_core::asset::store::create_asset;
use flowmint_core::exporters::claude_code::preview_claude_code_sync;
use flowmint_core::exporters::target::preview_target_sync;
use flowmint_core::project::global_profiles::{GlobalSyncProfiles, write_global_sync_profiles};
use flowmint_core::project::manifest::{
    ProjectExportProfile, attach_prompt, init_project_manifest,
};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::apply::apply_sync;
use flowmint_core::sync::conflict::SyncConflictKind;
use flowmint_core::sync::plan::{SyncOperation, SyncPlan, SyncScope};
use flowmint_core::sync::plan_cache::PlanCache;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-codex-{name}-{}", std::process::id()));
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
        description: None,
        tags: Vec::new(),
        root_dir: PathBuf::new(),
        skill_md: "---\nname: research-helper\ndescription: Research with primary sources.\n---\n\nUse primary sources.\n".to_string(),
        metadata: None,
        files: Vec::new(),
    }
}

fn playbook(id: &str) -> PlaybookAsset {
    PlaybookAsset {
        id: id.to_string(),
        name: "Release Check".to_string(),
        description: None,
        tags: vec!["release".to_string()],
        trigger: "Before release".to_string(),
        inputs: Vec::new(),
        steps: vec![PlaybookStep {
            title: "Run checks".to_string(),
            body: "Run the full verification suite.".to_string(),
        }],
        verification: "All checks pass.".to_string(),
        failure_handling: "Stop and report failures.".to_string(),
        side_effect_level: PlaybookSideEffectLevel::RunsCommands,
        recommended_invocation: PlaybookInvocation::Manual,
        target_compatibility: vec!["codex".to_string()],
    }
}

fn instruction_rule(id: &str) -> RuleAsset {
    RuleAsset {
        id: id.to_string(),
        name: "TypeScript Style".to_string(),
        description: None,
        tags: vec!["typescript".to_string()],
        rule_kind: RuleKind::Instruction,
        path_globs: vec!["src/**/*.ts".to_string()],
        command_rule: None,
        target_compatibility: vec!["codex".to_string()],
        body: "Prefer explicit return types.".to_string(),
    }
}

fn command_rule(id: &str) -> RuleAsset {
    RuleAsset {
        id: id.to_string(),
        name: "Safe PR View".to_string(),
        description: None,
        tags: vec!["codex".to_string()],
        rule_kind: RuleKind::Command,
        path_globs: Vec::new(),
        command_rule: Some(CommandRule {
            prefix: vec!["gh".to_string(), "pr".to_string(), "view".to_string()],
            decision: CommandRuleDecision::Prompt,
        }),
        target_compatibility: vec!["codex".to_string()],
        body: "Viewing PRs is allowed with approval.".to_string(),
    }
}

#[test]
fn codex_project_apply_writes_supported_assets_to_codex_skill_path() {
    let home = test_path("project-home");
    let project_dir = test_path("project");
    setup_library_project(&home, &project_dir);
    create_supported_assets(&home);
    write_codex_manifest(&project_dir, SyncScope::Project);

    let plan = preview_target_sync(&home, &project_dir, "codex", SyncScope::Project)
        .expect("codex preview should plan");
    assert!(plan.conflicts.is_empty());
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".codex/skills/research-helper/SKILL.md"),
    );
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".codex/skills/release-check/SKILL.md"),
    );
    assert_has_create_file(&plan.operations, &project_dir.join("AGENTS.md"));
    assert_has_create_file(
        &plan.operations,
        &project_dir.join(".codex/rules/safe-gh-pr-view.rules"),
    );
    assert!(!plan.operations.iter().any(|operation| {
        operation_target(operation)
            .to_string_lossy()
            .contains(".agents/skills")
    }));

    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);
    apply_sync(&home, &mut cache, &plan_id).expect("codex apply should succeed");

    let agents_md = fs::read_to_string(project_dir.join("AGENTS.md")).expect("AGENTS.md exists");
    let command_rule = fs::read_to_string(project_dir.join(".codex/rules/safe-gh-pr-view.rules"))
        .expect("command rule exists");
    assert!(
        project_dir
            .join(".codex/skills/research-helper/SKILL.md")
            .is_file()
    );
    assert!(
        project_dir
            .join(".codex/skills/release-check/SKILL.md")
            .is_file()
    );
    assert!(agents_md.contains("<!-- FLOWMINT:CODEX:BEGIN -->"));
    assert!(agents_md.contains("src/**/*.ts"));
    assert!(agents_md.contains("Prefer explicit return types."));
    assert!(command_rule.contains("prefix_rule("));
    assert!(command_rule.contains("pattern = [\"gh\", \"pr\", \"view\"]"));
    assert!(command_rule.contains("decision = \"prompt\""));
    assert!(command_rule.contains("match = ["));
    assert!(project_dir.join(".flowmint.lock").is_file());

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn codex_prompt_export_is_blocked_as_unsupported_mapping() {
    let home = test_path("prompt-home");
    let project_dir = test_path("prompt-project");
    setup_library_project(&home, &project_dir);
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("daily-plan"),
            },
        },
    )
    .expect("prompt should create");
    fs::write(
        project_dir.join(".flowmint.toml"),
        r#"[project]
name = "project"

[[exports]]
target = "codex"
scope = "project"
prompts = ["daily-plan"]
skills = []
playbooks = []
instruction_rules = []
command_rules = []
"#,
    )
    .expect("manifest should write");

    let plan = preview_target_sync(&home, &project_dir, "codex", SyncScope::Project)
        .expect("codex preview should plan unsupported prompt");

    assert_eq!(plan.conflicts[0].kind, SyncConflictKind::UnsupportedMapping);
    assert!(plan.operations.is_empty());

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn codex_global_apply_writes_user_level_outputs_and_global_lockfile() {
    let user_home = test_path("global-user");
    let home = user_home.join(".flowmint");
    let project_dir = user_home.join("project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    create_supported_assets(&home);
    write_global_sync_profiles(
        &home,
        &GlobalSyncProfiles {
            profiles: vec![ProjectExportProfile {
                target: "codex".to_string(),
                scope: SyncScope::GlobalUser,
                prompts: Vec::new(),
                skills: vec!["research-helper".to_string()],
                playbooks: vec!["release-check".to_string()],
                instruction_rules: vec!["typescript-style".to_string()],
                command_rules: vec!["safe-gh-pr-view".to_string()],
            }],
        },
    )
    .expect("global profile should write");

    let plan = preview_target_sync(&home, &project_dir, "codex", SyncScope::GlobalUser)
        .expect("global codex preview should plan");
    let plan_id = plan.plan_id.clone();
    let confirmed_paths = operation_paths(&plan);
    let mut cache = PlanCache::default();
    cache.insert(plan);
    cache
        .acknowledge_global_plan(&plan_id, &confirmed_paths)
        .expect("global codex plan should acknowledge");
    apply_sync(&home, &mut cache, &plan_id).expect("global codex apply should succeed");

    assert!(
        user_home
            .join(".codex/skills/research-helper/SKILL.md")
            .is_file()
    );
    assert!(
        user_home
            .join(".codex/skills/release-check/SKILL.md")
            .is_file()
    );
    assert!(user_home.join(".codex/AGENTS.md").is_file());
    assert!(
        user_home
            .join(".codex/rules/safe-gh-pr-view.rules")
            .is_file()
    );
    let lockfile = fs::read_to_string(home.join("global-sync.lock")).expect("global lock exists");
    assert!(lockfile.contains("target = \"codex\""));
    assert!(lockfile.contains("scope = \"global-user\""));

    cleanup(&user_home);
}

#[test]
fn codex_preview_does_not_delete_other_target_lockfile_records() {
    let home = test_path("mixed-home");
    let project_dir = test_path("mixed-project");
    setup_library_project(&home, &project_dir);
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
    let claude_plan = preview_claude_code_sync(&home, &project_dir).expect("claude preview");
    let claude_plan_id = claude_plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(claude_plan);
    apply_sync(&home, &mut cache, &claude_plan_id).expect("claude apply should succeed");

    create_supported_assets(&home);
    write_codex_manifest(&project_dir, SyncScope::Project);
    let codex_plan = preview_target_sync(&home, &project_dir, "codex", SyncScope::Project)
        .expect("codex preview should plan");

    assert!(!codex_plan.operations.iter().any(|operation| matches!(
        operation,
        SyncOperation::DeleteGeneratedFile { target_path, .. }
            if target_path == &project_dir.join(".claude/commands/daily-plan.md")
    )));
    let codex_plan_id = codex_plan.plan_id.clone();
    cache.insert(codex_plan);
    apply_sync(&home, &mut cache, &codex_plan_id).expect("codex apply should succeed");
    assert!(project_dir.join(".claude/commands/daily-plan.md").is_file());
    let lockfile = fs::read_to_string(project_dir.join(".flowmint.lock")).expect("lockfile exists");
    assert!(lockfile.contains("target = \"claude-code\""));
    assert!(lockfile.contains("target = \"codex\""));

    cleanup(&home);
    cleanup(&project_dir);
}

fn setup_library_project(home: &Path, project_dir: &Path) {
    fs::create_dir_all(project_dir).expect("project dir should create");
    init_library_at(home).expect("library should initialize");
    init_project_manifest(project_dir).expect("project should initialize");
}

fn create_supported_assets(home: &Path) {
    create_asset(
        home,
        CreateAssetInput {
            asset: AssetDetail::Skill {
                asset: skill("research-helper"),
            },
        },
    )
    .expect("skill should create");
    create_asset(
        home,
        CreateAssetInput {
            asset: AssetDetail::Playbook {
                asset: playbook("release-check"),
            },
        },
    )
    .expect("playbook should create");
    create_asset(
        home,
        CreateAssetInput {
            asset: AssetDetail::InstructionRule {
                asset: instruction_rule("typescript-style"),
            },
        },
    )
    .expect("instruction rule should create");
    create_asset(
        home,
        CreateAssetInput {
            asset: AssetDetail::CommandRule {
                asset: command_rule("safe-gh-pr-view"),
            },
        },
    )
    .expect("command rule should create");
}

fn write_codex_manifest(project_dir: &Path, scope: SyncScope) {
    let scope = match scope {
        SyncScope::Project => "project",
        SyncScope::GlobalUser => "global-user",
    };
    fs::write(
        project_dir.join(".flowmint.toml"),
        format!(
            r#"[project]
name = "project"

[[exports]]
target = "codex"
scope = "{scope}"
prompts = []
skills = ["research-helper"]
playbooks = ["release-check"]
instruction_rules = ["typescript-style"]
command_rules = ["safe-gh-pr-view"]
"#
        ),
    )
    .expect("manifest should write");
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

fn operation_target(operation: &SyncOperation) -> &Path {
    match operation {
        SyncOperation::CreateFile { target_path, .. }
        | SyncOperation::UpdateFile { target_path, .. }
        | SyncOperation::CreateDir { target_path }
        | SyncOperation::DeleteGeneratedFile { target_path, .. }
        | SyncOperation::Noop { target_path, .. } => target_path,
    }
}

fn operation_paths(plan: &SyncPlan) -> Vec<PathBuf> {
    plan.operations
        .iter()
        .filter_map(|operation| match operation {
            SyncOperation::Noop { .. } => None,
            SyncOperation::CreateFile { target_path, .. }
            | SyncOperation::UpdateFile { target_path, .. }
            | SyncOperation::CreateDir { target_path }
            | SyncOperation::DeleteGeneratedFile { target_path, .. } => Some(target_path.clone()),
        })
        .collect()
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

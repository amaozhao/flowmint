use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{
    AssetDetail, CreateAssetInput, PlaybookAsset, PlaybookInvocation, PlaybookSideEffectLevel,
    PlaybookStep, PromptAsset, RuleAsset, RuleKind, SkillAsset, SkillMetadata,
};
use flowmint_core::asset::store::create_asset;
use flowmint_core::exporters::claude_code::preview_claude_code_sync;
use flowmint_core::exporters::target::preview_target_sync;
use flowmint_core::project::global_profiles::{GlobalSyncProfiles, write_global_sync_profiles};
use flowmint_core::project::manifest::ProjectExportProfile;
use flowmint_core::project::manifest::{attach_prompt, attach_skill, init_project_manifest};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::apply::apply_sync;
use flowmint_core::sync::plan::{SyncOperation, SyncPlan, SyncScope};
use flowmint_core::sync::plan_cache::PlanCache;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-apply-{name}-{}", std::process::id()));
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
        skill_md: "# Research Helper\n".to_string(),
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
        target_compatibility: vec!["claude-code".to_string()],
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
        target_compatibility: vec!["claude-code".to_string()],
        body: "Prefer explicit return types.".to_string(),
    }
}

#[test]
fn apply_sync_writes_prompt_skill_supported_files_managed_block_and_lockfile() {
    let home = test_path("home");
    let project_dir = test_path("project");
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
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);

    let result = apply_sync(&home, &mut cache, &plan_id).expect("apply should succeed");

    assert!(result.written_files >= 6);
    assert_eq!(
        fs::read_to_string(project_dir.join(".claude/commands/daily-plan.md"))
            .expect("prompt command should exist"),
        "Write a daily plan."
    );
    assert_eq!(
        fs::read_to_string(project_dir.join(".claude/skills/research-helper/SKILL.md"))
            .expect("skill markdown should exist"),
        "# Research Helper\n"
    );
    assert!(
        project_dir
            .join(".claude/skills/research-helper/metadata.toml")
            .is_file()
    );
    assert!(
        project_dir
            .join(".claude/skills/research-helper/examples/example.md")
            .is_file()
    );
    assert!(
        project_dir
            .join(".claude/skills/research-helper/resources/data.txt")
            .is_file()
    );
    let claude_md =
        fs::read_to_string(project_dir.join("CLAUDE.md")).expect("CLAUDE.md should exist");
    assert!(claude_md.contains("<!-- FLOWMINT:BEGIN -->"));
    assert!(claude_md.contains("- research-helper"));
    assert!(project_dir.join(".flowmint.lock").is_file());
    let lockfile =
        fs::read_to_string(project_dir.join(".flowmint.lock")).expect("lockfile should exist");
    assert!(lockfile.contains("output_path = \".claude/commands/daily-plan.md\""));
    assert!(lockfile.contains("output_path = \".claude/skills/research-helper/SKILL.md\""));

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn apply_sync_writes_claude_playbook_and_instruction_rule() {
    let home = test_path("playbook-rule-home");
    let project_dir = test_path("playbook-rule-project");
    setup_library_project(&home, &project_dir);
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
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);

    apply_sync(&home, &mut cache, &plan_id).expect("apply should succeed");

    let playbook_skill =
        fs::read_to_string(project_dir.join(".claude/skills/release-check/SKILL.md"))
            .expect("playbook skill should exist");
    let rule = fs::read_to_string(project_dir.join(".claude/rules/typescript-style.md"))
        .expect("rule should exist");
    let claude_md =
        fs::read_to_string(project_dir.join("CLAUDE.md")).expect("CLAUDE.md should exist");

    assert!(playbook_skill.contains("# Release Check"));
    assert!(playbook_skill.contains("Run the full verification suite."));
    assert!(playbook_skill.contains("runs-commands"));
    assert!(rule.contains("paths:"));
    assert!(rule.contains("src/**/*.ts"));
    assert!(rule.contains("Prefer explicit return types."));
    assert!(claude_md.contains("- release-check"));
    assert!(claude_md.contains("- typescript-style"));

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn apply_sync_writes_claude_global_profile_outputs_and_global_lockfile() {
    let user_home = test_path("global-user");
    let home = user_home.join(".flowmint");
    let project_dir = user_home.join("project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
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
    write_global_sync_profiles(
        &home,
        &GlobalSyncProfiles {
            profiles: vec![ProjectExportProfile {
                target: "claude-code".to_string(),
                scope: SyncScope::GlobalUser,
                prompts: vec!["daily-plan".to_string()],
                skills: vec!["research-helper".to_string()],
                playbooks: vec!["release-check".to_string()],
                instruction_rules: vec!["typescript-style".to_string()],
                command_rules: Vec::new(),
            }],
        },
    )
    .expect("global profiles should write");

    let plan = preview_target_sync(&home, &project_dir, "claude-code", SyncScope::GlobalUser)
        .expect("global preview should plan");
    let plan_id = plan.plan_id.clone();
    let confirmed_paths = operation_paths(&plan);
    let mut cache = PlanCache::default();
    cache.insert(plan);
    cache
        .acknowledge_global_plan(&plan_id, &confirmed_paths)
        .expect("global plan should acknowledge");

    apply_sync(&home, &mut cache, &plan_id).expect("global apply should succeed");

    assert!(user_home.join(".claude/commands/daily-plan.md").is_file());
    assert!(
        user_home
            .join(".claude/skills/research-helper/SKILL.md")
            .is_file()
    );
    assert!(
        user_home
            .join(".claude/skills/release-check/SKILL.md")
            .is_file()
    );
    assert!(
        user_home
            .join(".claude/rules/typescript-style.md")
            .is_file()
    );
    assert!(user_home.join(".claude/CLAUDE.md").is_file());
    assert!(!user_home.join("CLAUDE.md").exists());
    let lockfile =
        fs::read_to_string(home.join("global-sync.lock")).expect("global lock should exist");
    assert!(lockfile.contains("output_path = \".claude/commands/daily-plan.md\""));

    cleanup(&user_home);
}

#[test]
fn apply_sync_rechecks_plan_and_refuses_new_conflict() {
    let home = test_path("conflict-home");
    let project_dir = test_path("conflict-project");
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
    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);
    fs::create_dir_all(project_dir.join(".claude/commands")).expect("commands should create");
    fs::write(
        project_dir.join(".claude/commands/daily-plan.md"),
        "user content",
    )
    .expect("conflicting file should write");

    let result = apply_sync(&home, &mut cache, &plan_id);

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(project_dir.join(".claude/commands/daily-plan.md"))
            .expect("conflicting file should remain"),
        "user content"
    );

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn apply_sync_rejects_unknown_plan_id() {
    let home = test_path("unknown-home");
    init_library_at(&home).expect("library should initialize");
    let mut cache = PlanCache::default();

    assert!(apply_sync(&home, &mut cache, "frontend-supplied-plan").is_err());

    cleanup(&home);
}

#[test]
fn apply_sync_appends_managed_block_without_rewriting_user_content() {
    let home = test_path("append-home");
    let project_dir = test_path("append-project");
    setup_library_project(&home, &project_dir);
    fs::write(project_dir.join("CLAUDE.md"), "# User Notes\nKeep this.\n")
        .expect("user CLAUDE.md should write");
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
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);

    apply_sync(&home, &mut cache, &plan_id).expect("apply should succeed");

    let claude_md =
        fs::read_to_string(project_dir.join("CLAUDE.md")).expect("CLAUDE.md should exist");
    assert!(claude_md.contains("# User Notes\nKeep this."));
    assert!(claude_md.contains("<!-- FLOWMINT:BEGIN -->"));
    assert!(claude_md.contains("- daily-plan"));

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn apply_sync_replaces_only_existing_managed_block() {
    let home = test_path("replace-home");
    let project_dir = test_path("replace-project");
    setup_library_project(&home, &project_dir);
    fs::write(
        project_dir.join("CLAUDE.md"),
        "before\n<!-- FLOWMINT:BEGIN -->\nold managed content\n<!-- FLOWMINT:END -->\nafter\n",
    )
    .expect("user CLAUDE.md should write");
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
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);

    apply_sync(&home, &mut cache, &plan_id).expect("apply should succeed");

    let claude_md =
        fs::read_to_string(project_dir.join("CLAUDE.md")).expect("CLAUDE.md should exist");
    assert!(claude_md.starts_with("before\n"));
    assert!(claude_md.contains("\nafter\n"));
    assert!(claude_md.contains("- daily-plan"));
    assert!(!claude_md.contains("old managed content"));

    cleanup(&home);
    cleanup(&project_dir);
}

fn setup_library_project(home: &Path, project_dir: &Path) {
    fs::create_dir_all(project_dir).expect("project dir should create");
    init_library_at(home).expect("library should initialize");
    init_project_manifest(project_dir).expect("project should initialize");
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
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

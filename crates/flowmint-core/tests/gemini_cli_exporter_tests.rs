use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{
    AssetDetail, CommandRule, CommandRuleDecision, CreateAssetInput, PlaybookAsset,
    PlaybookInvocation, PlaybookSideEffectLevel, PlaybookStep, PromptAsset, PromptVariable,
    RuleAsset, RuleKind, SkillAsset,
};
use flowmint_core::asset::store::create_asset;
use flowmint_core::exporters::target::preview_target_sync;
use flowmint_core::project::global_profiles::{GlobalSyncProfiles, write_global_sync_profiles};
use flowmint_core::project::manifest::{ProjectExportProfile, init_project_manifest};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::apply::apply_sync;
use flowmint_core::sync::conflict::SyncConflictKind;
use flowmint_core::sync::plan::{SyncOperation, SyncPlan, SyncScope};
use flowmint_core::sync::plan_cache::PlanCache;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-gemini-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn prompt(id: &str) -> PromptAsset {
    PromptAsset {
        id: id.to_string(),
        name: "Review Code".to_string(),
        description: Some("Review a target file".to_string()),
        tags: Vec::new(),
        variables: vec![PromptVariable {
            name: "target".to_string(),
            description: None,
            default_value: None,
        }],
        body: "Review the requested code for correctness.".to_string(),
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
        target_compatibility: vec!["gemini-cli".to_string()],
        body: "Prefer explicit return types.".to_string(),
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
        metadata: None,
        files: Vec::new(),
    }
}

fn playbook(id: &str) -> PlaybookAsset {
    PlaybookAsset {
        id: id.to_string(),
        name: "Release Check".to_string(),
        description: None,
        tags: Vec::new(),
        trigger: "Before release".to_string(),
        inputs: Vec::new(),
        steps: vec![PlaybookStep {
            title: "Run checks".to_string(),
            body: "Run checks.".to_string(),
        }],
        verification: "All checks pass.".to_string(),
        failure_handling: "Stop.".to_string(),
        side_effect_level: PlaybookSideEffectLevel::RunsCommands,
        recommended_invocation: PlaybookInvocation::Manual,
        target_compatibility: vec!["gemini-cli".to_string()],
    }
}

fn command_rule(id: &str) -> RuleAsset {
    RuleAsset {
        id: id.to_string(),
        name: "Unsafe Command".to_string(),
        description: None,
        tags: Vec::new(),
        rule_kind: RuleKind::Command,
        path_globs: Vec::new(),
        command_rule: Some(CommandRule {
            prefix: vec!["rm".to_string(), "-rf".to_string()],
            decision: CommandRuleDecision::Forbid,
        }),
        target_compatibility: vec!["gemini-cli".to_string()],
        body: "Do not allow destructive removal.".to_string(),
    }
}

#[test]
fn gemini_project_apply_writes_prompt_command_and_instruction_block() {
    let home = test_path("project-home");
    let project_dir = test_path("project");
    setup_library_project(&home, &project_dir);
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("review-code"),
            },
        },
    )
    .expect("prompt should create");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::InstructionRule {
                asset: instruction_rule("typescript-style"),
            },
        },
    )
    .expect("rule should create");
    write_gemini_manifest(
        &project_dir,
        SyncScope::Project,
        r#"prompts = ["review-code"]
skills = []
playbooks = []
instruction_rules = ["typescript-style"]
command_rules = []
"#,
    );

    let plan = preview_target_sync(&home, &project_dir, "gemini-cli", SyncScope::Project)
        .expect("gemini preview should plan");
    assert!(plan.conflicts.is_empty());
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);
    apply_sync(&home, &mut cache, &plan_id).expect("gemini apply should succeed");

    let command = fs::read_to_string(project_dir.join(".gemini/commands/review-code.toml"))
        .expect("command should exist");
    let gemini_md =
        fs::read_to_string(project_dir.join("GEMINI.md")).expect("GEMINI.md should exist");
    assert!(command.contains("description = \"Review a target file\""));
    assert!(command.contains("prompt = \"\"\""));
    assert!(command.contains("Review the requested code for correctness."));
    assert!(command.contains("{{args}}"));
    assert!(gemini_md.contains("<!-- FLOWMINT:GEMINI:BEGIN -->"));
    assert!(gemini_md.contains("src/**/*.ts"));
    assert!(gemini_md.contains("Prefer explicit return types."));

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn gemini_blocks_skill_playbook_and_command_rule_until_supported() {
    let home = test_path("unsupported-home");
    let project_dir = test_path("unsupported-project");
    setup_library_project(&home, &project_dir);
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
            asset: AssetDetail::CommandRule {
                asset: command_rule("dangerous-rm"),
            },
        },
    )
    .expect("command rule should create");
    write_gemini_manifest(
        &project_dir,
        SyncScope::Project,
        r#"prompts = []
skills = ["research-helper"]
playbooks = ["release-check"]
instruction_rules = []
command_rules = ["dangerous-rm"]
"#,
    );

    let plan = preview_target_sync(&home, &project_dir, "gemini-cli", SyncScope::Project)
        .expect("gemini preview should report unsupported mappings");

    assert_eq!(plan.conflicts.len(), 3);
    assert!(
        plan.conflicts
            .iter()
            .all(|conflict| { conflict.kind == SyncConflictKind::UnsupportedMapping })
    );
    assert!(plan.operations.is_empty());

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn gemini_global_apply_writes_user_level_outputs_and_global_lockfile() {
    let user_home = test_path("global-user");
    let home = user_home.join(".flowmint");
    let project_dir = user_home.join("project");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    init_library_at(&home).expect("library should initialize");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("review-code"),
            },
        },
    )
    .expect("prompt should create");
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
                target: "gemini-cli".to_string(),
                scope: SyncScope::GlobalUser,
                prompts: vec!["review-code".to_string()],
                skills: Vec::new(),
                playbooks: Vec::new(),
                instruction_rules: vec!["typescript-style".to_string()],
                command_rules: Vec::new(),
            }],
        },
    )
    .expect("global profile should write");

    let plan = preview_target_sync(&home, &project_dir, "gemini-cli", SyncScope::GlobalUser)
        .expect("global gemini preview should plan");
    let plan_id = plan.plan_id.clone();
    let confirmed_paths = operation_paths(&plan);
    let mut cache = PlanCache::default();
    cache.insert(plan);
    cache
        .acknowledge_global_plan(&plan_id, &confirmed_paths)
        .expect("global gemini plan should acknowledge");
    apply_sync(&home, &mut cache, &plan_id).expect("global gemini apply should succeed");

    assert!(
        user_home
            .join(".gemini/commands/review-code.toml")
            .is_file()
    );
    assert!(user_home.join(".gemini/GEMINI.md").is_file());
    let lockfile = fs::read_to_string(home.join("global-sync.lock")).expect("global lock exists");
    assert!(lockfile.contains("target = \"gemini-cli\""));
    assert!(lockfile.contains("scope = \"global-user\""));

    cleanup(&user_home);
}

fn setup_library_project(home: &Path, project_dir: &Path) {
    fs::create_dir_all(project_dir).expect("project dir should create");
    init_library_at(home).expect("library should initialize");
    init_project_manifest(project_dir).expect("project should initialize");
}

fn write_gemini_manifest(project_dir: &Path, scope: SyncScope, attachments: &str) {
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
target = "gemini-cli"
scope = "{scope}"
{attachments}"#
        ),
    )
    .expect("manifest should write");
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

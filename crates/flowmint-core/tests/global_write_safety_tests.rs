use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{AssetDetail, CreateAssetInput, PromptAsset};
use flowmint_core::asset::store::create_asset;
use flowmint_core::exporters::target::preview_target_sync;
use flowmint_core::project::global_profiles::{GlobalSyncProfiles, write_global_sync_profiles};
use flowmint_core::project::manifest::ProjectExportProfile;
use flowmint_core::store::init_library_at;
use flowmint_core::sync::apply::apply_sync;
use flowmint_core::sync::plan::{SyncOperation, SyncPlan, SyncScope};
use flowmint_core::sync::plan_cache::PlanCache;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-global-write-safety-{name}-{}",
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

#[test]
fn global_apply_requires_explicit_acknowledgement() {
    let user_home = test_path("missing-ack");
    let home = user_home.join(".flowmint");
    let project_dir = user_home.join("project");
    setup_global_prompt_profile(&home, &project_dir);

    let plan = preview_target_sync(&home, &project_dir, "claude-code", SyncScope::GlobalUser)
        .expect("global preview should plan");
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);

    let result = apply_sync(&home, &mut cache, &plan_id);

    assert!(result.is_err());
    assert!(!user_home.join(".claude/commands/daily-plan.md").exists());

    cleanup(&user_home);
}

#[test]
fn global_acknowledgement_must_match_current_plan_paths() {
    let user_home = test_path("path-mismatch");
    let home = user_home.join(".flowmint");
    let project_dir = user_home.join("project");
    setup_global_prompt_profile(&home, &project_dir);

    let plan = preview_target_sync(&home, &project_dir, "claude-code", SyncScope::GlobalUser)
        .expect("global preview should plan");
    let mut cache = PlanCache::default();
    cache.insert(plan.clone());

    let result = cache.acknowledge_global_plan(&plan.plan_id, &[]);

    assert!(result.is_err());
    assert!(!cache.is_global_acknowledged(&plan.plan_id));

    cleanup(&user_home);
}

#[test]
fn acknowledged_global_plan_can_apply_once() {
    let user_home = test_path("ack");
    let home = user_home.join(".flowmint");
    let project_dir = user_home.join("project");
    setup_global_prompt_profile(&home, &project_dir);

    let plan = preview_target_sync(&home, &project_dir, "claude-code", SyncScope::GlobalUser)
        .expect("global preview should plan");
    let plan_id = plan.plan_id.clone();
    let confirmed_paths = operation_paths(&plan);
    let mut cache = PlanCache::default();
    cache.insert(plan);
    cache
        .acknowledge_global_plan(&plan_id, &confirmed_paths)
        .expect("ack should match plan");

    apply_sync(&home, &mut cache, &plan_id).expect("acknowledged global apply should succeed");

    assert!(user_home.join(".claude/commands/daily-plan.md").is_file());
    assert!(!cache.is_global_acknowledged(&plan_id));

    cleanup(&user_home);
}

fn setup_global_prompt_profile(home: &Path, project_dir: &Path) {
    fs::create_dir_all(project_dir).expect("project dir should create");
    init_library_at(home).expect("library should initialize");
    create_asset(
        home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("daily-plan"),
            },
        },
    )
    .expect("prompt should create");
    write_global_sync_profiles(
        home,
        &GlobalSyncProfiles {
            profiles: vec![ProjectExportProfile {
                target: "claude-code".to_string(),
                scope: SyncScope::GlobalUser,
                prompts: vec!["daily-plan".to_string()],
                skills: Vec::new(),
                playbooks: Vec::new(),
                instruction_rules: Vec::new(),
                command_rules: Vec::new(),
            }],
        },
    )
    .expect("global profile should write");
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

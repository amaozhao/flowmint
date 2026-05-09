use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{AssetDetail, CreateAssetInput, PromptAsset};
use flowmint_core::asset::store::create_asset;
use flowmint_core::exporters::claude_code::preview_claude_code_sync;
use flowmint_core::project::manifest::{attach_prompt, init_project_manifest};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::apply::apply_sync;
use flowmint_core::sync::conflict::SyncConflictKind;
use flowmint_core::sync::plan_cache::PlanCache;

fn test_path(name: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("flowmint-conflict-{name}-{}", std::process::id()));
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
fn modified_generated_file_is_reported_as_conflict() {
    let home = test_path("home");
    let project_dir = test_path("project");
    setup(&home, &project_dir);
    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);
    apply_sync(&home, &mut cache, &plan_id).expect("apply should succeed");
    fs::write(
        project_dir.join(".claude/commands/daily-plan.md"),
        "edited outside Flowmint",
    )
    .expect("generated file should edit");

    let changed_plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");

    assert_eq!(
        changed_plan.conflicts[0].kind,
        SyncConflictKind::ModifiedGeneratedFile
    );

    cleanup(&home);
    cleanup(&project_dir);
}

#[cfg(unix)]
#[test]
fn symlink_target_is_reported_as_conflict() {
    let home = test_path("symlink-home");
    let project_dir = test_path("symlink-project");
    setup(&home, &project_dir);
    fs::create_dir_all(project_dir.join(".claude/commands")).expect("commands should create");
    fs::write(project_dir.join("outside.md"), "outside").expect("outside file should write");
    std::os::unix::fs::symlink(
        project_dir.join("outside.md"),
        project_dir.join(".claude/commands/daily-plan.md"),
    )
    .expect("symlink should create");

    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");

    assert_eq!(plan.conflicts[0].kind, SyncConflictKind::UnsafeSymlink);

    cleanup(&home);
    cleanup(&project_dir);
}

fn setup(home: &Path, project_dir: &Path) {
    fs::create_dir_all(project_dir).expect("project dir should create");
    init_library_at(home).expect("library should initialize");
    init_project_manifest(project_dir).expect("project should initialize");
    create_asset(
        home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("daily-plan"),
            },
        },
    )
    .expect("prompt should create");
    attach_prompt(project_dir, "daily-plan").expect("prompt should attach");
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

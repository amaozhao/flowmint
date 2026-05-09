use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{AssetDetail, CreateAssetInput, PromptAsset};
use flowmint_core::asset::store::create_asset;
use flowmint_core::exporters::claude_code::preview_claude_code_sync;
use flowmint_core::project::manifest::{attach_prompt, init_project_manifest};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::apply::apply_sync;
use flowmint_core::sync::plan::SyncOperation;
use flowmint_core::sync::plan_cache::PlanCache;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-idempotency-{name}-{}",
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
fn repeated_preview_after_apply_reports_no_file_changes() {
    let home = test_path("home");
    let project_dir = test_path("project");
    setup(&home, &project_dir);

    let first_plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");
    let plan_id = first_plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(first_plan);
    apply_sync(&home, &mut cache, &plan_id).expect("apply should succeed");

    let second_plan =
        preview_claude_code_sync(&home, &project_dir).expect("preview should re-plan");

    assert!(second_plan.conflicts.is_empty());
    assert!(
        second_plan
            .operations
            .iter()
            .all(|operation| matches!(operation, SyncOperation::Noop { .. }))
    );

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn lockfile_records_required_export_fields() {
    let home = test_path("lock-home");
    let project_dir = test_path("lock-project");
    setup(&home, &project_dir);
    let plan = preview_claude_code_sync(&home, &project_dir).expect("preview should plan");
    let plan_id = plan.plan_id.clone();
    let mut cache = PlanCache::default();
    cache.insert(plan);

    apply_sync(&home, &mut cache, &plan_id).expect("apply should succeed");

    let lockfile =
        fs::read_to_string(project_dir.join(".flowmint.lock")).expect("lockfile should exist");
    assert!(lockfile.contains("target = \"claude-code\""));
    assert!(lockfile.contains("asset_type = \"prompt\""));
    assert!(lockfile.contains("asset_id = \"daily-plan\""));
    assert!(lockfile.contains("source_hash = \"fnv1a64:"));
    assert!(lockfile.contains("output_path = \".claude/commands/daily-plan.md\""));
    assert!(lockfile.contains("output_hash = \"fnv1a64:"));
    assert!(lockfile.contains("updated_at = \"unix:"));

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

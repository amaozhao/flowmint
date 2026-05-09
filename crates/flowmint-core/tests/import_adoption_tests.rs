use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{AssetDetail, AssetType};
use flowmint_core::asset::store::get_asset;
use flowmint_core::import::adopt::{
    ImportAdoptionMode, ImportAdoptionSelection, apply_import_adoption, preview_import_adoption,
};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::plan::SyncScope;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-import-adoption-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

#[test]
fn copy_prompt_import_creates_library_asset_without_lockfile() {
    let home = test_path("copy-home");
    let project_dir = test_path("copy-project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(project_dir.join(".claude/commands")).expect("commands should create");
    let source_path = project_dir.join(".claude/commands/review-pr.md");
    fs::write(&source_path, "Review this PR.").expect("source should write");

    let plan = preview_import_adoption(
        &home,
        &project_dir,
        "claude-code",
        SyncScope::Project,
        vec![selection(
            "review-pr",
            AssetType::Prompt,
            source_path,
            ImportAdoptionMode::CopyIntoLibrary,
        )],
    )
    .expect("preview should succeed");
    assert!(plan.conflicts.is_empty());

    let result = apply_import_adoption(&home, &project_dir, &plan).expect("apply should succeed");

    assert_eq!(result.copied_assets, 1);
    assert_eq!(result.adopted_assets, 0);
    let asset = get_asset(&home, "prompt:review-pr").expect("prompt should exist");
    match asset {
        AssetDetail::Prompt { asset } => assert_eq!(asset.body, "Review this PR."),
        _ => panic!("expected prompt asset"),
    }
    assert!(!project_dir.join(".flowmint.lock").exists());

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn adopt_prompt_import_creates_asset_and_lock_record_after_apply() {
    let home = test_path("adopt-home");
    let project_dir = test_path("adopt-project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(project_dir.join(".claude/commands")).expect("commands should create");
    let source_path = project_dir.join(".claude/commands/review-pr.md");
    fs::write(&source_path, "Review this PR.").expect("source should write");

    let plan = preview_import_adoption(
        &home,
        &project_dir,
        "claude-code",
        SyncScope::Project,
        vec![selection(
            "review-pr",
            AssetType::Prompt,
            source_path,
            ImportAdoptionMode::AdoptIntoFlowmint,
        )],
    )
    .expect("preview should succeed");
    assert!(!project_dir.join(".flowmint.lock").exists());

    let result = apply_import_adoption(&home, &project_dir, &plan).expect("apply should succeed");

    assert_eq!(result.copied_assets, 0);
    assert_eq!(result.adopted_assets, 1);
    let lockfile =
        fs::read_to_string(project_dir.join(".flowmint.lock")).expect("lock should exist");
    assert!(lockfile.contains("target = \"claude-code\""));
    assert!(lockfile.contains("output_path = \".claude/commands/review-pr.md\""));

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn source_change_between_preview_and_apply_blocks_import() {
    let home = test_path("changed-home");
    let project_dir = test_path("changed-project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(project_dir.join(".claude/commands")).expect("commands should create");
    let source_path = project_dir.join(".claude/commands/review-pr.md");
    fs::write(&source_path, "Review this PR.").expect("source should write");

    let plan = preview_import_adoption(
        &home,
        &project_dir,
        "claude-code",
        SyncScope::Project,
        vec![selection(
            "review-pr",
            AssetType::Prompt,
            source_path.clone(),
            ImportAdoptionMode::AdoptIntoFlowmint,
        )],
    )
    .expect("preview should succeed");
    fs::write(&source_path, "Changed content.").expect("source should mutate");

    let result = apply_import_adoption(&home, &project_dir, &plan);

    assert!(result.is_err());
    assert!(get_asset(&home, "prompt:review-pr").is_err());
    assert!(!project_dir.join(".flowmint.lock").exists());

    cleanup(&home);
    cleanup(&project_dir);
}

fn selection(
    id: &str,
    asset_type: AssetType,
    source_path: PathBuf,
    mode: ImportAdoptionMode,
) -> ImportAdoptionSelection {
    ImportAdoptionSelection {
        id: id.to_string(),
        asset_type,
        source_path,
        mode,
    }
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::error::FlowmintError;
use flowmint_core::exporters::target::preview_target_sync;
use flowmint_core::sync::plan::SyncScope;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-exporter-router-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn create_dir(name: &str) -> PathBuf {
    let path = test_path(name);
    fs::create_dir_all(&path).expect("dir should create");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

#[test]
fn exporter_router_routes_claude_code_project_preview() {
    let home = create_dir("home");
    let project = create_dir("project");

    let plan = preview_target_sync(&home, &project, "claude-code", SyncScope::Project)
        .expect("claude-code project preview should route");

    assert_eq!(plan.exporter, "claude-code");
    assert_eq!(plan.scope, SyncScope::Project);

    cleanup(&home);
    cleanup(&project);
}

#[test]
fn exporter_router_rejects_unknown_target() {
    let home = create_dir("unknown-home");
    let project = create_dir("unknown-project");

    let error = preview_target_sync(&home, &project, "cursor", SyncScope::Project)
        .expect_err("unknown target should fail");

    assert!(matches!(
        error,
        FlowmintError::UnsupportedSyncTarget { target } if target == "cursor"
    ));

    cleanup(&home);
    cleanup(&project);
}

#[test]
fn exporter_router_routes_claude_code_global_preview() {
    let home = create_dir("scope-home");
    let project = create_dir("scope-project");

    let plan = preview_target_sync(&home, &project, "claude-code", SyncScope::GlobalUser)
        .expect("global claude sync should route after FM-421");

    assert_eq!(plan.exporter, "claude-code");
    assert_eq!(plan.scope, SyncScope::GlobalUser);

    cleanup(&home);
    cleanup(&project);
}

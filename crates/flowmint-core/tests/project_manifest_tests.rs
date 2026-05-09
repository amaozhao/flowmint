use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::project::manifest::{
    ProjectExportProfile, attach_prompt, attach_skill, detach_prompt, detach_skill,
    init_project_manifest, load_project_manifest, write_project_manifest,
};
use flowmint_core::project::recent::{add_recent_project, list_recent_projects};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::plan::SyncScope;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-project-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn create_project_dir(name: &str) -> PathBuf {
    let path = test_path(name);
    fs::create_dir_all(&path).expect("project dir should create");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

#[test]
fn init_project_manifest_creates_default_manifest_when_missing() {
    let project_dir = create_project_dir("default-manifest");

    let manifest = init_project_manifest(&project_dir).expect("manifest should initialize");

    assert_eq!(
        manifest.project.name,
        project_dir.file_name().unwrap().to_string_lossy()
    );
    assert_eq!(manifest.export.target, "claude-code");
    assert!(manifest.attach.prompts.is_empty());
    assert!(manifest.attach.skills.is_empty());

    let content = fs::read_to_string(project_dir.join(".flowmint.toml"))
        .expect("manifest file should be readable");
    assert!(content.contains("[project]"));
    assert!(content.contains("[export]"));
    assert!(content.contains("[attach]"));
    assert!(content.contains("target = \"claude-code\""));
    assert!(!content.contains("playbooks"));

    cleanup(&project_dir);
}

#[test]
fn existing_project_manifest_is_read_without_overwrite() {
    let project_dir = create_project_dir("existing-manifest");
    let manifest_path = project_dir.join(".flowmint.toml");
    let existing = r#"[project]
name = "Custom Project"

[export]
target = "cursor"

[attach]
prompts = ["daily-plan"]
skills = ["research-helper"]
"#;
    fs::write(&manifest_path, existing).expect("manifest should write");

    let manifest = init_project_manifest(&project_dir).expect("existing manifest should load");

    assert_eq!(manifest.project.name, "Custom Project");
    assert_eq!(manifest.export.target, "cursor");
    assert_eq!(manifest.attach.prompts, vec!["daily-plan"]);
    assert_eq!(manifest.attach.skills, vec!["research-helper"]);
    assert_eq!(
        fs::read_to_string(&manifest_path).expect("manifest should still exist"),
        existing
    );

    cleanup(&project_dir);
}

#[test]
fn attach_and_detach_assets_persist_manifest() {
    let project_dir = create_project_dir("attach-detach");
    init_project_manifest(&project_dir).expect("manifest should initialize");

    attach_prompt(&project_dir, "daily-plan").expect("prompt should attach");
    attach_prompt(&project_dir, "daily-plan").expect("duplicate prompt attach should be ignored");
    attach_skill(&project_dir, "research-helper").expect("skill should attach");

    let attached = load_project_manifest(&project_dir).expect("manifest should reload");
    assert_eq!(attached.attach.prompts, vec!["daily-plan"]);
    assert_eq!(attached.attach.skills, vec!["research-helper"]);

    detach_prompt(&project_dir, "daily-plan").expect("prompt should detach");
    detach_skill(&project_dir, "research-helper").expect("skill should detach");

    let detached = load_project_manifest(&project_dir).expect("manifest should reload");
    assert!(detached.attach.prompts.is_empty());
    assert!(detached.attach.skills.is_empty());

    cleanup(&project_dir);
}

#[test]
fn v2_project_manifest_parses_multiple_export_profiles() {
    let project_dir = create_project_dir("v2-manifest");
    fs::write(
        project_dir.join(".flowmint.toml"),
        r#"[project]
name = "Multi Target"

[[exports]]
target = "claude-code"
scope = "project"
prompts = ["review-pr"]
skills = ["api-helper"]
playbooks = ["release-check"]
instruction_rules = ["typescript-style"]
command_rules = []

[[exports]]
target = "codex"
scope = "global-user"
prompts = []
skills = ["api-helper"]
playbooks = []
instruction_rules = ["personal-style"]
command_rules = ["safe-git-status"]
"#,
    )
    .expect("manifest should write");

    let manifest = load_project_manifest(&project_dir).expect("manifest should parse");

    assert_eq!(manifest.project.name, "Multi Target");
    assert_eq!(manifest.exports.len(), 2);
    assert_eq!(manifest.exports[0].target, "claude-code");
    assert_eq!(manifest.exports[0].scope, SyncScope::Project);
    assert_eq!(manifest.exports[0].prompts, vec!["review-pr"]);
    assert_eq!(manifest.exports[0].skills, vec!["api-helper"]);
    assert_eq!(manifest.exports[0].playbooks, vec!["release-check"]);
    assert_eq!(
        manifest.exports[0].instruction_rules,
        vec!["typescript-style"]
    );
    assert_eq!(manifest.exports[1].target, "codex");
    assert_eq!(manifest.exports[1].scope, SyncScope::GlobalUser);
    assert_eq!(manifest.exports[1].command_rules, vec!["safe-git-status"]);

    assert_eq!(manifest.export.target, "claude-code");
    assert_eq!(manifest.attach.prompts, vec!["review-pr"]);
    assert_eq!(manifest.attach.skills, vec!["api-helper"]);

    cleanup(&project_dir);
}

#[test]
fn v2_project_manifest_renders_when_non_v1_fields_are_present() {
    let project_dir = create_project_dir("render-v2-manifest");
    let manifest = flowmint_core::project::manifest::ProjectManifest {
        project: flowmint_core::project::manifest::ProjectMetadata {
            name: "Render V2".to_string(),
        },
        export: flowmint_core::project::manifest::ProjectExport {
            target: "claude-code".to_string(),
        },
        attach: flowmint_core::project::manifest::ProjectAttachments {
            prompts: vec!["review-pr".to_string()],
            skills: vec!["api-helper".to_string()],
        },
        exports: vec![ProjectExportProfile {
            target: "claude-code".to_string(),
            scope: SyncScope::Project,
            prompts: vec!["review-pr".to_string()],
            skills: vec!["api-helper".to_string()],
            playbooks: vec!["release-check".to_string()],
            instruction_rules: vec!["typescript-style".to_string()],
            command_rules: Vec::new(),
        }],
    };

    write_project_manifest(&project_dir, &manifest).expect("manifest should write");
    let content =
        fs::read_to_string(project_dir.join(".flowmint.toml")).expect("manifest should read");

    assert!(content.contains("[[exports]]"));
    assert!(content.contains("scope = \"project\""));
    assert!(content.contains("playbooks = [\"release-check\"]"));
    assert!(content.contains("instruction_rules = [\"typescript-style\"]"));

    cleanup(&project_dir);
}

#[test]
fn recent_projects_persist_and_deduplicate() {
    let home = test_path("recent-home");
    let first = create_project_dir("recent-first");
    let second = create_project_dir("recent-second");
    init_library_at(&home).expect("library should initialize");

    add_recent_project(&home, &first).expect("first project should add");
    add_recent_project(&home, &second).expect("second project should add");
    add_recent_project(&home, &first).expect("existing project should move to front");

    let projects = list_recent_projects(&home).expect("recent projects should load");
    let first = first
        .canonicalize()
        .expect("first project path should canonicalize");
    let second = second
        .canonicalize()
        .expect("second project path should canonicalize");
    assert_eq!(projects, vec![first.clone(), second.clone()]);

    let reloaded = list_recent_projects(&home).expect("recent projects should reload");
    assert_eq!(reloaded, projects);

    cleanup(&home);
    cleanup(&first);
    cleanup(&second);
}

use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::project::global_profiles::{
    GlobalSyncProfiles, attach_global_profile_asset, detach_global_profile_asset,
    global_sync_profiles_path, load_global_sync_profiles, write_global_sync_profiles,
};
use flowmint_core::project::manifest::ProjectExportProfile;
use flowmint_core::sync::plan::SyncScope;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-global-profiles-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn global_profile(target: &str) -> ProjectExportProfile {
    ProjectExportProfile {
        target: target.to_string(),
        scope: SyncScope::GlobalUser,
        prompts: Vec::new(),
        skills: vec!["api-helper".to_string()],
        playbooks: Vec::new(),
        instruction_rules: vec!["personal-style".to_string()],
        command_rules: Vec::new(),
    }
}

#[test]
fn global_sync_profiles_default_to_empty_when_missing() {
    let home = test_path("missing");

    let profiles = load_global_sync_profiles(&home).expect("missing profiles should load");

    assert!(profiles.profiles.is_empty());
    assert_eq!(
        global_sync_profiles_path(&home),
        home.join("global-sync-profiles.toml")
    );

    cleanup(&home);
}

#[test]
fn global_sync_profiles_round_trip_global_user_profiles() {
    let home = test_path("round-trip");
    let profiles = GlobalSyncProfiles {
        profiles: vec![global_profile("claude-code"), global_profile("codex")],
    };

    write_global_sync_profiles(&home, &profiles).expect("profiles should write");
    let reloaded = load_global_sync_profiles(&home).expect("profiles should reload");

    assert_eq!(reloaded, profiles);
    let content = fs::read_to_string(global_sync_profiles_path(&home)).expect("file should read");
    assert!(content.contains("[[profiles]]"));
    assert!(content.contains("scope = \"global-user\""));

    cleanup(&home);
}

#[test]
fn global_sync_profiles_reject_project_scoped_profiles() {
    let home = test_path("reject-project");
    let profiles = GlobalSyncProfiles {
        profiles: vec![ProjectExportProfile {
            scope: SyncScope::Project,
            ..global_profile("claude-code")
        }],
    };

    let result = write_global_sync_profiles(&home, &profiles);

    assert!(result.is_err());
    assert!(!global_sync_profiles_path(&home).exists());

    cleanup(&home);
}

#[test]
fn global_sync_profiles_attach_and_detach_assets_by_target() {
    let home = test_path("attach-detach");

    let profiles = attach_global_profile_asset(&home, "codex", "skill:api-helper")
        .expect("skill should attach");
    assert_eq!(profiles.profiles.len(), 1);
    assert_eq!(profiles.profiles[0].target, "codex");
    assert_eq!(profiles.profiles[0].scope, SyncScope::GlobalUser);
    assert_eq!(profiles.profiles[0].skills, vec!["api-helper"]);

    attach_global_profile_asset(&home, "codex", "instruction-rule:personal-style")
        .expect("rule should attach");
    let profiles = detach_global_profile_asset(&home, "codex", "skill:api-helper")
        .expect("skill should detach");

    assert!(profiles.profiles[0].skills.is_empty());
    assert_eq!(
        profiles.profiles[0].instruction_rules,
        vec!["personal-style"]
    );

    cleanup(&home);
}

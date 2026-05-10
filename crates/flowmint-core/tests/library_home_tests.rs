use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::store::{get_app_state_for_home, global_user_home_dir, init_library_at};

fn assert_library_structure(home: &Path) {
    assert!(home.join("config.toml").is_file());
    assert!(home.join("recent-projects.toml").is_file());
    assert!(home.join("prompts").is_dir());
    assert!(home.join("skills").is_dir());
    assert!(home.join("playbooks").is_dir());
    assert!(home.join("rules").is_dir());
    assert!(home.join("import-sources").is_dir());
    assert!(home.join("templates").is_dir());
    assert!(home.join("cache").is_dir());
    assert!(home.join("backups").is_dir());
}

fn test_home(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

#[test]
fn init_library_creates_expected_local_structure() {
    let home = test_home("init-structure");

    let info = init_library_at(&home).expect("library should initialize");

    assert_eq!(info.path, home);
    assert!(info.initialized);
    assert_library_structure(&info.path);

    fs::remove_dir_all(info.path).expect("test library should be removable");
}

#[test]
fn app_state_reports_missing_library_before_init_and_existing_library_after_init() {
    let home = test_home("app-state");

    let missing = get_app_state_for_home(&home).expect("state should load for missing library");
    assert!(!missing.library.initialized);

    init_library_at(&home).expect("library should initialize");

    let existing =
        get_app_state_for_home(&home).expect("state should load for initialized library");
    assert!(existing.library.initialized);
    assert_library_structure(&existing.library.path);

    fs::remove_dir_all(existing.library.path).expect("test library should be removable");
}

#[test]
fn init_library_is_idempotent() {
    let home = test_home("idempotent");

    let first = init_library_at(&home).expect("first init should succeed");
    let second = init_library_at(&home).expect("second init should succeed");

    assert_eq!(first.path, second.path);
    assert!(second.initialized);
    assert_library_structure(&second.path);

    fs::remove_dir_all(second.path).expect("test library should be removable");
}

#[test]
fn app_state_keeps_legacy_library_initialized_without_import_sources() {
    let home = test_home("legacy-without-import-sources");
    fs::create_dir_all(&home).expect("legacy home should create");
    fs::write(home.join("config.toml"), "").expect("config should create");
    fs::write(home.join("recent-projects.toml"), "").expect("recent projects should create");
    for dir in [
        "prompts",
        "skills",
        "playbooks",
        "rules",
        "templates",
        "cache",
        "backups",
    ] {
        fs::create_dir_all(home.join(dir)).expect("legacy dir should create");
    }

    let state = get_app_state_for_home(&home).expect("legacy state should load");

    assert!(state.library.initialized);
    assert!(!home.join("import-sources").exists());

    fs::remove_dir_all(home).expect("test library should be removable");
}

#[test]
fn global_user_home_uses_parent_for_default_flowmint_home() {
    let user_home = test_home("global-default-user");
    let home = user_home.join(".flowmint");

    assert_eq!(
        global_user_home_dir(&home).expect("global user home should resolve"),
        user_home
    );
}

#[test]
fn global_user_home_uses_os_home_for_custom_library_path() {
    let custom_library = test_home("global-custom-library").join("library");
    let expected_home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("test environment should expose a user home");

    assert_eq!(
        global_user_home_dir(&custom_library).expect("global user home should resolve"),
        expected_home
    );
}

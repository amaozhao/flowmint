use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{AssetDetail, AssetType, CreateAssetInput, PromptAsset};
use flowmint_core::asset::store::create_asset;
use flowmint_core::import::{ImportConfidence, scan_import_candidates};
use flowmint_core::store::init_library_at;
use flowmint_core::sync::plan::SyncScope;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-import-scan-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn prompt(id: &str) -> PromptAsset {
    PromptAsset {
        id: id.to_string(),
        name: "Review PR".to_string(),
        description: None,
        tags: Vec::new(),
        variables: Vec::new(),
        body: "Review this PR.".to_string(),
    }
}

#[test]
fn scans_claude_project_assets_read_only_and_reports_collisions() {
    let home = test_path("claude-home");
    let project_dir = test_path("claude-project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(project_dir.join(".claude/commands")).expect("commands should create");
    fs::create_dir_all(project_dir.join(".claude/skills/api-helper")).expect("skill should create");
    fs::create_dir_all(project_dir.join(".claude/rules")).expect("rules should create");
    fs::write(
        project_dir.join(".claude/commands/review-pr.md"),
        "Review this PR.",
    )
    .expect("prompt should write");
    fs::write(
        project_dir.join(".claude/skills/api-helper/SKILL.md"),
        "# API Helper\n",
    )
    .expect("skill should write");
    fs::write(
        project_dir.join(".claude/rules/typescript-style.md"),
        "Prefer explicit return types.",
    )
    .expect("rule should write");
    fs::write(project_dir.join("CLAUDE.md"), "Project instructions.")
        .expect("claude instructions should write");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("review-pr"),
            },
        },
    )
    .expect("existing prompt should create");

    let candidates = scan_import_candidates(&home, &project_dir, "claude-code", SyncScope::Project)
        .expect("scan should succeed");

    assert_eq!(candidates.len(), 4);
    assert_eq!(candidates[0].id, "review-pr");
    assert_eq!(candidates[0].asset_type, AssetType::Prompt);
    assert!(candidates[0].collision.is_some());
    assert_eq!(candidates[1].id, "api-helper");
    assert_eq!(candidates[1].asset_type, AssetType::Skill);
    assert_eq!(candidates[2].id, "claude-project-instructions");
    assert_eq!(candidates[2].asset_type, AssetType::InstructionRule);
    assert_eq!(candidates[3].id, "typescript-style");
    assert_eq!(candidates[3].asset_type, AssetType::InstructionRule);
    assert!(candidates.iter().all(|candidate| {
        candidate.scope == SyncScope::Project && candidate.confidence == ImportConfidence::High
    }));
    assert!(!project_dir.join(".flowmint.lock").exists());

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn scans_global_codex_and_gemini_paths_from_user_home() {
    let user_home = test_path("global-user");
    let home = user_home.join(".flowmint");
    let project_dir = user_home.join("project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(&project_dir).expect("project should create");
    fs::create_dir_all(user_home.join(".agents/skills/research-helper"))
        .expect("legacy codex skill should create");
    fs::write(
        user_home.join(".agents/skills/research-helper/SKILL.md"),
        "# Legacy Research Helper\n",
    )
    .expect("legacy codex skill should write");
    fs::create_dir_all(user_home.join(".codex/skills/codex-helper"))
        .expect("codex skill should create");
    fs::write(
        user_home.join(".codex/skills/codex-helper/SKILL.md"),
        "# Codex Helper\n",
    )
    .expect("codex skill should write");
    fs::create_dir_all(user_home.join(".codex/rules")).expect("codex rules should create");
    fs::write(
        user_home.join(".codex/rules/safe-git-status.rules"),
        "prefix_rule = [\"git\", \"status\"]\n",
    )
    .expect("codex command rule should write");
    fs::write(
        user_home.join(".codex/AGENTS.md"),
        "Global Codex instructions.",
    )
    .expect("codex instructions should write");
    fs::create_dir_all(user_home.join(".gemini/commands")).expect("gemini commands should create");
    fs::write(
        user_home.join(".gemini/commands/review-code.toml"),
        "description = \"Review\"\nprompt = \"\"\"Review code.\"\"\"\n",
    )
    .expect("gemini command should write");

    let codex = scan_import_candidates(&home, &project_dir, "codex", SyncScope::GlobalUser)
        .expect("codex global scan should succeed");
    let gemini = scan_import_candidates(&home, &project_dir, "gemini-cli", SyncScope::GlobalUser)
        .expect("gemini global scan should succeed");

    assert_eq!(codex.len(), 4);
    assert_eq!(codex[0].id, "codex-helper");
    assert_eq!(
        codex[0].source_path,
        user_home.join(".codex/skills/codex-helper")
    );
    assert_eq!(codex[1].id, "research-helper");
    assert_eq!(
        codex[1].source_path,
        user_home.join(".agents/skills/research-helper")
    );
    assert_eq!(codex[2].id, "codex-global-agents");
    assert_eq!(codex[2].asset_type, AssetType::InstructionRule);
    assert_eq!(codex[3].id, "safe-git-status");
    assert_eq!(codex[3].asset_type, AssetType::CommandRule);
    assert_eq!(gemini.len(), 1);
    assert_eq!(gemini[0].id, "review-code");
    assert_eq!(gemini[0].asset_type, AssetType::Prompt);

    cleanup(&user_home);
}

#[test]
fn scans_codex_project_skills_agents_and_rules() {
    let home = test_path("codex-project-home");
    let project_dir = test_path("codex-project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(project_dir.join(".codex/skills/project-helper"))
        .expect("codex project skill should create");
    fs::write(
        project_dir.join(".codex/skills/project-helper/SKILL.md"),
        "# Project Helper\n",
    )
    .expect("codex project skill should write");
    fs::create_dir_all(project_dir.join(".codex/rules")).expect("codex rules should create");
    fs::write(
        project_dir.join(".codex/rules/safe-git-status.rules"),
        "prefix_rule = [\"git\", \"status\"]\n",
    )
    .expect("codex command rule should write");
    fs::write(project_dir.join("AGENTS.md"), "Project Codex instructions.")
        .expect("codex agents should write");

    let candidates = scan_import_candidates(&home, &project_dir, "codex", SyncScope::Project)
        .expect("codex project scan should succeed");

    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0].id, "project-helper");
    assert_eq!(candidates[0].asset_type, AssetType::Skill);
    assert_eq!(candidates[1].id, "codex-project-agents");
    assert_eq!(candidates[1].asset_type, AssetType::InstructionRule);
    assert_eq!(candidates[2].id, "safe-git-status");
    assert_eq!(candidates[2].asset_type, AssetType::CommandRule);

    cleanup(&home);
    cleanup(&project_dir);
}

#[test]
fn scans_claude_global_instructions_from_claude_home() {
    let user_home = test_path("claude-global-user");
    let home = user_home.join(".flowmint");
    let project_dir = user_home.join("project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(&project_dir).expect("project should create");
    fs::create_dir_all(user_home.join(".claude")).expect("claude dir should create");
    fs::write(
        user_home.join(".claude/CLAUDE.md"),
        "Global Claude instructions.",
    )
    .expect("claude global instructions should write");

    let candidates =
        scan_import_candidates(&home, &project_dir, "claude-code", SyncScope::GlobalUser)
            .expect("claude global scan should succeed");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].id, "claude-global-instructions");
    assert_eq!(candidates[0].asset_type, AssetType::InstructionRule);
    assert_eq!(
        candidates[0].source_path,
        user_home.join(".claude/CLAUDE.md")
    );

    cleanup(&user_home);
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

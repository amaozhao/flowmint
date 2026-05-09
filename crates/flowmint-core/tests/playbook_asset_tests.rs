use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{
    AssetDetail, AssetFilter, AssetType, CreateAssetInput, PlaybookAsset, PlaybookInput,
    PlaybookInvocation, PlaybookSideEffectLevel, PlaybookStep,
};
use flowmint_core::asset::playbook::{
    create_playbook, get_playbook, list_playbooks, promote_skill_to_playbook,
    render_playbook_skill_md,
};
use flowmint_core::asset::skill::create_skill;
use flowmint_core::asset::store::{create_asset, get_asset, list_assets};

fn test_home(name: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("flowmint-playbook-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn playbook(id: &str) -> PlaybookAsset {
    PlaybookAsset {
        id: id.to_string(),
        name: "Release Check".to_string(),
        description: Some("Repeatable release review".to_string()),
        tags: vec!["release".to_string()],
        trigger: "Before publishing a release".to_string(),
        inputs: vec![PlaybookInput {
            name: "version".to_string(),
            description: Some("Release version".to_string()),
            required: true,
        }],
        steps: vec![
            PlaybookStep {
                title: "Run checks".to_string(),
                body: "Run the full verification suite.".to_string(),
            },
            PlaybookStep {
                title: "Review output".to_string(),
                body: "Confirm all generated artifacts exist.".to_string(),
            },
        ],
        verification: "All checks pass.".to_string(),
        failure_handling: "Stop and report the failing command.".to_string(),
        side_effect_level: PlaybookSideEffectLevel::RunsCommands,
        recommended_invocation: PlaybookInvocation::Manual,
        target_compatibility: vec!["claude-code".to_string(), "codex".to_string()],
    }
}

#[test]
fn create_list_get_and_render_playbook() {
    let home = test_home("create-list");

    create_playbook(&home, playbook("release-check")).expect("playbook should create");
    let playbooks = list_playbooks(&home).expect("playbooks should list");
    let loaded = get_playbook(&home, "release-check").expect("playbook should load");
    let skill_md = render_playbook_skill_md(&loaded);

    assert_eq!(playbooks.len(), 1);
    assert_eq!(playbooks[0].asset_type, AssetType::Playbook);
    assert_eq!(
        loaded.side_effect_level,
        PlaybookSideEffectLevel::RunsCommands
    );
    assert!(skill_md.contains("# Release Check"));
    assert!(skill_md.contains("## Steps"));
    assert!(skill_md.contains("Run the full verification suite."));

    cleanup(&home);
}

#[test]
fn playbook_requires_at_least_one_step() {
    let home = test_home("missing-step");
    let invalid = PlaybookAsset {
        steps: Vec::new(),
        ..playbook("release-check")
    };

    let result = create_playbook(&home, invalid);

    assert!(result.is_err());
    assert!(!home.join("playbooks/release-check.md").exists());

    cleanup(&home);
}

#[test]
fn asset_store_handles_playbook_assets() {
    let home = test_home("asset-store");

    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Playbook {
                asset: playbook("release-check"),
            },
        },
    )
    .expect("playbook should create through asset store");

    let loaded = get_asset(&home, "playbook:release-check").expect("playbook should load");
    let summaries = list_assets(
        &home,
        AssetFilter {
            asset_type: Some(AssetType::Playbook),
            query: None,
        },
    )
    .expect("playbooks should list");

    assert!(matches!(loaded, AssetDetail::Playbook { .. }));
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].id, "release-check");

    cleanup(&home);
}

#[test]
fn tagged_skill_can_be_promoted_to_playbook_without_deleting_skill() {
    let home = test_home("promote");
    create_skill(
        &home,
        flowmint_core::asset::model::SkillAsset {
            id: "daily-playbook".to_string(),
            name: "Daily Playbook".to_string(),
            description: Some("Legacy playbook skill".to_string()),
            tags: vec!["playbook".to_string(), "daily".to_string()],
            root_dir: PathBuf::new(),
            skill_md: "# Daily Playbook\n\n## Steps\n\n1. Check status.\n".to_string(),
            metadata: None,
            files: Vec::new(),
        },
    )
    .expect("skill should create");

    let promoted =
        promote_skill_to_playbook(&home, "daily-playbook", "daily-review").expect("should promote");

    assert_eq!(promoted.id, "daily-review");
    assert_eq!(promoted.tags, vec!["playbook", "daily"]);
    assert!(home.join("skills/daily-playbook/SKILL.md").exists());
    assert!(home.join("playbooks/daily-review.md").exists());

    cleanup(&home);
}

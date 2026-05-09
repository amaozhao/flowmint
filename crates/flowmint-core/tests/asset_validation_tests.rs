use std::fs;
use std::path::Path;

use flowmint_core::asset::model::{AssetType, PromptAsset, SkillAsset};
use flowmint_core::asset::{is_safe_asset_id, validate_new_asset_id};
use flowmint_core::validation::{ValidationStatus, validate_prompt, validate_skill};

fn test_home(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-asset-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

#[test]
fn asset_ids_allow_only_lowercase_digits_hyphen_and_underscore() {
    for id in ["fastapi-review", "prd_review", "skill1", "a"] {
        assert!(is_safe_asset_id(id), "{id} should be valid");
    }

    for id in ["", "FastAPI", "has space", "../escape", "a.b", "中文"] {
        assert!(!is_safe_asset_id(id), "{id} should be invalid");
    }
}

#[test]
fn prompt_validation_reports_missing_required_fields() {
    let prompt = PromptAsset {
        id: "bad prompt".to_string(),
        name: String::new(),
        description: None,
        tags: Vec::new(),
        variables: Vec::new(),
        body: String::new(),
    };

    let report = validate_prompt(&prompt);

    assert_eq!(report.status, ValidationStatus::Invalid);
    assert!(report.messages.iter().any(|message| message.contains("id")));
    assert!(
        report
            .messages
            .iter()
            .any(|message| message.contains("name"))
    );
    assert!(
        report
            .messages
            .iter()
            .any(|message| message.contains("body"))
    );
}

#[test]
fn skill_validation_requires_safe_id_and_non_empty_skill_markdown() {
    let skill = SkillAsset {
        id: "BadSkill".to_string(),
        name: "Bad Skill".to_string(),
        description: None,
        tags: Vec::new(),
        root_dir: Path::new("/tmp/bad-skill").to_path_buf(),
        skill_md: "   ".to_string(),
        metadata: None,
        files: Vec::new(),
    };

    let report = validate_skill(&skill);

    assert_eq!(report.status, ValidationStatus::Invalid);
    assert!(report.messages.iter().any(|message| message.contains("id")));
    assert!(
        report
            .messages
            .iter()
            .any(|message| message.contains("SKILL.md"))
    );
}

#[test]
fn new_asset_id_validation_detects_existing_skill_directory() {
    let home = test_home("duplicate-skill");
    let skills_dir = home.join("skills");
    fs::create_dir_all(skills_dir.join("nginx-debug")).expect("test skill should exist");

    let report = validate_new_asset_id(&home, AssetType::Skill, "nginx-debug");

    assert_eq!(report.status, ValidationStatus::Invalid);
    assert!(
        report
            .messages
            .iter()
            .any(|message| message.contains("already exists"))
    );

    fs::remove_dir_all(home).expect("test home should be removable");
}

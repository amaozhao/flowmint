use std::fs;

use flowmint_core::asset::model::PromptAsset;
use flowmint_core::asset::prompt::{create_prompt, get_prompt, list_prompts, update_prompt};
use flowmint_core::store::init_library_at;

fn test_home(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-prompt-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn prompt(id: &str, body: &str) -> PromptAsset {
    PromptAsset {
        id: id.to_string(),
        name: "FastAPI Review".to_string(),
        description: Some("Review FastAPI backend changes".to_string()),
        tags: vec!["fastapi".to_string(), "review".to_string()],
        variables: Vec::new(),
        body: body.to_string(),
    }
}

#[test]
fn create_prompt_writes_markdown_file_and_can_reload_it() {
    let home = test_home("create");
    init_library_at(&home).expect("library should initialize");

    let created = create_prompt(&home, prompt("fastapi-review", "Review this code."))
        .expect("prompt should be created");

    assert_eq!(created.id, "fastapi-review");
    assert!(home.join("prompts/fastapi-review.md").is_file());

    let loaded = get_prompt(&home, "fastapi-review").expect("prompt should reload");
    assert_eq!(loaded.body, "Review this code.");
    assert_eq!(loaded.tags, vec!["fastapi", "review"]);

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn list_prompts_returns_saved_prompt_summaries() {
    let home = test_home("list");
    init_library_at(&home).expect("library should initialize");
    create_prompt(&home, prompt("fastapi-review", "Review this code."))
        .expect("prompt should be created");

    let prompts = list_prompts(&home).expect("prompts should list");

    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].id, "fastapi-review");
    assert_eq!(prompts[0].name, "FastAPI Review");
    assert!(
        prompts[0].updated_at.is_some(),
        "prompt summary should expose file modification time"
    );

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn update_prompt_rewrites_existing_prompt_file() {
    let home = test_home("update");
    init_library_at(&home).expect("library should initialize");
    create_prompt(&home, prompt("fastapi-review", "Old body")).expect("prompt should be created");

    update_prompt(&home, prompt("fastapi-review", "New body")).expect("prompt should update");

    let loaded = get_prompt(&home, "fastapi-review").expect("prompt should reload");
    assert_eq!(loaded.body, "New body");

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn invalid_prompt_does_not_write_partial_file() {
    let home = test_home("invalid");
    init_library_at(&home).expect("library should initialize");

    let result = create_prompt(&home, prompt("Bad Id", ""));

    assert!(result.is_err());
    assert!(!home.join("prompts/Bad Id.md").exists());

    fs::remove_dir_all(home).expect("test home should be removable");
}

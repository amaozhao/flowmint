use std::fs;
use std::path::PathBuf;

use flowmint_core::asset::model::{SkillAsset, SkillFile, SkillFileKind, SkillMetadata};
use flowmint_core::asset::skill::{create_skill, get_skill, list_skills, update_skill};
use flowmint_core::store::init_library_at;

fn test_home(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-skill-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn skill(id: &str, skill_md: &str) -> SkillAsset {
    SkillAsset {
        id: id.to_string(),
        name: "FastAPI Backend Review".to_string(),
        description: Some("Review backend changes".to_string()),
        tags: vec!["fastapi".to_string(), "review".to_string()],
        root_dir: PathBuf::new(),
        skill_md: skill_md.to_string(),
        metadata: None,
        files: Vec::new(),
    }
}

#[test]
fn create_skill_writes_directory_and_can_reload_it() {
    let home = test_home("create");
    init_library_at(&home).expect("library should initialize");

    create_skill(
        &home,
        skill("fastapi-backend-review", "# FastAPI Backend Review\n"),
    )
    .expect("skill should create");

    assert!(
        home.join("skills/fastapi-backend-review/SKILL.md")
            .is_file()
    );
    assert!(
        home.join("skills/fastapi-backend-review/metadata.toml")
            .is_file()
    );

    let loaded = get_skill(&home, "fastapi-backend-review").expect("skill should reload");
    assert_eq!(loaded.name, "FastAPI Backend Review");
    assert_eq!(loaded.tags, vec!["fastapi", "review"]);
    assert_eq!(loaded.skill_md, "# FastAPI Backend Review\n");

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn update_skill_rewrites_skill_markdown() {
    let home = test_home("update");
    init_library_at(&home).expect("library should initialize");
    create_skill(&home, skill("fastapi-backend-review", "Old")).expect("skill should create");

    update_skill(&home, skill("fastapi-backend-review", "New")).expect("skill should update");

    let loaded = get_skill(&home, "fastapi-backend-review").expect("skill should reload");
    assert_eq!(loaded.skill_md, "New");

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn update_skill_writes_and_prunes_supporting_files() {
    let home = test_home("supporting-files");
    init_library_at(&home).expect("library should initialize");
    create_skill(
        &home,
        skill("fastapi-backend-review", "# FastAPI Backend Review\n"),
    )
    .expect("skill should create");

    let mut with_files = skill("fastapi-backend-review", "# FastAPI Backend Review\n");
    with_files.files = vec![
        SkillFile {
            path: PathBuf::from("examples/request.md"),
            kind: SkillFileKind::Example,
            content: Some("Example request".to_string()),
        },
        SkillFile {
            path: PathBuf::from("resources/schema.json"),
            kind: SkillFileKind::Resource,
            content: Some("{\"ok\":true}".to_string()),
        },
    ];
    update_skill(&home, with_files).expect("supporting files should update");

    assert_eq!(
        fs::read_to_string(home.join("skills/fastapi-backend-review/examples/request.md"))
            .expect("example should read"),
        "Example request"
    );
    assert_eq!(
        fs::read_to_string(home.join("skills/fastapi-backend-review/resources/schema.json"))
            .expect("resource should read"),
        "{\"ok\":true}"
    );

    let mut without_resource = skill("fastapi-backend-review", "# FastAPI Backend Review\n");
    without_resource.files = vec![SkillFile {
        path: PathBuf::from("examples/request.md"),
        kind: SkillFileKind::Example,
        content: Some("Updated example".to_string()),
    }];
    update_skill(&home, without_resource).expect("supporting files should prune");

    assert_eq!(
        fs::read_to_string(home.join("skills/fastapi-backend-review/examples/request.md"))
            .expect("example should read"),
        "Updated example"
    );
    assert!(
        !home
            .join("skills/fastapi-backend-review/resources/schema.json")
            .exists()
    );

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn skill_supporting_files_can_be_nested() {
    let home = test_home("nested-supporting-files");
    init_library_at(&home).expect("library should initialize");

    let mut with_files = skill("fastapi-backend-review", "# FastAPI Backend Review\n");
    with_files.files = vec![SkillFile {
        path: PathBuf::from("resources/schemas/request.json"),
        kind: SkillFileKind::Resource,
        content: Some("{\"nested\":true}".to_string()),
    }];

    create_skill(&home, with_files).expect("nested supporting file should create");
    let loaded = get_skill(&home, "fastapi-backend-review").expect("skill should load");

    let nested_file = loaded
        .files
        .iter()
        .find(|file| file.path.ends_with("resources/schemas/request.json"))
        .expect("nested resource should be listed");
    assert_eq!(nested_file.content.as_deref(), Some("{\"nested\":true}"));

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn update_skill_preserves_non_text_supporting_files_that_remain_listed() {
    let home = test_home("binary-supporting-files");
    init_library_at(&home).expect("library should initialize");

    let root = home.join("skills/fastapi-backend-review");
    create_skill(
        &home,
        skill("fastapi-backend-review", "# FastAPI Backend Review\n"),
    )
    .expect("skill should create");
    fs::create_dir_all(root.join("resources")).expect("resources dir should create");
    fs::write(root.join("resources/image.bin"), [0_u8, 159, 146, 150])
        .expect("binary resource should write");
    fs::write(root.join("resources/old.txt"), "remove me").expect("stale resource should write");

    let mut loaded = get_skill(&home, "fastapi-backend-review").expect("skill should load");
    assert!(
        loaded
            .files
            .iter()
            .any(|file| file.path.ends_with("resources/image.bin") && file.content.is_none()),
        "non-text resources should stay listed even when content cannot be edited"
    );
    loaded
        .files
        .retain(|file| !file.path.ends_with("resources/old.txt"));
    update_skill(&home, loaded).expect("skill should update");

    assert_eq!(
        fs::read(root.join("resources/image.bin")).expect("binary resource should read"),
        vec![0_u8, 159, 146, 150]
    );
    assert!(!root.join("resources/old.txt").exists());

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn skill_metadata_preserves_custom_fields_while_refreshing_core_fields() {
    let home = test_home("metadata-preserve");
    init_library_at(&home).expect("library should initialize");

    create_skill(
        &home,
        SkillAsset {
            metadata: Some(SkillMetadata {
                raw_toml: "name = \"Stale Name\"\ntags = [\"stale\"]\ncustom_field = \"keep me\"\n"
                    .to_string(),
            }),
            ..skill("fastapi-backend-review", "# FastAPI Backend Review\n")
        },
    )
    .expect("skill should create");

    let metadata = fs::read_to_string(home.join("skills/fastapi-backend-review/metadata.toml"))
        .expect("metadata should be readable");

    assert!(metadata.contains("name = \"FastAPI Backend Review\""));
    assert!(metadata.contains("tags = [\"fastapi\", \"review\"]"));
    assert!(metadata.contains("custom_field = \"keep me\""));
    assert!(!metadata.contains("Stale Name"));

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn list_skills_returns_saved_skill_summaries() {
    let home = test_home("list");
    init_library_at(&home).expect("library should initialize");
    create_skill(
        &home,
        skill("fastapi-backend-review", "# FastAPI Backend Review\n"),
    )
    .expect("skill should create");

    let skills = list_skills(&home).expect("skills should list");

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].id, "fastapi-backend-review");
    assert_eq!(skills[0].name, "FastAPI Backend Review");
    assert!(
        skills[0].updated_at.is_some(),
        "skill summary should expose file modification time"
    );

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn invalid_or_duplicate_skill_does_not_write_partial_directory() {
    let home = test_home("invalid");
    init_library_at(&home).expect("library should initialize");

    let invalid = create_skill(&home, skill("Bad Skill", ""));
    assert!(invalid.is_err());
    assert!(!home.join("skills/Bad Skill").exists());

    create_skill(&home, skill("nginx-debug", "# Nginx Debug\n")).expect("skill should create");
    let duplicate = create_skill(&home, skill("nginx-debug", "# Nginx Debug\n"));
    assert!(duplicate.is_err());

    fs::remove_dir_all(home).expect("test home should be removable");
}

use std::fs;
use std::path::PathBuf;

use flowmint_core::asset::model::{
    AssetDetail, AssetFilter, AssetType, CreateAssetInput, PromptAsset, SkillAsset,
    UpdateAssetInput,
};
use flowmint_core::asset::store::{
    create_asset, delete_asset, get_asset, list_assets, update_asset, validate_asset,
};
use flowmint_core::store::init_library_at;
use flowmint_core::validation::ValidationStatus;

fn test_home(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flowmint-store-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn prompt(id: &str) -> PromptAsset {
    PromptAsset {
        id: id.to_string(),
        name: "FastAPI Review".to_string(),
        description: None,
        tags: vec!["backend".to_string()],
        variables: Vec::new(),
        body: "Review this code.".to_string(),
    }
}

fn skill(id: &str) -> SkillAsset {
    SkillAsset {
        id: id.to_string(),
        name: "Nginx Debug".to_string(),
        description: None,
        tags: vec!["ops".to_string()],
        root_dir: PathBuf::new(),
        skill_md: "# Nginx Debug\n".to_string(),
        metadata: None,
        files: Vec::new(),
    }
}

#[test]
fn asset_store_creates_lists_gets_and_filters_assets() {
    let home = test_home("list-get");
    init_library_at(&home).expect("library should initialize");

    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("fastapi-review"),
            },
        },
    )
    .expect("prompt should create");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Skill {
                asset: skill("nginx-debug"),
            },
        },
    )
    .expect("skill should create");

    let all_assets = list_assets(&home, AssetFilter::default()).expect("assets should list");
    assert_eq!(all_assets.len(), 2);

    let prompt_assets = list_assets(
        &home,
        AssetFilter {
            asset_type: Some(AssetType::Prompt),
            query: None,
        },
    )
    .expect("prompt assets should list");
    assert_eq!(prompt_assets.len(), 1);
    assert_eq!(prompt_assets[0].id, "fastapi-review");

    let asset = get_asset(&home, "skill:nginx-debug").expect("skill should load");
    assert!(matches!(asset, AssetDetail::Skill { .. }));

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn asset_store_updates_validates_and_deletes_assets() {
    let home = test_home("update-delete");
    init_library_at(&home).expect("library should initialize");
    create_asset(
        &home,
        CreateAssetInput {
            asset: AssetDetail::Prompt {
                asset: prompt("fastapi-review"),
            },
        },
    )
    .expect("prompt should create");

    let mut updated = prompt("fastapi-review");
    updated.body = "Updated body.".to_string();
    update_asset(
        &home,
        UpdateAssetInput {
            asset: AssetDetail::Prompt { asset: updated },
        },
    )
    .expect("prompt should update");

    let validation =
        validate_asset(&home, "prompt:fastapi-review").expect("prompt should validate");
    assert_eq!(validation.status, ValidationStatus::Valid);

    delete_asset(&home, "prompt:fastapi-review").expect("prompt should delete");
    assert!(get_asset(&home, "prompt:fastapi-review").is_err());

    fs::remove_dir_all(home).expect("test home should be removable");
}

#[test]
fn asset_store_rejects_unknown_or_unsafe_refs() {
    let home = test_home("bad-ref");
    init_library_at(&home).expect("library should initialize");

    assert!(get_asset(&home, "playbook:daily-review").is_err());
    assert!(get_asset(&home, "prompt:../escape").is_err());
    assert!(delete_asset(&home, "prompt:../escape").is_err());

    fs::remove_dir_all(home).expect("test home should be removable");
}

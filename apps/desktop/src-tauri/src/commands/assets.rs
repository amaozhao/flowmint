use flowmint_core::asset::model::{
    AssetDetail, AssetFilter, AssetSummary, CreateAssetInput, UpdateAssetInput,
};
use flowmint_core::validation::ValidationReport;
use std::path::Path;

fn library_home() -> Result<std::path::PathBuf, String> {
    flowmint_core::store::default_home_dir().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_assets(filter: AssetFilter) -> Result<Vec<AssetSummary>, String> {
    flowmint_core::asset::store::list_assets(&library_home()?, filter)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_asset(asset_ref: String) -> Result<AssetDetail, String> {
    flowmint_core::asset::store::get_asset(&library_home()?, &asset_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_asset(input: CreateAssetInput) -> Result<AssetDetail, String> {
    flowmint_core::asset::store::create_asset(&library_home()?, input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_asset(input: UpdateAssetInput) -> Result<AssetDetail, String> {
    flowmint_core::asset::store::update_asset(&library_home()?, input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_asset(asset_ref: String) -> Result<(), String> {
    flowmint_core::asset::store::delete_asset(&library_home()?, &asset_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn validate_asset(asset_ref: String) -> Result<ValidationReport, String> {
    flowmint_core::asset::store::validate_asset(&library_home()?, &asset_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_asset_folder(asset_ref: String) -> Result<(), String> {
    let library_home = library_home()?;
    let asset = flowmint_core::asset::store::get_asset(&library_home, &asset_ref)
        .map_err(|error| error.to_string())?;
    let folder = match asset {
        AssetDetail::Prompt { .. } => library_home.join("prompts"),
        AssetDetail::Skill { asset } => asset.root_dir,
        AssetDetail::Playbook { .. } => library_home.join("playbooks"),
        AssetDetail::InstructionRule { .. } | AssetDetail::CommandRule { .. } => {
            library_home.join("rules")
        }
    };

    open_path(&folder)
}

#[tauri::command]
pub fn promote_skill_to_playbook(
    skill_id: String,
    playbook_id: String,
) -> Result<AssetDetail, String> {
    flowmint_core::asset::playbook::promote_skill_to_playbook(
        &library_home()?,
        &skill_id,
        &playbook_id,
    )
    .map(|asset| AssetDetail::Playbook { asset })
    .map_err(|error| error.to_string())
}

fn open_path(path: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "macos") {
        std::process::Command::new("open")
    } else if cfg!(target_os = "windows") {
        let mut command = std::process::Command::new("explorer");
        command.arg(path);
        return spawn_open_command(command);
    } else {
        std::process::Command::new("xdg-open")
    };

    command.arg(path);
    spawn_open_command(command)
}

fn spawn_open_command(mut command: std::process::Command) -> Result<(), String> {
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("failed to open asset folder: {error}"))
}

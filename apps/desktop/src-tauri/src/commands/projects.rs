use std::path::PathBuf;

use flowmint_core::project::model::{ProjectDetail, ProjectSummary};
use flowmint_core::sync::plan::SyncScope;

fn library_home() -> Result<PathBuf, String> {
    flowmint_core::store::default_home_dir().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_projects() -> Result<Vec<ProjectSummary>, String> {
    flowmint_core::project::store::list_projects(&library_home()?)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_project(path: PathBuf) -> Result<ProjectDetail, String> {
    flowmint_core::project::store::add_project(&library_home()?, &path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_project(path: PathBuf) -> Result<ProjectDetail, String> {
    flowmint_core::project::store::get_project(&library_home()?, &path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn attach_asset(path: PathBuf, asset_ref: String) -> Result<ProjectDetail, String> {
    flowmint_core::project::store::attach_asset(&library_home()?, &path, &asset_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn detach_asset(path: PathBuf, asset_ref: String) -> Result<ProjectDetail, String> {
    flowmint_core::project::store::detach_asset(&library_home()?, &path, &asset_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn attach_asset_to_profile(
    path: PathBuf,
    target: String,
    scope: SyncScope,
    asset_ref: String,
) -> Result<ProjectDetail, String> {
    flowmint_core::project::store::attach_asset_to_profile(
        &library_home()?,
        &path,
        &target,
        scope,
        &asset_ref,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn detach_asset_from_profile(
    path: PathBuf,
    target: String,
    scope: SyncScope,
    asset_ref: String,
) -> Result<ProjectDetail, String> {
    flowmint_core::project::store::detach_asset_from_profile(
        &library_home()?,
        &path,
        &target,
        scope,
        &asset_ref,
    )
    .map_err(|error| error.to_string())
}

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use flowmint_core::exporters::capabilities::TargetCapabilities;
use flowmint_core::project::global_profiles::GlobalSyncProfiles;
use flowmint_core::sync::apply::SyncApplyResult;
use flowmint_core::sync::plan::{SyncPlan, SyncScope};
use flowmint_core::sync::plan_cache::PlanCache;

#[derive(Default)]
pub struct SyncState {
    plan_cache: Mutex<PlanCache>,
}

fn library_home() -> Result<PathBuf, String> {
    flowmint_core::store::default_home_dir().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn preview_sync(
    state: tauri::State<'_, SyncState>,
    project_path: PathBuf,
    target: String,
    scope: Option<SyncScope>,
) -> Result<SyncPlan, String> {
    let scope = scope.unwrap_or(SyncScope::Project);
    let plan = flowmint_core::exporters::target::preview_target_sync(
        &library_home()?,
        &project_path,
        &target,
        scope,
    )
    .map_err(|error| error.to_string())?;
    state
        .plan_cache
        .lock()
        .map_err(|_| "sync plan cache is unavailable".to_string())?
        .insert(plan.clone());
    Ok(plan)
}

#[tauri::command]
pub fn acknowledge_global_sync_plan(
    state: tauri::State<'_, SyncState>,
    plan_id: String,
    confirmed_paths: Vec<PathBuf>,
) -> Result<(), String> {
    state
        .plan_cache
        .lock()
        .map_err(|_| "sync plan cache is unavailable".to_string())?
        .acknowledge_global_plan(&plan_id, &confirmed_paths)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn apply_sync(
    state: tauri::State<'_, SyncState>,
    plan_id: String,
) -> Result<SyncApplyResult, String> {
    let mut plan_cache = state
        .plan_cache
        .lock()
        .map_err(|_| "sync plan cache is unavailable".to_string())?;
    flowmint_core::sync::apply::apply_sync(&library_home()?, &mut plan_cache, &plan_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_sync_target(path: PathBuf) -> Result<(), String> {
    open_path(&path)
}

#[tauri::command]
pub fn list_target_capabilities() -> Vec<TargetCapabilities> {
    flowmint_core::exporters::capabilities::list_target_capabilities()
}

#[tauri::command]
pub fn list_global_sync_profiles() -> Result<GlobalSyncProfiles, String> {
    flowmint_core::project::global_profiles::load_global_sync_profiles(&library_home()?)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn attach_global_profile_asset(
    target: String,
    asset_ref: String,
) -> Result<GlobalSyncProfiles, String> {
    let library_home = library_home()?;
    flowmint_core::asset::store::get_asset(&library_home, &asset_ref)
        .map_err(|error| error.to_string())?;
    flowmint_core::project::global_profiles::attach_global_profile_asset(
        &library_home,
        &target,
        &asset_ref,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn detach_global_profile_asset(
    target: String,
    asset_ref: String,
) -> Result<GlobalSyncProfiles, String> {
    flowmint_core::project::global_profiles::detach_global_profile_asset(
        &library_home()?,
        &target,
        &asset_ref,
    )
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
        .map_err(|error| format!("failed to open path: {error}"))
}

use std::path::{Path, PathBuf};

mod commands;

use commands::assets::{
    create_asset, delete_asset, get_asset, list_assets, open_asset_folder,
    promote_skill_to_playbook, update_asset, validate_asset,
};
use commands::import::{
    ImportState, apply_import_adoption, preview_import_adoption, scan_import_candidates,
};
use commands::projects::{
    add_project, attach_asset, attach_asset_to_profile, detach_asset, detach_asset_from_profile,
    get_project, list_projects,
};
use commands::sync::{
    SyncState, acknowledge_global_sync_plan, apply_sync, attach_global_profile_asset,
    detach_global_profile_asset, list_global_sync_profiles, list_target_capabilities,
    open_sync_target, preview_sync,
};
use commands::templates::{get_skill_template, list_skill_templates};

#[tauri::command]
fn get_app_state() -> Result<flowmint_core::store::AppState, String> {
    flowmint_core::store::get_app_state().map_err(|error| error.to_string())
}

#[tauri::command]
fn init_library(path: Option<PathBuf>) -> Result<flowmint_core::store::LibraryInfo, String> {
    flowmint_core::store::init_library(path).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_library_folder() -> Result<(), String> {
    let state = flowmint_core::store::get_app_state().map_err(|error| error.to_string())?;
    open_path(&state.library.path)
}

#[tauri::command]
fn pick_directory() -> Result<Option<PathBuf>, String> {
    pick_directory_native()
}

#[tauri::command]
fn rebuild_index() -> Result<flowmint_core::store::diagnostics::IndexSummary, String> {
    let state = flowmint_core::store::get_app_state().map_err(|error| error.to_string())?;
    flowmint_core::store::diagnostics::rebuild_index(&state.library.path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_debug_report() -> Result<PathBuf, String> {
    let state = flowmint_core::store::get_app_state().map_err(|error| error.to_string())?;
    flowmint_core::store::diagnostics::export_debug_report(&state.library.path)
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(SyncState::default())
        .manage(ImportState::default())
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            init_library,
            open_library_folder,
            pick_directory,
            rebuild_index,
            export_debug_report,
            list_assets,
            get_asset,
            create_asset,
            update_asset,
            delete_asset,
            validate_asset,
            open_asset_folder,
            promote_skill_to_playbook,
            list_projects,
            add_project,
            get_project,
            attach_asset,
            detach_asset,
            attach_asset_to_profile,
            detach_asset_from_profile,
            scan_import_candidates,
            preview_import_adoption,
            apply_import_adoption,
            preview_sync,
            acknowledge_global_sync_plan,
            apply_sync,
            open_sync_target,
            list_target_capabilities,
            list_global_sync_profiles,
            attach_global_profile_asset,
            detach_global_profile_asset,
            list_skill_templates,
            get_skill_template
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Flowmint desktop app");
}

fn pick_directory_native() -> Result<Option<PathBuf>, String> {
    if cfg!(target_os = "macos") {
        let mut command = std::process::Command::new("osascript");
        command.args([
            "-e",
            "POSIX path of (choose folder with prompt \"Select a Flowmint directory\")",
        ]);
        return run_directory_picker(command);
    }

    if cfg!(target_os = "windows") {
        let mut command = std::process::Command::new("powershell");
        command.args([
            "-NoProfile",
            "-Command",
            "Add-Type -AssemblyName System.Windows.Forms; $dialog = New-Object System.Windows.Forms.FolderBrowserDialog; if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { $dialog.SelectedPath }",
        ]);
        return run_directory_picker(command);
    }

    let mut zenity = std::process::Command::new("zenity");
    zenity.args(["--file-selection", "--directory"]);
    match run_directory_picker(zenity) {
        Ok(result) => return Ok(result),
        Err(error) if error.contains("No such file") || error.contains("not found") => {}
        Err(error) => return Err(error),
    }

    let mut kdialog = std::process::Command::new("kdialog");
    kdialog.arg("--getexistingdirectory");
    match run_directory_picker(kdialog) {
        Ok(result) => Ok(result),
        Err(error) if error.contains("No such file") || error.contains("not found") => {
            Err("no native directory picker is available; install zenity or kdialog, or paste the path manually".to_string())
        }
        Err(error) => Err(error),
    }
}

fn run_directory_picker(mut command: std::process::Command) -> Result<Option<PathBuf>, String> {
    let output = command
        .output()
        .map_err(|error| format!("failed to open directory picker: {error}"))?;
    if !output.status.success() {
        return Ok(None);
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Ok(None)
    } else {
        Ok(Some(PathBuf::from(path)))
    }
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
        .map_err(|error| format!("failed to open folder: {error}"))
}

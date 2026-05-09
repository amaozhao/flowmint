use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use flowmint_core::import::ImportCandidate;
use flowmint_core::import::adopt::{
    ImportAdoptionPlan, ImportAdoptionSelection, ImportApplyResult,
};
use flowmint_core::sync::plan::SyncScope;

#[derive(Default)]
pub struct ImportState {
    plan_cache: Mutex<HashMap<String, ImportAdoptionPlan>>,
}

fn library_home() -> Result<PathBuf, String> {
    flowmint_core::store::default_home_dir().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn scan_import_candidates(
    project_path: PathBuf,
    target: String,
    scope: SyncScope,
) -> Result<Vec<ImportCandidate>, String> {
    flowmint_core::import::scan_import_candidates(&library_home()?, &project_path, &target, scope)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn preview_import_adoption(
    state: tauri::State<'_, ImportState>,
    project_path: PathBuf,
    target: String,
    scope: SyncScope,
    selections: Vec<ImportAdoptionSelection>,
) -> Result<ImportAdoptionPlan, String> {
    let plan = flowmint_core::import::adopt::preview_import_adoption(
        &library_home()?,
        &project_path,
        &target,
        scope,
        selections,
    )
    .map_err(|error| error.to_string())?;
    state
        .plan_cache
        .lock()
        .map_err(|_| "import plan cache is unavailable".to_string())?
        .insert(plan.plan_id.clone(), plan.clone());
    Ok(plan)
}

#[tauri::command]
pub fn apply_import_adoption(
    state: tauri::State<'_, ImportState>,
    project_path: PathBuf,
    plan_id: String,
) -> Result<ImportApplyResult, String> {
    let plan = state
        .plan_cache
        .lock()
        .map_err(|_| "import plan cache is unavailable".to_string())?
        .remove(&plan_id)
        .ok_or_else(|| format!("import plan not found: {plan_id}"))?;
    flowmint_core::import::adopt::apply_import_adoption(&library_home()?, &project_path, &plan)
        .map_err(|error| error.to_string())
}

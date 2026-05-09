use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{FlowmintError, Result};
use crate::exporters::claude_code::PlannedFile;
use crate::exporters::target::build_target_sync;
use crate::fs_safety::write::write_file_atomic;
use crate::sync::lockfile::write_lockfile_path;
use crate::sync::plan::{SyncOperation, SyncPlan, SyncScope};
use crate::sync::plan_cache::PlanCache;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncApplyResult {
    pub plan_id: String,
    pub written_files: usize,
    pub deleted_files: usize,
    pub noops: usize,
}

pub fn apply_sync(
    library_home: &Path,
    plan_cache: &mut PlanCache,
    plan_id: &str,
) -> Result<SyncApplyResult> {
    let cached_plan =
        plan_cache
            .get(plan_id)
            .cloned()
            .ok_or_else(|| FlowmintError::SyncPlanNotFound {
                plan_id: plan_id.to_string(),
            })?;

    if cached_plan.scope == SyncScope::GlobalUser && !plan_cache.is_global_acknowledged(plan_id) {
        return Err(FlowmintError::GlobalSyncNotAcknowledged {
            plan_id: plan_id.to_string(),
        });
    }

    let rebuilt = build_target_sync(
        library_home,
        &cached_plan.project_path,
        &cached_plan.exporter,
        cached_plan.scope,
    )?;

    if rebuilt.plan().plan_id != cached_plan.plan_id {
        return Err(FlowmintError::SyncPlanChanged {
            plan_id: plan_id.to_string(),
        });
    }

    if !rebuilt.plan().conflicts.is_empty() {
        return Err(FlowmintError::SyncConflicts {
            plan_id: plan_id.to_string(),
            messages: rebuilt
                .plan()
                .conflicts
                .iter()
                .map(|conflict| conflict.message.clone())
                .collect(),
        });
    }

    let file_map = rebuilt
        .files()
        .iter()
        .map(|file| (file.target_path.clone(), file))
        .collect::<HashMap<PathBuf, &PlannedFile>>();
    let result = execute_plan(rebuilt.plan(), &file_map)?;
    let records = rebuilt
        .files()
        .iter()
        .filter_map(|file| file.lock_record.clone())
        .collect::<Vec<_>>();
    write_lockfile_path(
        rebuilt.lockfile_path(),
        &cached_plan.exporter,
        cached_plan.scope,
        &records,
    )?;
    let _ = plan_cache.take(plan_id);

    Ok(result)
}

fn execute_plan(
    plan: &SyncPlan,
    file_map: &HashMap<PathBuf, &PlannedFile>,
) -> Result<SyncApplyResult> {
    let mut written_files = 0;
    let mut deleted_files = 0;
    let mut noops = 0;

    for operation in &plan.operations {
        match operation {
            SyncOperation::CreateDir { target_path } => {
                std::fs::create_dir_all(target_path)
                    .map_err(|source| FlowmintError::io(target_path, source))?;
            }
            SyncOperation::CreateFile { target_path, .. }
            | SyncOperation::UpdateFile { target_path, .. } => {
                let file =
                    file_map
                        .get(target_path)
                        .ok_or_else(|| FlowmintError::SyncPlanChanged {
                            plan_id: plan.plan_id.clone(),
                        })?;
                write_file_atomic(target_path, &file.content)?;
                written_files += 1;
            }
            SyncOperation::DeleteGeneratedFile { target_path, .. } => {
                if target_path.exists() {
                    std::fs::remove_file(target_path)
                        .map_err(|source| FlowmintError::io(target_path, source))?;
                    deleted_files += 1;
                }
            }
            SyncOperation::Noop { .. } => {
                noops += 1;
            }
        }
    }

    Ok(SyncApplyResult {
        plan_id: plan.plan_id.clone(),
        written_files,
        deleted_files,
        noops,
    })
}

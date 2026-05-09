use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::error::{FlowmintError, Result};
use crate::sync::plan::{SyncOperation, SyncPlan, SyncScope};

#[derive(Debug, Default)]
pub struct PlanCache {
    plans: HashMap<String, SyncPlan>,
    acknowledged_global_plans: HashSet<String>,
}

impl PlanCache {
    pub fn insert(&mut self, plan: SyncPlan) {
        self.plans.insert(plan.plan_id.clone(), plan);
    }

    pub fn get(&self, plan_id: &str) -> Option<&SyncPlan> {
        self.plans.get(plan_id)
    }

    pub fn take(&mut self, plan_id: &str) -> Option<SyncPlan> {
        self.acknowledged_global_plans.remove(plan_id);
        self.plans.remove(plan_id)
    }

    pub fn acknowledge_global_plan(
        &mut self,
        plan_id: &str,
        confirmed_paths: &[PathBuf],
    ) -> Result<()> {
        let plan = self
            .plans
            .get(plan_id)
            .ok_or_else(|| FlowmintError::SyncPlanNotFound {
                plan_id: plan_id.to_string(),
            })?;

        if plan.scope != SyncScope::GlobalUser {
            return Err(FlowmintError::GlobalSyncAcknowledgementMismatch {
                plan_id: plan_id.to_string(),
            });
        }

        if operation_path_set(plan) != path_set(confirmed_paths) {
            return Err(FlowmintError::GlobalSyncAcknowledgementMismatch {
                plan_id: plan_id.to_string(),
            });
        }

        self.acknowledged_global_plans.insert(plan_id.to_string());
        Ok(())
    }

    pub fn is_global_acknowledged(&self, plan_id: &str) -> bool {
        self.acknowledged_global_plans.contains(plan_id)
    }
}

fn operation_path_set(plan: &SyncPlan) -> BTreeSet<String> {
    plan.operations
        .iter()
        .filter_map(|operation| match operation {
            SyncOperation::Noop { .. } => None,
            SyncOperation::CreateFile { target_path, .. }
            | SyncOperation::UpdateFile { target_path, .. }
            | SyncOperation::CreateDir { target_path }
            | SyncOperation::DeleteGeneratedFile { target_path, .. } => Some(path_key(target_path)),
        })
        .collect()
}

fn path_set(paths: &[PathBuf]) -> BTreeSet<String> {
    paths.iter().map(|path| path_key(path)).collect()
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

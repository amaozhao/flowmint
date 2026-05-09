use std::path::PathBuf;

use flowmint_core::sync::conflict::{SyncConflict, SyncConflictKind};
use flowmint_core::sync::plan::{SyncOperation, SyncPlan, SyncScope};
use flowmint_core::sync::plan_cache::PlanCache;

#[test]
fn sync_plan_serializes_operations_and_conflicts_for_ui() {
    let plan = SyncPlan::new(
        PathBuf::from("/tmp/project"),
        "claude-code",
        vec![
            SyncOperation::CreateDir {
                target_path: PathBuf::from("/tmp/project/.claude"),
            },
            SyncOperation::CreateFile {
                target_path: PathBuf::from("/tmp/project/.claude/commands/daily-plan.md"),
                content_hash: "hash-new".to_string(),
            },
            SyncOperation::UpdateFile {
                target_path: PathBuf::from("/tmp/project/CLAUDE.md"),
                previous_hash: Some("hash-old".to_string()),
                new_hash: "hash-new".to_string(),
            },
            SyncOperation::DeleteGeneratedFile {
                target_path: PathBuf::from("/tmp/project/.claude/commands/old.md"),
                previous_hash: "hash-old".to_string(),
            },
            SyncOperation::Noop {
                target_path: PathBuf::from("/tmp/project/.flowmint.toml"),
                reason: "Already up to date".to_string(),
            },
        ],
        vec![SyncConflict {
            target_path: PathBuf::from("/tmp/project/CLAUDE.md"),
            kind: SyncConflictKind::UnmanagedTarget,
            message: "Target exists outside Flowmint management.".to_string(),
        }],
    );

    let value = serde_json::to_value(&plan).expect("plan should serialize");

    assert_eq!(value["exporter"], "claude-code");
    assert_eq!(value["operations"][0]["operationType"], "create-dir");
    assert_eq!(value["operations"][1]["operationType"], "create-file");
    assert_eq!(value["operations"][2]["operationType"], "update-file");
    assert_eq!(
        value["operations"][3]["operationType"],
        "delete-generated-file"
    );
    assert_eq!(value["operations"][4]["operationType"], "noop");
    assert_eq!(value["conflicts"][0]["kind"], "unmanaged-target");
}

#[test]
fn sync_plan_id_is_stable_for_same_content_and_changes_when_content_changes() {
    let first = SyncPlan::new(
        PathBuf::from("/tmp/project"),
        "claude-code",
        vec![SyncOperation::Noop {
            target_path: PathBuf::from("/tmp/project/CLAUDE.md"),
            reason: "Already up to date".to_string(),
        }],
        Vec::new(),
    );
    let second = SyncPlan::new(
        PathBuf::from("/tmp/project"),
        "claude-code",
        vec![SyncOperation::Noop {
            target_path: PathBuf::from("/tmp/project/CLAUDE.md"),
            reason: "Already up to date".to_string(),
        }],
        Vec::new(),
    );
    let changed = SyncPlan::new(
        PathBuf::from("/tmp/project"),
        "claude-code",
        vec![SyncOperation::Noop {
            target_path: PathBuf::from("/tmp/project/CLAUDE.md"),
            reason: "Different reason".to_string(),
        }],
        Vec::new(),
    );

    assert_eq!(first.plan_id, second.plan_id);
    assert_ne!(first.plan_id, changed.plan_id);
}

#[test]
fn sync_plan_scope_serializes_and_changes_plan_id() {
    let project = SyncPlan::new_with_scope(
        PathBuf::from("/tmp/project"),
        "claude-code",
        SyncScope::Project,
        vec![SyncOperation::Noop {
            target_path: PathBuf::from("/tmp/project/CLAUDE.md"),
            reason: "Already up to date".to_string(),
        }],
        Vec::new(),
    );
    let global = SyncPlan::new_with_scope(
        PathBuf::from("/tmp/project"),
        "claude-code",
        SyncScope::GlobalUser,
        vec![SyncOperation::Noop {
            target_path: PathBuf::from("/home/example/.claude/CLAUDE.md"),
            reason: "Already up to date".to_string(),
        }],
        Vec::new(),
    );

    assert_eq!(project.scope, SyncScope::Project);
    assert_eq!(global.scope, SyncScope::GlobalUser);
    assert_ne!(project.plan_id, global.plan_id);

    let value = serde_json::to_value(&global).expect("plan should serialize");
    assert_eq!(value["scope"], "global-user");
}

#[test]
fn plan_cache_returns_only_backend_cached_plans_by_id() {
    let mut cache = PlanCache::default();
    let plan = SyncPlan::new(
        PathBuf::from("/tmp/project"),
        "claude-code",
        vec![SyncOperation::CreateDir {
            target_path: PathBuf::from("/tmp/project/.claude"),
        }],
        Vec::new(),
    );
    let plan_id = plan.plan_id.clone();

    cache.insert(plan.clone());

    assert_eq!(cache.get(&plan_id), Some(&plan));
    assert!(cache.get("frontend-supplied-plan").is_none());
    assert_eq!(cache.take(&plan_id), Some(plan));
    assert!(cache.get(&plan_id).is_none());
}

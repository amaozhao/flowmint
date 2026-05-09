use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::sync::conflict::{SyncConflict, SyncConflictKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncScope {
    Project,
    GlobalUser,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPlan {
    pub plan_id: String,
    pub project_path: PathBuf,
    pub exporter: String,
    pub scope: SyncScope,
    pub operations: Vec<SyncOperation>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operationType", rename_all = "kebab-case")]
pub enum SyncOperation {
    CreateFile {
        target_path: PathBuf,
        content_hash: String,
    },
    UpdateFile {
        target_path: PathBuf,
        previous_hash: Option<String>,
        new_hash: String,
    },
    CreateDir {
        target_path: PathBuf,
    },
    DeleteGeneratedFile {
        target_path: PathBuf,
        previous_hash: String,
    },
    Noop {
        target_path: PathBuf,
        reason: String,
    },
}

impl SyncPlan {
    pub fn new(
        project_path: PathBuf,
        exporter: impl Into<String>,
        operations: Vec<SyncOperation>,
        conflicts: Vec<SyncConflict>,
    ) -> Self {
        Self::new_with_scope(
            project_path,
            exporter,
            SyncScope::Project,
            operations,
            conflicts,
        )
    }

    pub fn new_with_scope(
        project_path: PathBuf,
        exporter: impl Into<String>,
        scope: SyncScope,
        operations: Vec<SyncOperation>,
        conflicts: Vec<SyncConflict>,
    ) -> Self {
        let exporter = exporter.into();
        let plan_id = build_plan_id(&project_path, &exporter, scope, &operations, &conflicts);

        Self {
            plan_id,
            project_path,
            exporter,
            scope,
            operations,
            conflicts,
        }
    }
}

fn build_plan_id(
    project_path: &Path,
    exporter: &str,
    scope: SyncScope,
    operations: &[SyncOperation],
    conflicts: &[SyncConflict],
) -> String {
    let mut fingerprint = String::new();
    push_field(&mut fingerprint, "project", &project_path.to_string_lossy());
    push_field(&mut fingerprint, "exporter", exporter);
    push_field(&mut fingerprint, "scope", scope.fingerprint());

    for operation in operations {
        push_field(&mut fingerprint, "operation", &operation.fingerprint());
    }

    for conflict in conflicts {
        push_field(&mut fingerprint, "conflict", &conflict.fingerprint());
    }

    format!("plan-{:016x}", fnv1a64(fingerprint.as_bytes()))
}

impl SyncScope {
    fn fingerprint(self) -> &'static str {
        match self {
            SyncScope::Project => "project",
            SyncScope::GlobalUser => "global-user",
        }
    }
}

fn push_field(target: &mut String, key: &str, value: &str) {
    target.push_str(key);
    target.push('=');
    target.push_str(&value.len().to_string());
    target.push(':');
    target.push_str(value);
    target.push('\n');
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

trait SyncOperationFingerprint {
    fn fingerprint(&self) -> String;
}

impl SyncOperationFingerprint for SyncOperation {
    fn fingerprint(&self) -> String {
        match self {
            SyncOperation::CreateFile {
                target_path,
                content_hash,
            } => format!(
                "create-file|{}|{}",
                target_path.to_string_lossy(),
                content_hash
            ),
            SyncOperation::UpdateFile {
                target_path,
                previous_hash,
                new_hash,
            } => format!(
                "update-file|{}|{}|{}",
                target_path.to_string_lossy(),
                previous_hash.as_deref().unwrap_or_default(),
                new_hash
            ),
            SyncOperation::CreateDir { target_path } => {
                format!("create-dir|{}", target_path.to_string_lossy())
            }
            SyncOperation::DeleteGeneratedFile {
                target_path,
                previous_hash,
            } => format!(
                "delete-generated-file|{}|{}",
                target_path.to_string_lossy(),
                previous_hash
            ),
            SyncOperation::Noop {
                target_path,
                reason,
            } => format!("noop|{}|{}", target_path.to_string_lossy(), reason),
        }
    }
}

trait SyncConflictFingerprint {
    fn fingerprint(&self) -> String;
}

impl SyncConflictFingerprint for SyncConflict {
    fn fingerprint(&self) -> String {
        format!(
            "{}|{}|{}",
            self.kind.fingerprint(),
            self.target_path.to_string_lossy(),
            self.message
        )
    }
}

trait SyncConflictKindFingerprint {
    fn fingerprint(self) -> &'static str;
}

impl SyncConflictKindFingerprint for SyncConflictKind {
    fn fingerprint(self) -> &'static str {
        match self {
            SyncConflictKind::UnmanagedTarget => "unmanaged-target",
            SyncConflictKind::ModifiedGeneratedFile => "modified-generated-file",
            SyncConflictKind::IncompleteManagedBlock => "incomplete-managed-block",
            SyncConflictKind::UnsafeSymlink => "unsafe-symlink",
            SyncConflictKind::UnsafeAssetId => "unsafe-asset-id",
            SyncConflictKind::OutputNotWritable => "output-not-writable",
            SyncConflictKind::MissingAsset => "missing-asset",
            SyncConflictKind::UnsupportedMapping => "unsupported-mapping",
        }
    }
}

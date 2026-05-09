use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncConflictKind {
    UnmanagedTarget,
    ModifiedGeneratedFile,
    IncompleteManagedBlock,
    UnsafeSymlink,
    UnsafeAssetId,
    OutputNotWritable,
    MissingAsset,
    UnsupportedMapping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflict {
    pub target_path: PathBuf,
    pub kind: SyncConflictKind,
    pub message: String,
}

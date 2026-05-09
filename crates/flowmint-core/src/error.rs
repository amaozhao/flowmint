use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub enum FlowmintError {
    HomeDirectoryUnavailable,
    AssetNotFound {
        asset_ref: String,
    },
    InvalidAsset {
        messages: Vec<String>,
    },
    InvalidPromptFile {
        path: PathBuf,
        message: String,
    },
    InvalidProjectManifest {
        path: PathBuf,
        message: String,
    },
    SyncPlanNotFound {
        plan_id: String,
    },
    SyncPlanChanged {
        plan_id: String,
    },
    SyncConflicts {
        plan_id: String,
        messages: Vec<String>,
    },
    GlobalSyncNotAcknowledged {
        plan_id: String,
    },
    GlobalSyncAcknowledgementMismatch {
        plan_id: String,
    },
    UnsupportedSyncTarget {
        target: String,
    },
    UnsupportedSyncScope {
        target: String,
        scope: String,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl FlowmintError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

impl Display for FlowmintError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HomeDirectoryUnavailable => write!(f, "home directory is unavailable"),
            Self::AssetNotFound { asset_ref } => write!(f, "asset not found: {asset_ref}"),
            Self::InvalidAsset { messages } => write!(f, "invalid asset: {}", messages.join("; ")),
            Self::InvalidPromptFile { path, message } => {
                write!(f, "invalid prompt file at {}: {message}", path.display())
            }
            Self::InvalidProjectManifest { path, message } => {
                write!(
                    f,
                    "invalid project manifest at {}: {message}",
                    path.display()
                )
            }
            Self::SyncPlanNotFound { plan_id } => write!(f, "sync plan not found: {plan_id}"),
            Self::SyncPlanChanged { plan_id } => {
                write!(f, "sync plan changed before apply: {plan_id}")
            }
            Self::SyncConflicts { plan_id, messages } => {
                write!(
                    f,
                    "sync plan {plan_id} has conflicts: {}",
                    messages.join("; ")
                )
            }
            Self::GlobalSyncNotAcknowledged { plan_id } => {
                write!(
                    f,
                    "global sync plan {plan_id} has not been explicitly acknowledged"
                )
            }
            Self::GlobalSyncAcknowledgementMismatch { plan_id } => {
                write!(
                    f,
                    "global sync acknowledgement does not match plan {plan_id}"
                )
            }
            Self::UnsupportedSyncTarget { target } => {
                write!(f, "unsupported sync target: {target}")
            }
            Self::UnsupportedSyncScope { target, scope } => {
                write!(f, "unsupported sync scope for {target}: {scope}")
            }
            Self::Io { path, source } => {
                write!(
                    f,
                    "filesystem operation failed at {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for FlowmintError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::HomeDirectoryUnavailable
            | Self::AssetNotFound { .. }
            | Self::InvalidAsset { .. }
            | Self::InvalidPromptFile { .. }
            | Self::InvalidProjectManifest { .. }
            | Self::SyncPlanNotFound { .. }
            | Self::SyncPlanChanged { .. }
            | Self::SyncConflicts { .. }
            | Self::GlobalSyncNotAcknowledged { .. }
            | Self::GlobalSyncAcknowledgementMismatch { .. }
            | Self::UnsupportedSyncTarget { .. }
            | Self::UnsupportedSyncScope { .. } => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, FlowmintError>;

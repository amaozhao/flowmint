use std::path::Path;

use crate::error::{FlowmintError, Result};
use crate::exporters::claude_code::{
    ClaudeCodeSync, PlannedFile, build_claude_code_sync_for_scope,
};
use crate::exporters::codex::{CodexSync, build_codex_sync_for_scope};
use crate::exporters::gemini_cli::{GeminiCliSync, build_gemini_cli_sync_for_scope};
use crate::sync::plan::{SyncPlan, SyncScope};

pub fn preview_target_sync(
    library_home: &Path,
    project_dir: &Path,
    target_id: &str,
    scope: SyncScope,
) -> Result<SyncPlan> {
    Ok(build_target_sync(library_home, project_dir, target_id, scope)?.into_plan())
}

pub(crate) fn build_target_sync(
    library_home: &Path,
    project_dir: &Path,
    target_id: &str,
    scope: SyncScope,
) -> Result<TargetSync> {
    match target_id {
        "claude-code" => Ok(TargetSync::ClaudeCode(build_claude_code_sync_for_scope(
            library_home,
            project_dir,
            scope,
        )?)),
        "codex" => Ok(TargetSync::Codex(build_codex_sync_for_scope(
            library_home,
            project_dir,
            scope,
        )?)),
        "gemini-cli" => Ok(TargetSync::GeminiCli(build_gemini_cli_sync_for_scope(
            library_home,
            project_dir,
            scope,
        )?)),
        _ => Err(FlowmintError::UnsupportedSyncTarget {
            target: target_id.to_string(),
        }),
    }
}

pub(crate) enum TargetSync {
    ClaudeCode(ClaudeCodeSync),
    Codex(CodexSync),
    GeminiCli(GeminiCliSync),
}

impl TargetSync {
    pub(crate) fn plan(&self) -> &SyncPlan {
        match self {
            TargetSync::ClaudeCode(sync) => &sync.plan,
            TargetSync::Codex(sync) => &sync.plan,
            TargetSync::GeminiCli(sync) => &sync.plan,
        }
    }

    pub(crate) fn files(&self) -> &[PlannedFile] {
        match self {
            TargetSync::ClaudeCode(sync) => &sync.files,
            TargetSync::Codex(sync) => &sync.files,
            TargetSync::GeminiCli(sync) => &sync.files,
        }
    }

    pub(crate) fn lockfile_path(&self) -> &Path {
        match self {
            TargetSync::ClaudeCode(sync) => &sync.lockfile_path,
            TargetSync::Codex(sync) => &sync.lockfile_path,
            TargetSync::GeminiCli(sync) => &sync.lockfile_path,
        }
    }

    fn into_plan(self) -> SyncPlan {
        match self {
            TargetSync::ClaudeCode(sync) => sync.plan,
            TargetSync::Codex(sync) => sync.plan,
            TargetSync::GeminiCli(sync) => sync.plan,
        }
    }
}

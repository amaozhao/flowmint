use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::asset::model::{AssetFilter, AssetType};
use crate::asset::store::list_assets;
use crate::error::Result;
use crate::project::store::list_projects;
use crate::store::home::{LibraryInfo, get_app_state_for_home};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexSummary {
    pub prompt_count: usize,
    pub skill_count: usize,
    pub playbook_skill_count: usize,
    pub project_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugReport {
    pub version: String,
    pub library: LibraryInfo,
    pub recent_projects: Vec<PathBuf>,
    pub index: IndexSummary,
}

pub fn rebuild_index(library_home: &Path) -> Result<IndexSummary> {
    let prompts = list_assets(
        library_home,
        AssetFilter {
            asset_type: Some(AssetType::Prompt),
            query: None,
        },
    )?;
    let skills = list_assets(
        library_home,
        AssetFilter {
            asset_type: Some(AssetType::Skill),
            query: None,
        },
    )?;
    let projects = list_projects(library_home)?;

    Ok(IndexSummary {
        prompt_count: prompts.len(),
        playbook_skill_count: skills
            .iter()
            .filter(|asset| asset.tags.iter().any(|tag| tag == "playbook"))
            .count(),
        skill_count: skills.len(),
        project_count: projects.len(),
    })
}

pub fn build_debug_report(library_home: &Path) -> Result<DebugReport> {
    let app_state = get_app_state_for_home(library_home)?;
    Ok(DebugReport {
        version: app_state.version,
        library: app_state.library,
        recent_projects: app_state.recent_projects,
        index: rebuild_index(library_home)?,
    })
}

pub fn export_debug_report(library_home: &Path) -> Result<PathBuf> {
    let report = build_debug_report(library_home)?;
    let path = library_home.join("cache").join("debug-report.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|source| crate::error::FlowmintError::io(parent, source))?;
    }
    let content = serde_json::to_string_pretty(&report).map_err(|error| {
        crate::error::FlowmintError::InvalidAsset {
            messages: vec![format!("debug report could not serialize: {error}")],
        }
    })?;
    std::fs::write(&path, content)
        .map_err(|source| crate::error::FlowmintError::io(&path, source))?;
    Ok(path)
}

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::asset::model::AssetSummary;
use crate::project::manifest::ProjectManifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectAssetType {
    Prompt,
    Skill,
    Playbook,
    InstructionRule,
    CommandRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttachedAssetState {
    Available,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedAsset {
    pub asset_type: ProjectAssetType,
    pub id: String,
    pub asset_ref: String,
    pub state: AttachedAssetState,
    pub summary: Option<AssetSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub path: PathBuf,
    pub name: String,
    pub initialized: bool,
    pub attached_prompts: usize,
    pub attached_skills: usize,
    pub attached_assets: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub path: PathBuf,
    pub initialized: bool,
    pub manifest: ProjectManifest,
    pub attached_assets: Vec<AttachedAsset>,
}

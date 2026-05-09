use std::path::Path;

use crate::asset::id::is_safe_asset_id;
use crate::asset::model::{AssetFilter, AssetSummary, AssetType};
use crate::asset::store::{get_asset, list_assets};
use crate::error::{FlowmintError, Result};
use crate::project::manifest::{
    attach_export_asset, attach_export_asset_to_profile, detach_export_asset,
    detach_export_asset_from_profile, init_project_manifest, load_project_manifest, manifest_path,
};
use crate::project::model::{
    AttachedAsset, AttachedAssetState, ProjectAssetType, ProjectDetail, ProjectSummary,
};
use crate::project::recent::{add_recent_project, list_recent_projects};
use crate::sync::plan::SyncScope;

pub fn list_projects(library_home: &Path) -> Result<Vec<ProjectSummary>> {
    list_recent_projects(library_home)?
        .into_iter()
        .map(|project_path| project_summary(&project_path))
        .collect()
}

pub fn add_project(library_home: &Path, project_dir: &Path) -> Result<ProjectDetail> {
    init_project_manifest(project_dir)?;
    add_recent_project(library_home, project_dir)?;
    get_project(library_home, project_dir)
}

pub fn get_project(library_home: &Path, project_dir: &Path) -> Result<ProjectDetail> {
    let initialized = manifest_path(project_dir).is_file();
    let manifest = load_project_manifest(project_dir)?;
    let attached_assets = resolve_attached_assets(library_home, &manifest)?;

    Ok(ProjectDetail {
        path: project_dir.to_path_buf(),
        initialized,
        manifest,
        attached_assets,
    })
}

pub fn attach_asset(
    library_home: &Path,
    project_dir: &Path,
    asset_ref: &str,
) -> Result<ProjectDetail> {
    get_asset(library_home, asset_ref)?;
    attach_export_asset(project_dir, asset_ref)?;

    get_project(library_home, project_dir)
}

pub fn detach_asset(
    library_home: &Path,
    project_dir: &Path,
    asset_ref: &str,
) -> Result<ProjectDetail> {
    parse_project_asset_ref(asset_ref)?;
    detach_export_asset(project_dir, asset_ref)?;

    get_project(library_home, project_dir)
}

pub fn attach_asset_to_profile(
    library_home: &Path,
    project_dir: &Path,
    target: &str,
    scope: SyncScope,
    asset_ref: &str,
) -> Result<ProjectDetail> {
    get_asset(library_home, asset_ref)?;
    attach_export_asset_to_profile(project_dir, target, scope, asset_ref)?;

    get_project(library_home, project_dir)
}

pub fn detach_asset_from_profile(
    library_home: &Path,
    project_dir: &Path,
    target: &str,
    scope: SyncScope,
    asset_ref: &str,
) -> Result<ProjectDetail> {
    parse_project_asset_ref(asset_ref)?;
    detach_export_asset_from_profile(project_dir, target, scope, asset_ref)?;

    get_project(library_home, project_dir)
}

fn project_summary(project_dir: &Path) -> Result<ProjectSummary> {
    let initialized = manifest_path(project_dir).is_file();
    let manifest = load_project_manifest(project_dir)?;

    Ok(ProjectSummary {
        path: project_dir.to_path_buf(),
        name: manifest.project.name,
        initialized,
        attached_prompts: manifest.attach.prompts.len(),
        attached_skills: manifest.attach.skills.len(),
        attached_assets: manifest
            .exports
            .iter()
            .map(export_profile_attachment_count)
            .sum(),
    })
}

fn export_profile_attachment_count(
    profile: &crate::project::manifest::ProjectExportProfile,
) -> usize {
    profile.prompts.len()
        + profile.skills.len()
        + profile.playbooks.len()
        + profile.instruction_rules.len()
        + profile.command_rules.len()
}

fn resolve_attached_assets(
    library_home: &Path,
    manifest: &crate::project::manifest::ProjectManifest,
) -> Result<Vec<AttachedAsset>> {
    let all_summaries = list_assets(library_home, AssetFilter::default())?;
    let empty = Vec::new();
    let profile = manifest.exports.first();

    let mut attached_assets = Vec::new();
    attached_assets.extend(
        profile
            .map(|profile| &profile.prompts)
            .unwrap_or(&manifest.attach.prompts)
            .iter()
            .map(|id| attached_asset(ProjectAssetType::Prompt, id, &all_summaries)),
    );
    attached_assets.extend(
        profile
            .map(|profile| &profile.skills)
            .unwrap_or(&manifest.attach.skills)
            .iter()
            .map(|id| attached_asset(ProjectAssetType::Skill, id, &all_summaries)),
    );
    attached_assets.extend(
        profile
            .map(|profile| &profile.playbooks)
            .unwrap_or(&empty)
            .iter()
            .map(|id| attached_asset(ProjectAssetType::Playbook, id, &all_summaries)),
    );
    attached_assets.extend(
        profile
            .map(|profile| &profile.instruction_rules)
            .unwrap_or(&empty)
            .iter()
            .map(|id| attached_asset(ProjectAssetType::InstructionRule, id, &all_summaries)),
    );
    attached_assets.extend(
        profile
            .map(|profile| &profile.command_rules)
            .unwrap_or(&empty)
            .iter()
            .map(|id| attached_asset(ProjectAssetType::CommandRule, id, &all_summaries)),
    );

    Ok(attached_assets)
}

fn attached_asset(
    asset_type: ProjectAssetType,
    id: &str,
    summaries: &[AssetSummary],
) -> AttachedAsset {
    let summary = summaries
        .iter()
        .find(|summary| summary.id == id && summary.asset_type == asset_type.asset_type())
        .cloned();

    AttachedAsset {
        asset_type,
        id: id.to_string(),
        asset_ref: format!("{}:{id}", asset_type.ref_prefix()),
        state: if summary.is_some() {
            AttachedAssetState::Available
        } else {
            AttachedAssetState::Missing
        },
        summary,
    }
}

fn parse_project_asset_ref(asset_ref: &str) -> Result<()> {
    let Some((asset_type, id)) = asset_ref.split_once(':') else {
        return Err(invalid_asset_ref(asset_ref));
    };

    if !is_safe_asset_id(id) {
        return Err(invalid_asset_ref(asset_ref));
    }

    match asset_type {
        "prompt" | "skill" | "playbook" | "instruction-rule" | "command-rule" => {}
        _ => return Err(invalid_asset_ref(asset_ref)),
    }

    Ok(())
}

fn invalid_asset_ref(asset_ref: &str) -> FlowmintError {
    FlowmintError::InvalidAsset {
        messages: vec![format!(
            "asset_ref '{asset_ref}' must use prompt:<id>, skill:<id>, playbook:<id>, instruction-rule:<id>, or command-rule:<id>"
        )],
    }
}

trait ProjectAssetTypeRef {
    fn ref_prefix(self) -> &'static str;
    fn asset_type(self) -> AssetType;
}

impl ProjectAssetTypeRef for ProjectAssetType {
    fn ref_prefix(self) -> &'static str {
        match self {
            ProjectAssetType::Prompt => "prompt",
            ProjectAssetType::Skill => "skill",
            ProjectAssetType::Playbook => "playbook",
            ProjectAssetType::InstructionRule => "instruction-rule",
            ProjectAssetType::CommandRule => "command-rule",
        }
    }

    fn asset_type(self) -> AssetType {
        match self {
            ProjectAssetType::Prompt => AssetType::Prompt,
            ProjectAssetType::Skill => AssetType::Skill,
            ProjectAssetType::Playbook => AssetType::Playbook,
            ProjectAssetType::InstructionRule => AssetType::InstructionRule,
            ProjectAssetType::CommandRule => AssetType::CommandRule,
        }
    }
}

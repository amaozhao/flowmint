use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::asset::id::is_safe_asset_id;
use crate::error::{FlowmintError, Result};
use crate::sync::plan::SyncScope;

const MANIFEST_FILE: &str = ".flowmint.toml";
const DEFAULT_EXPORT_TARGET: &str = "claude-code";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub project: ProjectMetadata,
    pub export: ProjectExport,
    pub attach: ProjectAttachments,
    pub exports: Vec<ProjectExportProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExport {
    pub target: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAttachments {
    pub prompts: Vec<String>,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExportProfile {
    pub target: String,
    pub scope: SyncScope,
    pub prompts: Vec<String>,
    pub skills: Vec<String>,
    pub playbooks: Vec<String>,
    pub instruction_rules: Vec<String>,
    pub command_rules: Vec<String>,
}

pub fn init_project_manifest(project_dir: &Path) -> Result<ProjectManifest> {
    let path = manifest_path(project_dir);
    if path.exists() {
        return load_project_manifest(project_dir);
    }

    let manifest = default_manifest(project_dir);
    write_project_manifest(project_dir, &manifest)?;
    Ok(manifest)
}

pub fn load_project_manifest(project_dir: &Path) -> Result<ProjectManifest> {
    let path = manifest_path(project_dir);
    if !path.exists() {
        return Ok(default_manifest(project_dir));
    }

    let content =
        std::fs::read_to_string(&path).map_err(|source| FlowmintError::io(&path, source))?;
    parse_project_manifest(project_dir, &path, &content)
}

pub fn write_project_manifest(project_dir: &Path, manifest: &ProjectManifest) -> Result<()> {
    if !project_dir.exists() {
        std::fs::create_dir_all(project_dir)
            .map_err(|source| FlowmintError::io(project_dir, source))?;
    }

    let path = manifest_path(project_dir);
    std::fs::write(&path, render_project_manifest(manifest))
        .map_err(|source| FlowmintError::io(path, source))
}

pub fn attach_prompt(project_dir: &Path, prompt_id: &str) -> Result<ProjectManifest> {
    update_attachment(
        project_dir,
        AttachmentKind::Prompt,
        prompt_id,
        AttachmentAction::Attach,
    )
}

pub fn attach_skill(project_dir: &Path, skill_id: &str) -> Result<ProjectManifest> {
    update_attachment(
        project_dir,
        AttachmentKind::Skill,
        skill_id,
        AttachmentAction::Attach,
    )
}

pub fn attach_export_asset(project_dir: &Path, asset_ref: &str) -> Result<ProjectManifest> {
    update_export_attachment(project_dir, asset_ref, AttachmentAction::Attach)
}

pub fn detach_export_asset(project_dir: &Path, asset_ref: &str) -> Result<ProjectManifest> {
    update_export_attachment(project_dir, asset_ref, AttachmentAction::Detach)
}

pub fn attach_export_asset_to_profile(
    project_dir: &Path,
    target: &str,
    scope: SyncScope,
    asset_ref: &str,
) -> Result<ProjectManifest> {
    update_export_profile_attachment(
        project_dir,
        target,
        scope,
        asset_ref,
        AttachmentAction::Attach,
    )
}

pub fn detach_export_asset_from_profile(
    project_dir: &Path,
    target: &str,
    scope: SyncScope,
    asset_ref: &str,
) -> Result<ProjectManifest> {
    update_export_profile_attachment(
        project_dir,
        target,
        scope,
        asset_ref,
        AttachmentAction::Detach,
    )
}

pub fn detach_prompt(project_dir: &Path, prompt_id: &str) -> Result<ProjectManifest> {
    update_attachment(
        project_dir,
        AttachmentKind::Prompt,
        prompt_id,
        AttachmentAction::Detach,
    )
}

pub fn detach_skill(project_dir: &Path, skill_id: &str) -> Result<ProjectManifest> {
    update_attachment(
        project_dir,
        AttachmentKind::Skill,
        skill_id,
        AttachmentAction::Detach,
    )
}

pub fn manifest_path(project_dir: &Path) -> PathBuf {
    project_dir.join(MANIFEST_FILE)
}

fn parse_project_manifest(
    project_dir: &Path,
    path: &Path,
    content: &str,
) -> Result<ProjectManifest> {
    let mut section = Section::None;
    let mut manifest = default_manifest(project_dir);
    let mut saw_v2_exports = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line == "[[exports]]" {
            if !saw_v2_exports {
                manifest.exports.clear();
                saw_v2_exports = true;
            }
            manifest.exports.push(default_export_profile());
            section = Section::ExportProfile(manifest.exports.len() - 1);
            continue;
        }

        if let Some(section_name) = line
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
        {
            section = match section_name.trim() {
                "project" => Section::Project,
                "export" => Section::Export,
                "attach" => Section::Attach,
                _ => Section::None,
            };
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        match (section, key) {
            (Section::Project, "name") => {
                manifest.project.name = parse_string(value)
                    .ok_or_else(|| invalid_manifest(path, "project.name must be a string"))?;
            }
            (Section::Export, "target") => {
                manifest.export.target = parse_string(value)
                    .ok_or_else(|| invalid_manifest(path, "export.target must be a string"))?;
            }
            (Section::Attach, "prompts") => {
                manifest.attach.prompts = parse_string_array(value).ok_or_else(|| {
                    invalid_manifest(path, "attach.prompts must be a string array")
                })?;
            }
            (Section::Attach, "skills") => {
                manifest.attach.skills = parse_string_array(value).ok_or_else(|| {
                    invalid_manifest(path, "attach.skills must be a string array")
                })?;
            }
            (Section::ExportProfile(index), "target") => {
                manifest.exports[index].target = parse_string(value)
                    .ok_or_else(|| invalid_manifest(path, "exports.target must be a string"))?;
            }
            (Section::ExportProfile(index), "scope") => {
                let value = parse_string(value)
                    .ok_or_else(|| invalid_manifest(path, "exports.scope must be a string"))?;
                manifest.exports[index].scope = parse_scope(&value).ok_or_else(|| {
                    invalid_manifest(path, "exports.scope must be project or global-user")
                })?;
            }
            (Section::ExportProfile(index), "prompts") => {
                manifest.exports[index].prompts = parse_string_array(value).ok_or_else(|| {
                    invalid_manifest(path, "exports.prompts must be a string array")
                })?;
            }
            (Section::ExportProfile(index), "skills") => {
                manifest.exports[index].skills = parse_string_array(value).ok_or_else(|| {
                    invalid_manifest(path, "exports.skills must be a string array")
                })?;
            }
            (Section::ExportProfile(index), "playbooks") => {
                manifest.exports[index].playbooks = parse_string_array(value).ok_or_else(|| {
                    invalid_manifest(path, "exports.playbooks must be a string array")
                })?;
            }
            (Section::ExportProfile(index), "instruction_rules") => {
                manifest.exports[index].instruction_rules =
                    parse_string_array(value).ok_or_else(|| {
                        invalid_manifest(path, "exports.instruction_rules must be a string array")
                    })?;
            }
            (Section::ExportProfile(index), "command_rules") => {
                manifest.exports[index].command_rules =
                    parse_string_array(value).ok_or_else(|| {
                        invalid_manifest(path, "exports.command_rules must be a string array")
                    })?;
            }
            _ => {}
        }
    }

    if manifest.export.target.trim().is_empty() {
        manifest.export.target = DEFAULT_EXPORT_TARGET.to_string();
    }

    if saw_v2_exports {
        if manifest.exports.is_empty() {
            manifest.exports.push(default_export_profile());
        }
        sync_legacy_fields_from_first_export(&mut manifest);
    } else {
        sync_exports_from_legacy_fields(&mut manifest);
    }

    Ok(manifest)
}

fn render_project_manifest(manifest: &ProjectManifest) -> String {
    if is_v1_compatible(manifest) {
        return format!(
            "[project]\nname = \"{}\"\n\n[export]\ntarget = \"{}\"\n\n[attach]\nprompts = [{}]\nskills = [{}]\n",
            escape_toml_string(&manifest.project.name),
            escape_toml_string(&manifest.export.target),
            render_array(&manifest.attach.prompts),
            render_array(&manifest.attach.skills)
        );
    }

    let mut content = format!(
        "[project]\nname = \"{}\"\n",
        escape_toml_string(&manifest.project.name)
    );
    for profile in normalized_exports(manifest) {
        content.push_str("\n[[exports]]\n");
        content.push_str(&format!(
            "target = \"{}\"\n",
            escape_toml_string(&profile.target)
        ));
        content.push_str(&format!("scope = \"{}\"\n", render_scope(profile.scope)));
        content.push_str(&format!("prompts = [{}]\n", render_array(&profile.prompts)));
        content.push_str(&format!("skills = [{}]\n", render_array(&profile.skills)));
        content.push_str(&format!(
            "playbooks = [{}]\n",
            render_array(&profile.playbooks)
        ));
        content.push_str(&format!(
            "instruction_rules = [{}]\n",
            render_array(&profile.instruction_rules)
        ));
        content.push_str(&format!(
            "command_rules = [{}]\n",
            render_array(&profile.command_rules)
        ));
    }
    content
}

fn default_manifest(project_dir: &Path) -> ProjectManifest {
    ProjectManifest {
        project: ProjectMetadata {
            name: project_dir
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "Untitled Project".to_string()),
        },
        export: ProjectExport {
            target: DEFAULT_EXPORT_TARGET.to_string(),
        },
        attach: ProjectAttachments::default(),
        exports: vec![default_export_profile()],
    }
}

fn update_attachment(
    project_dir: &Path,
    kind: AttachmentKind,
    asset_id: &str,
    action: AttachmentAction,
) -> Result<ProjectManifest> {
    if !is_safe_asset_id(asset_id) {
        return Err(FlowmintError::InvalidAsset {
            messages: vec![
                "attached asset id must use only a-z, 0-9, hyphen, or underscore".to_string(),
            ],
        });
    }

    let mut manifest = init_project_manifest(project_dir)?;
    let values = match kind {
        AttachmentKind::Prompt => &mut manifest.attach.prompts,
        AttachmentKind::Skill => &mut manifest.attach.skills,
    };

    match action {
        AttachmentAction::Attach => {
            if !values.iter().any(|value| value == asset_id) {
                values.push(asset_id.to_string());
            }
        }
        AttachmentAction::Detach => {
            values.retain(|value| value != asset_id);
        }
    }

    sync_exports_from_legacy_fields(&mut manifest);
    write_project_manifest(project_dir, &manifest)?;
    Ok(manifest)
}

fn parse_string(value: &str) -> Option<String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| value.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn parse_string_array(value: &str) -> Option<Vec<String>> {
    let values = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))?;

    let mut parsed = Vec::new();
    for item in values.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        parsed.push(parse_string(item)?);
    }
    Some(parsed)
}

fn parse_scope(value: &str) -> Option<SyncScope> {
    match value {
        "project" => Some(SyncScope::Project),
        "global-user" => Some(SyncScope::GlobalUser),
        _ => None,
    }
}

fn render_scope(scope: SyncScope) -> &'static str {
    match scope {
        SyncScope::Project => "project",
        SyncScope::GlobalUser => "global-user",
    }
}

fn render_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", escape_toml_string(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn default_export_profile() -> ProjectExportProfile {
    ProjectExportProfile {
        target: DEFAULT_EXPORT_TARGET.to_string(),
        scope: SyncScope::Project,
        prompts: Vec::new(),
        skills: Vec::new(),
        playbooks: Vec::new(),
        instruction_rules: Vec::new(),
        command_rules: Vec::new(),
    }
}

fn sync_legacy_fields_from_first_export(manifest: &mut ProjectManifest) {
    let Some(first) = manifest.exports.first() else {
        manifest.exports.push(default_export_profile());
        sync_legacy_fields_from_first_export(manifest);
        return;
    };
    manifest.export.target = first.target.clone();
    manifest.attach.prompts = first.prompts.clone();
    manifest.attach.skills = first.skills.clone();
}

fn sync_exports_from_legacy_fields(manifest: &mut ProjectManifest) {
    if manifest.exports.is_empty() {
        manifest.exports.push(default_export_profile());
    }
    let first = &mut manifest.exports[0];
    first.target = manifest.export.target.clone();
    first.scope = SyncScope::Project;
    first.prompts = manifest.attach.prompts.clone();
    first.skills = manifest.attach.skills.clone();
}

fn normalized_exports(manifest: &ProjectManifest) -> Vec<ProjectExportProfile> {
    if manifest.exports.is_empty() {
        return vec![ProjectExportProfile {
            target: manifest.export.target.clone(),
            scope: SyncScope::Project,
            prompts: manifest.attach.prompts.clone(),
            skills: manifest.attach.skills.clone(),
            playbooks: Vec::new(),
            instruction_rules: Vec::new(),
            command_rules: Vec::new(),
        }];
    }
    manifest.exports.clone()
}

fn is_v1_compatible(manifest: &ProjectManifest) -> bool {
    let exports = normalized_exports(manifest);
    if exports.len() != 1 {
        return false;
    }
    let first = &exports[0];
    first.target == manifest.export.target
        && first.scope == SyncScope::Project
        && first.prompts == manifest.attach.prompts
        && first.skills == manifest.attach.skills
        && first.playbooks.is_empty()
        && first.instruction_rules.is_empty()
        && first.command_rules.is_empty()
}

fn invalid_manifest(path: &Path, message: &str) -> FlowmintError {
    FlowmintError::InvalidProjectManifest {
        path: path.to_path_buf(),
        message: message.to_string(),
    }
}

#[derive(Debug, Clone, Copy)]
enum Section {
    None,
    Project,
    Export,
    Attach,
    ExportProfile(usize),
}

enum AttachmentKind {
    Prompt,
    Skill,
}

#[derive(Debug, Clone, Copy)]
pub enum AttachmentAction {
    Attach,
    Detach,
}

fn update_export_attachment(
    project_dir: &Path,
    asset_ref: &str,
    action: AttachmentAction,
) -> Result<ProjectManifest> {
    let mut manifest = init_project_manifest(project_dir)?;
    if manifest.exports.is_empty() {
        sync_exports_from_legacy_fields(&mut manifest);
    }
    update_export_profile_values(&mut manifest.exports[0], asset_ref, action)?;
    sync_legacy_fields_from_first_export(&mut manifest);
    write_project_manifest(project_dir, &manifest)?;
    Ok(manifest)
}

fn update_export_profile_attachment(
    project_dir: &Path,
    target: &str,
    scope: SyncScope,
    asset_ref: &str,
    action: AttachmentAction,
) -> Result<ProjectManifest> {
    if target.trim().is_empty() {
        return Err(FlowmintError::InvalidAsset {
            messages: vec!["export target is required".to_string()],
        });
    }

    let mut manifest = init_project_manifest(project_dir)?;
    if manifest.exports.is_empty() {
        sync_exports_from_legacy_fields(&mut manifest);
    }
    let index = ensure_export_profile(&mut manifest, target, scope);
    update_export_profile_values(&mut manifest.exports[index], asset_ref, action)?;
    sync_legacy_fields_from_first_export(&mut manifest);
    write_project_manifest(project_dir, &manifest)?;
    Ok(manifest)
}

pub fn update_export_profile_values(
    profile: &mut ProjectExportProfile,
    asset_ref: &str,
    action: AttachmentAction,
) -> Result<()> {
    let (kind, asset_id) = parse_export_asset_ref(asset_ref)?;
    if !is_safe_asset_id(asset_id) {
        return Err(FlowmintError::InvalidAsset {
            messages: vec![
                "attached asset id must use only a-z, 0-9, hyphen, or underscore".to_string(),
            ],
        });
    }

    let values = match kind {
        ExportAttachmentKind::Prompt => &mut profile.prompts,
        ExportAttachmentKind::Skill => &mut profile.skills,
        ExportAttachmentKind::Playbook => &mut profile.playbooks,
        ExportAttachmentKind::InstructionRule => &mut profile.instruction_rules,
        ExportAttachmentKind::CommandRule => &mut profile.command_rules,
    };

    match action {
        AttachmentAction::Attach => {
            if !values.iter().any(|value| value == asset_id) {
                values.push(asset_id.to_string());
            }
        }
        AttachmentAction::Detach => values.retain(|value| value != asset_id),
    }

    Ok(())
}

fn ensure_export_profile(manifest: &mut ProjectManifest, target: &str, scope: SyncScope) -> usize {
    if let Some(index) = manifest
        .exports
        .iter()
        .position(|profile| profile.target == target && profile.scope == scope)
    {
        return index;
    }

    manifest.exports.push(ProjectExportProfile {
        target: target.to_string(),
        scope,
        prompts: Vec::new(),
        skills: Vec::new(),
        playbooks: Vec::new(),
        instruction_rules: Vec::new(),
        command_rules: Vec::new(),
    });
    manifest.exports.len() - 1
}

fn parse_export_asset_ref(asset_ref: &str) -> Result<(ExportAttachmentKind, &str)> {
    let Some((asset_type, id)) = asset_ref.split_once(':') else {
        return Err(invalid_asset_ref(asset_ref));
    };

    let kind = match asset_type {
        "prompt" => ExportAttachmentKind::Prompt,
        "skill" => ExportAttachmentKind::Skill,
        "playbook" => ExportAttachmentKind::Playbook,
        "instruction-rule" => ExportAttachmentKind::InstructionRule,
        "command-rule" => ExportAttachmentKind::CommandRule,
        _ => return Err(invalid_asset_ref(asset_ref)),
    };

    Ok((kind, id))
}

fn invalid_asset_ref(asset_ref: &str) -> FlowmintError {
    FlowmintError::InvalidAsset {
        messages: vec![format!(
            "asset_ref '{asset_ref}' must use prompt:<id>, skill:<id>, playbook:<id>, instruction-rule:<id>, or command-rule:<id>"
        )],
    }
}

enum ExportAttachmentKind {
    Prompt,
    Skill,
    Playbook,
    InstructionRule,
    CommandRule,
}

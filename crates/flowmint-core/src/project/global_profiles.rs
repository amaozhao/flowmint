use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{FlowmintError, Result};
use crate::project::manifest::{
    AttachmentAction, ProjectExportProfile, update_export_profile_values,
};
use crate::sync::plan::SyncScope;

const GLOBAL_SYNC_PROFILES_FILE: &str = "global-sync-profiles.toml";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSyncProfiles {
    pub profiles: Vec<ProjectExportProfile>,
}

pub fn global_sync_profiles_path(library_home: &Path) -> PathBuf {
    library_home.join(GLOBAL_SYNC_PROFILES_FILE)
}

pub fn load_global_sync_profiles(library_home: &Path) -> Result<GlobalSyncProfiles> {
    let path = global_sync_profiles_path(library_home);
    if !path.exists() {
        return Ok(GlobalSyncProfiles::default());
    }

    let content =
        std::fs::read_to_string(&path).map_err(|source| FlowmintError::io(&path, source))?;
    parse_global_sync_profiles(&path, &content)
}

pub fn write_global_sync_profiles(
    library_home: &Path,
    profiles: &GlobalSyncProfiles,
) -> Result<()> {
    validate_global_sync_profiles(global_sync_profiles_path(library_home).as_path(), profiles)?;
    std::fs::create_dir_all(library_home)
        .map_err(|source| FlowmintError::io(library_home, source))?;
    let path = global_sync_profiles_path(library_home);
    std::fs::write(&path, render_global_sync_profiles(profiles))
        .map_err(|source| FlowmintError::io(path, source))
}

pub fn attach_global_profile_asset(
    library_home: &Path,
    target: &str,
    asset_ref: &str,
) -> Result<GlobalSyncProfiles> {
    update_global_profile_asset(library_home, target, asset_ref, AttachmentAction::Attach)
}

pub fn detach_global_profile_asset(
    library_home: &Path,
    target: &str,
    asset_ref: &str,
) -> Result<GlobalSyncProfiles> {
    update_global_profile_asset(library_home, target, asset_ref, AttachmentAction::Detach)
}

fn update_global_profile_asset(
    library_home: &Path,
    target: &str,
    asset_ref: &str,
    action: AttachmentAction,
) -> Result<GlobalSyncProfiles> {
    if target.trim().is_empty() {
        return Err(FlowmintError::InvalidAsset {
            messages: vec!["global profile target is required".to_string()],
        });
    }

    let mut profiles = load_global_sync_profiles(library_home)?;
    let index = ensure_global_profile(&mut profiles, target);
    update_export_profile_values(&mut profiles.profiles[index], asset_ref, action)?;
    write_global_sync_profiles(library_home, &profiles)?;
    Ok(profiles)
}

fn parse_global_sync_profiles(path: &Path, content: &str) -> Result<GlobalSyncProfiles> {
    let mut profiles = GlobalSyncProfiles::default();
    let mut current_profile: Option<usize> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line == "[[profiles]]" {
            profiles.profiles.push(default_global_profile());
            current_profile = Some(profiles.profiles.len() - 1);
            continue;
        }

        let Some(index) = current_profile else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "target" => {
                profiles.profiles[index].target = parse_string(value)
                    .ok_or_else(|| invalid_profiles(path, "profiles.target must be a string"))?;
            }
            "scope" => {
                let value = parse_string(value)
                    .ok_or_else(|| invalid_profiles(path, "profiles.scope must be a string"))?;
                profiles.profiles[index].scope = parse_scope(&value)
                    .ok_or_else(|| invalid_profiles(path, "profiles.scope must be global-user"))?;
            }
            "prompts" => {
                profiles.profiles[index].prompts = parse_string_array(value).ok_or_else(|| {
                    invalid_profiles(path, "profiles.prompts must be a string array")
                })?;
            }
            "skills" => {
                profiles.profiles[index].skills = parse_string_array(value).ok_or_else(|| {
                    invalid_profiles(path, "profiles.skills must be a string array")
                })?;
            }
            "playbooks" => {
                profiles.profiles[index].playbooks =
                    parse_string_array(value).ok_or_else(|| {
                        invalid_profiles(path, "profiles.playbooks must be a string array")
                    })?;
            }
            "instruction_rules" => {
                profiles.profiles[index].instruction_rules =
                    parse_string_array(value).ok_or_else(|| {
                        invalid_profiles(path, "profiles.instruction_rules must be a string array")
                    })?;
            }
            "command_rules" => {
                profiles.profiles[index].command_rules =
                    parse_string_array(value).ok_or_else(|| {
                        invalid_profiles(path, "profiles.command_rules must be a string array")
                    })?;
            }
            _ => {}
        }
    }

    validate_global_sync_profiles(path, &profiles)?;
    Ok(profiles)
}

fn render_global_sync_profiles(profiles: &GlobalSyncProfiles) -> String {
    let mut content = String::new();
    for profile in &profiles.profiles {
        content.push_str("[[profiles]]\n");
        push_toml_string(&mut content, "target", &profile.target);
        push_toml_string(&mut content, "scope", render_scope(profile.scope));
        push_toml_array(&mut content, "prompts", &profile.prompts);
        push_toml_array(&mut content, "skills", &profile.skills);
        push_toml_array(&mut content, "playbooks", &profile.playbooks);
        push_toml_array(
            &mut content,
            "instruction_rules",
            &profile.instruction_rules,
        );
        push_toml_array(&mut content, "command_rules", &profile.command_rules);
        content.push('\n');
    }
    content
}

fn validate_global_sync_profiles(path: &Path, profiles: &GlobalSyncProfiles) -> Result<()> {
    if profiles
        .profiles
        .iter()
        .any(|profile| profile.scope != SyncScope::GlobalUser)
    {
        return Err(invalid_profiles(
            path,
            "global sync profiles must use global-user scope",
        ));
    }
    Ok(())
}

fn default_global_profile() -> ProjectExportProfile {
    ProjectExportProfile {
        target: "claude-code".to_string(),
        scope: SyncScope::GlobalUser,
        prompts: Vec::new(),
        skills: Vec::new(),
        playbooks: Vec::new(),
        instruction_rules: Vec::new(),
        command_rules: Vec::new(),
    }
}

fn ensure_global_profile(profiles: &mut GlobalSyncProfiles, target: &str) -> usize {
    if let Some(index) = profiles
        .profiles
        .iter()
        .position(|profile| profile.target == target && profile.scope == SyncScope::GlobalUser)
    {
        return index;
    }

    profiles.profiles.push(ProjectExportProfile {
        target: target.to_string(),
        scope: SyncScope::GlobalUser,
        prompts: Vec::new(),
        skills: Vec::new(),
        playbooks: Vec::new(),
        instruction_rules: Vec::new(),
        command_rules: Vec::new(),
    });
    profiles.profiles.len() - 1
}

fn parse_scope(value: &str) -> Option<SyncScope> {
    match value {
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

fn push_toml_string(content: &mut String, key: &str, value: &str) {
    content.push_str(key);
    content.push_str(" = \"");
    content.push_str(&escape_toml_string(value));
    content.push_str("\"\n");
}

fn push_toml_array(content: &mut String, key: &str, values: &[String]) {
    content.push_str(key);
    content.push_str(" = [");
    content.push_str(
        &values
            .iter()
            .map(|value| format!("\"{}\"", escape_toml_string(value)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    content.push_str("]\n");
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn invalid_profiles(path: &Path, message: &str) -> FlowmintError {
    FlowmintError::InvalidProjectManifest {
        path: path.to_path_buf(),
        message: message.to_string(),
    }
}

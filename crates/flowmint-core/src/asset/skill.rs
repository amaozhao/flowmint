use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::asset::id::{is_safe_asset_id, validate_new_asset_id};
use crate::asset::model::{
    AssetSummary, AssetType, SkillAsset, SkillFile, SkillFileKind, SkillMetadata,
};
use crate::error::{FlowmintError, Result};
use crate::validation::{ValidationStatus, validate_skill};

pub fn create_skill(library_home: &Path, skill: SkillAsset) -> Result<SkillAsset> {
    validate_skill_for_write(library_home, &skill, true)?;
    write_skill(library_home, &skill)?;
    get_skill(library_home, &skill.id)
}

pub fn update_skill(library_home: &Path, skill: SkillAsset) -> Result<SkillAsset> {
    validate_skill_for_write(library_home, &skill, false)?;
    write_skill(library_home, &skill)?;
    get_skill(library_home, &skill.id)
}

pub fn get_skill(library_home: &Path, id: &str) -> Result<SkillAsset> {
    if !is_safe_asset_id(id) {
        return Err(FlowmintError::InvalidAsset {
            messages: vec!["id must use only a-z, 0-9, hyphen, or underscore".to_string()],
        });
    }

    read_skill(&skill_dir(library_home, id))
}

pub fn list_skills(library_home: &Path) -> Result<Vec<AssetSummary>> {
    let skills_dir = library_home.join("skills");
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    for entry in
        std::fs::read_dir(&skills_dir).map_err(|source| FlowmintError::io(&skills_dir, source))?
    {
        let entry = entry.map_err(|source| FlowmintError::io(&skills_dir, source))?;
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").is_file() {
            continue;
        }

        let skill = read_skill(&path)?;
        skills.push(AssetSummary {
            id: skill.id,
            asset_type: AssetType::Skill,
            name: skill.name,
            description: skill.description,
            tags: skill.tags,
            updated_at: modified_time_value(&path.join("SKILL.md")),
            path,
            validation_status: ValidationStatus::Valid,
        });
    }

    skills.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(skills)
}

fn validate_skill_for_write(
    library_home: &Path,
    skill: &SkillAsset,
    require_new_id: bool,
) -> Result<()> {
    let mut messages = validate_skill(skill).messages;

    if require_new_id {
        messages.extend(validate_new_asset_id(library_home, AssetType::Skill, &skill.id).messages);
    }

    if messages.is_empty() {
        Ok(())
    } else {
        Err(FlowmintError::InvalidAsset { messages })
    }
}

fn write_skill(library_home: &Path, skill: &SkillAsset) -> Result<()> {
    let root_dir = skill_dir(library_home, &skill.id);
    std::fs::create_dir_all(&root_dir).map_err(|source| FlowmintError::io(&root_dir, source))?;

    let skill_md_path = root_dir.join("SKILL.md");
    std::fs::write(&skill_md_path, &skill.skill_md)
        .map_err(|source| FlowmintError::io(&skill_md_path, source))?;

    let metadata_path = root_dir.join("metadata.toml");
    std::fs::write(&metadata_path, render_metadata(skill))
        .map_err(|source| FlowmintError::io(metadata_path, source))?;
    write_supporting_files(&root_dir, skill)
}

fn read_skill(root_dir: &Path) -> Result<SkillAsset> {
    let id = root_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| FlowmintError::InvalidAsset {
            messages: vec!["skill directory has no valid id".to_string()],
        })?
        .to_string();

    let skill_md_path = root_dir.join("SKILL.md");
    let skill_md = std::fs::read_to_string(&skill_md_path)
        .map_err(|source| FlowmintError::io(&skill_md_path, source))?;

    let metadata_path = root_dir.join("metadata.toml");
    let metadata_content = if metadata_path.exists() {
        Some(
            std::fs::read_to_string(&metadata_path)
                .map_err(|source| FlowmintError::io(&metadata_path, source))?,
        )
    } else {
        None
    };

    let parsed_metadata = metadata_content
        .as_deref()
        .map(parse_metadata)
        .unwrap_or_else(|| ParsedSkillMetadata {
            name: title_from_skill_md(&skill_md).unwrap_or_else(|| id.clone()),
            description: None,
            tags: Vec::new(),
        });

    Ok(SkillAsset {
        id,
        name: parsed_metadata.name,
        description: parsed_metadata.description,
        tags: parsed_metadata.tags,
        root_dir: root_dir.to_path_buf(),
        skill_md,
        metadata: metadata_content.map(|raw_toml| SkillMetadata { raw_toml }),
        files: collect_skill_files(root_dir)?,
    })
}

fn skill_dir(library_home: &Path, id: &str) -> PathBuf {
    library_home.join("skills").join(id)
}

fn render_metadata(skill: &SkillAsset) -> String {
    let core_metadata = render_core_metadata(skill);
    let Some(raw_toml) = skill
        .metadata
        .as_ref()
        .map(|metadata| metadata.raw_toml.as_str())
        .filter(|raw_toml| !raw_toml.trim().is_empty())
    else {
        return core_metadata;
    };

    let custom_lines = raw_toml
        .lines()
        .filter(|line| !is_core_metadata_line(line.trim_start()))
        .collect::<Vec<_>>();

    if custom_lines.is_empty() {
        return core_metadata;
    }

    format!("{core_metadata}{}\n", custom_lines.join("\n"))
}

fn render_core_metadata(skill: &SkillAsset) -> String {
    let description = skill.description.as_deref().unwrap_or_default();
    let tags = skill
        .tags
        .iter()
        .map(|tag| format!("\"{}\"", escape_toml_string(tag)))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "id = \"{}\"\nname = \"{}\"\ndescription = \"{}\"\ntags = [{}]\n",
        escape_toml_string(&skill.id),
        escape_toml_string(&skill.name),
        escape_toml_string(description),
        tags
    )
}

fn is_core_metadata_line(line: &str) -> bool {
    line.starts_with("id =")
        || line.starts_with("name =")
        || line.starts_with("description =")
        || line.starts_with("tags =")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSkillMetadata {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
}

fn parse_metadata(content: &str) -> ParsedSkillMetadata {
    ParsedSkillMetadata {
        name: parse_string_value(content, "name").unwrap_or_default(),
        description: parse_string_value(content, "description").filter(|value| !value.is_empty()),
        tags: parse_tags(content),
    }
}

fn parse_string_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(parse_toml_string))
}

fn parse_toml_string(value: &str) -> String {
    value
        .trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or_default()
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn parse_tags(content: &str) -> Vec<String> {
    let Some(values) = content
        .lines()
        .find_map(|line| line.trim().strip_prefix("tags = "))
    else {
        return Vec::new();
    };

    let Some(values) = values
        .trim()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };

    values
        .split(',')
        .filter_map(|value| {
            let value = parse_toml_string(value);
            if value.is_empty() { None } else { Some(value) }
        })
        .collect()
}

fn collect_skill_files(root_dir: &Path) -> Result<Vec<SkillFile>> {
    let mut files = Vec::new();
    let skill_md_path = root_dir.join("SKILL.md");
    if skill_md_path.exists() {
        files.push(SkillFile {
            path: skill_md_path,
            kind: SkillFileKind::SkillMarkdown,
            content: None,
        });
    }

    let metadata_path = root_dir.join("metadata.toml");
    if metadata_path.exists() {
        files.push(SkillFile {
            path: metadata_path,
            kind: SkillFileKind::Metadata,
            content: None,
        });
    }

    collect_child_files(root_dir, "examples", SkillFileKind::Example, &mut files)?;
    collect_child_files(root_dir, "resources", SkillFileKind::Resource, &mut files)?;

    Ok(files)
}

fn collect_child_files(
    root_dir: &Path,
    folder: &str,
    kind: SkillFileKind,
    files: &mut Vec<SkillFile>,
) -> Result<()> {
    let folder_path = root_dir.join(folder);
    if !folder_path.exists() {
        return Ok(());
    }

    collect_child_files_recursive(&folder_path, kind, files)
}

fn collect_child_files_recursive(
    folder_path: &Path,
    kind: SkillFileKind,
    files: &mut Vec<SkillFile>,
) -> Result<()> {
    for entry in
        std::fs::read_dir(folder_path).map_err(|source| FlowmintError::io(folder_path, source))?
    {
        let entry = entry.map_err(|source| FlowmintError::io(folder_path, source))?;
        let path = entry.path();
        if path.is_file() {
            files.push(SkillFile {
                content: std::fs::read_to_string(&path).ok(),
                path,
                kind: kind.clone(),
            });
        } else if path.is_dir() {
            collect_child_files_recursive(&path, kind.clone(), files)?;
        }
    }

    Ok(())
}

fn write_supporting_files(root_dir: &Path, skill: &SkillAsset) -> Result<()> {
    let mut desired_paths = HashSet::new();

    for file in &skill.files {
        if !matches!(file.kind, SkillFileKind::Example | SkillFileKind::Resource) {
            continue;
        }
        let target_path = supporting_file_target_path(root_dir, &skill.root_dir, file)?;
        desired_paths.insert(target_path.clone());
        if let Some(content) = &file.content {
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|source| FlowmintError::io(parent, source))?;
            }
            std::fs::write(&target_path, content)
                .map_err(|source| FlowmintError::io(target_path, source))?;
        }
    }

    for folder in ["examples", "resources"] {
        prune_supporting_folder(&root_dir.join(folder), &desired_paths)?;
    }

    Ok(())
}

fn prune_supporting_folder(folder_path: &Path, desired_paths: &HashSet<PathBuf>) -> Result<bool> {
    if !folder_path.exists() {
        return Ok(true);
    }

    let mut is_empty = true;
    for entry in
        std::fs::read_dir(folder_path).map_err(|source| FlowmintError::io(folder_path, source))?
    {
        let entry = entry.map_err(|source| FlowmintError::io(folder_path, source))?;
        let path = entry.path();
        if path.is_dir() {
            if prune_supporting_folder(&path, desired_paths)? {
                std::fs::remove_dir(&path).map_err(|source| FlowmintError::io(&path, source))?;
            } else {
                is_empty = false;
            }
        } else if desired_paths.contains(&path) {
            is_empty = false;
        } else {
            std::fs::remove_file(&path).map_err(|source| FlowmintError::io(&path, source))?;
        }
    }

    Ok(is_empty)
}

fn supporting_file_target_path(
    root_dir: &Path,
    previous_root_dir: &Path,
    file: &SkillFile,
) -> Result<PathBuf> {
    let folder = match file.kind {
        SkillFileKind::Example => "examples",
        SkillFileKind::Resource => "resources",
        SkillFileKind::SkillMarkdown | SkillFileKind::Metadata => {
            return Err(FlowmintError::InvalidAsset {
                messages: vec!["supporting file must be an example or resource".to_string()],
            });
        }
    };

    let relative_path = if file.path.is_absolute() && !previous_root_dir.as_os_str().is_empty() {
        file.path
            .strip_prefix(previous_root_dir)
            .map(Path::to_path_buf)
            .map_err(|_| FlowmintError::InvalidAsset {
                messages: vec![
                    "supporting file path must be inside the Skill directory".to_string(),
                ],
            })?
    } else {
        file.path.clone()
    };

    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(FlowmintError::InvalidAsset {
            messages: vec![
                "supporting file path must be relative and stay inside examples/ or resources/"
                    .to_string(),
            ],
        });
    }

    let folder_path = Path::new(folder);
    let target_relative = if relative_path.starts_with(folder_path) {
        relative_path
    } else {
        folder_path.join(relative_path)
    };

    if target_relative
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(FlowmintError::InvalidAsset {
            messages: vec![
                "supporting file path must be relative and stay inside examples/ or resources/"
                    .to_string(),
            ],
        });
    }

    Ok(root_dir.join(target_relative))
}

fn title_from_skill_md(skill_md: &str) -> Option<String> {
    skill_md
        .lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(str::trim))
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn modified_time_value(path: &Path) -> Option<String> {
    let seconds = std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(format!("unix:{seconds}"))
}

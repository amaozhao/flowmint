use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::asset::id::{is_safe_asset_id, validate_new_asset_id};
use crate::asset::model::{AssetSummary, AssetType, PromptAsset, PromptVariable};
use crate::error::{FlowmintError, Result};
use crate::validation::{ValidationStatus, validate_prompt};

const PROMPT_BEGIN: &str = "<!-- FLOWMINT:PROMPT:BEGIN\n";
const PROMPT_END: &str = "\nFLOWMINT:PROMPT:END -->\n\n";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromptMetadata {
    id: String,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    variables: Vec<PromptVariable>,
}

impl From<&PromptAsset> for PromptMetadata {
    fn from(prompt: &PromptAsset) -> Self {
        Self {
            id: prompt.id.clone(),
            name: prompt.name.clone(),
            description: prompt.description.clone(),
            tags: prompt.tags.clone(),
            variables: prompt.variables.clone(),
        }
    }
}

pub fn create_prompt(library_home: &Path, prompt: PromptAsset) -> Result<PromptAsset> {
    validate_prompt_for_write(library_home, &prompt, true)?;
    write_prompt(library_home, &prompt)?;
    Ok(prompt)
}

pub fn update_prompt(library_home: &Path, prompt: PromptAsset) -> Result<PromptAsset> {
    validate_prompt_for_write(library_home, &prompt, false)?;
    write_prompt(library_home, &prompt)?;
    Ok(prompt)
}

pub fn get_prompt(library_home: &Path, id: &str) -> Result<PromptAsset> {
    if !is_safe_asset_id(id) {
        return Err(FlowmintError::InvalidAsset {
            messages: vec!["id must use only a-z, 0-9, hyphen, or underscore".to_string()],
        });
    }

    read_prompt(&prompt_path(library_home, id))
}

pub fn list_prompts(library_home: &Path) -> Result<Vec<AssetSummary>> {
    let prompts_dir = library_home.join("prompts");
    if !prompts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut prompts = Vec::new();

    for entry in
        std::fs::read_dir(&prompts_dir).map_err(|source| FlowmintError::io(&prompts_dir, source))?
    {
        let entry = entry.map_err(|source| FlowmintError::io(&prompts_dir, source))?;
        let path = entry.path();

        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }

        let prompt = read_prompt(&path)?;
        prompts.push(AssetSummary {
            id: prompt.id,
            asset_type: AssetType::Prompt,
            name: prompt.name,
            description: prompt.description,
            tags: prompt.tags,
            updated_at: modified_time_value(&path),
            path,
            validation_status: ValidationStatus::Valid,
        });
    }

    prompts.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(prompts)
}

fn validate_prompt_for_write(
    library_home: &Path,
    prompt: &PromptAsset,
    require_new_id: bool,
) -> Result<()> {
    let mut messages = validate_prompt(prompt).messages;

    if require_new_id {
        messages
            .extend(validate_new_asset_id(library_home, AssetType::Prompt, &prompt.id).messages);
    }

    if messages.is_empty() {
        Ok(())
    } else {
        Err(FlowmintError::InvalidAsset { messages })
    }
}

fn write_prompt(library_home: &Path, prompt: &PromptAsset) -> Result<()> {
    let path = prompt_path(library_home, &prompt.id);
    let parent =
        path.parent()
            .map(PathBuf::from)
            .ok_or_else(|| FlowmintError::InvalidPromptFile {
                path: path.clone(),
                message: "prompt path has no parent directory".to_string(),
            })?;

    std::fs::create_dir_all(&parent).map_err(|source| FlowmintError::io(&parent, source))?;

    let metadata =
        serde_json::to_string_pretty(&PromptMetadata::from(prompt)).map_err(|error| {
            FlowmintError::InvalidAsset {
                messages: vec![format!("prompt metadata could not serialize: {error}")],
            }
        })?;

    let content = format!("{PROMPT_BEGIN}{metadata}{PROMPT_END}{}", prompt.body);
    std::fs::write(&path, content).map_err(|source| FlowmintError::io(path, source))
}

fn read_prompt(path: &Path) -> Result<PromptAsset> {
    let content =
        std::fs::read_to_string(path).map_err(|source| FlowmintError::io(path, source))?;
    let Some(rest) = content.strip_prefix(PROMPT_BEGIN) else {
        return Err(FlowmintError::InvalidPromptFile {
            path: path.to_path_buf(),
            message: "missing Flowmint metadata header".to_string(),
        });
    };

    let Some((metadata, body)) = rest.split_once(PROMPT_END) else {
        return Err(FlowmintError::InvalidPromptFile {
            path: path.to_path_buf(),
            message: "missing Flowmint metadata footer".to_string(),
        });
    };

    let metadata: PromptMetadata =
        serde_json::from_str(metadata).map_err(|error| FlowmintError::InvalidPromptFile {
            path: path.to_path_buf(),
            message: format!("metadata JSON is invalid: {error}"),
        })?;

    Ok(PromptAsset {
        id: metadata.id,
        name: metadata.name,
        description: metadata.description,
        tags: metadata.tags,
        variables: metadata.variables,
        body: body.to_string(),
    })
}

fn prompt_path(library_home: &Path, id: &str) -> PathBuf {
    library_home.join("prompts").join(format!("{id}.md"))
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

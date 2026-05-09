use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::asset::id::{is_safe_asset_id, validate_new_asset_id};
use crate::asset::model::{
    AssetSummary, AssetType, PlaybookAsset, PlaybookInvocation, PlaybookSideEffectLevel,
};
use crate::asset::skill::get_skill;
use crate::error::{FlowmintError, Result};
use crate::validation::{ValidationStatus, validate_playbook};

const PLAYBOOK_BEGIN: &str = "<!-- FLOWMINT:PLAYBOOK:BEGIN\n";
const PLAYBOOK_END: &str = "\nFLOWMINT:PLAYBOOK:END -->\n\n";

pub fn create_playbook(library_home: &Path, playbook: PlaybookAsset) -> Result<PlaybookAsset> {
    validate_playbook_for_write(library_home, &playbook, true)?;
    write_playbook(library_home, &playbook)?;
    Ok(playbook)
}

pub fn update_playbook(library_home: &Path, playbook: PlaybookAsset) -> Result<PlaybookAsset> {
    validate_playbook_for_write(library_home, &playbook, false)?;
    write_playbook(library_home, &playbook)?;
    Ok(playbook)
}

pub fn get_playbook(library_home: &Path, id: &str) -> Result<PlaybookAsset> {
    if !is_safe_asset_id(id) {
        return Err(FlowmintError::InvalidAsset {
            messages: vec!["id must use only a-z, 0-9, hyphen, or underscore".to_string()],
        });
    }

    read_playbook(&playbook_path(library_home, id))
}

pub fn list_playbooks(library_home: &Path) -> Result<Vec<AssetSummary>> {
    let playbooks_dir = library_home.join("playbooks");
    if !playbooks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut playbooks = Vec::new();
    for entry in std::fs::read_dir(&playbooks_dir)
        .map_err(|source| FlowmintError::io(&playbooks_dir, source))?
    {
        let entry = entry.map_err(|source| FlowmintError::io(&playbooks_dir, source))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }

        let playbook = read_playbook(&path)?;
        playbooks.push(AssetSummary {
            id: playbook.id,
            asset_type: AssetType::Playbook,
            name: playbook.name,
            description: playbook.description,
            tags: playbook.tags,
            updated_at: modified_time_value(&path),
            path,
            validation_status: ValidationStatus::Valid,
        });
    }

    playbooks.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(playbooks)
}

pub fn promote_skill_to_playbook(
    library_home: &Path,
    skill_id: &str,
    playbook_id: &str,
) -> Result<PlaybookAsset> {
    let skill = get_skill(library_home, skill_id)?;
    let playbook = PlaybookAsset {
        id: playbook_id.to_string(),
        name: skill.name,
        description: skill.description,
        tags: skill.tags,
        trigger: "Run this playbook when the legacy Playbook Skill applies.".to_string(),
        inputs: Vec::new(),
        steps: vec![crate::asset::model::PlaybookStep {
            title: "Follow legacy Skill instructions".to_string(),
            body: skill.skill_md,
        }],
        verification: "Complete the verification described by the legacy Skill.".to_string(),
        failure_handling: "Stop and report the blocker.".to_string(),
        side_effect_level: PlaybookSideEffectLevel::RunsCommands,
        recommended_invocation: PlaybookInvocation::Manual,
        target_compatibility: vec!["claude-code".to_string(), "codex".to_string()],
    };
    create_playbook(library_home, playbook)
}

pub fn render_playbook_skill_md(playbook: &PlaybookAsset) -> String {
    let mut content = String::new();
    content.push_str("# ");
    content.push_str(&playbook.name);
    content.push_str("\n\n");

    if let Some(description) = playbook
        .description
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        content.push_str(description);
        content.push_str("\n\n");
    }

    content.push_str("## Trigger\n\n");
    content.push_str(&playbook.trigger);
    content.push_str("\n\n");

    if !playbook.inputs.is_empty() {
        content.push_str("## Inputs\n\n");
        for input in &playbook.inputs {
            content.push_str("- ");
            content.push_str(&input.name);
            if input.required {
                content.push_str(" (required)");
            }
            if let Some(description) = input.description.as_deref() {
                content.push_str(": ");
                content.push_str(description);
            }
            content.push('\n');
        }
        content.push('\n');
    }

    content.push_str("## Steps\n\n");
    for (index, step) in playbook.steps.iter().enumerate() {
        content.push_str(&format!("{}. {}\n\n", index + 1, step.title));
        content.push_str(&step.body);
        content.push_str("\n\n");
    }

    content.push_str("## Verification\n\n");
    content.push_str(&playbook.verification);
    content.push_str("\n\n");

    content.push_str("## Failure Handling\n\n");
    content.push_str(&playbook.failure_handling);
    content.push_str("\n\n");

    content.push_str("## Side Effect Level\n\n");
    content.push_str(side_effect_label(playbook.side_effect_level));
    content.push('\n');

    content
}

fn validate_playbook_for_write(
    library_home: &Path,
    playbook: &PlaybookAsset,
    require_new_id: bool,
) -> Result<()> {
    let mut messages = validate_playbook(playbook).messages;

    if require_new_id {
        messages.extend(
            validate_new_asset_id(library_home, AssetType::Playbook, &playbook.id).messages,
        );
    }

    if messages.is_empty() {
        Ok(())
    } else {
        Err(FlowmintError::InvalidAsset { messages })
    }
}

fn write_playbook(library_home: &Path, playbook: &PlaybookAsset) -> Result<()> {
    let path = playbook_path(library_home, &playbook.id);
    let parent =
        path.parent()
            .map(PathBuf::from)
            .ok_or_else(|| FlowmintError::InvalidPromptFile {
                path: path.clone(),
                message: "playbook path has no parent directory".to_string(),
            })?;

    std::fs::create_dir_all(&parent).map_err(|source| FlowmintError::io(&parent, source))?;

    let metadata =
        serde_json::to_string_pretty(playbook).map_err(|error| FlowmintError::InvalidAsset {
            messages: vec![format!("playbook metadata could not serialize: {error}")],
        })?;
    let content = format!(
        "{PLAYBOOK_BEGIN}{metadata}{PLAYBOOK_END}{}",
        render_playbook_skill_md(playbook)
    );
    std::fs::write(&path, content).map_err(|source| FlowmintError::io(path, source))
}

fn read_playbook(path: &Path) -> Result<PlaybookAsset> {
    let content =
        std::fs::read_to_string(path).map_err(|source| FlowmintError::io(path, source))?;
    let Some(rest) = content.strip_prefix(PLAYBOOK_BEGIN) else {
        return Err(FlowmintError::InvalidPromptFile {
            path: path.to_path_buf(),
            message: "missing Flowmint playbook metadata header".to_string(),
        });
    };

    let Some((metadata, _rendered)) = rest.split_once(PLAYBOOK_END) else {
        return Err(FlowmintError::InvalidPromptFile {
            path: path.to_path_buf(),
            message: "missing Flowmint playbook metadata footer".to_string(),
        });
    };

    serde_json::from_str(metadata).map_err(|error| FlowmintError::InvalidPromptFile {
        path: path.to_path_buf(),
        message: format!("metadata JSON is invalid: {error}"),
    })
}

fn playbook_path(library_home: &Path, id: &str) -> PathBuf {
    library_home.join("playbooks").join(format!("{id}.md"))
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

fn side_effect_label(level: PlaybookSideEffectLevel) -> &'static str {
    match level {
        PlaybookSideEffectLevel::None => "none",
        PlaybookSideEffectLevel::ReadOnly => "read-only",
        PlaybookSideEffectLevel::WritesFiles => "writes-files",
        PlaybookSideEffectLevel::RunsCommands => "runs-commands",
        PlaybookSideEffectLevel::ExternalSideEffects => "external-side-effects",
    }
}

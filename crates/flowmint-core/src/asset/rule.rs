use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::asset::id::{is_safe_asset_id, validate_new_asset_id};
use crate::asset::model::{AssetSummary, AssetType, RuleAsset, RuleKind};
use crate::error::{FlowmintError, Result};
use crate::validation::{ValidationStatus, validate_rule};

const RULE_BEGIN: &str = "<!-- FLOWMINT:RULE:BEGIN\n";
const RULE_END: &str = "\nFLOWMINT:RULE:END -->\n\n";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuleMetadata {
    id: String,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    rule_kind: RuleKind,
    path_globs: Vec<String>,
    command_rule: Option<crate::asset::model::CommandRule>,
    target_compatibility: Vec<String>,
}

impl From<&RuleAsset> for RuleMetadata {
    fn from(rule: &RuleAsset) -> Self {
        Self {
            id: rule.id.clone(),
            name: rule.name.clone(),
            description: rule.description.clone(),
            tags: rule.tags.clone(),
            rule_kind: rule.rule_kind,
            path_globs: rule.path_globs.clone(),
            command_rule: rule.command_rule.clone(),
            target_compatibility: rule.target_compatibility.clone(),
        }
    }
}

pub fn create_rule(library_home: &Path, rule: RuleAsset) -> Result<RuleAsset> {
    validate_rule_for_write(library_home, &rule, true)?;
    write_rule(library_home, &rule)?;
    Ok(rule)
}

pub fn update_rule(library_home: &Path, rule: RuleAsset) -> Result<RuleAsset> {
    validate_rule_for_write(library_home, &rule, false)?;
    write_rule(library_home, &rule)?;
    Ok(rule)
}

pub fn get_rule(library_home: &Path, id: &str) -> Result<RuleAsset> {
    if !is_safe_asset_id(id) {
        return Err(FlowmintError::InvalidAsset {
            messages: vec!["id must use only a-z, 0-9, hyphen, or underscore".to_string()],
        });
    }

    read_rule(&rule_path(library_home, id))
}

pub fn list_rules(library_home: &Path, kind: Option<RuleKind>) -> Result<Vec<AssetSummary>> {
    let rules_dir = library_home.join("rules");
    if !rules_dir.exists() {
        return Ok(Vec::new());
    }

    let mut rules = Vec::new();
    for entry in
        std::fs::read_dir(&rules_dir).map_err(|source| FlowmintError::io(&rules_dir, source))?
    {
        let entry = entry.map_err(|source| FlowmintError::io(&rules_dir, source))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }

        let rule = read_rule(&path)?;
        if kind.is_some_and(|kind| kind != rule.rule_kind) {
            continue;
        }
        rules.push(AssetSummary {
            id: rule.id,
            asset_type: asset_type_for_rule_kind(rule.rule_kind),
            name: rule.name,
            description: rule.description,
            tags: rule.tags,
            updated_at: modified_time_value(&path),
            path,
            validation_status: ValidationStatus::Valid,
        });
    }

    rules.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(rules)
}

fn validate_rule_for_write(
    library_home: &Path,
    rule: &RuleAsset,
    require_new_id: bool,
) -> Result<()> {
    let mut messages = validate_rule(rule).messages;

    if require_new_id {
        messages.extend(
            validate_new_asset_id(
                library_home,
                asset_type_for_rule_kind(rule.rule_kind),
                &rule.id,
            )
            .messages,
        );
    }

    if messages.is_empty() {
        Ok(())
    } else {
        Err(FlowmintError::InvalidAsset { messages })
    }
}

fn write_rule(library_home: &Path, rule: &RuleAsset) -> Result<()> {
    let path = rule_path(library_home, &rule.id);
    let parent =
        path.parent()
            .map(PathBuf::from)
            .ok_or_else(|| FlowmintError::InvalidPromptFile {
                path: path.clone(),
                message: "rule path has no parent directory".to_string(),
            })?;

    std::fs::create_dir_all(&parent).map_err(|source| FlowmintError::io(&parent, source))?;

    let metadata = serde_json::to_string_pretty(&RuleMetadata::from(rule)).map_err(|error| {
        FlowmintError::InvalidAsset {
            messages: vec![format!("rule metadata could not serialize: {error}")],
        }
    })?;

    let content = format!("{RULE_BEGIN}{metadata}{RULE_END}{}", rule.body);
    std::fs::write(&path, content).map_err(|source| FlowmintError::io(path, source))
}

fn read_rule(path: &Path) -> Result<RuleAsset> {
    let content =
        std::fs::read_to_string(path).map_err(|source| FlowmintError::io(path, source))?;
    let Some(rest) = content.strip_prefix(RULE_BEGIN) else {
        return Err(FlowmintError::InvalidPromptFile {
            path: path.to_path_buf(),
            message: "missing Flowmint rule metadata header".to_string(),
        });
    };

    let Some((metadata, body)) = rest.split_once(RULE_END) else {
        return Err(FlowmintError::InvalidPromptFile {
            path: path.to_path_buf(),
            message: "missing Flowmint rule metadata footer".to_string(),
        });
    };

    let metadata: RuleMetadata =
        serde_json::from_str(metadata).map_err(|error| FlowmintError::InvalidPromptFile {
            path: path.to_path_buf(),
            message: format!("metadata JSON is invalid: {error}"),
        })?;

    Ok(RuleAsset {
        id: metadata.id,
        name: metadata.name,
        description: metadata.description,
        tags: metadata.tags,
        rule_kind: metadata.rule_kind,
        path_globs: metadata.path_globs,
        command_rule: metadata.command_rule,
        target_compatibility: metadata.target_compatibility,
        body: body.to_string(),
    })
}

fn rule_path(library_home: &Path, id: &str) -> PathBuf {
    library_home.join("rules").join(format!("{id}.md"))
}

fn asset_type_for_rule_kind(kind: RuleKind) -> AssetType {
    match kind {
        RuleKind::Instruction => AssetType::InstructionRule,
        RuleKind::Command => AssetType::CommandRule,
    }
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

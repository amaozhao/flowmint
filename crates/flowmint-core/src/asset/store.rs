use std::path::Path;

use crate::asset::id::is_safe_asset_id;
use crate::asset::model::{
    AssetDetail, AssetFilter, AssetSummary, AssetType, CreateAssetInput, RuleKind, UpdateAssetInput,
};
use crate::asset::playbook::{create_playbook, get_playbook, list_playbooks, update_playbook};
use crate::asset::prompt::{create_prompt, get_prompt, list_prompts, update_prompt};
use crate::asset::rule::{create_rule, get_rule, list_rules, update_rule};
use crate::asset::skill::{create_skill, get_skill, list_skills, update_skill};
use crate::error::{FlowmintError, Result};
use crate::validation::{ValidationReport, validate_prompt, validate_rule, validate_skill};

pub fn list_assets(library_home: &Path, filter: AssetFilter) -> Result<Vec<AssetSummary>> {
    let mut assets = Vec::new();

    if matches!(filter.asset_type, None | Some(AssetType::Prompt)) {
        assets.extend(list_prompts(library_home)?);
    }

    if matches!(filter.asset_type, None | Some(AssetType::Skill)) {
        assets.extend(list_skills(library_home)?);
    }

    if matches!(filter.asset_type, None | Some(AssetType::Playbook)) {
        assets.extend(list_playbooks(library_home)?);
    }

    if matches!(filter.asset_type, None | Some(AssetType::InstructionRule)) {
        assets.extend(list_rules(library_home, Some(RuleKind::Instruction))?);
    }

    if matches!(filter.asset_type, None | Some(AssetType::CommandRule)) {
        assets.extend(list_rules(library_home, Some(RuleKind::Command))?);
    }

    if let Some(query) = filter
        .query
        .as_deref()
        .map(str::trim)
        .filter(|query| !query.is_empty())
    {
        let query = query.to_lowercase();
        assets.retain(|asset| {
            asset.id.to_lowercase().contains(&query)
                || asset.name.to_lowercase().contains(&query)
                || asset
                    .description
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&query)
                || asset
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&query))
        });
    }

    assets.sort_by(|left, right| {
        left.asset_type
            .as_sort_key()
            .cmp(right.asset_type.as_sort_key())
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(assets)
}

pub fn get_asset(library_home: &Path, asset_ref: &str) -> Result<AssetDetail> {
    match parse_asset_ref(asset_ref)? {
        ParsedAssetRef::Prompt(id) => Ok(AssetDetail::Prompt {
            asset: get_prompt(library_home, id)?,
        }),
        ParsedAssetRef::Skill(id) => Ok(AssetDetail::Skill {
            asset: get_skill(library_home, id)?,
        }),
        ParsedAssetRef::Playbook(id) => Ok(AssetDetail::Playbook {
            asset: get_playbook(library_home, id)?,
        }),
        ParsedAssetRef::InstructionRule(id) => {
            let asset = get_rule(library_home, id)?;
            ensure_rule_kind(&asset, RuleKind::Instruction)?;
            Ok(AssetDetail::InstructionRule { asset })
        }
        ParsedAssetRef::CommandRule(id) => {
            let asset = get_rule(library_home, id)?;
            ensure_rule_kind(&asset, RuleKind::Command)?;
            Ok(AssetDetail::CommandRule { asset })
        }
    }
}

pub fn create_asset(library_home: &Path, input: CreateAssetInput) -> Result<AssetDetail> {
    match input.asset {
        AssetDetail::Prompt { asset } => Ok(AssetDetail::Prompt {
            asset: create_prompt(library_home, asset)?,
        }),
        AssetDetail::Skill { asset } => Ok(AssetDetail::Skill {
            asset: create_skill(library_home, asset)?,
        }),
        AssetDetail::Playbook { asset } => Ok(AssetDetail::Playbook {
            asset: create_playbook(library_home, asset)?,
        }),
        AssetDetail::InstructionRule { asset } => {
            ensure_rule_kind(&asset, RuleKind::Instruction)?;
            Ok(AssetDetail::InstructionRule {
                asset: create_rule(library_home, asset)?,
            })
        }
        AssetDetail::CommandRule { asset } => {
            ensure_rule_kind(&asset, RuleKind::Command)?;
            Ok(AssetDetail::CommandRule {
                asset: create_rule(library_home, asset)?,
            })
        }
    }
}

pub fn update_asset(library_home: &Path, input: UpdateAssetInput) -> Result<AssetDetail> {
    match input.asset {
        AssetDetail::Prompt { asset } => {
            ensure_asset_exists(library_home, &format!("prompt:{}", asset.id))?;
            Ok(AssetDetail::Prompt {
                asset: update_prompt(library_home, asset)?,
            })
        }
        AssetDetail::Skill { asset } => {
            ensure_asset_exists(library_home, &format!("skill:{}", asset.id))?;
            Ok(AssetDetail::Skill {
                asset: update_skill(library_home, asset)?,
            })
        }
        AssetDetail::Playbook { asset } => {
            ensure_asset_exists(library_home, &format!("playbook:{}", asset.id))?;
            Ok(AssetDetail::Playbook {
                asset: update_playbook(library_home, asset)?,
            })
        }
        AssetDetail::InstructionRule { asset } => {
            ensure_rule_kind(&asset, RuleKind::Instruction)?;
            ensure_asset_exists(library_home, &format!("instruction-rule:{}", asset.id))?;
            Ok(AssetDetail::InstructionRule {
                asset: update_rule(library_home, asset)?,
            })
        }
        AssetDetail::CommandRule { asset } => {
            ensure_rule_kind(&asset, RuleKind::Command)?;
            ensure_asset_exists(library_home, &format!("command-rule:{}", asset.id))?;
            Ok(AssetDetail::CommandRule {
                asset: update_rule(library_home, asset)?,
            })
        }
    }
}

pub fn delete_asset(library_home: &Path, asset_ref: &str) -> Result<()> {
    match parse_asset_ref(asset_ref)? {
        ParsedAssetRef::Prompt(id) => {
            let path = library_home.join("prompts").join(format!("{id}.md"));
            if !path.exists() {
                return Err(FlowmintError::AssetNotFound {
                    asset_ref: asset_ref.to_string(),
                });
            }
            std::fs::remove_file(&path).map_err(|source| FlowmintError::io(path, source))
        }
        ParsedAssetRef::Skill(id) => {
            let path = library_home.join("skills").join(id);
            if !path.exists() {
                return Err(FlowmintError::AssetNotFound {
                    asset_ref: asset_ref.to_string(),
                });
            }
            std::fs::remove_dir_all(&path).map_err(|source| FlowmintError::io(path, source))
        }
        ParsedAssetRef::Playbook(id) => {
            let path = library_home.join("playbooks").join(format!("{id}.md"));
            if !path.exists() {
                return Err(FlowmintError::AssetNotFound {
                    asset_ref: asset_ref.to_string(),
                });
            }
            std::fs::remove_file(&path).map_err(|source| FlowmintError::io(path, source))
        }
        ParsedAssetRef::InstructionRule(id) | ParsedAssetRef::CommandRule(id) => {
            let path = library_home.join("rules").join(format!("{id}.md"));
            if !path.exists() {
                return Err(FlowmintError::AssetNotFound {
                    asset_ref: asset_ref.to_string(),
                });
            }
            std::fs::remove_file(&path).map_err(|source| FlowmintError::io(path, source))
        }
    }
}

pub fn validate_asset(library_home: &Path, asset_ref: &str) -> Result<ValidationReport> {
    match get_asset(library_home, asset_ref)? {
        AssetDetail::Prompt { asset } => Ok(validate_prompt(&asset)),
        AssetDetail::Skill { asset } => Ok(validate_skill(&asset)),
        AssetDetail::Playbook { asset } => Ok(crate::validation::validate_playbook(&asset)),
        AssetDetail::InstructionRule { asset } | AssetDetail::CommandRule { asset } => {
            Ok(validate_rule(&asset))
        }
    }
}

fn ensure_asset_exists(library_home: &Path, asset_ref: &str) -> Result<()> {
    get_asset(library_home, asset_ref).map(|_| ())
}

fn ensure_rule_kind(rule: &crate::asset::model::RuleAsset, expected: RuleKind) -> Result<()> {
    if rule.rule_kind == expected {
        return Ok(());
    }

    Err(FlowmintError::InvalidAsset {
        messages: vec!["rule asset type does not match rule_kind".to_string()],
    })
}

enum ParsedAssetRef<'a> {
    Prompt(&'a str),
    Skill(&'a str),
    Playbook(&'a str),
    InstructionRule(&'a str),
    CommandRule(&'a str),
}

fn parse_asset_ref(asset_ref: &str) -> Result<ParsedAssetRef<'_>> {
    let Some((asset_type, id)) = asset_ref.split_once(':') else {
        return Err(invalid_asset_ref(asset_ref));
    };

    if !is_safe_asset_id(id) {
        return Err(invalid_asset_ref(asset_ref));
    }

    match asset_type {
        "prompt" => Ok(ParsedAssetRef::Prompt(id)),
        "skill" => Ok(ParsedAssetRef::Skill(id)),
        "playbook" => Ok(ParsedAssetRef::Playbook(id)),
        "instruction-rule" => Ok(ParsedAssetRef::InstructionRule(id)),
        "command-rule" => Ok(ParsedAssetRef::CommandRule(id)),
        _ => Err(invalid_asset_ref(asset_ref)),
    }
}

fn invalid_asset_ref(asset_ref: &str) -> FlowmintError {
    FlowmintError::InvalidAsset {
        messages: vec![format!(
            "asset_ref '{asset_ref}' must use prompt:<id>, skill:<id>, playbook:<id>, instruction-rule:<id>, or command-rule:<id>"
        )],
    }
}

trait AssetTypeSortKey {
    fn as_sort_key(&self) -> &'static str;
}

impl AssetTypeSortKey for AssetType {
    fn as_sort_key(&self) -> &'static str {
        match self {
            AssetType::Prompt => "0-prompt",
            AssetType::Skill => "1-skill",
            AssetType::Playbook => "2-playbook",
            AssetType::InstructionRule => "3-instruction-rule",
            AssetType::CommandRule => "4-command-rule",
        }
    }
}

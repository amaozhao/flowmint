use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::asset::model::{
    AssetType, CommandRule, CommandRuleDecision, PromptAsset, RuleAsset, RuleKind, SkillAsset,
};
use crate::asset::prompt::create_prompt;
use crate::asset::rule::create_rule;
use crate::asset::skill::create_skill;
use crate::error::{FlowmintError, Result};
use crate::exporters::claude_code::PlannedLockRecord;
use crate::sync::diff::content_hash;
use crate::sync::lockfile::merge_lockfile_records_path;
use crate::sync::plan::SyncScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportAdoptionMode {
    CopyIntoLibrary,
    AdoptIntoFlowmint,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAdoptionSelection {
    pub id: String,
    pub asset_type: AssetType,
    pub source_path: PathBuf,
    pub mode: ImportAdoptionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAdoptionPlan {
    pub plan_id: String,
    pub target: String,
    pub scope: SyncScope,
    pub sync_root: PathBuf,
    pub lockfile_path: PathBuf,
    pub items: Vec<ImportAdoptionItem>,
    pub conflicts: Vec<ImportAdoptionConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAdoptionItem {
    pub id: String,
    pub asset_type: AssetType,
    pub source_path: PathBuf,
    pub mode: ImportAdoptionMode,
    pub source_snapshots: Vec<ImportSourceSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSourceSnapshot {
    pub path: PathBuf,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAdoptionConflict {
    pub source_path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportApplyResult {
    pub plan_id: String,
    pub copied_assets: usize,
    pub adopted_assets: usize,
}

pub fn preview_import_adoption(
    library_home: &Path,
    project_dir: &Path,
    target: &str,
    scope: SyncScope,
    selections: Vec<ImportAdoptionSelection>,
) -> Result<ImportAdoptionPlan> {
    let sync_root = sync_root(library_home, project_dir, scope)?;
    let lockfile_path = match scope {
        SyncScope::Project => project_dir.join(".flowmint.lock"),
        SyncScope::GlobalUser => library_home.join("global-sync.lock"),
    };
    let mut items = Vec::new();
    let mut conflicts = Vec::new();

    for selection in selections {
        if library_destination(library_home, selection.asset_type, &selection.id).exists() {
            conflicts.push(ImportAdoptionConflict {
                source_path: selection.source_path,
                message: format!(
                    "asset '{}' already exists in the Flowmint library",
                    selection.id
                ),
            });
            continue;
        }

        let snapshots = match source_snapshots(&selection.source_path) {
            Ok(snapshots) => snapshots,
            Err(error) => {
                conflicts.push(ImportAdoptionConflict {
                    source_path: selection.source_path,
                    message: error.to_string(),
                });
                continue;
            }
        };

        items.push(ImportAdoptionItem {
            id: selection.id,
            asset_type: selection.asset_type,
            source_path: selection.source_path,
            mode: selection.mode,
            source_snapshots: snapshots,
        });
    }

    let plan_id = build_plan_id(target, scope, &sync_root, &items, &conflicts);
    Ok(ImportAdoptionPlan {
        plan_id,
        target: target.to_string(),
        scope,
        sync_root,
        lockfile_path,
        items,
        conflicts,
    })
}

pub fn apply_import_adoption(
    library_home: &Path,
    _project_dir: &Path,
    plan: &ImportAdoptionPlan,
) -> Result<ImportApplyResult> {
    if !plan.conflicts.is_empty() {
        return Err(FlowmintError::SyncConflicts {
            plan_id: plan.plan_id.clone(),
            messages: plan
                .conflicts
                .iter()
                .map(|conflict| conflict.message.clone())
                .collect(),
        });
    }

    for item in &plan.items {
        if source_snapshots(&item.source_path)? != item.source_snapshots {
            return Err(FlowmintError::SyncPlanChanged {
                plan_id: plan.plan_id.clone(),
            });
        }
    }

    let mut copied_assets = 0;
    let mut adopted_assets = 0;
    let mut lock_records = Vec::new();

    for item in &plan.items {
        import_item(library_home, item)?;
        match item.mode {
            ImportAdoptionMode::CopyIntoLibrary => copied_assets += 1,
            ImportAdoptionMode::AdoptIntoFlowmint => {
                adopted_assets += 1;
                lock_records.extend(lock_records_for_item(plan, item)?);
            }
        }
    }

    if !lock_records.is_empty() {
        merge_lockfile_records_path(&plan.lockfile_path, &lock_records)?;
    }

    Ok(ImportApplyResult {
        plan_id: plan.plan_id.clone(),
        copied_assets,
        adopted_assets,
    })
}

fn import_item(library_home: &Path, item: &ImportAdoptionItem) -> Result<()> {
    match item.asset_type {
        AssetType::Prompt => {
            create_prompt(library_home, prompt_from_source(item)?)?;
        }
        AssetType::Skill => {
            create_skill(library_home, skill_from_source(item)?)?;
        }
        AssetType::InstructionRule => {
            create_rule(library_home, instruction_rule_from_source(item)?)?;
        }
        AssetType::CommandRule => {
            create_rule(library_home, command_rule_from_source(item)?)?;
        }
        AssetType::Playbook => {
            return Err(FlowmintError::InvalidAsset {
                messages: vec![
                    "playbook import adoption is not supported from target files yet".to_string(),
                ],
            });
        }
    };
    Ok(())
}

fn prompt_from_source(item: &ImportAdoptionItem) -> Result<PromptAsset> {
    let content = read_text(&item.source_path)?;
    Ok(PromptAsset {
        id: item.id.clone(),
        name: title_from_id(&item.id),
        description: parse_toml_string_value(&content, "description"),
        tags: Vec::new(),
        variables: Vec::new(),
        body: parse_gemini_prompt_body(&content).unwrap_or(content),
    })
}

fn skill_from_source(item: &ImportAdoptionItem) -> Result<SkillAsset> {
    let skill_md = read_text(&item.source_path.join("SKILL.md"))?;
    Ok(SkillAsset {
        id: item.id.clone(),
        name: title_from_skill_md(&skill_md).unwrap_or_else(|| title_from_id(&item.id)),
        description: None,
        tags: Vec::new(),
        root_dir: PathBuf::new(),
        skill_md,
        metadata: None,
        files: Vec::new(),
    })
}

fn instruction_rule_from_source(item: &ImportAdoptionItem) -> Result<RuleAsset> {
    let content = read_text(&item.source_path)?;
    let (path_globs, body) = parse_paths_frontmatter(&content);
    Ok(RuleAsset {
        id: item.id.clone(),
        name: title_from_id(&item.id),
        description: None,
        tags: Vec::new(),
        rule_kind: RuleKind::Instruction,
        path_globs,
        command_rule: None,
        target_compatibility: Vec::new(),
        body,
    })
}

fn command_rule_from_source(item: &ImportAdoptionItem) -> Result<RuleAsset> {
    let content = read_text(&item.source_path)?;
    let prefix = parse_pattern_array(&content);
    if prefix.is_empty() {
        return Err(FlowmintError::InvalidAsset {
            messages: vec!["command rule import requires a pattern array".to_string()],
        });
    }
    Ok(RuleAsset {
        id: item.id.clone(),
        name: title_from_id(&item.id),
        description: None,
        tags: Vec::new(),
        rule_kind: RuleKind::Command,
        path_globs: Vec::new(),
        command_rule: Some(CommandRule {
            prefix,
            decision: parse_command_decision(&content),
        }),
        target_compatibility: vec!["codex".to_string()],
        body: content,
    })
}

fn source_snapshots(path: &Path) -> Result<Vec<ImportSourceSnapshot>> {
    let mut snapshots = Vec::new();
    if path.is_file() {
        push_snapshot(path, &mut snapshots)?;
    } else if path.is_dir() {
        collect_dir_snapshots(path, &mut snapshots)?;
    } else {
        return Err(FlowmintError::Io {
            path: path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "source path not found"),
        });
    }
    snapshots.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(snapshots)
}

fn collect_dir_snapshots(dir: &Path, snapshots: &mut Vec<ImportSourceSnapshot>) -> Result<()> {
    for entry in std::fs::read_dir(dir).map_err(|source| FlowmintError::io(dir, source))? {
        let entry = entry.map_err(|source| FlowmintError::io(dir, source))?;
        let path = entry.path();
        if path.is_dir() {
            collect_dir_snapshots(&path, snapshots)?;
        } else if path.is_file() {
            push_snapshot(&path, snapshots)?;
        }
    }
    Ok(())
}

fn push_snapshot(path: &Path, snapshots: &mut Vec<ImportSourceSnapshot>) -> Result<()> {
    let content = std::fs::read(path).map_err(|source| FlowmintError::io(path, source))?;
    snapshots.push(ImportSourceSnapshot {
        path: path.to_path_buf(),
        content_hash: content_hash(&content),
    });
    Ok(())
}

fn lock_records_for_item(
    plan: &ImportAdoptionPlan,
    item: &ImportAdoptionItem,
) -> Result<Vec<PlannedLockRecord>> {
    item.source_snapshots
        .iter()
        .map(|snapshot| {
            let output_path = snapshot
                .path
                .strip_prefix(&plan.sync_root)
                .map_err(|_| FlowmintError::InvalidAsset {
                    messages: vec!["adopted source path is outside the sync root".to_string()],
                })?
                .to_string_lossy()
                .replace('\\', "/");
            Ok(PlannedLockRecord {
                target: plan.target.clone(),
                scope: plan.scope,
                asset_type: asset_type_label(item.asset_type).to_string(),
                asset_id: item.id.clone(),
                source_hash: snapshot.content_hash.clone(),
                output_path,
                output_hash: snapshot.content_hash.clone(),
            })
        })
        .collect()
}

fn library_destination(library_home: &Path, asset_type: AssetType, id: &str) -> PathBuf {
    match asset_type {
        AssetType::Prompt => library_home.join("prompts").join(format!("{id}.md")),
        AssetType::Skill => library_home.join("skills").join(id),
        AssetType::Playbook => library_home.join("playbooks").join(format!("{id}.md")),
        AssetType::InstructionRule | AssetType::CommandRule => {
            library_home.join("rules").join(format!("{id}.md"))
        }
    }
}

fn sync_root(library_home: &Path, project_dir: &Path, scope: SyncScope) -> Result<PathBuf> {
    match scope {
        SyncScope::Project => Ok(project_dir.to_path_buf()),
        SyncScope::GlobalUser => crate::store::global_user_home_dir(library_home),
    }
}

fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|source| FlowmintError::io(path, source))
}

fn title_from_id(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_from_skill_md(skill_md: &str) -> Option<String> {
    skill_md
        .lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(str::trim))
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_gemini_prompt_body(content: &str) -> Option<String> {
    let (_, rest) = content.split_once("prompt = ")?;
    let rest = rest.trim_start();
    if let Some(rest) = rest.strip_prefix("\"\"\"") {
        return rest
            .split_once("\"\"\"")
            .map(|(body, _)| body.trim_matches('\n').to_string());
    }
    rest.strip_prefix('"')
        .and_then(|value| value.split_once('"'))
        .map(|(body, _)| body.to_string())
}

fn parse_toml_string_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    content.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)
            .and_then(|value| value.trim().strip_prefix('"'))
            .and_then(|value| value.strip_suffix('"'))
            .map(|value| value.replace("\\\"", "\"").replace("\\\\", "\\"))
    })
}

fn parse_paths_frontmatter(content: &str) -> (Vec<String>, String) {
    let Some(rest) = content.strip_prefix("---\n") else {
        return (Vec::new(), content.to_string());
    };
    let Some((frontmatter, body)) = rest.split_once("\n---\n") else {
        return (Vec::new(), content.to_string());
    };

    let mut paths = Vec::new();
    let mut in_paths = false;
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed == "paths:" {
            in_paths = true;
            continue;
        }
        if in_paths {
            if let Some(value) = trimmed.strip_prefix("- ") {
                paths.push(value.trim().trim_matches('"').to_string());
            } else if !trimmed.is_empty() {
                in_paths = false;
            }
        }
    }
    (paths, body.to_string())
}

fn parse_pattern_array(content: &str) -> Vec<String> {
    let Some(line) = content.lines().find(|line| line.contains("pattern = [")) else {
        return Vec::new();
    };
    let Some(values) = line
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(values, _)| values)
    else {
        return Vec::new();
    };
    values
        .split(',')
        .filter_map(|value| {
            let value = value.trim().trim_matches('"');
            (!value.is_empty()).then(|| value.to_string())
        })
        .collect()
}

fn parse_command_decision(content: &str) -> CommandRuleDecision {
    match parse_toml_string_value(content, "decision").as_deref() {
        Some("allow") => CommandRuleDecision::Allow,
        Some("forbid") | Some("forbidden") => CommandRuleDecision::Forbid,
        _ => CommandRuleDecision::Prompt,
    }
}

fn asset_type_label(asset_type: AssetType) -> &'static str {
    match asset_type {
        AssetType::Prompt => "prompt",
        AssetType::Skill => "skill",
        AssetType::Playbook => "playbook",
        AssetType::InstructionRule => "instruction-rule",
        AssetType::CommandRule => "command-rule",
    }
}

fn build_plan_id(
    target: &str,
    scope: SyncScope,
    sync_root: &Path,
    items: &[ImportAdoptionItem],
    conflicts: &[ImportAdoptionConflict],
) -> String {
    let mut hasher = DefaultHasher::new();
    target.hash(&mut hasher);
    scope.hash(&mut hasher);
    sync_root.hash(&mut hasher);
    items.hash(&mut hasher);
    conflicts.hash(&mut hasher);
    format!("import-plan-{:016x}", hasher.finish())
}

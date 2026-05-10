use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::asset::id::is_safe_asset_id;
use crate::asset::model::{
    AssetDetail, AssetType, CommandRule, CommandRuleDecision, CreateAssetInput, PlaybookAsset,
    PromptAsset, RuleAsset, RuleKind, SkillAsset, SkillFile, SkillFileKind, SkillMetadata,
};
use crate::asset::store::create_asset;
use crate::error::{FlowmintError, Result};
use crate::import::ImportConfidence;

const PLAYBOOK_BEGIN: &str = "<!-- FLOWMINT:PLAYBOOK:BEGIN\n";
const PLAYBOOK_END: &str = "\nFLOWMINT:PLAYBOOK:END -->";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteImportProvider {
    PublicGithub,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportSource {
    pub provider: RemoteImportProvider,
    pub owner: String,
    pub repo: String,
    pub ref_name: String,
    pub commit_sha: String,
    pub root_path: String,
    pub canonical_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFileEntry {
    pub path: PathBuf,
    pub content: String,
    pub size_bytes: u64,
    pub blob_sha: String,
    pub source_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportCandidate {
    pub candidate_id: String,
    pub id: String,
    pub asset_type: AssetType,
    pub confidence: ImportConfidence,
    pub source: RemoteImportSource,
    pub source_paths: Vec<PathBuf>,
    pub default_destination_id: String,
    pub collision: Option<RemoteImportCollision>,
    pub warnings: Vec<String>,
    pub importable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportCollision {
    pub asset_ref: String,
    pub library_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportSelection {
    pub candidate_id: String,
    pub destination_id: String,
    pub asset_type: AssetType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportPlan {
    pub plan_id: String,
    pub source: RemoteImportSource,
    pub items: Vec<RemoteImportPlanItem>,
    pub conflicts: Vec<RemoteImportConflict>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportPlanItem {
    pub candidate_id: String,
    pub destination_id: String,
    pub asset_type: AssetType,
    pub source_paths: Vec<PathBuf>,
    pub asset: AssetDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportConflict {
    pub candidate_id: String,
    pub destination_id: String,
    pub asset_type: AssetType,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportApplyResult {
    pub plan_id: String,
    pub imported_assets: usize,
    pub asset_refs: Vec<String>,
    pub provenance_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteImportSourceRecord {
    pub provider: RemoteImportProvider,
    pub owner: String,
    pub repo: String,
    pub ref_name: String,
    pub commit_sha: String,
    pub canonical_url: String,
    pub root_path: String,
    pub source_paths: Vec<PathBuf>,
    pub asset_type: AssetType,
    pub destination_id: String,
    pub imported_at: String,
}

pub fn scan_remote_import_candidates(
    library_home: &Path,
    source: RemoteImportSource,
    files: Vec<RemoteFileEntry>,
) -> Result<Vec<RemoteImportCandidate>> {
    let mut candidates = Vec::new();
    let mut claimed_paths = HashSet::new();

    scan_skill_candidates(
        library_home,
        &source,
        &files,
        &mut claimed_paths,
        &mut candidates,
    );
    scan_playbook_candidates(
        library_home,
        &source,
        &files,
        &mut claimed_paths,
        &mut candidates,
    );
    scan_prompt_candidates(
        library_home,
        &source,
        &files,
        &mut claimed_paths,
        &mut candidates,
    );
    scan_instruction_rule_candidates(
        library_home,
        &source,
        &files,
        &mut claimed_paths,
        &mut candidates,
    );
    scan_command_rule_candidates(
        library_home,
        &source,
        &files,
        &mut claimed_paths,
        &mut candidates,
    );

    candidates.sort_by(|left, right| {
        asset_key(left.asset_type)
            .cmp(asset_key(right.asset_type))
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.candidate_id.cmp(&right.candidate_id))
    });
    Ok(candidates)
}

pub fn preview_remote_import(
    library_home: &Path,
    source: RemoteImportSource,
    files: Vec<RemoteFileEntry>,
    selections: Vec<RemoteImportSelection>,
) -> Result<RemoteImportPlan> {
    let candidates = scan_remote_import_candidates(library_home, source.clone(), files.clone())?;
    let candidate_by_id = candidates
        .iter()
        .map(|candidate| (candidate.candidate_id.clone(), candidate))
        .collect::<BTreeMap<_, _>>();
    let file_by_path = files
        .iter()
        .map(|entry| (normalize_path(&entry.path), entry))
        .collect::<BTreeMap<_, _>>();
    let mut conflicts = Vec::new();
    let mut selected_destinations = HashSet::new();

    for selection in &selections {
        let Some(candidate) = candidate_by_id.get(&selection.candidate_id) else {
            conflicts.push(RemoteImportConflict {
                candidate_id: selection.candidate_id.clone(),
                destination_id: selection.destination_id.clone(),
                asset_type: selection.asset_type,
                message: "selected remote candidate no longer exists".to_string(),
            });
            continue;
        };

        if candidate.asset_type != selection.asset_type {
            conflicts.push(RemoteImportConflict {
                candidate_id: selection.candidate_id.clone(),
                destination_id: selection.destination_id.clone(),
                asset_type: selection.asset_type,
                message: "selected asset type does not match the remote candidate".to_string(),
            });
        }

        if !candidate.importable {
            conflicts.push(RemoteImportConflict {
                candidate_id: selection.candidate_id.clone(),
                destination_id: selection.destination_id.clone(),
                asset_type: selection.asset_type,
                message: "selected remote candidate is not importable".to_string(),
            });
        }

        if !is_safe_asset_id(&selection.destination_id) {
            conflicts.push(RemoteImportConflict {
                candidate_id: selection.candidate_id.clone(),
                destination_id: selection.destination_id.clone(),
                asset_type: selection.asset_type,
                message: "destination id must use only a-z, 0-9, hyphen, or underscore".to_string(),
            });
        }

        let destination_key = (selection.asset_type, selection.destination_id.clone());
        if !selected_destinations.insert(destination_key) {
            conflicts.push(RemoteImportConflict {
                candidate_id: selection.candidate_id.clone(),
                destination_id: selection.destination_id.clone(),
                asset_type: selection.asset_type,
                message: format!(
                    "duplicate destination id '{}' for {}",
                    selection.destination_id,
                    asset_type_label(selection.asset_type)
                ),
            });
        }

        if library_destination(
            library_home,
            selection.asset_type,
            &selection.destination_id,
        )
        .exists()
        {
            conflicts.push(RemoteImportConflict {
                candidate_id: selection.candidate_id.clone(),
                destination_id: selection.destination_id.clone(),
                asset_type: selection.asset_type,
                message: format!(
                    "{} '{}' already exists in the Flowmint library",
                    asset_type_label(selection.asset_type),
                    selection.destination_id
                ),
            });
        }
    }

    let mut items = Vec::new();
    let mut warnings = Vec::new();
    if conflicts.is_empty() {
        for selection in &selections {
            let candidate = candidate_by_id
                .get(&selection.candidate_id)
                .expect("candidate existence was checked before item build");
            warnings.extend(candidate.warnings.clone());
            items.push(RemoteImportPlanItem {
                candidate_id: selection.candidate_id.clone(),
                destination_id: selection.destination_id.clone(),
                asset_type: selection.asset_type,
                source_paths: candidate.source_paths.clone(),
                asset: asset_from_candidate(candidate, selection, &file_by_path, &source)?,
            });
        }
    }

    let plan_id = build_plan_id(&source, &selections, &items, &conflicts);
    Ok(RemoteImportPlan {
        plan_id,
        source,
        items,
        conflicts,
        warnings,
    })
}

pub fn apply_remote_import(
    library_home: &Path,
    plan: &RemoteImportPlan,
) -> Result<RemoteImportApplyResult> {
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

    let mut asset_refs = Vec::new();
    let mut provenance_paths = Vec::new();
    for item in &plan.items {
        create_asset(
            library_home,
            CreateAssetInput {
                asset: item.asset.clone(),
            },
        )?;
        asset_refs.push(format!(
            "{}:{}",
            asset_type_label(item.asset_type),
            item.destination_id
        ));
        provenance_paths.push(write_provenance(library_home, &plan.source, item)?);
    }

    Ok(RemoteImportApplyResult {
        plan_id: plan.plan_id.clone(),
        imported_assets: plan.items.len(),
        asset_refs,
        provenance_paths,
    })
}

fn scan_skill_candidates(
    library_home: &Path,
    source: &RemoteImportSource,
    files: &[RemoteFileEntry],
    claimed_paths: &mut HashSet<PathBuf>,
    candidates: &mut Vec<RemoteImportCandidate>,
) {
    let mut skill_roots = BTreeSet::new();
    for entry in files {
        if file_name(&entry.path) == Some("SKILL.md")
            && let Some(root) = entry.path.parent()
            && skill_confidence(root).is_some()
        {
            skill_roots.insert(normalize_path(root));
        }
    }

    for root in skill_roots {
        let Some(id) = root.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !is_safe_asset_id(id) {
            continue;
        }
        let Some(confidence) = skill_confidence(&root) else {
            continue;
        };
        let source_paths = files
            .iter()
            .filter(|entry| normalize_path(&entry.path).starts_with(&root))
            .map(|entry| normalize_path(&entry.path))
            .collect::<Vec<_>>();
        if source_paths.is_empty() {
            continue;
        }
        claimed_paths.extend(source_paths.iter().cloned());
        let warnings = unsupported_skill_file_warnings(&root, &source_paths);
        candidates.push(candidate(
            library_home,
            source,
            AssetType::Skill,
            id,
            confidence,
            source_paths,
            warnings,
            true,
        ));
    }
}

fn scan_playbook_candidates(
    library_home: &Path,
    source: &RemoteImportSource,
    files: &[RemoteFileEntry],
    claimed_paths: &mut HashSet<PathBuf>,
    candidates: &mut Vec<RemoteImportCandidate>,
) {
    for entry in files {
        if claimed_paths.contains(&normalize_path(&entry.path)) {
            continue;
        }
        if !entry.content.starts_with(PLAYBOOK_BEGIN) {
            continue;
        }
        let importable = parse_playbook(&entry.content).is_ok();
        let id = parse_playbook(&entry.content)
            .map(|playbook| playbook.id)
            .unwrap_or_else(|_| {
                file_stem_id(&entry.path).unwrap_or_else(|| "playbook".to_string())
            });
        if !is_safe_asset_id(&id) {
            continue;
        }
        let warnings = if importable {
            Vec::new()
        } else {
            vec!["Flowmint playbook metadata JSON is invalid".to_string()]
        };
        claimed_paths.insert(normalize_path(&entry.path));
        candidates.push(candidate(
            library_home,
            source,
            AssetType::Playbook,
            &id,
            ImportConfidence::High,
            vec![normalize_path(&entry.path)],
            warnings,
            importable,
        ));
    }
}

fn scan_prompt_candidates(
    library_home: &Path,
    source: &RemoteImportSource,
    files: &[RemoteFileEntry],
    claimed_paths: &mut HashSet<PathBuf>,
    candidates: &mut Vec<RemoteImportCandidate>,
) {
    for entry in files {
        let path = normalize_path(&entry.path);
        if claimed_paths.contains(&path) {
            continue;
        }
        let Some(id) = file_stem_id(&path) else {
            continue;
        };
        let confidence = if is_high_confidence_prompt_path(&path) {
            Some(ImportConfidence::High)
        } else if path.starts_with("prompts") && extension(&path) == Some("md") {
            Some(ImportConfidence::Low)
        } else {
            None
        };
        let Some(confidence) = confidence else {
            continue;
        };
        if !is_safe_asset_id(&id) {
            continue;
        }
        claimed_paths.insert(path.clone());
        candidates.push(candidate(
            library_home,
            source,
            AssetType::Prompt,
            &id,
            confidence,
            vec![path],
            Vec::new(),
            true,
        ));
    }
}

fn is_high_confidence_prompt_path(path: &Path) -> bool {
    (path.starts_with(".claude/commands") && extension(path) == Some("md"))
        || (path.starts_with(".gemini/commands") && extension(path) == Some("toml"))
}

fn scan_instruction_rule_candidates(
    library_home: &Path,
    source: &RemoteImportSource,
    files: &[RemoteFileEntry],
    claimed_paths: &mut HashSet<PathBuf>,
    candidates: &mut Vec<RemoteImportCandidate>,
) {
    for entry in files {
        let path = normalize_path(&entry.path);
        if claimed_paths.contains(&path) {
            continue;
        }
        let (id, confidence) = if path == Path::new("AGENTS.md") {
            ("agents".to_string(), ImportConfidence::High)
        } else if path == Path::new("CLAUDE.md") {
            ("claude".to_string(), ImportConfidence::High)
        } else if path == Path::new("GEMINI.md") {
            ("gemini".to_string(), ImportConfidence::High)
        } else if path.starts_with(".claude/rules") && extension(&path) == Some("md") {
            let Some(id) = file_stem_id(&path) else {
                continue;
            };
            (id, ImportConfidence::High)
        } else if path.starts_with("rules") && extension(&path) == Some("md") {
            let Some(id) = file_stem_id(&path) else {
                continue;
            };
            (id, ImportConfidence::Medium)
        } else {
            continue;
        };
        if !is_safe_asset_id(&id) {
            continue;
        }
        claimed_paths.insert(path.clone());
        candidates.push(candidate(
            library_home,
            source,
            AssetType::InstructionRule,
            &id,
            confidence,
            vec![path],
            Vec::new(),
            true,
        ));
    }
}

fn scan_command_rule_candidates(
    library_home: &Path,
    source: &RemoteImportSource,
    files: &[RemoteFileEntry],
    claimed_paths: &mut HashSet<PathBuf>,
    candidates: &mut Vec<RemoteImportCandidate>,
) {
    for entry in files {
        let path = normalize_path(&entry.path);
        if claimed_paths.contains(&path)
            || !path.starts_with(".codex/rules")
            || extension(&path) != Some("rules")
        {
            continue;
        }
        let Some(id) = file_stem_id(&path) else {
            continue;
        };
        if !is_safe_asset_id(&id) {
            continue;
        }
        let prefix = parse_command_prefix(&entry.content);
        let importable = !prefix.is_empty();
        let warnings = if importable {
            Vec::new()
        } else {
            vec!["command rule import requires a pattern or prefix_rule array".to_string()]
        };
        claimed_paths.insert(path.clone());
        candidates.push(candidate(
            library_home,
            source,
            AssetType::CommandRule,
            &id,
            ImportConfidence::High,
            vec![path],
            warnings,
            importable,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn candidate(
    library_home: &Path,
    source: &RemoteImportSource,
    asset_type: AssetType,
    id: &str,
    confidence: ImportConfidence,
    source_paths: Vec<PathBuf>,
    warnings: Vec<String>,
    importable: bool,
) -> RemoteImportCandidate {
    let primary_path = source_paths
        .first()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();
    RemoteImportCandidate {
        candidate_id: format!(
            "{}:{}:{:016x}",
            asset_type_label(asset_type),
            id,
            stable_hash(&primary_path)
        ),
        id: id.to_string(),
        asset_type,
        confidence,
        source: source.clone(),
        source_paths,
        default_destination_id: id.to_string(),
        collision: collision_for(library_home, asset_type, id),
        warnings,
        importable,
    }
}

fn asset_from_candidate(
    candidate: &RemoteImportCandidate,
    selection: &RemoteImportSelection,
    file_by_path: &BTreeMap<PathBuf, &RemoteFileEntry>,
    source: &RemoteImportSource,
) -> Result<AssetDetail> {
    match candidate.asset_type {
        AssetType::Prompt => Ok(AssetDetail::Prompt {
            asset: prompt_from_candidate(candidate, selection, file_by_path, source)?,
        }),
        AssetType::Skill => Ok(AssetDetail::Skill {
            asset: skill_from_candidate(candidate, selection, file_by_path, source)?,
        }),
        AssetType::Playbook => Ok(AssetDetail::Playbook {
            asset: playbook_from_candidate(candidate, selection, file_by_path, source)?,
        }),
        AssetType::InstructionRule => Ok(AssetDetail::InstructionRule {
            asset: instruction_rule_from_candidate(candidate, selection, file_by_path, source)?,
        }),
        AssetType::CommandRule => Ok(AssetDetail::CommandRule {
            asset: command_rule_from_candidate(candidate, selection, file_by_path, source)?,
        }),
    }
}

fn prompt_from_candidate(
    candidate: &RemoteImportCandidate,
    selection: &RemoteImportSelection,
    file_by_path: &BTreeMap<PathBuf, &RemoteFileEntry>,
    source: &RemoteImportSource,
) -> Result<PromptAsset> {
    let entry = first_entry(candidate, file_by_path)?;
    let body = if extension(&entry.path) == Some("toml") {
        parse_gemini_prompt_body(&entry.content).unwrap_or_else(|| entry.content.clone())
    } else {
        entry.content.clone()
    };
    Ok(PromptAsset {
        id: selection.destination_id.clone(),
        name: title_from_id(&selection.destination_id),
        description: parse_toml_string_value(&entry.content, "description"),
        tags: source_tags(source),
        variables: Vec::new(),
        body,
    })
}

fn skill_from_candidate(
    candidate: &RemoteImportCandidate,
    selection: &RemoteImportSelection,
    file_by_path: &BTreeMap<PathBuf, &RemoteFileEntry>,
    source: &RemoteImportSource,
) -> Result<SkillAsset> {
    let root = skill_root(candidate)?;
    let skill_md_path = root.join("SKILL.md");
    let skill_md = file_by_path
        .get(&skill_md_path)
        .ok_or_else(|| invalid_remote_asset("remote Skill is missing SKILL.md"))?
        .content
        .clone();
    let metadata = file_by_path
        .get(&root.join("metadata.toml"))
        .map(|entry| SkillMetadata {
            raw_toml: entry.content.clone(),
        });
    let files = candidate
        .source_paths
        .iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(&root).ok()?;
            let kind = if relative.starts_with("examples") {
                Some(SkillFileKind::Example)
            } else if relative.starts_with("resources") {
                Some(SkillFileKind::Resource)
            } else {
                None
            }?;
            let entry = file_by_path.get(path)?;
            Some(SkillFile {
                path: relative.to_path_buf(),
                kind,
                content: Some(entry.content.clone()),
            })
        })
        .collect();

    Ok(SkillAsset {
        id: selection.destination_id.clone(),
        name: title_from_skill_md(&skill_md)
            .unwrap_or_else(|| title_from_id(&selection.destination_id)),
        description: None,
        tags: source_tags(source),
        root_dir: PathBuf::new(),
        skill_md,
        metadata,
        files,
    })
}

fn playbook_from_candidate(
    candidate: &RemoteImportCandidate,
    selection: &RemoteImportSelection,
    file_by_path: &BTreeMap<PathBuf, &RemoteFileEntry>,
    source: &RemoteImportSource,
) -> Result<PlaybookAsset> {
    let entry = first_entry(candidate, file_by_path)?;
    let mut playbook = parse_playbook(&entry.content)?;
    playbook.id = selection.destination_id.clone();
    merge_source_tags(&mut playbook.tags, source);
    Ok(playbook)
}

fn instruction_rule_from_candidate(
    candidate: &RemoteImportCandidate,
    selection: &RemoteImportSelection,
    file_by_path: &BTreeMap<PathBuf, &RemoteFileEntry>,
    source: &RemoteImportSource,
) -> Result<RuleAsset> {
    let entry = first_entry(candidate, file_by_path)?;
    Ok(RuleAsset {
        id: selection.destination_id.clone(),
        name: title_from_id(&selection.destination_id),
        description: None,
        tags: source_tags(source),
        rule_kind: RuleKind::Instruction,
        path_globs: Vec::new(),
        command_rule: None,
        target_compatibility: Vec::new(),
        body: entry.content.clone(),
    })
}

fn command_rule_from_candidate(
    candidate: &RemoteImportCandidate,
    selection: &RemoteImportSelection,
    file_by_path: &BTreeMap<PathBuf, &RemoteFileEntry>,
    source: &RemoteImportSource,
) -> Result<RuleAsset> {
    let entry = first_entry(candidate, file_by_path)?;
    let prefix = parse_command_prefix(&entry.content);
    if prefix.is_empty() {
        return Err(invalid_remote_asset(
            "command rule import requires a pattern or prefix_rule array",
        ));
    }
    Ok(RuleAsset {
        id: selection.destination_id.clone(),
        name: title_from_id(&selection.destination_id),
        description: None,
        tags: source_tags(source),
        rule_kind: RuleKind::Command,
        path_globs: Vec::new(),
        command_rule: Some(CommandRule {
            prefix,
            decision: parse_command_decision(&entry.content),
        }),
        target_compatibility: vec!["codex".to_string()],
        body: entry.content.clone(),
    })
}

fn first_entry<'a>(
    candidate: &RemoteImportCandidate,
    file_by_path: &'a BTreeMap<PathBuf, &'a RemoteFileEntry>,
) -> Result<&'a RemoteFileEntry> {
    let path = candidate
        .source_paths
        .first()
        .ok_or_else(|| invalid_remote_asset("remote candidate has no source paths"))?;
    file_by_path
        .get(path)
        .copied()
        .ok_or_else(|| invalid_remote_asset("remote candidate source file was not fetched"))
}

fn skill_root(candidate: &RemoteImportCandidate) -> Result<PathBuf> {
    candidate
        .source_paths
        .iter()
        .find(|path| file_name(path) == Some("SKILL.md"))
        .and_then(|path| path.parent().map(normalize_path))
        .ok_or_else(|| invalid_remote_asset("remote Skill is missing SKILL.md"))
}

fn write_provenance(
    library_home: &Path,
    source: &RemoteImportSource,
    item: &RemoteImportPlanItem,
) -> Result<PathBuf> {
    let record = RemoteImportSourceRecord {
        provider: source.provider.clone(),
        owner: source.owner.clone(),
        repo: source.repo.clone(),
        ref_name: source.ref_name.clone(),
        commit_sha: source.commit_sha.clone(),
        canonical_url: source.canonical_url.clone(),
        root_path: source.root_path.clone(),
        source_paths: item.source_paths.clone(),
        asset_type: item.asset_type,
        destination_id: item.destination_id.clone(),
        imported_at: unix_timestamp(),
    };
    let path = library_home
        .join("import-sources")
        .join(asset_type_folder(item.asset_type))
        .join(format!("{}.json", item.destination_id));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| FlowmintError::io(parent, source))?;
    }
    let content =
        serde_json::to_string_pretty(&record).map_err(|error| FlowmintError::InvalidAsset {
            messages: vec![format!(
                "remote import provenance could not serialize: {error}"
            )],
        })?;
    std::fs::write(&path, content).map_err(|source| FlowmintError::io(&path, source))?;
    Ok(path)
}

fn skill_confidence(root: &Path) -> Option<ImportConfidence> {
    if root.starts_with(".claude/skills")
        || root.starts_with(".codex/skills")
        || root.starts_with(".agents/skills")
    {
        Some(ImportConfidence::High)
    } else if root.starts_with("skills") {
        Some(ImportConfidence::Medium)
    } else {
        None
    }
}

fn unsupported_skill_file_warnings(root: &Path, source_paths: &[PathBuf]) -> Vec<String> {
    source_paths
        .iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(root).ok()?;
            let supported = relative == Path::new("SKILL.md")
                || relative == Path::new("metadata.toml")
                || relative.starts_with("examples")
                || relative.starts_with("resources");
            (!supported).then(|| {
                format!(
                    "unsupported Skill file '{}' will not be imported",
                    relative.to_string_lossy()
                )
            })
        })
        .collect()
}

fn parse_playbook(content: &str) -> Result<PlaybookAsset> {
    let Some(rest) = content.strip_prefix(PLAYBOOK_BEGIN) else {
        return Err(invalid_remote_asset(
            "missing Flowmint playbook metadata header",
        ));
    };
    let Some((metadata, _rendered)) = rest.split_once(PLAYBOOK_END) else {
        return Err(invalid_remote_asset(
            "missing Flowmint playbook metadata footer",
        ));
    };
    serde_json::from_str(metadata).map_err(|error| FlowmintError::InvalidAsset {
        messages: vec![format!(
            "Flowmint playbook metadata JSON is invalid: {error}"
        )],
    })
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

fn parse_command_prefix(content: &str) -> Vec<String> {
    for key in ["pattern", "prefix_rule"] {
        let prefix = format!("{key} = [");
        if let Some(values) = parse_string_array_line(content, &prefix) {
            return values;
        }
    }
    Vec::new()
}

fn parse_string_array_line(content: &str, prefix: &str) -> Option<Vec<String>> {
    let line = content
        .lines()
        .find(|line| line.trim().starts_with(prefix))?;
    let values = line
        .split_once('[')
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(values, _)| values)?;
    Some(
        values
            .split(',')
            .filter_map(|value| {
                let value = value.trim().trim_matches('"');
                (!value.is_empty()).then(|| value.to_string())
            })
            .collect(),
    )
}

fn parse_command_decision(content: &str) -> CommandRuleDecision {
    match parse_toml_string_value(content, "decision").as_deref() {
        Some("allow") => CommandRuleDecision::Allow,
        Some("forbid") | Some("forbidden") => CommandRuleDecision::Forbid,
        _ => CommandRuleDecision::Prompt,
    }
}

fn collision_for(
    library_home: &Path,
    asset_type: AssetType,
    id: &str,
) -> Option<RemoteImportCollision> {
    let library_path = library_destination(library_home, asset_type, id);
    library_path.exists().then_some(RemoteImportCollision {
        asset_ref: format!("{}:{id}", asset_type_label(asset_type)),
        library_path,
    })
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

fn asset_type_folder(asset_type: AssetType) -> &'static str {
    match asset_type {
        AssetType::Prompt => "prompts",
        AssetType::Skill => "skills",
        AssetType::Playbook => "playbooks",
        AssetType::InstructionRule => "instruction-rules",
        AssetType::CommandRule => "command-rules",
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

fn asset_key(asset_type: AssetType) -> &'static str {
    match asset_type {
        AssetType::Prompt => "0-prompt",
        AssetType::Skill => "1-skill",
        AssetType::Playbook => "2-playbook",
        AssetType::InstructionRule => "3-instruction-rule",
        AssetType::CommandRule => "4-command-rule",
    }
}

fn source_tags(source: &RemoteImportSource) -> Vec<String> {
    let mut tags = vec![
        "source-github".to_string(),
        format!(
            "github-{}-{}",
            normalize_tag_part(&source.owner),
            normalize_tag_part(&source.repo)
        ),
    ];
    tags.sort();
    tags.dedup();
    tags
}

fn merge_source_tags(tags: &mut Vec<String>, source: &RemoteImportSource) {
    tags.extend(source_tags(source));
    tags.sort();
    tags.dedup();
}

fn normalize_tag_part(value: &str) -> String {
    value
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn title_from_skill_md(skill_md: &str) -> Option<String> {
    skill_md
        .lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(str::trim))
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
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

fn file_stem_id(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
}

fn file_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|value| value.to_str())
}

fn extension(path: &Path) -> Option<&str> {
    path.extension().and_then(|value| value.to_str())
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    normalized
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn build_plan_id(
    source: &RemoteImportSource,
    selections: &[RemoteImportSelection],
    items: &[RemoteImportPlanItem],
    conflicts: &[RemoteImportConflict],
) -> String {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    selections.hash(&mut hasher);
    for item in items {
        item.candidate_id.hash(&mut hasher);
        item.destination_id.hash(&mut hasher);
        item.asset_type.hash(&mut hasher);
    }
    for conflict in conflicts {
        conflict.candidate_id.hash(&mut hasher);
        conflict.destination_id.hash(&mut hasher);
        conflict.asset_type.hash(&mut hasher);
        conflict.message.hash(&mut hasher);
    }
    format!("remote-import-plan-{:016x}", hasher.finish())
}

fn unix_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{seconds}")
}

fn invalid_remote_asset(message: impl Into<String>) -> FlowmintError {
    FlowmintError::InvalidAsset {
        messages: vec![message.into()],
    }
}

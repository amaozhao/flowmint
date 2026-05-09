use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::asset::id::is_safe_asset_id;
use crate::asset::model::AssetType;
use crate::error::{FlowmintError, Result};
use crate::sync::plan::SyncScope;

pub mod adopt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCandidate {
    pub id: String,
    pub asset_type: AssetType,
    pub target: String,
    pub scope: SyncScope,
    pub source_path: PathBuf,
    pub confidence: ImportConfidence,
    pub collision: Option<ImportCollision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCollision {
    pub asset_ref: String,
    pub library_path: PathBuf,
}

pub fn scan_import_candidates(
    library_home: &Path,
    project_dir: &Path,
    target: &str,
    scope: SyncScope,
) -> Result<Vec<ImportCandidate>> {
    let root = sync_root(library_home, project_dir, scope)?;
    let mut candidates = Vec::new();

    match target {
        "claude-code" => scan_claude_code(library_home, &root, scope, &mut candidates)?,
        "codex" => scan_codex(library_home, &root, scope, &mut candidates)?,
        "gemini-cli" => scan_gemini_cli(library_home, &root, scope, &mut candidates)?,
        _ => {
            return Err(FlowmintError::UnsupportedSyncTarget {
                target: target.to_string(),
            });
        }
    }

    candidates.sort_by(|left, right| {
        (
            left.target.as_str(),
            scope_key(left.scope),
            asset_key(left.asset_type),
            left.id.as_str(),
            left.source_path.to_string_lossy(),
        )
            .cmp(&(
                right.target.as_str(),
                scope_key(right.scope),
                asset_key(right.asset_type),
                right.id.as_str(),
                right.source_path.to_string_lossy(),
            ))
    });
    Ok(candidates)
}

fn scan_claude_code(
    library_home: &Path,
    root: &Path,
    scope: SyncScope,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    scan_markdown_files(
        library_home,
        "claude-code",
        scope,
        root.join(".claude/commands"),
        AssetType::Prompt,
        candidates,
    )?;
    scan_skill_dirs(
        library_home,
        "claude-code",
        scope,
        root.join(".claude/skills"),
        candidates,
    )?;
    scan_markdown_files(
        library_home,
        "claude-code",
        scope,
        root.join(".claude/rules"),
        AssetType::InstructionRule,
        candidates,
    )?;
    match scope {
        SyncScope::Project => {
            scan_instruction_file(
                library_home,
                "claude-code",
                scope,
                root.join("CLAUDE.md"),
                "claude-project-instructions",
                candidates,
            )?;
            scan_instruction_file(
                library_home,
                "claude-code",
                scope,
                root.join(".claude/CLAUDE.md"),
                "claude-project-dotclaude-instructions",
                candidates,
            )
        }
        SyncScope::GlobalUser => {
            scan_instruction_file(
                library_home,
                "claude-code",
                scope,
                root.join(".claude/CLAUDE.md"),
                "claude-global-instructions",
                candidates,
            )?;
            scan_instruction_file(
                library_home,
                "claude-code",
                scope,
                root.join("CLAUDE.md"),
                "claude-global-root-instructions",
                candidates,
            )
        }
    }
}

fn scan_codex(
    library_home: &Path,
    root: &Path,
    scope: SyncScope,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    scan_skill_dirs(
        library_home,
        "codex",
        scope,
        root.join(".codex/skills"),
        candidates,
    )?;
    scan_skill_dirs(
        library_home,
        "codex",
        scope,
        root.join(".agents/skills"),
        candidates,
    )?;
    match scope {
        SyncScope::Project => scan_instruction_file(
            library_home,
            "codex",
            scope,
            root.join("AGENTS.md"),
            "codex-project-agents",
            candidates,
        )?,
        SyncScope::GlobalUser => scan_instruction_file(
            library_home,
            "codex",
            scope,
            root.join(".codex/AGENTS.md"),
            "codex-global-agents",
            candidates,
        )?,
    }
    scan_rule_files(
        library_home,
        "codex",
        scope,
        root.join(".codex/rules"),
        candidates,
    )
}

fn scan_instruction_file(
    library_home: &Path,
    target: &str,
    scope: SyncScope,
    path: PathBuf,
    id: &str,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    if !path.is_file() {
        return Ok(());
    }
    if is_safe_asset_id(id) {
        candidates.push(import_candidate(
            library_home,
            CandidateSource {
                target,
                scope,
                asset_type: AssetType::InstructionRule,
                id,
                source_path: path,
                confidence: ImportConfidence::High,
            },
        ));
    }
    Ok(())
}

fn scan_gemini_cli(
    library_home: &Path,
    root: &Path,
    scope: SyncScope,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    scan_toml_files(
        library_home,
        "gemini-cli",
        scope,
        root.join(".gemini/commands"),
        AssetType::Prompt,
        candidates,
    )
}

fn scan_markdown_files(
    library_home: &Path,
    target: &str,
    scope: SyncScope,
    dir: PathBuf,
    asset_type: AssetType,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    scan_files_with_extension(
        library_home,
        target,
        scope,
        dir,
        "md",
        asset_type,
        candidates,
    )
}

fn scan_toml_files(
    library_home: &Path,
    target: &str,
    scope: SyncScope,
    dir: PathBuf,
    asset_type: AssetType,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    scan_files_with_extension(
        library_home,
        target,
        scope,
        dir,
        "toml",
        asset_type,
        candidates,
    )
}

fn scan_rule_files(
    library_home: &Path,
    target: &str,
    scope: SyncScope,
    dir: PathBuf,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    scan_files_with_extension(
        library_home,
        target,
        scope,
        dir,
        "rules",
        AssetType::CommandRule,
        candidates,
    )
}

fn scan_files_with_extension(
    library_home: &Path,
    target: &str,
    scope: SyncScope,
    dir: PathBuf,
    extension: &str,
    asset_type: AssetType,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&dir).map_err(|source| FlowmintError::io(&dir, source))? {
        let entry = entry.map_err(|source| FlowmintError::io(&dir, source))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some(extension) {
            continue;
        }
        let Some(id) = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if is_safe_asset_id(&id) {
            candidates.push(import_candidate(
                library_home,
                CandidateSource {
                    target,
                    scope,
                    asset_type,
                    id: &id,
                    source_path: path,
                    confidence: ImportConfidence::High,
                },
            ));
        }
    }
    Ok(())
}

fn scan_skill_dirs(
    library_home: &Path,
    target: &str,
    scope: SyncScope,
    dir: PathBuf,
    candidates: &mut Vec<ImportCandidate>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&dir).map_err(|source| FlowmintError::io(&dir, source))? {
        let entry = entry.map_err(|source| FlowmintError::io(&dir, source))?;
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").is_file() {
            continue;
        }
        let Some(id) = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if is_safe_asset_id(&id) {
            candidates.push(import_candidate(
                library_home,
                CandidateSource {
                    target,
                    scope,
                    asset_type: AssetType::Skill,
                    id: &id,
                    source_path: path,
                    confidence: ImportConfidence::High,
                },
            ));
        }
    }
    Ok(())
}

fn import_candidate(library_home: &Path, source: CandidateSource<'_>) -> ImportCandidate {
    ImportCandidate {
        id: source.id.to_string(),
        asset_type: source.asset_type,
        target: source.target.to_string(),
        scope: source.scope,
        source_path: source.source_path,
        confidence: source.confidence,
        collision: collision_for(library_home, source.asset_type, source.id),
    }
}

struct CandidateSource<'a> {
    target: &'a str,
    scope: SyncScope,
    asset_type: AssetType,
    id: &'a str,
    source_path: PathBuf,
    confidence: ImportConfidence,
}

fn collision_for(library_home: &Path, asset_type: AssetType, id: &str) -> Option<ImportCollision> {
    let (asset_ref, library_path) = match asset_type {
        AssetType::Prompt => (
            format!("prompt:{id}"),
            library_home.join("prompts").join(format!("{id}.md")),
        ),
        AssetType::Skill => (format!("skill:{id}"), library_home.join("skills").join(id)),
        AssetType::Playbook => (
            format!("playbook:{id}"),
            library_home.join("playbooks").join(format!("{id}.md")),
        ),
        AssetType::InstructionRule => (
            format!("instruction-rule:{id}"),
            library_home.join("rules").join(format!("{id}.md")),
        ),
        AssetType::CommandRule => (
            format!("command-rule:{id}"),
            library_home.join("rules").join(format!("{id}.md")),
        ),
    };

    library_path.exists().then_some(ImportCollision {
        asset_ref,
        library_path,
    })
}

fn sync_root(library_home: &Path, project_dir: &Path, scope: SyncScope) -> Result<PathBuf> {
    match scope {
        SyncScope::Project => Ok(project_dir.to_path_buf()),
        SyncScope::GlobalUser => crate::store::global_user_home_dir(library_home),
    }
}

fn scope_key(scope: SyncScope) -> &'static str {
    match scope {
        SyncScope::Project => "project",
        SyncScope::GlobalUser => "global-user",
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

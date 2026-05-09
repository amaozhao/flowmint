use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use crate::asset::id::is_safe_asset_id;
use crate::asset::model::{PromptAsset, RuleAsset, RuleKind};
use crate::asset::prompt::get_prompt;
use crate::asset::rule::get_rule;
use crate::error::Result;
use crate::exporters::claude_code::PlannedFile;
use crate::fs_safety::{parent_is_writable, path_is_inside};
use crate::project::global_profiles::{global_sync_profiles_path, load_global_sync_profiles};
use crate::project::manifest::{
    ProjectExportProfile, ProjectManifest, load_project_manifest, manifest_path,
};
use crate::sync::conflict::{SyncConflict, SyncConflictKind};
use crate::sync::diff::{content_hash, file_hash};
use crate::sync::lockfile::{Lockfile, read_lockfile_path};
use crate::sync::plan::{SyncOperation, SyncPlan, SyncScope};

const EXPORTER: &str = "gemini-cli";
const MANAGED_BEGIN: &str = "<!-- FLOWMINT:GEMINI:BEGIN -->";
const MANAGED_END: &str = "<!-- FLOWMINT:GEMINI:END -->";

pub(crate) fn build_gemini_cli_sync_for_scope(
    library_home: &Path,
    project_dir: &Path,
    scope: SyncScope,
) -> Result<GeminiCliSync> {
    let context = build_context(library_home, project_dir, scope)?;
    let lock = read_lockfile_path(&context.lockfile_path)?;
    let mut dirs = BTreeSet::new();
    let mut files = Vec::new();
    let mut conflicts = Vec::new();

    collect_prompt_exports(
        library_home,
        &context,
        &mut dirs,
        &mut files,
        &mut conflicts,
    );
    collect_instruction_rules_block(library_home, &context, &mut files, &mut conflicts)?;
    collect_unsupported_asset_conflicts(&context, &mut conflicts);

    let desired_paths = files
        .iter()
        .filter_map(|file| {
            file.lock_record
                .as_ref()
                .map(|record| record.output_path.clone())
        })
        .collect::<HashSet<_>>();

    let mut operations = Vec::new();
    for dir in dirs {
        plan_dir(&context.sync_root, &dir, &mut operations, &mut conflicts)?;
    }
    for file in &files {
        plan_file(
            &context.sync_root,
            &lock,
            file,
            &mut operations,
            &mut conflicts,
        )?;
    }
    plan_stale_deletes(
        &context.sync_root,
        &lock,
        EXPORTER,
        context.scope,
        &desired_paths,
        &mut operations,
        &mut conflicts,
    )?;

    Ok(GeminiCliSync {
        plan: SyncPlan::new_with_scope(
            context.sync_root,
            EXPORTER,
            context.scope,
            operations,
            conflicts,
        ),
        files,
        lockfile_path: context.lockfile_path,
    })
}

fn build_context(
    library_home: &Path,
    project_dir: &Path,
    scope: SyncScope,
) -> Result<GeminiBuildContext> {
    match scope {
        SyncScope::Project => {
            let manifest = load_project_manifest(project_dir)?;
            let attachments = project_attachments_for_target(&manifest, EXPORTER);
            Ok(GeminiBuildContext {
                scope,
                sync_root: project_dir.to_path_buf(),
                source_path: manifest_path(project_dir),
                lockfile_path: project_dir.join(".flowmint.lock"),
                attachments,
            })
        }
        SyncScope::GlobalUser => {
            let profiles = load_global_sync_profiles(library_home)?;
            let matching_profiles = profiles
                .profiles
                .iter()
                .filter(|profile| profile.target == EXPORTER && profile.scope == scope)
                .collect::<Vec<_>>();
            let attachments = if matching_profiles.is_empty() {
                let manifest = load_project_manifest(project_dir)?;
                project_attachments_for_target(&manifest, EXPORTER)
            } else {
                attachments_from_profiles(matching_profiles.into_iter())
            };
            Ok(GeminiBuildContext {
                scope,
                sync_root: crate::store::global_user_home_dir(library_home)?,
                source_path: global_sync_profiles_path(library_home),
                lockfile_path: library_home.join("global-sync.lock"),
                attachments,
            })
        }
    }
}

fn project_attachments_for_target(manifest: &ProjectManifest, target: &str) -> GeminiAttachments {
    let matching_profiles = manifest
        .exports
        .iter()
        .filter(|profile| profile.target == target && profile.scope == SyncScope::Project)
        .collect::<Vec<_>>();

    if matching_profiles.is_empty() {
        attachments_from_profiles(manifest.exports.first().into_iter())
    } else {
        attachments_from_profiles(matching_profiles.into_iter())
    }
}

fn attachments_from_profiles<'a>(
    profiles: impl Iterator<Item = &'a ProjectExportProfile>,
) -> GeminiAttachments {
    let mut attachments = GeminiAttachments::default();

    for profile in profiles {
        push_unique_all(&mut attachments.prompts, &profile.prompts);
        push_unique_all(&mut attachments.skills, &profile.skills);
        push_unique_all(&mut attachments.playbooks, &profile.playbooks);
        push_unique_all(
            &mut attachments.instruction_rules,
            &profile.instruction_rules,
        );
        push_unique_all(&mut attachments.command_rules, &profile.command_rules);
    }

    attachments
}

fn push_unique_all(target: &mut Vec<String>, values: &[String]) {
    for value in values {
        if !target.iter().any(|existing| existing == value) {
            target.push(value.clone());
        }
    }
}

fn collect_prompt_exports(
    library_home: &Path,
    context: &GeminiBuildContext,
    dirs: &mut BTreeSet<PathBuf>,
    files: &mut Vec<PlannedFile>,
    conflicts: &mut Vec<SyncConflict>,
) {
    for prompt_id in &context.attachments.prompts {
        if !is_safe_asset_id(prompt_id) {
            conflicts.push(asset_id_conflict(&context.source_path, prompt_id));
            continue;
        }

        let prompt = match get_prompt(library_home, prompt_id) {
            Ok(prompt) => prompt,
            Err(_) => {
                conflicts.push(missing_asset_conflict(
                    &context.source_path,
                    "prompt",
                    prompt_id,
                ));
                continue;
            }
        };

        let command_root = context.sync_root.join(".gemini").join("commands");
        let target_path = command_root.join(format!("{}.toml", prompt.id));
        dirs.insert(context.sync_root.join(".gemini"));
        dirs.insert(command_root);
        files.push(PlannedFile::generated(
            &context.sync_root,
            target_path,
            render_prompt_command(&prompt).into_bytes(),
            EXPORTER,
            context.scope,
            "prompt",
            &prompt.id,
        ));
    }
}

fn collect_instruction_rules_block(
    library_home: &Path,
    context: &GeminiBuildContext,
    files: &mut Vec<PlannedFile>,
    conflicts: &mut Vec<SyncConflict>,
) -> Result<()> {
    if context.attachments.instruction_rules.is_empty() {
        return Ok(());
    }

    let mut rules = Vec::new();
    for rule_id in &context.attachments.instruction_rules {
        if !is_safe_asset_id(rule_id) {
            conflicts.push(asset_id_conflict(&context.source_path, rule_id));
            continue;
        }

        let rule = match get_rule(library_home, rule_id) {
            Ok(rule) => rule,
            Err(_) => {
                conflicts.push(missing_asset_conflict(
                    &context.source_path,
                    "instruction rule",
                    rule_id,
                ));
                continue;
            }
        };

        if rule.rule_kind != RuleKind::Instruction {
            conflicts.push(unsupported_mapping_conflict(
                &context.source_path,
                "instruction rule",
                rule_id,
                "is not an instruction rule.",
            ));
            continue;
        }
        rules.push(rule);
    }

    if rules.is_empty() {
        return Ok(());
    }

    let target_path = instruction_file_path(context);
    let block = render_instruction_rules_block(&rules);
    if is_symlink(&target_path) {
        conflicts.push(path_conflict(
            target_path,
            SyncConflictKind::UnsafeSymlink,
            "Target path is a symlink.",
        ));
        return Ok(());
    }

    if !target_path.exists() {
        files.push(PlannedFile::managed_instruction_file(
            target_path,
            block.into_bytes(),
        ));
        return Ok(());
    }

    let existing = std::fs::read_to_string(&target_path)
        .map_err(|source| crate::error::FlowmintError::io(&target_path, source))?;
    match render_managed_content(&existing, &block) {
        Ok(content) => files.push(PlannedFile::managed_instruction_file(
            target_path,
            content.into_bytes(),
        )),
        Err(kind) => conflicts.push(path_conflict(
            target_path,
            kind,
            "GEMINI.md has incomplete Flowmint managed markers.",
        )),
    }

    Ok(())
}

fn collect_unsupported_asset_conflicts(
    context: &GeminiBuildContext,
    conflicts: &mut Vec<SyncConflict>,
) {
    for skill_id in &context.attachments.skills {
        push_unsupported_if_safe(
            context,
            conflicts,
            "skill",
            skill_id,
            "requires local Gemini CLI Skill discovery validation before Flowmint can write it.",
        );
    }
    for playbook_id in &context.attachments.playbooks {
        push_unsupported_if_safe(
            context,
            conflicts,
            "playbook",
            playbook_id,
            "requires local Gemini CLI Skill discovery validation before Flowmint can write it.",
        );
    }
    for command_rule_id in &context.attachments.command_rules {
        push_unsupported_if_safe(
            context,
            conflicts,
            "command rule",
            command_rule_id,
            "cannot be exported to Gemini CLI because command permission rules are not supported in this phase.",
        );
    }
}

fn push_unsupported_if_safe(
    context: &GeminiBuildContext,
    conflicts: &mut Vec<SyncConflict>,
    asset_type: &str,
    asset_id: &str,
    reason: &str,
) {
    if !is_safe_asset_id(asset_id) {
        conflicts.push(asset_id_conflict(&context.source_path, asset_id));
        return;
    }

    conflicts.push(unsupported_mapping_conflict(
        &context.source_path,
        asset_type,
        asset_id,
        reason,
    ));
}

fn instruction_file_path(context: &GeminiBuildContext) -> PathBuf {
    match context.scope {
        SyncScope::Project => context.sync_root.join("GEMINI.md"),
        SyncScope::GlobalUser => context.sync_root.join(".gemini").join("GEMINI.md"),
    }
}

fn render_prompt_command(prompt: &PromptAsset) -> String {
    let mut body = prompt.body.clone();
    if !prompt.variables.is_empty() && !body.contains("{{args}}") {
        body.push_str("\n\nArguments: {{args}}");
    }

    let description = prompt.description.as_deref().unwrap_or(&prompt.name);
    let mut content = String::new();
    content.push_str("description = \"");
    content.push_str(&escape_toml_string(description));
    content.push_str("\"\n\n");
    content.push_str("prompt = \"\"\"\n");
    content.push_str(&body.replace("\"\"\"", "\\\"\\\"\\\""));
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\"\"\"\n");
    content
}

fn render_instruction_rules_block(rules: &[RuleAsset]) -> String {
    let mut block = String::from(MANAGED_BEGIN);
    block.push_str("\n## Flowmint Instruction Rules\n\n");
    for rule in rules {
        block.push_str("### ");
        block.push_str(&rule.name);
        block.push_str("\n\n");
        block.push_str("Asset ID: `");
        block.push_str(&rule.id);
        block.push_str("`\n\n");
        if !rule.path_globs.is_empty() {
            block.push_str("Paths:\n");
            for glob in &rule.path_globs {
                block.push_str("- `");
                block.push_str(glob);
                block.push_str("`\n");
            }
            block.push('\n');
        }
        block.push_str(&rule.body);
        block.push_str("\n\n");
    }
    block.push_str(MANAGED_END);
    block.push('\n');
    block
}

fn render_managed_content(
    existing: &str,
    block: &str,
) -> std::result::Result<String, SyncConflictKind> {
    let begin_count = existing.matches(MANAGED_BEGIN).count();
    let end_count = existing.matches(MANAGED_END).count();

    match (begin_count, end_count) {
        (0, 0) => {
            if existing.trim().is_empty() {
                Ok(block.to_string())
            } else {
                Ok(format!("{}\n\n{}", existing.trim_end(), block))
            }
        }
        (1, 1) => {
            let begin = existing
                .find(MANAGED_BEGIN)
                .ok_or(SyncConflictKind::IncompleteManagedBlock)?;
            let end = existing
                .find(MANAGED_END)
                .ok_or(SyncConflictKind::IncompleteManagedBlock)?;
            if end < begin {
                return Err(SyncConflictKind::IncompleteManagedBlock);
            }
            let end = end + MANAGED_END.len();
            let mut next = String::new();
            next.push_str(&existing[..begin]);
            next.push_str(block.trim_end());
            next.push_str(&existing[end..]);
            if !next.ends_with('\n') {
                next.push('\n');
            }
            Ok(next)
        }
        _ => Err(SyncConflictKind::IncompleteManagedBlock),
    }
}

fn plan_dir(
    sync_root: &Path,
    target_path: &Path,
    operations: &mut Vec<SyncOperation>,
    conflicts: &mut Vec<SyncConflict>,
) -> Result<()> {
    if !path_is_inside(sync_root, target_path) {
        conflicts.push(path_conflict(
            target_path.to_path_buf(),
            SyncConflictKind::UnsafeAssetId,
            "Target path escapes the sync root.",
        ));
        return Ok(());
    }

    if is_symlink(target_path) {
        conflicts.push(path_conflict(
            target_path.to_path_buf(),
            SyncConflictKind::UnsafeSymlink,
            "Target path is a symlink.",
        ));
        return Ok(());
    }

    if target_path.exists() {
        if target_path.is_dir() {
            return Ok(());
        }
        conflicts.push(path_conflict(
            target_path.to_path_buf(),
            SyncConflictKind::UnmanagedTarget,
            "Expected directory path is occupied by a file.",
        ));
        return Ok(());
    }

    if !parent_is_writable(target_path) {
        conflicts.push(path_conflict(
            target_path.to_path_buf(),
            SyncConflictKind::OutputNotWritable,
            "Output parent directory is not writable.",
        ));
        return Ok(());
    }

    operations.push(SyncOperation::CreateDir {
        target_path: target_path.to_path_buf(),
    });
    Ok(())
}

fn plan_file(
    sync_root: &Path,
    lock: &Lockfile,
    file: &PlannedFile,
    operations: &mut Vec<SyncOperation>,
    conflicts: &mut Vec<SyncConflict>,
) -> Result<()> {
    if !path_is_inside(sync_root, &file.target_path) {
        conflicts.push(path_conflict(
            file.target_path.clone(),
            SyncConflictKind::UnsafeAssetId,
            "Target path escapes the sync root.",
        ));
        return Ok(());
    }

    if is_symlink(&file.target_path) {
        conflicts.push(path_conflict(
            file.target_path.clone(),
            SyncConflictKind::UnsafeSymlink,
            "Target path is a symlink.",
        ));
        return Ok(());
    }

    if !parent_is_writable(&file.target_path) {
        conflicts.push(path_conflict(
            file.target_path.clone(),
            SyncConflictKind::OutputNotWritable,
            "Output parent directory is not writable.",
        ));
        return Ok(());
    }

    let new_hash = content_hash(&file.content);
    if !file.target_path.exists() {
        operations.push(SyncOperation::CreateFile {
            target_path: file.target_path.clone(),
            content_hash: new_hash,
        });
        return Ok(());
    }

    if file.target_path.is_dir() {
        conflicts.push(path_conflict(
            file.target_path.clone(),
            SyncConflictKind::UnmanagedTarget,
            "Expected file path is occupied by a directory.",
        ));
        return Ok(());
    }

    if file.allow_existing_unmanaged_update {
        plan_existing_file_update(file.target_path.clone(), new_hash, operations)?;
        return Ok(());
    }

    let Some(relative_path) = relative_output_path(sync_root, &file.target_path) else {
        conflicts.push(path_conflict(
            file.target_path.clone(),
            SyncConflictKind::UnsafeAssetId,
            "Target path escapes the sync root.",
        ));
        return Ok(());
    };

    let Some(record) = file.lock_record.as_ref() else {
        conflicts.push(path_conflict(
            file.target_path.clone(),
            SyncConflictKind::UnmanagedTarget,
            "Target file exists outside Flowmint management.",
        ));
        return Ok(());
    };

    let Some(lock_entry) = lock
        .entries
        .get(&relative_path)
        .filter(|entry| entry.target == record.target && entry.scope == record.scope)
    else {
        conflicts.push(path_conflict(
            file.target_path.clone(),
            SyncConflictKind::UnmanagedTarget,
            "Target file exists outside Flowmint management.",
        ));
        return Ok(());
    };

    let current_hash = file_hash(&file.target_path)?;
    if current_hash != lock_entry.output_hash {
        conflicts.push(path_conflict(
            file.target_path.clone(),
            SyncConflictKind::ModifiedGeneratedFile,
            "Generated file has changed since the last Flowmint sync.",
        ));
        return Ok(());
    }

    if current_hash == new_hash {
        operations.push(SyncOperation::Noop {
            target_path: file.target_path.clone(),
            reason: "Already up to date".to_string(),
        });
    } else {
        operations.push(SyncOperation::UpdateFile {
            target_path: file.target_path.clone(),
            previous_hash: Some(current_hash),
            new_hash,
        });
    }

    Ok(())
}

fn plan_existing_file_update(
    target_path: PathBuf,
    new_hash: String,
    operations: &mut Vec<SyncOperation>,
) -> Result<()> {
    let current_hash = file_hash(&target_path)?;
    if current_hash == new_hash {
        operations.push(SyncOperation::Noop {
            target_path,
            reason: "Already up to date".to_string(),
        });
    } else {
        operations.push(SyncOperation::UpdateFile {
            target_path,
            previous_hash: Some(current_hash),
            new_hash,
        });
    }
    Ok(())
}

fn plan_stale_deletes(
    sync_root: &Path,
    lock: &Lockfile,
    target: &str,
    scope: SyncScope,
    desired_paths: &HashSet<String>,
    operations: &mut Vec<SyncOperation>,
    conflicts: &mut Vec<SyncConflict>,
) -> Result<()> {
    for (relative_path, entry) in &lock.entries {
        if entry.target != target || entry.scope != scope {
            continue;
        }

        if desired_paths.contains(relative_path) {
            continue;
        }

        let target_path = sync_root.join(relative_path);
        if !target_path.exists() {
            continue;
        }

        if is_symlink(&target_path) {
            conflicts.push(path_conflict(
                target_path,
                SyncConflictKind::UnsafeSymlink,
                "Target path is a symlink.",
            ));
            continue;
        }

        let current_hash = file_hash(&target_path)?;
        if current_hash != entry.output_hash {
            conflicts.push(path_conflict(
                target_path,
                SyncConflictKind::ModifiedGeneratedFile,
                "Generated file has changed since the last Flowmint sync.",
            ));
            continue;
        }

        operations.push(SyncOperation::DeleteGeneratedFile {
            target_path,
            previous_hash: current_hash,
        });
    }

    Ok(())
}

fn relative_output_path(sync_root: &Path, target_path: &Path) -> Option<String> {
    target_path
        .strip_prefix(sync_root)
        .ok()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn asset_id_conflict(source_path: &Path, asset_id: &str) -> SyncConflict {
    path_conflict(
        source_path.to_path_buf(),
        SyncConflictKind::UnsafeAssetId,
        &format!("Attached asset id '{asset_id}' is not safe."),
    )
}

fn missing_asset_conflict(source_path: &Path, asset_type: &str, asset_id: &str) -> SyncConflict {
    path_conflict(
        source_path.to_path_buf(),
        SyncConflictKind::MissingAsset,
        &format!("Attached {asset_type} '{asset_id}' does not exist in the Flowmint library."),
    )
}

fn unsupported_mapping_conflict(
    source_path: &Path,
    asset_type: &str,
    asset_id: &str,
    reason: &str,
) -> SyncConflict {
    path_conflict(
        source_path.to_path_buf(),
        SyncConflictKind::UnsupportedMapping,
        &format!("Attached {asset_type} '{asset_id}' {reason}"),
    )
}

fn path_conflict(target_path: PathBuf, kind: SyncConflictKind, message: &str) -> SyncConflict {
    SyncConflict {
        target_path,
        kind,
        message: message.to_string(),
    }
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(crate) struct GeminiCliSync {
    pub plan: SyncPlan,
    pub files: Vec<PlannedFile>,
    pub lockfile_path: PathBuf,
}

#[derive(Debug, Default)]
struct GeminiAttachments {
    prompts: Vec<String>,
    skills: Vec<String>,
    playbooks: Vec<String>,
    instruction_rules: Vec<String>,
    command_rules: Vec<String>,
}

struct GeminiBuildContext {
    scope: SyncScope,
    sync_root: PathBuf,
    source_path: PathBuf,
    lockfile_path: PathBuf,
    attachments: GeminiAttachments,
}

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{FlowmintError, Result};
use crate::exporters::claude_code::PlannedLockRecord;
use crate::sync::plan::SyncScope;

pub(crate) fn write_lockfile_path(
    path: &Path,
    target: &str,
    scope: SyncScope,
    records: &[PlannedLockRecord],
) -> Result<()> {
    let mut entries = read_lockfile_path(path)?
        .entries
        .into_values()
        .filter(|entry| entry.target != target || entry.scope != scope)
        .collect::<Vec<_>>();
    write_lockfile_entries(path, &mut entries, records)
}

pub(crate) fn merge_lockfile_records_path(
    path: &Path,
    records: &[PlannedLockRecord],
) -> Result<()> {
    let replacing_paths = records
        .iter()
        .map(|record| {
            (
                record.target.as_str(),
                record.scope,
                record.output_path.as_str(),
            )
        })
        .collect::<std::collections::HashSet<_>>();
    let mut entries = read_lockfile_path(path)?
        .entries
        .into_values()
        .filter(|entry| {
            !replacing_paths.contains(&(
                entry.target.as_str(),
                entry.scope,
                entry.output_path.as_str(),
            ))
        })
        .collect::<Vec<_>>();
    write_lockfile_entries(path, &mut entries, records)
}

fn write_lockfile_entries(
    path: &Path,
    entries: &mut Vec<LockEntry>,
    records: &[PlannedLockRecord],
) -> Result<()> {
    let updated_at = updated_at_value();
    entries.extend(
        records
            .iter()
            .map(|record| LockEntry::from_planned(record, &updated_at)),
    );
    entries.sort_by(|left, right| {
        (
            left.target.as_str(),
            render_scope(left.scope),
            left.output_path.as_str(),
        )
            .cmp(&(
                right.target.as_str(),
                render_scope(right.scope),
                right.output_path.as_str(),
            ))
    });

    let mut content = String::new();

    for record in entries {
        content.push_str("[[exports]]\n");
        push_toml_string(&mut content, "target", &record.target);
        push_toml_string(&mut content, "scope", render_scope(record.scope));
        push_toml_string(&mut content, "asset_type", &record.asset_type);
        push_toml_string(&mut content, "asset_id", &record.asset_id);
        push_toml_string(&mut content, "source_hash", &record.source_hash);
        push_toml_string(&mut content, "output_path", &record.output_path);
        push_toml_string(&mut content, "content_hash", &record.output_hash);
        push_toml_string(&mut content, "output_hash", &record.output_hash);
        push_toml_string(&mut content, "generated_by", &record.generated_by);
        push_toml_string(&mut content, "updated_at", &record.updated_at);
        content.push('\n');
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| FlowmintError::io(parent, source))?;
    }
    std::fs::write(path, content).map_err(|source| FlowmintError::io(path, source))
}

pub(crate) fn read_lockfile_path(path: &Path) -> Result<Lockfile> {
    if !path.exists() {
        return Ok(Lockfile::default());
    }

    let content =
        std::fs::read_to_string(path).map_err(|source| FlowmintError::io(path, source))?;
    Ok(parse_lockfile(&content))
}

fn parse_lockfile(content: &str) -> Lockfile {
    let mut lockfile = Lockfile::default();
    let mut entry = PartialLockEntry::default();

    for line in content.lines().map(str::trim) {
        if line == "[[exports]]" {
            insert_lock_entry(&mut lockfile, std::mem::take(&mut entry));
            continue;
        }

        let Some((key, value)) = line.split_once(" = ") else {
            continue;
        };
        match key {
            "target" => entry.target = parse_toml_string(value),
            "scope" => entry.scope = parse_toml_string(value).and_then(|value| parse_scope(&value)),
            "asset_type" => entry.asset_type = parse_toml_string(value),
            "asset_id" => entry.asset_id = parse_toml_string(value),
            "source_hash" => entry.source_hash = parse_toml_string(value),
            "output_path" => entry.output_path = parse_toml_string(value),
            "output_hash" | "content_hash" => entry.output_hash = parse_toml_string(value),
            "generated_by" => entry.generated_by = parse_toml_string(value),
            "updated_at" => entry.updated_at = parse_toml_string(value),
            _ => {}
        }
    }

    insert_lock_entry(&mut lockfile, entry);
    lockfile
}

fn insert_lock_entry(lockfile: &mut Lockfile, entry: PartialLockEntry) {
    if let (Some(output_path), Some(output_hash)) = (entry.output_path, entry.output_hash) {
        lockfile.entries.insert(
            output_path.clone(),
            LockEntry {
                target: entry.target.unwrap_or_else(|| "claude-code".to_string()),
                scope: entry.scope.unwrap_or(SyncScope::Project),
                asset_type: entry.asset_type.unwrap_or_default(),
                asset_id: entry.asset_id.unwrap_or_default(),
                source_hash: entry.source_hash.unwrap_or_else(|| output_hash.clone()),
                output_path,
                output_hash,
                generated_by: entry.generated_by.unwrap_or_else(|| "flowmint".to_string()),
                updated_at: entry.updated_at.unwrap_or_default(),
            },
        );
    }
}

fn parse_toml_string(value: &str) -> Option<String> {
    value
        .trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| value.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn push_toml_string(content: &mut String, key: &str, value: &str) {
    content.push_str(key);
    content.push_str(" = \"");
    content.push_str(&escape_toml_string(value));
    content.push_str("\"\n");
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn updated_at_value() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{seconds}")
}

fn render_scope(scope: SyncScope) -> &'static str {
    match scope {
        SyncScope::Project => "project",
        SyncScope::GlobalUser => "global-user",
    }
}

fn parse_scope(value: &str) -> Option<SyncScope> {
    match value {
        "project" => Some(SyncScope::Project),
        "global-user" => Some(SyncScope::GlobalUser),
        _ => None,
    }
}

#[derive(Debug, Default)]
pub(crate) struct Lockfile {
    pub(crate) entries: std::collections::HashMap<String, LockEntry>,
}

#[derive(Debug, Clone)]
pub(crate) struct LockEntry {
    pub(crate) target: String,
    pub(crate) scope: SyncScope,
    pub(crate) asset_type: String,
    pub(crate) asset_id: String,
    pub(crate) source_hash: String,
    pub(crate) output_path: String,
    pub(crate) output_hash: String,
    pub(crate) generated_by: String,
    pub(crate) updated_at: String,
}

impl LockEntry {
    fn from_planned(record: &PlannedLockRecord, updated_at: &str) -> Self {
        Self {
            target: record.target.clone(),
            scope: record.scope,
            asset_type: record.asset_type.clone(),
            asset_id: record.asset_id.clone(),
            source_hash: record.source_hash.clone(),
            output_path: record.output_path.clone(),
            output_hash: record.output_hash.clone(),
            generated_by: "flowmint".to_string(),
            updated_at: updated_at.to_string(),
        }
    }
}

#[derive(Default)]
struct PartialLockEntry {
    target: Option<String>,
    scope: Option<SyncScope>,
    asset_type: Option<String>,
    asset_id: Option<String>,
    source_hash: Option<String>,
    output_path: Option<String>,
    output_hash: Option<String>,
    generated_by: Option<String>,
    updated_at: Option<String>,
}

use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use flowmint_core::import::remote::{
    RemoteFileEntry, RemoteImportApplyResult, RemoteImportCandidate, RemoteImportPlan,
    RemoteImportProvider, RemoteImportSelection, RemoteImportSource, apply_remote_import,
    preview_remote_import, scan_remote_import_candidates,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_SCAN_SESSION_LIMIT: usize = 5;
const DEFAULT_PLAN_LIMIT: usize = 10;
const DEFAULT_TTL: Duration = Duration::from_secs(30 * 60);
const MAX_DIRECTORIES: usize = 50;
const MAX_FILES: usize = 200;
const MAX_FILE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 10 * 1024 * 1024;
const MAX_DEPTH: usize = 8;

#[derive(Default)]
pub struct PublicGithubImportState {
    cache: Mutex<PublicGithubImportCache>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicGithubImportScanResult {
    pub session_id: String,
    pub source: RemoteImportSource,
    pub candidates: Vec<RemoteImportCandidate>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct PublicGithubImportSession {
    source: RemoteImportSource,
    files: Vec<RemoteFileEntry>,
    warnings: Vec<String>,
}

#[derive(Debug)]
struct Timed<T> {
    created_at: Instant,
    value: T,
}

#[derive(Debug)]
pub struct PublicGithubImportCache {
    scan_sessions: BTreeMap<String, Timed<PublicGithubImportSession>>,
    scan_order: VecDeque<String>,
    plans: BTreeMap<String, Timed<RemoteImportPlan>>,
    plan_order: VecDeque<String>,
    scan_session_limit: usize,
    plan_limit: usize,
    ttl: Duration,
}

impl Default for PublicGithubImportCache {
    fn default() -> Self {
        Self::new(DEFAULT_SCAN_SESSION_LIMIT, DEFAULT_PLAN_LIMIT, DEFAULT_TTL)
    }
}

impl PublicGithubImportCache {
    fn new(scan_session_limit: usize, plan_limit: usize, ttl: Duration) -> Self {
        Self {
            scan_sessions: BTreeMap::new(),
            scan_order: VecDeque::new(),
            plans: BTreeMap::new(),
            plan_order: VecDeque::new(),
            scan_session_limit,
            plan_limit,
            ttl,
        }
    }

    fn insert_scan_session(&mut self, session: PublicGithubImportSession) -> String {
        self.evict_expired();
        let session_id = format!("github-scan-{}", unique_suffix());
        self.scan_sessions.insert(
            session_id.clone(),
            Timed {
                created_at: Instant::now(),
                value: session,
            },
        );
        self.scan_order.push_back(session_id.clone());
        self.evict_oldest_scan_sessions();
        session_id
    }

    fn get_scan_session(&mut self, session_id: &str) -> Option<PublicGithubImportSession> {
        self.evict_expired();
        self.scan_sessions
            .get(session_id)
            .map(|timed| timed.value.clone())
    }

    fn insert_plan(&mut self, plan: RemoteImportPlan) {
        self.evict_expired();
        let plan_id = plan.plan_id.clone();
        self.plans.insert(
            plan_id.clone(),
            Timed {
                created_at: Instant::now(),
                value: plan,
            },
        );
        self.plan_order.push_back(plan_id);
        self.evict_oldest_plans();
    }

    fn remove_plan(&mut self, plan_id: &str) -> Option<RemoteImportPlan> {
        self.evict_expired();
        self.plan_order.retain(|value| value != plan_id);
        self.plans.remove(plan_id).map(|timed| timed.value)
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        self.scan_sessions
            .retain(|_, timed| now.duration_since(timed.created_at) <= self.ttl);
        self.plans
            .retain(|_, timed| now.duration_since(timed.created_at) <= self.ttl);
        self.scan_order
            .retain(|session_id| self.scan_sessions.contains_key(session_id));
        self.plan_order
            .retain(|plan_id| self.plans.contains_key(plan_id));
    }

    fn evict_oldest_scan_sessions(&mut self) {
        while self.scan_sessions.len() > self.scan_session_limit {
            if let Some(session_id) = self.scan_order.pop_front() {
                self.scan_sessions.remove(&session_id);
            } else {
                break;
            }
        }
    }

    fn evict_oldest_plans(&mut self) {
        while self.plans.len() > self.plan_limit {
            if let Some(plan_id) = self.plan_order.pop_front() {
                self.plans.remove(&plan_id);
            } else {
                break;
            }
        }
    }

    #[cfg(test)]
    fn insert_scan_session_for_test(&mut self, session_id: &str) {
        self.evict_expired();
        self.scan_sessions.insert(
            session_id.to_string(),
            Timed {
                created_at: Instant::now(),
                value: PublicGithubImportSession {
                    source: RemoteImportSource {
                        provider: RemoteImportProvider::PublicGithub,
                        owner: "example".to_string(),
                        repo: "repo".to_string(),
                        ref_name: "main".to_string(),
                        commit_sha: "abc123".to_string(),
                        root_path: "".to_string(),
                        canonical_url: "https://github.com/example/repo/tree/main".to_string(),
                    },
                    files: Vec::new(),
                    warnings: Vec::new(),
                },
            },
        );
        self.scan_order.push_back(session_id.to_string());
        self.evict_oldest_scan_sessions();
    }

    #[cfg(test)]
    fn has_scan_session_for_test(&self, session_id: &str) -> bool {
        self.scan_sessions.contains_key(session_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedGithubUrl {
    owner: String,
    repo: String,
    ref_name: Option<String>,
    root_path: String,
}

trait GithubHttpClient {
    fn get_json(&self, url: &str) -> Result<Value, String>;
    fn get_text(&self, url: &str, max_bytes: usize) -> Result<String, String>;
}

struct ReqwestGithubHttpClient {
    client: reqwest::blocking::Client,
}

impl ReqwestGithubHttpClient {
    fn new() -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Flowmint/0.1")
            .build()
            .map_err(|error| format!("failed to initialize GitHub HTTP client: {error}"))?;
        Ok(Self { client })
    }
}

impl GithubHttpClient for ReqwestGithubHttpClient {
    fn get_json(&self, url: &str) -> Result<Value, String> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .map_err(|error| format!("GitHub request failed: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            return Err(github_status_error(status.as_u16()));
        }
        response
            .json::<Value>()
            .map_err(|error| format!("GitHub response JSON is invalid: {error}"))
    }

    fn get_text(&self, url: &str, max_bytes: usize) -> Result<String, String> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|error| format!("GitHub raw file request failed: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            return Err(github_status_error(status.as_u16()));
        }
        let bytes = response
            .bytes()
            .map_err(|error| format!("GitHub raw file bytes could not be read: {error}"))?;
        if bytes.len() > max_bytes {
            return Err(format!("file is larger than {max_bytes} bytes"));
        }
        String::from_utf8(bytes.to_vec())
            .map_err(|_| "file is binary or not valid UTF-8".to_string())
    }
}

#[tauri::command]
pub fn scan_public_github_import(
    state: tauri::State<'_, PublicGithubImportState>,
    url: String,
) -> Result<PublicGithubImportScanResult, String> {
    let client = ReqwestGithubHttpClient::new()?;
    let session = fetch_public_github_import_session(&client, &url)?;
    let library_home =
        flowmint_core::store::default_home_dir().map_err(|error| error.to_string())?;
    let candidates =
        scan_remote_import_candidates(&library_home, session.source.clone(), session.files.clone())
            .map_err(|error| error.to_string())?;
    let mut cache = state
        .cache
        .lock()
        .map_err(|_| "GitHub import cache is unavailable".to_string())?;
    let session_id = cache.insert_scan_session(session.clone());
    Ok(PublicGithubImportScanResult {
        session_id,
        source: session.source,
        candidates,
        warnings: session.warnings,
    })
}

#[tauri::command]
pub fn preview_public_github_import(
    state: tauri::State<'_, PublicGithubImportState>,
    session_id: String,
    selections: Vec<RemoteImportSelection>,
) -> Result<RemoteImportPlan, String> {
    let mut cache = state
        .cache
        .lock()
        .map_err(|_| "GitHub import cache is unavailable".to_string())?;
    let session = cache
        .get_scan_session(&session_id)
        .ok_or_else(|| format!("GitHub import session not found or expired: {session_id}"))?;
    let library_home =
        flowmint_core::store::default_home_dir().map_err(|error| error.to_string())?;
    let plan = preview_remote_import(&library_home, session.source, session.files, selections)
        .map_err(|error| error.to_string())?;
    if plan.conflicts.is_empty() {
        cache.insert_plan(plan.clone());
    }
    Ok(plan)
}

#[tauri::command]
pub fn apply_public_github_import(
    state: tauri::State<'_, PublicGithubImportState>,
    plan_id: String,
) -> Result<RemoteImportApplyResult, String> {
    let plan = state
        .cache
        .lock()
        .map_err(|_| "GitHub import cache is unavailable".to_string())?
        .remove_plan(&plan_id)
        .ok_or_else(|| format!("GitHub import plan not found or expired: {plan_id}"))?;
    let library_home =
        flowmint_core::store::default_home_dir().map_err(|error| error.to_string())?;
    apply_remote_import(&library_home, &plan).map_err(|error| error.to_string())
}

fn fetch_public_github_import_session(
    client: &dyn GithubHttpClient,
    url: &str,
) -> Result<PublicGithubImportSession, String> {
    let parsed = parse_public_github_url(url)?;
    let repo_url = format!(
        "https://api.github.com/repos/{}/{}",
        parsed.owner, parsed.repo
    );
    let repo = client.get_json(&repo_url)?;
    let ref_name = parsed
        .ref_name
        .clone()
        .or_else(|| {
            repo.get("default_branch")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .ok_or_else(|| "GitHub repository default branch is unavailable".to_string())?;
    let commit_url = format!(
        "https://api.github.com/repos/{}/{}/commits/{}",
        parsed.owner,
        parsed.repo,
        percent_encode_component(&ref_name)
    );
    let commit = client.get_json(&commit_url)?;
    let commit_sha = commit
        .get("sha")
        .and_then(Value::as_str)
        .ok_or_else(|| "GitHub commit SHA is unavailable".to_string())?
        .to_string();
    let mut traversal = GithubTraversal::default();
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    fetch_contents_recursive(
        client,
        &parsed.owner,
        &parsed.repo,
        &ref_name,
        &parsed.root_path,
        0,
        true,
        &mut traversal,
        &mut files,
        &mut warnings,
    )?;

    Ok(PublicGithubImportSession {
        source: RemoteImportSource {
            provider: RemoteImportProvider::PublicGithub,
            owner: parsed.owner.clone(),
            repo: parsed.repo.clone(),
            ref_name: ref_name.clone(),
            commit_sha,
            root_path: parsed.root_path.clone(),
            canonical_url: canonical_github_url(
                &parsed.owner,
                &parsed.repo,
                &ref_name,
                &parsed.root_path,
            ),
        },
        files,
        warnings,
    })
}

#[derive(Default)]
struct GithubTraversal {
    directories: usize,
    files: usize,
    total_bytes: usize,
}

#[allow(clippy::too_many_arguments)]
fn fetch_contents_recursive(
    client: &dyn GithubHttpClient,
    owner: &str,
    repo: &str,
    ref_name: &str,
    path: &str,
    depth: usize,
    is_root: bool,
    traversal: &mut GithubTraversal,
    files: &mut Vec<RemoteFileEntry>,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    if depth > MAX_DEPTH {
        warnings.push(format!("traversal depth cap reached at '{path}'"));
        return Ok(());
    }
    if traversal.directories >= MAX_DIRECTORIES {
        warnings.push(format!("directory traversal cap reached before '{path}'"));
        return Ok(());
    }
    traversal.directories += 1;

    let value = match client.get_json(&contents_url(owner, repo, path, ref_name)) {
        Ok(value) => value,
        Err(error) if is_root => return Err(error),
        Err(error) => {
            warnings.push(format!("skipped '{path}': {error}"));
            return Ok(());
        }
    };

    if let Some(items) = value.as_array() {
        for item in items {
            let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
            let item_path = item.get("path").and_then(Value::as_str).unwrap_or_default();
            match item_type {
                "dir" => fetch_contents_recursive(
                    client,
                    owner,
                    repo,
                    ref_name,
                    item_path,
                    depth + 1,
                    false,
                    traversal,
                    files,
                    warnings,
                )?,
                "file" => fetch_file_item(client, item, traversal, files, warnings),
                _ => warnings.push(format!("skipped unsupported GitHub item '{item_path}'")),
            }
        }
    } else {
        fetch_file_item(client, &value, traversal, files, warnings);
    }

    Ok(())
}

fn fetch_file_item(
    client: &dyn GithubHttpClient,
    item: &Value,
    traversal: &mut GithubTraversal,
    files: &mut Vec<RemoteFileEntry>,
    warnings: &mut Vec<String>,
) {
    let path = item.get("path").and_then(Value::as_str).unwrap_or_default();
    if traversal.files >= MAX_FILES {
        warnings.push(format!("file cap reached before '{path}'"));
        return;
    }
    let size = item.get("size").and_then(Value::as_u64).unwrap_or_default() as usize;
    if size > MAX_FILE_BYTES {
        warnings.push(format!("skipped '{path}': file too large"));
        return;
    }
    if traversal.total_bytes.saturating_add(size) > MAX_TOTAL_BYTES {
        warnings.push(format!("skipped '{path}': total fetched text cap reached"));
        return;
    }
    let Some(download_url) = item.get("download_url").and_then(Value::as_str) else {
        warnings.push(format!(
            "skipped '{path}': file download URL is unavailable"
        ));
        return;
    };

    match client.get_text(download_url, MAX_FILE_BYTES) {
        Ok(content) => {
            traversal.files += 1;
            traversal.total_bytes += content.len();
            files.push(RemoteFileEntry {
                path: PathBuf::from(path),
                size_bytes: size as u64,
                content,
                blob_sha: item
                    .get("sha")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                source_url: item
                    .get("html_url")
                    .and_then(Value::as_str)
                    .unwrap_or(download_url)
                    .to_string(),
            });
        }
        Err(error) => warnings.push(format!("skipped '{path}': {error}")),
    }
}

fn parse_public_github_url(url: &str) -> Result<ParsedGithubUrl, String> {
    let url = url.trim();
    let Some(rest) = url.strip_prefix("https://github.com/") else {
        return Err("public GitHub import requires a https://github.com URL".to_string());
    };
    let (path_part, query) = rest.split_once('?').unwrap_or((rest, ""));
    let query_ref = query_value(query, "ref");
    let query_path = query_value(query, "path").unwrap_or_default();
    let segments = path_part
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return Err("GitHub URL must include owner and repository".to_string());
    }
    let owner = segments[0].to_string();
    let repo = segments[1].trim_end_matches(".git").to_string();

    if let Some(ref_name) = query_ref {
        return Ok(ParsedGithubUrl {
            owner,
            repo,
            ref_name: Some(ref_name),
            root_path: trim_slashes(&query_path).to_string(),
        });
    }

    if matches!(segments.get(2), Some(&"tree" | &"blob")) {
        let Some(ref_name) = segments.get(3) else {
            return Err("GitHub tree/blob URL must include a ref".to_string());
        };
        return Ok(ParsedGithubUrl {
            owner,
            repo,
            ref_name: Some((*ref_name).to_string()),
            root_path: segments
                .iter()
                .skip(4)
                .copied()
                .collect::<Vec<_>>()
                .join("/"),
        });
    }

    Ok(ParsedGithubUrl {
        owner,
        repo,
        ref_name: None,
        root_path: trim_slashes(&query_path).to_string(),
    })
}

fn query_value(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|part| {
        let (left, right) = part.split_once('=')?;
        (left == key).then(|| percent_decode(right))
    })
}

fn contents_url(owner: &str, repo: &str, path: &str, ref_name: &str) -> String {
    let encoded_ref = percent_encode_query(ref_name);
    if path.trim_matches('/').is_empty() {
        format!("https://api.github.com/repos/{owner}/{repo}/contents?ref={encoded_ref}")
    } else {
        format!(
            "https://api.github.com/repos/{owner}/{repo}/contents/{}?ref={encoded_ref}",
            percent_encode_path(path)
        )
    }
}

fn canonical_github_url(owner: &str, repo: &str, ref_name: &str, path: &str) -> String {
    if path.is_empty() {
        format!("https://github.com/{owner}/{repo}/tree/{ref_name}")
    } else {
        format!(
            "https://github.com/{owner}/{repo}/tree/{ref_name}/{}",
            trim_slashes(path)
        )
    }
}

fn trim_slashes(value: &str) -> &str {
    value.trim_matches('/')
}

fn percent_encode_path(path: &str) -> String {
    path.split('/')
        .map(percent_encode_component)
        .collect::<Vec<_>>()
        .join("/")
}

fn percent_encode_query(value: &str) -> String {
    percent_encode_component(value)
}

fn percent_encode_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let mut output = String::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16)
        {
            output.push(hex as char);
            index += 3;
            continue;
        }
        output.push(if bytes[index] == b'+' {
            ' '
        } else {
            bytes[index] as char
        });
        index += 1;
    }
    output
}

fn github_status_error(status: u16) -> String {
    match status {
        403 => "GitHub rate limit reached for unauthenticated public requests".to_string(),
        404 => "GitHub repository or path is missing, private, or not public".to_string(),
        _ => format!("GitHub request failed with HTTP {status}"),
    }
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{nanos}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_public_github_tree_url_with_ref_and_path() {
        let parsed = parse_public_github_url(
            "https://github.com/example/agent-assets/tree/main/.codex/skills/research-helper",
        )
        .expect("url should parse");

        assert_eq!(parsed.owner, "example");
        assert_eq!(parsed.repo, "agent-assets");
        assert_eq!(parsed.ref_name.as_deref(), Some("main"));
        assert_eq!(parsed.root_path, ".codex/skills/research-helper");
    }

    #[test]
    fn rejects_non_github_hosts() {
        let error = parse_public_github_url("https://example.com/user/repo")
            .expect_err("non-GitHub host should fail");

        assert!(error.contains("github.com"));
    }

    #[test]
    fn cache_evicts_oldest_scan_session_after_limit() {
        let mut cache = PublicGithubImportCache::new(2, 10, Duration::from_secs(1800));
        cache.insert_scan_session_for_test("a");
        cache.insert_scan_session_for_test("b");
        cache.insert_scan_session_for_test("c");

        assert!(!cache.has_scan_session_for_test("a"));
        assert!(cache.has_scan_session_for_test("b"));
        assert!(cache.has_scan_session_for_test("c"));
    }

    #[test]
    fn scan_session_lookup_does_not_remove_session() {
        let mut cache = PublicGithubImportCache::new(2, 10, Duration::from_secs(1800));
        cache.insert_scan_session_for_test("session");

        let found = cache.get_scan_session("session");

        assert!(found.is_some());
        assert!(cache.has_scan_session_for_test("session"));
    }

    #[test]
    #[ignore = "requires live GitHub network access"]
    fn live_scans_openai_codex_agents_md_from_public_github() {
        let client = ReqwestGithubHttpClient::new().expect("client should initialize");
        let session = fetch_public_github_import_session(
            &client,
            "https://github.com/openai/codex/blob/main/AGENTS.md",
        )
        .expect("public GitHub AGENTS.md should fetch");
        let home = std::env::temp_dir().join(format!(
            "flowmint-live-github-import-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&home);
        flowmint_core::store::init_library_at(&home).expect("library should initialize");

        let candidates =
            scan_remote_import_candidates(&home, session.source.clone(), session.files.clone())
                .expect("remote scan should succeed");

        assert_eq!(session.source.owner, "openai");
        assert_eq!(session.source.repo, "codex");
        assert_eq!(session.source.ref_name, "main");
        assert_eq!(session.source.root_path, "AGENTS.md");
        assert_eq!(session.files.len(), 1);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, "agents");
        assert_eq!(
            candidates[0].asset_type,
            flowmint_core::asset::model::AssetType::InstructionRule
        );

        let plan = preview_remote_import(
            &home,
            session.source,
            session.files,
            vec![RemoteImportSelection {
                candidate_id: candidates[0].candidate_id.clone(),
                destination_id: "openai-codex-agents".to_string(),
                asset_type: flowmint_core::asset::model::AssetType::InstructionRule,
            }],
        )
        .expect("remote import preview should succeed");
        assert!(plan.conflicts.is_empty());
        assert_eq!(plan.items.len(), 1);

        let result = apply_remote_import(&home, &plan).expect("remote import should apply");
        assert_eq!(result.imported_assets, 1);
        assert!(home.join("rules/openai-codex-agents.md").is_file());
        assert!(
            home.join("import-sources/instruction-rules/openai-codex-agents.json")
                .is_file()
        );

        let _ = std::fs::remove_dir_all(home);
    }

    #[test]
    #[ignore = "requires live GitHub network access"]
    fn live_scans_openai_skill_directory_from_public_github() {
        let client = ReqwestGithubHttpClient::new().expect("client should initialize");
        let session = fetch_public_github_import_session(
            &client,
            "https://github.com/openai/skills/tree/main/skills/.system/skill-installer",
        )
        .expect("public GitHub skill directory should fetch");
        let home = std::env::temp_dir().join(format!(
            "flowmint-live-github-skill-import-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&home);
        flowmint_core::store::init_library_at(&home).expect("library should initialize");

        let candidates =
            scan_remote_import_candidates(&home, session.source.clone(), session.files.clone())
                .expect("remote skill scan should succeed");

        assert_eq!(session.source.owner, "openai");
        assert_eq!(session.source.repo, "skills");
        assert_eq!(session.source.ref_name, "main");
        assert_eq!(session.source.root_path, "skills/.system/skill-installer");
        assert!(
            session
                .files
                .iter()
                .any(|file| file.path.ends_with("SKILL.md"))
        );
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, "skill-installer");
        assert_eq!(
            candidates[0].asset_type,
            flowmint_core::asset::model::AssetType::Skill
        );

        let plan = preview_remote_import(
            &home,
            session.source,
            session.files,
            vec![RemoteImportSelection {
                candidate_id: candidates[0].candidate_id.clone(),
                destination_id: "openai-skill-installer".to_string(),
                asset_type: flowmint_core::asset::model::AssetType::Skill,
            }],
        )
        .expect("remote skill import preview should succeed");
        assert!(plan.conflicts.is_empty());
        assert_eq!(plan.items.len(), 1);

        let result = apply_remote_import(&home, &plan).expect("remote skill import should apply");
        assert_eq!(result.imported_assets, 1);
        assert!(
            home.join("skills/openai-skill-installer/SKILL.md")
                .is_file()
        );
        assert!(
            home.join("import-sources/skills/openai-skill-installer.json")
                .is_file()
        );

        let _ = std::fs::remove_dir_all(home);
    }
}

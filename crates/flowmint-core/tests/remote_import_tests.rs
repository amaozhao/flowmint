use std::fs;
use std::path::{Path, PathBuf};

use flowmint_core::asset::model::{
    AssetType, PlaybookAsset, PlaybookInvocation, PlaybookSideEffectLevel, PlaybookStep,
};
use flowmint_core::import::ImportConfidence;
use flowmint_core::import::remote::{
    RemoteFileEntry, RemoteImportProvider, RemoteImportSelection, RemoteImportSource,
    apply_remote_import, preview_remote_import, scan_remote_import_candidates,
};
use flowmint_core::store::init_library_at;

fn test_path(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-remote-import-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn source() -> RemoteImportSource {
    RemoteImportSource {
        provider: RemoteImportProvider::PublicGithub,
        owner: "example".to_string(),
        repo: "agent-assets".to_string(),
        ref_name: "main".to_string(),
        commit_sha: "abc123def456".to_string(),
        root_path: "".to_string(),
        canonical_url: "https://github.com/example/agent-assets/tree/main".to_string(),
    }
}

fn file(path: &str, content: &str) -> RemoteFileEntry {
    RemoteFileEntry {
        path: PathBuf::from(path),
        content: content.to_string(),
        size_bytes: content.len() as u64,
        blob_sha: format!("sha-{path}"),
        source_url: format!("https://github.com/example/agent-assets/blob/main/{path}"),
    }
}

#[test]
fn detects_github_skill_directory_with_supported_file_warnings() {
    let home = test_path("detect-skill");
    init_library_at(&home).expect("library should initialize");
    let files = vec![
        file(
            ".codex/skills/research-helper/SKILL.md",
            "# Research Helper\n",
        ),
        file(
            ".codex/skills/research-helper/metadata.toml",
            "name = \"Research Helper\"\n",
        ),
        file(
            ".codex/skills/research-helper/examples/basic.md",
            "Use it for repo research.",
        ),
        file(
            ".codex/skills/research-helper/scripts/run.sh",
            "echo never-run",
        ),
    ];

    let candidates =
        scan_remote_import_candidates(&home, source(), files).expect("scan should succeed");

    assert_eq!(candidates.len(), 1);
    let candidate = &candidates[0];
    assert_eq!(candidate.id, "research-helper");
    assert_eq!(candidate.asset_type, AssetType::Skill);
    assert_eq!(candidate.confidence, ImportConfidence::High);
    assert!(candidate.importable);
    assert!(
        candidate
            .warnings
            .iter()
            .any(|warning| warning.contains("scripts/run.sh"))
    );

    cleanup(&home);
}

#[test]
fn detects_current_flowmint_playbook_metadata_header() {
    let home = test_path("detect-playbook");
    init_library_at(&home).expect("library should initialize");
    let playbook = PlaybookAsset {
        id: "release-check".to_string(),
        name: "Release Check".to_string(),
        description: Some("Run release checks.".to_string()),
        tags: vec!["release".to_string()],
        trigger: "Before release".to_string(),
        inputs: Vec::new(),
        steps: vec![PlaybookStep {
            title: "Run tests".to_string(),
            body: "Run the release checks.".to_string(),
        }],
        verification: "All checks pass.".to_string(),
        failure_handling: "Stop and report.".to_string(),
        side_effect_level: PlaybookSideEffectLevel::RunsCommands,
        recommended_invocation: PlaybookInvocation::Manual,
        target_compatibility: vec!["codex".to_string()],
    };
    let metadata = serde_json::to_string_pretty(&playbook).expect("metadata should serialize");
    let content = format!(
        "<!-- FLOWMINT:PLAYBOOK:BEGIN\n{metadata}\nFLOWMINT:PLAYBOOK:END -->\n\n# Release Check\n"
    );

    let candidates = scan_remote_import_candidates(
        &home,
        source(),
        vec![file("playbooks/release-check.md", &content)],
    )
    .expect("scan should succeed");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].id, "release-check");
    assert_eq!(candidates[0].asset_type, AssetType::Playbook);
    assert_eq!(candidates[0].confidence, ImportConfidence::High);
    assert!(candidates[0].importable);

    cleanup(&home);
}

#[test]
fn duplicate_destination_ids_block_preview() {
    let home = test_path("duplicate-destination");
    init_library_at(&home).expect("library should initialize");
    let remote_source = source();
    let files = vec![
        file(".claude/commands/review-pr.md", "Review this PR."),
        file(".claude/commands/review-code.md", "Review this code."),
    ];
    let candidates = scan_remote_import_candidates(&home, remote_source.clone(), files.clone())
        .expect("scan should succeed");

    let plan = preview_remote_import(
        &home,
        remote_source,
        files,
        vec![
            RemoteImportSelection {
                candidate_id: candidates[0].candidate_id.clone(),
                destination_id: "review".to_string(),
                asset_type: AssetType::Prompt,
            },
            RemoteImportSelection {
                candidate_id: candidates[1].candidate_id.clone(),
                destination_id: "review".to_string(),
                asset_type: AssetType::Prompt,
            },
        ],
    )
    .expect("preview should return conflicts");

    assert_eq!(plan.items.len(), 0);
    assert!(
        plan.conflicts
            .iter()
            .any(|conflict| conflict.message.contains("duplicate destination id"))
    );
    assert!(!home.join("prompts/review.md").exists());

    cleanup(&home);
}

#[test]
fn remote_import_apply_writes_library_only_and_provenance_sidecar() {
    let home = test_path("apply-skill");
    let project_dir = test_path("apply-skill-project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(&project_dir).expect("project dir should create");
    let remote_source = source();
    let files = vec![
        file(
            ".codex/skills/research-helper/SKILL.md",
            "# Research Helper\n",
        ),
        file(
            ".codex/skills/research-helper/examples/basic.md",
            "Use it for repo research.",
        ),
    ];
    let candidates = scan_remote_import_candidates(&home, remote_source.clone(), files.clone())
        .expect("scan should succeed");
    let plan = preview_remote_import(
        &home,
        remote_source,
        files,
        vec![RemoteImportSelection {
            candidate_id: candidates[0].candidate_id.clone(),
            destination_id: "research-helper".to_string(),
            asset_type: AssetType::Skill,
        }],
    )
    .expect("preview should succeed");

    let result = apply_remote_import(&home, &plan).expect("apply should succeed");

    assert_eq!(result.imported_assets, 1);
    assert!(home.join("skills/research-helper/SKILL.md").is_file());
    assert!(
        home.join("skills/research-helper/examples/basic.md")
            .is_file()
    );
    let provenance = home.join("import-sources/skills/research-helper.json");
    assert!(provenance.is_file());
    let provenance_content =
        fs::read_to_string(&provenance).expect("provenance should be readable");
    assert!(provenance_content.contains("\"commitSha\": \"abc123def456\""));
    assert!(!project_dir.join(".flowmint.toml").exists());
    assert!(!project_dir.join(".flowmint.lock").exists());

    cleanup(&home);
    cleanup(&project_dir);
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

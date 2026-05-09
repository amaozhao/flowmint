use std::fs;
use std::path::PathBuf;

use flowmint_core::asset::model::{PromptAsset, SkillAsset};
use flowmint_core::asset::prompt::create_prompt;
use flowmint_core::asset::skill::create_skill;
use flowmint_core::project::store::add_project;
use flowmint_core::store::diagnostics::{build_debug_report, export_debug_report, rebuild_index};
use flowmint_core::store::init_library_at;

fn test_home(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "flowmint-diagnostics-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn prompt(id: &str) -> PromptAsset {
    PromptAsset {
        id: id.to_string(),
        name: "Daily Plan".to_string(),
        description: None,
        tags: vec!["planning".to_string()],
        variables: Vec::new(),
        body: "# Daily Plan\n".to_string(),
    }
}

fn skill(id: &str, tags: Vec<&str>) -> SkillAsset {
    SkillAsset {
        id: id.to_string(),
        name: "Workflow Skill".to_string(),
        description: None,
        tags: tags.into_iter().map(ToOwned::to_owned).collect(),
        root_dir: PathBuf::new(),
        skill_md: "# Workflow Skill\n".to_string(),
        metadata: None,
        files: Vec::new(),
    }
}

#[test]
fn diagnostics_counts_assets_projects_and_playbook_skills() {
    let home = test_home("counts");
    let project = test_home("project");
    init_library_at(&home).expect("library should initialize");
    fs::create_dir_all(&project).expect("project dir should exist");

    create_prompt(&home, prompt("daily-plan")).expect("prompt should create");
    create_skill(&home, skill("basic-skill", vec!["skill"])).expect("skill should create");
    create_skill(&home, skill("daily-playbook", vec!["playbook"]))
        .expect("playbook skill should create");
    add_project(&home, &project).expect("project should add");

    let summary = rebuild_index(&home).expect("index summary should build");
    assert_eq!(summary.prompt_count, 1);
    assert_eq!(summary.skill_count, 2);
    assert_eq!(summary.playbook_skill_count, 1);
    assert_eq!(summary.project_count, 1);

    let report = build_debug_report(&home).expect("debug report should build");
    assert_eq!(report.index, summary);

    let report_path = export_debug_report(&home).expect("debug report should export");
    assert!(report_path.is_file());
    assert!(report_path.ends_with("cache/debug-report.json"));

    fs::remove_dir_all(home).expect("test home should be removable");
    fs::remove_dir_all(project).expect("project home should be removable");
}

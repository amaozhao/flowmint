use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::store::recent_projects::{load_recent_projects, save_recent_projects};

pub fn list_recent_projects(home: &Path) -> Result<Vec<PathBuf>> {
    load_recent_projects(home)
}

pub fn add_recent_project(home: &Path, project_path: &Path) -> Result<Vec<PathBuf>> {
    let project_path = normalize_project_path(project_path);
    let mut projects = load_recent_projects(home)?;

    projects.retain(|existing| existing != &project_path);
    projects.insert(0, project_path);

    save_recent_projects(home, &projects)?;
    Ok(projects)
}

fn normalize_project_path(project_path: &Path) -> PathBuf {
    std::fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf())
}

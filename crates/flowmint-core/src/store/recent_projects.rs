use std::path::{Path, PathBuf};

use crate::error::{FlowmintError, Result};

const RECENT_PROJECTS_FILE: &str = "recent-projects.toml";

pub fn ensure_recent_projects(home: &Path) -> Result<()> {
    let path = home.join(RECENT_PROJECTS_FILE);
    if path.exists() {
        return Ok(());
    }

    std::fs::write(&path, "projects = []\n").map_err(|source| FlowmintError::io(path, source))
}

pub fn load_recent_projects(home: &Path) -> Result<Vec<PathBuf>> {
    let path = home.join(RECENT_PROJECTS_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content =
        std::fs::read_to_string(&path).map_err(|source| FlowmintError::io(&path, source))?;
    Ok(parse_projects_array(&content)
        .into_iter()
        .map(PathBuf::from)
        .collect())
}

pub fn save_recent_projects(home: &Path, projects: &[PathBuf]) -> Result<()> {
    let path = home.join(RECENT_PROJECTS_FILE);
    let values = projects
        .iter()
        .map(|project| format!("\"{}\"", escape_toml_string(&project.to_string_lossy())))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(&path, format!("projects = [{values}]\n"))
        .map_err(|source| FlowmintError::io(path, source))
}

fn parse_projects_array(content: &str) -> Vec<String> {
    let Some(line) = content
        .lines()
        .find(|line| line.trim_start().starts_with("projects"))
    else {
        return Vec::new();
    };

    let Some((_, values)) = line.split_once('=') else {
        return Vec::new();
    };

    let values = values.trim();
    let Some(values) = values
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };

    values
        .split(',')
        .filter_map(|item| {
            let item = item.trim();
            item.strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .map(|value| value.replace("\\\"", "\"").replace("\\\\", "\\"))
        })
        .collect()
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::parse_projects_array;

    #[test]
    fn parses_recent_project_paths() {
        let projects = parse_projects_array(r#"projects = ["/tmp/a", "/tmp/b"]"#);
        assert_eq!(projects, vec!["/tmp/a", "/tmp/b"]);
    }
}

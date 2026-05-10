use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{FlowmintError, Result};
use crate::store::config::ensure_config;
use crate::store::recent_projects::{ensure_recent_projects, load_recent_projects};

const LIBRARY_DIRS: &[&str] = &[
    "prompts",
    "skills",
    "playbooks",
    "rules",
    "import-sources",
    "templates",
    "cache",
    "backups",
];

const REQUIRED_LIBRARY_DIRS: &[&str] = &[
    "prompts",
    "skills",
    "playbooks",
    "rules",
    "templates",
    "cache",
    "backups",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfo {
    pub path: PathBuf,
    pub initialized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub version: String,
    pub library: LibraryInfo,
    pub recent_projects: Vec<PathBuf>,
}

pub fn default_home_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("FLOWMINT_HOME") {
        return Ok(PathBuf::from(path));
    }

    if let Some(path) = read_selected_home_dir()? {
        return Ok(path);
    }

    canonical_default_home_dir()
}

pub fn global_user_home_dir(library_home: &Path) -> Result<PathBuf> {
    if library_home.file_name().and_then(|value| value.to_str()) == Some(".flowmint")
        && let Some(parent) = library_home.parent()
    {
        return Ok(parent.to_path_buf());
    }

    user_home_dir()
}

fn user_home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or(FlowmintError::HomeDirectoryUnavailable)
}

fn canonical_default_home_dir() -> Result<PathBuf> {
    Ok(user_home_dir()?.join(".flowmint"))
}

fn home_selection_path() -> Result<PathBuf> {
    Ok(user_home_dir()?.join(".flowmint-home"))
}

fn read_selected_home_dir() -> Result<Option<PathBuf>> {
    let path = home_selection_path()?;
    if !path.is_file() {
        return Ok(None);
    }

    let value =
        std::fs::read_to_string(&path).map_err(|source| FlowmintError::io(&path, source))?;
    let value = value.trim();
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(PathBuf::from(value)))
    }
}

fn persist_selected_home_dir(home: &Path) -> Result<()> {
    let path = home_selection_path()?;
    std::fs::write(&path, home.to_string_lossy().as_bytes())
        .map_err(|source| FlowmintError::io(path, source))
}

pub fn get_app_state() -> Result<AppState> {
    let home = default_home_dir()?;
    get_app_state_for_home(&home)
}

pub fn get_app_state_for_home(home: &Path) -> Result<AppState> {
    let library = LibraryInfo {
        path: home.to_path_buf(),
        initialized: is_initialized(home),
    };

    let recent_projects = if library.initialized {
        load_recent_projects(home)?
    } else {
        Vec::new()
    };

    Ok(AppState {
        version: crate::version().to_string(),
        library,
        recent_projects,
    })
}

pub fn init_library(path: Option<PathBuf>) -> Result<LibraryInfo> {
    let selected_home = path;
    let home = match selected_home.clone() {
        Some(path) => path,
        None => default_home_dir()?,
    };

    let library = init_library_at(&home)?;
    if selected_home.is_some() {
        persist_selected_home_dir(&home)?;
    }
    Ok(library)
}

pub fn init_library_at(home: &Path) -> Result<LibraryInfo> {
    std::fs::create_dir_all(home).map_err(|source| FlowmintError::io(home, source))?;

    for dir in LIBRARY_DIRS {
        let path = home.join(dir);
        std::fs::create_dir_all(&path).map_err(|source| FlowmintError::io(path, source))?;
    }

    ensure_config(home)?;
    ensure_recent_projects(home)?;

    Ok(LibraryInfo {
        path: home.to_path_buf(),
        initialized: true,
    })
}

fn is_initialized(home: &Path) -> bool {
    home.join("config.toml").is_file()
        && home.join("recent-projects.toml").is_file()
        && REQUIRED_LIBRARY_DIRS
            .iter()
            .all(|dir| home.join(dir).is_dir())
}

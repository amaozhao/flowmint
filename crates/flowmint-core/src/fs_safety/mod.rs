pub mod write;

use std::path::Path;

pub fn path_is_inside(root: &Path, path: &Path) -> bool {
    path.starts_with(root)
}

pub fn nearest_existing_parent(path: &Path) -> Option<&Path> {
    let mut current = path.parent();
    while let Some(path) = current {
        if path.exists() {
            return Some(path);
        }
        current = path.parent();
    }
    None
}

pub fn parent_is_writable(path: &Path) -> bool {
    nearest_existing_parent(path)
        .and_then(|parent| std::fs::metadata(parent).ok())
        .map(|metadata| !metadata.permissions().readonly())
        .unwrap_or(true)
}

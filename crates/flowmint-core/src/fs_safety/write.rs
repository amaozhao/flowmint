use std::path::Path;

use crate::error::{FlowmintError, Result};

pub fn write_file_atomic(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| FlowmintError::io(parent, source))?;
    }

    let temp_path = path.with_extension(format!(
        "{}.flowmint-tmp",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("tmp")
    ));
    std::fs::write(&temp_path, content).map_err(|source| FlowmintError::io(&temp_path, source))?;
    std::fs::rename(&temp_path, path).map_err(|source| FlowmintError::io(path, source))
}

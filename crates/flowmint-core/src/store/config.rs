use std::path::Path;

use crate::error::{FlowmintError, Result};

const CONFIG_FILE: &str = "config.toml";

pub fn ensure_config(home: &Path) -> Result<()> {
    let path = home.join(CONFIG_FILE);
    if path.exists() {
        return Ok(());
    }

    let content = format!(
        "version = \"{}\"\nlibrary_path = \"{}\"\n",
        crate::version(),
        escape_toml_string(&home.display().to_string())
    );

    std::fs::write(&path, content).map_err(|source| FlowmintError::io(path, source))
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

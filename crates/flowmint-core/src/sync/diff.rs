use std::path::Path;

use crate::error::{FlowmintError, Result};

pub fn content_hash(content: &[u8]) -> String {
    format!("fnv1a64:{:016x}", fnv1a64(content))
}

pub fn file_hash(path: &Path) -> Result<String> {
    let content = std::fs::read(path).map_err(|source| FlowmintError::io(path, source))?;
    Ok(content_hash(&content))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

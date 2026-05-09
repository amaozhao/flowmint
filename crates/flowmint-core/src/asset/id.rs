use std::path::Path;

use crate::asset::model::AssetType;
use crate::validation::{ValidationReport, ValidationStatus};

pub fn is_safe_asset_id(id: &str) -> bool {
    !id.is_empty()
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
}

pub fn validate_new_asset_id(
    library_home: &Path,
    asset_type: AssetType,
    id: &str,
) -> ValidationReport {
    let mut report = ValidationReport::valid();

    if !is_safe_asset_id(id) {
        report.push_error("id must use only a-z, 0-9, hyphen, or underscore");
        return report;
    }

    let path = match asset_type {
        AssetType::Prompt => library_home.join("prompts").join(format!("{id}.md")),
        AssetType::Skill => library_home.join("skills").join(id),
        AssetType::Playbook => library_home.join("playbooks").join(format!("{id}.md")),
        AssetType::InstructionRule | AssetType::CommandRule => {
            library_home.join("rules").join(format!("{id}.md"))
        }
    };

    if path.exists() {
        report.push_error(format!("asset id '{id}' already exists"));
    }

    if report.messages.is_empty() {
        report.status = ValidationStatus::Valid;
    }

    report
}

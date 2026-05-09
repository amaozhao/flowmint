use flowmint_core::store::template_store::{
    SkillTemplate, SkillTemplateKind, get_skill_template as core_get_skill_template,
    list_skill_templates as core_list_skill_templates,
};

#[tauri::command]
pub fn list_skill_templates() -> Vec<SkillTemplate> {
    core_list_skill_templates()
}

#[tauri::command]
pub fn get_skill_template(kind: SkillTemplateKind) -> SkillTemplate {
    core_get_skill_template(kind)
}

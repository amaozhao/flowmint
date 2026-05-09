use flowmint_core::store::template_store::{
    SkillTemplateKind, get_skill_template, list_skill_templates,
};

#[test]
fn skill_templates_include_basic_and_playbook_shapes() {
    let templates = list_skill_templates();

    assert_eq!(templates.len(), 2);
    assert_eq!(templates[0].kind, SkillTemplateKind::Basic);
    assert_eq!(templates[1].kind, SkillTemplateKind::Playbook);
    assert!(templates[0].skill_md.contains("## Instructions"));
    assert!(templates[1].skill_md.contains("## Steps"));
    assert!(templates[1].tags.iter().any(|tag| tag == "playbook"));
}

#[test]
fn playbook_template_is_a_skill_template_not_an_asset_type() {
    let template = get_skill_template(SkillTemplateKind::Playbook);

    assert_eq!(template.kind, SkillTemplateKind::Playbook);
    assert_eq!(template.name, "Playbook Skill");
    assert!(template.description.contains("repeatable workflow"));
    assert!(template.skill_md.starts_with("# Playbook Skill"));
}

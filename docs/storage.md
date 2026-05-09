# Flowmint Storage Notes

Flowmint uses the local file system as the source of truth.

## Library Home

Default library:

```text
~/.flowmint/
  config.toml
  recent-projects.toml
  global-sync-profiles.toml
  global-sync.lock
  prompts/
  skills/
  playbooks/
  rules/
  templates/
  cache/
  backups/
```

## Project Files

Each managed project can contain:

```text
<project>/
  .flowmint.toml
  .flowmint.lock
```

`.flowmint.toml` supports both the v0.1 Prompt/Skill shape and v2
`[[exports]]` profiles for target/scope-specific Prompt, Skill, Playbook,
Instruction Rule, and Command Rule attachments.

`.flowmint.lock` and `global-sync.lock` record generated outputs by target and
scope. Lockfile merge behavior preserves records for other targets/scopes.

## Safety Invariants

- UI pages do not write workflow files directly.
- Sync and import adoption apply only backend-cached plans.
- Global sync apply requires exact path acknowledgement.
- Generated file conflicts are detected by target path and content hash.

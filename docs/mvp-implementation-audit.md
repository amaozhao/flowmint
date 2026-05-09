# Flowmint Implementation Audit

Source documents:

- `docs/mvp.md`
- `docs/multi-agent-sync-feature-spec.md`
- `docs/multi-agent-sync-task-plan.md`

This audit reflects the current local-first desktop implementation.

## Core Surface

| Requirement | Status | Evidence |
|---|---:|---|
| Local asset library | Implemented | Rust Core initializes `prompts/`, `skills/`, `playbooks/`, `rules/`, `templates/`, `cache/`, and `backups/`. |
| Desktop UI management | Implemented | Tauri + React pages exist for Overview, Assets, Projects, Sync, Import, and Settings; Overview includes local asset/project/target support charts. |
| Project binding | Implemented | Projects can attach/detach assets per target profile for project scope. |
| Global user profiles | Implemented | Settings can attach/detach assets per global target profile. |
| Sync preview/apply | Implemented | Preview shows operations/conflicts before apply; apply uses backend-cached plans. Global apply also shows root and mutating paths before confirmation. |
| Import/adoption | Implemented | Import scan is read-only; Copy/Adopt/Skip use preview/apply plans. |

## Asset Types

| Asset type | Status | Notes |
|---|---:|---|
| Prompt | Implemented | Create/edit/list/search/export to Claude Code and Gemini CLI where supported. |
| Skill | Implemented | Directory-backed Skill with `SKILL.md`, editable `metadata.toml`, editable recursive `examples/`, and editable recursive `resources/` support. |
| Playbook | Implemented | First-class editor/store/export-as-Skill plus promotion from legacy Playbook Skill. |
| Instruction Rule | Implemented | Editor/store/export to Claude Code, Codex `AGENTS.md`, and Gemini `GEMINI.md`. |
| Command Rule | Implemented | Editor/store/export to Codex `.codex/rules/*.rules`; other targets are blocked as unsupported. |

## Target Support

| Target | Status |
|---|---:|
| Claude Code | Prompt, Skill, Playbook, Instruction Rule supported; Command Rule blocked. |
| Codex | Skill, Playbook, Instruction Rule, Command Rule supported; Prompt blocked pending explicit conversion. |
| Gemini CLI | Prompt and Instruction Rule supported; Skill, Playbook, and Command Rule blocked pending validation/support. |
| Cursor | Not implemented. |

## Safety

| Safety requirement | Status |
|---|---:|
| No silent overwrite of user files | Implemented |
| Path traversal/symlink conflict detection | Implemented |
| Backend-owned apply plans | Implemented |
| Global write second confirmation | Implemented |
| No Skill script execution | Implemented |
| No cloud upload / no account / no LLM API call | Implemented |

## Remaining Verification

1. Run the full manual desktop smoke path in `docs/manual-smoke-test.md`.
2. Run the GitHub release workflow and smoke-test macOS/Windows packages on those operating systems before calling them supported.
3. Keep advanced conflict actions out until backend-owned semantics are designed and tested.

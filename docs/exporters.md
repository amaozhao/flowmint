# Flowmint Exporter Notes

Flowmint exports from the local library through Rust Core only. The desktop UI
always requests a `SyncPlan` first, and `apply_sync` writes only a cached,
revalidated plan.

## Target Matrix

| Target | Project Scope | Global User Scope | Supported assets |
| --- | --- | --- | --- |
| Claude Code | `<project>/.claude/*`, `CLAUDE.md` | `~/.claude/*`, `~/.claude/CLAUDE.md` | Prompt, Skill, Playbook-as-Skill, Instruction Rule |
| Codex | `<project>/.codex/skills`, `AGENTS.md`, `.codex/rules` | `~/.codex/skills`, `~/.codex/AGENTS.md`, `~/.codex/rules` | Skill, Playbook-as-Skill, Instruction Rule, Command Rule |
| Gemini CLI | `<project>/.gemini/commands`, `GEMINI.md` | `~/.gemini/commands`, `~/.gemini/GEMINI.md` | Prompt, Instruction Rule |

Unsupported mappings are blocking preview conflicts:

- Claude Code Command Rules are deferred.
- Codex Prompt commands are deferred unless an explicit Prompt-as-Skill
  conversion is added.
- Gemini Skill, Playbook, and Command Rule export remains blocked until local
  target support is validated.

## Scope Safety

- Project scope writes only inside the selected project root.
- Global user scope writes under the inferred user tool configuration root and
  requires a second explicit acknowledgement before apply.
- Global apply is rejected if the acknowledged paths do not exactly match the
  latest cached plan.
- The frontend cannot submit arbitrary operations to `apply_sync`.

## Import And Adoption

Import scans are read-only. Users choose Project or Global User scope before
scan, can paste or browse for a project path when Project scope is selected,
and then choose per candidate:

- `Copy`: create a Flowmint library asset and leave the source unmanaged.
- `Adopt`: mark the existing target file as Flowmint-managed after apply.
- `Skip`: ignore the candidate.

Adoption writes lock records only after apply and rejects stale plans if the
source file changes between preview and apply.

The scanner covers Claude Code commands, skills, rule markdown, and
`CLAUDE.md` instruction files; Codex `.codex/skills`, legacy `.agents/skills`,
`AGENTS.md`, and `.codex/rules`; and Gemini CLI command files.

# Multi-Agent Sync Spec And Plan Review

Reviewed documents:

- `docs/multi-agent-sync-feature-spec.md`
- `docs/multi-agent-sync-task-plan.md`

Review date: 2026-05-08.

## Verdict

No blocking issues remain in the feature specification or task plan.

One planning gap was found during review and fixed before this review was
written: global user sync profiles cannot live only inside per-project
`.flowmint.toml` manifests. The spec now defines
`~/.flowmint/global-sync-profiles.toml`, and the task plan now includes FM-404
for the global profile store.

## Coverage Review

| Requirement | Covered In Spec | Covered In Plan | Review Result |
| --- | --- | --- | --- |
| User chooses global vs project scope | Sync Scope, UI Requirements | FM-401, FM-404, FM-440, FM-450 | Covered |
| Global writes require confirmation | Safety Requirements | FM-440, FM-450, FM-452 | Covered |
| Prompt management | Asset Types, Target Capability Matrix | FM-403, FM-421, FM-422, FM-423 | Covered |
| Skill management | Asset Types, Target Capability Matrix | FM-410, FM-421, FM-422, FM-423, FM-441 | Covered |
| Playbook management | Playbook section | FM-411, FM-421, FM-422, FM-423, FM-441 | Covered |
| Rule management | Rule section | FM-410, FM-421, FM-422, FM-423, FM-441 | Covered |
| Instruction Rules vs Command Rules | Rule section | FM-410, FM-422, FM-451 | Covered |
| Claude Code exporter | Target Capability Matrix | FM-421, FM-452 | Covered |
| Codex exporter | Target Capability Matrix | FM-422, FM-452 | Covered |
| Gemini CLI exporter | Target Capability Matrix | FM-423, FM-452 | Covered |
| Import existing tool files | Import Requirements | FM-430, FM-431, FM-442 | Covered |
| Preview/apply safety | Sync Preview, Lockfile, Safety | FM-420, FM-431, FM-450 | Covered |
| v0.1 backward compatibility | Current Baseline, Manifest v2 | FM-402, FM-420, FM-421, FM-452 | Covered |
| Unsupported mappings are blocked | Product Goals, Sync Preview | FM-403, FM-422, FM-423, FM-452 | Covered |

## Tool-Specific Accuracy Review

### Claude Code

The plan uses Claude Code native concepts correctly:

- Prompt commands map to `.claude/commands/` or `~/.claude/commands/`.
- Skills and Playbooks map to `.claude/skills/` or `~/.claude/skills/`.
- Instruction Rules map to `.claude/rules/` or `~/.claude/rules/`.
- `CLAUDE.md` remains a managed instruction summary, not the only storage
  mechanism.

No unsupported Claude Code behavior is required by the plan.

### Codex

The plan avoids the common Codex mistake:

- Codex Instruction Rules are rendered into `AGENTS.md`, not `.codex/rules/`.
- `.codex/rules/` is reserved for Codex Command Rules that control command
  execution outside the sandbox.
- Codex Skills use `.codex/skills/` or `~/.codex/skills/`; import scanning also
  recognizes legacy `.agents/skills/` so existing user assets can be adopted.
- Codex Prompt command export is explicitly unsupported unless the user opts
  into Prompt-as-Skill conversion.

No unsupported Codex prompt command path is listed as supported.

### Gemini CLI

The plan is intentionally conservative:

- Prompt commands map to `.gemini/commands/*.toml` or
  `~/.gemini/commands/*.toml`.
- Instruction Rules map to `GEMINI.md` managed content or imports.
- Skill and Playbook export to `.gemini/skills/` is gated behind local CLI
  validation before enabling, because this path should be verified against the
  installed Gemini CLI.
- Command Rules are not exported to Gemini in the first multi-target release.

No Gemini Command Rule behavior is claimed as supported.

## Safety Review

The plan keeps safety boundaries intact:

- Apply remains backend-owned.
- Plans include target and scope in identity.
- Global writes cannot reuse project confirmations.
- Lockfiles track target, scope, asset type, asset ID, path, and hashes.
- Imports are read-only until adoption apply.
- Symlinks and path escapes remain conflicts.
- Script files may be copied but are never executed by Flowmint.

## Remaining Implementation Risks

These are not spec gaps, but they need attention during implementation:

- Gemini Skill discovery must be validated against the locally installed CLI
  before enabling Skill/Playbook export.
- Codex Prompt-as-Skill conversion needs explicit UI wording so users know it is
  not a native slash command.
- Global profile UI must make it obvious that global assets affect future
  projects, not just the project currently open in Flowmint.
- Manifest v2 migration should preserve v0.1 formatting unless a v2-only
  feature is actually used.

## Final Review Checklist

- No `TODO` or `TBD` placeholders remain in the new planning docs.
- Codex Skill export and scan paths use `.codex/skills/`.
- Codex `.rules` are only used for Command Rules.
- Global sync storage is separate from project manifests.
- Playbook migration from tagged Skill is explicit.
- Both positive and blocked smoke scenarios are planned.

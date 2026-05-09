# Multi-Agent Asset Sync Feature Specification

Status: implemented local-first multi-target scope.

Last reviewed: 2026-05-08.

## Purpose

Flowmint should become the local source of truth for reusable AI coding assets and
sync them safely into the coding agents a developer actually uses: Claude Code,
Codex, and Gemini CLI.

The current v0.1 implementation only exports Prompt and Skill assets to Claude
Code at project scope. The next phase adds multi-target sync, explicit global vs
project scope selection, first-class Rule management, and richer Playbook
management without turning Flowmint into an agent runtime.

## Product Goals

1. Let users manage Prompt, Skill, Playbook, and Rule assets in one local
   library.
2. Let users choose where an asset is synced: project scope or global user
   scope.
3. Let users sync the same canonical asset into Claude Code, Codex, and Gemini
   CLI using each tool's native file layout.
4. Keep preview-before-apply, lockfile hashing, and conflict detection as the
   core safety model.
5. Make imported files visible and reviewable before Flowmint adopts or
   overwrites anything.
6. Avoid claiming portability where a target tool does not support an equivalent
   feature.

## Non-Goals

- No cloud sync or account system.
- No marketplace.
- No LLM API calls.
- No agent runtime, workflow execution engine, or background automation.
- No direct editing of enterprise/admin-managed configuration paths.
- No silent writes into global tool directories.
- No unmanaged overwrite of user-authored files.

## Current Implementation Baseline

Implemented today:

- Prompt library: `~/.flowmint/prompts/<prompt-id>.md`.
- Skill library: `~/.flowmint/skills/<skill-id>/`.
- Playbook library: `~/.flowmint/playbooks/<playbook-id>.md`.
- Rule library: `~/.flowmint/rules/<rule-id>.md`.
- Project manifest: `<project>/.flowmint.toml`.
- Project and global scope sync preview/apply.
- Claude Code export:
  - Prompt -> `<project>/.claude/commands/<prompt-id>.md`.
  - Skill -> `<project>/.claude/skills/<skill-id>/`.
  - Playbook -> `<project>/.claude/skills/<playbook-id>/`.
  - Instruction Rule -> `<project>/.claude/rules/<rule-id>.md`.
  - Managed summary block -> `<project>/CLAUDE.md`.
  - Lockfile -> `<project>/.flowmint.lock`.
- Codex export:
  - Skill and Playbook -> `.codex/skills/<id>/`.
  - Instruction Rule -> `AGENTS.md` managed block.
  - Command Rule -> `.codex/rules/<rule-id>.rules`.
- Gemini CLI export:
  - Prompt -> `.gemini/commands/<prompt-id>.toml`.
  - Instruction Rule -> `GEMINI.md` managed block.
- Import scanner and adoption UI for project/global target files.

Known limitations:

- Codex Prompt command export is blocked until explicit Prompt-as-Skill
  conversion exists.
- Gemini Skill/Playbook export is blocked until local Gemini CLI support is
  validated.
- Claude Code Command Rule export and Gemini Command Rule export are deferred.
- Existing Playbook Skills remain valid and can be promoted to first-class
  Playbooks from the desktop UI.

## Core Concepts

### Asset Types

#### Prompt

A reusable prompt body meant to be invoked directly by the user.

Canonical Flowmint data:

- `id`
- `name`
- `description`
- `tags`
- `variables`
- `body`
- `targetCompatibility`

Target rendering:

- Claude Code: custom command markdown.
- Gemini CLI: custom command TOML.
- Codex: no documented custom prompt-file equivalent in the checked official
  docs; render as a Skill when the user opts into "Prompt as Skill", or include
  in generated instruction material. Do not pretend Codex has a Claude-style
  command directory.

#### Skill

A reusable capability package with `SKILL.md` and optional supporting files.

Canonical Flowmint data:

- `id`
- `name`
- `description`
- `tags`
- `skillMd`
- optional `metadata.toml`
- optional support folders
- `targetCompatibility`
- invocation policy hints, stored as portable metadata first and rendered only
  where the target supports them.

Supported content folders should expand beyond v0.1:

- `examples/`
- `resources/`
- `references/`
- `assets/`
- `scripts/`

Scripts are copied as files but never executed by Flowmint.

#### Playbook

A repeatable multi-step workflow. In the next phase, Playbook becomes a
first-class Flowmint asset while still exporting to target-native Skill formats.

Canonical Flowmint data:

- `id`
- `name`
- `description`
- `tags`
- `trigger`
- `inputs`
- `steps`
- `verification`
- `failureHandling`
- `sideEffectLevel`: `none`, `read-only`, `writes-files`, `runs-commands`,
  `external-side-effects`
- `recommendedInvocation`: `manual`, `model`, or `both`
- `targetCompatibility`

Target rendering:

- Claude Code: `.claude/skills/<playbook-id>/SKILL.md`.
- Codex: `.codex/skills/<playbook-id>/SKILL.md`.
- Gemini CLI: `.gemini/skills/<playbook-id>/SKILL.md` after local validation of
  Gemini skill discovery, or an extension/linked skill path if the installed CLI
  requires that path.

Migration:

- Existing v0.1 Skills tagged `playbook` remain valid.
- Users can promote a tagged Skill into a first-class Playbook.
- Promotion creates a new Playbook record and does not delete the original Skill
  unless the user explicitly chooses to replace it.

#### Rule

A persistent instruction or policy that should shape agent behavior.

Flowmint must split Rule into two subtypes because target tools use "rules" for
different things.

##### Instruction Rule

Context given to an agent, such as code style, architecture conventions, test
requirements, and repo-specific operating agreements.

Canonical fields:

- `id`
- `name`
- `description`
- `tags`
- `body`
- optional `paths` globs
- `scope`
- `targetCompatibility`

Target rendering:

- Claude Code:
  - Project: `.claude/rules/<rule-id>.md`.
  - Global: `~/.claude/rules/<rule-id>.md`.
  - Path-specific rules can use `paths` frontmatter.
- Codex:
  - Project: managed block in `AGENTS.md`.
  - Global: managed block in `~/.codex/AGENTS.md`.
  - Codex `rules/` files are not instruction rules; they control command
    execution outside the sandbox. Path-specific instruction rules should be
    marked "limited" for Codex unless Flowmint writes nested AGENTS files.
- Gemini CLI:
  - Project: managed block in `GEMINI.md`, or an imported file referenced from
    `GEMINI.md`.
  - Global: managed block in `~/.gemini/GEMINI.md`.
  - Path-specific rules are supported only if rendered through a target pattern
    Gemini actually loads; otherwise mark as limited.

##### Command Rule

Permission policy for command execution, such as "allow `git status`",
"prompt before `gh pr view`", or "forbid destructive commands".

Initial support:

- Codex only, because official Codex docs define `.rules` files under
  `rules/` next to active config layers.

Deferred support:

- Claude permissions/settings rule export.
- Gemini policy/hook export.

Command Rules are high-risk and must require explicit confirmation even at
project scope.

## Sync Scope

Every export action must require a scope selection.

### Project Scope

Writes files inside the selected project.

Use for team-shared assets committed with the repository.

Examples:

- `<project>/.claude/skills/<id>/`
- `<project>/.codex/skills/<id>/`
- `<project>/.gemini/commands/<id>.toml`
- `<project>/AGENTS.md`
- `<project>/GEMINI.md`

Default scope: Project.

### Global User Scope

Writes files into the current user's home tool configuration.

Use for personal assets available across all projects.

Examples:

- `~/.claude/skills/<id>/`
- `~/.claude/commands/<id>.md`
- `~/.codex/AGENTS.md`
- `~/.codex/skills/<id>/`
- `~/.gemini/GEMINI.md`
- `~/.gemini/commands/<id>.toml`

Global writes must show a second confirmation screen with exact paths and a
warning that all future projects may be affected.

### Local Private Scope

Optional later scope for project-specific personal settings that should not be
committed.

Examples:

- `.claude/settings.local.json`
- future tool-specific local-only files.

This is not required for the first multi-target release.

## Target Capability Matrix

| Capability | Claude Code | Codex | Gemini CLI |
| --- | --- | --- | --- |
| Project instructions | `CLAUDE.md` or `.claude/CLAUDE.md` | `AGENTS.md` | `GEMINI.md` |
| Global instructions | `~/.claude/CLAUDE.md` | `~/.codex/AGENTS.md` | `~/.gemini/GEMINI.md` |
| Project skills | `.claude/skills/<id>/` | `.codex/skills/<id>/` | `.gemini/skills/<id>/` after validation |
| Global skills | `~/.claude/skills/<id>/` | `~/.codex/skills/<id>/` | `~/.gemini/skills/<id>/` after validation |
| Project prompt commands | `.claude/commands/<id>.md` | no equivalent confirmed | `.gemini/commands/<id>.toml` |
| Global prompt commands | `~/.claude/commands/<id>.md` | no equivalent confirmed | `~/.gemini/commands/<id>.toml` |
| Instruction rules | `.claude/rules/<id>.md` | `AGENTS.md` managed block | `GEMINI.md` managed block/import |
| Command permission rules | settings/permissions later | `.codex/rules/<id>.rules` | policy/hook later |

## Project Manifest v2

The v2 manifest must support multiple targets and scopes while preserving v0.1
files.

Proposed shape:

```toml
[project]
name = "my-project"

[[exports]]
target = "claude-code"
scope = "project"
prompts = ["review-pr"]
skills = ["api-helper"]
playbooks = ["release-check"]
instruction_rules = ["typescript-style"]
command_rules = []

[[exports]]
target = "codex"
scope = "project"
prompts = []
skills = ["api-helper"]
playbooks = ["release-check"]
instruction_rules = ["typescript-style"]
command_rules = ["safe-git-status"]

[[exports]]
target = "gemini-cli"
scope = "project"
prompts = ["review-pr"]
skills = ["api-helper"]
playbooks = ["release-check"]
instruction_rules = ["typescript-style"]
command_rules = []
```

Backward compatibility:

- Existing v0.1 manifests with `[export] target = "claude-code"` and `[attach]`
  must continue to load.
- On save, Flowmint may preserve v0.1 format until the user adds another target,
  scope, Rule, or Playbook attachment.
- Once upgraded to v2, Flowmint writes `[[exports]]`.

## Global Sync Profiles

Global user scope cannot belong only to one project manifest. Flowmint also
needs library-level sync profiles for assets the user wants everywhere.

Proposed file:

```text
~/.flowmint/global-sync-profiles.toml
```

Proposed shape:

```toml
[[profiles]]
target = "claude-code"
scope = "global-user"
prompts = ["review-pr"]
skills = ["api-helper"]
playbooks = ["release-check"]
instruction_rules = ["personal-style"]
command_rules = []

[[profiles]]
target = "codex"
scope = "global-user"
prompts = []
skills = ["api-helper"]
playbooks = []
instruction_rules = ["personal-style"]
command_rules = ["safe-gh-pr-view"]
```

Rules:

- Project manifests store project-associated sync profiles.
- Global sync profiles store all-project personal profiles.
- The UI may let a user create a global profile while viewing a project, but the
  resulting profile is saved in the library-level global profile store.
- Global sync still uses `~/.flowmint/global-sync.lock` for ownership records.

## Sync Preview Requirements

Preview must group planned writes by:

- target tool
- scope
- asset type
- asset id
- output path
- operation: create, update, noop, delete stale
- risk level

Preview must show:

- exact destination paths
- whether a file is global or project-local
- conflicts
- unsupported target mappings
- lockfile ownership status
- whether an asset is being transformed, such as Prompt -> Codex Skill

Apply must be blocked when:

- any selected output has a conflict
- a target mapping is unsupported
- a global write was not explicitly confirmed
- the cached plan no longer matches the regenerated plan

## Lockfile Requirements

The lockfile must track target and scope, not just path.

Required record fields:

- `target`
- `scope`
- `asset_type`
- `asset_id`
- `output_path`
- `content_hash`
- `source_hash`
- `generated_by`
- `updated_at`

Project-scope lockfiles:

- `<project>/.flowmint.lock`

Global-scope lockfile:

- `~/.flowmint/global-sync.lock`

Global lockfile records must never include secrets.

## Import Requirements

Flowmint must support importing existing tool files into the library.

Import flow:

1. User chooses target tool.
2. User chooses scope: project or global.
3. Flowmint scans known paths.
4. Flowmint presents detected assets.
5. User chooses import mode per item:
   - `Copy into library`: create a Flowmint asset, leave original unmanaged.
   - `Adopt into Flowmint`: create a Flowmint asset and mark target path as
     managed after preview/apply.
   - `Skip`.
6. Flowmint detects collisions and asks the user to rename or skip.

Import should be read-only until the user applies an adoption plan.

## UI Requirements

### Assets

Add Rule and Playbook as visible asset types.

Required filters:

- Prompt
- Skill
- Playbook
- Instruction Rule
- Command Rule
- target compatibility
- tags

### Asset Editors

Prompt editor:

- show target compatibility
- show target-specific render warnings

Skill editor:

- support supporting folders beyond v0.1
- show whether scripts are present
- warn that Flowmint copies scripts but does not execute them

Playbook editor:

- structured fields for trigger, steps, verification, failure handling, and
  side-effect level
- render preview as target-specific Skill content

Rule editor:

- choose Rule subtype
- choose path globs for Instruction Rules
- choose command prefix and decision for Codex Command Rules
- preview rendered target files

### Projects

Project detail must show an export matrix:

- rows: assets
- columns: target + scope profiles
- cells: attached, unsupported, conflict, pending

### Sync

Sync preview must support:

- preview selected target/scope
- preview all enabled target/scope profiles
- apply selected safe plans
- show global writes separately

### Settings

Settings must include:

- enabled target tools
- detected target paths
- default scope
- global write confirmation policy
- target compatibility diagnostics
- import scanner entry point

## Safety Requirements

1. Project scope remains the default.
2. Global scope requires explicit selection and second confirmation.
3. Flowmint never writes into admin/managed configuration paths.
4. Flowmint never follows unsafe symlinks.
5. Flowmint rejects asset IDs that can escape target directories.
6. Flowmint never overwrites unmanaged files.
7. Flowmint detects user edits after last sync and blocks apply.
8. Flowmint does not execute generated scripts, skill scripts, shell snippets, or
   hook files.
9. Command Rules default to prompt or forbidden; never default to broad allow.
10. Unsupported target mappings are visible and blocked, not silently skipped.

## Documentation Sources

- Claude Code skills: https://code.claude.com/docs/en/skills
- Claude Code memory and rules: https://code.claude.com/docs/en/memory
- Claude Code settings scopes: https://code.claude.com/docs/en/settings
- Codex AGENTS.md: https://developers.openai.com/codex/guides/agents-md
- Codex skills: https://developers.openai.com/codex/skills
- Codex rules: https://developers.openai.com/codex/rules
- Codex config basics: https://developers.openai.com/codex/config-basic
- Gemini CLI GEMINI.md: https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/gemini-md.md
- Gemini CLI custom commands: https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/custom-commands.md
- Gemini CLI configuration: https://github.com/google-gemini/gemini-cli/blob/main/docs/reference/configuration.md
- Gemini CLI reference: https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/cli-reference.md

## Review Checklist

- Scope selection is explicit for every sync target.
- Global writes require second confirmation.
- Rules are split into Instruction Rules and Command Rules.
- Codex command rules are not confused with normal instruction rules.
- Codex prompts are not exported to an unsupported custom command path.
- Playbooks become first-class assets but export through target-native Skill
  layouts.
- Existing v0.1 Claude Code export remains backward compatible.
- Preview/apply safety remains backend-owned.
- Import is read-only until adoption apply.
- Unsupported mappings are visible and blocking.

# Multi-Agent Asset Sync Task Plan

Status: proposed implementation plan.

Source spec: `docs/multi-agent-sync-feature-spec.md`.

Goal: add scoped multi-target sync for Prompt, Skill, Playbook, and Rule assets
across Claude Code, Codex, and Gemini CLI while preserving the v0.1 Claude Code
behavior and safety model.

## Current Progress

Last updated: 2026-05-08.

| Task | Status | Notes |
| --- | --- | --- |
| FM-401 | Implemented | `SyncScope` is part of `SyncPlan`, plan IDs include scope, and Tauri preview/apply supports Project and Global User scopes through target adapters. |
| FM-402 | Implemented | Project manifests parse v0.1 and v2 `[[exports]]`; v0.1 rendering is preserved until v2-only fields are used. |
| FM-403 | Implemented | Target capability registry covers Claude Code, Codex, and Gemini CLI, including unsupported Codex prompt commands and Gemini skill validation requirements. |
| FM-404 | Implemented | Global sync profiles read/write `~/.flowmint/global-sync-profiles.toml` and reject non-global scopes. |
| FM-410 | Implemented | Rule asset backend and desktop editor support Instruction Rule and Command Rule creation, listing, validation, routing, deletion, and target-aware export behavior. |
| FM-411 | Implemented | First-class Playbook backend and desktop editor support create/list/get/update/delete, validation, SKILL.md rendering, export as Skill, and promotion from legacy Playbook Skill without deleting the Skill. |
| FM-420 | Implemented | Exporter router routes Claude Code project preview/apply through a shared target entry point and returns structured errors for unknown targets or unsupported scopes. |
| FM-421 | Implemented | Claude Code exporter supports project/global Prompt, Skill, Playbook-as-Skill, and Instruction Rule exports, writes project/global lockfiles, and blocks Command Rules as unsupported mappings. UI confirmation hardening for global writes remains in FM-450. |
| FM-422 | Implemented | Codex exporter supports project/global Skills, Playbook-as-Skill, Instruction Rules in `AGENTS.md`, and Command Rules as `.codex/rules/*.rules`; Prompt export is blocked as unsupported until explicit Prompt-as-Skill conversion exists. |
| FM-423 | Implemented | Gemini CLI exporter supports project/global Prompt commands and Instruction Rules in `GEMINI.md`; Skill, Playbook, and Command Rule exports are blocked as unsupported until validation/support exists. |
| FM-430 | Implemented | Import scanner covers Claude Code, Codex, and Gemini CLI project/global paths and reports collisions without writing. |
| FM-431 | Implemented | Import adoption supports Copy and Adopt preview/apply plans, source snapshot checks, and lockfile merge records. |
| FM-440 | Implemented | Sync UI exposes target and Project/Global User scope selection, project target profiles can attach assets per target, Settings manages global target profiles, and global apply requires second confirmation. |
| FM-441 | Implemented | Desktop asset editor supports first-class Playbook, Instruction Rule, and Command Rule creation/editing while keeping existing Prompt/Skill editors. |
| FM-442 | Implemented | Import page exposes target/scope scan, per-candidate Copy/Adopt/Skip decisions, preview, conflicts, and apply. |
| FM-450 | Implemented | Backend rejects global apply without exact acknowledgement of the cached mutating paths. |
| FM-451 | Implemented | Exporter/import docs and smoke checklist describe target matrix, Rule subtypes, unsupported mappings, scope safety, and import/adoption behavior. |
| FM-452 | Implemented | Manual smoke checklist covers Claude Code, Codex, Gemini CLI, import, unsupported mappings, and global confirmation. |

## Execution Principles

- Keep the current v0.1 Claude Code path working during every phase.
- Add tests before changing core sync behavior.
- Implement target support through adapters, not conditional logic scattered
  through UI commands.
- Treat global writes as high-risk.
- Treat unsupported target mappings as blocking preview errors.
- Do not execute scripts, hooks, shell snippets, or generated commands during
  Flowmint sync.

## Phase 1: Domain Model And Manifest Foundation

### FM-401: Add Sync Scope Model

Files:

- Modify: `crates/flowmint-core/src/project/manifest.rs`
- Modify: `crates/flowmint-core/src/sync/plan.rs`
- Modify: `apps/desktop/src/api/sync.ts`
- Modify: `apps/desktop/src-tauri/src/commands/sync.rs`
- Test: `crates/flowmint-core/tests/sync_scope_tests.rs`

Work:

1. Add `SyncScope` enum with `Project` and `GlobalUser`.
2. Add scope serialization as `project` and `global-user`.
3. Add target path resolver helpers that return project-local or user-global
   roots.
4. Add scope to `SyncPlan`, plan fingerprinting, and plan cache identity.
5. Keep default scope as `Project`.

Acceptance criteria:

- Existing v0.1 sync calls still default to project scope.
- Two otherwise identical plans with different scopes have different plan IDs.
- Global paths are never resolved through project path joins.

### FM-402: Upgrade Project Manifest To Multi-Export V2

Files:

- Modify: `crates/flowmint-core/src/project/manifest.rs`
- Modify: `crates/flowmint-core/src/project/model.rs`
- Modify: `crates/flowmint-core/src/project/store.rs`
- Test: `crates/flowmint-core/tests/project_manifest_v2_tests.rs`

Work:

1. Add `ExportProfile` with `target`, `scope`, `prompts`, `skills`,
   `playbooks`, `instruction_rules`, and `command_rules`.
2. Parse existing v0.1 `[export]` + `[attach]` into one `ExportProfile`.
3. Render v0.1 format when a manifest has exactly one `claude-code` project
   profile with only prompts and skills.
4. Render `[[exports]]` when the user adds another target, scope, Playbook, or
   Rule.
5. Validate target ID and attached asset IDs.

Acceptance criteria:

- Existing `.flowmint.toml` files load without data loss.
- v2 manifests round-trip without reordering asset IDs.
- Unknown target IDs are preserved for display but preview returns unsupported
  mapping errors.

### FM-403: Add Target Capability Registry

Files:

- Create: `crates/flowmint-core/src/exporters/capabilities.rs`
- Modify: `crates/flowmint-core/src/exporters/mod.rs`
- Test: `crates/flowmint-core/tests/export_capability_tests.rs`

Work:

1. Define target IDs: `claude-code`, `codex`, `gemini-cli`.
2. Define capabilities by asset type and scope.
3. Define unsupported mapping reasons.
4. Expose capabilities to Tauri for UI display.

Acceptance criteria:

- UI can ask the backend which targets support Prompt, Skill, Playbook,
  Instruction Rule, and Command Rule.
- Codex Prompt command export is marked unsupported unless user chooses
  Prompt-as-Skill conversion.
- Command Rules are initially supported for Codex only.

### FM-404: Add Global Sync Profile Store

Files:

- Create: `crates/flowmint-core/src/project/global_profiles.rs`
- Modify: `crates/flowmint-core/src/project/mod.rs`
- Modify: `crates/flowmint-core/src/store/home.rs`
- Test: `crates/flowmint-core/tests/global_sync_profile_tests.rs`

Work:

1. Store global profiles at `~/.flowmint/global-sync-profiles.toml`.
2. Use the same attachment fields as manifest v2 export profiles.
3. Require `scope = "global-user"` for all global profiles.
4. Reject project scope entries in the global profile store.
5. Expose list, get, save, attach, and detach operations to Tauri.

Acceptance criteria:

- Users can manage all-project personal sync profiles without tying them to one
  project.
- Global profile attachments are independent from `.flowmint.toml`.
- Invalid project-scoped global profile entries fail validation.

## Phase 2: Rule And Playbook Assets

### FM-410: Implement Rule Asset Store

Files:

- Modify: `crates/flowmint-core/src/asset/model.rs`
- Create: `crates/flowmint-core/src/asset/rule.rs`
- Modify: `crates/flowmint-core/src/asset/store.rs`
- Modify: `crates/flowmint-core/src/asset/id.rs`
- Modify: `crates/flowmint-core/src/validation/mod.rs`
- Test: `crates/flowmint-core/tests/rule_asset_tests.rs`
- Test: `crates/flowmint-core/tests/asset_store_tests.rs`

Work:

1. Add `AssetType::InstructionRule` and `AssetType::CommandRule`.
2. Store rules under `~/.flowmint/rules/<rule-id>.md`.
3. Store metadata with rule subtype, path globs, command prefix, decision, and
   target compatibility.
4. Validate safe IDs, non-empty body, and valid command rule decisions.
5. List, get, create, update, and delete Rule assets through existing asset
   APIs.

Acceptance criteria:

- Rule assets appear in library listing.
- Instruction Rule can have zero or more path globs.
- Command Rule requires a non-empty command prefix and decision.
- Invalid command decisions are rejected before write.

### FM-411: Implement First-Class Playbook Asset Store

Files:

- Modify: `crates/flowmint-core/src/asset/model.rs`
- Create: `crates/flowmint-core/src/asset/playbook.rs`
- Modify: `crates/flowmint-core/src/asset/store.rs`
- Modify: `crates/flowmint-core/src/store/template_store.rs`
- Modify: `crates/flowmint-core/src/validation/mod.rs`
- Test: `crates/flowmint-core/tests/playbook_asset_tests.rs`
- Test: `crates/flowmint-core/tests/template_store_tests.rs`

Work:

1. Add `PlaybookAsset` with trigger, inputs, steps, verification,
   failure-handling, side-effect level, and invocation recommendation.
2. Store playbooks under `~/.flowmint/playbooks/<playbook-id>.md` with metadata.
3. Keep existing playbook-tagged Skills valid.
4. Add promotion logic from Skill tagged `playbook` to Playbook asset.
5. Render target-neutral `SKILL.md` preview from Playbook.

Acceptance criteria:

- Existing Playbook Skills still list and export as Skills.
- New Playbook assets can be created independently.
- A Playbook with no steps fails validation.
- Side-effect level is visible to exporters and UI.

## Phase 3: Exporter Architecture

### FM-420: Introduce Exporter Trait And Planner Router

Files:

- Create: `crates/flowmint-core/src/exporters/target.rs`
- Modify: `crates/flowmint-core/src/exporters/mod.rs`
- Modify: `crates/flowmint-core/src/exporters/claude_code.rs`
- Modify: `crates/flowmint-core/src/sync/apply.rs`
- Modify: `apps/desktop/src-tauri/src/commands/sync.rs`
- Test: `crates/flowmint-core/tests/exporter_router_tests.rs`

Work:

1. Define an exporter interface with `target_id`, `preview`, and render helpers.
2. Route preview/apply by target and scope.
3. Move Claude Code hardcoded target checks behind the router.
4. Preserve backend-owned plan regeneration before apply.

Acceptance criteria:

- Existing Claude Code tests still pass.
- Unknown target preview returns a structured unsupported-target error.
- Apply refuses cached plans whose regenerated target/scope differs.

### FM-421: Extend Claude Code Exporter

Files:

- Modify: `crates/flowmint-core/src/exporters/claude_code.rs`
- Modify: `crates/flowmint-core/src/exporters/target.rs`
- Modify: `crates/flowmint-core/src/sync/apply.rs`
- Test: `crates/flowmint-core/tests/claude_code_planner_tests.rs`
- Test: `crates/flowmint-core/tests/apply_sync_tests.rs`
- Test: `crates/flowmint-core/tests/exporter_router_tests.rs`

Work:

1. Add global user scope path resolution.
2. Export global Prompt commands to `~/.claude/commands/<id>.md`.
3. Export global Skills to `~/.claude/skills/<id>/`.
4. Export Playbooks as Claude Skills.
5. Export Instruction Rules to `.claude/rules/<id>.md` or
   `~/.claude/rules/<id>.md`.
6. Keep project `CLAUDE.md` managed block behavior.
7. Write project lockfiles to `.flowmint.lock` and global lockfiles to
   `~/.flowmint/global-sync.lock`.
8. Return unsupported mapping conflicts for Claude Code Command Rules.
9. Keep UI-level global confirmation hardening in FM-450.

Acceptance criteria:

- Project exports match v0.1 output for existing Prompt and Skill assets.
- Global exports never write inside a project directory.
- Rule `paths` metadata renders as frontmatter for Claude Code.
- Playbook exports include side-effect warnings in the generated `SKILL.md`.
- Claude Code Command Rules block preview/apply with `UnsupportedMapping`.
- Global apply records generated outputs in `global-sync.lock`.

### FM-422: Add Codex Exporter

Files:

- Create: `crates/flowmint-core/src/exporters/codex.rs`
- Modify: `crates/flowmint-core/src/exporters/mod.rs`
- Modify: `crates/flowmint-core/src/exporters/target.rs`
- Modify: `crates/flowmint-core/src/sync/lockfile.rs`
- Test: `crates/flowmint-core/tests/codex_exporter_tests.rs`

Work:

1. Export Skills to project `.codex/skills/<id>/` or global
   `~/.codex/skills/<id>/`.
2. Export Playbooks as Codex Skills.
3. Export Instruction Rules into managed blocks in `AGENTS.md` or
   `~/.codex/AGENTS.md`.
4. Export Codex Command Rules to `.codex/rules/<id>.rules` or
   `~/.codex/rules/<id>.rules`.
5. Mark Prompt command export unsupported by default.
6. Add optional Prompt-as-Skill conversion later in the same exporter only when
   the UI exposes that conversion explicitly.

Acceptance criteria:

- Codex exporter writes Skills to `.codex/skills/`, while import scanning also
  recognizes legacy `.agents/skills` for existing user content.
- Codex instruction rules do not use `.codex/rules/`.
- Codex command rules produce valid `prefix_rule()` entries with match examples.
- Prompt attachment to Codex without conversion returns an unsupported mapping.

### FM-423: Add Gemini CLI Exporter

Files:

- Create: `crates/flowmint-core/src/exporters/gemini_cli.rs`
- Modify: `crates/flowmint-core/src/exporters/mod.rs`
- Modify: `crates/flowmint-core/src/exporters/target.rs`
- Test: `crates/flowmint-core/tests/gemini_cli_exporter_tests.rs`

Work:

1. Export Prompt commands to `.gemini/commands/<id>.toml` or
   `~/.gemini/commands/<id>.toml`.
2. Export Instruction Rules into managed blocks in `GEMINI.md` or
   `~/.gemini/GEMINI.md`.
3. Export Skills and Playbooks to `.gemini/skills/<id>/` or
   `~/.gemini/skills/<id>/` only after local validation confirms the installed
   Gemini CLI scans those paths.
4. If local validation is not available, keep Skill/Playbook export disabled
   with an actionable unsupported mapping message.
5. Do not generate Gemini command files for Command Rules in this phase.

Acceptance criteria:

- Gemini Prompt command TOML includes `description` and `prompt`.
- Gemini command arguments use `{{args}}` when the Flowmint prompt declares
  variables.
- GEMINI.md managed block preserves user-authored content outside markers.
- Gemini Skill/Playbook export is either validated and enabled or blocked with a
  clear reason.

## Phase 4: Import And Adoption

### FM-430: Add Import Scanner

Files:

- Create: `crates/flowmint-core/src/import/mod.rs`
- Create: `crates/flowmint-core/src/import/claude_code.rs`
- Create: `crates/flowmint-core/src/import/codex.rs`
- Create: `crates/flowmint-core/src/import/gemini_cli.rs`
- Add tests under: `crates/flowmint-core/tests/import_*_tests.rs`

Work:

1. Scan target/scope paths read-only.
2. Detect prompts, skills, playbooks, and rules where the target format makes
   that possible.
3. Return import candidates with source path, inferred asset type, target, scope,
   and confidence.
4. Detect ID collisions with existing Flowmint assets.
5. Do not write lockfiles during scan.

Acceptance criteria:

- Scanner can run without modifying the filesystem.
- Import candidates are deterministic and sorted.
- Collisions are visible before import.

### FM-431: Add Import Adoption Plans

Files:

- Create: `crates/flowmint-core/src/import/adopt.rs`
- Modify: `crates/flowmint-core/src/sync/lockfile.rs`
- Test: `crates/flowmint-core/tests/import_adoption_tests.rs`

Work:

1. Implement `Copy into library`.
2. Implement `Adopt into Flowmint` as a preview/apply plan.
3. Add lock records for adopted files only after apply.
4. Reject adoption if source file changes between preview and apply.

Acceptance criteria:

- Copy import creates library assets and leaves source files unmanaged.
- Adopt import marks files managed only after apply.
- Edited source files block adoption apply.

## Phase 5: Desktop UI

### FM-440: Add Scope And Target Selection UI

Files:

- Modify: `apps/desktop/src/pages/ProjectDetailPage.tsx`
- Modify: `apps/desktop/src/pages/SyncPage.tsx`
- Modify: `apps/desktop/src/api/projects.ts`
- Modify: `apps/desktop/src/api/sync.ts`
- Modify: `apps/desktop/src/i18n/messages.ts`
- Add tests under: `apps/desktop/tests/`

Work:

1. Add target profile list to project detail.
2. Add scope selector with Project as default.
3. Show Global User as high-risk with exact destination root.
4. Allow attach/detach per target profile.
5. Show unsupported mappings in the attach matrix.

Acceptance criteria:

- User can attach one asset to Claude Code project scope without changing the
  old workflow.
- User cannot apply global sync without a second confirmation.
- Unsupported cells are disabled and explain why.

### FM-441: Add Rule And Playbook Editors

Files:

- Modify: `apps/desktop/src/pages/AssetEditorPage.tsx`
- Modify: `apps/desktop/src/pages/assetEditorModel.ts`
- Modify: `apps/desktop/src/components/AssetList.tsx`
- Modify: `apps/desktop/src/api/assets.ts`
- Modify: `apps/desktop/src/i18n/messages.ts`
- Add tests under: `apps/desktop/tests/`

Work:

1. Add Rule creation flow.
2. Add Instruction Rule editor.
3. Add Command Rule editor.
4. Add first-class Playbook editor.
5. Add promotion flow from Playbook Skill to Playbook.
6. Add target compatibility warning panel.

Acceptance criteria:

- Rule validation errors are shown before save.
- Command Rule editor defaults to `prompt`, not `allow`.
- Playbook side-effect level is required.
- Existing Skill editor still supports v0.1 Playbook Skills.

### FM-442: Add Import UI

Files:

- Create: `apps/desktop/src/pages/ImportPage.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/api/import.ts`
- Modify: `apps/desktop/src/i18n/messages.ts`
- Add tests under: `apps/desktop/tests/`

Work:

1. Add import entry point from Settings and Assets.
2. Let user choose target and scope.
3. Show scan results.
4. Let user choose Copy, Adopt, or Skip per candidate.
5. Show collision resolution before apply.

Acceptance criteria:

- Scan is visibly read-only.
- User can import without syncing back to the source tool.
- Adoption uses the same preview/apply safety model as sync.

## Phase 6: Safety, Docs, And Release Verification

### FM-450: Harden Global Write Safety

Files:

- Modify: `crates/flowmint-core/src/fs_safety.rs`
- Modify: `crates/flowmint-core/src/sync/apply.rs`
- Modify: `apps/desktop/src/pages/SyncPage.tsx`
- Test: `crates/flowmint-core/tests/global_write_safety_tests.rs`

Work:

1. Add explicit global write acknowledgement to plan cache.
2. Reject apply if acknowledgement is missing or stale.
3. Block writes to known managed/admin paths.
4. Block unsafe symlinks and path escapes for global roots.

Acceptance criteria:

- Project apply cannot be replayed as global apply.
- Global apply cannot proceed from a frontend-supplied operation list.
- Symlinked global destination paths are conflicts.

### FM-451: Update Documentation

Files:

- Modify: `docs/asset-management-and-import.md`
- Modify: `docs/exporters.md`
- Modify: `docs/manual-smoke-test.md`
- Create or modify release checklist docs as needed.

Work:

1. Document scope selection.
2. Document Rule subtypes.
3. Document target capability matrix.
4. Document import/adoption behavior.
5. Document unsupported mappings.

Acceptance criteria:

- Docs do not describe Codex prompt commands as supported.
- Docs explain that Codex `.rules` are command execution rules.
- Docs explain global writes and second confirmation.

### FM-452: Add End-To-End Smoke Paths

Files:

- Modify: `docs/manual-smoke-test.md`
- Add integration tests where practical.

Smoke scenarios:

1. Claude Code project sync: Prompt, Skill, Playbook, Instruction Rule.
2. Claude Code global sync: Prompt and Skill with second confirmation.
3. Codex project sync: Skill, Playbook, Instruction Rule, Command Rule.
4. Gemini project sync: Prompt command and Instruction Rule.
5. Import existing Claude Code project assets into Flowmint.
6. Attempt unsupported Codex Prompt command export and confirm apply is blocked.
7. Attempt global write without confirmation and confirm apply is blocked.

Acceptance criteria:

- All smoke scenarios have expected generated file paths.
- Unsupported scenarios are verified as blocked.
- Existing v0.1 Claude Code smoke still passes.

## Dependency Order

1. FM-401
2. FM-402
3. FM-403
4. FM-404
5. FM-410 and FM-411 in parallel
6. FM-420
7. FM-421
8. FM-422 and FM-423 in parallel after FM-420
9. FM-430
10. FM-431
11. FM-440 and FM-441 after domain/exporter APIs stabilize
12. FM-442 after FM-430
13. FM-450
14. FM-451
15. FM-452

## Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Target tool formats change | Generated files stop working | Keep all target-specific logic inside exporters and cite source docs in exporter docs. |
| Global writes surprise users | User's all-project agent behavior changes | Default to project scope and require second confirmation. |
| Codex rules confused with instruction rules | Dangerous permissions or wrong behavior | Separate Instruction Rule and Command Rule in model, UI, docs, and exporters. |
| Gemini skill path drift | Broken Skill/Playbook export | Gate Gemini Skill export behind local CLI validation before enabling. |
| Playbook duplication with Skill | Confusing asset model | Preserve tagged Skills and add explicit promotion flow. |
| Manifest migration breaks old projects | Existing users lose sync | Parse v0.1 forever and render v0.1 when no v2-only features are used. |

## Plan Review Checklist

- Every new asset type has backend model, store, validation, UI, exporter, and
  tests.
- Every target has project/global scope handling or an explicit unsupported
  mapping.
- Global user profiles are stored outside individual project manifests.
- Every global write has second confirmation.
- Prompt export gaps are explicit for Codex.
- Rules are not treated as one universal feature across tools.
- Playbooks are first-class in Flowmint but exported through target-native Skill
  layouts.
- Import is read-only until the user applies an adoption plan.
- Existing v0.1 Claude Code behavior remains covered.
- Release smoke includes both positive and blocked scenarios.

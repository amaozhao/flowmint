# Flowmint UI-first MVP Task Plan

> Source: `docs/mvp.md`
>
> Purpose: turn the UI-first MVP document into an executable task plan. This plan is intentionally scoped to the MVP: desktop app first, Rust Core as the business engine, file-system storage, safe sync preview, and Claude Code export only.

## Planning Principles

1. Desktop UI is the primary product surface. CLI work is postponed until after v0.1 unless needed for diagnostics.
2. Rust Core owns all business file operations: asset parsing, validation, project manifest handling, sync planning, exporter logic, hashing, and write safety.
3. React/Tauri UI never writes workflow files directly. It calls Tauri commands only.
4. Every sync writes through a generated `SyncPlan`; Apply Sync is a separate action.
5. Default behavior must not silently overwrite user-authored files.
6. MVP supports Prompt and Skill. Playbook exists only as a Skill template shape. Rule is deferred.
7. MVP exports only to Claude Code.

## Release Shape

| Release | Goal | User-visible completion |
|---|---|---|
| v0.1-alpha | Desktop shell + local asset library | User can create/edit a Prompt and a Skill in the UI, restart the app, and see them again. |
| v0.1-beta | Project binding + Sync Preview | User can add a local project, attach a Prompt/Skill, and see a safe Claude Code sync plan without writing files. |
| v0.1 | Apply Sync + packageable MVP | User can create Prompt/Skill assets, attach them to a project, preview sync, apply sync, and see Claude Code files generated safely. |

## Workstreams

| Workstream | Owner lane | Main responsibility |
|---|---|---|
| Core | Rust | File-system source of truth, domain models, validation, project manifests, sync planning, exporter safety. |
| Desktop | React/Tauri | Navigation, pages, editors, diff preview, conflict/result UI, Tauri API wrappers. |
| Integration | Rust + Tauri | Command contracts, app state, path permissions, plan cache, error mapping. |
| QA/Release | Cross-cutting | Unit tests, integration tests, idempotency tests, CI, packaging, smoke tests, README. |

## Phase 0: Repository And Architecture Baseline

### FM-001: Initialize Workspace Layout

**Release:** v0.1-alpha
**Depends on:** none

**Create/modify:**
- `Cargo.toml`
- `package.json`
- `crates/flowmint-core/`
- `apps/desktop/`
- `examples/prompts/`
- `examples/skills/`
- `examples/projects/`
- `docs/storage.md`
- `docs/exporters.md`
- `docs/ui.md`

**Scope:**
- Create the Rust workspace.
- Create the Tauri 2 desktop app shell under `apps/desktop`.
- Add React, Vite, and TypeScript setup.
- Keep `flowmint-cli` out of the first implementation unless a minimal debug binary becomes necessary.

**Done when:**
- `cargo metadata` succeeds.
- Frontend dependencies install.
- The desktop app can render a placeholder screen.
- The repo structure matches the MVP architecture.

### FM-002: Add Shared Tooling And CI Skeleton

**Release:** v0.1-alpha
**Depends on:** FM-001

**Create/modify:**
- `.github/workflows/ci.yml`
- `crates/flowmint-core/Cargo.toml`
- `apps/desktop/package.json`
- `apps/desktop/tsconfig.json`

**Scope:**
- Add Rust fmt, clippy, and tests to CI.
- Add frontend typecheck and build to CI.
- Add a single release-readiness command in `package.json` if useful.

**Done when:**
- CI can run `cargo fmt --check`, `cargo clippy`, `cargo test`, frontend typecheck, and frontend build.
- No packaging workflow is required yet.

## Phase 1: v0.1-alpha, Desktop Shell And Local Asset Library

### FM-101: Core App State And Library Home

**Release:** v0.1-alpha
**Depends on:** FM-001

**Create/modify:**
- `crates/flowmint-core/src/store/home.rs`
- `crates/flowmint-core/src/store/config.rs`
- `crates/flowmint-core/src/store/recent_projects.rs`
- `crates/flowmint-core/src/lib.rs`
- `apps/desktop/src-tauri/src/main.rs`

**Scope:**
- Resolve the default Flowmint home directory: `~/.flowmint`.
- Support custom library path during onboarding.
- Create the expected library folders: `prompts/`, `skills/`, `templates/`, `cache/`, `backups/`.
- Load and persist `config.toml` and `recent-projects.toml`.
- Expose Tauri commands:
  - `get_app_state`
  - `init_library`
  - `open_library_folder`

**Done when:**
- First startup can detect whether a library exists.
- `init_library(None)` creates the default structure.
- Reopening the app returns the same library state.

### FM-102: Desktop App Shell And Onboarding

**Release:** v0.1-alpha
**Depends on:** FM-101

**Create/modify:**
- `apps/desktop/src/main.tsx`
- `apps/desktop/src/app/App.tsx`
- `apps/desktop/src/components/AppSidebar.tsx`
- `apps/desktop/src/components/TopBar.tsx`
- `apps/desktop/src/pages/DashboardPage.tsx`
- `apps/desktop/src/pages/SettingsPage.tsx`
- `apps/desktop/src/pages/OnboardingPage.tsx`
- `apps/desktop/src/api/tauri.ts`
- `apps/desktop/src/api/settings.ts`

**Scope:**
- Build the desktop navigation shell:
  - Overview
  - Assets
  - Projects
  - Sync
  - Settings
- Add onboarding for local library creation.
- Add dashboard counts and recent project placeholders from `AppState`.
- Add settings display for Flowmint home directory.

**Done when:**
- New users see onboarding.
- Existing users land on Dashboard.
- Dashboard can display current asset counts, even if zero.

### FM-103: Asset Domain Models And Validation

**Release:** v0.1-alpha
**Depends on:** FM-101

**Create/modify:**
- `crates/flowmint-core/src/asset/mod.rs`
- `crates/flowmint-core/src/asset/model.rs`
- `crates/flowmint-core/src/asset/id.rs`
- `crates/flowmint-core/src/validation/mod.rs`
- `crates/flowmint-core/src/error.rs`

**Scope:**
- Define `AssetType`, `AssetSummary`, `PromptAsset`, `SkillAsset`, `ValidationStatus`, and `ValidationReport`.
- Enforce safe IDs: `a-z0-9-_`.
- Validate required fields:
  - Prompt requires ID, name, and body.
  - Skill requires ID and non-empty `SKILL.md`.
- Keep Playbook as an asset type only where needed for template selection; do not implement an execution engine.

**Done when:**
- Invalid IDs fail validation.
- Empty Prompt/Skill content fails validation.
- Duplicate Skill IDs can be detected before write.

### FM-104: Prompt Parser And Writer

**Release:** v0.1-alpha
**Depends on:** FM-103

**Create/modify:**
- `crates/flowmint-core/src/asset/prompt.rs`
- `crates/flowmint-core/src/store/asset_store.rs`
- `crates/flowmint-core/tests/prompt_asset_tests.rs`

**Scope:**
- Store prompts as `~/.flowmint/prompts/<id>.md`.
- Support front matter or TOML side metadata only if it keeps parsing simple; otherwise store metadata in a predictable header block.
- Implement create, load, update, list, and validate behavior.
- Preserve prompt body Markdown exactly enough for editing round trips.

**Done when:**
- Creating a prompt writes the expected file.
- Editing a prompt updates the same file.
- Restart-style reload lists the saved prompt.
- Required-field failures do not write partial files.

### FM-105: Skill Parser, Writer, And Templates

**Release:** v0.1-alpha
**Depends on:** FM-103

**Create/modify:**
- `crates/flowmint-core/src/asset/skill.rs`
- `crates/flowmint-core/src/store/template_store.rs`
- `crates/flowmint-core/tests/skill_asset_tests.rs`
- `examples/skills/basic-skill/SKILL.md`
- `examples/skills/playbook-skill/SKILL.md`

**Scope:**
- Store skills as `~/.flowmint/skills/<id>/SKILL.md`.
- Read optional `metadata.toml`, `examples/`, and `resources/`.
- Create Basic Skill and Playbook Skill templates.
- Do not copy or execute `scripts/` in MVP.

**Done when:**
- Creating a Skill writes `SKILL.md`.
- Empty `SKILL.md` is rejected.
- Skill ID collisions are rejected.
- Existing skill folders can be loaded and listed.

### FM-106: Asset Tauri Commands

**Release:** v0.1-alpha
**Depends on:** FM-104, FM-105

**Create/modify:**
- `apps/desktop/src-tauri/src/main.rs`
- `apps/desktop/src-tauri/src/commands/assets.rs`
- `apps/desktop/src/api/assets.ts`

**Scope:**
- Expose:
  - `list_assets`
  - `get_asset`
  - `create_asset`
  - `update_asset`
  - `delete_asset`
  - `validate_asset`
- Add TypeScript wrappers so pages do not call `invoke(...)` directly.
- Normalize backend errors into UI-friendly messages.

**Done when:**
- Frontend can create, update, list, and validate Prompt/Skill through command wrappers.
- Commands reject unsafe paths and invalid IDs.

### FM-107: Assets Page And Editors

**Release:** v0.1-alpha
**Depends on:** FM-106

**Create/modify:**
- `apps/desktop/src/pages/AssetsPage.tsx`
- `apps/desktop/src/pages/AssetEditorPage.tsx`
- `apps/desktop/src/components/AssetList.tsx`
- `apps/desktop/src/components/AssetCard.tsx`
- `apps/desktop/src/components/AssetTypeBadge.tsx`
- `apps/desktop/src/components/TagInput.tsx`
- `apps/desktop/src/components/MarkdownEditor.tsx`
- `apps/desktop/src/components/MetadataForm.tsx`
- `apps/desktop/src/components/EmptyState.tsx`
- `apps/desktop/src/components/ErrorBoundary.tsx`

**Scope:**
- Add asset list with search, type filters, and tag display.
- Add Prompt editor fields: ID, name, description, tags, body, variables placeholder, preview, validate.
- Add Skill editor fields: ID, name, description, tags, `SKILL.md`, metadata view, resources/examples list, validate.
- Use a plain textarea or lightweight Markdown editor for the first pass; Monaco/CodeMirror is optional later.

**Done when:**
- User can create one Prompt and one Skill from the UI.
- Validation errors block save and explain why.
- Closing and reopening the app still shows both assets.

## Phase 2: v0.1-beta, Project Binding And Sync Preview

### FM-201: Project Manifest Parser And Recent Projects

**Release:** v0.1-beta
**Depends on:** FM-101, FM-103

**Create/modify:**
- `crates/flowmint-core/src/project/mod.rs`
- `crates/flowmint-core/src/project/manifest.rs`
- `crates/flowmint-core/src/project/recent.rs`
- `crates/flowmint-core/tests/project_manifest_tests.rs`

**Scope:**
- Parse and write `project/.flowmint.toml`.
- Support:
  - `[project]`
  - `[export]`
  - `[attach] prompts`
  - `[attach] skills`
- Maintain `~/.flowmint/recent-projects.toml`.
- Default exporter target is `claude-code`.
- Keep Playbook out of the project manifest in v0.1; Playbook is only a Skill template shape until a dedicated exporter/UI path exists.

**Done when:**
- Adding a project initializes `.flowmint.toml` if missing.
- Existing `.flowmint.toml` is read without overwriting user choices.
- Recent projects persist across app restarts.

### FM-202: Project Tauri Commands

**Release:** v0.1-beta
**Depends on:** FM-201

**Create/modify:**
- `apps/desktop/src-tauri/src/commands/projects.rs`
- `apps/desktop/src/api/projects.ts`

**Scope:**
- Expose:
  - `list_projects`
  - `add_project`
  - `get_project`
  - `attach_asset`
  - `detach_asset`
- Return missing asset state when a project references an asset that no longer exists.

**Done when:**
- UI can add a local project path.
- UI can read project detail and attached assets.
- Missing assets are represented explicitly instead of disappearing.

### FM-203: Projects Page And Attach Modal

**Release:** v0.1-beta
**Depends on:** FM-202, FM-107

**Create/modify:**
- `apps/desktop/src/pages/ProjectsPage.tsx`
- `apps/desktop/src/pages/ProjectDetailPage.tsx`
- `apps/desktop/src/components/ProjectList.tsx`
- `apps/desktop/src/components/AttachedAssetList.tsx`
- `apps/desktop/src/components/AttachAssetModal.tsx`

**Scope:**
- Add Projects list and detail view.
- Add local project directory picker.
- Show whether a project is initialized.
- Add attach/detach flow for Prompt and Skill.
- Prevent duplicate attaches.

**Done when:**
- User can add a local project.
- User can attach and detach assets from UI.
- `.flowmint.toml` changes match the UI state.

### FM-204: SyncPlan Model And Plan Cache

**Release:** v0.1-beta
**Depends on:** FM-201

**Create/modify:**
- `crates/flowmint-core/src/sync/mod.rs`
- `crates/flowmint-core/src/sync/plan.rs`
- `crates/flowmint-core/src/sync/conflict.rs`
- `crates/flowmint-core/src/sync/plan_cache.rs`
- `crates/flowmint-core/tests/sync_plan_tests.rs`

**Scope:**
- Define `SyncPlan`, `SyncOperation`, and `SyncConflict`.
- Include operations for create file, update file, create dir, delete generated file, noop.
- Add a backend-held plan cache or deterministic re-plan mechanism.
- Do not let `apply_sync` trust arbitrary frontend-supplied operations.

**Done when:**
- A SyncPlan can be generated and serialized for the UI.
- Each plan has a stable `plan_id` or deterministic regeneration key.
- The model can represent conflicts without applying writes.

### FM-205: Claude Code Sync Planner

**Release:** v0.1-beta
**Depends on:** FM-104, FM-105, FM-204

**Create/modify:**
- `crates/flowmint-core/src/exporters/mod.rs`
- `crates/flowmint-core/src/exporters/claude_code.rs`
- `crates/flowmint-core/src/sync/diff.rs`
- `crates/flowmint-core/src/fs_safety/mod.rs`
- `crates/flowmint-core/tests/claude_code_planner_tests.rs`

**Scope:**
- Plan Prompt export to `.claude/commands/<prompt-id>.md`.
- Plan Skill export to `.claude/skills/<id>/`.
- Plan full Skill directory export for supported files: `SKILL.md`, optional `metadata.toml`, optional `examples/`, and optional `resources/`.
- Plan `CLAUDE.md` managed block create/update.
- Detect conflicts:
  - Target file exists but is not in `.flowmint.lock`.
  - Target hash differs from lock.
  - `CLAUDE.md` managed marker is incomplete.
  - Target path is a symlink.
  - Asset ID can escape project directory after path normalization.
  - Output directory is not writable.
- Do not write files in this task.

**Done when:**
- Preview shows creates/updates/noops/conflicts for Claude Code output.
- Existing unmanaged target files are conflicts.
- Broken managed block is a conflict.
- Path traversal IDs cannot generate operations.

### FM-206: Sync Preview UI

**Release:** v0.1-beta
**Depends on:** FM-205

**Create/modify:**
- `apps/desktop/src-tauri/src/commands/sync.rs`
- `apps/desktop/src/api/sync.ts`
- `apps/desktop/src/pages/SyncPreviewPage.tsx`
- `apps/desktop/src/components/SyncOperationList.tsx`
- `apps/desktop/src/components/DiffViewer.tsx`
- `apps/desktop/src/components/ConflictBanner.tsx`

**Scope:**
- Expose `preview_sync`.
- Show operation summary: creates, updates, conflicts, noops.
- Show file list and diff/content preview.
- Show conflict reason with available MVP actions:
  - Cancel
  - Open File
- Do not expose `Mark as unmanaged` or `Force overwrite` in v0.1 unless backend-owned commands, lockfile semantics, and safe apply behavior are implemented first.

**Done when:**
- User can preview sync for a project.
- Preview clearly shows what files would be created or changed.
- Apply Sync is hidden or disabled until Phase 3.

## Phase 3: v0.1, Apply Sync And MVP Release

### FM-301: Safe Apply Sync Engine

**Release:** v0.1
**Depends on:** FM-205, FM-206

**Create/modify:**
- `crates/flowmint-core/src/sync/apply.rs`
- `crates/flowmint-core/src/sync/lockfile.rs`
- `crates/flowmint-core/src/fs_safety/write.rs`
- `crates/flowmint-core/tests/apply_sync_tests.rs`

**Scope:**
- Implement `apply_sync(plan_id)` using backend-held or regenerated plan state.
- Recheck conflicts immediately before writing.
- Create directories and files atomically where practical.
- Refuse writes through dangerous symlinks.
- Refuse path traversal.
- Write `.flowmint.lock` after successful generated file writes.

**Done when:**
- Apply writes `.claude/skills/<id>/SKILL.md`.
- Apply copies supported Skill directory content when present: `metadata.toml`, `examples/`, and `resources/`.
- Apply writes `.claude/commands/<prompt-id>.md`.
- Apply updates only the Flowmint managed block in `CLAUDE.md`.
- Apply fails safely when conflicts appear after preview.

### FM-302: Idempotency And Lock Hash Tracking

**Release:** v0.1
**Depends on:** FM-301

**Create/modify:**
- `crates/flowmint-core/src/sync/hash.rs`
- `crates/flowmint-core/src/sync/lockfile.rs`
- `crates/flowmint-core/tests/sync_idempotency_tests.rs`
- `crates/flowmint-core/tests/conflict_detection_tests.rs`

**Scope:**
- Record source hash, output path, output hash, target, asset type, asset ID, and timestamp in `.flowmint.lock`.
- Repeated sync with unchanged assets should produce no file changes.
- If a generated file is edited outside Flowmint, preview must show conflict.

**Done when:**
- Two consecutive syncs produce the same lock data except allowed timestamp behavior.
- Manual modification of a generated file is detected.
- Unmanaged target files are never overwritten silently.

### FM-303: Apply Sync Desktop Flow

**Release:** v0.1
**Depends on:** FM-301, FM-302

**Create/modify:**
- `apps/desktop/src-tauri/src/commands/sync.rs`
- `apps/desktop/src/api/sync.ts`
- `apps/desktop/src/pages/SyncPreviewPage.tsx`
- `apps/desktop/src/pages/SyncResultPage.tsx`

**Scope:**
- Expose `apply_sync`.
- Enable Apply Sync only when the latest preview has no blocking conflict.
- Show success/failure result with affected files.
- Refresh project detail after apply.

**Done when:**
- User can complete the full MVP loop from UI:
  create Skill -> add project -> attach Skill -> preview sync -> apply sync -> inspect generated Claude Code files.

### FM-304: Core Test Coverage For MVP Safety

**Release:** v0.1
**Depends on:** FM-301, FM-302

**Create/modify:**
- `crates/flowmint-core/tests/`

**Scope:**
- Add tests for:
  - Prompt create/load/update.
  - Skill create/load/update.
  - Project manifest initialize/read/update.
  - Attach/detach idempotency.
  - SyncPlan generation.
  - `CLAUDE.md` managed block append/replace/conflict.
  - Symlink rejection.
  - Path traversal rejection.
  - Lock hash conflict detection.
  - Sync idempotency.

**Done when:**
- Core tests prove the MVP safety requirements.
- High-risk sync code is covered before packaging.

### FM-305: Desktop Build And Smoke Tests

**Release:** v0.1
**Depends on:** FM-303, FM-304

**Create/modify:**
- `apps/desktop/`
- `.github/workflows/ci.yml`
- `docs/manual-smoke-test.md`

**Scope:**
- Add a manual smoke test script/checklist:
  1. Launch app.
  2. Create library.
  3. Create Prompt.
  4. Create Skill.
  5. Add temp project.
  6. Attach Prompt and Skill.
  7. Preview sync.
  8. Apply sync.
  9. Verify `.claude/skills/<id>/SKILL.md`.
  10. Verify `.claude/commands/<prompt-id>.md`.
  11. Verify `CLAUDE.md` managed block.
  12. Run second sync and verify no extra changes.
  13. Manually edit generated file and verify conflict.
- Add frontend build verification to CI.
- Add Tauri desktop build command where feasible.

**Done when:**
- Local smoke test passes on the primary development OS.
- CI proves core tests and frontend build.

### FM-306: Packaging And Release Workflow

**Release:** v0.1
**Depends on:** FM-305

**Create/modify:**
- `.github/workflows/release.yml`
- `apps/desktop/src-tauri/tauri.conf.json`
- `README.md`

**Scope:**
- Configure Tauri bundling metadata.
- Add basic macOS/Linux/Windows packaging path.
- Avoid promising unsupported platforms until smoke tested.
- Add README quickstart matching the MVP loop.

**Done when:**
- At least one local desktop bundle can be produced.
- Release workflow is documented even if not fully automated for every OS.
- README explains storage, safety, and MVP usage.

## Backlog After v0.1

These were explicitly out of the original v0.1 MVP implementation path. Some
items have since been implemented by `docs/multi-agent-sync-task-plan.md`.

1. `flowmint-cli` as a first-class user entry.
2. Cursor exporter.
3. Search index with Tantivy or SQLite.
4. Git-based sync or registry.
5. Cloud sync, accounts, team permissions, marketplace.
6. AI chat, LLM API calls, agent runtime, eval system.
7. VS Code extension.

Implemented after the original MVP plan:

1. Codex exporter.
2. Gemini CLI exporter.
3. First-class Playbook editor/store.
4. Rule assets and target-specific Rule export.
5. Import scan/adoption.
6. Global user sync profiles.

## Parallel Execution Guidance

Safe parallel lanes after FM-001:

1. Core library lane: FM-101, FM-103, FM-104, FM-105.
2. Desktop shell lane: FM-102, then FM-107 after commands exist.
3. QA/tooling lane: FM-002 and early test harness setup.

Safe parallel lanes after v0.1-alpha:

1. Project management lane: FM-201, FM-202, FM-203.
2. Sync planning lane: FM-204, FM-205.
3. UI preview lane: FM-206 after the SyncPlan contract stabilizes.

Do not parallelize:

1. `apply_sync` before `preview_sync` and conflict rules are stable.
2. Desktop Apply Sync before the backend apply engine rechecks conflicts.
3. Packaging before the full MVP smoke path works locally.

## Verification Gates

### Gate A: v0.1-alpha

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/desktop && npm run typecheck && npm run build
```

Manual proof:
- Create local library.
- Create Prompt.
- Create Skill.
- Restart app.
- Confirm both assets load and can be edited.

### Gate B: v0.1-beta

Run all Gate A commands plus core project/sync-plan tests.

Manual proof:
- Add a local project.
- Initialize `.flowmint.toml`.
- Attach a Prompt and a Skill.
- Preview Claude Code sync.
- Confirm preview shows planned `.claude/skills/<id>/SKILL.md` without writing files.
- Confirm preview shows planned `.claude/commands/<prompt-id>.md` without writing files.
- Confirm unmanaged target file is shown as conflict.

### Gate C: v0.1

Run all Gate B commands plus idempotency and conflict tests.

Manual proof:
- Complete the full MVP loop:
  create Prompt and Skill -> add project -> attach Prompt and Skill -> preview sync -> apply sync.
- Confirm generated files:
  - `.claude/skills/<id>/SKILL.md`
  - `.claude/commands/<prompt-id>.md` when a Prompt is attached
  - `CLAUDE.md` managed block
  - `.flowmint.lock`
- Run sync again and confirm no extra changes.
- Modify a generated file manually and confirm conflict on next preview.

## Issue Creation Order

Recommended issue order:

1. FM-001
2. FM-002
3. FM-101
4. FM-103
5. FM-104
6. FM-105
7. FM-106
8. FM-102
9. FM-107
10. FM-201
11. FM-202
12. FM-203
13. FM-204
14. FM-205
15. FM-206
16. FM-301
17. FM-302
18. FM-303
19. FM-304
20. FM-305
21. FM-306

## MVP Completion Definition

MVP is complete only when all of the following are true:

1. Desktop app starts and can create a local Flowmint library.
2. User can create and edit Prompt and Skill assets.
3. Assets are stored in the local file system and survive restart.
4. User can add a local project and initialize `.flowmint.toml`.
5. User can attach and detach assets without duplicate bindings.
6. User can preview Claude Code sync before any write.
7. User can apply sync and generate Claude Code-compatible files.
8. Sync is idempotent.
9. User-authored files are not silently overwritten.
10. Symlinks, path traversal, broken managed markers, and modified generated files are handled as conflicts.
11. No cloud sync, account system, LLM API call, or agent runtime exists in the MVP.
12. README quickstart and manual smoke checklist match the shipped UI.

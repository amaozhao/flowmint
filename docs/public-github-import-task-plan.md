# Public GitHub Import Task Plan

Status: proposed implementation plan.

Goal: let users paste a public GitHub URL and import reusable Prompt, Skill,
Playbook, Instruction Rule, and Command Rule assets into the Flowmint library,
then use the existing preview/apply sync flow to send those assets to Claude
Code, Codex, or Gemini CLI.

Source decision: first version supports public GitHub repositories only. Private
repositories, tokens, GitHub account login, marketplace ranking, cloud sync, and
direct writes into agent configuration from GitHub are out of scope.

Official API reference:

- GitHub repository contents API:
  `https://docs.github.com/en/rest/repos/contents`
- GitHub commit/ref lookup:
  `https://docs.github.com/en/rest/commits/commits#get-a-commit`

## Current Gap

Current import support is local-only:

- `crates/flowmint-core/src/import/mod.rs` scans existing tool directories under
  a project or user home.
- `crates/flowmint-core/src/import/adopt.rs` reads local files from
  `source_path`.
- `apps/desktop/src/pages/ImportPage.tsx` only exposes target, scope, and local
  project path selection.
- Existing import adoption has `Copy`, `Adopt`, and `Skip`. Remote GitHub import
  must not expose `Adopt`, because Flowmint cannot manage a remote source file.

## Product Rules

1. GitHub is an import source, not a sync target.
2. Imported remote content must enter the Flowmint library first.
3. Flowmint must never execute scripts, hooks, commands, or generated code from
   GitHub during scan, preview, import, or sync.
4. GitHub import must be preview-first and apply-only-after-selection.
5. Public repository access must work without a GitHub token.
6. Private repository support must fail clearly with an authentication/private
   repository message, not a generic parsing or network error.
7. Branch names are mutable, so scan must resolve and record the concrete commit
   SHA used for import.
8. Remote import must support ID collision handling before apply.
9. Unsupported or skipped files must be visible as warnings; Flowmint must not
   silently drop files that are part of a Skill directory.
10. Imported assets remain subject to the existing target capability matrix when
    synced to Claude Code, Codex, and Gemini CLI.

## Architecture

Keep `flowmint-core` network-free. Add remote-import domain types and asset
conversion logic in core, but perform HTTPS requests in the Tauri desktop crate.

Flow:

1. UI receives a public GitHub URL.
2. Tauri parses the URL and fetches repository metadata, commit SHA, directory
   listings, and raw UTF-8 file contents from GitHub.
3. Tauri stores a short-lived remote import session in memory.
4. Core scans the fetched file tree into remote import candidates.
5. UI shows candidates, warnings, collisions, and editable destination IDs.
6. Tauri asks core to build a remote import plan from selected candidates.
7. Apply writes assets into `~/.flowmint/...` only.
8. The user attaches/syncs imported assets through the existing project/global
   sync flows.

Storage semantics:

- `~/.flowmint/import-sources/<asset-type>/<id>.json` is library-level
  provenance for a reusable Flowmint asset. For example:
  `~/.flowmint/import-sources/skills/research-helper.json`.
- This record answers "where did this library asset originally come from?"
  and follows the asset when it is reused across multiple projects or global
  sync profiles.
- The current project directory remains responsible for project binding and
  generated output state only:
  - `<project>/.flowmint.toml` records which library assets are attached to that
    project and target profile.
  - `<project>/.flowmint.lock` records files generated into the target tool for
    that project.
- If the user imports from GitHub while a project is selected and chooses
  "import and attach to current project", Flowmint updates
  `<project>/.flowmint.toml`. It does not duplicate the GitHub provenance JSON
  under the project directory.

Use the Contents API recursively for this MVP. The implementation must enforce
caps so a root repo URL cannot exhaust API quota, memory, or disk:

- Max directories listed: 50.
- Max files fetched: 200.
- Max single file size: 1 MiB.
- Max total fetched text: 10 MiB.
- Max recursion depth: 8.

The limits should be surfaced in warnings when reached.

## File Map

### Core

- Create: `crates/flowmint-core/src/import/remote.rs`
  - Remote import source/session/plan/item types.
  - Remote candidate detection.
  - Remote asset conversion.
  - Remote import apply into Flowmint library.
- Modify: `crates/flowmint-core/src/import/mod.rs`
  - Export the `remote` module.
  - Export shared collision helpers used by both local and remote import.
- Modify: `crates/flowmint-core/src/asset/skill.rs`
  - Preserve supported remote Skill text files without path traversal.
  - Keep `SKILL.md`, `metadata.toml`, `examples/`, and `resources/`.
  - Add explicit warnings for ignored folders/files instead of silent drops.
  - Do not add generic script or binary asset support in this MVP.
- Test: `crates/flowmint-core/tests/remote_import_tests.rs`
  - URL-independent scanner/apply tests using in-memory remote file trees.

### Desktop Tauri

- Modify: `apps/desktop/src-tauri/Cargo.toml`
  - Add the minimal HTTPS client dependency for public GitHub GET requests.
  - Use `reqwest` with `json`, `rustls-tls`, and `blocking`, only in the
    desktop crate, so the synchronous command helper trait below is directly
    implementable with `reqwest::blocking::Client`.
- Create: `apps/desktop/src-tauri/src/commands/github_import.rs`
  - URL parsing.
  - GitHub API request helpers.
  - Recursive Contents API traversal with caps.
  - In-memory session and plan cache.
- Modify: `apps/desktop/src-tauri/src/commands/mod.rs`
  - Register the new command module.
- Modify: `apps/desktop/src-tauri/src/lib.rs`
  - Manage GitHub import state and expose commands.
- Test: Tauri command logic should be covered by Rust unit tests where possible
  without real network calls, by separating fetcher trait/mock from command glue.

### Frontend

- Modify: `apps/desktop/src/api/import.ts`
  - Add public GitHub import API types and command wrappers.
- Modify: `apps/desktop/src/pages/ImportPage.tsx`
  - Add source selector: `Local Tool` and `Public GitHub URL`.
  - Keep local import behavior unchanged.
  - Add GitHub URL field, scan button, candidate list, destination ID editor,
    warnings, preview, and apply.
  - Hide `Adopt` for GitHub import.
- Modify: `apps/desktop/src/pages/importPageModel.ts`
  - Add pure model helpers for source selection, URL validation state, candidate
    defaults, collision defaults, and destination ID normalization.
- Modify: `apps/desktop/package.json`
  - Include the new import page model test in the explicit `test:model` compile
    and runner command.
- Modify: `apps/desktop/src/i18n/messages.ts`
  - Add English and Chinese copy for GitHub source, warnings, errors, limits,
    collision rename, and import result.
- Test: `apps/desktop/tests/importPageModel.test.ts`
  - Source selector behavior.
  - GitHub URL validation.
  - Collision default behavior.
  - Destination ID validation.

### Docs

- Modify: `docs/asset-management-and-import.md`
  - Add public GitHub import flow and constraints.
- Modify: `docs/manual-smoke-test.md`
  - Add public GitHub import smoke path.
- Modify: `docs/mvp-remaining-scope.md`
  - Move public GitHub import from missing/deferred into current surface after
    implementation.

## Task List

### GHI-101: Define Remote Import Domain Types

Files:

- Create: `crates/flowmint-core/src/import/remote.rs`
- Modify: `crates/flowmint-core/src/import/mod.rs`
- Test: `crates/flowmint-core/tests/remote_import_tests.rs`

Work:

1. Add `RemoteImportSource`:

   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   #[serde(rename_all = "camelCase")]
   pub struct RemoteImportSource {
       pub provider: RemoteImportProvider,
       pub owner: String,
       pub repo: String,
       pub ref_name: String,
       pub commit_sha: String,
       pub root_path: String,
       pub canonical_url: String,
   }
   ```

2. Add `RemoteImportProvider::PublicGithub`.
3. Add `RemoteFileEntry` with repository-relative path, UTF-8 content, size,
   GitHub blob SHA, and source URL.
4. Add `RemoteImportCandidate` with:
   - `id`
   - `asset_type`
   - `confidence`
   - `source`
   - `source_paths`
   - `default_destination_id`
   - `collision`
   - `warnings`
   - `importable`
5. Add `RemoteImportSelection` with:
   - `candidate_id`
   - `destination_id`
   - `asset_type`
6. Add `RemoteImportPlan` and `RemoteImportApplyResult`.
7. Add `RemoteImportSourceRecord` for persistent provenance sidecars under
   `~/.flowmint/import-sources/<asset-type>/<id>.json`.
8. Reuse existing `AssetType`, `ImportConfidence`, and collision semantics.

Acceptance criteria:

- Core remote types serialize as camelCase for Tauri.
- Remote candidates can represent a multi-file Skill.
- Remote candidates can represent an unimportable item with warnings.
- Local import APIs remain unchanged.

Verification:

- `cargo test -p flowmint-core --test remote_import_tests remote_import_types_serialize`
- `cargo test -p flowmint-core --test import_scanner_tests`

### GHI-102: Implement Remote Candidate Detection

Files:

- Modify: `crates/flowmint-core/src/import/remote.rs`
- Test: `crates/flowmint-core/tests/remote_import_tests.rs`

Detection rules:

1. Skill candidates:
   - High confidence: `.claude/skills/<id>/SKILL.md`,
     `.codex/skills/<id>/SKILL.md`, `.agents/skills/<id>/SKILL.md`.
   - Medium confidence: `skills/<id>/SKILL.md`.
   - Candidate source includes all files under that Skill directory within caps.
2. Prompt candidates:
   - High confidence: `.claude/commands/<id>.md`.
   - High confidence: `.gemini/commands/<id>.toml`.
   - Low confidence: `prompts/<id>.md`.
3. Instruction Rule candidates:
   - High confidence: `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`.
   - High confidence: `.claude/rules/<id>.md`.
   - Medium confidence: `rules/<id>.md`.
4. Command Rule candidates:
   - High confidence: `.codex/rules/<id>.rules` when a command prefix can be
     parsed.
   - Unimportable warning when `.rules` content does not contain a recognizable
     prefix pattern.
5. Playbook candidates:
   - High confidence only for current Flowmint-native playbook markdown that
     starts with the existing HTML metadata header:

     ```md
     <!-- FLOWMINT:PLAYBOOK:BEGIN
     { ... PlaybookAsset JSON metadata ... }
     FLOWMINT:PLAYBOOK:END -->
     ```

   - The detector must parse that JSON metadata using the same field names as
     `PlaybookAsset` and reject invalid metadata as an unimportable candidate
     with a warning.
   - `SKILL.md` files tagged `playbook` remain Skill candidates with a warning
     that they can be promoted after import.

Acceptance criteria:

- Detection is deterministic and sorted by asset type, ID, and path.
- Generic markdown is not over-classified as a Playbook.
- The same GitHub repo content produces stable candidate IDs.
- Candidate warnings explain unsupported folders, binary files, files over cap,
  and low-confidence detection.

Verification:

- `cargo test -p flowmint-core --test remote_import_tests detects_github_skill_directory`
- `cargo test -p flowmint-core --test remote_import_tests detects_agent_instruction_rules`
- `cargo test -p flowmint-core --test remote_import_tests detects_codex_command_rules`
- `cargo test -p flowmint-core --test remote_import_tests only_detects_explicit_playbooks`

### GHI-103: Implement Remote Import Preview And Apply

Files:

- Modify: `crates/flowmint-core/src/import/remote.rs`
- Modify: `crates/flowmint-core/src/asset/skill.rs`
- Test: `crates/flowmint-core/tests/remote_import_tests.rs`

Work:

1. Add `preview_remote_import(library_home, source, files, selections)`.
2. Validate every `destination_id` with the existing asset ID rules.
3. Reject selected candidates with unresolved collisions unless the user changed
   the destination ID.
4. Reject duplicate selected `(asset_type, destination_id)` pairs inside the
   same preview before any apply operation can run.
5. Convert remote Prompt candidates into `PromptAsset`.
6. Convert remote Skill candidates into `SkillAsset`.
7. Convert remote Instruction Rule candidates into `RuleAsset`.
8. Convert remote Command Rule candidates into `RuleAsset`.
9. Convert explicit Flowmint Playbook markdown into `PlaybookAsset`.
10. Add `apply_remote_import(library_home, plan)` that writes only into the
   Flowmint library.
11. Store source metadata for every imported asset:
    - Add tags `source-github` and `github-<owner>-<repo>`.
    - Write a provenance sidecar at
      `~/.flowmint/import-sources/<asset-type>/<destination-id>.json` containing
      provider, owner, repo, ref name, commit SHA, canonical URL, source paths,
      imported asset type, and destination ID.
    - Do not change Prompt, Rule, or Playbook renderer formats just to store
      GitHub provenance.

Acceptance criteria:

- Apply never writes to project paths, `.codex`, `.claude`, `.gemini`, or global
  agent directories.
- Remote import has no `Adopt` mode and writes no local sync lockfile.
- Skill directory import preserves `SKILL.md`, `metadata.toml`, `examples/`,
  and `resources/`.
- Unsupported Skill files are shown in warnings before apply.
- Duplicate selected destination IDs are preview conflicts and apply cannot
  partially write either duplicate.
- Each applied asset has a provenance sidecar with the concrete GitHub commit
  SHA used for the scan.
- Existing local import tests still pass.

Verification:

- `cargo test -p flowmint-core --test remote_import_tests remote_import_apply_writes_library_only`
- `cargo test -p flowmint-core --test remote_import_tests remote_skill_import_preserves_supported_files`
- `cargo test -p flowmint-core --test remote_import_tests collision_requires_new_destination_id`
- `cargo test -p flowmint-core --test remote_import_tests duplicate_destination_ids_block_preview`
- `cargo test -p flowmint-core --test remote_import_tests remote_import_writes_github_provenance_sidecar`
- `cargo test -p flowmint-core --test import_adoption_tests`

### GHI-104: Add Public GitHub Fetcher In Tauri

Files:

- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Create: `apps/desktop/src-tauri/src/commands/github_import.rs`
- Modify: `apps/desktop/src-tauri/src/commands/mod.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Test: unit tests inside `github_import.rs`

Work:

1. Add a small fetcher abstraction:

   ```rust
   trait GithubHttpClient {
       fn get_json(&self, url: &str) -> Result<serde_json::Value, String>;
       fn get_text(&self, url: &str, max_bytes: usize) -> Result<String, String>;
   }
   ```

2. Implement it with `reqwest::blocking::Client` for production. Do not use
   async `reqwest::Client` unless this plan is also changed to make the command
   helpers async and to add async test runtime coverage.
3. Add URL parser support for:
   - `https://github.com/<owner>/<repo>`
   - `https://github.com/<owner>/<repo>/tree/<ref>/<path>`
   - `https://github.com/<owner>/<repo>/blob/<ref>/<path>`
   - optional query override: `?ref=<branch-or-tag>&path=<path>`
4. For ambiguous branch names with slashes, prefer explicit `?ref=` and expose a
   user-facing error that tells the user to supply `ref`.
5. Resolve default branch when no ref is provided.
6. Resolve commit SHA with `GET /repos/{owner}/{repo}/commits/{ref}`.
7. Traverse Contents API directories recursively within caps.
8. Download raw UTF-8 file content through `download_url`.
9. Reject non-GitHub hosts and non-HTTPS URLs.
10. Return clear fatal errors for root-level failures:
    - 403 unauthenticated rate limit.
    - 404 private or missing repository/path.
    - invalid GitHub URL.
    - unavailable GitHub API response for the selected root path.
11. Treat per-file traversal problems as scan warnings, not fatal command
    errors, when at least one directory listing succeeds:
    - file too large.
    - binary or non-UTF-8 content.
    - unsupported file type.
    - traversal caps reached.
    - individual child path 404 after a parent directory was listed.

Acceptance criteria:

- Public repo URL scan works without tokens.
- No arbitrary host can be fetched through the command.
- Network fetching is confined to GitHub API/raw content hosts.
- Fetcher tests use a mock client, not live GitHub.
- Runtime error messages are actionable in Chinese and English through the UI.
- A repo containing large images, binary assets, or unrelated generated files
  can still return importable candidates plus warnings.

Verification:

- `cargo test -p flowmint-desktop github_import`
- `cargo test -p flowmint-core --test remote_import_tests`

### GHI-105: Add Tauri Commands And State Cache

Files:

- Modify: `apps/desktop/src-tauri/src/commands/github_import.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/commands/mod.rs`
- Test: unit tests inside `github_import.rs`

Commands:

1. `scan_public_github_import(url: String) -> PublicGithubImportScanResult`
   - Fetches the remote file tree.
   - Creates a session ID.
   - Stores the fetched source/files in memory.
   - Returns candidates and warnings.
2. `preview_public_github_import(session_id, selections) -> RemoteImportPlan`
   - Builds a core plan from the cached session.
   - Stores the plan by `plan_id`.
3. `apply_public_github_import(plan_id) -> RemoteImportApplyResult`
   - Removes the cached plan.
   - Writes selected assets into the Flowmint library.

Cache lifecycle:

1. Store at most 5 scan sessions and 10 preview plans.
2. Expire both scan sessions and preview plans after 30 minutes.
3. Evict expired entries before every scan, preview, and apply command.
4. If a new scan exceeds the session cap, evict the oldest scan session.
5. After `preview_public_github_import` succeeds, remove the scan session and
   keep only the bounded selected plan content needed by apply.
6. After `apply_public_github_import` succeeds or fails, remove that plan ID.

Acceptance criteria:

- A scan session cannot be applied directly; preview is required.
- Applying an unknown or expired plan ID returns a clear error.
- Cached file content is bounded by the caps from this plan.
- Local import plan cache remains independent.
- Repeated scans cannot grow memory without bound because session and plan
  caches have count caps and TTL eviction.

Verification:

- `cargo test -p flowmint-desktop github_import_state`
- `npm run check`

### GHI-106: Add Frontend API Types

Files:

- Modify: `apps/desktop/src/api/import.ts`
- Test: TypeScript compile through `npm run frontend:model-test`

Work:

1. Add API wrappers:

   ```ts
   export function scanPublicGithubImport(url: string): Promise<PublicGithubImportScanResult>;
   export function previewPublicGithubImport(
     sessionId: string,
     selections: RemoteImportSelection[],
   ): Promise<RemoteImportPlan>;
   export function applyPublicGithubImport(planId: string): Promise<RemoteImportApplyResult>;
   ```

2. Add types that mirror Tauri camelCase payloads.
3. Keep existing local import functions unchanged.

Acceptance criteria:

- Existing local import UI compiles unchanged.
- Remote import API types include warnings, collisions, importability, and
  destination ID.

Verification:

- `npm run frontend:model-test`
- `npm run check`

### GHI-107: Add Import Page GitHub Source UI

Files:

- Modify: `apps/desktop/src/pages/ImportPage.tsx`
- Modify: `apps/desktop/src/pages/importPageModel.ts`
- Modify: `apps/desktop/package.json`
  - Add `tests/importPageModel.test.ts` to the explicit `test:model` TypeScript
    compile list and add
    `node ../../target/frontend-model-tests/tests/importPageModel.test.js` to
    the explicit runner list.
- Test: `apps/desktop/tests/importPageModel.test.ts`

Work:

1. Add source selector:
   - `Local Tool`
   - `Public GitHub URL`
2. Preserve the current local import layout when `Local Tool` is selected.
3. For GitHub source, show:
   - URL input.
   - optional project path selector for "import and attach to current project".
   - Scan button.
   - Repository/ref/path summary after scan.
   - Candidate list with asset type, confidence, source paths, warnings, and
     editable destination ID.
   - Importability state.
   - Preview button.
   - Apply button.
4. Hide `Copy/Adopt/Skip` mode selector for GitHub and replace it with
   `Import/Skip`.
5. Default collided candidates to `Skip` until the user enters a non-colliding
   destination ID.
6. Update `apps/desktop/package.json` so `npm run frontend:model-test` compiles
   and executes `apps/desktop/tests/importPageModel.test.ts`.
7. If the user selects "import and attach to current project", update the chosen
   project manifest after the library import succeeds:
   - Skill assets go into the chosen target profile `skills`.
   - Playbook assets go into `playbooks`.
   - Prompt assets go into `prompts` only for targets that support Prompt
     export.
   - Instruction Rule assets go into `instruction_rules`.
   - Command Rule assets go into `command_rules` only for Codex profiles.
8. After apply, show imported asset refs and a next-step message:
   - attach to a project profile, or
   - attach to a global user profile, then run sync preview.

Acceptance criteria:

- Local import behavior is unchanged.
- GitHub import cannot expose `Adopt`.
- User cannot preview with zero selected candidates.
- User cannot apply a plan with collisions or invalid destination IDs.
- Optional current-project attachment writes only `<project>/.flowmint.toml`;
  GitHub provenance remains library-level under `~/.flowmint/import-sources`.
- Warnings are visible before apply.
- UI text is available in English and Chinese.

Verification:

- `npm run frontend:model-test`
- `npm run check`
- Manual desktop check: GitHub source tab can scan a known public repo fixture
  URL and preview selected candidates.

### GHI-108: Add I18n Copy

Files:

- Modify: `apps/desktop/src/i18n/messages.ts`
- Test: `apps/desktop/tests/i18n.test.ts`

Required keys:

- `import.source`
- `import.sourceLocal`
- `import.sourceGithub`
- `import.githubUrl`
- `import.githubScan`
- `import.githubPublicOnly`
- `import.githubRepoSummary`
- `import.githubWarnings`
- `import.githubDestinationId`
- `import.githubImport`
- `import.githubSkip`
- `import.githubApplyMessage`
- `import.githubPrivateOrMissing`
- `import.githubRateLimited`
- `import.githubCapsReached`
- `import.githubUnsupportedFile`
- `import.githubSessionExpired`
- `import.githubNextStep`

Acceptance criteria:

- English and Chinese key sets remain identical.
- No raw internal error strings are required for common GitHub failures.

Verification:

- `npm run frontend:model-test`

### GHI-109: Update Documentation And Smoke Checklist

Files:

- Modify: `docs/asset-management-and-import.md`
- Modify: `docs/manual-smoke-test.md`
- Modify: `docs/mvp-remaining-scope.md`

Work:

1. Document public GitHub import as an import source.
2. Explicitly document that private repos are not supported in this version.
3. Document that GitHub import writes only to the Flowmint library.
4. Document that users sync imported assets using the existing project/global
   target profiles.
5. Document why GitHub provenance is stored with the Flowmint library asset and
   why the project directory only stores attachment and lock state.
6. Add a manual smoke case:
   - scan public GitHub URL,
   - import Skill,
   - import Instruction Rule,
   - choose import-and-attach for the current project,
   - handle an ID collision by renaming,
   - sync imported Skill to Codex project scope,
   - verify output file exists under `.codex/skills/<id>/SKILL.md`.

Acceptance criteria:

- Docs do not imply private repo support.
- Docs do not imply direct GitHub-to-agent writes.
- Docs distinguish library-level GitHub provenance from project-level
  attachment state.
- Smoke checklist verifies import plus downstream sync.

Verification:

- `rg -n "private|GitHub|github|Adopt|direct" docs/asset-management-and-import.md docs/manual-smoke-test.md docs/mvp-remaining-scope.md`

### GHI-110: Release Verification

Files:

- No new source files unless verification finds a defect.

Required checks:

1. Core tests:

   ```bash
   cargo test -p flowmint-core
   ```

2. Desktop command tests:

   ```bash
   cargo test -p flowmint-desktop
   ```

3. Frontend model/i18n tests:

   ```bash
   npm run frontend:model-test
   ```

4. Full repo check:

   ```bash
   npm run check
   ```

5. Desktop build:

   ```bash
   npm run desktop:build
   ```

6. Diff hygiene:

   ```bash
   git diff --check
   ```

Manual smoke:

1. Start the desktop app.
2. Open Import.
3. Select `Public GitHub URL`.
4. Scan a public repo/subpath containing a `SKILL.md`.
5. Import at least one Skill and one Instruction Rule.
6. Select import-and-attach for the current project.
7. Rename one collided asset before preview.
8. Confirm imported assets appear in Assets.
9. Confirm `<project>/.flowmint.toml` references the imported Skill.
10. Preview and apply project sync.
11. Verify `.codex/skills/<id>/SKILL.md` exists in the selected project.

Acceptance criteria:

- All automated checks pass.
- Manual smoke confirms remote import plus downstream sync.
- No private repo/token UI appears.
- No remote-import path writes outside `~/.flowmint`.

## Deferred Explicitly

These are not part of this plan:

1. Private GitHub repository import.
2. GitHub token storage.
3. GitHub OAuth/login.
4. Marketplace search/ranking.
5. Auto-update from upstream GitHub sources.
6. Direct GitHub-to-agent sync.
7. Running or validating imported scripts.
8. Binary asset preservation.
9. Cursor/Windsurf rule formats.
10. Remote import from GitLab, Gitea, or arbitrary ZIP URLs.

## Risk Review

| Risk | Impact | Mitigation |
| --- | --- | --- |
| GitHub branch changes after scan | User imports unexpected content | Resolve and record commit SHA during scan; apply cached plan content. |
| Large repo exhausts rate limit or memory | Slow app or failed scan | Enforce directory, file, byte, and depth caps. |
| Malicious Skill scripts | User syncs unsafe code into agent config | Never execute scripts; warn about unsupported/script files before import. |
| Silent Skill file loss | Imported Skill does not work | Show warnings for unsupported files; preserve supported text files. |
| ID collisions block useful import | User cannot import popular assets | Allow destination ID rename before preview. |
| GitHub private repo looks like missing repo | Confusing UX | Map 404/403 into "public-only, missing, or private" error copy. |
| URL parsing ambiguity for refs with slashes | Wrong branch/path split | Prefer explicit `?ref=` override and show actionable error. |
| Direct import bypasses sync safety | Agent config overwritten unexpectedly | Remote apply writes only to Flowmint library. Existing sync preview/apply remains mandatory. |
| Scan session cache grows without bound | Desktop process memory grows after repeated scans | Cap scan sessions and plans, evict expired entries on every command, and remove sessions after preview. |
| Provenance stored in project directory | Same imported asset has divergent source records across projects | Store GitHub provenance once with the library asset; project manifests store only attachments. |

## Self-Review

Spec coverage:

- Public GitHub URL import is covered by GHI-104 through GHI-107.
- Flowmint-library-first behavior is covered by GHI-103 and GHI-109.
- Library-level provenance vs current-project attachment is covered by Storage
  semantics, GHI-107, GHI-109, and Risk Review.
- No private repo support is stated in Product Rules, Deferred Explicitly, and
  docs tasks.
- Skill, Prompt, Rule, and Playbook detection are covered by GHI-102.
- Collision handling is covered by GHI-103 and GHI-107.
- Safety caps and untrusted content warnings are covered by GHI-102, GHI-104,
  GHI-107, and Risk Review.
- Frontend test runner wiring is covered by the File Map and GHI-107.
- Public GitHub HTTP client implementation details are covered by GHI-104 with
  `reqwest::blocking::Client`.
- Remote file skip warning semantics are covered by GHI-104.
- Session cache bounds and eviction are covered by GHI-105 and Risk Review.
- Duplicate selected destination IDs are covered by GHI-103.
- Current Flowmint Playbook metadata detection is covered by GHI-102.
- Existing local import preservation is covered by GHI-101, GHI-106, and
  GHI-107.
- Verification and smoke testing are covered by GHI-110.

Placeholder scan:

- No unresolved placeholder markers remain.
- Deferred items are explicit non-goals, not hidden implementation gaps.

Type consistency:

- Remote import type names are consistently `RemoteImport*` in core and
  `PublicGithubImport*` for Tauri/UI command payloads.
- Existing local import types remain unchanged.

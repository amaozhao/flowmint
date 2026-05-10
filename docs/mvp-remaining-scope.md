# Flowmint Remaining Scope

This file separates currently implemented local-first functionality from items
that are intentionally deferred.

## Current Implemented Surface

- Desktop-first Tauri app is the main product surface.
- Rust Core owns asset storage, validation, project manifests, sync planning,
  import adoption, lockfiles, and apply behavior.
- Prompt, Skill, Playbook, Instruction Rule, and Command Rule assets can be
  created and edited in the UI.
- Overview shows counts plus charts for asset mix, project readiness, and
  agent target support.
- Existing Skills tagged `playbook` can be promoted to first-class Playbook
  assets from the UI.
- Projects can attach assets per target profile for project scope.
- Settings can manage global user sync profiles per target.
- Project and library setup support a native directory picker when the host OS
  exposes one, with manual path input retained as a fallback.
- Sync Preview supports Claude Code, Codex, and Gemini CLI target selection,
  plus Project and Global User scopes.
- Global sync apply requires a second confirmation and exact backend
  acknowledgement of mutating paths; the UI shows the global root and concrete
  paths before apply.
- Import supports read-only scan plus Copy / Adopt / Skip preview/apply.
- Import supports public GitHub repository, tree, and blob URLs for Prompt,
  Skill, Playbook, Instruction Rule, and Codex Command Rule candidates.
- GitHub imports write selected assets into the local library, store source
  provenance under `import-sources/`, and can attach imported assets to project
  or global target profiles.
- Skill examples/resources can be edited as supporting text files and are
  exported recursively.
- English and Chinese UI switching works locally.

## Remaining Product/Release Tasks

1. Run the full manual smoke path from `docs/manual-smoke-test.md` in the
   desktop app.
2. Run the GitHub release workflow and smoke-test macOS and Windows artifacts
   before calling those platforms supported.
3. Design backend-owned conflict semantics before adding advanced actions such
   as `Mark as unmanaged` or `Force overwrite`.

## Intentional Target Limitations

1. Codex Prompt command export remains blocked until explicit Prompt-as-Skill
   conversion exists.
2. Gemini Skill/Playbook export remains blocked until local Gemini CLI support
   is validated.
3. Claude Code and Gemini Command Rule export are deferred.
4. Cursor exporter is not implemented yet.

## Deferred Product Lines

1. `flowmint-cli` as a first-class user entry.
2. Search index with Tantivy or SQLite.
3. Git-based bidirectional sync, private repository import, or registry.
4. Cloud sync, accounts, team permissions, marketplace.
5. AI chat, LLM API calls, agent runtime, and eval system.
6. VS Code extension.

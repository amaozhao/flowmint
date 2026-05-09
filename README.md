# Flowmint

Flowmint is a local-first desktop manager for reusable AI workflow assets. The MVP focuses on creating Prompt and Skill assets, binding them to local projects, previewing a Claude Code sync plan, and applying that sync safely.

## Current MVP Scope

- Desktop app: Tauri 2 + React/Vite/TypeScript.
- Core engine: Rust workspace crate `flowmint-core`.
- Local storage only; no accounts, cloud sync, marketplace, or LLM calls.
- Asset types: Prompt and Skill. Playbook remains a Skill template shape for the MVP.
- Export target: Claude Code.

## Storage

Flowmint keeps source data in the local filesystem:

- Global library: `~/.flowmint`
- Prompts: `~/.flowmint/prompts/<id>.md`
- Skills: `~/.flowmint/skills/<id>/`
- Project manifest: `<project>/.flowmint.toml`
- Sync lockfile: `<project>/.flowmint.lock`

Claude Code sync writes only after preview:

- Prompt commands: `<project>/.claude/commands/<prompt-id>.md`
- Skills: `<project>/.claude/skills/<skill-id>/`
- Managed block: `<project>/CLAUDE.md`

## Safety Model

The UI does not write workflow files directly. It calls Tauri commands, which delegate to Rust Core.

Sync is planned before apply. Apply uses a backend-cached `plan_id`, rebuilds the plan, and refuses to write if conflicts appear after preview. The MVP blocks unmanaged target files, modified generated files, broken `CLAUDE.md` managed markers, unsafe symlinks, and unsafe asset IDs.

## Development

Install dependencies:

```bash
npm ci
```

Run all checks:

```bash
npm run check
```

Start the desktop app:

```bash
npm run desktop:dev
```

Build Linux desktop bundles:

```bash
npm run desktop:build
```

The local Linux build currently produces:

- `target/release/flowmint-desktop`
- `target/release/bundle/deb/*.deb`
- `target/release/bundle/rpm/*.rpm`

AppImage is not part of the default local bundle target because it failed in the current environment with a read-only filesystem error.

## MVP Usage Loop

1. Launch the desktop app.
2. Create the local Flowmint library.
3. Create a Prompt.
4. Create a Skill.
5. Create a Playbook Skill when a structured workflow template is useful.
6. Add a local project path.
7. Attach the Prompt and Skills to the project.
8. Preview Claude Code sync.
9. Apply sync only when the preview has no conflicts.
10. Verify generated Prompt command, Skill files, `CLAUDE.md`, and `.flowmint.lock`.
11. Preview again and confirm unchanged files are noops.

The manual checklist is in `docs/manual-smoke-test.md`.
The MVP implementation audit is in `docs/mvp-implementation-audit.md`.
The explicit remaining scope is tracked in `docs/mvp-remaining-scope.md`.
Asset management and programming-tool import behavior is documented in `docs/asset-management-and-import.md`.

## Packaging Notes

Linux deb/rpm bundling is the only packaging path smoke-tested locally so far.

Unsmoked platform command paths:

```bash
npm run desktop:build:mac
npm run desktop:build:windows
```

The release workflow builds Linux, Windows, macOS Apple Silicon, and macOS
Intel artifacts on native GitHub Actions runners. Pushes to `main` generate
downloadable workflow artifacts without requiring a tag. On tag pushes such as
`v0.1.0`, or manual runs with a tag input, it also creates or updates a draft
GitHub Release and uploads the `.deb`, `.rpm`, `.dmg`, and `.msi` assets. Manual
runs without a tag only upload workflow artifacts. Do not treat macOS or Windows
packages as supported until the workflow artifacts have been installed and
smoke-tested on those platforms.

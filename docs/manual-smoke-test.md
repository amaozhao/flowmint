# Flowmint Manual Smoke Test

Run this checklist before tagging an MVP build.

## Preconditions

- Linux desktop dependencies from `docs/development.md` are installed.
- `npm run check` passes from the repository root.
- Start the desktop app with `npm run desktop:dev`.

## Smoke Path

1. Launch the desktop app.
2. Create the local Flowmint library when onboarding appears. Use either the default path or a temporary custom path.
3. Confirm Overview shows real Prompt, Skill, Playbook, Project, and Pending Sync counts plus dashboard charts for asset mix, project readiness, and agent target support.
4. Switch the UI language between English and Chinese from the top bar and confirm the selection persists after refresh/relaunch.
5. Open Assets.
6. Create a Prompt with a safe ID such as `daily-plan`, including one variable.
7. Create a Skill with a safe ID such as `research-helper`, add one example file and one resource file, save, reopen, and confirm both files are still editable.
8. Create a first-class Playbook with a safe ID such as `release-check`.
9. Create an Instruction Rule with a safe ID such as `typescript-style`.
10. Create a Codex Command Rule with a safe ID such as `safe-git-status`, prefix `git, status`, and decision `prompt`.
11. Confirm search, type filters, and tag filter work for all asset types.
12. Reopen a saved Skill and click Open Folder.
13. Open Projects.
14. Add a temporary local project directory using Browse when available, or by pasting the path manually.
15. Attach the Prompt, Skill, Playbook, Instruction Rule, and Command Rule to the project using multi-select.
16. From the project detail, click Preview Sync.
17. Run Preview Sync for Claude Code project scope.
18. Confirm the preview includes:
    - `.claude/commands/daily-plan.md`
    - `.claude/skills/research-helper/SKILL.md`
    - `.claude/skills/release-check/SKILL.md`
    - `.claude/rules/typescript-style.md`
    - `CLAUDE.md`
19. Confirm the Claude Code Command Rule attachment is reported as an unsupported mapping conflict.
20. Detach the Command Rule or switch to a target that supports it before applying.
21. Confirm Apply Sync is enabled only when the preview has zero conflicts.
22. Apply sync.
23. Inspect the project directory and verify:
    - `.claude/commands/daily-plan.md` exists.
    - `.claude/skills/research-helper/SKILL.md` exists.
    - `.claude/skills/release-check/SKILL.md` exists.
    - `.claude/rules/typescript-style.md` exists.
    - `CLAUDE.md` contains a `FLOWMINT` managed block.
    - `.flowmint.lock` exists.
24. Preview sync again and verify unchanged generated files are shown as noops.
25. Edit `.claude/commands/daily-plan.md` manually.
26. Preview sync again and verify the modified generated file is shown as a conflict.
27. Run Preview Sync for Codex project scope and verify:
    - `.codex/skills/research-helper/SKILL.md`
    - `.codex/skills/release-check/SKILL.md`
    - `AGENTS.md`
    - `.codex/rules/safe-git-status.rules`
28. Run Preview Sync for Gemini CLI project scope and verify:
    - `.gemini/commands/daily-plan.toml`
    - `GEMINI.md`
    - Skill, Playbook, and Command Rule attachments are blocked as unsupported or requires-validation conflicts.
29. Open Settings, choose a Global Profile target, attach one Prompt or Skill, and confirm it appears in that global target profile.
30. Run Preview Sync for Global User scope and verify the global root and mutating paths are visible and a second confirmation is required before Apply Sync.
31. Open Import.
32. Browse or paste a project path for Project scope; choose Global User scope without a project path and confirm scan is still allowed.
33. Choose target and scope, scan candidates, and verify the scan is read-only for Claude Code `CLAUDE.md` / `.claude/*` and Codex `AGENTS.md` / `.codex/skills` / `.codex/rules`.
34. Choose Copy for one candidate and Adopt for one candidate, preview the import plan, then apply only if there are no conflicts.
35. Switch Import source to Public GitHub URL.
36. Paste a small public GitHub repository, tree, or blob URL that contains at least one supported Prompt, Skill, Playbook, or Rule path.
37. Scan the URL and confirm skipped remote files appear as warnings, not fatal errors.
38. Choose Import for at least one candidate, edit the destination ID, preview, and confirm existing-library collisions or duplicate selected destination IDs block apply.
39. Apply the GitHub import and verify the asset exists in the local library.
40. If Attach after import is enabled, verify the asset appears in the selected project profile or global profile before running Sync.
41. Open Settings, run Rebuild Index, and export the debug report.

## Pass Criteria

- Prompt export, Skill export, Playbook-as-Skill export, Instruction Rule export, target lockfiles, and managed blocks are produced.
- Codex Command Rule export works through `.codex/rules`.
- Unsupported mappings are blocked in preview and cannot be applied.
- Global writes require explicit second confirmation.
- Global write confirmation shows the root directory and concrete mutating paths.
- Skill `metadata.toml`, `examples/`, and `resources/` are preserved and exported recursively.
- Global target profiles can be managed from Settings.
- Import scan is read-only and adoption writes lock records only after apply.
- Public GitHub URL import writes selected remote assets into the local library and records provenance under `import-sources/`.
- GitHub-imported assets can be attached to project or global profiles before sync writes target agent files.
- Repeated sync is idempotent.
- Manual edits to generated files are not overwritten silently.
- No Force Overwrite or Mark as unmanaged action is exposed in the MVP UI.
- English and Chinese UI switching works and persists locally.

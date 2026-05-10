# Asset Management And Tool Import

Flowmint is the local source of truth for reusable AI coding assets. Claude
Code, Codex, and Gemini CLI receive generated or adopted copies only after the
user previews and applies a plan.

## Asset Types

### Prompt

- Stored at `~/.flowmint/prompts/<prompt-id>.md`.
- Editable fields: ID, name, description, tags, variables, and body.
- Exports to Claude Code commands and Gemini CLI custom command TOML.
- Codex prompt-command export is intentionally unsupported until an explicit
  Prompt-as-Skill conversion exists.

### Skill

- Stored at `~/.flowmint/skills/<skill-id>/`.
- Core file: `SKILL.md`.
- Supported copied content: `metadata.toml`, `examples/`, and `resources/`.
- Exports to Claude Code `.claude/skills/<id>/` and Codex `.codex/skills/<id>/`.

### Playbook

- Stored at `~/.flowmint/playbooks/<playbook-id>.md`.
- First-class fields: trigger, inputs, steps, verification, failure handling,
  side-effect level, recommended invocation, and target compatibility.
- Exports as a target-native Skill for Claude Code and Codex.
- Existing v0.1 Skills tagged `playbook` remain valid Skill assets.
- Existing Playbook Skills can be promoted to first-class Playbooks from the
  desktop editor.

### Instruction Rule

- Stored at `~/.flowmint/rules/<rule-id>.md`.
- Represents persistent agent instructions such as code style or test policy.
- Exports to Claude Code rule markdown, Codex `AGENTS.md` managed blocks, and
  Gemini `GEMINI.md` managed blocks.

### Command Rule

- Stored at `~/.flowmint/rules/<rule-id>.md`.
- Represents command execution policy with prefix and decision:
  `prompt`, `allow`, or `forbid`.
- Currently exports to Codex `.codex/rules/<id>.rules` only.
- It is separate from Instruction Rules because Codex `.rules` files control
  command execution rather than general agent context.

## Project Binding

Each project has a Flowmint manifest at `<project>/.flowmint.toml`.

Legacy-compatible manifests still render as:

```toml
[project]
name = "my-project"

[export]
target = "claude-code"

[attach]
prompts = ["daily-plan"]
skills = ["research-helper"]
```

When Playbooks, Rules, multiple target profiles, or non-default scopes are used,
Flowmint renders v2 profiles:

```toml
[[exports]]
target = "codex"
scope = "project"
prompts = []
skills = ["research-helper"]
playbooks = ["release-check"]
instruction_rules = ["typescript-style"]
command_rules = ["safe-git-status"]
```

The Projects page can attach Prompt, Skill, Playbook, Instruction Rule, and
Command Rule assets per project target profile. The Settings page manages
Global User target profiles. Sync preview remains the source of truth for
whether a selected target supports a specific mapping.

## Scope Selection

Every sync and import flow exposes an explicit scope choice:

- `Project`: read/write under the selected project.
- `Global User`: read/write under the user-level tool configuration root.

Global sync apply requires a second confirmation and backend acknowledgement of
the exact mutating paths.

## Local Tool Import Flow

The Import page scans existing tool files without writing:

1. Choose target and scope. For Project scope, paste or browse for a project path.
2. Scan candidates.
3. Choose `Copy`, `Adopt`, or `Skip` per candidate.
4. Preview the adoption plan.
5. Apply only if there are no conflicts.

Collision information is shown before preview. Adopt mode writes Flowmint lock
records only after apply and fails if the source file changed after preview.

Scan coverage:

- Claude Code: `.claude/commands`, `.claude/skills`, `.claude/rules`,
  project `CLAUDE.md`, and global `~/.claude/CLAUDE.md`.
- Codex: `.codex/skills`, legacy `.agents/skills`, `AGENTS.md`,
  `~/.codex/AGENTS.md`, and `.codex/rules`.
- Gemini CLI: `.gemini/commands`.

## Public GitHub URL Import Flow

The Import page can also load reusable assets from a public GitHub URL:

1. Choose `Public GitHub URL` as the source.
2. Paste a public GitHub repository, tree, or blob URL.
3. Choose the target and scope only for the optional post-import attachment.
4. Scan the remote path. Remote traversal is read-only and skipped files are
   returned as warnings instead of blocking the whole scan.
5. Choose `Import` or `Skip` per candidate and edit the destination ID when
   needed.
6. Preview. Existing library collisions and duplicate selected destination IDs
   block apply.
7. Apply. Flowmint writes the selected assets into the local library.
8. If `Attach after import` is enabled, Flowmint also attaches the imported
   assets to either the selected project profile or the selected global profile.

Remote GitHub imports are not written into the project directory directly.
Imported assets belong to the Flowmint local library first. Project-level use is
represented by `<project>/.flowmint.toml`; global use is represented by
`~/.flowmint/global-sync-profiles.toml`.

GitHub source provenance is stored beside the library asset under
`~/.flowmint/import-sources/<asset-type>/<asset-id>.json`. This records the
provider, repository, ref, commit SHA, canonical URL, and source paths so the
asset can later be audited without coupling that metadata to one project.

GitHub scan coverage:

- Claude Code: `.claude/commands`, `.claude/skills`, `.claude/rules`, and
  `CLAUDE.md`.
- Codex: `.codex/skills`, legacy `.agents/skills`, `AGENTS.md`, and
  `.codex/rules`.
- Gemini CLI: `.gemini/commands`.
- Flowmint: first-class Playbook markdown containing the
  `FLOWMINT:PLAYBOOK` metadata header.

---

# 资产管理与工具导入

Flowmint 是本地 AI 编程资产的源头。Claude Code、Codex、Gemini CLI 只会在用户预览并应用计划后收到生成文件或被接管的文件。

## 资产类型

- Prompt：存储在 `~/.flowmint/prompts/<prompt-id>.md`，可导出到 Claude Code commands 和 Gemini CLI custom commands；Codex Prompt command 暂不支持。
- Skill：存储在 `~/.flowmint/skills/<skill-id>/`，核心文件是 `SKILL.md`；桌面编辑器可管理 `metadata.toml`、`examples/`、`resources/` 文本文件，同步时会递归复制这些 supported files。
- Playbook：存储在 `~/.flowmint/playbooks/<playbook-id>.md`，作为一等资产管理，导出时渲染为目标工具的 Skill；旧的 Playbook Skill 可以在桌面编辑器中提升为一等 Playbook。
- Instruction Rule：存储在 `~/.flowmint/rules/<rule-id>.md`，用于代码风格、测试要求、项目约定等长期指令。
- Command Rule：存储在 `~/.flowmint/rules/<rule-id>.md`，用于命令执行策略，目前仅导出到 Codex `.codex/rules/<id>.rules`。

## 项目绑定

项目通过 `<project>/.flowmint.toml` 保存绑定关系。旧版 Prompt/Skill 会保持兼容格式；当使用 Playbook、Rule、多目标或非默认范围时，会写入 `[[exports]]` v2 profile。

Projects 页面可以按项目目标配置绑定 Prompt、Skill、Playbook、Instruction Rule 和 Command Rule。Settings 页面管理全局用户目标配置。具体目标是否支持该资产，由 Sync Preview 的冲突结果决定。

## 范围选择

同步和导入都必须选择范围：

- `Project`：读取或写入当前项目。
- `Global User`：读取或写入用户级工具配置。

全局同步应用前必须二次确认。UI 会显示全局根目录和实际变更路径，后端还会校验确认的路径和缓存计划完全一致。

## 本地工具导入流程

Import 页面扫描已有工具文件时是只读的：

1. 选择目标工具和范围。项目范围可粘贴路径或通过 Browse 选择路径。
2. 扫描候选项。
3. 对每个候选项选择复制、接管或跳过。
4. 预览导入计划。
5. 无冲突时应用。

接管模式只会在应用后写入 Flowmint lock 记录；如果预览后源文件发生变化，应用会失败。

扫描覆盖：

- Claude Code：`.claude/commands`、`.claude/skills`、`.claude/rules`、项目 `CLAUDE.md`、全局 `~/.claude/CLAUDE.md`。
- Codex：`.codex/skills`、旧版 `.agents/skills`、`AGENTS.md`、`~/.codex/AGENTS.md`、`.codex/rules`。
- Gemini CLI：`.gemini/commands`。

## 公开 GitHub URL 导入流程

Import 页面也可以从公开 GitHub URL 加载资产：

1. 来源选择 `公开 GitHub URL`。
2. 粘贴公开 GitHub 仓库、tree 或 blob URL。
3. 目标工具和范围只用于“导入后绑定”。
4. 扫描远程路径。远程扫描是只读的；二进制、超大或不支持的文件会作为 warning 返回，不会让整个扫描失败。
5. 对每个候选项选择导入或跳过，必要时修改目标 ID。
6. 预览。已有本地库冲突和本次选择里的重复目标 ID 都会阻止应用。
7. 应用。Flowmint 会把选中的资产写入本地库。
8. 如果开启“导入后绑定”，Flowmint 会把导入的资产绑定到所选项目 profile 或全局 profile。

GitHub 远程导入不会直接把资产写进项目目录。导入后的资产先属于
Flowmint 本地库；项目级使用通过 `<project>/.flowmint.toml` 表示，全局使用通过 `~/.flowmint/global-sync-profiles.toml` 表示。

GitHub 来源记录保存在本地库资产旁边：
`~/.flowmint/import-sources/<asset-type>/<asset-id>.json`。这里记录 provider、仓库、ref、commit SHA、canonical URL 和源文件路径，方便以后审计这个资产来自哪里，同时不会把这份来源元数据绑定到某一个项目。

GitHub 扫描覆盖：

- Claude Code：`.claude/commands`、`.claude/skills`、`.claude/rules`、`CLAUDE.md`。
- Codex：`.codex/skills`、旧版 `.agents/skills`、`AGENTS.md`、`.codex/rules`。
- Gemini CLI：`.gemini/commands`。
- Flowmint：带 `FLOWMINT:PLAYBOOK` 元数据头的一等 Playbook markdown。

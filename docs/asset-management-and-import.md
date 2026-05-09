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

## Import Flow

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

## 导入流程

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

# Flowmint UI-first MVP 需求与开发规划文档

> 文档目标：把 Flowmint 从「CLI-first 工具」调整为「带桌面界面的本地 AI 工作流资产管理工具」，明确 MVP 范围、界面结构、核心用户流程、数据存储、技术架构、开发里程碑和验收标准。

---

## 0. 文档信息

| 字段 | 内容 |
|---|---|
| 项目名称 | Flowmint |
| 文档类型 | UI-first MVP 需求与开发规划 |
| 文档版本 | v0.2-draft |
| 推荐产品形态 | 桌面 App 为主，CLI 为辅 |
| 推荐技术栈 | Tauri 2 + Rust Core + React/Vite/TypeScript |
| 核心定位 | Local-first AI workflow asset manager |
| MVP 主目标 | 让用户用界面创建、管理、绑定、预览并同步 prompt / skill 到项目 |
| 第一优先目标工具 | Claude Code |
| 非目标 | 不做云同步、不做 AI 对话、不做 agent runtime、不做 marketplace |

---

## 1. 方向调整结论

之前的规划偏向 CLI-first：

```bash
flowmint init
flowmint new skill fastapi-review
flowmint attach skill/fastapi-review
flowmint sync --target claude-code
```

这个方案适合开发者工具，但不符合当前新的产品预期。现在应调整为：

> **Flowmint 是一个带桌面界面的本地管理工具。CLI 不是主入口，而是底层引擎的可选调用方式。**

新的产品体验应该是：

1. 用户打开 Flowmint 桌面 App；
2. 在界面中看到自己的 prompt、skill、playbook、项目；
3. 可以用编辑器创建和修改资产；
4. 可以选择本地项目目录；
5. 可以把资产绑定到项目；
6. 可以在同步前看到将要写入哪些文件；
7. 点击按钮同步到 Claude Code / 后续 Codex / Cursor；
8. 全部数据仍然保存在本地文件系统中。

所以新架构是：

```text
Desktop UI  = 用户主入口
Rust Core   = 文件、校验、同步、导出、搜索的业务引擎
CLI         = 可选高级入口 / 调试入口 / 自动化入口
```

---

## 2. 产品定位

### 2.1 一句话定位

> Flowmint 是一个本地优先的桌面管理工具，用来管理可复用的 AI 开发资产：prompts、skills、playbooks、project rules，并将它们同步到 Claude Code、Codex、Cursor 等 AI 编程工具。

### 2.2 更短的 GitHub 描述

```text
Local-first desktop manager for AI prompts, skills, playbooks, and coding-agent workflows.
```

### 2.3 目标用户

MVP 主要面向三类用户：

1. **重度 AI 编程工具用户**
   使用 Claude Code、Codex、Cursor、Gemini CLI 等工具，希望复用自己的提示词和工作流程。

2. **独立开发者 / 后端开发者**
   经常做项目初始化、代码 review、数据库设计、部署排障，希望沉淀自己的工作方法。

3. **技术团队中的早期使用者**
   想把团队规范、review checklist、项目规则保存为本地可版本化文件。

---

## 3. 核心问题定义

用户现在的问题不是“没有 prompt”，而是：

```text
好用的 AI 工作流资产散落在聊天记录、Markdown、Notion、项目 README、Claude/Cursor/Codex 配置里，无法统一管理、搜索、复用、绑定到项目和安全同步。
```

Flowmint 要解决的是：

1. 我有哪些 prompt / skill / workflow？
2. 哪些资产适合当前项目？
3. 这个项目已经绑定了哪些 AI 工作流？
4. 同步到 Claude Code 会生成或修改哪些文件？
5. 会不会覆盖我已有的 `CLAUDE.md`、`.claude/skills` 或其他配置？
6. 如何把这些资产用 Git 保存和迁移？

---

## 4. MVP 产品边界

### 4.1 MVP 必须做

MVP 聚焦 5 件事：

```text
1. 本地资产库
2. 桌面界面管理
3. 项目绑定
4. 同步预览
5. Claude Code 导出
```

### 4.2 MVP 资产类型

MVP 不要把资产类型做得太复杂。建议第一版支持 3 类：

| 类型 | MVP 是否支持 | 说明 |
|---|---:|---|
| Prompt | 支持 | 单个可复用提示词，适合复制、渲染、后续转 command |
| Skill | 支持 | 目录型能力包，核心文件是 `SKILL.md` |
| Playbook | 轻量支持 | MVP 中作为 Skill 的一种模板形态，不做独立执行引擎 |
| Rule | 暂缓 | 可在后续版本作为项目规范资产加入 |

关键取舍：

> MVP 中的 Playbook 不做“可执行工作流”，只做“结构化步骤说明”，最终渲染为 `SKILL.md` 或 Markdown 文档。

这样既能表达 playbook 的价值，又不会过早进入 agent runtime。

### 4.3 MVP 不做

明确不做：

1. 不做云同步；
2. 不做账号系统；
3. 不做团队权限；
4. 不做 marketplace；
5. 不做 AI 聊天窗口；
6. 不调用 LLM API；
7. 不做 agent runtime；
8. 不做复杂 eval；
9. 不做 VS Code 插件；
10. 不做所有 AI 工具 exporter，只先做 Claude Code。

---

## 5. 推荐技术方案

### 5.1 推荐：Tauri 2 + React + Rust Core

推荐采用：

```text
Tauri 2
Rust Core
React + Vite + TypeScript
Monaco Editor 或 CodeMirror
Tailwind CSS 或普通 CSS Modules
```

原因：

1. Flowmint 本质是本地文件管理工具，需要安全访问文件系统；
2. Rust 适合做文件扫描、校验、同步、hash、路径处理；
3. Web UI 更适合做复杂列表、编辑器、diff preview、设置页；
4. Tauri 可以把 Web UI 和 Rust 后端组合成跨平台桌面 App；
5. 后续仍然可以共用同一个 Rust Core 做 CLI。

### 5.2 为什么不是纯 CLI

CLI 的优势是简单、工程化、适合自动化；但它的问题是：

1. 对管理类产品不直观；
2. 用户难以浏览大量资产；
3. 不适合做可视化 diff 和冲突提示；
4. 不适合编辑复杂 Markdown/YAML；
5. 不适合项目绑定关系管理。

所以 CLI 应该是辅助，而不是主产品。

### 5.3 为什么不是纯 Rust GUI

可以考虑 egui、Dioxus、Slint，但 MVP 建议不要一开始走纯 Rust UI：

| 方案 | 优点 | 风险 |
|---|---|---|
| Tauri + React | UI 生态成熟，编辑器/diff 组件丰富，Rust 后端强 | 需要写 TypeScript |
| Dioxus | Rust-only 倾向，跨平台能力强 | 复杂编辑器和成熟 UI 生态不如 Web 生态直接 |
| egui | 简单、纯 Rust、开发快 | 做复杂管理界面、Markdown 编辑、diff preview 不够理想 |
| Slint | 原生 GUI，适合桌面和嵌入式 | 对 Flowmint 这种内容管理型 App，Web UI 组件生态更省事 |

MVP 需要的是“快速做出一个好用的管理界面”，所以推荐 Tauri + Web UI。

---

## 6. 总体架构

### 6.1 架构图

```text
┌──────────────────────────────────────────────┐
│              Flowmint Desktop UI             │
│  React / TypeScript / Editor / Diff Preview  │
└───────────────────────┬──────────────────────┘
                        │ Tauri commands
┌───────────────────────▼──────────────────────┐
│                 Flowmint Core                 │
│  asset store / project store / validation     │
│  sync planner / exporters / file safety       │
└───────────────────────┬──────────────────────┘
                        │ filesystem
┌───────────────────────▼──────────────────────┐
│                Local Filesystem               │
│  ~/.flowmint/                                 │
│  project/.flowmint.toml                       │
│  project/.claude/skills                       │
│  project/CLAUDE.md                            │
└──────────────────────────────────────────────┘
```

### 6.2 Rust workspace 结构

推荐仓库结构：

```text
flowmint/
  README.md
  Cargo.toml
  package.json

  crates/
    flowmint-core/
      src/
        asset/
        project/
        store/
        sync/
        exporters/
        validation/
        fs_safety/
        lib.rs

    flowmint-cli/              # P1，可选
      src/main.rs

  apps/
    desktop/
      package.json
      index.html
      src/
        main.tsx
        app/
        components/
        pages/
        routes/
        api/
        styles/
      src-tauri/
        Cargo.toml
        tauri.conf.json
        src/main.rs

  examples/
    skills/
    prompts/
    projects/

  docs/
    mvp.md
    ui.md
    storage.md
    exporters.md
```

### 6.3 核心原则

1. **Source of truth 是文件系统，不是数据库。**
2. **UI 只调用 Rust Core，不直接写业务文件。**
3. **sync 先生成计划，再写入文件。**
4. **所有写入都要可预览。**
5. **默认不覆盖用户手写内容。**
6. **CLI 和 Desktop 共用同一个 Rust Core。**

---

## 7. 本地数据存储设计

### 7.1 全局目录

用户资产默认保存在：

```text
~/.flowmint/
```

结构：

```text
~/.flowmint/
  config.toml
  recent-projects.toml

  prompts/
    fastapi-code-review.md
    prd-review.md

  skills/
    fastapi-backend-review/
      SKILL.md
      metadata.toml
      examples/
      resources/

    nginx-debug/
      SKILL.md
      metadata.toml
      resources/

  templates/
    skill-basic/
    skill-playbook/
    prompt-basic/

  cache/
    thumbnails/
    search/

  backups/
```

### 7.2 项目目录

项目中保存绑定关系：

```text
my-project/
  .flowmint.toml
  .flowmint.lock
```

`.flowmint.toml` 示例：

```toml
[project]
name = "my-fastapi-project"

[export]
target = "claude-code"

[attach]
prompts = [
  "fastapi-code-review"
]

skills = [
  "fastapi-backend-review",
  "nginx-debug"
]
```

`.flowmint.lock` 示例：

```toml
[[exports]]
target = "claude-code"
asset_type = "skill"
asset_id = "fastapi-backend-review"
source_hash = "sha256:..."
output_path = ".claude/skills/fastapi-backend-review/SKILL.md"
output_hash = "sha256:..."
updated_at = "2026-05-07T00:00:00Z"
```

### 7.3 为什么仍然不用数据库

MVP 阶段不建议用 SQLite。原因：

1. 文件数量不会太大；
2. 文件系统更透明；
3. 用户可以直接 Git 管理；
4. Debug 简单；
5. 桌面 App 可以启动时扫描目录；
6. 搜索可以先用内存索引，后续再引入 Tantivy/SQLite。

---

## 8. UI 信息架构

### 8.1 App 主导航

MVP 主界面分 5 个区域：

```text
Dashboard
Assets
Projects
Sync
Settings
```

推荐侧边栏：

```text
Flowmint

Overview
Assets
  Prompts
  Skills
  Playbooks
Projects
Sync
Settings
```

### 8.2 Dashboard

目标：让用户一打开就知道当前状态。

展示：

1. 全局资产数量；
2. 最近编辑的 prompt / skill；
3. 最近项目；
4. 待同步项目；
5. 常用操作按钮。

低保真结构：

```text
┌──────────────────────────────────────────────┐
│ Flowmint                                     │
│ Local AI workflow assets                     │
├──────────────────────────────────────────────┤
│ [New Prompt] [New Skill] [Add Project]       │
├──────────────────────────────────────────────┤
│ Assets                                       │
│ Prompts: 12   Skills: 5   Playbooks: 3       │
├──────────────────────────────────────────────┤
│ Recent Projects                              │
│ - stablecoin-backend     2 assets attached   │
│ - nginx-server-config    1 asset attached    │
├──────────────────────────────────────────────┤
│ Recent Assets                                │
│ - fastapi-backend-review                     │
│ - prd-review                                 │
└──────────────────────────────────────────────┘
```

### 8.3 Assets 页面

目标：浏览、搜索、创建、编辑本地资产。

功能：

1. 左侧列表；
2. 标签过滤；
3. 类型过滤；
4. 搜索；
5. 右侧详情；
6. 编辑按钮；
7. 验证状态；
8. 打开文件所在目录。

低保真结构：

```text
┌────────────────────────────────────────────────────────┐
│ Assets                         [New Asset]             │
├──────────────┬─────────────────────────────────────────┤
│ Search...    │ FastAPI Backend Review                  │
│              │ type: skill                             │
│ Prompts      │ tags: fastapi, backend, review          │
│ Skills       │ status: valid                           │
│ Playbooks    │                                         │
│              │ [Edit] [Attach to Project] [Open Folder]│
│ fastapi...   │                                         │
│ nginx...     │ Preview:                                │
│ prd-review   │ ┌─────────────────────────────────────┐ │
│              │ │ # FastAPI Backend Review Skill      │ │
│              │ │ ...                                 │ │
│              │ └─────────────────────────────────────┘ │
└──────────────┴─────────────────────────────────────────┘
```

### 8.4 Asset Editor

目标：让用户不用离开 App 就能编辑 prompt / skill。

#### Prompt Editor

字段：

1. ID；
2. Name；
3. Description；
4. Tags；
5. Body；
6. Variables；
7. Preview；
8. Validate。

#### Skill Editor

字段：

1. ID；
2. Name；
3. Description；
4. Tags；
5. `SKILL.md` Markdown 编辑区；
6. metadata；
7. resources 文件列表，可编辑文本资源并保留非文本资源；
8. examples 文件列表，可编辑递归子目录中的示例文件；
9. Validate。

Skill Editor 低保真：

```text
┌────────────────────────────────────────────────────────┐
│ Edit Skill: fastapi-backend-review       [Save]        │
├────────────────────────────────────────────────────────┤
│ Name: FastAPI Backend Review                           │
│ Tags: [fastapi] [backend] [review]                     │
├──────────────┬─────────────────────────────────────────┤
│ Files        │ SKILL.md                                │
│ - SKILL.md   │ ┌─────────────────────────────────────┐ │
│ - metadata   │ │ # FastAPI Backend Review            │ │
│ - examples   │ │                                     │ │
│ - resources  │ │ Use this skill when...              │ │
│              │ └─────────────────────────────────────┘ │
├──────────────┴─────────────────────────────────────────┤
│ Validation: ✅ valid                                    │
└────────────────────────────────────────────────────────┘
```

### 8.5 Projects 页面

目标：管理本地项目和项目绑定关系。

功能：

1. 添加本地项目目录；
2. 展示项目是否已初始化；
3. 展示项目绑定的资产；
4. 绑定 / 解绑资产；
5. 查看目标 exporter；
6. 进入 sync preview。

低保真结构：

```text
┌────────────────────────────────────────────────────────┐
│ Projects                              [Add Project]    │
├──────────────┬─────────────────────────────────────────┤
│ stablecoin   │ stablecoin-backend                      │
│ nginx-config │ path: ~/code/stablecoin-backend         │
│              │ target: claude-code                     │
│              │                                         │
│              │ Attached Assets                         │
│              │ Skills                                  │
│              │ - fastapi-backend-review   [detach]     │
│              │ - sqlalchemy-review        [detach]     │
│              │ Prompts                                 │
│              │ - prd-review               [detach]     │
│              │                                         │
│              │ [Attach Asset] [Preview Sync] [Sync]    │
└──────────────┴─────────────────────────────────────────┘
```

### 8.6 Attach Asset Modal

目标：让用户从资产库选择资产绑定到项目。

功能：

1. 搜索资产；
2. 按类型筛选；
3. 多选；
4. 展示资产描述和标签；
5. 点击 attach。

```text
┌────────────────────────────────────────────┐
│ Attach Assets to stablecoin-backend        │
├────────────────────────────────────────────┤
│ Search assets...                           │
│ [x] fastapi-backend-review    skill        │
│ [x] prd-review                prompt       │
│ [ ] nginx-debug               skill        │
│                                            │
│                 [Cancel] [Attach Selected] │
└────────────────────────────────────────────┘
```

### 8.7 Sync Preview 页面

这是 MVP 最关键的界面。

目标：让用户在写入项目文件前看到具体影响。

展示：

1. 本次将创建的文件；
2. 本次将修改的文件；
3. 本次不会触碰的文件；
4. 冲突文件；
5. managed block diff；
6. apply 按钮。

低保真结构：

```text
┌────────────────────────────────────────────────────────┐
│ Sync Preview: stablecoin-backend → Claude Code         │
├────────────────────────────────────────────────────────┤
│ Summary                                                │
│ + create 2 files                                       │
│ ~ update 1 file                                        │
│ ! conflict 0 files                                     │
├──────────────┬─────────────────────────────────────────┤
│ Files        │ Diff                                    │
│ + .claude/   │ ┌─────────────────────────────────────┐ │
│ + skills/... │ │ + # FastAPI Backend Review          │ │
│ ~ CLAUDE.md  │ │ + Managed by Flowmint               │ │
│              │ │ ...                                 │ │
│              │ └─────────────────────────────────────┘ │
├──────────────┴─────────────────────────────────────────┤
│ [Cancel] [Apply Sync]                                  │
└────────────────────────────────────────────────────────┘
```

### 8.8 Settings 页面

MVP 设置项：

1. Flowmint home directory；
2. 默认 exporter；
3. 外部编辑器；
4. 是否显示高级功能；
5. 打开全局资产目录；
6. 重建索引；
7. 导出 debug report。

---

## 9. MVP 核心用户流程

### 9.1 首次启动

```text
用户打开 Flowmint
→ App 检查 ~/.flowmint 是否存在
→ 不存在则显示 onboarding
→ 用户点击 Create Local Library
→ 创建 ~/.flowmint
→ 进入 Dashboard
```

验收标准：

1. 首次启动能自动检测本地库；
2. 用户可以选择默认路径或自定义路径；
3. 创建后能在文件系统看到目录；
4. 关闭并重开 App 后状态保持。

### 9.2 创建 Prompt

```text
Assets → New Asset → Prompt
→ 输入 id/name/tags/body
→ Save
→ 文件写入 ~/.flowmint/prompts/<id>.md
→ 资产列表出现新 prompt
```

验收标准：

1. 必填字段缺失时不能保存；
2. ID 只能使用安全字符：`a-z0-9-_`；
3. 保存后文件存在；
4. 编辑后能正确更新文件；
5. App 重启后仍能读取。

### 9.3 创建 Skill

```text
Assets → New Asset → Skill
→ 选择 Basic Skill 或 Playbook Skill 模板
→ 输入 id/name/tags
→ 编辑 SKILL.md
→ Save
→ 文件写入 ~/.flowmint/skills/<id>/SKILL.md
```

验收标准：

1. Skill 必须有 `SKILL.md`；
2. `SKILL.md` 不能为空；
3. Skill ID 不允许重复；
4. 可以打开 skill 文件夹；
5. 可以从 UI 编辑 `SKILL.md`。

### 9.4 添加项目

```text
Projects → Add Project
→ 选择本地项目目录
→ App 检查 .flowmint.toml
→ 不存在则提示初始化
→ 创建 .flowmint.toml
→ 项目加入 recent-projects
```

验收标准：

1. 用户可以选择任意本地目录；
2. 已有 `.flowmint.toml` 时读取配置；
3. 没有 `.flowmint.toml` 时可以初始化；
4. 最近项目列表持久化。

### 9.5 绑定资产

```text
Projects → Select Project → Attach Asset
→ 选择 prompt / skill
→ 保存到 project/.flowmint.toml
```

验收标准：

1. 绑定关系写入 `.flowmint.toml`；
2. 不重复绑定；
3. 可以解绑；
4. 如果资产不存在，项目详情显示 missing 状态。

### 9.6 同步到 Claude Code

```text
Projects → Select Project → Preview Sync
→ 查看即将写入的文件
→ Apply Sync
→ 生成 .claude/skills/<id>/SKILL.md
→ 更新 CLAUDE.md managed block
→ 写入 .flowmint.lock
```

验收标准：

1. Apply 前必须能看到 sync plan；
2. 默认不覆盖用户手写文件；
3. 如果目标文件被用户手改，显示 conflict；
4. 重复 sync 幂等；
5. `.flowmint.lock` 记录输出文件 hash。

---

## 10. Claude Code Exporter MVP 设计

### 10.1 输出目录

对项目：

```text
my-project/
  .claude/
    skills/
      fastapi-backend-review/
        SKILL.md
        metadata.toml
        examples/
        resources/
  CLAUDE.md
```

### 10.2 Prompt 如何导出

MVP 中 prompt 可以导出为 Claude command 或写入 `CLAUDE.md` managed block。为了降低复杂度，建议 MVP 先采用：

```text
prompts → .claude/commands/<prompt-id>.md
```

如果后续 Claude Code 命令格式变化，可以通过 exporter 调整。

### 10.3 Skill 如何导出

```text
~/.flowmint/skills/<id>/
→ project/.claude/skills/<id>/
```

必须复制：

1. `SKILL.md`；
2. `metadata.toml`，如果存在；
3. `examples/`，如果存在；
4. `resources/`，如果存在。

MVP 暂不复制或执行 `scripts/`，除非用户明确开启。

### 10.4 CLAUDE.md managed block

Flowmint 只写入 managed block：

```md
<!-- FLOWMINT:BEGIN -->
## Flowmint Managed AI Workflows

This project uses the following Flowmint assets:

### Skills
- fastapi-backend-review
- nginx-debug

### Prompts
- prd-review

<!-- FLOWMINT:END -->
```

规则：

1. 如果没有 managed block，则追加到文件末尾；
2. 如果有 managed block，则只替换 block 内内容；
3. 不修改 block 外用户内容；
4. 如果 marker 被破坏，提示冲突，不自动修复。

---

## 11. 写入安全设计

### 11.1 SyncPlan

所有同步必须先生成计划：

```rust
pub struct SyncPlan {
    pub project_path: PathBuf,
    pub target: ExportTarget,
    pub operations: Vec<SyncOperation>,
    pub conflicts: Vec<SyncConflict>,
}

pub enum SyncOperation {
    CreateFile { path: PathBuf, content_preview: String },
    UpdateFile { path: PathBuf, old_hash: String, new_hash: String, diff: String },
    CreateDir { path: PathBuf },
    DeleteGeneratedFile { path: PathBuf },
    Noop { path: PathBuf },
}
```

UI 的 Sync Preview 只展示 SyncPlan。用户点击 Apply 后才执行写入。

### 11.2 默认冲突规则

以下情况必须显示冲突，不允许静默覆盖：

1. 目标文件存在，但不在 `.flowmint.lock` 中；
2. 目标文件 hash 与 lock 记录不一致；
3. `CLAUDE.md` managed marker 不完整；
4. 目标路径是 symlink；
5. 资产 ID 解析后会逃逸项目目录；
6. 输出目录没有写入权限。

### 11.3 解决冲突

MVP 中冲突解决只支持：

1. Cancel；
2. Open File。

`Mark as unmanaged` 和 `Force overwrite` 需要后端拥有明确的 lockfile 语义、状态变更语义和安全 apply 行为后再开放。v0.1 默认不提供激进覆盖按钮。

---

## 12. Tauri Commands 设计

前端不直接操作文件系统，而是调用 Tauri commands。

### 12.1 Library commands

```rust
#[tauri::command]
fn get_app_state() -> Result<AppState, Error>;

#[tauri::command]
fn init_library(path: Option<PathBuf>) -> Result<LibraryInfo, Error>;

#[tauri::command]
fn open_library_folder() -> Result<(), Error>;
```

### 12.2 Asset commands

```rust
#[tauri::command]
fn list_assets(filter: AssetFilter) -> Result<Vec<AssetSummary>, Error>;

#[tauri::command]
fn get_asset(asset_ref: String) -> Result<AssetDetail, Error>;

#[tauri::command]
fn create_asset(input: CreateAssetInput) -> Result<AssetDetail, Error>;

#[tauri::command]
fn update_asset(input: UpdateAssetInput) -> Result<AssetDetail, Error>;

#[tauri::command]
fn delete_asset(asset_ref: String) -> Result<(), Error>;

#[tauri::command]
fn validate_asset(asset_ref: String) -> Result<ValidationReport, Error>;
```

### 12.3 Project commands

```rust
#[tauri::command]
fn list_projects() -> Result<Vec<ProjectSummary>, Error>;

#[tauri::command]
fn add_project(path: PathBuf) -> Result<ProjectDetail, Error>;

#[tauri::command]
fn get_project(path: PathBuf) -> Result<ProjectDetail, Error>;

#[tauri::command]
fn attach_asset(project_path: PathBuf, asset_ref: String) -> Result<ProjectDetail, Error>;

#[tauri::command]
fn detach_asset(project_path: PathBuf, asset_ref: String) -> Result<ProjectDetail, Error>;
```

### 12.4 Sync commands

```rust
#[tauri::command]
fn preview_sync(project_path: PathBuf, target: ExportTarget) -> Result<SyncPlan, Error>;

#[tauri::command]
fn apply_sync(plan_id: String) -> Result<SyncResult, Error>;
```

注意：`apply_sync` 不应盲目信任前端传回来的任意文件操作，应该通过后端保存或重新生成 plan。

---

## 13. 核心数据结构建议

### 13.1 Asset

```rust
pub enum AssetType {
    Prompt,
    Skill,
    Playbook,
}

pub struct AssetSummary {
    pub id: String,
    pub asset_type: AssetType,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub path: PathBuf,
    pub validation_status: ValidationStatus,
    pub updated_at: Option<DateTime<Utc>>,
}
```

### 13.2 Prompt

```rust
pub struct PromptAsset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub variables: Vec<PromptVariable>,
    pub body: String,
}
```

### 13.3 Skill

```rust
pub struct SkillAsset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub root_dir: PathBuf,
    pub skill_md: String,
    pub metadata: Option<SkillMetadata>,
    pub files: Vec<SkillFile>,
}

pub struct SkillFile {
    pub path: PathBuf,
    pub kind: SkillFileKind,
    pub content: Option<String>,
}
```

### 13.4 Project

```rust
pub struct ProjectManifest {
    pub project: ProjectInfo,
    pub export: ExportConfig,
    pub attach: AttachedAssets,
}

pub struct AttachedAssets {
    pub prompts: Vec<String>,
    pub skills: Vec<String>,
}
```

Playbook 在 v0.1 中只是 Skill 模板形态，不作为独立项目绑定字段。

---

## 14. 前端组件规划

### 14.1 页面组件

```text
src/pages/
  DashboardPage.tsx
  AssetsPage.tsx
  AssetEditorPage.tsx
  ProjectsPage.tsx
  ProjectDetailPage.tsx
  SyncPreviewPage.tsx
  SettingsPage.tsx
```

### 14.2 业务组件

```text
src/components/
  AppSidebar.tsx
  TopBar.tsx
  AssetList.tsx
  AssetCard.tsx
  AssetTypeBadge.tsx
  TagInput.tsx
  MarkdownEditor.tsx
  MetadataForm.tsx
  ProjectList.tsx
  AttachedAssetList.tsx
  SyncOperationList.tsx
  DiffViewer.tsx
  ConflictBanner.tsx
  EmptyState.tsx
  ErrorBoundary.tsx
```

### 14.3 API 封装

```text
src/api/
  tauri.ts
  assets.ts
  projects.ts
  sync.ts
  settings.ts
```

前端所有 Tauri 调用统一封装，避免页面直接写 `invoke(...)`。

---

## 15. MVP 版本切分

### 15.1 v0.1-alpha：桌面壳 + 本地资产库

目标：App 能启动，能创建和编辑资产。

必须完成：

1. Tauri 项目初始化；
2. React 主界面；
3. 首次启动 onboarding；
4. 创建 `~/.flowmint`；
5. Assets 页面；
6. Prompt 创建 / 编辑 / 保存；
7. Skill 创建 / 编辑 / 保存；
8. 基础 validate。

不要求：

1. 项目绑定；
2. sync；
3. exporter；
4. diff preview。

完成定义：

> 用户可以打开桌面 App，在 UI 中创建一个 Prompt 和一个 Skill，关闭重开后仍能看到并编辑。

### 15.2 v0.1-beta：项目绑定 + Sync Preview

目标：App 能管理项目，并生成同步计划。

新增：

1. Projects 页面；
2. Add Project；
3. 初始化 `.flowmint.toml`；
4. Attach / Detach；
5. Preview Sync；
6. SyncPlan 展示；
7. 冲突检测展示。

完成定义：

> 用户可以添加一个本地项目，把 skill 绑定进去，并在 UI 中看到将要写入 `.claude/skills/...` 的计划。

### 15.3 v0.1：Claude Code Apply Sync + 打包发布

目标：App 可以真实同步到 Claude Code。

新增：

1. Apply Sync；
2. `.claude/skills/<id>/SKILL.md` 写入；
3. `CLAUDE.md` managed block；
4. `.flowmint.lock`；
5. 重复 sync 幂等；
6. macOS / Linux / Windows 基础打包；
7. GitHub Actions CI；
8. README quickstart。

完成定义：

> 用户可以用桌面界面完成：创建 skill → 添加项目 → 绑定 skill → 预览同步 → 应用同步 → 在 Claude Code 项目里看到生成的 skill。

---

## 16. GitHub Issue 拆分建议

### Milestone 0：仓库和基础架构

1. `chore: initialize Rust workspace and Tauri desktop app`
2. `chore: add frontend build setup with React, Vite, TypeScript`
3. `chore: add flowmint-core crate`
4. `chore: add shared error handling and serde models`
5. `chore: setup GitHub Actions for fmt, clippy, test, frontend build`

### Milestone 1：本地库

1. `feat(core): resolve Flowmint home directory`
2. `feat(core): initialize local library structure`
3. `feat(desktop): onboarding screen for local library setup`
4. `feat(core): load config and recent projects`

### Milestone 2：资产管理

1. `feat(core): implement prompt asset parser and writer`
2. `feat(core): implement skill asset parser and writer`
3. `feat(core): implement asset listing and validation`
4. `feat(desktop): assets page with filters and search`
5. `feat(desktop): prompt editor`
6. `feat(desktop): skill editor`

### Milestone 3：项目管理

1. `feat(core): project manifest parser and writer`
2. `feat(desktop): add project dialog`
3. `feat(desktop): project detail page`
4. `feat(core): attach and detach asset to project`
5. `feat(desktop): attach asset modal`

### Milestone 4：同步预览

1. `feat(core): define SyncPlan and SyncOperation`
2. `feat(core): implement claude-code sync planner`
3. `feat(core): implement managed block generation for CLAUDE.md`
4. `feat(desktop): sync preview page`
5. `feat(desktop): diff viewer`
6. `feat(desktop): conflict banner`

### Milestone 5：真实同步

1. `feat(core): apply sync operations safely`
2. `feat(core): write .flowmint.lock`
3. `feat(core): detect modified generated files using hash`
4. `feat(desktop): apply sync action and result screen`
5. `test(core): sync idempotency tests`
6. `test(core): conflict detection tests`

### Milestone 6：发布准备

1. `docs: add README quickstart`
2. `docs: add examples for prompt and skill`
3. `chore: configure Tauri bundling`
4. `chore: add release workflow`
5. `fix: cross-platform path handling`
6. `test: manual smoke test on macOS/Linux/Windows`

---

## 17. MVP 验收标准

### 17.1 功能验收

MVP 完成时必须满足：

1. 用户可以启动桌面 App；
2. 用户可以创建本地 Flowmint library；
3. 用户可以创建 Prompt；
4. 用户可以创建 Skill；
5. 用户可以编辑 Prompt / Skill，包括 Skill 的 `metadata.toml`、`examples/` 和 `resources/` 文本文件；
6. 用户可以添加本地项目，桌面环境可用时支持目录选择器，仍保留手动路径输入；
7. 用户可以初始化项目 `.flowmint.toml`；
8. 用户可以把 asset 绑定到项目；
9. 用户可以预览同步计划；
10. 用户可以同步到 Claude Code；
11. 用户可以看到生成的 `.claude/skills/<id>/SKILL.md`；
12. 用户可以看到 Skill 的 `metadata.toml`、`examples/`、`resources/` 被递归导出；
13. 用户可以看到 `CLAUDE.md` managed block；
14. 全局用户同步必须显示根目录和实际变更路径，并要求二次确认；
15. 重复同步不会产生额外变更；
16. 目标文件被手改时会提示冲突。

### 17.2 安全验收

1. 不静默覆盖用户文件；
2. 不写出项目目录；
3. 不跟随危险 symlink；
4. 不执行 skill 里的脚本；
5. 不上传任何本地文件；
6. 不调用任何 AI API；
7. 所有文件写入都经过 Rust Core。

### 17.3 体验验收

1. 首次启动 30 秒内能理解产品用途；
2. 创建 skill 不需要看文档；
3. 同步前能清楚看到变更；
4. 冲突提示能说明原因；
5. 资产列表能快速搜索；
6. App 重启后状态完整恢复。

---

## 18. README 首版结构

```md
# Flowmint

Local-first desktop manager for AI prompts, skills, playbooks, and coding-agent workflows.

## What is Flowmint?

Flowmint helps developers manage reusable AI workflow assets locally and sync them to tools like Claude Code.

## Why?

AI workflows are becoming reusable assets, but they are scattered across prompts, project docs, and tool-specific config files.

## Features

- Local prompt and skill library
- Desktop asset manager
- Project binding
- Sync preview
- Safe Claude Code export
- File-based storage

## Quickstart

1. Download Flowmint
2. Create local library
3. Create a skill
4. Add a project
5. Attach the skill
6. Preview sync
7. Apply sync

## Storage

Flowmint stores assets in `~/.flowmint` and project bindings in `.flowmint.toml`.

## Roadmap

- Codex exporter (implemented in the later multi-agent sync phase)
- Gemini CLI exporter (implemented in the later multi-agent sync phase)
- Cursor exporter
- Playbook editor (implemented in the later multi-agent sync phase)
- Rule assets (implemented in the later multi-agent sync phase)
- Search index
- Git-based sync
```

---

## 19. 关键风险与应对

### 风险 1：UI MVP 过大

应对：

```text
第一版只做 Prompt + Skill + Project + Claude Code Sync。
Playbook 只作为 Skill 模板，不做独立复杂功能。
```

### 风险 2：编辑器复杂度过高

应对：

```text
先用普通 textarea 或轻量 Markdown editor。
Monaco / CodeMirror 可以后续替换。
```

### 风险 3：Tauri 文件权限和跨平台路径问题

应对：

```text
所有路径处理放 Rust Core。
写入前做 path normalization。
CI 加跨平台测试。
```

### 风险 4：Claude Code 格式变化

应对：

```text
Exporter 独立成模块。
不要把 Claude Code 路径和格式写死在 UI 里。
```

### 风险 5：用户担心文件被覆盖

应对：

```text
Sync Preview 是核心功能。
默认所有冲突拒绝写入。
所有 Flowmint 写入文件都记录 lock hash。
```

---

## 20. 最终建议

新的 MVP 不应该继续按 CLI-first 做。推荐路线是：

```text
Flowmint Desktop App
  → 本地资产库
  → Prompt / Skill 管理界面
  → 项目绑定界面
  → Sync Preview
  → Claude Code Exporter
```

CLI 可以保留，但放在后面：

```text
v0.1: Desktop only, Rust Core ready
v0.2: Optional CLI using same core
v0.3: Codex exporter / Gemini CLI exporter / first-class Playbook / Rules (implemented)
v0.4: Cursor exporter and richer workflow templates
v0.5: Git sync / registry
```

最小真实闭环应该变成：

```text
打开 Flowmint 桌面 App
→ 创建一个 Skill
→ 添加一个本地项目
→ 绑定这个 Skill
→ 预览同步结果
→ 点击 Apply Sync
→ 项目中生成 Claude Code 可用的 skill
```

这比 CLI-first 更符合“带界面的管理工具”的产品形态，也更容易让非命令行重度用户理解 Flowmint 的价值。

---

## 21. 资料依据

- Tauri 官方文档：Tauri 支持使用 Web 前端构建界面，并使用 Rust 作为后端应用逻辑。
- Tauri 官方文档：Tauri 支持跨平台桌面应用开发。
- Dioxus 官方介绍：Dioxus 是 Rust fullstack/cross-platform app framework。
- egui 官方 README：egui 是 Rust immediate mode GUI library。
- Slint 官方介绍：Slint 是面向 Rust/C++/JavaScript/Python 的 declarative GUI toolkit。

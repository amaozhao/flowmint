import type { AssetDetail } from "../src/api/assets";
import {
  buildDraftFromSkillTemplate,
  buildEmptyAssetDraft,
  getDraftValidationMessages,
  toAssetDetail,
} from "../src/pages/assetEditorModel";
import {
  buildAssetDistributionRows,
  buildProjectSyncRows,
  buildTargetSupportRows,
} from "../src/pages/dashboardModel";
import { importProjectPathRequired, projectPathForImport } from "../src/pages/importPageModel";
import { createTranslator } from "../src/i18n/messages";

function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertDeepEqual<T>(actual: T, expected: T, message: string) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

const emptyPrompt = buildEmptyAssetDraft("prompt");
assertDeepEqual(
  getDraftValidationMessages(emptyPrompt),
  ["ID is required.", "Name is required.", "Prompt body is required."],
  "empty prompt reports required fields",
);
assertDeepEqual(
  getDraftValidationMessages(emptyPrompt, createTranslator("zh")),
  ["ID 为必填项。", "名称为必填项。", "提示词内容为必填项。"],
  "empty prompt reports localized required fields",
);

const badSkill = {
  ...buildEmptyAssetDraft("skill"),
  id: "Bad ID",
  name: "Research Helper",
};
assertDeepEqual(
  getDraftValidationMessages(badSkill),
  ["ID must use lowercase letters, numbers, dashes, or underscores.", "SKILL.md is required."],
  "bad skill reports safe ID and SKILL.md errors",
);

const promptDetail: AssetDetail = toAssetDetail({
  ...buildEmptyAssetDraft("prompt"),
  id: "daily-plan",
  name: "Daily Plan",
  description: "Turn notes into a plan",
  tags: ["planning", "ops"],
  variables: [
    {
      name: "topic",
      description: "Planning topic",
      defaultValue: "today",
    },
  ],
  body: "# Plan\n\nSummarize this.",
});

if (promptDetail.assetType !== "prompt") {
  throw new Error("prompt draft converts to prompt detail");
}
assert(promptDetail.asset.id === "daily-plan", "prompt ID is preserved");
assert(promptDetail.asset.variables.length === 1, "prompt variables are preserved");
assert(promptDetail.asset.variables[0].name === "topic", "prompt variable name is preserved");

const skillDetail: AssetDetail = toAssetDetail({
  ...buildEmptyAssetDraft("skill"),
  id: "research-helper",
  name: "Research Helper",
  tags: ["research"],
  skillMd: "# Research Helper\n\nUse primary sources.",
  metadataToml: "version = \"0.1.0\"",
  files: [
    {
      path: "examples/request.md",
      kind: "example",
      content: "Example request",
    },
  ],
});

if (skillDetail.assetType !== "skill") {
  throw new Error("skill draft converts to skill detail");
}
assert(skillDetail.asset.rootDir === "", "new skill root dir is left for the backend to resolve");
assert(skillDetail.asset.metadata?.rawToml === "version = \"0.1.0\"", "skill metadata is included");
assert(skillDetail.asset.files[0].content === "Example request", "skill supporting file content is included");

const playbookDraft = buildDraftFromSkillTemplate({
  kind: "playbook",
  name: "Playbook Skill",
  description: "Structured workflow steps",
  tags: ["playbook"],
  skillMd: "# Playbook Skill\n\n## Steps\n\n1. Define the entry condition.\n",
});

assert(playbookDraft.assetType === "skill", "playbook template creates a skill draft");
assert(playbookDraft.id === "", "playbook template leaves id for the user");
assert(playbookDraft.tags.includes("playbook"), "playbook template tags the skill");
assert(playbookDraft.skillMd.includes("## Steps"), "playbook template includes structured steps");
assertDeepEqual(
  getDraftValidationMessages(playbookDraft),
  ["ID is required."],
  "playbook template only requires the user to provide an id",
);

const firstClassPlaybook = toAssetDetail({
  ...buildEmptyAssetDraft("playbook"),
  id: "release-check",
  name: "Release Check",
  trigger: "Before release",
  steps: [{ title: "Run checks", body: "Run the full suite." }],
  verification: "All checks pass.",
  failureHandling: "Stop and report failures.",
  sideEffectLevel: "runs-commands",
  recommendedInvocation: "manual",
  targetCompatibility: ["claude-code", "codex"],
});

if (firstClassPlaybook.assetType !== "playbook") {
  throw new Error("playbook draft converts to playbook detail");
}
assert(firstClassPlaybook.asset.steps.length === 1, "playbook steps are preserved");
assert(firstClassPlaybook.asset.sideEffectLevel === "runs-commands", "playbook side effect level is preserved");

assertDeepEqual(
  getDraftValidationMessages({
    ...buildEmptyAssetDraft("playbook"),
    id: "release-check",
    name: "Release Check",
  }),
  [
    "Playbook trigger is required.",
    "Playbook requires at least one step.",
    "Playbook verification is required.",
    "Playbook failure handling is required.",
  ],
  "playbook draft reports playbook-specific required fields",
);

const instructionRule = toAssetDetail({
  ...buildEmptyAssetDraft("instruction-rule"),
  id: "typescript-style",
  name: "TypeScript Style",
  body: "Prefer explicit return types.",
  pathGlobs: ["src/**/*.ts"],
});

if (instructionRule.assetType !== "instruction-rule") {
  throw new Error("instruction rule draft converts to instruction rule detail");
}
assert(instructionRule.asset.ruleKind === "instruction", "instruction rule kind is set");
assert(instructionRule.asset.commandRule === null, "instruction rule has no command rule payload");

const commandRule = toAssetDetail({
  ...buildEmptyAssetDraft("command-rule"),
  id: "safe-git-status",
  name: "Safe Git Status",
  body: "Prompt before status checks.",
  commandPrefix: ["git", "status"],
});

if (commandRule.assetType !== "command-rule") {
  throw new Error("command rule draft converts to command rule detail");
}
assert(commandRule.asset.commandRule?.decision === "prompt", "command rule defaults to prompt");
assertDeepEqual(
  getDraftValidationMessages({
    ...buildEmptyAssetDraft("command-rule"),
    id: "safe-git-status",
    name: "Safe Git Status",
    body: "Prompt before status checks.",
  }),
  ["Command Rule prefix is required."],
  "command rule requires a prefix",
);

assert(importProjectPathRequired("project"), "project import requires a project path");
assert(!importProjectPathRequired("global-user"), "global import does not require a project path");
assert(projectPathForImport("", "global-user") === ".", "global import can run without a selected project");
assert(
  projectPathForImport(" /tmp/project ", "project") === "/tmp/project",
  "project import trims the selected path",
);

const assetChartRows = buildAssetDistributionRows([
  {
    id: "daily-plan",
    assetType: "prompt",
    name: "Daily Plan",
    description: null,
    tags: [],
    path: "/tmp/daily-plan.md",
    validationStatus: "valid",
    updatedAt: null,
  },
  {
    id: "research-helper",
    assetType: "skill",
    name: "Research Helper",
    description: null,
    tags: [],
    path: "/tmp/research-helper",
    validationStatus: "valid",
    updatedAt: null,
  },
  {
    id: "legacy-playbook",
    assetType: "skill",
    name: "Legacy Playbook",
    description: null,
    tags: ["playbook"],
    path: "/tmp/legacy-playbook",
    validationStatus: "valid",
    updatedAt: null,
  },
  {
    id: "style",
    assetType: "instruction-rule",
    name: "Style",
    description: null,
    tags: [],
    path: "/tmp/style.md",
    validationStatus: "valid",
    updatedAt: null,
  },
]);
assertDeepEqual(
  assetChartRows.map((row) => [row.id, row.value, row.percent]),
  [
    ["prompt", 1, 25],
    ["skill", 1, 25],
    ["playbook", 1, 25],
    ["instruction-rule", 1, 25],
    ["command-rule", 0, 0],
  ],
  "asset chart separates normal skills from playbook skills",
);

assertDeepEqual(
  buildProjectSyncRows([
    {
      path: "/tmp/ready",
      name: "ready",
      initialized: true,
      attachedPrompts: 1,
      attachedSkills: 0,
      attachedAssets: 1,
    },
    {
      path: "/tmp/empty",
      name: "empty",
      initialized: true,
      attachedPrompts: 0,
      attachedSkills: 0,
      attachedAssets: 0,
    },
  ]).map((row) => [row.id, row.value, row.percent]),
  [
    ["configured", 1, 50],
    ["empty", 1, 50],
  ],
  "project chart shows configured versus empty projects",
);

assertDeepEqual(
  buildTargetSupportRows([
    {
      targetId: "codex",
      displayName: "Codex",
      capabilities: [
        { assetKind: "skill", scope: "project", support: "supported", outputHint: "", reason: "" },
        { assetKind: "prompt", scope: "project", support: "unsupported", outputHint: "", reason: "" },
        { assetKind: "playbook", scope: "project", support: "requires-validation", outputHint: "", reason: "" },
      ],
    },
  ]),
  [
    {
      targetId: "codex",
      displayName: "Codex",
      supported: 1,
      blocked: 1,
      requiresValidation: 1,
      total: 3,
      supportedPercent: 33,
      blockedPercent: 33,
      requiresValidationPercent: 33,
    },
  ],
  "target support chart aggregates target capability status",
);

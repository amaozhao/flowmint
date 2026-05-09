import type { AssetSummary } from "../src/api/assets";
import type { TargetCapabilities } from "../src/api/sync";
import { unsupportedReasonsForAssets } from "../src/utils/targetCapabilities";

function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

const capabilities: TargetCapabilities[] = [
  {
    targetId: "codex",
    displayName: "Codex",
    capabilities: [
      {
        assetKind: "prompt",
        scope: "project",
        support: "unsupported",
        outputHint: "",
        reason: "Codex prompt commands are not supported.",
      },
      {
        assetKind: "skill",
        scope: "project",
        support: "supported",
        outputHint: ".agents/skills/<id>/",
        reason: "",
      },
    ],
  },
  {
    targetId: "gemini-cli",
    displayName: "Gemini CLI",
    capabilities: [
      {
        assetKind: "skill",
        scope: "project",
        support: "requires-validation",
        outputHint: ".gemini/skills/<id>/",
        reason: "Gemini skill discovery requires local validation.",
      },
    ],
  },
];

const assets: AssetSummary[] = [
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
];

const codexReasons = unsupportedReasonsForAssets(capabilities, "codex", "project", assets);
assert(
  codexReasons["prompt:daily-plan"] === "Codex prompt commands are not supported.",
  "Codex prompt attachment is disabled with backend reason",
);
assert(!codexReasons["skill:research-helper"], "Codex skill attachment remains enabled");

const geminiReasons = unsupportedReasonsForAssets(capabilities, "gemini-cli", "project", assets);
assert(
  geminiReasons["skill:research-helper"] === "Gemini skill discovery requires local validation.",
  "Gemini skill attachment is disabled while validation is required",
);

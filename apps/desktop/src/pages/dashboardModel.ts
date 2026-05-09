import type { AssetSummary } from "../api/assets";
import type { ProjectSummary } from "../api/projects";
import type { TargetCapabilities } from "../api/sync";

export type ChartRowId =
  | "prompt"
  | "skill"
  | "playbook"
  | "instruction-rule"
  | "command-rule"
  | "configured"
  | "empty";

export type ChartRow = {
  id: ChartRowId;
  value: number;
  percent: number;
};

export type TargetSupportRow = {
  targetId: string;
  displayName: string;
  supported: number;
  blocked: number;
  requiresValidation: number;
  total: number;
  supportedPercent: number;
  blockedPercent: number;
  requiresValidationPercent: number;
};

export function buildAssetDistributionRows(assets: AssetSummary[]): ChartRow[] {
  return withPercentages([
    { id: "prompt", value: assets.filter((asset) => asset.assetType === "prompt").length },
    {
      id: "skill",
      value: assets.filter((asset) => asset.assetType === "skill" && !asset.tags.includes("playbook")).length,
    },
    {
      id: "playbook",
      value: assets.filter((asset) => asset.assetType === "playbook" || asset.tags.includes("playbook")).length,
    },
    {
      id: "instruction-rule",
      value: assets.filter((asset) => asset.assetType === "instruction-rule").length,
    },
    { id: "command-rule", value: assets.filter((asset) => asset.assetType === "command-rule").length },
  ]);
}

export function buildProjectSyncRows(projects: ProjectSummary[]): ChartRow[] {
  return withPercentages([
    { id: "configured", value: projects.filter((project) => project.attachedAssets > 0).length },
    { id: "empty", value: projects.filter((project) => project.attachedAssets === 0).length },
  ]);
}

export function buildTargetSupportRows(targetCapabilities: TargetCapabilities[]): TargetSupportRow[] {
  return targetCapabilities.map((target) => {
    const supported = target.capabilities.filter((capability) => capability.support === "supported").length;
    const blocked = target.capabilities.filter((capability) => capability.support === "unsupported").length;
    const requiresValidation = target.capabilities.filter(
      (capability) => capability.support === "requires-validation",
    ).length;
    const total = supported + blocked + requiresValidation;

    return {
      targetId: target.targetId,
      displayName: target.displayName,
      supported,
      blocked,
      requiresValidation,
      total,
      supportedPercent: percentage(supported, total),
      blockedPercent: percentage(blocked, total),
      requiresValidationPercent: percentage(requiresValidation, total),
    };
  });
}

function withPercentages(rows: Array<Omit<ChartRow, "percent">>): ChartRow[] {
  const total = rows.reduce((sum, row) => sum + row.value, 0);
  return rows.map((row) => ({ ...row, percent: percentage(row.value, total) }));
}

function percentage(value: number, total: number): number {
  if (total === 0) {
    return 0;
  }
  return Math.floor((value / total) * 100);
}

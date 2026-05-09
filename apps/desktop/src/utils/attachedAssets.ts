import { assetRefForSummary, type AssetSummary, type AssetType } from "../api/assets";
import type { AttachedAsset } from "../api/projects";
import type { ExportProfile } from "../api/sync";

export function attachedAssetsForProfile(
  profile: ExportProfile | null | undefined,
  summaries: AssetSummary[],
): AttachedAsset[] {
  if (!profile) {
    return [];
  }

  return [
    ...profile.prompts.map((id) => attachedAsset("prompt", id, summaries)),
    ...profile.skills.map((id) => attachedAsset("skill", id, summaries)),
    ...profile.playbooks.map((id) => attachedAsset("playbook", id, summaries)),
    ...profile.instructionRules.map((id) => attachedAsset("instruction-rule", id, summaries)),
    ...profile.commandRules.map((id) => attachedAsset("command-rule", id, summaries)),
  ];
}

function attachedAsset(assetType: AssetType, id: string, summaries: AssetSummary[]): AttachedAsset {
  const summary =
    summaries.find((asset) => asset.assetType === assetType && asset.id === id) ?? null;
  const assetRef = `${assetType}:${id}`;

  return {
    assetType,
    id,
    assetRef,
    state: summary ? "available" : "missing",
    summary,
  };
}

export function unattachedAssetsForProfile(
  assets: AssetSummary[],
  attachedAssets: AttachedAsset[],
): AssetSummary[] {
  const attachedRefs = new Set(attachedAssets.map((asset) => asset.assetRef));
  return assets.filter((asset) => !attachedRefs.has(assetRefForSummary(asset)));
}

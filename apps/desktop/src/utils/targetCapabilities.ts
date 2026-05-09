import type { AssetSummary, AssetType } from "../api/assets";
import type { ExportAssetKind, SyncScope, TargetCapabilities } from "../api/sync";

export function unsupportedReasonsForAssets(
  targetCapabilities: TargetCapabilities[],
  targetId: string,
  scope: SyncScope,
  assets: AssetSummary[],
): Record<string, string> {
  const target = targetCapabilities.find((capabilities) => capabilities.targetId === targetId);
  if (!target) {
    return {};
  }

  const reasons: Record<string, string> = {};
  for (const asset of assets) {
    const capability = target.capabilities.find(
      (entry) => entry.assetKind === exportAssetKindForAssetType(asset.assetType) && entry.scope === scope,
    );
    if (!capability || capability.support === "supported") {
      continue;
    }
    reasons[assetRefForAsset(asset)] =
      capability.reason || (capability.support === "requires-validation" ? "Requires validation." : "Unsupported.");
  }
  return reasons;
}

function exportAssetKindForAssetType(assetType: AssetType): ExportAssetKind {
  return assetType;
}

function assetRefForAsset(asset: Pick<AssetSummary, "assetType" | "id">): string {
  return `${asset.assetType}:${asset.id}`;
}

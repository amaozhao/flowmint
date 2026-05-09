import type { AttachedAsset } from "../api/projects";
import { useI18n } from "../i18n/I18nProvider";
import { AssetTypeBadge } from "./AssetTypeBadge";
import { EmptyState } from "./EmptyState";

type AttachedAssetListProps = {
  assets: AttachedAsset[];
  onDetach: (assetRef: string) => void;
};

export function AttachedAssetList({ assets, onDetach }: AttachedAssetListProps) {
  const { t } = useI18n();

  if (assets.length === 0) {
    return <EmptyState title={t("projects.noAttachedTitle")} message={t("projects.noAttachedMessage")} />;
  }

  return (
    <div className="attached-asset-list">
      {assets.map((asset) => (
        <article className={asset.state === "missing" ? "attached-asset missing" : "attached-asset"} key={asset.assetRef}>
          <div className="attached-asset-main">
            <div className="asset-card-header">
              <AssetTypeBadge assetType={asset.assetType} />
              <span className="asset-id">{asset.id}</span>
              <span className={asset.state === "missing" ? "state-pill missing" : "state-pill"}>
                {asset.state === "missing" ? t("common.missingState") : t("common.available")}
              </span>
            </div>
            <strong>{asset.summary?.name ?? asset.id}</strong>
            {asset.summary?.description ? (
              <p className="asset-description">{asset.summary.description}</p>
            ) : null}
          </div>
          <button className="secondary-action" type="button" onClick={() => onDetach(asset.assetRef)}>
            {t("common.remove")}
          </button>
        </article>
      ))}
    </div>
  );
}

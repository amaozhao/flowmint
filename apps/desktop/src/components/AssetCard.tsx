import { assetRefForSummary, type AssetSummary } from "../api/assets";
import { AssetTypeBadge } from "./AssetTypeBadge";

type AssetCardProps = {
  asset: AssetSummary;
  active: boolean;
  disabled?: boolean;
  disabledReason?: string;
  onSelect: (asset: AssetSummary) => void;
};

export function AssetCard({ asset, active, disabled = false, disabledReason, onSelect }: AssetCardProps) {
  return (
    <button
      className={active ? "asset-card active" : disabled ? "asset-card disabled" : "asset-card"}
      type="button"
      disabled={disabled}
      onClick={() => onSelect(asset)}
      aria-pressed={active}
    >
      <span className="asset-card-header">
        <AssetTypeBadge assetType={asset.assetType} />
        <span className="asset-id">{asset.id}</span>
      </span>

      <span className="asset-name">{asset.name}</span>
      {asset.description ? <span className="asset-description">{asset.description}</span> : null}

      {asset.tags.length > 0 ? (
        <span className="tag-row">
          {asset.tags.map((tag) => (
            <span className="tag" key={`${assetRefForSummary(asset)}:${tag}`}>
              {tag}
            </span>
          ))}
        </span>
      ) : null}

      {disabledReason ? <span className="asset-description">{disabledReason}</span> : null}
    </button>
  );
}

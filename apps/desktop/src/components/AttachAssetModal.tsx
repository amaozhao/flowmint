import { useMemo, useState } from "react";
import { assetRefForSummary, type AssetSummary, type AssetType } from "../api/assets";
import { useI18n } from "../i18n/I18nProvider";
import { AssetCard } from "./AssetCard";
import { EmptyState } from "./EmptyState";

type AttachTypeFilter = "all" | AssetType;

type AttachAssetModalProps = {
  assets: AssetSummary[];
  attachedRefs: string[];
  unsupportedReasons?: Record<string, string>;
  onAttach: (assetRefs: string[]) => void;
  onClose: () => void;
};

export function AttachAssetModal({
  assets,
  attachedRefs,
  unsupportedReasons = {},
  onAttach,
  onClose,
}: AttachAssetModalProps) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const [assetType, setAssetType] = useState<AttachTypeFilter>("all");
  const [selectedRefs, setSelectedRefs] = useState<string[]>([]);
  const attachedRefSet = useMemo(() => new Set(attachedRefs), [attachedRefs]);
  const availableAssets = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    return assets.filter((asset) => {
      if (attachedRefSet.has(assetRefForSummary(asset))) {
        return false;
      }
      if (assetType !== "all" && asset.assetType !== assetType) {
        return false;
      }
      if (!normalizedQuery) {
        return true;
      }
      return (
        asset.id.toLowerCase().includes(normalizedQuery) ||
        asset.name.toLowerCase().includes(normalizedQuery) ||
        asset.description?.toLowerCase().includes(normalizedQuery) ||
        asset.tags.some((tag) => tag.toLowerCase().includes(normalizedQuery))
      );
    });
  }, [assetType, assets, attachedRefSet, query]);

  function toggleAsset(asset: AssetSummary) {
    const assetRef = assetRefForSummary(asset);
    if (unsupportedReasons[assetRef]) {
      return;
    }
    setSelectedRefs((current) =>
      current.includes(assetRef)
        ? current.filter((selectedRef) => selectedRef !== assetRef)
        : [...current, assetRef],
    );
  }

  return (
    <div className="modal-backdrop" role="presentation">
      <section className="modal-panel" role="dialog" aria-modal="true" aria-labelledby="attach-asset-title">
        <header className="modal-header">
          <h3 id="attach-asset-title">{t("projects.attachTitle")}</h3>
          <button className="secondary-action" type="button" onClick={onClose}>
            {t("common.close")}
          </button>
        </header>

        <label className="field compact-field" htmlFor="attach-search">
          <span>{t("common.search")}</span>
          <input
            id="attach-search"
            className="field-input"
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
        </label>

        <div className="segmented-control" aria-label={t("projects.attachTitle")}>
          {(["all", "prompt", "skill", "playbook", "instruction-rule", "command-rule"] as AttachTypeFilter[]).map((filter) => (
            <button
              className={filter === assetType ? "segment active" : "segment"}
              key={filter}
              type="button"
              onClick={() => setAssetType(filter)}
            >
              {filter === "all" ? t("common.all") : assetTypeLabel(filter, t)}
            </button>
          ))}
        </div>

        <div className="attach-asset-grid">
          {availableAssets.length === 0 ? (
            <EmptyState title={t("projects.noAvailableTitle")} message={t("projects.noAvailableMessage")} />
          ) : (
            availableAssets.map((asset) => (
              <AssetCard
                active={selectedRefs.includes(assetRefForSummary(asset))}
                asset={asset}
                key={assetRefForSummary(asset)}
                disabled={Boolean(unsupportedReasons[assetRefForSummary(asset)])}
                disabledReason={unsupportedReasons[assetRefForSummary(asset)]}
                onSelect={toggleAsset}
              />
            ))
          )}
        </div>

        <div className="button-row modal-actions">
          <button className="secondary-action" type="button" onClick={onClose}>
            {t("common.cancel")}
          </button>
          <button
            className="primary-action"
            type="button"
            disabled={selectedRefs.length === 0}
            onClick={() => onAttach(selectedRefs)}
          >
            {t("projects.attachSelected")}
          </button>
        </div>
      </section>
    </div>
  );
}

function assetTypeLabel(assetType: AssetType, t: ReturnType<typeof useI18n>["t"]): string {
  if (assetType === "prompt") {
    return t("assets.prompt");
  }
  if (assetType === "skill") {
    return t("assets.skill");
  }
  if (assetType === "playbook") {
    return t("assets.playbook");
  }
  if (assetType === "instruction-rule") {
    return t("assets.instructionRule");
  }
  return t("assets.commandRule");
}

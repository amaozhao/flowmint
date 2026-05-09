import type { AssetSummary, AssetType } from "../api/assets";
import type { SkillTemplateKind } from "../api/templates";
import { useI18n } from "../i18n/I18nProvider";
import type { TranslationKey } from "../i18n/messages";
import type { EditableAssetType } from "../pages/assetEditorModel";
import { EmptyState } from "./EmptyState";
import { AssetCard } from "./AssetCard";

export type AssetTypeFilter = "all" | AssetType;

type AssetListProps = {
  assets: AssetSummary[];
  loading: boolean;
  query: string;
  assetType: AssetTypeFilter;
  tagFilter: string;
  availableTags: string[];
  selectedAssetRef: string | null;
  onQueryChange: (value: string) => void;
  onAssetTypeChange: (value: AssetTypeFilter) => void;
  onTagFilterChange: (value: string) => void;
  onSelect: (asset: AssetSummary) => void;
  onCreate: (assetType: EditableAssetType, templateKind?: SkillTemplateKind) => void;
};

const typeFilters: Array<{ label: TranslationKey; value: AssetTypeFilter }> = [
  { label: "common.all", value: "all" },
  { label: "common.prompts", value: "prompt" },
  { label: "common.skills", value: "skill" },
  { label: "common.playbooks", value: "playbook" },
  { label: "common.instructionRules", value: "instruction-rule" },
  { label: "common.commandRules", value: "command-rule" },
];

export function AssetList({
  assets,
  loading,
  query,
  assetType,
  tagFilter,
  availableTags,
  selectedAssetRef,
  onQueryChange,
  onAssetTypeChange,
  onTagFilterChange,
  onSelect,
  onCreate,
}: AssetListProps) {
  const { t } = useI18n();

  return (
    <aside className="asset-list-panel" aria-label={t("nav.assets")}>
      <div className="asset-list-header">
        <div>
          <h3>{t("nav.assets")}</h3>
          <p>{loading ? t("common.loading") : t(assets.length === 1 ? "counts.item" : "counts.items", { count: assets.length })}</p>
        </div>
        <div className="compact-actions">
          <button className="secondary-action" type="button" onClick={() => onCreate("prompt")}>
            {t("assets.prompt")}
          </button>
          <button className="secondary-action" type="button" onClick={() => onCreate("skill")}>
            {t("assets.skill")}
          </button>
          <button className="secondary-action" type="button" onClick={() => onCreate("playbook")}>
            {t("assets.playbook")}
          </button>
          <button className="secondary-action" type="button" onClick={() => onCreate("instruction-rule")}>
            {t("assets.instructionRule")}
          </button>
          <button className="secondary-action" type="button" onClick={() => onCreate("command-rule")}>
            {t("assets.commandRule")}
          </button>
        </div>
      </div>

      <label className="field compact-field" htmlFor="asset-search">
        <span>{t("common.search")}</span>
        <input
          id="asset-search"
          className="field-input"
          type="search"
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
        />
      </label>

      <div className="segmented-control" aria-label={t("nav.assets")}>
        {typeFilters.map((filter) => (
          <button
            className={filter.value === assetType ? "segment active" : "segment"}
            key={filter.value}
            type="button"
            onClick={() => onAssetTypeChange(filter.value)}
          >
            {t(filter.label)}
          </button>
        ))}
      </div>

      <label className="field compact-field" htmlFor="asset-tag-filter">
        <span>{t("common.tag")}</span>
        <select
          id="asset-tag-filter"
          className="field-input"
          value={tagFilter}
          onChange={(event) => onTagFilterChange(event.target.value)}
        >
          <option value="all">{t("common.allTags")}</option>
          {availableTags.map((tag) => (
            <option key={tag} value={tag}>
              {tag}
            </option>
          ))}
        </select>
      </label>

      <div className="asset-card-list">
        {assets.length === 0 ? (
          <EmptyState title={t("assets.listEmptyTitle")} message={t("assets.listEmptyMessage")} />
        ) : (
          assets.map((asset) => (
            <AssetCard
              active={`${asset.assetType}:${asset.id}` === selectedAssetRef}
              asset={asset}
              key={`${asset.assetType}:${asset.id}`}
              onSelect={onSelect}
            />
          ))
        )}
      </div>
    </aside>
  );
}

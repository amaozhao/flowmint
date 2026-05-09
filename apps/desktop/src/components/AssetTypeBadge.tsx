import type { AssetType } from "../api/assets";
import { useI18n } from "../i18n/I18nProvider";

type AssetTypeBadgeProps = {
  assetType: AssetType;
};

export function AssetTypeBadge({ assetType }: AssetTypeBadgeProps) {
  const { t } = useI18n();
  const label =
    assetType === "prompt"
      ? t("assets.prompt")
      : assetType === "skill"
        ? t("assets.skill")
        : assetType === "playbook"
          ? t("assets.playbook")
          : assetType === "instruction-rule"
            ? t("assets.instructionRule")
            : t("assets.commandRule");
  return <span className={`asset-type-badge ${assetType}`}>{label}</span>;
}

import type { AssetEditorDraft } from "../pages/assetEditorModel";
import { useI18n } from "../i18n/I18nProvider";
import { TagInput } from "./TagInput";

type MetadataFormProps = {
  draft: AssetEditorDraft;
  idLocked: boolean;
  onChange: (draft: AssetEditorDraft) => void;
};

export function MetadataForm({ draft, idLocked, onChange }: MetadataFormProps) {
  const { t } = useI18n();

  return (
    <section className="form-section" aria-label="Asset metadata">
      <div className="form-grid">
        <label className="field" htmlFor="asset-id">
          <span>ID</span>
          <input
            id="asset-id"
            className="field-input"
            type="text"
            value={draft.id}
            disabled={idLocked}
            onChange={(event) => onChange({ ...draft, id: event.target.value })}
          />
        </label>

        <label className="field" htmlFor="asset-name">
          <span>{t("common.name")}</span>
          <input
            id="asset-name"
            className="field-input"
            type="text"
            value={draft.name}
            onChange={(event) => onChange({ ...draft, name: event.target.value })}
          />
        </label>
      </div>

      <label className="field" htmlFor="asset-description">
        <span>{t("common.description")}</span>
        <textarea
          id="asset-description"
          className="field-input text-area"
          rows={3}
          value={draft.description}
          onChange={(event) => onChange({ ...draft, description: event.target.value })}
        />
      </label>

      <TagInput
        id="asset-tags"
        label={t("common.tag")}
        value={draft.tags}
        onChange={(tags) => onChange({ ...draft, tags })}
      />
    </section>
  );
}

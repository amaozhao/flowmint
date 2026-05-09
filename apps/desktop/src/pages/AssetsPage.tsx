import { useEffect, useMemo, useState } from "react";
import {
  assetRefForDetail,
  assetRefForSummary,
  createAsset,
  deleteAsset,
  getAsset,
  listAssets,
  openAssetFolder,
  promoteSkillToPlaybook,
  updateAsset,
  type EditableAssetDetail,
  type AssetSummary,
} from "../api/assets";
import { getSkillTemplate, type SkillTemplateKind } from "../api/templates";
import { AssetList, type AssetTypeFilter } from "../components/AssetList";
import { EmptyState } from "../components/EmptyState";
import { useI18n } from "../i18n/I18nProvider";
import { AssetEditorPage } from "./AssetEditorPage";
import {
  buildDraftFromSkillTemplate,
  type AssetEditorDraft,
  type EditableAssetType,
} from "./assetEditorModel";

type EditorState =
  | { mode: "empty" }
  | { mode: "create"; assetType: EditableAssetType; initialDraft: AssetEditorDraft | null }
  | { mode: "edit"; assetRef: string; assetType: EditableAssetType; assetDetail: EditableAssetDetail };

export function AssetsPage() {
  const { t } = useI18n();
  const [assets, setAssets] = useState<AssetSummary[]>([]);
  const [query, setQuery] = useState("");
  const [assetType, setAssetType] = useState<AssetTypeFilter>("all");
  const [tagFilter, setTagFilter] = useState("all");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editorState, setEditorState] = useState<EditorState>({ mode: "empty" });

  const selectedAssetRef = editorState.mode === "edit" ? editorState.assetRef : null;

  const filter = useMemo(
    () => ({
      assetType: assetType === "all" ? null : assetType,
      query: query.trim() || null,
    }),
    [assetType, query],
  );
  const availableTags = useMemo(
    () =>
      Array.from(new Set(assets.flatMap((asset) => asset.tags)))
        .filter(Boolean)
        .sort((left, right) => left.localeCompare(right)),
    [assets],
  );
  const visibleAssets = useMemo(
    () =>
      assets.filter((asset) => {
        return tagFilter === "all" || asset.tags.includes(tagFilter);
      }),
    [assets, tagFilter],
  );

  async function reloadAssets() {
    setLoading(true);
    try {
      setAssets(await listAssets(filter));
      setError(null);
    } catch (loadError) {
      setError(messageFromError(loadError));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void reloadAssets();
  }, [filter]);

  async function handleSelect(asset: AssetSummary) {
    const assetRef = assetRefForSummary(asset);
    setError(null);
    try {
      const assetDetail = await getAsset(assetRef);
      setEditorState({
        mode: "edit",
        assetRef,
        assetType: assetDetail.assetType,
        assetDetail,
      });
    } catch (selectError) {
      setError(messageFromError(selectError));
    }
  }

  async function handleCreate(nextAssetType: EditableAssetType, templateKind?: SkillTemplateKind) {
    setError(null);
    if (nextAssetType === "skill" && templateKind) {
      setSaving(true);
      try {
        const template = await getSkillTemplate(templateKind);
        setEditorState({
          mode: "create",
          assetType: "skill",
          initialDraft: buildDraftFromSkillTemplate(template),
        });
      } catch (templateError) {
        setError(messageFromError(templateError));
      } finally {
        setSaving(false);
      }
      return;
    }

    setEditorState({ mode: "create", assetType: nextAssetType, initialDraft: null });
  }

  async function handleSave(asset: EditableAssetDetail) {
    setSaving(true);
    setError(null);
    try {
      const savedAsset =
        editorState.mode === "create"
          ? await createAsset({ asset })
          : await updateAsset({ asset });
      const assetRef = assetRefForDetail(savedAsset);
      setEditorState({
        mode: "edit",
        assetRef,
        assetType: savedAsset.assetType,
        assetDetail: savedAsset,
      });
      await reloadAssets();
    } catch (saveError) {
      setError(messageFromError(saveError));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    if (editorState.mode !== "edit") {
      return;
    }

    const confirmed = window.confirm(t("assets.deleteConfirm", { assetRef: editorState.assetRef }));
    if (!confirmed) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      await deleteAsset(editorState.assetRef);
      setEditorState({ mode: "empty" });
      await reloadAssets();
    } catch (deleteError) {
      setError(messageFromError(deleteError));
    } finally {
      setSaving(false);
    }
  }

  async function handleOpenFolder() {
    if (editorState.mode !== "edit") {
      return;
    }

    setError(null);
    try {
      await openAssetFolder(editorState.assetRef);
    } catch (openError) {
      setError(messageFromError(openError));
    }
  }

  async function handlePromoteSkill(skillId: string) {
    const playbookId = window.prompt(t("assets.promotePlaybookPrompt"), `${skillId}-playbook`);
    if (!playbookId) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const promoted = await promoteSkillToPlaybook(skillId, playbookId.trim());
      if (promoted.assetType !== "playbook") {
        throw new Error(t("assets.promoteUnexpected"));
      }
      setEditorState({
        mode: "edit",
        assetRef: assetRefForDetail(promoted),
        assetType: "playbook",
        assetDetail: promoted,
      });
      await reloadAssets();
    } catch (promoteError) {
      setError(messageFromError(promoteError));
    } finally {
      setSaving(false);
    }
  }

  return (
    <section className="assets-page">
      <AssetList
        assets={visibleAssets}
        loading={loading}
        query={query}
        assetType={assetType}
        tagFilter={tagFilter}
        availableTags={availableTags}
        selectedAssetRef={selectedAssetRef}
        onQueryChange={setQuery}
        onAssetTypeChange={setAssetType}
        onTagFilterChange={setTagFilter}
        onSelect={(asset) => void handleSelect(asset)}
        onCreate={(nextAssetType, templateKind) => void handleCreate(nextAssetType, templateKind)}
      />

      <div className="asset-workspace">
        {editorState.mode === "empty" ? (
          <EmptyState
            title={t("assets.noSelectionTitle")}
            message={t("assets.noSelectionMessage")}
            action={
              <>
                <button className="primary-action" type="button" onClick={() => void handleCreate("prompt")}>
                  {t("dashboard.newPrompt")}
                </button>
                <button className="secondary-action" type="button" onClick={() => void handleCreate("skill")}>
                  {t("dashboard.newSkill")}
                </button>
                <button
                  className="secondary-action"
                  type="button"
                  onClick={() => void handleCreate("playbook")}
                >
                  {t("assets.newPlaybook")}
                </button>
                <button className="secondary-action" type="button" onClick={() => void handleCreate("instruction-rule")}>
                  {t("assets.newInstructionRule")}
                </button>
                <button className="secondary-action" type="button" onClick={() => void handleCreate("command-rule")}>
                  {t("assets.newCommandRule")}
                </button>
              </>
            }
          />
        ) : (
          <AssetEditorPage
            mode={editorState.mode}
            assetType={editorState.assetType}
            assetDetail={editorState.mode === "edit" ? editorState.assetDetail : null}
            initialDraft={editorState.mode === "create" ? editorState.initialDraft : null}
            saving={saving}
            error={error}
            onSave={handleSave}
            onCancel={() => setEditorState({ mode: "empty" })}
            onDelete={editorState.mode === "edit" ? handleDelete : null}
            onOpenFolder={editorState.mode === "edit" ? handleOpenFolder : null}
            onPromoteSkill={editorState.mode === "edit" ? handlePromoteSkill : null}
          />
        )}
      </div>
    </section>
  );
}

function messageFromError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

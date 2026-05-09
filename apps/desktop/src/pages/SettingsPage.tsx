import { useEffect, useMemo, useState } from "react";
import { listAssets, type AssetSummary } from "../api/assets";
import {
  exportDebugReport,
  rebuildIndex,
  type AppState,
  type IndexSummary,
} from "../api/settings";
import {
  attachGlobalProfileAsset,
  detachGlobalProfileAsset,
  listTargetCapabilities,
  listGlobalSyncProfiles,
  type GlobalSyncProfiles,
  type TargetCapabilities,
} from "../api/sync";
import { AttachedAssetList } from "../components/AttachedAssetList";
import { AttachAssetModal } from "../components/AttachAssetModal";
import { useI18n } from "../i18n/I18nProvider";
import { attachedAssetsForProfile } from "../utils/attachedAssets";
import { unsupportedReasonsForAssets } from "../utils/targetCapabilities";

type SettingsPageProps = {
  appState: AppState;
  onOpenLibrary: () => void;
  onReload: () => void;
};

export function SettingsPage({ appState, onOpenLibrary, onReload }: SettingsPageProps) {
  const { t } = useI18n();
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [indexSummary, setIndexSummary] = useState<IndexSummary | null>(null);
  const [assets, setAssets] = useState<AssetSummary[]>([]);
  const [globalProfiles, setGlobalProfiles] = useState<GlobalSyncProfiles>({ profiles: [] });
  const [targetCapabilities, setTargetCapabilities] = useState<TargetCapabilities[]>([]);
  const [selectedGlobalTarget, setSelectedGlobalTarget] = useState("claude-code");
  const [globalAttachModalOpen, setGlobalAttachModalOpen] = useState(false);
  const [debugReportPath, setDebugReportPath] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const selectedGlobalProfile = useMemo(
    () =>
      globalProfiles.profiles.find(
        (profile) => profile.target === selectedGlobalTarget && profile.scope === "global-user",
      ) ?? null,
    [globalProfiles.profiles, selectedGlobalTarget],
  );
  const selectedGlobalAssets = useMemo(
    () => attachedAssetsForProfile(selectedGlobalProfile, assets),
    [assets, selectedGlobalProfile],
  );
  const globalUnsupportedReasons = useMemo(
    () => unsupportedReasonsForAssets(targetCapabilities, selectedGlobalTarget, "global-user", assets),
    [assets, selectedGlobalTarget, targetCapabilities],
  );

  async function reloadGlobalProfiles() {
    const [nextAssets, nextProfiles, nextTargetCapabilities] = await Promise.all([
      listAssets({ assetType: null, query: null }),
      listGlobalSyncProfiles(),
      listTargetCapabilities(),
    ]);
    setAssets(nextAssets);
    setGlobalProfiles(nextProfiles);
    setTargetCapabilities(nextTargetCapabilities);
  }

  useEffect(() => {
    void reloadGlobalProfiles().catch((loadError) => setError(messageFromError(loadError)));
  }, []);

  async function handleRebuildIndex() {
    setError(null);
    try {
      const summary = await rebuildIndex();
      setIndexSummary(summary);
      setStatus(t("settings.indexRebuilt"));
    } catch (rebuildError) {
      setError(messageFromError(rebuildError));
    }
  }

  async function handleExportDebugReport() {
    setError(null);
    try {
      const path = await exportDebugReport();
      setDebugReportPath(path);
      setStatus(t("settings.debugExported"));
    } catch (exportError) {
      setError(messageFromError(exportError));
    }
  }

  async function handleAttachGlobal(assetRefs: string[]) {
    setError(null);
    try {
      let profiles = globalProfiles;
      for (const assetRef of assetRefs) {
        profiles = await attachGlobalProfileAsset(selectedGlobalTarget, assetRef);
      }
      setGlobalProfiles(profiles);
      setGlobalAttachModalOpen(false);
      setStatus(t("settings.globalProfilesSaved"));
    } catch (attachError) {
      setError(messageFromError(attachError));
    }
  }

  async function handleDetachGlobal(assetRef: string) {
    setError(null);
    try {
      setGlobalProfiles(await detachGlobalProfileAsset(selectedGlobalTarget, assetRef));
      setStatus(t("settings.globalProfilesSaved"));
    } catch (detachError) {
      setError(messageFromError(detachError));
    }
  }

  return (
    <section className="settings-stack">
      {error ? (
        <div className="validation-panel invalid" role="alert">
          <p>{error}</p>
        </div>
      ) : null}

      {status ? (
        <div className="validation-panel valid">
          <p>{status}</p>
        </div>
      ) : null}

      <section className="panel">
        <h3>{t("settings.library")}</h3>
        <dl className="detail-list">
          <div>
            <dt>{t("common.path")}</dt>
            <dd>{appState.library.path}</dd>
          </div>
          <div>
            <dt>{t("common.status")}</dt>
            <dd>{appState.library.initialized ? t("common.initialized") : t("common.missing")}</dd>
          </div>
        </dl>

        <div className="button-row">
          <button className="secondary-action" type="button" onClick={onOpenLibrary}>
            {t("settings.openLibrary")}
          </button>
          <button className="secondary-action" type="button" onClick={onReload}>
            {t("common.refresh")}
          </button>
        </div>
      </section>

      <section className="panel">
        <h3>{t("settings.defaults")}</h3>
        <dl className="detail-list">
          <div>
            <dt>{t("settings.defaultExporter")}</dt>
            <dd>Claude Code</dd>
          </div>
          <div>
            <dt>{t("settings.externalEditor")}</dt>
            <dd>{t("settings.systemDefault")}</dd>
          </div>
          <div>
            <dt>{t("settings.advancedFeatures")}</dt>
            <dd>
              <label className="toggle-field">
                <input
                  type="checkbox"
                  checked={showAdvanced}
                  onChange={(event) => setShowAdvanced(event.target.checked)}
                />
                <span>{showAdvanced ? t("settings.shown") : t("settings.hidden")}</span>
              </label>
            </dd>
          </div>
        </dl>
      </section>

      <section className="panel">
        <div className="section-heading">
          <h3>{t("settings.globalProfiles")}</h3>
          <span className="muted-text">{t("sync.scopeGlobal")}</span>
        </div>
        <div className="form-grid">
          <label className="field compact-field" htmlFor="global-profile-target">
            <span>{t("sync.target")}</span>
            <select
              id="global-profile-target"
              className="field-input"
              value={selectedGlobalTarget}
              onChange={(event) => setSelectedGlobalTarget(event.target.value)}
            >
              <option value="claude-code">Claude Code</option>
              <option value="codex">Codex</option>
              <option value="gemini-cli">Gemini CLI</option>
            </select>
          </label>
        </div>
        <div className="button-row">
          <button className="secondary-action" type="button" onClick={() => setGlobalAttachModalOpen(true)}>
            {t("settings.attachGlobalAsset")}
          </button>
          <button
            className="secondary-action"
            type="button"
            onClick={() => void reloadGlobalProfiles().catch((loadError) => setError(messageFromError(loadError)))}
          >
            {t("common.refresh")}
          </button>
        </div>
        <section className="form-section">
          <div className="section-heading">
            <h4>{t("projects.attachedAssets")}</h4>
            <span className="muted-text">{selectedGlobalAssets.length}</span>
          </div>
          <AttachedAssetList assets={selectedGlobalAssets} onDetach={(assetRef) => void handleDetachGlobal(assetRef)} />
        </section>
      </section>

      {globalAttachModalOpen ? (
        <AttachAssetModal
          assets={assets}
          attachedRefs={selectedGlobalAssets.map((asset) => asset.assetRef)}
          unsupportedReasons={globalUnsupportedReasons}
          onAttach={(assetRefs) => void handleAttachGlobal(assetRefs)}
          onClose={() => setGlobalAttachModalOpen(false)}
        />
      ) : null}

      <section className="panel">
        <h3>{t("settings.maintenance")}</h3>
        <div className="button-row">
          <button className="secondary-action" type="button" onClick={() => void handleRebuildIndex()}>
            {t("settings.rebuildIndex")}
          </button>
          <button className="secondary-action" type="button" onClick={() => void handleExportDebugReport()}>
            {t("settings.exportDebugReport")}
          </button>
        </div>

        {indexSummary ? (
          <dl className="detail-list">
            <div>
              <dt>{t("common.prompts")}</dt>
              <dd>{indexSummary.promptCount}</dd>
            </div>
            <div>
              <dt>{t("common.skills")}</dt>
              <dd>{indexSummary.skillCount}</dd>
            </div>
            <div>
              <dt>{t("settings.playbookSkills")}</dt>
              <dd>{indexSummary.playbookSkillCount}</dd>
            </div>
            <div>
              <dt>{t("common.projects")}</dt>
              <dd>{indexSummary.projectCount}</dd>
            </div>
          </dl>
        ) : null}

        {debugReportPath ? <p className="muted-text">{debugReportPath}</p> : null}
      </section>
    </section>
  );
}

function messageFromError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

import { useEffect, useMemo, useState } from "react";
import type { AssetSummary } from "../api/assets";
import type { SyncScope, TargetCapabilities } from "../api/sync";
import type { ProjectDetail } from "../api/projects";
import { AttachedAssetList } from "../components/AttachedAssetList";
import { AttachAssetModal } from "../components/AttachAssetModal";
import { useI18n } from "../i18n/I18nProvider";
import { attachedAssetsForProfile } from "../utils/attachedAssets";
import { unsupportedReasonsForAssets } from "../utils/targetCapabilities";

type ProjectDetailPageProps = {
  project: ProjectDetail;
  assets: AssetSummary[];
  targetCapabilities: TargetCapabilities[];
  attachModalOpen: boolean;
  onOpenAttachModal: () => void;
  onCloseAttachModal: () => void;
  onAttach: (assetRefs: string[], target: string, scope: SyncScope) => void;
  onDetach: (assetRef: string, target: string, scope: SyncScope) => void;
  onPreviewSync: (projectPath: string, target: string, scope: SyncScope) => void;
};

export function ProjectDetailPage({
  project,
  assets,
  targetCapabilities,
  attachModalOpen,
  onOpenAttachModal,
  onCloseAttachModal,
  onAttach,
  onDetach,
  onPreviewSync,
}: ProjectDetailPageProps) {
  const { t } = useI18n();
  const [selectedTarget, setSelectedTarget] = useState(project.manifest.export.target || "claude-code");
  const selectedScope: SyncScope = "project";
  const targetOptions = ["claude-code", "codex", "gemini-cli"];
  const selectedProfile = useMemo(
    () =>
      project.manifest.exports.find(
        (profile) => profile.target === selectedTarget && profile.scope === selectedScope,
      ) ?? null,
    [project.manifest.exports, selectedScope, selectedTarget],
  );
  const selectedProfileAssets = useMemo(
    () => attachedAssetsForProfile(selectedProfile, assets),
    [assets, selectedProfile],
  );
  const selectedProfileAttachedRefs = selectedProfileAssets.map((asset) => asset.assetRef);
  const unsupportedReasons = useMemo(
    () => unsupportedReasonsForAssets(targetCapabilities, selectedTarget, selectedScope, assets),
    [assets, selectedScope, selectedTarget, targetCapabilities],
  );

  useEffect(() => {
    setSelectedTarget(project.manifest.export.target || "claude-code");
  }, [project.path, project.manifest.export.target]);

  return (
    <section className="project-detail">
      <header className="project-detail-header">
        <div>
          <h3>{project.manifest.project.name}</h3>
          <p className="project-path">{project.path}</p>
        </div>
        <div className="button-row inline-actions">
          <button className="secondary-action" type="button" onClick={onOpenAttachModal}>
            {t("projects.attachAsset")}
          </button>
          <button
            className="primary-action"
            type="button"
            onClick={() => onPreviewSync(project.path, selectedTarget, selectedScope)}
          >
            {t("projects.previewSync")}
          </button>
        </div>
      </header>

      <section className="project-summary-grid" aria-label="Project summary">
        <article className="metric-card compact-metric">
          <span>{t("projects.export")}</span>
          <strong>{targetLabel(selectedTarget)}</strong>
        </article>
        <article className="metric-card compact-metric">
          <span>{t("common.prompts")}</span>
          <strong>{selectedProfile?.prompts.length ?? 0}</strong>
        </article>
        <article className="metric-card compact-metric">
          <span>{t("common.skills")}</span>
          <strong>{selectedProfile?.skills.length ?? 0}</strong>
        </article>
        <article className="metric-card compact-metric">
          <span>{t("common.playbooks")}</span>
          <strong>{selectedProfile?.playbooks.length ?? 0}</strong>
        </article>
        <article className="metric-card compact-metric">
          <span>{t("common.rules")}</span>
          <strong>
            {(selectedProfile?.instructionRules.length ?? 0) + (selectedProfile?.commandRules.length ?? 0)}
          </strong>
        </article>
      </section>

      <section className="panel profile-panel">
        <div className="section-heading">
          <h4>{t("projects.profile")}</h4>
          <span className="muted-text">{t("sync.scopeProject")}</span>
        </div>
        <div className="form-grid">
          <label className="field compact-field" htmlFor="project-profile-target">
            <span>{t("sync.target")}</span>
            <select
              id="project-profile-target"
              className="field-input"
              value={selectedTarget}
              onChange={(event) => setSelectedTarget(event.target.value)}
            >
              {targetOptions.map((target) => (
                <option key={target} value={target}>
                  {targetLabel(target)}
                </option>
              ))}
            </select>
          </label>
          <label className="field compact-field" htmlFor="project-profile-scope">
            <span>{t("sync.scope")}</span>
            <input
              id="project-profile-scope"
              className="field-input"
              type="text"
              value={t("sync.scopeProject")}
              disabled
            />
          </label>
        </div>
      </section>

      <section className="form-section">
        <div className="section-heading">
          <h4>{t("projects.attachedAssets")}</h4>
          <span className="muted-text">{selectedProfileAssets.length}</span>
        </div>
        <AttachedAssetList
          assets={selectedProfileAssets}
          onDetach={(assetRef) => onDetach(assetRef, selectedTarget, selectedScope)}
        />
      </section>

      {attachModalOpen ? (
        <AttachAssetModal
          assets={assets}
          attachedRefs={selectedProfileAttachedRefs}
          unsupportedReasons={unsupportedReasons}
          onAttach={(assetRefs) => onAttach(assetRefs, selectedTarget, selectedScope)}
          onClose={onCloseAttachModal}
        />
      ) : null}
    </section>
  );
}

function targetLabel(target: string): string {
  if (target === "claude-code") {
    return "Claude Code";
  }
  if (target === "gemini-cli") {
    return "Gemini CLI";
  }
  return "Codex";
}

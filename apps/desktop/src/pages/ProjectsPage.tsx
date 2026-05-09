import { useEffect, useMemo, useState } from "react";
import { listAssets, type AssetSummary } from "../api/assets";
import {
  addProject,
  attachAssetToProfile,
  detachAssetFromProfile,
  getProject,
  listProjects,
  type ProjectDetail,
  type ProjectSummary,
} from "../api/projects";
import { pickDirectory } from "../api/settings";
import { listTargetCapabilities, type SyncScope, type TargetCapabilities } from "../api/sync";
import { EmptyState } from "../components/EmptyState";
import { ProjectList } from "../components/ProjectList";
import { useI18n } from "../i18n/I18nProvider";
import { ProjectDetailPage } from "./ProjectDetailPage";

type ProjectsPageProps = {
  onPreviewSync: (projectPath: string, target: string, scope: SyncScope) => void;
};

export function ProjectsPage({ onPreviewSync }: ProjectsPageProps) {
  const { t } = useI18n();
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [assets, setAssets] = useState<AssetSummary[]>([]);
  const [targetCapabilities, setTargetCapabilities] = useState<TargetCapabilities[]>([]);
  const [selectedProject, setSelectedProject] = useState<ProjectDetail | null>(null);
  const [pathValue, setPathValue] = useState("");
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [attachModalOpen, setAttachModalOpen] = useState(false);

  const attachableAssets = useMemo(
    () => assets,
    [assets],
  );

  async function reloadProjects() {
    setLoading(true);
    try {
      const [nextProjects, nextAssets, nextTargetCapabilities] = await Promise.all([
        listProjects(),
        listAssets({ assetType: null, query: null }),
        listTargetCapabilities(),
      ]);
      setProjects(nextProjects);
      setAssets(nextAssets);
      setTargetCapabilities(nextTargetCapabilities);
      setError(null);
    } catch (loadError) {
      setError(messageFromError(loadError));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void reloadProjects();
  }, []);

  async function handleAddProject() {
    const path = pathValue.trim();
    if (!path) {
      setError(t("validation.projectPathRequired"));
      return;
    }

    setBusy(true);
    setError(null);
    try {
      const project = await addProject(path);
      setSelectedProject(project);
      setPathValue("");
      await reloadProjects();
    } catch (addError) {
      setError(messageFromError(addError));
    } finally {
      setBusy(false);
    }
  }

  async function handlePickProjectPath() {
    setBusy(true);
    setError(null);
    try {
      const selectedPath = await pickDirectory();
      if (selectedPath) {
        setPathValue(selectedPath);
      }
    } catch (pickError) {
      setError(messageFromError(pickError));
    } finally {
      setBusy(false);
    }
  }

  async function handleSelectProject(project: ProjectSummary) {
    setBusy(true);
    setError(null);
    try {
      setSelectedProject(await getProject(project.path));
      setAttachModalOpen(false);
    } catch (selectError) {
      setError(messageFromError(selectError));
    } finally {
      setBusy(false);
    }
  }

  async function handleAttach(assetRefs: string[], target = "claude-code", scope: SyncScope = "project") {
    if (!selectedProject) {
      return;
    }

    setBusy(true);
    setError(null);
    try {
      let project = selectedProject;
      for (const assetRef of assetRefs) {
        project = await attachAssetToProfile(project.path, target, scope, assetRef);
      }
      setSelectedProject(project);
      setAttachModalOpen(false);
      await reloadProjects();
    } catch (attachError) {
      setError(messageFromError(attachError));
    } finally {
      setBusy(false);
    }
  }

  async function handleDetach(assetRef: string, target = "claude-code", scope: SyncScope = "project") {
    if (!selectedProject) {
      return;
    }

    setBusy(true);
    setError(null);
    try {
      const project = await detachAssetFromProfile(selectedProject.path, target, scope, assetRef);
      setSelectedProject(project);
      await reloadProjects();
    } catch (detachError) {
      setError(messageFromError(detachError));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="projects-page">
      <ProjectList
        projects={projects}
        selectedPath={selectedProject?.path ?? null}
        pathValue={pathValue}
        loading={loading || busy}
        onPathChange={setPathValue}
        onAddProject={() => void handleAddProject()}
        onPickProjectPath={() => void handlePickProjectPath()}
        onSelect={(project) => void handleSelectProject(project)}
      />

      <div className="project-workspace">
        {error ? (
          <div className="validation-panel invalid" role="alert">
            <p>{error}</p>
          </div>
        ) : null}

        {selectedProject ? (
          <ProjectDetailPage
            project={selectedProject}
            assets={attachableAssets}
            targetCapabilities={targetCapabilities}
            attachModalOpen={attachModalOpen}
            onOpenAttachModal={() => setAttachModalOpen(true)}
            onCloseAttachModal={() => setAttachModalOpen(false)}
            onAttach={(assetRefs, target, scope) => void handleAttach(assetRefs, target, scope)}
            onDetach={(assetRef, target, scope) => void handleDetach(assetRef, target, scope)}
            onPreviewSync={onPreviewSync}
          />
        ) : (
          <EmptyState title={t("projects.noSelectionTitle")} message={t("projects.noSelectionMessage")} />
        )}
      </div>
    </section>
  );
}

function messageFromError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

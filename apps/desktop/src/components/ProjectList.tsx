import type { ProjectSummary } from "../api/projects";
import { useI18n } from "../i18n/I18nProvider";
import { EmptyState } from "./EmptyState";

type ProjectListProps = {
  projects: ProjectSummary[];
  selectedPath: string | null;
  pathValue: string;
  loading: boolean;
  onPathChange: (value: string) => void;
  onAddProject: () => void;
  onPickProjectPath: () => void;
  onSelect: (project: ProjectSummary) => void;
};

export function ProjectList({
  projects,
  selectedPath,
  pathValue,
  loading,
  onPathChange,
  onAddProject,
  onPickProjectPath,
  onSelect,
}: ProjectListProps) {
  const { t } = useI18n();

  return (
    <aside className="project-list-panel" aria-label={t("nav.projects")}>
      <div className="project-list-header">
        <div>
          <h3>{t("nav.projects")}</h3>
          <p>{loading ? t("common.loading") : t(projects.length === 1 ? "counts.item" : "counts.items", { count: projects.length })}</p>
        </div>
      </div>

      <div className="project-add-form">
        <label className="field compact-field" htmlFor="project-path">
          <span>{t("projects.projectPath")}</span>
          <input
            id="project-path"
            className="field-input"
            type="text"
            value={pathValue}
            onChange={(event) => onPathChange(event.target.value)}
          />
        </label>
        <div className="button-row compact-actions">
          <button className="secondary-action" type="button" onClick={onPickProjectPath}>
            {t("common.browse")}
          </button>
          <button className="primary-action" type="button" onClick={onAddProject}>
            {t("projects.addProject")}
          </button>
        </div>
      </div>

      <div className="project-card-list">
        {projects.length === 0 ? (
          <EmptyState title={t("projects.noProjectsTitle")} message={t("projects.noProjectsMessage")} />
        ) : (
          projects.map((project) => (
            <button
              className={project.path === selectedPath ? "project-card active" : "project-card"}
              key={project.path}
              type="button"
              onClick={() => onSelect(project)}
              aria-pressed={project.path === selectedPath}
            >
              <span className="project-name">{project.name}</span>
              <span className="project-path">{project.path}</span>
              <span className="project-meta">
                {project.initialized ? t("common.initialized") : t("common.missingManifest")} -{" "}
                {t("dashboard.assetsAttached", { count: project.attachedAssets })}
              </span>
            </button>
          ))
        )}
      </div>
    </aside>
  );
}

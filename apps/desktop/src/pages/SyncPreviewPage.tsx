import { useEffect, useMemo, useState } from "react";
import { listProjects, type ProjectSummary } from "../api/projects";
import {
  acknowledgeGlobalSyncPlan,
  applySync,
  openSyncTarget,
  previewSync,
  type SyncApplyResult,
  type SyncOperation,
  type SyncPlan,
  type SyncScope,
} from "../api/sync";
import { ConflictBanner } from "../components/ConflictBanner";
import { DiffViewer } from "../components/DiffViewer";
import { EmptyState } from "../components/EmptyState";
import { SyncOperationList } from "../components/SyncOperationList";
import { useI18n } from "../i18n/I18nProvider";
import { SyncResultPage } from "./SyncResultPage";

type SyncPreviewPageProps = {
  initialProjectPath?: string;
  initialTarget?: string;
  initialScope?: SyncScope;
};

export function SyncPreviewPage({
  initialProjectPath = "",
  initialTarget = "claude-code",
  initialScope = "project",
}: SyncPreviewPageProps) {
  const { t } = useI18n();
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [projectPath, setProjectPath] = useState("");
  const [target, setTarget] = useState(initialTarget);
  const [scope, setScope] = useState<SyncScope>(initialScope);
  const [plan, setPlan] = useState<SyncPlan | null>(null);
  const [applyResult, setApplyResult] = useState<SyncApplyResult | null>(null);
  const [selectedOperation, setSelectedOperation] = useState<SyncOperation | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const operationCounts = useMemo(() => {
    const counts = {
      creates: 0,
      updates: 0,
      deletes: 0,
      noops: 0,
    };
    for (const operation of plan?.operations ?? []) {
      if (operation.operationType === "create-file" || operation.operationType === "create-dir") {
        counts.creates += 1;
      } else if (operation.operationType === "update-file") {
        counts.updates += 1;
      } else if (operation.operationType === "delete-generated-file") {
        counts.deletes += 1;
      } else {
        counts.noops += 1;
      }
    }
    return counts;
  }, [plan]);
  const mutatingPaths = useMemo(
    () => (plan ? mutatingOperationPaths(plan.operations) : []),
    [plan],
  );

  useEffect(() => {
    async function loadProjects() {
      try {
        const nextProjects = await listProjects();
        setProjects(nextProjects);
        setProjectPath((current) => initialProjectPath || current || nextProjects[0]?.path || "");
        setTarget(initialTarget);
        setScope(initialScope);
        setError(null);
      } catch (loadError) {
        setError(messageFromError(loadError));
      } finally {
        setLoading(false);
      }
    }

    void loadProjects();
  }, [initialProjectPath, initialScope, initialTarget]);

  async function handlePreview() {
    if (!projectPath) {
      setError(t("validation.projectPathRequired"));
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const nextPlan = await previewSync(projectPath, target, scope);
      setPlan(nextPlan);
      setApplyResult(null);
      setSelectedOperation(nextPlan.operations[0] ?? null);
    } catch (previewError) {
      setError(messageFromError(previewError));
    } finally {
      setLoading(false);
    }
  }

  async function handleApply() {
    if (!plan || plan.conflicts.length > 0) {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      if (plan.scope === "global-user") {
        const confirmed = window.confirm(
          t("sync.globalConfirm", { count: mutatingPaths.length, root: plan.projectPath }),
        );
        if (!confirmed) {
          setLoading(false);
          return;
        }
        await acknowledgeGlobalSyncPlan(plan.planId, mutatingPaths);
      }
      const result = await applySync(plan.planId);
      setApplyResult(result);
      setPlan(null);
      setSelectedOperation(null);
      const nextProjects = await listProjects();
      setProjects(nextProjects);
    } catch (applyError) {
      setError(messageFromError(applyError));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section className="sync-preview-page">
      <section className="sync-toolbar">
        <label className="field compact-field" htmlFor="sync-project">
          <span>{t("sync.project")}</span>
          <select
            id="sync-project"
            className="field-input"
            value={projectPath}
            onChange={(event) => setProjectPath(event.target.value)}
          >
            {projects.map((project) => (
              <option key={project.path} value={project.path}>
                {project.name}
              </option>
            ))}
          </select>
        </label>
        <label className="field compact-field" htmlFor="sync-target">
          <span>{t("sync.target")}</span>
          <select
            id="sync-target"
            className="field-input"
            value={target}
            onChange={(event) => setTarget(event.target.value)}
          >
            <option value="claude-code">Claude Code</option>
            <option value="codex">Codex</option>
            <option value="gemini-cli">Gemini CLI</option>
          </select>
        </label>
        <label className="field compact-field" htmlFor="sync-scope">
          <span>{t("sync.scope")}</span>
          <select
            id="sync-scope"
            className="field-input"
            value={scope}
            onChange={(event) => setScope(event.target.value === "global-user" ? "global-user" : "project")}
          >
            <option value="project">{t("sync.scopeProject")}</option>
            <option value="global-user">{t("sync.scopeGlobal")}</option>
          </select>
        </label>
        <button
          className="primary-action"
          type="button"
          disabled={loading}
          onClick={() => void handlePreview()}
        >
          {t("sync.previewSync")}
        </button>
      </section>

      {error ? (
        <div className="validation-panel invalid" role="alert">
          <p>{error}</p>
        </div>
      ) : null}

      {applyResult ? (
        <SyncResultPage
          result={applyResult}
          projectPath={projectPath}
          onOpenProject={() => void openSyncTarget(projectPath)}
        />
      ) : null}

      {plan ? (
        <>
          <section className="project-summary-grid" aria-label="Sync summary">
            <article className="metric-card compact-metric">
              <span>{t("sync.creates")}</span>
              <strong>{operationCounts.creates}</strong>
            </article>
            <article className="metric-card compact-metric">
              <span>{t("sync.updates")}</span>
              <strong>{operationCounts.updates}</strong>
            </article>
            <article className="metric-card compact-metric">
              <span>{t("sync.conflicts")}</span>
              <strong>{plan.conflicts.length}</strong>
            </article>
            <article className="metric-card compact-metric">
              <span>{t("sync.scope")}</span>
              <strong>{plan.scope === "global-user" ? t("sync.scopeGlobal") : t("sync.scopeProject")}</strong>
            </article>
          </section>

          <ConflictBanner
            conflicts={plan.conflicts}
            onCancel={() => setPlan(null)}
            onOpenFile={(path) => void openSyncTarget(path)}
          />

          {plan.scope === "global-user" ? (
            <section className="validation-panel valid">
              <h3>{t("sync.globalNoticeTitle")}</h3>
              <p>{t("sync.globalNoticeMessage")}</p>
              <dl className="detail-list">
                <div>
                  <dt>{t("sync.globalRoot")}</dt>
                  <dd>{plan.projectPath}</dd>
                </div>
                <div>
                  <dt>{t("sync.globalMutatingPaths")}</dt>
                  <dd>{mutatingPaths.length}</dd>
                </div>
              </dl>
              {mutatingPaths.length > 0 ? (
                <ul className="compact-list path-list">
                  {mutatingPaths.map((path) => (
                    <li key={path}>
                      <code>{path}</code>
                    </li>
                  ))}
                </ul>
              ) : null}
            </section>
          ) : null}

          <div className="sync-apply-row">
            <button
              className="primary-action"
              type="button"
              disabled={loading || plan.conflicts.length > 0}
              onClick={() => void handleApply()}
            >
              {t("sync.apply")}
            </button>
          </div>

          <section className="sync-preview-layout">
            <SyncOperationList
              operations={plan.operations}
              selectedTarget={selectedOperation?.targetPath ?? null}
              onSelect={setSelectedOperation}
            />
            <DiffViewer operation={selectedOperation} />
          </section>
        </>
      ) : !applyResult ? (
        <EmptyState title={t("sync.noPlanTitle")} message={t("sync.noPlanMessage")} />
      ) : null}
    </section>
  );
}

function messageFromError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function mutatingOperationPaths(operations: SyncOperation[]): string[] {
  return operations
    .filter((operation) => operation.operationType !== "noop")
    .map((operation) => operation.targetPath);
}

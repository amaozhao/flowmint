import { useEffect, useMemo, useState } from "react";
import {
  applyImportAdoption,
  previewImportAdoption,
  scanImportCandidates,
  type ImportAdoptionMode,
  type ImportAdoptionPlan,
  type ImportApplyResult,
  type ImportCandidate,
} from "../api/import";
import { pickDirectory } from "../api/settings";
import { listProjects, type ProjectSummary } from "../api/projects";
import type { SyncScope } from "../api/sync";
import { AssetTypeBadge } from "../components/AssetTypeBadge";
import { EmptyState } from "../components/EmptyState";
import { useI18n } from "../i18n/I18nProvider";
import { importProjectPathRequired, projectPathForImport } from "./importPageModel";

type CandidateDecision = ImportAdoptionMode | "skip";

export function ImportPage() {
  const { t } = useI18n();
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [projectPath, setProjectPath] = useState("");
  const [target, setTarget] = useState("claude-code");
  const [scope, setScope] = useState<SyncScope>("project");
  const [candidates, setCandidates] = useState<ImportCandidate[]>([]);
  const [decisions, setDecisions] = useState<Record<string, CandidateDecision>>({});
  const [plan, setPlan] = useState<ImportAdoptionPlan | null>(null);
  const [result, setResult] = useState<ImportApplyResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadProjects() {
      try {
        const nextProjects = await listProjects();
        setProjects(nextProjects);
        setProjectPath((current) => current || nextProjects[0]?.path || "");
      } catch (loadError) {
        setError(messageFromError(loadError));
      }
    }

    void loadProjects();
  }, []);

  const selectedCount = useMemo(
    () => Object.values(decisions).filter((decision) => decision !== "skip").length,
    [decisions],
  );

  async function handleScan() {
    if (importProjectPathRequired(scope) && !projectPath.trim()) {
      setError(t("validation.projectPathRequired"));
      return;
    }

    const scanProjectPath = projectPathForImport(projectPath, scope);
    setLoading(true);
    setError(null);
    setPlan(null);
    setResult(null);
    try {
      const nextCandidates = await scanImportCandidates(scanProjectPath, target, scope);
      setCandidates(nextCandidates);
      setDecisions(
        Object.fromEntries(
          nextCandidates.map((candidate) => [
            candidateKey(candidate),
            candidate.collision ? "skip" : "copy-into-library",
          ]),
        ),
      );
    } catch (scanError) {
      setError(messageFromError(scanError));
    } finally {
      setLoading(false);
    }
  }

  async function handlePreview() {
    const scanProjectPath = projectPathForImport(projectPath, scope);
    const selections = candidates
      .filter((candidate) => decisions[candidateKey(candidate)] !== "skip")
      .map((candidate) => ({
        id: candidate.id,
        assetType: candidate.assetType,
        sourcePath: candidate.sourcePath,
        mode: decisions[candidateKey(candidate)] as ImportAdoptionMode,
      }));
    if (selections.length === 0) {
      setError(t("import.noSelection"));
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);
    try {
      setPlan(await previewImportAdoption(scanProjectPath, target, scope, selections));
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
      setResult(await applyImportAdoption(projectPathForImport(projectPath, plan.scope), plan.planId));
      setPlan(null);
      setCandidates([]);
      setDecisions({});
    } catch (applyError) {
      setError(messageFromError(applyError));
    } finally {
      setLoading(false);
    }
  }

  async function handlePickProjectPath() {
    setLoading(true);
    setError(null);
    try {
      const selectedPath = await pickDirectory();
      if (selectedPath) {
        setProjectPath(selectedPath);
      }
    } catch (pickError) {
      setError(messageFromError(pickError));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section className="import-page">
      <section className="sync-toolbar">
        <label className="field compact-field" htmlFor="import-project">
          <span>{t("sync.project")}</span>
          <div className="compact-actions">
            <input
              id="import-project"
              className="field-input"
              list="import-project-options"
              placeholder={scope === "global-user" ? t("import.projectOptional") : t("validation.projectPathRequired")}
              value={projectPath}
              onChange={(event) => setProjectPath(event.target.value)}
            />
            <button className="secondary-action" type="button" disabled={loading} onClick={() => void handlePickProjectPath()}>
              {t("common.browse")}
            </button>
          </div>
          <datalist id="import-project-options">
            {projects.map((project) => (
              <option key={project.path} value={project.path}>
                {project.name}
              </option>
            ))}
          </datalist>
        </label>
        <label className="field compact-field" htmlFor="import-target">
          <span>{t("sync.target")}</span>
          <select
            id="import-target"
            className="field-input"
            value={target}
            onChange={(event) => setTarget(event.target.value)}
          >
            <option value="claude-code">Claude Code</option>
            <option value="codex">Codex</option>
            <option value="gemini-cli">Gemini CLI</option>
          </select>
        </label>
        <label className="field compact-field" htmlFor="import-scope">
          <span>{t("sync.scope")}</span>
          <select
            id="import-scope"
            className="field-input"
            value={scope}
            onChange={(event) => setScope(event.target.value === "global-user" ? "global-user" : "project")}
          >
            <option value="project">{t("sync.scopeProject")}</option>
            <option value="global-user">{t("sync.scopeGlobal")}</option>
          </select>
        </label>
        <button className="primary-action" type="button" disabled={loading} onClick={() => void handleScan()}>
          {t("import.scan")}
        </button>
      </section>

      <div className="validation-panel valid">
        <p>{t("import.readOnlyNotice")}</p>
      </div>

      {error ? (
        <div className="validation-panel invalid" role="alert">
          <p>{error}</p>
        </div>
      ) : null}

      {result ? (
        <section className="sync-result">
          <h3>{t("import.appliedTitle")}</h3>
          <p>
            {t("import.appliedMessage", {
              copied: result.copiedAssets,
              adopted: result.adoptedAssets,
            })}
          </p>
        </section>
      ) : null}

      {candidates.length > 0 ? (
        <section className="panel">
          <div className="section-heading">
            <h3>{t("import.candidates")}</h3>
            <span className="muted-text">{t("counts.items", { count: candidates.length })}</span>
          </div>
          <div className="import-candidate-list">
            {candidates.map((candidate) => {
              const key = candidateKey(candidate);
              return (
                <article className="import-candidate" key={key}>
                  <div className="import-candidate-main">
                    <div className="asset-card-header">
                      <AssetTypeBadge assetType={candidate.assetType} />
                      <strong>{candidate.id}</strong>
                      <span className="state-pill">{candidate.confidence}</span>
                    </div>
                    <code>{candidate.sourcePath}</code>
                    {candidate.collision ? (
                      <p className="asset-description">
                        {t("import.collision", { assetRef: candidate.collision.assetRef })}
                      </p>
                    ) : null}
                  </div>
                  <label className="field compact-field import-mode-field" htmlFor={`import-mode-${key}`}>
                    <span>{t("import.mode")}</span>
                    <select
                      id={`import-mode-${key}`}
                      className="field-input"
                      value={decisions[key] ?? "skip"}
                      onChange={(event) =>
                        setDecisions((current) => ({
                          ...current,
                          [key]: event.target.value as CandidateDecision,
                        }))
                      }
                    >
                      <option value="copy-into-library">{t("import.copy")}</option>
                      <option value="adopt-into-flowmint">{t("import.adopt")}</option>
                      <option value="skip">{t("import.skip")}</option>
                    </select>
                  </label>
                </article>
              );
            })}
          </div>
          <div className="button-row inline-actions">
            <button
              className="primary-action"
              type="button"
              disabled={loading || selectedCount === 0}
              onClick={() => void handlePreview()}
            >
              {t("import.preview")}
            </button>
          </div>
        </section>
      ) : !loading && !result ? (
        <EmptyState title={t("import.noCandidatesTitle")} message={t("import.noCandidatesMessage")} />
      ) : null}

      {plan ? (
        <section className="panel">
          <div className="section-heading">
            <h3>{t("import.plan")}</h3>
            <span className="muted-text">{plan.planId}</span>
          </div>
          <dl className="detail-list">
            <div>
              <dt>{t("sync.scope")}</dt>
              <dd>{plan.scope === "global-user" ? t("sync.scopeGlobal") : t("sync.scopeProject")}</dd>
            </div>
            <div>
              <dt>{t("common.path")}</dt>
              <dd>{plan.syncRoot}</dd>
            </div>
            <div>
              <dt>{t("import.items")}</dt>
              <dd>{plan.items.length}</dd>
            </div>
            <div>
              <dt>{t("sync.conflicts")}</dt>
              <dd>{plan.conflicts.length}</dd>
            </div>
          </dl>
          {plan.conflicts.length > 0 ? (
            <div className="validation-panel invalid" role="alert">
              {plan.conflicts.map((conflict) => (
                <p key={conflict.sourcePath}>{conflict.message}</p>
              ))}
            </div>
          ) : null}
          <div className="button-row inline-actions">
            <button
              className="primary-action"
              type="button"
              disabled={loading || plan.conflicts.length > 0}
              onClick={() => void handleApply()}
            >
              {t("import.apply")}
            </button>
          </div>
        </section>
      ) : null}
    </section>
  );
}

function candidateKey(candidate: Pick<ImportCandidate, "assetType" | "id" | "sourcePath">): string {
  return `${candidate.assetType}:${candidate.id}:${candidate.sourcePath}`;
}

function messageFromError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

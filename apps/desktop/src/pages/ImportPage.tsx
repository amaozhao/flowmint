import { useEffect, useMemo, useState } from "react";
import {
  applyImportAdoption,
  applyPublicGithubImport,
  previewImportAdoption,
  previewPublicGithubImport,
  scanPublicGithubImport,
  scanImportCandidates,
  type ImportAdoptionMode,
  type ImportAdoptionPlan,
  type ImportApplyResult,
  type ImportCandidate,
  type RemoteImportSelection,
  type PublicGithubImportScanResult,
  type RemoteImportApplyResult,
  type RemoteImportCandidate,
  type RemoteImportPlan,
} from "../api/import";
import { pickDirectory } from "../api/settings";
import { attachAssetToProfile, listProjects, type ProjectSummary } from "../api/projects";
import { attachGlobalProfileAsset, type SyncScope } from "../api/sync";
import { AssetTypeBadge } from "../components/AssetTypeBadge";
import { EmptyState } from "../components/EmptyState";
import { useI18n } from "../i18n/I18nProvider";
import {
  defaultGithubDecisions,
  defaultGithubDestinationIds,
  importProjectPathRequiredForSource,
  isValidPublicGithubUrl,
  normalizeRemoteDestinationId,
  projectPathForImport,
  type ImportSourceMode,
  type RemoteCandidateDecision,
} from "./importPageModel";

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
  const [sourceMode, setSourceMode] = useState<ImportSourceMode>("local-tool");
  const [githubUrl, setGithubUrl] = useState("");
  const [githubScan, setGithubScan] = useState<PublicGithubImportScanResult | null>(null);
  const [githubCandidates, setGithubCandidates] = useState<RemoteImportCandidate[]>([]);
  const [githubDecisions, setGithubDecisions] = useState<Record<string, RemoteCandidateDecision>>({});
  const [githubDestinationIds, setGithubDestinationIds] = useState<Record<string, string>>({});
  const [githubPlan, setGithubPlan] = useState<RemoteImportPlan | null>(null);
  const [githubResult, setGithubResult] = useState<RemoteImportApplyResult | null>(null);
  const [attachGithubToProfile, setAttachGithubToProfile] = useState(true);
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

  const githubSelectedCount = useMemo(
    () => Object.values(githubDecisions).filter((decision) => decision === "import").length,
    [githubDecisions],
  );

  async function handleScan() {
    if (importProjectPathRequiredForSource(sourceMode, scope) && !projectPath.trim()) {
      setError(t("validation.projectPathRequired"));
      return;
    }

    const scanProjectPath = projectPathForImport(projectPath, scope);
    setLoading(true);
    setError(null);
    setPlan(null);
    setResult(null);
    setGithubPlan(null);
    setGithubResult(null);
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

  async function handleGithubScan() {
    const scanUrl = githubUrl.trim();
    if (!isValidPublicGithubUrl(scanUrl)) {
      setError(t("import.githubInvalidUrl"));
      return;
    }

    setLoading(true);
    setError(null);
    setCandidates([]);
    setDecisions({});
    setPlan(null);
    setResult(null);
    setGithubPlan(null);
    setGithubResult(null);
    try {
      const nextScan = await scanPublicGithubImport(scanUrl);
      setGithubScan(nextScan);
      setGithubCandidates(nextScan.candidates);
      setGithubDecisions(defaultGithubDecisions(nextScan.candidates));
      setGithubDestinationIds(defaultGithubDestinationIds(nextScan.candidates));
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

  async function handleGithubPreview() {
    if (!githubScan) {
      setError(t("import.githubSessionExpired"));
      return;
    }

    const selections = githubSelections(githubCandidates, githubDecisions, githubDestinationIds);
    if (selections.length === 0) {
      setError(t("import.noSelection"));
      return;
    }
    if (selections.some((selection) => selection.destinationId.length === 0)) {
      setError(t("validation.idRequired"));
      return;
    }

    setLoading(true);
    setError(null);
    setGithubResult(null);
    try {
      setGithubPlan(await previewPublicGithubImport(githubScan.sessionId, selections));
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

  async function handleGithubApply() {
    if (!githubPlan || githubPlan.conflicts.length > 0) {
      return;
    }
    if (attachGithubToProfile && scope === "project" && !projectPath.trim()) {
      setError(t("validation.projectPathRequired"));
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const nextResult = await applyPublicGithubImport(githubPlan.planId);
      setGithubResult(nextResult);
      setGithubPlan(null);
      setGithubScan(null);
      setGithubCandidates([]);
      setGithubDecisions({});
      setGithubDestinationIds({});
      if (attachGithubToProfile) {
        const assetRefs = githubPlan.items.map((item) => `${item.assetType}:${item.destinationId}`);
        try {
          if (scope === "global-user") {
            await Promise.all(assetRefs.map((assetRef) => attachGlobalProfileAsset(target, assetRef)));
          } else {
            await Promise.all(
              assetRefs.map((assetRef) => attachAssetToProfile(projectPath.trim(), target, "project", assetRef)),
            );
          }
        } catch (attachError) {
          setError(`${t("import.githubAttachFailed")} ${messageFromError(attachError)}`);
        }
      }
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
        <label className="field compact-field" htmlFor="import-source">
          <span>{t("import.source")}</span>
          <select
            id="import-source"
            className="field-input"
            value={sourceMode}
            onChange={(event) => {
              setSourceMode(event.target.value === "public-github" ? "public-github" : "local-tool");
              setError(null);
            }}
          >
            <option value="local-tool">{t("import.sourceLocal")}</option>
            <option value="public-github">{t("import.sourceGithub")}</option>
          </select>
        </label>

        {sourceMode === "local-tool" ? (
          <>
            <ProjectPathField
              id="import-project"
              label={t("sync.project")}
              projectPath={projectPath}
              projects={projects}
              placeholder={scope === "global-user" ? t("import.projectOptional") : t("validation.projectPathRequired")}
              loading={loading}
              browseLabel={t("common.browse")}
              onChange={setProjectPath}
              onBrowse={() => void handlePickProjectPath()}
            />
            <TargetField target={target} onChange={setTarget} label={t("sync.target")} />
            <ScopeField scope={scope} onChange={setScope} label={t("sync.scope")} projectLabel={t("sync.scopeProject")} globalLabel={t("sync.scopeGlobal")} />
            <button className="primary-action" type="button" disabled={loading} onClick={() => void handleScan()}>
              {t("import.scan")}
            </button>
          </>
        ) : (
          <>
            <label className="field compact-field" htmlFor="github-import-url">
              <span>{t("import.githubUrl")}</span>
              <input
                id="github-import-url"
                className="field-input"
                placeholder="https://github.com/org/repo/tree/main/path"
                value={githubUrl}
                onChange={(event) => setGithubUrl(event.target.value)}
              />
            </label>
            <TargetField target={target} onChange={setTarget} label={t("sync.target")} />
            <ScopeField scope={scope} onChange={setScope} label={t("sync.scope")} projectLabel={t("sync.scopeProject")} globalLabel={t("sync.scopeGlobal")} />
            <label className="field compact-field" htmlFor="github-attach-profile">
              <span>{t("import.githubAttachToProfile")}</span>
              <span className="toggle-field">
                <input
                  id="github-attach-profile"
                  type="checkbox"
                  checked={attachGithubToProfile}
                  onChange={(event) => setAttachGithubToProfile(event.target.checked)}
                />
                {scope === "global-user" ? t("import.githubAttachGlobal") : t("import.githubAttachProject")}
              </span>
            </label>
            {attachGithubToProfile && scope === "project" ? (
              <ProjectPathField
                id="github-import-project"
                label={t("sync.project")}
                projectPath={projectPath}
                projects={projects}
                placeholder={t("validation.projectPathRequired")}
                loading={loading}
                browseLabel={t("common.browse")}
                onChange={setProjectPath}
                onBrowse={() => void handlePickProjectPath()}
              />
            ) : null}
            <button className="primary-action" type="button" disabled={loading} onClick={() => void handleGithubScan()}>
              {t("import.githubScan")}
            </button>
          </>
        )}
      </section>

      <div className="validation-panel valid">
        <p>{sourceMode === "public-github" ? t("import.githubPublicOnly") : t("import.readOnlyNotice")}</p>
      </div>

      {error ? (
        <div className="validation-panel invalid" role="alert">
          <p>{error}</p>
        </div>
      ) : null}

      {sourceMode === "local-tool" && result ? (
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

      {sourceMode === "public-github" && githubResult ? (
        <section className="sync-result">
          <h3>{t("import.appliedTitle")}</h3>
          <p>
            {t("import.githubApplyMessage", {
              imported: githubResult.importedAssets,
              refs: githubResult.assetRefs.join(", "),
            })}
          </p>
          {attachGithubToProfile ? <p>{t("import.githubNextStep")}</p> : null}
        </section>
      ) : null}

      {sourceMode === "local-tool" && candidates.length > 0 ? (
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
      ) : sourceMode === "local-tool" && !loading && !result ? (
        <EmptyState title={t("import.noCandidatesTitle")} message={t("import.noCandidatesMessage")} />
      ) : null}

      {sourceMode === "public-github" && githubScan ? (
        <section className="panel">
          <div className="section-heading">
            <h3>{t("import.candidates")}</h3>
            <span className="muted-text">{t("counts.items", { count: githubCandidates.length })}</span>
          </div>
          <dl className="detail-list">
            <div>
              <dt>{t("import.githubRepoSummary")}</dt>
              <dd>
                {githubScan.source.owner}/{githubScan.source.repo}@{githubScan.source.refName}
              </dd>
            </div>
            <div>
              <dt>{t("common.path")}</dt>
              <dd>{githubScan.source.rootPath || "/"}</dd>
            </div>
          </dl>
          {githubScan.warnings.length > 0 ? (
            <div className="validation-panel invalid" role="alert">
              <p>{t("import.githubWarnings")}</p>
              {githubScan.warnings.map((warning) => (
                <p key={warning}>{warning}</p>
              ))}
            </div>
          ) : null}
          {githubCandidates.length > 0 ? (
            <div className="import-candidate-list">
              {githubCandidates.map((candidate) => {
                const destinationId = githubDestinationIds[candidate.candidateId] ?? candidate.defaultDestinationId;
                return (
                  <article className="import-candidate" key={candidate.candidateId}>
                    <div className="import-candidate-main">
                      <div className="asset-card-header">
                        <AssetTypeBadge assetType={candidate.assetType} />
                        <strong>{candidate.id}</strong>
                        <span className="state-pill">{candidate.confidence}</span>
                      </div>
                      {candidate.sourcePaths.map((sourcePath) => (
                        <code key={sourcePath}>{sourcePath}</code>
                      ))}
                      {candidate.collision ? (
                        <p className="asset-description">
                          {t("import.collision", { assetRef: candidate.collision.assetRef })}
                        </p>
                      ) : null}
                      {candidate.warnings.map((warning) => (
                        <p className="asset-description" key={warning}>
                          {warning}
                        </p>
                      ))}
                    </div>
                    <div className="remote-import-controls">
                      <label className="field compact-field" htmlFor={`github-mode-${candidate.candidateId}`}>
                        <span>{t("import.mode")}</span>
                        <select
                          id={`github-mode-${candidate.candidateId}`}
                          className="field-input"
                          value={githubDecisions[candidate.candidateId] ?? "skip"}
                          onChange={(event) =>
                            setGithubDecisions((current) => ({
                              ...current,
                              [candidate.candidateId]: event.target.value as RemoteCandidateDecision,
                            }))
                          }
                        >
                          <option value="import">{t("import.githubImport")}</option>
                          <option value="skip">{t("import.githubSkip")}</option>
                        </select>
                      </label>
                      <label className="field compact-field" htmlFor={`github-destination-${candidate.candidateId}`}>
                        <span>{t("import.githubDestinationId")}</span>
                        <input
                          id={`github-destination-${candidate.candidateId}`}
                          className="field-input"
                          value={destinationId}
                          disabled={(githubDecisions[candidate.candidateId] ?? "skip") === "skip"}
                          onBlur={(event) =>
                            setGithubDestinationIds((current) => ({
                              ...current,
                              [candidate.candidateId]: normalizeRemoteDestinationId(event.target.value),
                            }))
                          }
                          onChange={(event) =>
                            setGithubDestinationIds((current) => ({
                              ...current,
                              [candidate.candidateId]: event.target.value,
                            }))
                          }
                        />
                      </label>
                    </div>
                  </article>
                );
              })}
            </div>
          ) : (
            <EmptyState title={t("import.noCandidatesTitle")} message={t("import.githubNoCandidatesMessage")} />
          )}
          <div className="button-row inline-actions">
            <button
              className="primary-action"
              type="button"
              disabled={loading || githubSelectedCount === 0}
              onClick={() => void handleGithubPreview()}
            >
              {t("import.preview")}
            </button>
          </div>
        </section>
      ) : sourceMode === "public-github" && !loading && !githubResult ? (
        <EmptyState title={t("import.githubNoScanTitle")} message={t("import.githubNoScanMessage")} />
      ) : null}

      {sourceMode === "local-tool" && plan ? (
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

      {sourceMode === "public-github" && githubPlan ? (
        <section className="panel">
          <div className="section-heading">
            <h3>{t("import.plan")}</h3>
            <span className="muted-text">{githubPlan.planId}</span>
          </div>
          <dl className="detail-list">
            <div>
              <dt>{t("import.githubRepoSummary")}</dt>
              <dd>
                {githubPlan.source.owner}/{githubPlan.source.repo}@{githubPlan.source.refName}
              </dd>
            </div>
            <div>
              <dt>{t("import.items")}</dt>
              <dd>{githubPlan.items.length}</dd>
            </div>
            <div>
              <dt>{t("sync.conflicts")}</dt>
              <dd>{githubPlan.conflicts.length}</dd>
            </div>
          </dl>
          {githubPlan.warnings.length > 0 ? (
            <div className="validation-panel invalid" role="alert">
              <p>{t("import.githubWarnings")}</p>
              {githubPlan.warnings.map((warning) => (
                <p key={warning}>{warning}</p>
              ))}
            </div>
          ) : null}
          {githubPlan.conflicts.length > 0 ? (
            <div className="validation-panel invalid" role="alert">
              {githubPlan.conflicts.map((conflict) => (
                <p key={`${conflict.candidateId}:${conflict.destinationId}`}>{conflict.message}</p>
              ))}
            </div>
          ) : null}
          <div className="button-row inline-actions">
            <button
              className="primary-action"
              type="button"
              disabled={loading || githubPlan.conflicts.length > 0}
              onClick={() => void handleGithubApply()}
            >
              {t("import.apply")}
            </button>
          </div>
        </section>
      ) : null}
    </section>
  );
}

function ProjectPathField({
  id,
  label,
  projectPath,
  projects,
  placeholder,
  loading,
  browseLabel,
  onChange,
  onBrowse,
}: {
  id: string;
  label: string;
  projectPath: string;
  projects: ProjectSummary[];
  placeholder: string;
  loading: boolean;
  browseLabel: string;
  onChange: (value: string) => void;
  onBrowse: () => void;
}) {
  return (
    <label className="field compact-field" htmlFor={id}>
      <span>{label}</span>
      <div className="compact-actions">
        <input
          id={id}
          className="field-input"
          list={`${id}-options`}
          placeholder={placeholder}
          value={projectPath}
          onChange={(event) => onChange(event.target.value)}
        />
        <button className="secondary-action" type="button" disabled={loading} onClick={onBrowse}>
          {browseLabel}
        </button>
      </div>
      <datalist id={`${id}-options`}>
        {projects.map((project) => (
          <option key={project.path} value={project.path}>
            {project.name}
          </option>
        ))}
      </datalist>
    </label>
  );
}

function TargetField({
  target,
  onChange,
  label,
}: {
  target: string;
  onChange: (target: string) => void;
  label: string;
}) {
  return (
    <label className="field compact-field" htmlFor="import-target">
      <span>{label}</span>
      <select id="import-target" className="field-input" value={target} onChange={(event) => onChange(event.target.value)}>
        <option value="claude-code">Claude Code</option>
        <option value="codex">Codex</option>
        <option value="gemini-cli">Gemini CLI</option>
      </select>
    </label>
  );
}

function ScopeField({
  scope,
  onChange,
  label,
  projectLabel,
  globalLabel,
}: {
  scope: SyncScope;
  onChange: (scope: SyncScope) => void;
  label: string;
  projectLabel: string;
  globalLabel: string;
}) {
  return (
    <label className="field compact-field" htmlFor="import-scope">
      <span>{label}</span>
      <select
        id="import-scope"
        className="field-input"
        value={scope}
        onChange={(event) => onChange(event.target.value === "global-user" ? "global-user" : "project")}
      >
        <option value="project">{projectLabel}</option>
        <option value="global-user">{globalLabel}</option>
      </select>
    </label>
  );
}

function candidateKey(candidate: Pick<ImportCandidate, "assetType" | "id" | "sourcePath">): string {
  return `${candidate.assetType}:${candidate.id}:${candidate.sourcePath}`;
}

function githubSelections(
  candidates: RemoteImportCandidate[],
  decisions: Record<string, RemoteCandidateDecision>,
  destinationIds: Record<string, string>,
): RemoteImportSelection[] {
  return candidates
    .filter((candidate) => decisions[candidate.candidateId] === "import")
    .map((candidate) => ({
      candidateId: candidate.candidateId,
      assetType: candidate.assetType,
      destinationId: normalizeRemoteDestinationId(destinationIds[candidate.candidateId] ?? candidate.defaultDestinationId),
    }));
}

function messageFromError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

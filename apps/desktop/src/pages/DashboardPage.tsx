import { useEffect, useMemo, useState, type ReactNode } from "react";
import { listAssets, type AssetSummary } from "../api/assets";
import { listProjects, type ProjectSummary } from "../api/projects";
import type { AppState } from "../api/settings";
import { listTargetCapabilities, type TargetCapabilities } from "../api/sync";
import type { NavItem } from "../components/AppSidebar";
import { useI18n } from "../i18n/I18nProvider";
import {
  buildAssetDistributionRows,
  buildProjectSyncRows,
  buildTargetSupportRows,
  type ChartRow,
  type ChartRowId,
  type TargetSupportRow,
} from "./dashboardModel";

type DashboardPageProps = {
  appState: AppState;
  onNavigate: (item: NavItem) => void;
};

export function DashboardPage({ appState, onNavigate }: DashboardPageProps) {
  const { t } = useI18n();
  const [assets, setAssets] = useState<AssetSummary[]>([]);
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [targetCapabilities, setTargetCapabilities] = useState<TargetCapabilities[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadDashboard() {
      try {
        const [nextAssets, nextProjects, nextTargetCapabilities] = await Promise.all([
          listAssets({ assetType: null, query: null }),
          listProjects(),
          listTargetCapabilities(),
        ]);
        setAssets(nextAssets);
        setProjects(nextProjects);
        setTargetCapabilities(nextTargetCapabilities);
        setError(null);
      } catch (loadError) {
        setError(loadError instanceof Error ? loadError.message : String(loadError));
      }
    }

    void loadDashboard();
  }, []);

  const counts = useMemo(
    () => ({
      prompts: assets.filter((asset) => asset.assetType === "prompt").length,
      skills: assets.filter((asset) => asset.assetType === "skill").length,
      playbooks: assets.filter(
        (asset) => asset.assetType === "playbook" || (asset.assetType === "skill" && asset.tags.includes("playbook")),
      ).length,
      pendingProjects: projects.filter((project) => project.attachedAssets > 0).length,
    }),
    [assets, projects],
  );
  const recentAssets = useMemo(
    () =>
      [...assets]
        .sort((left, right) => (right.updatedAt ?? "").localeCompare(left.updatedAt ?? ""))
        .slice(0, 5),
    [assets],
  );
  const recentProjects = projects.slice(0, 5);
  const assetChartRows = useMemo(() => buildAssetDistributionRows(assets), [assets]);
  const projectChartRows = useMemo(() => buildProjectSyncRows(projects), [projects]);
  const targetSupportRows = useMemo(() => buildTargetSupportRows(targetCapabilities), [targetCapabilities]);

  return (
    <>
      <section className="panel">
        <div className="button-row">
          <button className="primary-action" type="button" onClick={() => onNavigate("Assets")}>
            {t("dashboard.newPrompt")}
          </button>
          <button className="secondary-action" type="button" onClick={() => onNavigate("Assets")}>
            {t("dashboard.newSkill")}
          </button>
          <button className="secondary-action" type="button" onClick={() => onNavigate("Projects")}>
            {t("dashboard.addProject")}
          </button>
        </div>
      </section>

      {error ? (
        <div className="validation-panel invalid" role="alert">
          <p>{error}</p>
        </div>
      ) : null}

      <section className="dashboard-grid" aria-label="Dashboard summary">
        <article className="metric-card">
          <span>{t("common.prompts")}</span>
          <strong>{counts.prompts}</strong>
        </article>
        <article className="metric-card">
          <span>{t("common.skills")}</span>
          <strong>{counts.skills}</strong>
        </article>
        <article className="metric-card">
          <span>{t("common.playbooks")}</span>
          <strong>{counts.playbooks}</strong>
        </article>
        <article className="metric-card">
          <span>{t("common.projects")}</span>
          <strong>{projects.length || appState.recentProjects.length}</strong>
        </article>
        <article className="metric-card">
          <span>{t("dashboard.pendingSync")}</span>
          <strong>{counts.pendingProjects}</strong>
        </article>
      </section>

      <section className="dashboard-charts" aria-label={t("dashboard.charts")}>
        <ChartCard title={t("dashboard.assetMix")}>
          <DistributionChart rows={assetChartRows} labelForRow={(id) => assetChartLabel(id, t)} />
        </ChartCard>
        <ChartCard title={t("dashboard.projectReadiness")}>
          <DistributionChart rows={projectChartRows} labelForRow={(id) => projectChartLabel(id, t)} />
        </ChartCard>
        <ChartCard title={t("dashboard.targetSupport")}>
          <TargetSupportChart rows={targetSupportRows} />
        </ChartCard>
      </section>

      <section className="panel">
        <h3>{t("dashboard.localLibrary")}</h3>
        <dl className="detail-list">
          <div>
            <dt>{t("common.path")}</dt>
            <dd>{appState.library.path}</dd>
          </div>
          <div>
            <dt>{t("common.version")}</dt>
            <dd>{appState.version}</dd>
          </div>
        </dl>
      </section>

      <section className="dashboard-columns">
        <article className="panel">
          <h3>{t("dashboard.recentProjects")}</h3>
          {recentProjects.length === 0 ? (
            <p className="muted-text">{t("dashboard.noRecentProjects")}</p>
          ) : (
            <ul className="compact-list">
              {recentProjects.map((project) => (
                <li key={project.path}>
                  <strong>{project.name}</strong>
                  <span className="muted-text">
                    {t("dashboard.assetsAttached", { count: project.attachedAssets })}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </article>

        <article className="panel">
          <h3>{t("dashboard.recentAssets")}</h3>
          {recentAssets.length === 0 ? (
            <p className="muted-text">{t("dashboard.noAssetsYet")}</p>
          ) : (
            <ul className="compact-list">
              {recentAssets.map((asset) => (
                <li key={`${asset.assetType}:${asset.id}`}>
                  <strong>{asset.name}</strong>
                  <span className="muted-text">{asset.assetType}</span>
                </li>
              ))}
            </ul>
          )}
        </article>
      </section>
    </>
  );
}

function ChartCard({ title, children }: { title: string; children: ReactNode }) {
  return (
    <article className="chart-card">
      <h3>{title}</h3>
      {children}
    </article>
  );
}

function DistributionChart({ rows, labelForRow }: { rows: ChartRow[]; labelForRow: (id: ChartRowId) => string }) {
  const { t } = useI18n();
  const total = rows.reduce((sum, row) => sum + row.value, 0);
  if (total === 0) {
    return <p className="muted-text">{t("dashboard.noChartData")}</p>;
  }

  return (
    <div className="bar-chart">
      {rows.map((row) => (
        <div className="chart-row" key={row.id}>
          <div className="chart-row-label">
            <span className="chart-swatch" style={{ background: chartColor(row.id) }} />
            <span>{labelForRow(row.id)}</span>
          </div>
          <div className="chart-track" aria-hidden="true">
            <span
              className="chart-fill"
              style={{
                width: `${row.percent}%`,
                background: chartColor(row.id),
              }}
            />
          </div>
          <strong>{row.value}</strong>
          <span className="muted-text">{row.percent}%</span>
        </div>
      ))}
    </div>
  );
}

function TargetSupportChart({ rows }: { rows: TargetSupportRow[] }) {
  const { t } = useI18n();
  if (rows.length === 0) {
    return <p className="muted-text">{t("dashboard.noChartData")}</p>;
  }

  return (
    <div className="target-support-chart">
      {rows.map((row) => (
        <div className="target-support-row" key={row.targetId}>
          <div className="target-support-header">
            <strong>{row.displayName}</strong>
            <span className="muted-text">
              {t("dashboard.supportedCount", { supported: row.supported, total: row.total })}
            </span>
          </div>
          <div className="stacked-bar" aria-hidden="true">
            <span className="stacked-supported" style={{ width: `${row.supportedPercent}%` }} />
            <span className="stacked-validation" style={{ width: `${row.requiresValidationPercent}%` }} />
            <span className="stacked-blocked" style={{ width: `${row.blockedPercent}%` }} />
          </div>
          <div className="chart-legend">
            <span>
              <i className="legend-dot supported" />
              {t("dashboard.supported")}
            </span>
            <span>
              <i className="legend-dot validation" />
              {t("dashboard.validationNeeded")}
            </span>
            <span>
              <i className="legend-dot blocked" />
              {t("dashboard.blocked")}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}

function assetChartLabel(id: ChartRowId, t: ReturnType<typeof useI18n>["t"]): string {
  switch (id) {
    case "prompt":
      return t("common.prompts");
    case "skill":
      return t("common.skills");
    case "playbook":
      return t("common.playbooks");
    case "instruction-rule":
      return t("common.instructionRules");
    case "command-rule":
      return t("common.commandRules");
    case "configured":
    case "empty":
      return id;
  }
}

function projectChartLabel(id: ChartRowId, t: ReturnType<typeof useI18n>["t"]): string {
  switch (id) {
    case "configured":
      return t("dashboard.configuredProjects");
    case "empty":
      return t("dashboard.emptyProjects");
    case "prompt":
    case "skill":
    case "playbook":
    case "instruction-rule":
    case "command-rule":
      return id;
  }
}

function chartColor(id: ChartRowId): string {
  switch (id) {
    case "prompt":
      return "#0b6bcb";
    case "skill":
      return "#0f7a4f";
    case "playbook":
      return "#b7791f";
    case "instruction-rule":
      return "#6f42c1";
    case "command-rule":
      return "#b42318";
    case "configured":
      return "#157f7a";
    case "empty":
      return "#829ab1";
  }
}

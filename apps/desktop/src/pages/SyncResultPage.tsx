import type { SyncApplyResult } from "../api/sync";
import { useI18n } from "../i18n/I18nProvider";

type SyncResultPageProps = {
  result: SyncApplyResult;
  projectPath: string;
  onOpenProject: () => void;
};

export function SyncResultPage({ result, projectPath, onOpenProject }: SyncResultPageProps) {
  const { t } = useI18n();

  return (
    <section className="sync-result">
      <header className="project-detail-header">
        <div>
          <h3>{t("sync.appliedTitle")}</h3>
          <p className="project-path">{projectPath}</p>
        </div>
        <button className="secondary-action" type="button" onClick={onOpenProject}>
          {t("sync.openProject")}
        </button>
      </header>

      <section className="project-summary-grid" aria-label="Sync result">
        <article className="metric-card compact-metric">
          <span>{t("sync.resultWritten")}</span>
          <strong>{result.writtenFiles}</strong>
        </article>
        <article className="metric-card compact-metric">
          <span>{t("sync.resultDeleted")}</span>
          <strong>{result.deletedFiles}</strong>
        </article>
        <article className="metric-card compact-metric">
          <span>{t("sync.resultNoops")}</span>
          <strong>{result.noops}</strong>
        </article>
      </section>

      <dl className="detail-list">
        <div>
          <dt>{t("sync.planId")}</dt>
          <dd>{result.planId}</dd>
        </div>
      </dl>
    </section>
  );
}

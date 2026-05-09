import type { SyncOperation } from "../api/sync";
import { useI18n } from "../i18n/I18nProvider";
import { EmptyState } from "./EmptyState";

type DiffViewerProps = {
  operation: SyncOperation | null;
};

export function DiffViewer({ operation }: DiffViewerProps) {
  const { t } = useI18n();

  if (!operation) {
    return <EmptyState title={t("sync.noOperationTitle")} message={t("sync.noOperationMessage")} />;
  }

  return (
    <section className="diff-viewer">
      <header className="section-heading">
        <h4>{operation.operationType}</h4>
      </header>
      <dl className="detail-list">
        <div>
          <dt>{t("common.target")}</dt>
          <dd>{operation.targetPath}</dd>
        </div>
        {operation.operationType === "create-file" ? (
          <div>
            <dt>{t("sync.contentHash")}</dt>
            <dd>{operation.contentHash}</dd>
          </div>
        ) : null}
        {operation.operationType === "update-file" ? (
          <>
            <div>
              <dt>{t("sync.previousHash")}</dt>
              <dd>{operation.previousHash ?? t("common.none")}</dd>
            </div>
            <div>
              <dt>{t("sync.newHash")}</dt>
              <dd>{operation.newHash}</dd>
            </div>
          </>
        ) : null}
        {operation.operationType === "delete-generated-file" ? (
          <div>
            <dt>{t("sync.previousHash")}</dt>
            <dd>{operation.previousHash}</dd>
          </div>
        ) : null}
        {operation.operationType === "noop" ? (
          <div>
            <dt>{t("common.reason")}</dt>
            <dd>{operation.reason}</dd>
          </div>
        ) : null}
      </dl>
    </section>
  );
}

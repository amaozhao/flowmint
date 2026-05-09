import type { SyncConflict } from "../api/sync";
import { useI18n } from "../i18n/I18nProvider";

type ConflictBannerProps = {
  conflicts: SyncConflict[];
  onOpenFile: (path: string) => void;
  onCancel: () => void;
};

export function ConflictBanner({ conflicts, onOpenFile, onCancel }: ConflictBannerProps) {
  const { t } = useI18n();

  if (conflicts.length === 0) {
    return null;
  }

  return (
    <section className="conflict-banner" aria-label={t("sync.conflictTitle")}>
      <header className="conflict-header">
        <div>
          <h3>{t("sync.conflictTitle")}</h3>
          <p>{t(conflicts.length === 1 ? "counts.item" : "counts.items", { count: conflicts.length })}</p>
        </div>
        <button className="secondary-action" type="button" onClick={onCancel}>
          {t("common.cancel")}
        </button>
      </header>

      <div className="conflict-list">
        {conflicts.map((conflict) => (
          <article className="conflict-item" key={`${conflict.kind}:${conflict.targetPath}`}>
            <div>
              <span className="state-pill missing">{conflict.kind}</span>
              <p>{conflict.message}</p>
              <code>{conflict.targetPath}</code>
            </div>
            <button
              className="secondary-action"
              type="button"
              onClick={() => onOpenFile(conflict.targetPath)}
            >
              {t("sync.openFile")}
            </button>
          </article>
        ))}
      </div>
    </section>
  );
}

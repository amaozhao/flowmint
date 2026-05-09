import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";

type OnboardingPageProps = {
  libraryPath: string;
  onCreateLibrary: (path: string) => void;
  onPickDirectory: () => Promise<string | null>;
};

export function OnboardingPage({ libraryPath, onCreateLibrary, onPickDirectory }: OnboardingPageProps) {
  const { t } = useI18n();
  const [pathValue, setPathValue] = useState(libraryPath);
  const [error, setError] = useState<string | null>(null);

  async function handlePickDirectory() {
    setError(null);
    try {
      const selectedPath = await onPickDirectory();
      if (selectedPath) {
        setPathValue(selectedPath);
      }
    } catch (pickError) {
      setError(pickError instanceof Error ? pickError.message : String(pickError));
    }
  }

  return (
    <main className="onboarding">
      <section className="onboarding-panel">
        <div className="brand">
          <span className="brand-mark">F</span>
          <div>
            <h1>Flowmint</h1>
            <p>{t("brand.tagline")}</p>
          </div>
        </div>

        <label className="field" htmlFor="library-path">
          <span>{t("onboarding.libraryPath")}</span>
          <input
            id="library-path"
            className="field-input"
            type="text"
            value={pathValue}
            onChange={(event) => setPathValue(event.target.value)}
          />
        </label>

        {error ? (
          <div className="validation-panel invalid" role="alert">
            <p>{error}</p>
          </div>
        ) : null}

        <div className="button-row">
          <button className="secondary-action" type="button" onClick={() => void handlePickDirectory()}>
            {t("common.browse")}
          </button>
          <button className="primary-action" type="button" onClick={() => onCreateLibrary(pathValue)}>
            {t("onboarding.createLibrary")}
          </button>
        </div>
      </section>
    </main>
  );
}

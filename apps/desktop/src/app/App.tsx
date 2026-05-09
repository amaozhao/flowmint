import { useEffect, useState } from "react";
import { getAppState, initLibrary, openLibraryFolder, pickDirectory, type AppState } from "../api/settings";
import { AppSidebar, type NavItem } from "../components/AppSidebar";
import { ErrorBoundary } from "../components/ErrorBoundary";
import { TopBar } from "../components/TopBar";
import { useI18n } from "../i18n/I18nProvider";
import type { TranslationKey } from "../i18n/messages";
import { AssetsPage } from "../pages/AssetsPage";
import { DashboardPage } from "../pages/DashboardPage";
import { OnboardingPage } from "../pages/OnboardingPage";
import { ImportPage } from "../pages/ImportPage";
import { ProjectsPage } from "../pages/ProjectsPage";
import { SettingsPage } from "../pages/SettingsPage";
import { SyncPreviewPage } from "../pages/SyncPreviewPage";
import type { SyncScope } from "../api/sync";

const navItems: NavItem[] = ["Overview", "Assets", "Projects", "Sync", "Import", "Settings"];

type LoadState =
  | { status: "loading" }
  | { status: "ready"; appState: AppState }
  | { status: "error"; message: string };

export function App() {
  const { locale, setLocale, t } = useI18n();
  const [selectedNav, setSelectedNav] = useState<NavItem>("Overview");
  const [syncProjectPath, setSyncProjectPath] = useState("");
  const [syncTarget, setSyncTarget] = useState("claude-code");
  const [syncScope, setSyncScope] = useState<SyncScope>("project");
  const [loadState, setLoadState] = useState<LoadState>({ status: "loading" });

  async function reloadAppState() {
    setLoadState({ status: "loading" });
    try {
      setLoadState({ status: "ready", appState: await getAppState() });
    } catch (error) {
      setLoadState({
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  }

  useEffect(() => {
    void reloadAppState();
  }, []);

  async function handleCreateLibrary(path: string) {
    try {
      await initLibrary(path.trim() || undefined);
      await reloadAppState();
    } catch (error) {
      setLoadState({
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  }

  if (loadState.status === "loading") {
    return <div className="centered-status">{t("common.loading")} Flowmint...</div>;
  }

  if (loadState.status === "error") {
    return (
      <div className="centered-status error-state">
        <h1>Flowmint</h1>
        <p>{loadState.message}</p>
        <button className="primary-action" type="button" onClick={() => void reloadAppState()}>
          {t("common.retry")}
        </button>
      </div>
    );
  }

  if (!loadState.appState.library.initialized) {
    return (
      <OnboardingPage
        libraryPath={loadState.appState.library.path}
        onCreateLibrary={(path) => void handleCreateLibrary(path)}
        onPickDirectory={pickDirectory}
      />
    );
  }

  return (
    <main className="app-shell">
      <AppSidebar activeItem={selectedNav} items={navItems} onSelect={setSelectedNav} />

      <section className="workspace">
        <TopBar
          title={t(navLabelKey(selectedNav))}
          subtitle={t("app.subtitle")}
          action={
            <label className="language-switcher" htmlFor="language-select">
              <span>{t("language.label")}</span>
              <select
                id="language-select"
                className="field-input"
                value={locale}
                onChange={(event) => setLocale(event.target.value === "zh" ? "zh" : "en")}
              >
                <option value="en">{t("language.en")}</option>
                <option value="zh">{t("language.zh")}</option>
              </select>
            </label>
          }
        />

        <ErrorBoundary title={t("error.boundaryTitle")}>
          {selectedNav === "Overview" ? (
            <DashboardPage appState={loadState.appState} onNavigate={setSelectedNav} />
          ) : null}
          {selectedNav === "Assets" ? <AssetsPage /> : null}
          {selectedNav === "Projects" ? (
            <ProjectsPage
              onPreviewSync={(projectPath, target, scope) => {
                setSyncProjectPath(projectPath);
                setSyncTarget(target);
                setSyncScope(scope);
                setSelectedNav("Sync");
              }}
            />
          ) : null}
          {selectedNav === "Sync" ? (
            <SyncPreviewPage initialProjectPath={syncProjectPath} initialTarget={syncTarget} initialScope={syncScope} />
          ) : null}
          {selectedNav === "Import" ? <ImportPage /> : null}
          {selectedNav === "Settings" ? (
            <SettingsPage
              appState={loadState.appState}
              onOpenLibrary={() => void openLibraryFolder()}
              onReload={() => void reloadAppState()}
            />
          ) : null}
        </ErrorBoundary>
      </section>
    </main>
  );
}

function navLabelKey(item: NavItem): TranslationKey {
  switch (item) {
    case "Overview":
      return "nav.overview";
    case "Assets":
      return "nav.assets";
    case "Projects":
      return "nav.projects";
    case "Sync":
      return "nav.syncHistory";
    case "Import":
      return "nav.import";
    case "Settings":
      return "nav.settings";
  }
}

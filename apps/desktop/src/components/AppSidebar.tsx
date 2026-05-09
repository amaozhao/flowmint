import { useI18n } from "../i18n/I18nProvider";
import type { TranslationKey } from "../i18n/messages";

export type NavItem = "Overview" | "Assets" | "Projects" | "Sync" | "Import" | "Settings";

type AppSidebarProps = {
  activeItem: NavItem;
  items: NavItem[];
  onSelect: (item: NavItem) => void;
};

export function AppSidebar({ activeItem, items, onSelect }: AppSidebarProps) {
  const { t } = useI18n();

  return (
    <aside className="sidebar" aria-label="Flowmint">
      <div className="brand">
        <span className="brand-mark">F</span>
        <div>
          <h1>Flowmint</h1>
          <p>{t("brand.tagline")}</p>
        </div>
      </div>

      <nav className="nav-list">
        {items.map((item) => (
          <button
            className={item === activeItem ? "nav-item active" : "nav-item"}
            key={item}
            type="button"
            onClick={() => onSelect(item)}
          >
            {t(navLabelKey(item))}
          </button>
        ))}
      </nav>
    </aside>
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

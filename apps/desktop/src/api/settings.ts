import { callCommand } from "./tauri";

export type LibraryInfo = {
  path: string;
  initialized: boolean;
};

export type AppState = {
  version: string;
  library: LibraryInfo;
  recentProjects: string[];
};

export type IndexSummary = {
  promptCount: number;
  skillCount: number;
  playbookSkillCount: number;
  projectCount: number;
};

export function getAppState(): Promise<AppState> {
  return callCommand<AppState>("get_app_state");
}

export function initLibrary(path?: string): Promise<LibraryInfo> {
  return callCommand<LibraryInfo>("init_library", { path: path ?? null });
}

export function openLibraryFolder(): Promise<void> {
  return callCommand<void>("open_library_folder");
}

export async function pickDirectory(): Promise<string | null> {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Flowmint directory",
    });
    return Array.isArray(selected) ? (selected[0] ?? null) : selected;
  } catch {
    return callCommand<string | null>("pick_directory");
  }
}

export function rebuildIndex(): Promise<IndexSummary> {
  return callCommand<IndexSummary>("rebuild_index");
}

export function exportDebugReport(): Promise<string> {
  return callCommand<string>("export_debug_report");
}

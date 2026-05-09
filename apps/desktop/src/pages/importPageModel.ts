import type { SyncScope } from "../api/sync";

export function importProjectPathRequired(scope: SyncScope): boolean {
  return scope === "project";
}

export function projectPathForImport(projectPath: string, scope: SyncScope): string {
  if (importProjectPathRequired(scope)) {
    return projectPath.trim();
  }
  return projectPath.trim() || ".";
}

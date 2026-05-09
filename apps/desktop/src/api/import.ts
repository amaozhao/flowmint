import { callCommand } from "./tauri";
import type { AssetType } from "./assets";
import type { SyncScope } from "./sync";

export type ImportConfidence = "high" | "medium" | "low";

export type ImportCollision = {
  assetRef: string;
  libraryPath: string;
};

export type ImportCandidate = {
  id: string;
  assetType: AssetType;
  target: string;
  scope: SyncScope;
  sourcePath: string;
  confidence: ImportConfidence;
  collision: ImportCollision | null;
};

export type ImportAdoptionMode = "copy-into-library" | "adopt-into-flowmint";

export type ImportAdoptionSelection = {
  id: string;
  assetType: AssetType;
  sourcePath: string;
  mode: ImportAdoptionMode;
};

export type ImportSourceSnapshot = {
  path: string;
  contentHash: string;
};

export type ImportAdoptionItem = {
  id: string;
  assetType: AssetType;
  sourcePath: string;
  mode: ImportAdoptionMode;
  sourceSnapshots: ImportSourceSnapshot[];
};

export type ImportAdoptionConflict = {
  sourcePath: string;
  message: string;
};

export type ImportAdoptionPlan = {
  planId: string;
  target: string;
  scope: SyncScope;
  syncRoot: string;
  lockfilePath: string;
  items: ImportAdoptionItem[];
  conflicts: ImportAdoptionConflict[];
};

export type ImportApplyResult = {
  planId: string;
  copiedAssets: number;
  adoptedAssets: number;
};

export function scanImportCandidates(
  projectPath: string,
  target: string,
  scope: SyncScope,
): Promise<ImportCandidate[]> {
  return callCommand<ImportCandidate[]>("scan_import_candidates", { projectPath, target, scope });
}

export function previewImportAdoption(
  projectPath: string,
  target: string,
  scope: SyncScope,
  selections: ImportAdoptionSelection[],
): Promise<ImportAdoptionPlan> {
  return callCommand<ImportAdoptionPlan>("preview_import_adoption", {
    projectPath,
    target,
    scope,
    selections,
  });
}

export function applyImportAdoption(projectPath: string, planId: string): Promise<ImportApplyResult> {
  return callCommand<ImportApplyResult>("apply_import_adoption", { projectPath, planId });
}

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

export type RemoteImportProvider = "public-github";

export type RemoteImportSource = {
  provider: RemoteImportProvider;
  owner: string;
  repo: string;
  refName: string;
  commitSha: string;
  rootPath: string;
  canonicalUrl: string;
};

export type RemoteImportCollision = {
  assetRef: string;
  libraryPath: string;
};

export type RemoteImportCandidate = {
  candidateId: string;
  id: string;
  assetType: AssetType;
  confidence: ImportConfidence;
  source: RemoteImportSource;
  sourcePaths: string[];
  defaultDestinationId: string;
  collision: RemoteImportCollision | null;
  warnings: string[];
  importable: boolean;
};

export type RemoteImportSelection = {
  candidateId: string;
  destinationId: string;
  assetType: AssetType;
};

export type RemoteImportConflict = {
  candidateId: string;
  destinationId: string;
  assetType: AssetType;
  message: string;
};

export type RemoteImportPlanItem = {
  candidateId: string;
  destinationId: string;
  assetType: AssetType;
  sourcePaths: string[];
};

export type RemoteImportPlan = {
  planId: string;
  source: RemoteImportSource;
  items: RemoteImportPlanItem[];
  conflicts: RemoteImportConflict[];
  warnings: string[];
};

export type RemoteImportApplyResult = {
  planId: string;
  importedAssets: number;
  assetRefs: string[];
  provenancePaths: string[];
};

export type PublicGithubImportScanResult = {
  sessionId: string;
  source: RemoteImportSource;
  candidates: RemoteImportCandidate[];
  warnings: string[];
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

export function scanPublicGithubImport(url: string): Promise<PublicGithubImportScanResult> {
  return callCommand<PublicGithubImportScanResult>("scan_public_github_import", { url });
}

export function previewPublicGithubImport(
  sessionId: string,
  selections: RemoteImportSelection[],
): Promise<RemoteImportPlan> {
  return callCommand<RemoteImportPlan>("preview_public_github_import", { sessionId, selections });
}

export function applyPublicGithubImport(planId: string): Promise<RemoteImportApplyResult> {
  return callCommand<RemoteImportApplyResult>("apply_public_github_import", { planId });
}

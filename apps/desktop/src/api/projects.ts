import type { AssetSummary, AssetType } from "./assets";
import type { SyncScope } from "./sync";
import { callCommand } from "./tauri";

export type ProjectAssetType = AssetType;
export type AttachedAssetState = "available" | "missing";

export type ProjectManifest = {
  project: {
    name: string;
  };
  export: {
    target: string;
  };
  attach: {
    prompts: string[];
    skills: string[];
  };
  exports: Array<{
    target: string;
    scope: "project" | "global-user";
    prompts: string[];
    skills: string[];
    playbooks: string[];
    instructionRules: string[];
    commandRules: string[];
  }>;
};

export type AttachedAsset = {
  assetType: ProjectAssetType;
  id: string;
  assetRef: string;
  state: AttachedAssetState;
  summary: AssetSummary | null;
};

export type ProjectSummary = {
  path: string;
  name: string;
  initialized: boolean;
  attachedPrompts: number;
  attachedSkills: number;
  attachedAssets: number;
};

export type ProjectDetail = {
  path: string;
  initialized: boolean;
  manifest: ProjectManifest;
  attachedAssets: AttachedAsset[];
};

export function listProjects(): Promise<ProjectSummary[]> {
  return callCommand<ProjectSummary[]>("list_projects");
}

export function addProject(path: string): Promise<ProjectDetail> {
  return callCommand<ProjectDetail>("add_project", { path });
}

export function getProject(path: string): Promise<ProjectDetail> {
  return callCommand<ProjectDetail>("get_project", { path });
}

export function attachAsset(path: string, assetRef: string): Promise<ProjectDetail> {
  return callCommand<ProjectDetail>("attach_asset", { path, assetRef });
}

export function detachAsset(path: string, assetRef: string): Promise<ProjectDetail> {
  return callCommand<ProjectDetail>("detach_asset", { path, assetRef });
}

export function attachAssetToProfile(
  path: string,
  target: string,
  scope: SyncScope,
  assetRef: string,
): Promise<ProjectDetail> {
  return callCommand<ProjectDetail>("attach_asset_to_profile", { path, target, scope, assetRef });
}

export function detachAssetFromProfile(
  path: string,
  target: string,
  scope: SyncScope,
  assetRef: string,
): Promise<ProjectDetail> {
  return callCommand<ProjectDetail>("detach_asset_from_profile", { path, target, scope, assetRef });
}

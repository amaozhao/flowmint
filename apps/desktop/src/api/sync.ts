import { callCommand } from "./tauri";

export type SyncConflictKind =
  | "unmanaged-target"
  | "modified-generated-file"
  | "incomplete-managed-block"
  | "unsafe-symlink"
  | "unsafe-asset-id"
  | "output-not-writable"
  | "missing-asset"
  | "unsupported-mapping";

export type SyncConflict = {
  targetPath: string;
  kind: SyncConflictKind;
  message: string;
};

export type SyncOperation =
  | {
      operationType: "create-file";
      targetPath: string;
      contentHash: string;
    }
  | {
      operationType: "update-file";
      targetPath: string;
      previousHash: string | null;
      newHash: string;
    }
  | {
      operationType: "create-dir";
      targetPath: string;
    }
  | {
      operationType: "delete-generated-file";
      targetPath: string;
      previousHash: string;
    }
  | {
      operationType: "noop";
      targetPath: string;
      reason: string;
    };

export type SyncScope = "project" | "global-user";
export type ExportAssetKind = "prompt" | "skill" | "playbook" | "instruction-rule" | "command-rule";
export type ExportSupport = "supported" | "unsupported" | "requires-validation";

export type TargetCapability = {
  assetKind: ExportAssetKind;
  scope: SyncScope;
  support: ExportSupport;
  outputHint: string;
  reason: string;
};

export type TargetCapabilities = {
  targetId: string;
  displayName: string;
  capabilities: TargetCapability[];
};

export type ExportProfile = {
  target: string;
  scope: SyncScope;
  prompts: string[];
  skills: string[];
  playbooks: string[];
  instructionRules: string[];
  commandRules: string[];
};

export type GlobalSyncProfiles = {
  profiles: ExportProfile[];
};

export type SyncPlan = {
  planId: string;
  projectPath: string;
  exporter: string;
  scope: SyncScope;
  operations: SyncOperation[];
  conflicts: SyncConflict[];
};

export type SyncApplyResult = {
  planId: string;
  writtenFiles: number;
  deletedFiles: number;
  noops: number;
};

export function previewSync(projectPath: string, target = "claude-code", scope: SyncScope = "project"): Promise<SyncPlan> {
  return callCommand<SyncPlan>("preview_sync", { projectPath, target, scope });
}

export function applySync(planId: string): Promise<SyncApplyResult> {
  return callCommand<SyncApplyResult>("apply_sync", { planId });
}

export function acknowledgeGlobalSyncPlan(planId: string, confirmedPaths: string[]): Promise<void> {
  return callCommand<void>("acknowledge_global_sync_plan", { planId, confirmedPaths });
}

export function openSyncTarget(path: string): Promise<void> {
  return callCommand<void>("open_sync_target", { path });
}

export function listTargetCapabilities(): Promise<TargetCapabilities[]> {
  return callCommand<TargetCapabilities[]>("list_target_capabilities");
}

export function listGlobalSyncProfiles(): Promise<GlobalSyncProfiles> {
  return callCommand<GlobalSyncProfiles>("list_global_sync_profiles");
}

export function attachGlobalProfileAsset(target: string, assetRef: string): Promise<GlobalSyncProfiles> {
  return callCommand<GlobalSyncProfiles>("attach_global_profile_asset", { target, assetRef });
}

export function detachGlobalProfileAsset(target: string, assetRef: string): Promise<GlobalSyncProfiles> {
  return callCommand<GlobalSyncProfiles>("detach_global_profile_asset", { target, assetRef });
}

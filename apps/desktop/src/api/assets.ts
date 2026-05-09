import { callCommand } from "./tauri";

export type AssetType = "prompt" | "skill" | "playbook" | "instruction-rule" | "command-rule";
export type ValidationStatus = "valid" | "invalid";

export type AssetSummary = {
  id: string;
  assetType: AssetType;
  name: string;
  description: string | null;
  tags: string[];
  path: string;
  validationStatus: ValidationStatus;
  updatedAt: string | null;
};

export type PromptVariable = {
  name: string;
  description: string | null;
  defaultValue: string | null;
};

export type PromptAsset = {
  id: string;
  name: string;
  description: string | null;
  tags: string[];
  variables: PromptVariable[];
  body: string;
};

export type SkillFileKind = "skill-markdown" | "metadata" | "example" | "resource";

export type SkillFile = {
  path: string;
  kind: SkillFileKind;
  content: string | null;
};

export type SkillMetadata = {
  rawToml: string;
};

export type SkillAsset = {
  id: string;
  name: string;
  description: string | null;
  tags: string[];
  rootDir: string;
  skillMd: string;
  metadata: SkillMetadata | null;
  files: SkillFile[];
};

export type PlaybookSideEffectLevel = "none" | "read-only" | "writes-files" | "runs-commands" | "external-side-effects";
export type PlaybookInvocation = "manual" | "model" | "both";

export type PlaybookInput = {
  name: string;
  description: string | null;
  required: boolean;
};

export type PlaybookStep = {
  title: string;
  body: string;
};

export type PlaybookAsset = {
  id: string;
  name: string;
  description: string | null;
  tags: string[];
  trigger: string;
  inputs: PlaybookInput[];
  steps: PlaybookStep[];
  verification: string;
  failureHandling: string;
  sideEffectLevel: PlaybookSideEffectLevel;
  recommendedInvocation: PlaybookInvocation;
  targetCompatibility: string[];
};

export type RuleKind = "instruction" | "command";
export type CommandRuleDecision = "prompt" | "allow" | "forbid";

export type CommandRule = {
  prefix: string[];
  decision: CommandRuleDecision;
};

export type RuleAsset = {
  id: string;
  name: string;
  description: string | null;
  tags: string[];
  ruleKind: RuleKind;
  pathGlobs: string[];
  commandRule: CommandRule | null;
  targetCompatibility: string[];
  body: string;
};

export type EditableAssetDetail =
  | {
      assetType: "prompt";
      asset: PromptAsset;
    }
  | {
      assetType: "skill";
      asset: SkillAsset;
    }
  | {
      assetType: "playbook";
      asset: PlaybookAsset;
    }
  | {
      assetType: "instruction-rule";
      asset: RuleAsset;
    }
  | {
      assetType: "command-rule";
      asset: RuleAsset;
    };

export type AssetDetail =
  | EditableAssetDetail;

export type AssetFilter = {
  assetType?: AssetType | null;
  query?: string | null;
};

export type CreateAssetInput = {
  asset: AssetDetail;
};

export type UpdateAssetInput = {
  asset: AssetDetail;
};

export type ValidationReport = {
  status: ValidationStatus;
  messages: string[];
};

export function listAssets(filter: AssetFilter = {}): Promise<AssetSummary[]> {
  return callCommand<AssetSummary[]>("list_assets", { filter });
}

export function getAsset(assetRef: string): Promise<AssetDetail> {
  return callCommand<AssetDetail>("get_asset", { assetRef });
}

export function createAsset(input: CreateAssetInput): Promise<AssetDetail> {
  return callCommand<AssetDetail>("create_asset", { input });
}

export function updateAsset(input: UpdateAssetInput): Promise<AssetDetail> {
  return callCommand<AssetDetail>("update_asset", { input });
}

export function deleteAsset(assetRef: string): Promise<void> {
  return callCommand<void>("delete_asset", { assetRef });
}

export function validateAsset(assetRef: string): Promise<ValidationReport> {
  return callCommand<ValidationReport>("validate_asset", { assetRef });
}

export function openAssetFolder(assetRef: string): Promise<void> {
  return callCommand<void>("open_asset_folder", { assetRef });
}

export function promoteSkillToPlaybook(skillId: string, playbookId: string): Promise<AssetDetail> {
  return callCommand<AssetDetail>("promote_skill_to_playbook", { skillId, playbookId });
}

export function assetRefForSummary(asset: Pick<AssetSummary, "assetType" | "id">): string {
  return `${asset.assetType}:${asset.id}`;
}

export function assetRefForDetail(detail: AssetDetail): string {
  return `${detail.assetType}:${detail.asset.id}`;
}

export function isEditableAssetDetail(detail: AssetDetail): detail is EditableAssetDetail {
  return (
    detail.assetType === "prompt" ||
    detail.assetType === "skill" ||
    detail.assetType === "playbook" ||
    detail.assetType === "instruction-rule" ||
    detail.assetType === "command-rule"
  );
}

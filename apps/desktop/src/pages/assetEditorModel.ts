import type {
  AssetDetail,
  AssetType,
  CommandRuleDecision,
  EditableAssetDetail,
  PlaybookAsset,
  PlaybookInput,
  PlaybookInvocation,
  PlaybookSideEffectLevel,
  PlaybookStep,
  PromptAsset,
  PromptVariable,
  RuleAsset,
  SkillAsset,
  SkillFile,
} from "../api/assets";
import type { SkillTemplate } from "../api/templates";
import { createTranslator, type TranslationKey } from "../i18n/messages";

export type EditableAssetType = AssetType;

export type AssetEditorDraft = {
  assetType: EditableAssetType;
  id: string;
  name: string;
  description: string;
  tags: string[];
  variables: PromptVariable[];
  body: string;
  skillMd: string;
  metadataToml: string;
  rootDir: string;
  files: SkillFile[];
  trigger: string;
  inputs: PlaybookInput[];
  steps: PlaybookStep[];
  verification: string;
  failureHandling: string;
  sideEffectLevel: PlaybookSideEffectLevel;
  recommendedInvocation: PlaybookInvocation;
  pathGlobs: string[];
  commandPrefix: string[];
  commandDecision: CommandRuleDecision;
  targetCompatibility: string[];
};

const SAFE_ASSET_ID_PATTERN = /^[a-z0-9_-]+$/;
type Translator = (key: TranslationKey, params?: Record<string, string | number>) => string;
const defaultTranslator = createTranslator("en");

export function buildEmptyAssetDraft(assetType: EditableAssetType): AssetEditorDraft {
  return {
    assetType,
    id: "",
    name: "",
    description: "",
    tags: [],
    variables: [],
    body: "",
    skillMd: "",
    metadataToml: "",
    rootDir: "",
    files: [],
    trigger: "",
    inputs: [],
    steps: [{ title: "", body: "" }],
    verification: "",
    failureHandling: "",
    sideEffectLevel: "read-only",
    recommendedInvocation: "manual",
    pathGlobs: [],
    commandPrefix: [],
    commandDecision: "prompt",
    targetCompatibility: defaultTargetCompatibility(assetType),
  };
}

export function buildDraftFromSkillTemplate(template: SkillTemplate): AssetEditorDraft {
  return {
    ...buildEmptyAssetDraft("skill"),
    name: template.name,
    description: template.description,
    tags: template.tags,
    skillMd: template.skillMd,
  };
}

export function draftFromAssetDetail(detail: EditableAssetDetail): AssetEditorDraft {
  switch (detail.assetType) {
    case "prompt":
      return draftFromPrompt(detail.asset);
    case "skill":
      return draftFromSkill(detail.asset);
    case "playbook":
      return draftFromPlaybook(detail.asset);
    case "instruction-rule":
    case "command-rule":
      return draftFromRule(detail.asset, detail.assetType);
  }
}

export function getDraftValidationMessages(
  draft: AssetEditorDraft,
  t: Translator = defaultTranslator,
): string[] {
  const messages: string[] = [];

  if (!draft.id.trim()) {
    messages.push(t("validation.idRequired"));
  } else if (!SAFE_ASSET_ID_PATTERN.test(draft.id)) {
    messages.push(t("validation.idSafe"));
  }

  if (!draft.name.trim()) {
    messages.push(t("validation.nameRequired"));
  }

  if (draft.assetType === "prompt" && !draft.body.trim()) {
    messages.push(t("validation.promptBodyRequired"));
  }

  if (draft.assetType === "skill" && !draft.skillMd.trim()) {
    messages.push(t("validation.skillMdRequired"));
  }

  if (draft.assetType === "playbook") {
    if (!draft.trigger.trim()) {
      messages.push(t("validation.playbookTriggerRequired"));
    }
    if (normalizeSteps(draft.steps).length === 0) {
      messages.push(t("validation.playbookStepRequired"));
    }
    if (!draft.verification.trim()) {
      messages.push(t("validation.playbookVerificationRequired"));
    }
    if (!draft.failureHandling.trim()) {
      messages.push(t("validation.playbookFailureRequired"));
    }
  }

  if (isRuleAssetType(draft.assetType)) {
    if (!draft.body.trim()) {
      messages.push(t("validation.ruleBodyRequired"));
    }
    if (draft.assetType === "command-rule" && draft.commandPrefix.length === 0) {
      messages.push(t("validation.commandPrefixRequired"));
    }
  }

  return messages;
}

export function normalizeTags(tags: string[]): string[] {
  const normalized = new Set<string>();
  for (const tag of tags) {
    const value = tag.trim();
    if (value) {
      normalized.add(value);
    }
  }
  return Array.from(normalized);
}

export function parseTags(value: string): string[] {
  return normalizeTags(value.split(","));
}

export function toAssetDetail(draft: AssetEditorDraft): EditableAssetDetail {
  switch (draft.assetType) {
    case "prompt":
      return {
        assetType: "prompt",
        asset: {
          id: draft.id.trim(),
          name: draft.name.trim(),
          description: optionalText(draft.description),
          tags: normalizeTags(draft.tags),
          variables: normalizeVariables(draft.variables),
          body: draft.body,
        },
      };
    case "skill":
      return {
        assetType: "skill",
        asset: {
          id: draft.id.trim(),
          name: draft.name.trim(),
          description: optionalText(draft.description),
          tags: normalizeTags(draft.tags),
          rootDir: draft.rootDir,
          skillMd: draft.skillMd,
          metadata: optionalText(draft.metadataToml)
            ? {
                rawToml: draft.metadataToml,
              }
            : null,
          files: draft.files,
        },
      };
    case "playbook":
      return {
        assetType: "playbook",
        asset: {
          id: draft.id.trim(),
          name: draft.name.trim(),
          description: optionalText(draft.description),
          tags: normalizeTags(draft.tags),
          trigger: draft.trigger,
          inputs: normalizeInputs(draft.inputs),
          steps: normalizeSteps(draft.steps),
          verification: draft.verification,
          failureHandling: draft.failureHandling,
          sideEffectLevel: draft.sideEffectLevel,
          recommendedInvocation: draft.recommendedInvocation,
          targetCompatibility: normalizeTags(draft.targetCompatibility),
        },
      };
    case "instruction-rule":
    case "command-rule":
      return {
        assetType: draft.assetType,
        asset: {
          id: draft.id.trim(),
          name: draft.name.trim(),
          description: optionalText(draft.description),
          tags: normalizeTags(draft.tags),
          ruleKind: draft.assetType === "command-rule" ? "command" : "instruction",
          pathGlobs: normalizeTags(draft.pathGlobs),
          commandRule:
            draft.assetType === "command-rule"
              ? {
                  prefix: normalizeTags(draft.commandPrefix),
                  decision: draft.commandDecision,
                }
              : null,
          targetCompatibility: normalizeTags(draft.targetCompatibility),
          body: draft.body,
        },
      };
  }
}

function draftFromPrompt(prompt: PromptAsset): AssetEditorDraft {
  return {
    ...buildEmptyAssetDraft("prompt"),
    id: prompt.id,
    name: prompt.name,
    description: prompt.description ?? "",
    tags: prompt.tags,
    variables: prompt.variables,
    body: prompt.body,
  };
}

function draftFromSkill(skill: SkillAsset): AssetEditorDraft {
  return {
    ...buildEmptyAssetDraft("skill"),
    id: skill.id,
    name: skill.name,
    description: skill.description ?? "",
    tags: skill.tags,
    skillMd: skill.skillMd,
    metadataToml: skill.metadata?.rawToml ?? "",
    rootDir: skill.rootDir,
    files: skill.files,
  };
}

function draftFromPlaybook(playbook: PlaybookAsset): AssetEditorDraft {
  return {
    ...buildEmptyAssetDraft("playbook"),
    id: playbook.id,
    name: playbook.name,
    description: playbook.description ?? "",
    tags: playbook.tags,
    trigger: playbook.trigger,
    inputs: playbook.inputs,
    steps: playbook.steps.length > 0 ? playbook.steps : [{ title: "", body: "" }],
    verification: playbook.verification,
    failureHandling: playbook.failureHandling,
    sideEffectLevel: playbook.sideEffectLevel,
    recommendedInvocation: playbook.recommendedInvocation,
    targetCompatibility: playbook.targetCompatibility,
  };
}

function draftFromRule(rule: RuleAsset, assetType: Extract<AssetType, "instruction-rule" | "command-rule">): AssetEditorDraft {
  return {
    ...buildEmptyAssetDraft(assetType),
    id: rule.id,
    name: rule.name,
    description: rule.description ?? "",
    tags: rule.tags,
    pathGlobs: rule.pathGlobs,
    commandPrefix: rule.commandRule?.prefix ?? [],
    commandDecision: rule.commandRule?.decision ?? "prompt",
    targetCompatibility: rule.targetCompatibility,
    body: rule.body,
  };
}

function defaultTargetCompatibility(assetType: AssetType): string[] {
  switch (assetType) {
    case "prompt":
      return ["claude-code", "gemini-cli"];
    case "skill":
    case "playbook":
      return ["claude-code", "codex"];
    case "instruction-rule":
      return ["claude-code", "codex", "gemini-cli"];
    case "command-rule":
      return ["codex"];
  }
}

function isRuleAssetType(assetType: AssetType): assetType is "instruction-rule" | "command-rule" {
  return assetType === "instruction-rule" || assetType === "command-rule";
}

function optionalText(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function normalizeVariables(variables: PromptVariable[]): PromptVariable[] {
  return variables
    .map((variable) => ({
      name: variable.name.trim(),
      description: optionalText(variable.description ?? ""),
      defaultValue: optionalText(variable.defaultValue ?? ""),
    }))
    .filter((variable) => variable.name.length > 0);
}

function normalizeInputs(inputs: PlaybookInput[]): PlaybookInput[] {
  return inputs
    .map((input) => ({
      name: input.name.trim(),
      description: optionalText(input.description ?? ""),
      required: input.required,
    }))
    .filter((input) => input.name.length > 0);
}

function normalizeSteps(steps: PlaybookStep[]): PlaybookStep[] {
  return steps
    .map((step) => ({
      title: step.title.trim(),
      body: step.body.trim(),
    }))
    .filter((step) => step.title.length > 0 && step.body.length > 0);
}

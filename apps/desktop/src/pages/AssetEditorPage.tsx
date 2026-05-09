import { useEffect, useMemo, useState } from "react";
import type { EditableAssetDetail, PlaybookInput, PlaybookStep, PromptVariable, SkillFile } from "../api/assets";
import { AssetTypeBadge } from "../components/AssetTypeBadge";
import { MarkdownEditor } from "../components/MarkdownEditor";
import { MetadataForm } from "../components/MetadataForm";
import { TagInput } from "../components/TagInput";
import { useI18n } from "../i18n/I18nProvider";
import {
  buildEmptyAssetDraft,
  draftFromAssetDetail,
  getDraftValidationMessages,
  toAssetDetail,
  type AssetEditorDraft,
  type EditableAssetType,
} from "./assetEditorModel";

type AssetEditorMode = "create" | "edit";

type AssetEditorPageProps = {
  mode: AssetEditorMode;
  assetType: EditableAssetType;
  assetDetail: EditableAssetDetail | null;
  initialDraft: AssetEditorDraft | null;
  saving: boolean;
  error: string | null;
  onSave: (asset: EditableAssetDetail) => Promise<void>;
  onCancel: () => void;
  onDelete: (() => Promise<void>) | null;
  onOpenFolder: (() => Promise<void>) | null;
  onPromoteSkill: ((skillId: string) => Promise<void>) | null;
};

export function AssetEditorPage({
  mode,
  assetType,
  assetDetail,
  initialDraft: initialDraftOverride,
  saving,
  error,
  onSave,
  onCancel,
  onDelete,
  onOpenFolder,
  onPromoteSkill,
}: AssetEditorPageProps) {
  const { t } = useI18n();
  const [draft, setDraft] = useState<AssetEditorDraft>(() =>
    initialDraft(mode, assetType, assetDetail, initialDraftOverride),
  );
  const [messages, setMessages] = useState<string[]>([]);
  const [validationStatus, setValidationStatus] = useState<string | null>(null);

  useEffect(() => {
    setDraft(initialDraft(mode, assetType, assetDetail, initialDraftOverride));
    setMessages([]);
    setValidationStatus(null);
  }, [assetDetail, assetType, initialDraftOverride, mode]);

  const title = useMemo(() => {
    if (draft.assetType === "prompt") {
      return t(mode === "create" ? "assets.newPromptTitle" : "assets.editPromptTitle");
    }
    if (draft.assetType === "skill") {
      if (draft.tags.includes("playbook")) {
        return t(mode === "create" ? "assets.newPlaybookSkillTitle" : "assets.editPlaybookSkillTitle");
      }
      return t(mode === "create" ? "assets.newSkillTitle" : "assets.editSkillTitle");
    }
    if (draft.assetType === "playbook") {
      return t(mode === "create" ? "assets.newPlaybookTitle" : "assets.editPlaybookTitle");
    }
    if (draft.assetType === "instruction-rule") {
      return t(mode === "create" ? "assets.newInstructionRuleTitle" : "assets.editInstructionRuleTitle");
    }
    return t(mode === "create" ? "assets.newCommandRuleTitle" : "assets.editCommandRuleTitle");
  }, [draft.assetType, draft.tags, mode, t]);

  function handleValidate() {
    const nextMessages = getDraftValidationMessages(draft, t);
    setMessages(nextMessages);
    setValidationStatus(nextMessages.length === 0 ? t("assets.draftValid") : null);
  }

  async function handleSave() {
    const nextMessages = getDraftValidationMessages(draft, t);
    setMessages(nextMessages);
    setValidationStatus(null);
    if (nextMessages.length > 0) {
      return;
    }

    await onSave(toAssetDetail(draft));
  }

  return (
    <section className="asset-editor">
      <header className="asset-editor-header">
        <div>
          <div className="title-row">
            <AssetTypeBadge assetType={draft.assetType} />
            <h3>{title}</h3>
          </div>
          {mode === "edit" ? <p className="muted-text">{draft.id}</p> : null}
        </div>

        <div className="button-row inline-actions">
          <button className="secondary-action" type="button" onClick={handleValidate}>
            {t("common.validate")}
          </button>
          <button className="secondary-action" type="button" onClick={onCancel}>
            {t("common.cancel")}
          </button>
          {onDelete ? (
            <button className="danger-action" type="button" onClick={() => void onDelete()}>
              {t("common.delete")}
            </button>
          ) : null}
          {onOpenFolder ? (
            <button className="secondary-action" type="button" onClick={() => void onOpenFolder()}>
              {t("assets.openFolder")}
            </button>
          ) : null}
          {onPromoteSkill && draft.assetType === "skill" && draft.tags.includes("playbook") ? (
            <button className="secondary-action" type="button" onClick={() => void onPromoteSkill(draft.id)}>
              {t("assets.promotePlaybook")}
            </button>
          ) : null}
          <button
            className="primary-action"
            type="button"
            disabled={saving}
            onClick={() => void handleSave()}
          >
            {saving ? t("common.saving") : t("common.save")}
          </button>
        </div>
      </header>

      {messages.length > 0 ? (
        <div className="validation-panel invalid" role="alert">
          {messages.map((message) => (
            <p key={message}>{message}</p>
          ))}
        </div>
      ) : null}

      {validationStatus ? (
        <div className="validation-panel valid">
          <p>{validationStatus}</p>
        </div>
      ) : null}

      {error ? (
        <div className="validation-panel invalid" role="alert">
          <p>{error}</p>
        </div>
      ) : null}

      <MetadataForm draft={draft} idLocked={mode === "edit"} onChange={setDraft} />

      {draft.assetType === "prompt" ? (
        <PromptEditor draft={draft} onChange={setDraft} />
      ) : draft.assetType === "skill" ? (
        <SkillEditor draft={draft} onChange={setDraft} />
      ) : draft.assetType === "playbook" ? (
        <PlaybookEditor draft={draft} onChange={setDraft} />
      ) : (
        <RuleEditor draft={draft} onChange={setDraft} />
      )}
    </section>
  );
}

function PromptEditor({
  draft,
  onChange,
}: {
  draft: AssetEditorDraft;
  onChange: (draft: AssetEditorDraft) => void;
}) {
  const { t } = useI18n();

  return (
    <>
      <MarkdownEditor
        id="prompt-body"
        label={t("assets.promptMarkdown")}
        value={draft.body}
        onChange={(body) => onChange({ ...draft, body })}
      />

      <section className="form-section">
        <div className="section-heading">
          <h4>{t("assets.variables")}</h4>
          <span className="muted-text">{draft.variables.length}</span>
        </div>
        <PromptVariables variables={draft.variables} onChange={(variables) => onChange({ ...draft, variables })} />
      </section>

      <section className="form-section preview-section">
        <div className="section-heading">
          <h4>{t("assets.preview")}</h4>
        </div>
        <pre className="markdown-preview">{draft.body || t("assets.noContent")}</pre>
      </section>
    </>
  );
}

function PromptVariables({
  variables,
  onChange,
}: {
  variables: PromptVariable[];
  onChange: (variables: PromptVariable[]) => void;
}) {
  const { t } = useI18n();

  function updateVariable(index: number, variable: PromptVariable) {
    onChange(variables.map((current, currentIndex) => (currentIndex === index ? variable : current)));
  }

  function removeVariable(index: number) {
    onChange(variables.filter((_, currentIndex) => currentIndex !== index));
  }

  return (
    <div className="variable-list">
      {variables.length === 0 ? <div className="placeholder-strip">{t("common.none")}</div> : null}

      {variables.map((variable, index) => (
        <div className="variable-row" key={index}>
          <label className="field compact-field" htmlFor={`prompt-variable-name-${index}`}>
            <span>{t("common.name")}</span>
            <input
              id={`prompt-variable-name-${index}`}
              className="field-input"
              type="text"
              value={variable.name}
              onChange={(event) => updateVariable(index, { ...variable, name: event.target.value })}
            />
          </label>
          <label className="field compact-field" htmlFor={`prompt-variable-description-${index}`}>
            <span>{t("common.description")}</span>
            <input
              id={`prompt-variable-description-${index}`}
              className="field-input"
              type="text"
              value={variable.description ?? ""}
              onChange={(event) => updateVariable(index, { ...variable, description: event.target.value })}
            />
          </label>
          <label className="field compact-field" htmlFor={`prompt-variable-default-${index}`}>
            <span>{t("common.default")}</span>
            <input
              id={`prompt-variable-default-${index}`}
              className="field-input"
              type="text"
              value={variable.defaultValue ?? ""}
              onChange={(event) => updateVariable(index, { ...variable, defaultValue: event.target.value })}
            />
          </label>
          <button className="secondary-action" type="button" onClick={() => removeVariable(index)}>
            {t("common.remove")}
          </button>
        </div>
      ))}

      <button
        className="secondary-action"
        type="button"
        onClick={() => onChange([...variables, { name: "", description: null, defaultValue: null }])}
      >
        {t("assets.addVariable")}
      </button>
    </div>
  );
}

function SkillEditor({
  draft,
  onChange,
}: {
  draft: AssetEditorDraft;
  onChange: (draft: AssetEditorDraft) => void;
}) {
  return (
    <>
      <MarkdownEditor
        id="skill-md"
        label="SKILL.md"
        rows={16}
        value={draft.skillMd}
        onChange={(skillMd) => onChange({ ...draft, skillMd })}
      />

      <label className="field markdown-field" htmlFor="skill-metadata">
        <span>metadata.toml</span>
        <textarea
          id="skill-metadata"
          className="field-input text-area markdown-textarea"
          rows={8}
          value={draft.metadataToml}
          onChange={(event) => onChange({ ...draft, metadataToml: event.target.value })}
        />
      </label>

      <SkillFiles files={draft.files} />
      <SkillSupportingFiles
        files={draft.files}
        onChange={(files) => onChange({ ...draft, files })}
      />
    </>
  );
}

function SkillFiles({ files }: { files: SkillFile[] }) {
  const { t } = useI18n();

  return (
    <section className="form-section">
      <div className="section-heading">
        <h4>{t("common.files")}</h4>
        <span className="muted-text">{files.length}</span>
      </div>

      {files.length === 0 ? (
        <div className="placeholder-strip">{t("assets.noFiles")}</div>
      ) : (
        <ul className="file-list">
          {files.map((file) => (
            <li key={`${file.kind}:${file.path}`}>
              <span>{file.kind}</span>
              <code>{file.path}</code>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function SkillSupportingFiles({
  files,
  onChange,
}: {
  files: SkillFile[];
  onChange: (files: SkillFile[]) => void;
}) {
  const { t } = useI18n();
  const editableFiles = files.filter((file) => file.kind === "example" || file.kind === "resource");

  function updateFile(index: number, file: SkillFile) {
    const editableIndex = editableFiles[index];
    onChange(files.map((current) => (current === editableIndex ? file : current)));
  }

  function removeFile(index: number) {
    const editableFile = editableFiles[index];
    onChange(files.filter((file) => file !== editableFile));
  }

  return (
    <section className="form-section">
      <div className="section-heading">
        <h4>{t("assets.supportingFiles")}</h4>
        <span className="muted-text">{editableFiles.length}</span>
      </div>
      {editableFiles.length === 0 ? <div className="placeholder-strip">{t("assets.noFiles")}</div> : null}
      {editableFiles.map((file, index) => (
        <div className="supporting-file-row" key={`${file.kind}:${file.path}:${index}`}>
          <div className="form-grid">
            <label className="field compact-field" htmlFor={`skill-file-kind-${index}`}>
              <span>{t("common.type")}</span>
              <select
                id={`skill-file-kind-${index}`}
                className="field-input"
                value={file.kind}
                onChange={(event) =>
                  updateFile(index, {
                    ...file,
                    kind: event.target.value === "resource" ? "resource" : "example",
                  })
                }
              >
                <option value="example">{t("assets.exampleFile")}</option>
                <option value="resource">{t("assets.resourceFile")}</option>
              </select>
            </label>
            <label className="field compact-field" htmlFor={`skill-file-path-${index}`}>
              <span>{t("common.path")}</span>
              <input
                id={`skill-file-path-${index}`}
                className="field-input"
                type="text"
                value={file.path}
                onChange={(event) => updateFile(index, { ...file, path: event.target.value })}
              />
            </label>
          </div>
          <MarkdownEditor
            id={`skill-file-content-${index}`}
            label={t("assets.fileContent")}
            rows={8}
            value={file.content ?? ""}
            onChange={(content) => updateFile(index, { ...file, content })}
          />
          <button className="secondary-action" type="button" onClick={() => removeFile(index)}>
            {t("common.remove")}
          </button>
        </div>
      ))}
      <div className="button-row">
        <button
          className="secondary-action"
          type="button"
          onClick={() =>
            onChange([
              ...files,
              {
                kind: "example",
                path: "examples/example.md",
                content: "",
              },
            ])
          }
        >
          {t("assets.addExampleFile")}
        </button>
        <button
          className="secondary-action"
          type="button"
          onClick={() =>
            onChange([
              ...files,
              {
                kind: "resource",
                path: "resources/resource.md",
                content: "",
              },
            ])
          }
        >
          {t("assets.addResourceFile")}
        </button>
      </div>
    </section>
  );
}

function PlaybookEditor({
  draft,
  onChange,
}: {
  draft: AssetEditorDraft;
  onChange: (draft: AssetEditorDraft) => void;
}) {
  const { t } = useI18n();

  return (
    <>
      <MarkdownEditor
        id="playbook-trigger"
        label={t("assets.playbookTrigger")}
        rows={6}
        value={draft.trigger}
        onChange={(trigger) => onChange({ ...draft, trigger })}
      />

      <section className="form-section">
        <div className="form-grid">
          <label className="field" htmlFor="playbook-side-effect">
            <span>{t("assets.sideEffectLevel")}</span>
            <select
              id="playbook-side-effect"
              className="field-input"
              value={draft.sideEffectLevel}
              onChange={(event) =>
                onChange({
                  ...draft,
                  sideEffectLevel: event.target.value as AssetEditorDraft["sideEffectLevel"],
                })
              }
            >
              <option value="none">{t("assets.sideEffectNone")}</option>
              <option value="read-only">{t("assets.sideEffectReadOnly")}</option>
              <option value="writes-files">{t("assets.sideEffectWritesFiles")}</option>
              <option value="runs-commands">{t("assets.sideEffectRunsCommands")}</option>
              <option value="external-side-effects">{t("assets.sideEffectExternal")}</option>
            </select>
          </label>
          <label className="field" htmlFor="playbook-invocation">
            <span>{t("assets.recommendedInvocation")}</span>
            <select
              id="playbook-invocation"
              className="field-input"
              value={draft.recommendedInvocation}
              onChange={(event) =>
                onChange({
                  ...draft,
                  recommendedInvocation: event.target.value as AssetEditorDraft["recommendedInvocation"],
                })
              }
            >
              <option value="manual">{t("assets.invocationManual")}</option>
              <option value="model">{t("assets.invocationModel")}</option>
              <option value="both">{t("assets.invocationBoth")}</option>
            </select>
          </label>
        </div>
      </section>

      <PlaybookInputs inputs={draft.inputs} onChange={(inputs) => onChange({ ...draft, inputs })} />
      <PlaybookSteps steps={draft.steps} onChange={(steps) => onChange({ ...draft, steps })} />

      <MarkdownEditor
        id="playbook-verification"
        label={t("assets.verification")}
        rows={6}
        value={draft.verification}
        onChange={(verification) => onChange({ ...draft, verification })}
      />
      <MarkdownEditor
        id="playbook-failure-handling"
        label={t("assets.failureHandling")}
        rows={6}
        value={draft.failureHandling}
        onChange={(failureHandling) => onChange({ ...draft, failureHandling })}
      />
      <TagInput
        id="playbook-target-compatibility"
        label={t("assets.targetCompatibility")}
        value={draft.targetCompatibility}
        onChange={(targetCompatibility) => onChange({ ...draft, targetCompatibility })}
      />
    </>
  );
}

function PlaybookInputs({
  inputs,
  onChange,
}: {
  inputs: PlaybookInput[];
  onChange: (inputs: PlaybookInput[]) => void;
}) {
  const { t } = useI18n();

  function updateInput(index: number, input: PlaybookInput) {
    onChange(inputs.map((current, currentIndex) => (currentIndex === index ? input : current)));
  }

  function removeInput(index: number) {
    onChange(inputs.filter((_, currentIndex) => currentIndex !== index));
  }

  return (
    <section className="form-section">
      <div className="section-heading">
        <h4>{t("assets.inputs")}</h4>
        <span className="muted-text">{inputs.length}</span>
      </div>
      {inputs.length === 0 ? <div className="placeholder-strip">{t("common.none")}</div> : null}
      {inputs.map((input, index) => (
        <div className="playbook-input-row" key={index}>
          <label className="field compact-field" htmlFor={`playbook-input-name-${index}`}>
            <span>{t("common.name")}</span>
            <input
              id={`playbook-input-name-${index}`}
              className="field-input"
              type="text"
              value={input.name}
              onChange={(event) => updateInput(index, { ...input, name: event.target.value })}
            />
          </label>
          <label className="field compact-field" htmlFor={`playbook-input-description-${index}`}>
            <span>{t("common.description")}</span>
            <input
              id={`playbook-input-description-${index}`}
              className="field-input"
              type="text"
              value={input.description ?? ""}
              onChange={(event) => updateInput(index, { ...input, description: event.target.value })}
            />
          </label>
          <label className="toggle-field">
            <input
              type="checkbox"
              checked={input.required}
              onChange={(event) => updateInput(index, { ...input, required: event.target.checked })}
            />
            <span>{t("assets.required")}</span>
          </label>
          <button className="secondary-action" type="button" onClick={() => removeInput(index)}>
            {t("common.remove")}
          </button>
        </div>
      ))}
      <button
        className="secondary-action"
        type="button"
        onClick={() => onChange([...inputs, { name: "", description: null, required: false }])}
      >
        {t("assets.addInput")}
      </button>
    </section>
  );
}

function PlaybookSteps({
  steps,
  onChange,
}: {
  steps: PlaybookStep[];
  onChange: (steps: PlaybookStep[]) => void;
}) {
  const { t } = useI18n();

  function updateStep(index: number, step: PlaybookStep) {
    onChange(steps.map((current, currentIndex) => (currentIndex === index ? step : current)));
  }

  function removeStep(index: number) {
    onChange(steps.filter((_, currentIndex) => currentIndex !== index));
  }

  return (
    <section className="form-section">
      <div className="section-heading">
        <h4>{t("assets.steps")}</h4>
        <span className="muted-text">{steps.length}</span>
      </div>
      {steps.map((step, index) => (
        <div className="playbook-step-row" key={index}>
          <label className="field compact-field" htmlFor={`playbook-step-title-${index}`}>
            <span>{t("assets.stepTitle")}</span>
            <input
              id={`playbook-step-title-${index}`}
              className="field-input"
              type="text"
              value={step.title}
              onChange={(event) => updateStep(index, { ...step, title: event.target.value })}
            />
          </label>
          <MarkdownEditor
            id={`playbook-step-body-${index}`}
            label={t("assets.stepBody")}
            rows={6}
            value={step.body}
            onChange={(body) => updateStep(index, { ...step, body })}
          />
          <button className="secondary-action" type="button" onClick={() => removeStep(index)}>
            {t("common.remove")}
          </button>
        </div>
      ))}
      <button
        className="secondary-action"
        type="button"
        onClick={() => onChange([...steps, { title: "", body: "" }])}
      >
        {t("assets.addStep")}
      </button>
    </section>
  );
}

function RuleEditor({
  draft,
  onChange,
}: {
  draft: AssetEditorDraft;
  onChange: (draft: AssetEditorDraft) => void;
}) {
  const { t } = useI18n();

  return (
    <>
      <MarkdownEditor
        id="rule-body"
        label={t("assets.ruleBody")}
        rows={14}
        value={draft.body}
        onChange={(body) => onChange({ ...draft, body })}
      />
      <TagInput
        id="rule-path-globs"
        label={t("assets.pathGlobs")}
        value={draft.pathGlobs}
        onChange={(pathGlobs) => onChange({ ...draft, pathGlobs })}
      />
      {draft.assetType === "command-rule" ? (
        <section className="form-section">
          <div className="form-grid">
            <TagInput
              id="command-rule-prefix"
              label={t("assets.commandPrefix")}
              value={draft.commandPrefix}
              onChange={(commandPrefix) => onChange({ ...draft, commandPrefix })}
            />
            <label className="field" htmlFor="command-rule-decision">
              <span>{t("assets.commandDecision")}</span>
              <select
                id="command-rule-decision"
                className="field-input"
                value={draft.commandDecision}
                onChange={(event) =>
                  onChange({
                    ...draft,
                    commandDecision: event.target.value as AssetEditorDraft["commandDecision"],
                  })
                }
              >
                <option value="prompt">{t("assets.decisionPrompt")}</option>
                <option value="allow">{t("assets.decisionAllow")}</option>
                <option value="forbid">{t("assets.decisionForbid")}</option>
              </select>
            </label>
          </div>
        </section>
      ) : null}
      <TagInput
        id="rule-target-compatibility"
        label={t("assets.targetCompatibility")}
        value={draft.targetCompatibility}
        onChange={(targetCompatibility) => onChange({ ...draft, targetCompatibility })}
      />
      <div className="validation-panel valid">
        <p>{t("assets.targetCompatibilityHint")}</p>
      </div>
    </>
  );
}

function initialDraft(
  mode: AssetEditorMode,
  assetType: EditableAssetType,
  assetDetail: EditableAssetDetail | null,
  initialDraftOverride: AssetEditorDraft | null,
): AssetEditorDraft {
  if (mode === "edit" && assetDetail) {
    return draftFromAssetDetail(assetDetail);
  }

  if (initialDraftOverride) {
    return initialDraftOverride;
  }

  return buildEmptyAssetDraft(assetType);
}

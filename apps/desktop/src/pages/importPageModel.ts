import type { SyncScope } from "../api/sync";
import type { RemoteImportCandidate } from "../api/import";

export type ImportSourceMode = "local-tool" | "public-github";
export type RemoteCandidateDecision = "import" | "skip";

export function importProjectPathRequired(scope: SyncScope): boolean {
  return scope === "project";
}

export function importProjectPathRequiredForSource(source: ImportSourceMode, scope: SyncScope): boolean {
  return source === "local-tool" && importProjectPathRequired(scope);
}

export function projectPathForImport(projectPath: string, scope: SyncScope): string {
  if (importProjectPathRequired(scope)) {
    return projectPath.trim();
  }
  return projectPath.trim() || ".";
}

export function isValidPublicGithubUrl(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed.startsWith("https://github.com/")) {
    return false;
  }
  const withoutQuery = trimmed.slice("https://github.com/".length).split("?")[0];
  const segments = withoutQuery.split("/").filter(Boolean);
  return segments.length >= 2 && segments[0].length > 0 && segments[1].length > 0;
}

export function normalizeRemoteDestinationId(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^[-_]+|[-_]+$/g, "");
}

export function defaultGithubCandidateDecision(candidate: RemoteImportCandidate): RemoteCandidateDecision {
  return "skip";
}

export function defaultGithubDestinationIds(candidates: RemoteImportCandidate[]): Record<string, string> {
  return Object.fromEntries(
    candidates.map((candidate) => [
      candidate.candidateId,
      normalizeRemoteDestinationId(candidate.defaultDestinationId || candidate.id),
    ]),
  );
}

export function defaultGithubDecisions(candidates: RemoteImportCandidate[]): Record<string, RemoteCandidateDecision> {
  return Object.fromEntries(
    candidates.map((candidate) => [candidate.candidateId, defaultGithubCandidateDecision(candidate)]),
  );
}
